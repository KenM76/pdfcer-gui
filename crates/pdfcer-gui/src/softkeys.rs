//! Text typed on a platform's on-screen keyboard that the window system does
//! not deliver as events.
//!
//! On Android, winit 0.30 drops GameActivity's IME text events, so a soft
//! keyboard opens over a focused text box and nothing it types arrives. The
//! entry point installs a [`SoftKeys`] source; [`apply`] turns what it typed
//! into `egui::Event::Text`, with each line break as an Enter press, before
//! the frame runs. Keys the keyboard sends as key events (Backspace, Enter on
//! most layouts) still arrive through winit. On the desktop nothing is
//! installed and [`apply`] does nothing.

use std::sync::OnceLock;

/// What the platform supplies: the text committed since the last call.
pub trait SoftKeys: Send + Sync {
    /// The committed text, taken: a second call returns only what was typed
    /// after the first. Empty when nothing was typed.
    fn take_committed(&self) -> String;
}

static SOURCE: OnceLock<Box<dyn SoftKeys>> = OnceLock::new();

/// Installs the soft keyboard source. The first call wins.
pub fn install(source: Box<dyn SoftKeys>) {
    let _ = SOURCE.set(source);
}

/// Appends what the installed source typed to `raw`'s events.
pub fn apply(raw: &mut egui::RawInput) {
    let Some(source) = SOURCE.get() else {
        return;
    };
    events_for(&source.take_committed(), raw.modifiers, &mut raw.events);
}

/// `text` as egui events: each line as `Text`, each line break as Enter.
fn events_for(text: &str, modifiers: egui::Modifiers, events: &mut Vec<egui::Event>) {
    let mut lines = text.split('\n').peekable();
    while let Some(line) = lines.next() {
        let line = line.trim_end_matches('\r');
        if !line.is_empty() {
            events.push(egui::Event::Text(line.to_owned()));
        }
        if lines.peek().is_some() {
            for pressed in [true, false] {
                events.push(egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn events(text: &str) -> Vec<egui::Event> {
        let mut out = Vec::new();
        events_for(text, egui::Modifiers::NONE, &mut out);
        out
    }

    #[test]
    fn nothing_typed_is_no_event() {
        assert!(events("").is_empty());
    }

    #[test]
    fn a_line_is_one_text_event() {
        assert_eq!(events("57"), vec![egui::Event::Text("57".to_owned())]);
    }

    #[test]
    fn a_line_break_is_an_enter_press_and_release() {
        let got = events("7\n");
        assert_eq!(got.len(), 3);
        assert_eq!(got[0], egui::Event::Text("7".to_owned()));
        assert!(matches!(
            got[1],
            egui::Event::Key {
                key: egui::Key::Enter,
                pressed: true,
                ..
            }
        ));
        assert!(matches!(
            got[2],
            egui::Event::Key {
                key: egui::Key::Enter,
                pressed: false,
                ..
            }
        ));
    }
}
