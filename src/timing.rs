// src/timing.rs
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use crate::state::AppState;
use crate::midi::Note;

pub fn run(
    notes: Vec<Note>,
    ticks_per_beat: u16,
    microseconds_per_beat: u32,  // tempo — default 500000 = 120bpm
    shared_state: Arc<Mutex<AppState>>,
) {
    // spawn a new thread — the closure is what runs on it
    // `move` means the closure TAKES OWNERSHIP of everything it uses
    thread::spawn(move || {
        let mut note_index = 0;  // which note we're about to play next
        let start_time = Instant::now();  // wall clock when playback began

        loop {
            // check if we're done or quitting
            {
                let state = shared_state.lock().unwrap();
                if state.quit || note_index >= notes.len() {
                    break;
                }
            } // lock drops here — important to drop before sleeping

            // check if paused
            {
                let state = shared_state.lock().unwrap();
                if state.paused {
                    drop(state); // explicitly drop so we don't hold lock while sleeping
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }
            }

            let next_note = &notes[note_index];

            // convert the note's start_tick to real microseconds
            // formula: (ticks / ticks_per_beat) * microseconds_per_beat
            let note_time_us = (next_note.start_tick as u64
                * microseconds_per_beat as u64)
                / ticks_per_beat as u64;

            // how long since playback started (in microseconds)
            let elapsed_us = start_time.elapsed().as_micros() as u64;

            // if it's not time for this note yet, sleep a bit and check again
            if elapsed_us < note_time_us {
                let wait = note_time_us - elapsed_us;
                // sleep in small chunks so we stay responsive to pause/quit
                thread::sleep(Duration::from_micros(wait.min(10_000)));
                continue;
            }

            // it's time — fire the note
            {
                let mut state = shared_state.lock().unwrap();
                state.active_notes.insert(next_note.pitch);
                state.current_tick = next_note.start_tick;
            }

            // schedule note OFF — spawn a tiny thread just for this
            // so we don't block the main timing loop
            let shared_state_clone = Arc::clone(&shared_state);
            let pitch = next_note.pitch;
            let duration_us = (next_note.duration as u64
                * microseconds_per_beat as u64)
                / ticks_per_beat as u64;

            thread::spawn(move || {
                thread::sleep(Duration::from_micros(duration_us));
                let mut state = shared_state_clone.lock().unwrap();
                state.active_notes.remove(&pitch);
            });

            note_index += 1;
        }
    });
}