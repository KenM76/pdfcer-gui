//! # `editmodel::history` — the draft's own undo and redo
//!
//! A draft's text, caret and selection before each edit, so `Ctrl+Z` inside a
//! draft takes back a keystroke rather than a document command. A run of
//! typing, or of deleting, is one entry; anything else, or a caret move in
//! between, starts a new one.

/// The draft's state at one moment: its text, caret and selection mark, as
/// character indices into the text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snap {
    pub text: String,
    pub caret: usize,
    pub mark: Option<usize>,
}

/// What an edit was, for coalescing: consecutive `Typing` edits are one undo
/// entry, as are consecutive `Deleting` ones; every `Other` is its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKind {
    Typing,
    Deleting,
    Other,
}

/// The undo and redo stacks of one draft.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DraftHistory {
    undo: Vec<Snap>,
    redo: Vec<Snap>,
    run: Option<EditKind>,
}

impl DraftHistory {
    /// Record `before`, the state an edit of `kind` is about to change. Clears
    /// redo; joins the previous entry when `kind` continues its run.
    pub fn record(&mut self, before: Snap, kind: EditKind) {
        self.redo.clear();
        let joins = kind != EditKind::Other && self.run == Some(kind);
        if !joins {
            self.undo.push(before);
        }
        self.run = Some(kind);
    }

    /// End the current run, so the next edit is a new entry. Called on a caret
    /// move.
    pub const fn break_run(&mut self) {
        self.run = None;
    }

    /// The state to restore for an undo from `now`, or `None` when there is
    /// nothing to undo.
    pub fn undo(&mut self, now: Snap) -> Option<Snap> {
        let back = self.undo.pop()?;
        self.redo.push(now);
        self.run = None;
        Some(back)
    }

    /// The state to restore for a redo from `now`, or `None` when there is
    /// nothing to redo.
    pub fn redo(&mut self, now: Snap) -> Option<Snap> {
        let forward = self.redo.pop()?;
        self.undo.push(now);
        self.run = None;
        Some(forward)
    }

    /// Whether the draft has no edit to undo.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.undo.is_empty()
    }

    /// Whether the draft has an undone edit to redo.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

/// How many spaces take a caret at `x_pt` to the next tab stop: stops every
/// `stop_pt` from 0, each space `space_pt` wide, and always at least one.
/// All three in the same unit.
#[must_use]
pub fn tab_spaces(x_pt: f32, space_pt: f32, stop_pt: f32) -> usize {
    if !(space_pt > 0.0 && stop_pt > 0.0 && x_pt.is_finite()) {
        return 1;
    }
    let next = ((x_pt / stop_pt).floor() + 1.0) * stop_pt;
    let n = ((next - x_pt) / space_pt).round();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = n.clamp(1.0, 256.0) as usize;
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(text: &str) -> Snap {
        Snap {
            text: text.to_owned(),
            caret: text.chars().count(),
            mark: None,
        }
    }

    #[test]
    fn a_run_of_typing_is_one_entry_and_a_move_ends_it() {
        let mut h = DraftHistory::default();
        h.record(snap(""), EditKind::Typing);
        h.record(snap("a"), EditKind::Typing);
        h.break_run();
        h.record(snap("ab"), EditKind::Typing);
        assert_eq!(h.undo(snap("abc")), Some(snap("ab")));
        assert_eq!(h.undo(snap("ab")), Some(snap("")));
        assert_eq!(h.undo(snap("")), None);
    }

    #[test]
    fn deleting_after_typing_is_a_new_entry_and_other_never_joins() {
        let mut h = DraftHistory::default();
        h.record(snap(""), EditKind::Typing);
        h.record(snap("ab"), EditKind::Deleting);
        h.record(snap("a"), EditKind::Other);
        h.record(snap("a!"), EditKind::Other);
        assert_eq!(h.undo(snap("a!!")), Some(snap("a!")));
        assert_eq!(h.undo(snap("a!")), Some(snap("a")));
        assert_eq!(h.undo(snap("a")), Some(snap("ab")));
        assert_eq!(h.undo(snap("ab")), Some(snap("")));
    }

    #[test]
    fn redo_walks_back_and_a_new_edit_clears_it() {
        let mut h = DraftHistory::default();
        h.record(snap(""), EditKind::Typing);
        let back = h.undo(snap("x")).unwrap();
        assert_eq!(back, snap(""));
        assert!(h.can_redo());
        assert_eq!(h.redo(back), Some(snap("x")));
        assert!(!h.is_empty());
        h.undo(snap("x"));
        h.record(snap(""), EditKind::Typing);
        assert!(!h.can_redo());
    }

    #[test]
    fn a_tab_reaches_the_next_half_inch_stop() {
        assert_eq!(tab_spaces(0.0, 4.0, 36.0), 9);
        assert_eq!(tab_spaces(30.0, 3.0, 36.0), 2);
        assert_eq!(tab_spaces(36.0, 4.0, 36.0), 9);
        assert_eq!(tab_spaces(35.5, 4.0, 36.0), 1);
        assert_eq!(tab_spaces(10.0, 0.0, 36.0), 1);
    }
}
