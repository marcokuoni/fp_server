use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerEvent {
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
