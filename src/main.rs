mod config;
mod errors;
mod handlers;
mod models;

use std::net::{Ipv6Addr, SocketAddr};

use axum::{
    Router,
    routing::{get, post},
};

use handlers::{
    create_student_handler, delete_student_handler, get_all_students_handler, get_student_handler,
};

use models::AppStore;
use tokio::net::TcpListener;

use crate::config::get_env_vars;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let fallback_port = 8080;
    let port: u16 = get_env_vars::<u16>("PORT".to_string()).unwrap_or(fallback_port);

    let listening_address: SocketAddr = SocketAddr::from((Ipv6Addr::LOCALHOST, port));
    let store = AppStore::new();
    let app = Router::new()
        .route("/", get(|| async { "Hello from Axum! 🦀" }))
        .route(
            "/student",
            post(create_student_handler).get(get_all_students_handler),
        )
        .route(
            "/student/{id}",
            get(get_student_handler).delete(delete_student_handler),
        )
        .with_state(store);

    let binder: TcpListener = TcpListener::bind(listening_address)
        .await
        .expect("Failed to bind address");
    println!("Listening on http://127.0.0.1:3000");
    axum::serve(binder, app).await.unwrap();
}

// HTTP VERBS

/****
 * POST
 * GET
 * DELETE
 * PATCH
 * PUT
 *
 *
 */
