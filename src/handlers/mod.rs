use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::models::{AppStore, CreateStudentRequest};

pub async fn create_student_handler(
    State(app_store): State<AppStore>,
    Json(req): Json<CreateStudentRequest>,
) -> impl IntoResponse {
    match app_store.create_student(req).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json("Student created successfully".to_string()),
        ),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())),
    }
}

pub async fn get_all_students_handler(State(app_store): State<AppStore>) -> impl IntoResponse {
    match app_store.get_all_students().await {
        Ok(students) => (StatusCode::OK, Json(students)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}

pub async fn get_student_handler(
    State(app_store): State<AppStore>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match app_store.get_student(id).await {
        Ok(student) => (StatusCode::OK, Json(student)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}

pub async fn delete_student_handler(
    State(app_store): State<AppStore>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match app_store.delete_student(id).await {
        Ok(_) => (StatusCode::OK, Json("Student deleted!")).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}
