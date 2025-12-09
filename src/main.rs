use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tracing_subscriber::EnvFilter;

mod states;
use crate::states::app_state::AppState;
use crate::states::quiz_state::QuizState;

mod enums;
use crate::enums::server_event::ServerEvent;

mod app;
use crate::app::build_app;

mod client_message;
mod ws;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let quiz_state = QuizState::new();
    let (tx, _rx) = broadcast::channel::<ServerEvent>(32);

    let state = AppState {
        shared: Arc::new(Mutex::new(quiz_state)),
        tx,
    };

    let app = build_app(state);

    let addr: SocketAddr = std::env::var("ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()
        .expect("Invalid ADDR");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
