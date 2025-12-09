use std::collections::HashMap;

#[derive(Debug)]
pub struct QuizState {
    pub current_slide: u32,
    pub show_results: bool,
    pub language: HashMap<String, u32>,
    pub formality: HashMap<String, u32>,
    pub exercises: HashMap<String, u32>,
}

impl QuizState {
    pub fn new() -> Self {
        Self {
            current_slide: 0,
            show_results: false,
            language: HashMap::new(),
            formality: HashMap::new(),
            exercises: HashMap::new(),
        }
    }
}
