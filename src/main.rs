mod auth;
mod config;
mod errors;
mod handlers;
mod logger;
mod models;
mod routes;
mod services;

use std::net::{Ipv6Addr, SocketAddr};
use models::AppStore;
use tokio::net::TcpListener;
use config::get_env_vars;
use routes::create_router;
use logger::AppLogger;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    AppLogger::init();
    let port: u16 = get_env_vars::<u16>("PORT".to_string()).unwrap_or(8080);
    let listening_address = SocketAddr::from((Ipv6Addr::LOCALHOST, port));
    let store = AppStore::new();
    let app = create_router(store);
    let binder = TcpListener::bind(listening_address)
        .await
        .expect("Failed to bind address");
    AppLogger::info(&format!("Server listening at {}", listening_address));
    axum::serve(binder, app).await.unwrap();
}

