use crate::enums::server_event::ServerEvent;
use crate::states::quiz_state::QuizState;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub shared: Arc<Mutex<QuizState>>,
    pub tx: broadcast::Sender<ServerEvent>,
}
