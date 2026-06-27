// src/ui.rs
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::io;
use crate::state::AppState;
use crate::midi::Note;

// how many piano keys to show — we show one octave range around middle C
// MIDI note 21 = A0 (lowest piano key), 108 = C8 (highest)
const PIANO_START: u8 = 36; // C2
const PIANO_END: u8 = 96;   // C7

pub fn run(
    notes: &[Note],
    shared_state: Arc<Mutex<AppState>>,
) -> io::Result<()> {
    // set up terminal — raw mode means we get keypresses immediately
    // without waiting for enter, and the terminal doesn't echo them
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, notes, shared_state);

    // always restore terminal even if we crashed
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    notes: &[Note],
    shared_state: Arc<Mutex<AppState>>,
) -> io::Result<()> {
    loop {
        // --- draw ---
        terminal.draw(|frame| {
            let state = shared_state.lock().unwrap();

            // split screen into three sections vertically
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(10),     // waterfall — takes remaining space
                    Constraint::Length(4),   // piano keyboard
                    Constraint::Length(1),   // status bar
                ])
                .split(frame.size());

            draw_waterfall(frame, chunks[0], notes, &state);
            draw_keyboard(frame, chunks[1], &state);
            draw_statusbar(frame, chunks[2], &state);
        })?;

        // --- input --- poll with 33ms timeout = ~30fps
        if event::poll(Duration::from_millis(33))? {
            if let Event::Key(key) = event::read()? {
                let mut state = shared_state.lock().unwrap();
                match key.code {
                    KeyCode::Char('q') => {
                        state.quit = true;
                        break;
                    }
                    KeyCode::Char(' ') => {
                        state.paused = !state.paused;
                    }
                    KeyCode::Char('+') => {
                        state.tempo_multiplier = (state.tempo_multiplier + 0.1).min(2.0);
                    }
                    KeyCode::Char('-') => {
                        state.tempo_multiplier = (state.tempo_multiplier - 0.1).max(0.1);
                    }
                    _ => {}
                }
            }
        }

        // check if timing engine signalled quit
        {
            let state = shared_state.lock().unwrap();
            if state.quit {
                break;
            }
        }
    }

    Ok(())
}

fn draw_waterfall(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    notes: &[Note],
    state: &AppState,
) {
    let width = area.width as u8;
    let height = area.height as u16;
    let key_range = PIANO_END - PIANO_START;

    // build the waterfall as a vec of Lines (rows)
    // each row represents a moment in time
    // current time is at the bottom, past is above
    let ticks_per_row = 120u32; // how many ticks each row of the waterfall represents

    let mut rows: Vec<Line> = Vec::new();

    for row in 0..height {
        // row 0 = top (furthest in future), row height-1 = bottom (now)
        let row_tick_offset = (height - 1 - row) as u32 * ticks_per_row;
        let row_start_tick = state.current_tick.saturating_add(row_tick_offset / 4);
        let row_end_tick = row_start_tick + ticks_per_row;

        let mut spans: Vec<Span> = Vec::new();

        for pitch in PIANO_START..PIANO_END {
            // is any note active during this row's time window?
            let active = notes.iter().any(|n| {
                n.pitch == pitch
                    && n.start_tick < row_end_tick
                    && (n.start_tick + n.duration) > row_start_tick
            });

            let col_width = (area.width as usize) / (key_range as usize);
            let col_width = col_width.max(1);

            let ch = " ".repeat(col_width);

            if active {
                // color by pitch — cycle through some nice colors
                let color = pitch_to_color(pitch);
                spans.push(Span::styled(ch, Style::default().bg(color)));
            } else {
                spans.push(Span::raw(ch));
            }
        }

        rows.push(Line::from(spans));
    }

    let waterfall = Paragraph::new(rows)
        .block(Block::default().borders(Borders::ALL).title(" synthesia "));

    frame.render_widget(waterfall, area);
}

fn draw_keyboard(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &AppState,
) {
    let key_range = (PIANO_END - PIANO_START) as usize;
    let col_width = (area.width as usize).saturating_sub(2) / key_range;
    let col_width = col_width.max(1);

    // build two rows: top row shows black keys, bottom shows white keys
    let mut top_row: Vec<Span> = Vec::new();
    let mut bot_row: Vec<Span> = Vec::new();

    for pitch in PIANO_START..PIANO_END {
        let active = state.active_notes.contains(&pitch);
        let is_black = is_black_key(pitch);

        let ch = " ".repeat(col_width);

        let top_style = if is_black {
            if active {
                Style::default().bg(Color::Cyan)
            } else {
                Style::default().bg(Color::Black)
            }
        } else {
            Style::default() // white keys don't show in top row
        };

        let bot_style = if active {
            Style::default().bg(Color::Cyan)
        } else if is_black {
            Style::default().bg(Color::Black)
        } else {
            Style::default().bg(Color::White)
        };

        top_row.push(Span::styled(ch.clone(), top_style));
        bot_row.push(Span::styled(ch, bot_style));
    }

    let keyboard = Paragraph::new(vec![
        Line::from(top_row),
        Line::from(bot_row),
    ])
    .block(Block::default().borders(Borders::ALL).title(" keyboard "));

    frame.render_widget(keyboard, area);
}

fn draw_statusbar(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &AppState,
) {
    let progress = if state.total_ticks > 0 {
        (state.current_tick as f32 / state.total_ticks as f32 * 100.0) as u32
    } else {
        0
    };

    let paused = if state.paused { "paused" } else { "playing" };

    let text = format!(
        " [space] {paused}   [+/-] tempo: {:.1}x   [q] quit   progress: {progress}%",
        state.tempo_multiplier
    );

    let bar = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(bar, area);
}

// maps a pitch to a color — cycles through a pleasing palette
fn pitch_to_color(pitch: u8) -> Color {
    match pitch % 12 {
        0  => Color::Red,
        1  => Color::LightRed,
        2  => Color::Yellow,
        3  => Color::LightYellow,
        4  => Color::Green,
        5  => Color::LightGreen,
        6  => Color::Cyan,
        7  => Color::LightCyan,
        8  => Color::Blue,
        9  => Color::LightBlue,
        10 => Color::Magenta,
        _  => Color::LightMagenta,
    }
}

// black keys are the sharps/flats — pattern repeats every octave
fn is_black_key(pitch: u8) -> bool {
    matches!(pitch % 12, 1 | 3 | 6 | 8 | 10)
}