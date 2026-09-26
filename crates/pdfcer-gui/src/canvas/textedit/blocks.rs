//! # `canvas::textedit::blocks` — the page's lines, reassembled into paragraphs
//!
//! ## What this is, and where it came from
//!
//!
//! > *"there was an acrobat feature in the original pdfcer-gui that attempted to
//! > reassemble individual lines into paragraphs and the cursor would move to
//! > the next block of text using the navigation keys."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/blocks.md`.

use pdfcer_core::text_edit::{BlockRecognitionOptions, EditableTextModel, TextPosition};

use super::{Anchor, Draft, commit_into, store};
use crate::app::state::OpenDoc;

/// Which way a navigation key moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vertical {
    /// Toward the top of the page.
    Up,
    /// Toward the bottom.
    Down,
}

/// **Where the caret goes when Up or Down is pressed in a run**, as
/// `(run, character offset)`.
#[must_use]
pub fn neighbour(doc: &OpenDoc, run: usize, caret: usize, dir: Vertical) -> Option<(usize, usize)> {
    let text = doc.page_text()?;
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    // CHARACTERS in, BYTES to the model, characters out.
    //
    // `Draft::caret` is a character index — its own docs are explicit, and the
    // reason is that a keystroke moves the caret by one character and `é` is one
    // keystroke and two bytes. `TextPosition::byte_offset` is a byte offset "on
    // a glyph boundary". Handing one to the other without converting compiles,
    // and puts the caret inside a multi-byte character on the first document
    // with an accent in it.
    let here = TextPosition::new(run, byte_offset(&text, run, caret)?);
    let x = model.caret_x(here).unwrap_or(0.0);
    let there = match dir {
        Vertical::Up => model.caret_up(here, x),
        Vertical::Down => model.caret_down(here, x),
    };
    if there == here {
        // The model says there is nothing that way. Answering `None` rather than
        // the same position lets the caller tell "moved nowhere" from "moved to
        // where it already was", which is the difference between redrawing and
        // committing a draft for no reason.
        return None;
    }
    Some((there.run, char_offset(&text, there.run, there.byte_offset)?))
}

/// **The start or end of the caret's own line**, as `(run, character offset)`.
#[must_use]
pub fn line_end(doc: &OpenDoc, run: usize, caret: usize, end: bool) -> Option<(usize, usize)> {
    let text = doc.page_text()?;
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    let here = TextPosition::new(run, byte_offset(&text, run, caret)?);
    let (start, stop) = model.line_range_at(here)?;
    let there = if end { stop } else { start };
    if there == here {
        return None;
    }
    Some((there.run, char_offset(&text, there.run, there.byte_offset)?))
}

/// A character index into run `run`, as a byte offset.
fn byte_offset(
    text: &pdfcer_core::text_extract::PageText,
    run: usize,
    caret: usize,
) -> Option<usize> {
    let s = &text.runs.get(run)?.text;
    Some(
        s.char_indices()
            .nth(caret)
            .map_or_else(|| s.len(), |(i, _)| i),
    )
}

/// A byte offset into run `run`, as a character index.
fn char_offset(
    text: &pdfcer_core::text_extract::PageText,
    run: usize,
    byte_offset: usize,
) -> Option<usize> {
    let s = &text.runs.get(run)?.text;
    let clamped = byte_offset.min(s.len());
    Some(s[..clamped].chars().count())
}

/// **Press Up or Down in a run-anchored draft.** `true` when the caret moved
/// and the caller must stop handling the event.
pub(super) fn step(
    ctx: &egui::Context,
    doc: &OpenDoc,
    draft: &Draft,
    dir: Vertical,
    actions: &mut Vec<crate::app::actions::Action>,
) -> bool {
    // A BOX draft is deliberately excluded, and so is a bare-page one. Their
    // lines are the shell's wrap rather than the page's, so this model would
    // move the caret to a run somewhere else on the sheet mid-paragraph.
    let Anchor::Run { run, .. } = draft.anchor else {
        return false;
    };
    let there = neighbour(doc, run, draft.caret, dir);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // BOTH run indices on the success line, because the number a wrong
        // build gets wrong is *which* run it landed in: a build that moved
        // within the run it was already in would trace a caret change and look
        // identical to one that crossed a paragraph.
        match there {
            Some((to, caret)) => {
                format!("text-caret-step dir={dir:?} from_run={run} to_run={to} to_caret={caret}")
            }
            None => {
                let caret = draft.caret;
                format!("text-caret-nowhere dir={dir:?} run={run} caret={caret}")
            }
        }
    });
    let Some((to_run, to_caret)) = there else {
        return false;
    };
    land(ctx, doc, draft, to_run, to_caret, actions);
    true
}

/// **Press Home or End in a run-anchored draft.** `true` when the caret moved
/// to a slot the draft did not already contain, and the caller must stop.
pub(super) fn line(
    ctx: &egui::Context,
    doc: &OpenDoc,
    draft: &Draft,
    end: bool,
    actions: &mut Vec<crate::app::actions::Action>,
) -> bool {
    let Anchor::Run { run, .. } = draft.anchor else {
        return false;
    };
    let Some((to_run, to_caret)) = line_end(doc, run, draft.caret, end) else {
        return false;
    };
    if to_run == run {
        // The line begins and ends inside this run, so the caller's own
        // within-draft move is both correct and cheaper. Deliberately NOT
        // handled here: landing would commit and re-seed a draft to put the
        // caret where a single assignment puts it.
        return false;
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        let to = to_run;
        format!("text-caret-line end={end} from_run={run} to_run={to} to_caret={to_caret}")
    });
    land(ctx, doc, draft, to_run, to_caret, actions);
    true
}

/// Commit the draft being left and open one on `to_run`.
fn land(
    ctx: &egui::Context,
    doc: &OpenDoc,
    draft: &Draft,
    to_run: usize,
    to_caret: usize,
    actions: &mut Vec<crate::app::actions::Action>,
) {
    commit_into(ctx, draft, actions);
    // The run's text is re-read from the page rather than carried across:
    // `Anchor::Run` holds the ORIGINAL for the next commit to compare against,
    // and the original is a fact about the moment the caret landed.
    let original = doc
        .page_text()
        .and_then(|t| t.runs.get(to_run).map(|r| r.text.clone()))
        .unwrap_or_default();
    store(
        ctx,
        Draft {
            page: draft.page,
            kind: draft.kind,
            anchor: Anchor::Run {
                run: to_run,
                original: original.clone(),
            },
            caret: to_caret.min(original.chars().count()),
            text: original,
            mark: None,
            seeded: true,
        },
    );
}

#[cfg(test)]
mod tests {
    /// **The two conversions are inverses**, which is the only property in
    /// this module a test can reach without a document.
    #[test]
    fn characters_and_bytes_round_trip_through_an_accent() {
        let s = "café";
        assert_eq!(s.chars().count(), 4);
        assert_eq!(s.len(), 5);
        for caret in 0..=s.chars().count() {
            let bytes = s
                .char_indices()
                .nth(caret)
                .map_or_else(|| s.len(), |(i, _)| i);
            assert_eq!(
                s[..bytes].chars().count(),
                caret,
                "character {caret} did not survive the round trip through byte {bytes}"
            );
        }
    }
}
