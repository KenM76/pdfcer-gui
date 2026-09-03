//! # `canvas::textedit::blocks` — the page's lines, reassembled into paragraphs
//!
//! ## What this is, and where it came from
//!
//! The operator, 2026-08-21:
//!
//! > *"there was an acrobat feature in the original pdfcer-gui that attempted to
//! > reassemble individual lines into paragraphs and the cursor would move to
//! > the next block of text using the navigation keys."*
//!
//! **This is salvage.** It existed in the shell this project is replacing, and
//! the first act was to read it there rather than to design it. What it did,
//! from `D:\Dev\pdfce\crates\pdfce-gui\src\main.rs`:
//!
//! ```text
//! egui::Key::ArrowUp   => model.caret_up(cur, model.caret_x(cur).unwrap_or(0.0)),
//! egui::Key::ArrowDown => model.caret_down(cur, model.caret_x(cur).unwrap_or(0.0)),
//! egui::Key::Home      => model.line_range_at(cur).map_or(cur, |(s, _)| s),
//! egui::Key::End       => model.line_range_at(cur).map_or(cur, |(_, e)| e),
//! ```
//!
//! ★★ **The reassembly is `pdfcer-core`'s and always was.**
//! `EditableTextModel::recognize` groups a page's show operators into lines and
//! lines into `Block`s by column band, and `caret_up` / `caret_down` walk
//! *lines* rather than runs — so a caret at the end of one paragraph's last line
//! steps into the next paragraph without anything here knowing what a paragraph
//! is. The old shell's whole contribution was **asking**.
//!
//! This shell had not been asking. Its caret is a character index into a
//! one-run draft, so Up and Down had no meaning and were not bound at all: there
//! is no line above a single run.
//!
//! ## ★ Why a page-space model rather than the draft's own string
//!
//! Because *"the next block of text"* is a fact about the **page**, not about
//! what is being typed. A draft knows one run's characters and nothing about
//! what is above or below it on the sheet — and the answer has to come from
//! geometry, because two runs adjacent in content order can be at opposite
//! corners of a drawing.
//!
//! That is why every function here takes the document and rebuilds the model.
//! It is not cheap (see [`neighbour`]'s note) and it is not on a per-frame path.
//!
//! ## What this deliberately does not do
//!
//! - **It does not move within a text BOX.** A box draft is multi-line in its
//!   own right and its lines are the shell's wrap, not the page's — so Up and
//!   Down there are a different question with a different answer, and answering
//!   it with this model would move the caret to a run somewhere else on the page
//!   mid-paragraph. Named rather than left to be discovered.
//! - **It does not draw the paragraph.** Showing which block the caret is in is
//!   worth doing and is a separate surface; this is the navigation.

use pdfcer_core::text_edit::{BlockRecognitionOptions, EditableTextModel, TextPosition};

use super::{Anchor, Draft, commit_into, store};
use crate::app::state::OpenDoc;

/// Which way a navigation key moves.
///
/// Named for what the operator pressed rather than for a sign, because "up" on
/// screen is a **larger** baseline y in PDF user space and a `-1` here would be
/// a number whose meaning depends on which space the reader has in mind. This
/// project has met that confusion four times in coordinate arithmetic; a
/// two-variant enum cannot have it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vertical {
    /// Toward the top of the page.
    Up,
    /// Toward the bottom.
    Down,
}

/// **Where the caret goes when Up or Down is pressed in a run**, as
/// `(run, character offset)`.
///
/// `None` when there is nothing there — the top line of the topmost block, a
/// page whose text will not extract, a run the model does not recognise. The
/// caller leaves the caret where it is, which is what every editor does at the
/// end of a document.
///
/// # ★★ It crosses paragraphs without knowing what one is
///
/// `caret_up` and `caret_down` walk the model's **lines**, and a `Block` is a
/// group of lines. So a caret on the last line of one paragraph steps to the
/// first line of the next, and nothing in this function had to look at a block
/// to make that happen. That is the whole of the operator's *"the cursor would
/// move to the next block of text"*, and it is `pdfcer-core`'s recognition doing
/// the work.
///
/// # ★ The desired column is preserved, which is what makes repeated presses
/// # behave
///
/// `caret_x` is the caret's page-space x; passing it to `caret_up` asks for the
/// nearest slot in the same column on the line above. Without it a caret
/// stepping through lines of unequal length would drift toward whichever end the
/// implementation happened to clamp to, and three presses down and three back up
/// would not return it to where it started.
///
/// Not carried across presses, deliberately: a true "desired column" is
/// remembered from the *first* vertical press and survives short lines in
/// between, which is what a text editor does. That is a second piece of state
/// and it is not built — recorded here so it is a decision rather than an
/// oversight. What is built is right for one press at a time.
///
/// # Cost
///
/// One provenance-free extraction and one recognition of the whole page, per
/// press. Measured elsewhere in this crate at **336 ms on the benchmark CAD
/// sheet** for the extraction alone — which is why this is on a *keystroke*
/// path and not a frame path, and why the caller must not call it speculatively.
#[must_use]
pub fn neighbour(doc: &OpenDoc, run: usize, caret: usize, dir: Vertical) -> Option<(usize, usize)> {
    let text = doc.page_text()?;
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    // ★ CHARACTERS in, BYTES to the model, characters out.
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
///
/// Home and End. Salvaged from the same four lines as [`neighbour`] and using
/// the same model, so a line means the same thing to both.
///
/// ★ A *line* here is the page's, not the run's — a line drawn as four separate
/// show operators (a CAD title block's row, which is the shape this operator's
/// documents are full of) is one line to the model, so End reaches the end of
/// what he can see rather than the end of the fragment he happens to be in.
/// That is the same recognition that made `Reason::SharesTheLine` necessary, put
/// to a second use.
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
///
/// # ★★ Why this is a function and not four lines in the match arm
///
/// Because the arm has to do three things in a fixed order and two of them are
/// easy to leave out: trace the outcome, **commit the draft it is leaving**,
/// and seed the new one from the page rather than from the draft it just left.
/// A caret that walks out of a run with unsaved keystrokes in it silently
/// discards them, which is this project's defining defect class — and
/// `commit_into` writes nothing when the text is unchanged, so an operator
/// merely *reading* with the arrow keys puts nothing on the undo stack.
///
/// # ★★★ The NOWHERE outcome is traced too, and that came from a driven run
///
/// The first live run of `arrow_keys_walk_between_blocks` failed with *"the
/// arrow keys moved the caret nowhere"*, and the trace could not say which of
/// the two causes the check itself had named it was:
///
/// | cause | where it lives | what it means |
/// |---|---|---|
/// | the keys never reached this arm | the shell — an earlier arm ate them | a **defect** |
/// | [`neighbour`] answered `None` | the page — no line that way | a **fact about the document** |
///
/// One trace line covered only the success, so both failures looked identical
/// from outside: silence. `text-caret-nowhere` is the other half. That is
/// `DEFECTS.md` D14's rule in its less obvious form — *a trace must be able to
/// say the thing did not happen*, not only that it did, because a check that
/// cannot tell a build defect from a fixture fact will eventually accuse the
/// wrong one.
///
/// ★ And `None` is genuinely ordinary here, which is why it must not read as
/// an error: `caret_up`/`caret_down` never cross a **column band**
/// (`pdfcer-core`'s reading order), so a lone label in the middle of a drawing
/// has nothing above or below it by construction.
pub(super) fn step(
    ctx: &egui::Context,
    doc: &OpenDoc,
    draft: &Draft,
    dir: Vertical,
    actions: &mut Vec<crate::app::actions::Action>,
) -> bool {
    // ★ A BOX draft is deliberately excluded, and so is a bare-page one. Their
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
///
/// # ★★ A LINE IS THE PAGE'S, NOT THE RUN'S — which is the whole point
///
/// A row of a CAD title block is drawn as four or five separate show
/// operators, and the operator sees **one line**. So End belongs at the end of
/// what he can see rather than at the end of whichever fragment he happened to
/// click in, and reaching it means landing in a **different run** exactly as
/// [`step`] does. That is the same recognition that made `Reason::SharesTheLine`
/// necessary, put to a second use.
///
/// ★ `false` is the ordinary answer, not a failure: on a line that is one run,
/// [`line_end`] reports there is nowhere new to go and the caller falls back to
/// moving within the draft — which lands in the same place, one allocation
/// cheaper and without touching the undo stack.
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
///
/// ★★ The order is load-bearing: **commit first**. A caret that walks out of a
/// run with unsaved keystrokes in it silently discards them, which is this
/// project's defining defect class — and `commit_into` writes nothing when the
/// text is unchanged, so an operator merely *reading* with the navigation keys
/// puts nothing on the undo stack.
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
    /// ★ **The two conversions are inverses**, which is the only property in
    /// this module a test can reach without a document.
    ///
    /// Everything else here is `pdfcer-core`'s recognition, which has its own
    /// tests and needs a real page. What is *this* module's own is the
    /// character ⟷ byte hop, and it is exactly the kind of arithmetic that
    /// compiles either way round and puts the caret inside a multi-byte
    /// character on the first document with an accent in it.
    ///
    /// Asserted on a string that has one: `"café"` is five bytes and four
    /// characters, so a caret index of 4 is a byte offset of 5 and any
    /// implementation that confused them would be off by one at the end.
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
