use axum::{
    extract::{
        State,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};

use crate::client_message::handle_client_message;
use crate::enums::client_message::ClientMessage;
use crate::states::app_state::AppState;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
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
            if let Message::Text(text) = msg
                && let Ok(parsed) = serde_json::from_str::<ClientMessage>(&text)
            {
                handle_client_message(parsed, &recv_state).await;
            }
        }
    });

    let _ = tokio::join!(send_task, recv_task);
}
