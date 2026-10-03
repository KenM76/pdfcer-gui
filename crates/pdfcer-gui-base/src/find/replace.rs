//! # `find::replace` — what the Replace row holds and what a press asks for
//!
//! The row's own state lives on [`FindState`]; the rewriting is the app's
//! (`pdfcer_gui::app::actions::replace`), because it edits the document.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/find/replace.md`.

use pdfcer_core::annot_author::Quad;

use super::{FindState, Readout, reveal};
use crate::opendoc::OpenDoc;

/// Everything a Replace press needs from the bar, taken in one read.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplaceAsk {
    /// The query the hits were found with, as handed to the engine.
    pub query: String,
    /// Whether that search matched case.
    pub case_sensitive: bool,
    /// Whether that search used wildcards, which Replace refuses.
    pub wildcards: bool,
    /// The replacement, exactly as typed.
    pub with: String,
    /// The hits to replace, as (page, box in unrotated PDF user space): the
    /// current one alone, or every hit in document order.
    pub hits: Vec<(usize, Quad)>,
    /// The zero-based index of the current hit; 0 for Replace all.
    pub current: usize,
}

impl FindState {
    /// Whether the Replace row is showing.
    #[must_use]
    pub fn replace_open(&self) -> bool {
        self.replace_open
    }

    /// Show or hide the Replace row.
    pub fn toggle_replace(&mut self) {
        self.replace_open = !self.replace_open;
    }

    /// The replacement field's text.
    pub fn replacement_mut(&mut self) -> &mut String {
        &mut self.replacement
    }

    /// The press, or `None` when the bar is not on a current hit of the
    /// document at `epoch` — a stale or empty search replaces nothing.
    #[must_use]
    pub fn replace_ask(&self, epoch: u64, all: bool) -> Option<ReplaceAsk> {
        if !matches!(self.readout(epoch), Readout::At { .. }) {
            return None;
        }
        let results = self.results.as_ref()?;
        let pick = |h: &super::Hit| (h.page, h.quad);
        let hits = if all {
            results.hits.iter().map(pick).collect()
        } else {
            vec![pick(results.hits.get(results.current)?)]
        };
        Some(ReplaceAsk {
            query: results.query.clone(),
            case_sensitive: results.options.case_sensitive,
            wildcards: results.options.wildcards,
            with: self.replacement.clone(),
            hits,
            current: if all { 0 } else { results.current },
        })
    }

    /// Make hit `index` current (clamped to the last hit) and bring it into
    /// view, so a single Replace lands on the hit that followed the one it
    /// rewrote.
    pub fn land_on(&mut self, doc: &mut OpenDoc, index: usize) {
        let Some(results) = self.results.as_mut() else {
            return;
        };
        let Some(last) = results.hits.len().checked_sub(1) else {
            return;
        };
        let index = index.min(last);
        if results.current != index {
            results.current = index;
            reveal::reveal_current(self, doc);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_answer_asks_for_nothing() {
        assert!(FindState::default().replace_ask(0, true).is_none());
    }

    #[test]
    fn a_stale_search_asks_for_nothing() {
        assert!(
            FindState::searched("the", 3, 0)
                .replace_ask(1, true)
                .is_none()
        );
    }

    #[test]
    fn replace_all_asks_for_every_hit_and_one_for_the_current() {
        let mut state = FindState::searched("the", 3, 0);
        state.replacement_mut().push_str("our");
        let all = state.replace_ask(0, true).expect("on a hit");
        assert_eq!(
            (all.hits.len(), all.current, all.with.as_str()),
            (3, 0, "our")
        );
        let one = state.replace_ask(0, false).expect("on a hit");
        assert_eq!((one.hits.len(), one.current), (1, 0));
    }

    #[test]
    fn the_toggle_flips_the_row() {
        let mut state = FindState::default();
        assert!(!state.replace_open());
        state.toggle_replace();
        assert!(state.replace_open());
    }
}
