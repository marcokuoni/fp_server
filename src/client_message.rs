use crate::enums::client_message::ClientMessage;
use crate::enums::server_event::ServerEvent;
use crate::states::app_state::AppState;
use crate::states::quiz_state::QuizState;
use std::collections::HashMap;

pub async fn handle_client_message(msg: ClientMessage, state: &AppState) {
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
