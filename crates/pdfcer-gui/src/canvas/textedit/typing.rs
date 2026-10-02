//! What the next typed characters will carry (`OPERATOR_REQUESTS.md` O273).
//!
//! Bold, Italic, Underline or Strikethrough pressed inside a draft with no
//! range and no word under the caret does nothing to the page yet: it records
//! a pending style at the caret. At commit, the characters typed at that point
//! — the text between the original and the draft's common prefix and suffix —
//! are styled with it, as a range restyle after the text commit. A second
//! press of the same control takes it back off. The record dies with the
//! draft.

use pdfcer_core::text_edit::TextPosition;

use crate::app::actions::Action;
use crate::app::actions::text::{Decoration, TextAction};
use crate::app::actions::textstyle::StyleChange;

const MEMORY_KEY: &str = "pdfcer-textedit-typing-style"; // ui-text-exempt: internal memory id, never displayed

/// One pending style.
#[derive(Debug, Clone, PartialEq)]
pub enum Pending {
    /// A restyle.
    Style(StyleChange),
    /// A drawn line.
    Decorate(Decoration),
}

/// The pending styles and the caret they were set at.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Typing {
    /// The caret, in characters of the draft's text, when they were set.
    pub at: usize,
    /// Each pending style, keyed by the command that set it.
    pub pending: Vec<(&'static str, Pending)>,
}

impl Typing {
    /// Whether `command` has a pending style that still applies at `caret`.
    #[must_use]
    pub fn holds(&self, command: &str, caret: usize) -> bool {
        caret >= self.at && self.pending.iter().any(|(id, _)| *id == command)
    }
}

/// The pending styles, if any.
#[must_use]
pub fn read(ctx: &egui::Context) -> Option<Typing> {
    ctx.data(|d| d.get_temp::<Typing>(egui::Id::new(MEMORY_KEY)))
}

/// Set `command`'s pending style at `at`, or take it off if it is already set
/// there. Returns whether it is now set.
pub fn toggle(ctx: &egui::Context, at: usize, command: &'static str, pending: Pending) -> bool {
    let mut typing = read(ctx).filter(|t| t.at == at).unwrap_or(Typing {
        at,
        pending: Vec::new(),
    });
    let before = typing.pending.len();
    typing.pending.retain(|(id, _)| *id != command);
    let set = typing.pending.len() == before;
    if set {
        typing.pending.push((command, pending));
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(MEMORY_KEY), typing));
    set
}

/// Forget the pending styles.
pub fn forget(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<Typing>(egui::Id::new(MEMORY_KEY)));
}

/// The byte range of `edited` that was typed in place of nothing or of
/// something in `original`, by common prefix and suffix, with the character
/// index it starts at.
fn inserted(original: &str, edited: &str) -> Option<(usize, std::ops::Range<usize>)> {
    let prefix = original
        .chars()
        .zip(edited.chars())
        .take_while(|(a, b)| a == b)
        .count();
    let room = original.chars().count().min(edited.chars().count()) - prefix;
    let suffix = original
        .chars()
        .rev()
        .zip(edited.chars().rev())
        .take(room)
        .take_while(|(a, b)| a == b)
        .count();
    let byte = |chars: usize| {
        edited
            .char_indices()
            .nth(chars)
            .map_or(edited.len(), |(b, _)| b)
    };
    let end_chars = edited.chars().count() - suffix;
    (end_chars > prefix).then(|| (prefix, byte(prefix)..byte(end_chars)))
}

/// Push the range actions that give the characters typed at the pending point
/// their pending styles. Called after the text commit is pushed.
pub(super) fn follow(
    ctx: &egui::Context,
    page: usize,
    run: usize,
    original: &str,
    edited: &str,
    actions: &mut Vec<Action>,
) {
    let Some(typing) = read(ctx) else {
        return;
    };
    let Some((start, bytes)) = inserted(original, edited) else {
        return;
    };
    if start != typing.at {
        return;
    }
    let from = TextPosition::new(run, bytes.start);
    let to = TextPosition::new(run, bytes.end);
    let expected = edited[bytes].to_owned();
    for (_, pending) in typing.pending {
        actions.push(Action::Text(match pending {
            Pending::Style(change) => TextAction::SpanStyle {
                page,
                from,
                to,
                expected: expected.clone(),
                change,
            },
            Pending::Decorate(kind) => TextAction::Decorate {
                page,
                from,
                to,
                expected: expected.clone(),
                kind,
            },
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::inserted;

    #[test]
    fn the_typed_stretch_is_between_the_common_ends() {
        assert_eq!(inserted("ab", "aXYb"), Some((1, 1..3)));
        assert_eq!(inserted("ab", "abXY"), Some((2, 2..4)));
        assert_eq!(inserted("aa", "aaa"), Some((2, 2..3)));
        assert_eq!(inserted("ab", "ab"), None);
        assert_eq!(inserted("é", "éé"), Some((1, 2..4)));
    }
}
