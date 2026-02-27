use axum::{
    Json,
    body::Bytes,
    extract::{Extension, Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use hmac::{Hmac, Mac};
use sha2::Sha512;
use uuid::Uuid;

use crate::{
    auth::middleware::AuthSchool,
    config::get_env_vars,
    models::{AppStore, CreateStudentRequest, LoginSchoolRequest, RegisterSchoolRequest},
    services::initialize_paystack_transaction,
};

// -- Auth handlers --

pub async fn register_handler(
    State(store): State<AppStore>,
    Json(req): Json<RegisterSchoolRequest>,
) -> impl IntoResponse {
    match store.register_school(req).await {
        Ok(school) => (StatusCode::CREATED, Json(serde_json::json!({
            "message": "School registered successfully",
            "id": school.id,
            "username": school.username,
        }))).into_response(),
        Err(e) => (StatusCode::CONFLICT, Json(e.to_string())).into_response(),
    }
}

pub async fn login_handler(
    State(store): State<AppStore>,
    Json(req): Json<LoginSchoolRequest>,
) -> impl IntoResponse {
    let secret: String = match get_env_vars("JWT_SECRET".to_string()) {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    };

    // Find the school by username
    let school = match store.find_school_by_username(&req.username).await {
        Ok(s) => s,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json("Invalid username or password".to_string()),
            )
                .into_response()
        }
    };

    // Verify the password against the stored hash
    let valid = bcrypt::verify(&req.password, &school.password_hash)
        .unwrap_or(false);

    if !valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json("Invalid username or password".to_string()),
        )
            .into_response();
    }

    // Generate and return the JWT
    match crate::auth::create_jwt(school.id, &school.username, &secret) {
        Ok(token) => (StatusCode::OK, Json(serde_json::json!({
            "token": token,
            "school_id": school.id,
            "username": school.username,
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}

// -- Student handlers (all scoped to the logged in school) --

pub async fn create_student_handler(
    State(store): State<AppStore>,
    Extension(auth): Extension<AuthSchool>,
    Json(req): Json<CreateStudentRequest>,
) -> impl IntoResponse {
    match store.create_student(auth.school_id, auth.username, req).await {
        Ok(()) => (StatusCode::CREATED, Json("Student created successfully")).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}

pub async fn get_all_students_handler(
    State(store): State<AppStore>,
    Extension(auth): Extension<AuthSchool>,
) -> impl IntoResponse {
    match store.get_all_students(auth.school_id).await {
        Ok(students) => (StatusCode::OK, Json(students)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}

pub async fn get_student_handler(
    State(store): State<AppStore>,
    Extension(auth): Extension<AuthSchool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match store.get_student(auth.school_id, id).await {
        Ok(student) => (StatusCode::OK, Json(student)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(e.to_string())).into_response(),
    }
}

pub async fn delete_student_handler(
    State(store): State<AppStore>,
    Extension(auth): Extension<AuthSchool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match store.delete_student(auth.school_id, id).await {
        Ok(_) => (StatusCode::OK, Json("Student deleted!")).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(e.to_string())).into_response(),
    }
}

pub async fn initiate_payment_handler(
    State(store): State<AppStore>,
    Extension(auth): Extension<AuthSchool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let secret_key: String = match get_env_vars("PAYSTACK_SECRET_KEY".to_string()) {
        Ok(k) => k,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    };

    let student = match store.get_student(auth.school_id, id).await {
        Ok(s) => s,
        Err(e) => return (StatusCode::NOT_FOUND, Json(e.to_string())).into_response(),
    };

    let reference = format!("sch-{}", Uuid::new_v4());
    let amount_kobo: u64 = 500_000;

    match initialize_paystack_transaction(&secret_key, &student.email, amount_kobo, &reference).await {
        Ok(data) => {
            let _ = store.set_payment_reference(auth.school_id, id, data.reference.clone()).await;
            (StatusCode::OK, Json(serde_json::json!({
                "authorization_url": data.authorization_url,
                "reference": data.reference,
            }))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}

pub async fn paystack_webhook_handler(
    State(store): State<AppStore>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let secret_key: String = match get_env_vars("PAYSTACK_SECRET_KEY".to_string()) {
        Ok(k) => k,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let signature = match headers
        .get("x-paystack-signature")
        .and_then(|v| v.to_str().ok())
    {
        Some(sig) => sig.to_string(),
        None => return StatusCode::UNAUTHORIZED,
    };

    let mut mac = Hmac::<Sha512>::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(&body);
    let expected = hex::encode(mac.finalize().into_bytes());

    if expected != signature {
        return StatusCode::UNAUTHORIZED;
    }

    let event: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    if event["event"] == "charge.success" {
        if let Some(reference) = event["data"]["reference"].as_str() {
            let _ = store.mark_student_paid_by_reference(reference).await;
        }
    }

    StatusCode::OK
}