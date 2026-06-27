// src/main.rs
mod midi;
mod state;
mod timing;
mod ui;

use state::AppState;
use std::sync::{Arc, Mutex};

fn main() {
    let bytes = std::fs::read("asset/under-the-sea.mid").unwrap();
    let (notes, ticks_per_beat) = midi::parse(&bytes);

    let total_ticks = notes
        .iter()
        .map(|n| n.start_tick + n.duration)
        .max()
        .unwrap_or(0);

    let shared_state = Arc::new(Mutex::new(AppState::new(total_ticks)));

    let state_for_timing = Arc::clone(&shared_state);
    let state_for_ui = Arc::clone(&shared_state);

    // timing runs on its own thread
    timing::run(notes.clone(), ticks_per_beat, 500_000, state_for_timing);

    // ui runs on main thread — blocks until user quits
    ui::run(&notes, state_for_ui).unwrap();
}