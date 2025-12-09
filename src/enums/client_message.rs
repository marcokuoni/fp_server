use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
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
