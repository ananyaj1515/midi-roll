# MIDI Waterfall
 
A terminal-based MIDI visualizer written in Rust. It parses MIDI files into structured note events, then plays them back as a Synthesia-style falling-note waterfall with a live piano keyboard — all rendered in your terminal.
 
No audio output, no synth setup required. Just point it at a `.mid` file and watch the notes fall.

<img width="800" height="343" alt="ScreenRecording2026-06-27at1 43 23PM-ezgif com-video-to-gif-converter" src="https://github.com/user-attachments/assets/837c4284-d266-41a1-917f-ee81f8a887a6" />
 
## Features
 
- Parses standard MIDI files into timed note events (pitch, velocity, duration, channel)
- Synthesia-style waterfall — notes fall toward a piano keyboard that lights up in real time
- Adjustable playback tempo
- Pause / resume
- Pure terminal UI, runs anywhere — no audio drivers or external synth dependencies
## Built with
 
- [`midly`](https://crates.io/crates/midly) — MIDI file parsing
- [`ratatui`](https://crates.io/crates/ratatui) — terminal UI rendering
- [`crossterm`](https://crates.io/crates/crossterm) — terminal input/raw mode handling
## Usage
 
```bash
cargo run --release
```
 
By default it loads `asset/mario.mid`. Drop in any standard `.mid` file and update the path in `main.rs` to try your own.
 
### Controls
 
| Key       | Action          |
|-----------|-----------------|
| `space`   | Pause / resume  |
| `+`       | Speed up        |
| `-`       | Slow down       |
| `q`       | Quit            |
 
## How it works
 
```
MIDI file → parser (midi.rs) → Vec<Note>
                                    │
                                    ▼
                         timing engine (timing.rs)
                         walks notes in real time,
                         updates shared AppState
                                    │
                                    ▼
                          UI thread (ui.rs)
                       reads AppState, redraws
                       waterfall + keyboard at ~30fps
```
 
The timing engine and the UI run on separate threads, coordinating through a shared `Arc<Mutex<AppState>>`. The timing thread walks through the parsed notes, sleeping between events based on the MIDI file's tempo, and flips notes on/off in the shared state. The UI thread polls that state roughly 30 times a second and redraws the waterfall and keyboard accordingly.
 
## Project structure
 
```
src/
  main.rs     — entry point, spawns threads, wires everything together
  midi.rs     — MIDI file parsing into Note structs
  state.rs    — shared AppState struct
  timing.rs   — playback timing engine (runs on its own thread)
  ui.rs       — ratatui rendering + input handling
```
 
## Status
 
Personal learning project: built to get hands-on with Rust ownership, threading, and terminal UI programming. Rough edges expected. Audio features to be supported in the future (WIP)

 
