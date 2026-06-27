use std::collections::HashSet;

pub struct AppState {
    pub active_notes: HashSet<u8>,
    pub current_tick: u32,
    pub total_ticks: u32,
    pub tempo_multiplier: f32,
    pub paused: bool,
    pub quit: bool
}

impl AppState {
    pub fn new(total_ticks: u32) -> Self {
        Self {
            active_notes: HashSet::new(),
            current_tick: 0,
            total_ticks,
            tempo_multiplier: 1.0,
            paused: false,
            quit: false
        }
    }
}