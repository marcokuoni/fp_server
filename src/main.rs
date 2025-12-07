use axum::{
    Router,
    extract::{
        State,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tracing_subscriber::EnvFilter;

// === Shared state ===

#[derive(Debug)]
struct QuizState {
    current_slide: u32,
    show_results: bool,
    language: HashMap<String, u32>,
    formality: HashMap<String, u32>,
    exercises: HashMap<String, u32>,
}

impl QuizState {
    fn new() -> Self {
        Self {
            current_slide: 0,
            show_results: false,
            language: HashMap::new(),
            formality: HashMap::new(),
            exercises: HashMap::new(),
        }
    }
}

#[derive(Clone)]
struct AppState {
    shared: Arc<Mutex<QuizState>>,
    tx: broadcast::Sender<ServerEvent>,
}

// === Messages ===

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "join")]
    Join { role: String, session_id: String },

    #[serde(rename = "set_slide")]
    SetSlide { slide: u32 },

    #[serde(rename = "answer")]
    Answer {
        question_id: String,
        #[serde(default)]
        value: serde_json::Value, // string or array
    },

    #[serde(rename = "reveal_results")]
    RevealResults { show: bool },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
enum ServerEvent {
    #[serde(rename = "slide")]
    Slide { slide: u32 },

    #[serde(rename = "results")]
    Results {
        show: bool,
        language: HashMap<String, u32>,
        formality: HashMap<String, u32>,
        exercises: HashMap<String, u32>,
    },
}

// === main ===

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

    let app = Router::new()
        .route("/ws/demo", get(ws_handler))
        // you would also serve your Qwik dist/ here, e.g. at "/"
        .with_state(state);

    // run it with hyper
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(stream: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = stream.split();

    // subscribe to broadcast of server events
    let mut rx = state.tx.subscribe();

    // task: send server events to this client
    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let json = match serde_json::to_string(&event) {
                Ok(j) => j,
                Err(_) => continue,
            };
            if sender
                .send(Message::Text(Utf8Bytes::from(json)))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // task: receive client messages
    let recv_state = state.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(parsed) = serde_json::from_str::<ClientMessage>(&text) {
                    handle_client_message(parsed, &recv_state).await;
                }
            }
        }
    });

    let _ = tokio::join!(send_task, recv_task);
}

async fn handle_client_message(msg: ClientMessage, state: &AppState) {
    match msg {
        ClientMessage::Join { .. } => {
            // on join, immediately send current slide and results state
            let snapshot = {
                let q = state.shared.lock().unwrap();
                snapshot_results(&q)
            };

            let _ = state.tx.send(ServerEvent::Slide {
                slide: snapshot.current_slide,
            });

            let _ = state.tx.send(ServerEvent::Results {
                show: snapshot.show_results,
                language: snapshot.language,
                formality: snapshot.formality,
                exercises: snapshot.exercises,
            });
        }

        ClientMessage::SetSlide { slide } => {
            {
                let mut q = state.shared.lock().unwrap();
                q.current_slide = slide;
            }
            let _ = state.tx.send(ServerEvent::Slide { slide });
        }

        ClientMessage::Answer { question_id, value } => {
            let mut q = state.shared.lock().unwrap();

            match question_id.as_str() {
                "language" => {
                    if let Some(ans) = value.as_str() {
                        *q.language.entry(ans.to_string()).or_insert(0) += 1;
                    }
                }
                "formality" => {
                    if let Some(ans) = value.as_str() {
                        *q.formality.entry(ans.to_string()).or_insert(0) += 1;
                    }
                }
                "exercises" => {
                    if let Some(arr) = value.as_array() {
                        for v in arr {
                            if let Some(s) = v.as_str() {
                                *q.exercises.entry(s.to_string()).or_insert(0) += 1;
                            }
                        }
                    }
                }
                _ => {}
            }

            // if results are already shown, push updated aggregation
            if q.show_results {
                let snap = snapshot_results(&q);
                let _ = state.tx.send(ServerEvent::Results {
                    show: snap.show_results,
                    language: snap.language,
                    formality: snap.formality,
                    exercises: snap.exercises,
                });
            }
        }

        ClientMessage::RevealResults { show } => {
            let snap = {
                let mut q = state.shared.lock().unwrap();
                q.show_results = show;
                snapshot_results(&q)
            };
            let _ = state.tx.send(ServerEvent::Results {
                show: snap.show_results,
                language: snap.language,
                formality: snap.formality,
                exercises: snap.exercises,
            });
        }
    }
}

struct ResultsSnapshot {
    current_slide: u32,
    show_results: bool,
    language: HashMap<String, u32>,
    formality: HashMap<String, u32>,
    exercises: HashMap<String, u32>,
}

fn snapshot_results(q: &QuizState) -> ResultsSnapshot {
    ResultsSnapshot {
        current_slide: q.current_slide,
        show_results: q.show_results,
        language: q.language.clone(),
        formality: q.formality.clone(),
        exercises: q.exercises.clone(),
    }
}
