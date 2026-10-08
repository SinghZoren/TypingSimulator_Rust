#[cfg(target_os = "macos")]
use enigo::NewConError;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use rand::Rng;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
pub struct TypingSettings {
    pub wpm: u32,
    pub delay_seconds: f32,
    pub typo_chance: f32,
    pub timing_variation: f32,
    pub thinking_chance: f32,
    pub thinking_pause_seconds: f32,
}

pub enum TypingEvent {
    Countdown(u32),
    Typing,
    Thinking,
    Progress(usize),
    Finished,
    Stopped,
    Error(String),
}

pub fn spawn(
    text: String,
    settings: TypingSettings,
    active: Arc<AtomicBool>,
    events: Sender<TypingEvent>,
) {
    thread::spawn(move || {
        let start = Instant::now();
        let delay = Duration::from_secs_f32(settings.delay_seconds);
        let mut last_count = u32::MAX;
        while start.elapsed() < delay {
            if !active.load(Ordering::SeqCst) {
                let _ = events.send(TypingEvent::Stopped);
                return;
            }
            let remaining = (delay - start.elapsed()).as_secs_f32().ceil() as u32;
            if remaining != last_count {
                let _ = events.send(TypingEvent::Countdown(remaining));
                last_count = remaining;
            }
            thread::sleep(Duration::from_millis(20));
        }
        if !active.load(Ordering::SeqCst) {
            let _ = events.send(TypingEvent::Stopped);
            return;
        }

        // The system dialog cannot repair an identity mismatch and otherwise
        // repeats on every Start. Our UI reports the denied check instead.
        let enigo_settings = Settings {
            open_prompt_to_get_permissions: !cfg!(target_os = "macos"),
            ..Settings::default()
        };
        let mut enigo = match Enigo::new(&enigo_settings) {
            Ok(enigo) => enigo,
            #[cfg(target_os = "macos")]
            Err(NewConError::NoPermission) => {
                fail(&active, &events, "macOS still reports this process as untrusted, even if Typing Simulator is enabled in Accessibility. This may be an app identity/signing problem; the app will not ask again in a loop.".into());
                return;
            }
            Err(error) => {
                fail(&active, &events, error.to_string());
                return;
            }
        };
        let _ = events.send(TypingEvent::Typing);
        let mut rng = rand::rng();

        let characters: Vec<char> = text.replace("\r\n", "\n").chars().collect();
        for (index, character) in characters.iter().copied().enumerate() {
            if !active.load(Ordering::SeqCst) {
                let _ = events.send(TypingEvent::Stopped);
                return;
            }
            if character.is_ascii_alphabetic() && rng.random::<f32>() < settings.typo_chance {
                let wrong = rng.random_range(b'a'..=b'z') as char;
                if let Err(error) = enigo.text(&wrong.to_string()) {
                    fail(&active, &events, error.to_string());
                    return;
                }
                if !wait(Duration::from_millis(80), &active) {
                    let _ = events.send(TypingEvent::Stopped);
                    return;
                }
                if let Err(error) = enigo.key(Key::Backspace, Direction::Click) {
                    fail(&active, &events, error.to_string());
                    return;
                }
                if !wait(Duration::from_millis(80), &active) {
                    let _ = events.send(TypingEvent::Stopped);
                    return;
                }
            }
            let result = match character {
                '\n' | '\r' => enigo.key(Key::Return, Direction::Click),
                '\t' => enigo.key(Key::Tab, Direction::Click),
                _ => enigo.text(&character.to_string()),
            };
            if let Err(error) = result {
                fail(&active, &events, error.to_string());
                return;
            }
            // Standard WPM counts five characters as a word.
            let base = 60.0 / (settings.wpm.max(1) as f32 * 5.0);
            let variation =
                rng.random_range(-settings.timing_variation..=settings.timing_variation);
            if !wait(
                Duration::from_secs_f32(base * (1.0 + variation).max(0.05)),
                &active,
            ) {
                let _ = events.send(TypingEvent::Stopped);
                return;
            }
            let _ = events.send(TypingEvent::Progress(index + 1));
            if index + 1 < characters.len()
                && is_word_boundary(&characters, index)
                && rng.random::<f32>() < settings.thinking_chance
            {
                let _ = events.send(TypingEvent::Thinking);
                if !wait(
                    Duration::from_secs_f32(settings.thinking_pause_seconds),
                    &active,
                ) {
                    let _ = events.send(TypingEvent::Stopped);
                    return;
                }
                let _ = events.send(TypingEvent::Typing);
            }
        }
        active.store(false, Ordering::SeqCst);
        let _ = events.send(TypingEvent::Finished);
    });
}

fn is_word_boundary(characters: &[char], index: usize) -> bool {
    !characters[index].is_whitespace()
        && characters
            .get(index + 1)
            .is_none_or(|next| next.is_whitespace())
}

pub fn estimate_seconds(text: &str, settings: TypingSettings, completed_chars: usize) -> f32 {
    let characters: Vec<char> = text.replace("\r\n", "\n").chars().collect();
    let start = completed_chars.min(characters.len());
    let remaining = &characters[start..];
    let base = 60.0 / (settings.wpm.max(1) as f32 * 5.0);
    let typing = remaining.len() as f32 * base;
    let typos = remaining
        .iter()
        .filter(|character| character.is_ascii_alphabetic())
        .count() as f32
        * settings.typo_chance
        * 0.16;
    let pauses = (start..characters.len().saturating_sub(1))
        .filter(|&index| is_word_boundary(&characters, index))
        .count() as f32
        * settings.thinking_chance
        * settings.thinking_pause_seconds;
    typing + typos + pauses
}

fn wait(duration: Duration, active: &AtomicBool) -> bool {
    let start = Instant::now();
    while start.elapsed() < duration {
        if !active.load(Ordering::SeqCst) {
            return false;
        }
        thread::sleep((duration - start.elapsed()).min(Duration::from_millis(10)));
    }
    active.load(Ordering::SeqCst)
}

fn fail(active: &AtomicBool, events: &Sender<TypingEvent>, error: String) {
    active.store(false, Ordering::SeqCst);
    let _ = events.send(TypingEvent::Error(format!(
        "Keyboard input failed: {error}"
    )));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thinking_pauses_only_follow_complete_words() {
        let chars: Vec<_> = "hello world!".chars().collect();
        let boundaries: Vec<_> = (0..chars.len())
            .filter(|&index| is_word_boundary(&chars, index))
            .collect();
        assert_eq!(boundaries, vec![4, 11]);
    }

    #[test]
    fn estimate_includes_expected_word_pause() {
        let settings = TypingSettings {
            wpm: 60,
            delay_seconds: 0.0,
            typo_chance: 0.0,
            timing_variation: 0.0,
            thinking_chance: 1.0,
            thinking_pause_seconds: 2.0,
        };
        // Eleven characters at 200 ms each, plus one pause between the words.
        assert!((estimate_seconds("hello world", settings, 0) - 4.2).abs() < 0.001);
    }
}
