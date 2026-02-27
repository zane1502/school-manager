use axum::{Router, middleware, routing::{get, post}};

use crate::{
    auth::middleware::auth_middleware,
    handlers::{
        create_student_handler, delete_student_handler, get_all_students_handler,
        get_student_handler, initiate_payment_handler, login_handler,
        paystack_webhook_handler, register_handler,
    },
    models::AppStore,
};

pub fn create_router(store: AppStore) -> Router {
    // Public routes — no token needed
    let public_routes = Router::new()
        .route("/", get(|| async { "Hello from Axum! 🦀" }))
        .route("/auth/register", post(register_handler))
        .route("/auth/login", post(login_handler))
        .route("/webhook/paystack", post(paystack_webhook_handler));

    // Protected routes — token required
    let protected_routes = Router::new()
        .route("/students", post(create_student_handler).get(get_all_students_handler))
        .route("/students/{id}", get(get_student_handler).delete(delete_student_handler))
        .route("/students/{id}/pay", post(initiate_payment_handler))
        .layer(middleware::from_fn_with_state(store.clone(), auth_middleware));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(store)
}