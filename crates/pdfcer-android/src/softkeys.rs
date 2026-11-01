//! GameActivity's IME text, which winit 0.30 drops, handed to
//! `pdfcer_gui::softkeys`.
//!
//! The IME edits a text buffer GameActivity keeps. Each frame the buffer is
//! read; once nothing in it is still being composed, its text is taken and the
//! buffer emptied, so the next keystroke starts from nothing and Backspace on
//! the empty buffer comes as a key event winit does deliver. Text being
//! composed (a word with suggestions showing) waits until the keyboard commits
//! it; suggestions are switched off at install to keep that rare.

use winit::platform::android::activity::AndroidApp;
use winit::platform::android::activity::input::{
    ImeOptions, InputType, TextInputAction, TextInputState, TextSpan,
};

/// The activity's IME buffer as a `pdfcer_gui::softkeys::SoftKeys` source.
pub struct GameKeys {
    app: AndroidApp,
}

impl GameKeys {
    /// A source over `app`'s IME buffer; asks the keyboard for plain
    /// multi-line text without suggestions. Multi-line makes its Enter key
    /// type a line break into the buffer, which `pdfcer_gui::softkeys` turns
    /// into an Enter press; on a single-line field the keyboard sends Enter
    /// as an editor action, which GameActivity does not forward.
    pub fn new(app: AndroidApp) -> Self {
        app.set_ime_editor_info(
            InputType::TYPE_CLASS_TEXT
                | InputType::TYPE_TEXT_FLAG_NO_SUGGESTIONS
                | InputType::TYPE_TEXT_FLAG_MULTI_LINE,
            TextInputAction::Unspecified,
            ImeOptions::IME_FLAG_NO_ENTER_ACTION | ImeOptions::IME_FLAG_NO_FULLSCREEN,
        );
        Self { app }
    }
}

impl pdfcer_gui::softkeys::SoftKeys for GameKeys {
    fn take_committed(&self) -> String {
        let state = self.app.text_input_state();
        if state.text.is_empty() || state.compose_region.is_some() {
            return String::new();
        }
        self.app.set_text_input_state(TextInputState {
            text: String::new(),
            selection: TextSpan { start: 0, end: 0 },
            compose_region: None,
        });
        state.text
    }
}
