use axum::{Router, routing::get};

use crate::AppState;
use crate::ws::ws_handler;

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .nest("/ws", Router::new().route("/quiz", get(ws_handler)))
        .with_state(state)
}
