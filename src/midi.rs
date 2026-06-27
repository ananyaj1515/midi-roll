// src/midi.rs
use std::collections::HashMap;
use midly::{MidiMessage, Smf, TrackEventKind};

#[derive(Debug, Clone)]
pub struct Note {
    pub pitch: u8,
    pub channel: u8,
    pub start_tick: u32,
    pub duration: u32,
    pub velocity: u8,
}

pub fn parse(bytes: &[u8]) -> (Vec<Note>, u16) {
    let smf = Smf::parse(bytes).unwrap();

    // ticks_per_beat tells us how to convert ticks → real time later
    let ticks_per_beat = match smf.header.timing {
        midly::Timing::Metrical(t) => t.as_int(),
        _ => 480, // sensible default
    };

    let mut active: HashMap<(u8, u8), (u32, u8)> = HashMap::new();
    let mut notes: Vec<Note> = Vec::new();

    for track in &smf.tracks {
        let mut current_tick: u32 = 0;

        for event in track {
            current_tick += event.delta.as_int();

            if let TrackEventKind::Midi { channel, message } = &event.kind {
                let ch = channel.as_int();

                match message {
                    MidiMessage::NoteOn { key, vel } if vel.as_int() > 0 => {
                        active.insert((ch, key.as_int()), (current_tick, vel.as_int()));
                    }
                    MidiMessage::NoteOn { key, .. } | MidiMessage::NoteOff { key, .. } => {
                        if let Some((start, vel)) = active.remove(&(ch, key.as_int())) {
                            notes.push(Note {
                                pitch: key.as_int(),
                                channel: ch,
                                start_tick: start,
                                duration: current_tick - start,
                                velocity: vel,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    notes.sort_by_key(|n| n.start_tick);
    (notes, ticks_per_beat)
}