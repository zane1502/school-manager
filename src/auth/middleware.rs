use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};

use crate::{auth::verify_jwt, config::get_env_vars, models::AppStore};

// This extension gets attached to the request so handlers can read the school_id
#[derive(Clone)]
pub struct AuthSchool {
    pub school_id: uuid::Uuid,
    pub username: String,
}

pub async fn auth_middleware(
    State(_store): State<AppStore>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<String>)> {
    let secret: String = get_env_vars("JWT_SECRET".to_string())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;

    // Extract the token from the Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(t) => t.to_string(),
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json("Missing authorization header".to_string()),
            ))
        }
    };

    // Verify the token and extract claims
    let claims = verify_jwt(&token, &secret)
        .map_err(|e| (StatusCode::UNAUTHORIZED, Json(e.to_string())))?;

    let school_id = claims.school_id.parse::<uuid::Uuid>().map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json("Invalid school id in token".to_string()),
        )
    })?;

    // Attach the school identity to the request for handlers to use
    req.extensions_mut().insert(AuthSchool {
        school_id,
        username: claims.username,
    });

    Ok(next.run(req).await)
}