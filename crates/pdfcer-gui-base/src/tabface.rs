//! # `tabface` — what a ribbon or dock tab shows, as a preference
//!
//! The value is the shell's [`TabLabel`]; this module gives it its file
//! tokens and the order the Settings window offers it in. The ribbon's and
//! the dock's are separate preferences (O299).

pub use egui_shell::TabLabel;

/// All three, in the order the Settings window offers them.
pub const ALL: [TabLabel; 3] = [TabLabel::Both, TabLabel::Glyph, TabLabel::Text];

/// The token a face is written under in the preferences file.
#[must_use]
pub const fn key(face: TabLabel) -> &'static str {
    match face {
        // ui-text-exempt: file tokens, never displayed.
        TabLabel::Both => "both",
        TabLabel::Glyph => "glyph",
        TabLabel::Text => "text",
    }
}

/// The face a file token names, or `None` for a token this build can't read.
#[must_use]
pub fn from_key(token: &str) -> Option<TabLabel> {
    ALL.into_iter().find(|f| key(*f) == token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_face_round_trips_through_its_token() {
        for face in ALL {
            assert_eq!(from_key(key(face)), Some(face));
        }
        assert_eq!(from_key("icons"), None);
    }

    #[test]
    fn tabs_show_glyph_and_text_until_told_otherwise() {
        assert_eq!(TabLabel::default(), TabLabel::Both);
    }
}
