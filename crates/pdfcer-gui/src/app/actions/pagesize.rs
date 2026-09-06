//! # `app::actions::pagesize` — changing the **paper** an open drawing sits on
//!
//! The body of [`PageAction::SetPageSize`], and the pre-commit
//! [`survey`] the sheet-size window reads to tell the operator, *before* he
//! commits, which of two very different things he is about to get.
//!
//! Split out of [`super::pages`] under **R2** rather than added to it: that
//! file was at 1,374 of its 1,500 lines, and its subject is *"a page index is a
//! position, not an identity"* — the resync a **structural** edit owes. A media
//! box change is not structural in that sense. It adds, removes and renumbers
//! nothing, so every one of that file's five table rows is "unchanged" for this
//! verb, and putting it there would have made the reader look for a row that is
//! not there.
//!
//! ---
//!
//! ## ★★★ 1. THE ONE FACT THIS MODULE EXISTS TO CARRY
//!
//! **`/MediaBox` is the paper. Changing it does not move, scale or reflow one
//! byte of what is drawn on the page.**
//!
//! So an A1 drawing put onto A4 paper is **cropped, not shrunk**, and the
//! operator who reaches for this control expecting *"print it smaller"* loses
//! his title block instead. Every other page-size control he has ever used —
//! Word, LibreOffice, a print dialog's Fit-to-page — reflows or scales. This
//! one does not.
//!
//! ### Measured, not assumed — 2026-09-06, engine v0.41.0
//!
//! Through the engine's own `set-page-size` (which calls the same
//! `EditSession::set_media_boxes` this module calls), on
//! `fixtures/a1-titleblock.pdf`, A1 → A4:
//!
//! | what was measured | result |
//! |---|---|
//! | every glyph run's coordinate (`extract-text --json`, full diff) | **byte-identical**. The title block sits at x 1831–2207 pt before and after; an A4 sheet stops at 595.28, so it is entirely off the paper and still in the content stream |
//! | `/CropBox`, `/BleedBox`, `/TrimBox`, `/ArtBox` | **all four untouched**. Only `/MediaBox` is rewritten |
//! | annotation `/Rect` (`annots-with-everything.pdf` → A6) | **identical**. A `Square` at 120,560–320,700 is still there on a 298 × 420 sheet, i.e. off it |
//! | form fields (`list-fields`, full diff) | **identical** |
//! | ce dimensions (`dimension-list`, `/PieceInfo` sidecar, printed value) | **identical** — the value stayed `400.00 pt`, which is *correct*: the drawing did not move, so the measurement is still true |
//! | a certified document | refused by name, `CertificationForbidsChange` |
//! | an encrypted document with no password | refused by name at open |
//!
//! ★★ **R8b rule 15, and the reason this verb is safe.** A **pdf dimension** —
//! the printed measurement a CAD exporter drew — is page content: it does not
//! move, and it can end up off the sheet. A **ce dimension** — the one pdfcer
//! authored — is an annotation plus a `/PieceInfo` sidecar: it does not move
//! either, and its number stays true *because* nothing moved. A verb that
//! scaled the drawing to fit would have to rescale every ce dimension group's
//! calibration to keep its numbers honest, and would silently falsify every
//! group it missed. This verb cannot get that wrong, because it does not touch
//! them.
//!
//! ## ★★ 2. Why there is no "scale to fit", and what it would have cost
//!
//! The engine has no scale-to-fit verb, and composing one here would not have
//! been a small thing. `EditSession::transform_objects` can wrap a selection of
//! **page objects** in `q <cm> … Q` — kind-agnostic, one undoable command — but
//! it takes explicit object indices on **one** page and does not touch
//! annotations, form-field widgets or ce dimensions, which have their own five
//! verbs (`move_annotation`, `resize_annotation`, `move_widget`,
//! `move_dimension`, `set_group_scale`). A true scale-to-fit is therefore six
//! verbs composed across N pages, each with its own refusals, and its most
//! likely failure — a ce dimension group left at the old calibration — produces
//! a drawing that *prints a wrong measurement and looks perfectly correct*.
//!
//! ⇒ **pdfcer changes the paper.** The decision is not "scale-to-fit is hard",
//! it is that a half-built scale-to-fit is the worst artefact in this problem
//! space. What is built instead is the thing that makes the crop survivable:
//! the operator is told, **before he commits and in points**, exactly how far
//! his drawing runs past the paper he has picked. See [`survey`].
//!
//! ## ★★★ 3. The shell measures what the engine says it cannot
//!
//! `MediaBoxChange::lost_area` carries a named residual, in the engine's own
//! words: it reports that the **sheet** shrank, *not* that any **content** was
//! in the region it lost, because *"pdfcer has no page-content bounding-box
//! facility yet"*.
//!
//! ★ That sentence is true of `EditSession` and **false of `pdfcer-core`**:
//! `pdfcer_core::vector::PageObjects::page_bbox` is *"the union of every
//! object's page bbox — the page's drawn extent in page space"*, and this shell
//! already holds one per page in [`crate::app::cache`]. So the disclosure the
//! engine could only make in geometry — *"the sheet lost area"* — is made here
//! in the operator's terms: *"the drawing runs 1,636 pt past the right edge"*.
//!
//! ⚠ **And the boundary is stated rather than implied.** `page_bbox` is the
//! union of the **drawn** objects. Annotations are separate objects with their
//! own `/Rect`, which it does not walk; they keep their coordinates too, so
//! they can fall off exactly as content can.
//! [`crate::text::page_size::annots_not_counted`] says so on screen, and its
//! own test holds that it keeps saying so.
//!
//! ## 4. Why the PLURAL verb, even for one sheet
//!
//! `EditSession::set_media_boxes` — *"a sheet set is resized as a set"*, one
//! undo entry however many pages, refusals raised **before anything is
//! committed** so an out-of-range index leaves the document untouched rather
//! than half-resized. Calling the singular verb in a loop would be functionally
//! identical and would leave the operator pressing Undo once per sheet.
//!
//! ★ It was written for the drawing-set case on 2026-08-18 and **called by
//! nothing for nineteen days**, because no `pages.resize` command existed. Its
//! own doc comment names this shell as the caller it was written for. That is
//! the fourth instance this week of a capability arriving, being written down,
//! and the writing-down being mistaken for the acting-on — `crate::app::blank`
//! §3a records the same verb's arrival in as many words and did nothing with
//! the plural half of it.

use pdfcer_core::edit::{EditError, EditSession, MediaBoxChange, MediaBoxEntry};
use pdfcer_core::page_tree::Rect;

use crate::app::state::OpenDoc;
use crate::text::page_size as t;

/// **What the picked sheets are, and what is drawn on them** — everything the
/// sheet-size window needs to answer *"what will this do to my drawing?"*
/// before the operator commits.
///
/// Built once when the window opens, from state the application already holds:
/// the flattened page vector (walked every frame anyway) and the object-model
/// cache (built for the current page anyway). **Nothing here decomposes a page
/// that was not already decomposed** — see [`Self::unread`] for the price that
/// bounded cost is paid in, and why it is paid in honesty rather than in
/// silence.
#[derive(Debug, Clone, PartialEq)]
pub struct SheetSurvey {
    /// The operand pages, 0-based, ascending and unique — whatever
    /// `crate::panels::pages::ops::operands` resolved: the picked sheets when
    /// there are any, the current sheet when there are none.
    pub pages: Vec<usize>,
    /// Each operand's resolved media box, in `pages` order.
    pub boxes: Vec<Rect>,
    /// The union of the drawn extents that could be read, in page space, or
    /// `None` when none could be.
    ///
    /// A union across sheets is the right shape here because the question the
    /// window asks is *"does the new paper hold all of this"*, and the answer
    /// for a set is the answer for its worst member.
    pub drawn: Option<Rect>,
    /// How many operand sheets' drawn extent could **not** be read.
    ///
    /// ★★ Non-zero is the ordinary case for a multi-sheet pick, and it is a
    /// deliberate design point rather than a defect. The object-model cache
    /// holds **one page** — it is keyed on `(page, epoch)` — so reading every
    /// sheet of a drawing set would mean decomposing each in turn on the frame
    /// the window opens. His `ncored-benchmark-cad-drawing.pdf` holds 129,758
    /// objects on one page; doing that ten times to populate a label would
    /// freeze the application for the operator who most needs this control.
    ///
    /// ⇒ So the extent is read for the sheets already decomposed, the count of
    /// the rest is carried here, and
    /// [`crate::text::page_size::overhang_unmeasurable`] says so out loud. A
    /// measurement over one of ten sheets reported as *"nothing falls off"*
    /// would be a false negative dressed as a fact, which is exactly the
    /// failure the engine's own residual refuses to commit.
    pub unread: usize,
    /// The lower-left corner every operand shares, when they share one.
    ///
    /// `None` when they disagree — see [`target_rect`], which is where it
    /// changes what gets written.
    pub common_origin: Option<(f64, f64)>,
}

impl SheetSurvey {
    /// The one size every operand already is, when they are all the same.
    ///
    /// `None` on a mixed pick — the ordinary state of a drawing set with a
    /// detail sheet in it, which is why the window has a sentence for it.
    /// Compared to [`pdfcer_core::paper::PaperSize::CLASSIFY_TOLERANCE`], for
    /// the reason that constant exists: producers round A4 to 595.276, 595.28
    /// and 595.32, and exact equality would call an obviously uniform set
    /// mixed.
    #[must_use]
    pub fn uniform(&self) -> Option<Rect> {
        let first = *self.boxes.first()?;
        let tol = pdfcer_core::paper::PaperSize::CLASSIFY_TOLERANCE;
        self.boxes
            .iter()
            .all(|r| {
                (r.width() - first.width()).abs() <= tol
                    && (r.height() - first.height()).abs() <= tol
            })
            .then_some(first)
    }

    /// How many distinct sizes the pick holds, to
    /// [`pdfcer_core::paper::PaperSize::CLASSIFY_TOLERANCE`].
    ///
    /// Reported rather than a bare "they differ" because *"9 sheets in 2
    /// different sizes"* tells the operator he has one odd sheet and *"9 sheets
    /// in 7 different sizes"* tells him he has picked the wrong thing.
    #[must_use]
    pub fn distinct_sizes(&self) -> usize {
        let tol = pdfcer_core::paper::PaperSize::CLASSIFY_TOLERANCE;
        let mut seen: Vec<Rect> = Vec::new();
        for r in &self.boxes {
            if !seen.iter().any(|s| {
                (s.width() - r.width()).abs() <= tol && (s.height() - r.height()).abs() <= tol
            }) {
                seen.push(*r);
            }
        }
        seen.len()
    }

    /// **How far the drawing runs past `target`**, per edge, in points:
    /// `(left, right, bottom, top)`, each clamped at zero.
    ///
    /// `None` when nothing could be measured — which is *not* the same as zero
    /// and must not be collapsed into it. The caller words the two differently:
    /// `Some((0,0,0,0))` is [`crate::text::page_size::fits`], a promise;
    /// `None` is [`crate::text::page_size::overhang_unmeasurable`], a stated
    /// boundary.
    #[must_use]
    pub fn overhang(&self, target: Rect) -> Option<(f64, f64, f64, f64)> {
        let drawn = self.drawn?;
        Some((
            (target.llx - drawn.llx).max(0.0),
            (drawn.urx - target.urx).max(0.0),
            (target.lly - drawn.lly).max(0.0),
            (drawn.ury - target.ury).max(0.0),
        ))
    }

    /// **The rectangle to write for a sheet of `w_pt` × `h_pt`.**
    ///
    /// # ★★ The lower-left corner is the operands' own, not the origin
    ///
    /// `PaperSize::rect_with` puts a named sheet at `(0, 0)`, and its own doc
    /// comment calls that *"a choice, not a law"*: §7.7.3.3 does not require a
    /// media box to start at the origin, and imposition output and cropped
    /// scans really do carry offset ones. On such a page, writing an
    /// origin-anchored sheet moves **the paper relative to the drawing** —
    /// which is not what "change the sheet size" means and is invisible until a
    /// print comes out shifted.
    ///
    /// So the new sheet keeps the corner the old sheets had, when they share
    /// one. On the overwhelmingly common `(0, 0)` case this is byte-identical
    /// to `rect_with`, which is what makes it safe unconditionally.
    ///
    /// ★ When the operands do **not** share a corner, one rectangle cannot
    /// preserve all of them — `set_media_boxes` takes one rectangle for the
    /// whole selection, and that is the property that buys the single undo
    /// entry. The fallback is the origin, and
    /// [`crate::text::page_size::origin_differs`] is drawn in the window rather
    /// than the choice being made quietly.
    #[must_use]
    pub fn target_rect(&self, w_pt: f64, h_pt: f64) -> Rect {
        let (llx, lly) = self.common_origin.unwrap_or((0.0, 0.0));
        Rect::from_corners(llx, lly, llx + w_pt, lly + h_pt)
    }
}

/// Read the picked sheets and what is drawn on them.
///
/// `pages` is the operand list `crate::app::dispatch::pages` already resolved,
/// so this function never re-decides which sheets are meant — one statement of
/// the operand rule, which is the same argument
/// `SelectionState::deletable_objects_on` makes.
///
/// # What it costs
///
/// One index into `doc.pages` per operand, plus **at most one** borrow of the
/// object-model cache — `OpenDoc::page_objects` returns the decomposition for
/// the page currently on screen, already built if the canvas has drawn it. No
/// page is decomposed on this call's account. See [`SheetSurvey::unread`].
#[must_use]
pub fn survey(doc: &OpenDoc, pages: &[usize]) -> SheetSurvey {
    let boxes: Vec<Rect> = pages
        .iter()
        .filter_map(|&i| doc.pages.get(i).map(|page| page.media_box))
        .collect();

    // The corner every operand shares, if they share one. Compared with the
    // classify tolerance rather than exactly, for `SheetSurvey::uniform`'s
    // reason: a producer that wrote `0.0001` for a corner has not moved it.
    let tol = pdfcer_core::paper::PaperSize::CLASSIFY_TOLERANCE;
    let common_origin = boxes.first().map(|r| (r.llx, r.lly)).filter(|&(x, y)| {
        boxes
            .iter()
            .all(|r| (r.llx - x).abs() <= tol && (r.lly - y).abs() <= tol)
    });

    // The drawn extent, for the operand pages whose decomposition is already
    // in hand. Today that is the page on screen and only when it is an operand;
    // the count of the rest is what the window reports.
    let mut drawn: Option<Rect> = None;
    let mut measured = 0_usize;
    if pages.contains(&doc.view.page_index)
        && let Some(provider) = doc.page_objects()
    {
        let bounds = provider.page_objects().page_bbox();
        if !bounds.is_empty() {
            drawn = Some(Rect::from_corners(
                bounds.min.x,
                bounds.min.y,
                bounds.max.x,
                bounds.max.y,
            ));
        }
        // ★ Counted as measured even when the page draws NOTHING. An empty page
        // genuinely has no content to lose, and reporting it as unread would
        // make a blank sheet look like a failure to look — which is the
        // difference between `crate::text::page_size::fits` (a promise) and
        // `overhang_unmeasurable` (a stated boundary), and they must not swap.
        measured = 1;
    }

    SheetSurvey {
        pages: pages.to_vec(),
        boxes,
        drawn,
        unread: pages.len().saturating_sub(measured),
        common_origin,
    }
}

/// **The engine call behind [`PageAction::SetPageSize`], with its disclosures.**
///
/// Handed to `super::apply::vector_edit` as a closure rather than run here, so
/// the whole four-step protocol — cancel the worker, mutate through
/// `Arc::get_mut`, bump the epoch, drop the texture, resync — is the one in
/// `apply.rs` and not a fifth copy.
///
/// # What the returned sentences are, and why they are not optional
///
/// Rule 4. Three of `MediaBoxChange`'s fields are consequences that are
/// **invisible in the page view**, and a fourth is invisible in the file:
///
/// * `lost_area` — the sheet shrank. pdfcer removes no content, but
///   §14.11.2.1 licenses any *other* tool to discard what is now outside the
///   media box *"without affecting the meaning of the PDF file"*. Reversible
///   here by Undo; not reversible after a round trip through anything else.
///   **That asymmetry is the disclosure.**
/// * `crop_box_outside` — a `/CropBox` the new sheet no longer contains. Left
///   alone deliberately (a conforming reader intersects the two), so the
///   visible region is now the smaller of them and the operator cannot see
///   which reading he is getting.
/// * `size_advisory` — outside Annex C.2's recommended range. Advice, worded as
///   advice; ISO 32000-2 dropped the range entirely.
/// * `entry == InheritedSoOwnEntryRemoved` — the page's own `/MediaBox` was
///   **removed** because an ancestor already said that size, so the page is now
///   sized by inheritance. That changes what a *later* edit does to it, which
///   is exactly the kind of fact nothing else will ever tell him.
///
/// ⚠ **What is NOT disclosed, because the engine does not report it.**
/// `/BleedBox`, `/TrimBox` and `/ArtBox` are left byte-identical (measured
/// 2026-09-06 — a `/BleedBox [10 10 1000 1000]` survived a resize to 595 × 842
/// untouched), and `MediaBoxChange` carries **no field for them**: only
/// `crop_box_outside`. A CAD or press export that carries a bleed box therefore
/// gets one overhang disclosed and three not. Filed for the engine; nothing is
/// faked here in the meantime, because a disclosure this shell computed from a
/// walk the engine did not do would be a fifth source of truth about the same
/// page dictionary.
///
/// # Errors
///
/// [`EditError::CertificationForbidsChange`] — measured, and the one refusal an
/// operator will actually meet. [`EditError::MediaBoxDegenerate`] — unreachable
/// from the window, which bounds its own custom fields, and reachable from a
/// future caller. [`EditError::PageOutOfRange`], [`EditError::PageTree`],
/// [`EditError::NotADictionary`].
///
/// [`PageAction::SetPageSize`]: super::pages::PageAction::SetPageSize
pub(super) fn set(
    session: &mut EditSession,
    pages: &[usize],
    rect: Rect,
) -> Result<Vec<String>, EditError> {
    let changes = session.set_media_boxes(pages, rect)?;

    // ★★★ Traced from the SESSION's own page tree, re-walked after the commit —
    // not from `rect`, and not from `change.after`.
    //
    // `crate::app::blank`'s `document_sized` makes the identical argument for
    // the identical reason, and `ui-verify` reads that line for it: *a trace of
    // the request says what this function was told; a trace of the page tree
    // says what a reader of the resulting file will see.* A build that recorded
    // the request and dropped the write would have a perfect `w=`/`h=` here and
    // an unchanged document — which is the whole class of defect this project
    // is named after.
    let after = session.pages().unwrap_or_default();
    for &index in pages {
        let media = after.get(index).map(|page| page.media_box);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-size-sheet index={index} w={:.2} h={:.2} llx={:.2} lly={:.2}",
                media.map_or(0.0, |m| m.urx - m.llx),
                media.map_or(0.0, |m| m.ury - m.lly),
                media.map_or(0.0, |m| m.llx),
                media.map_or(0.0, |m| m.lly),
            )
        });
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "page-size-applied n={} asked_w={:.2} asked_h={:.2} lost_area={} crop_outside={} \
             advisories={} explicit={} inherited_removed={} base_kept={}",
            changes.len(),
            rect.width(),
            rect.height(),
            changes.iter().filter(|c| c.lost_area).count(),
            changes
                .iter()
                .filter(|c| c.crop_box_outside.is_some())
                .count(),
            changes.iter().filter(|c| c.size_advisory.is_some()).count(),
            count_entry(&changes, MediaBoxEntry::ExplicitWritten),
            count_entry(&changes, MediaBoxEntry::InheritedSoOwnEntryRemoved),
            count_entry(&changes, MediaBoxEntry::BaseSpellingKept),
        )
    });

    Ok(disclosures(&changes))
}

/// How many changes ended in `want`.
///
/// A helper rather than three inline filters because `MediaBoxEntry` is
/// `#[non_exhaustive]`: one place to look when a variant is added, instead of
/// three that each silently stop counting it.
fn count_entry(changes: &[MediaBoxChange], want: MediaBoxEntry) -> usize {
    changes.iter().filter(|c| c.entry == want).count()
}

/// The rule-4 sentences for a set of [`MediaBoxChange`]s.
///
/// **Counted across the set rather than named per page**, which is the opposite
/// of what the CLI does and right for the opposite reason. The CLI's invocation
/// *is* the commit and it has one file in front of it, so it prints a note per
/// page. This surface is a status line an operator reads in passing while the
/// document is on screen in front of him: *"7 sheets lost area"* is a fact he
/// can act on, and seven sentences differing only in a number is a wall he will
/// stop reading — which is how a disclosure stops being one.
fn disclosures(changes: &[MediaBoxChange]) -> Vec<String> {
    let mut notes = Vec::new();

    let lost = changes.iter().filter(|c| c.lost_area).count();
    if lost > 0 {
        notes.push(t::disclosure_lost_area(lost));
    }

    let cropped = changes
        .iter()
        .filter(|c| c.crop_box_outside.is_some())
        .count();
    if cropped > 0 {
        notes.push(t::disclosure_crop_outside(cropped));
    }

    let inherited = count_entry(changes, MediaBoxEntry::InheritedSoOwnEntryRemoved);
    if inherited > 0 {
        notes.push(t::disclosure_inherited(inherited));
    }

    // ★ The two Annex C.2 directions are separate sentences, because
    // `PageSizeAdvisory` sets both flags at once for a long thin sheet (2 ×
    // 20,000) and a single line reading "outside the recommended range" would
    // lose which end. The engine keeps them a pair of facts rather than an enum
    // for the same reason.
    let below = changes
        .iter()
        .filter(|c| c.size_advisory.is_some_and(|a| a.below_minimum))
        .count();
    if below > 0 {
        notes.push(t::disclosure_size_advisory(below, true));
    }
    let above = changes
        .iter()
        .filter(|c| c.size_advisory.is_some_and(|a| a.above_maximum))
        .count();
    if above > 0 {
        notes.push(t::disclosure_size_advisory(above, false));
    }

    notes
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A survey with the given boxes and no drawn extent.
    fn survey_of(boxes: Vec<Rect>) -> SheetSurvey {
        let common_origin = boxes.first().map(|r| (r.llx, r.lly)).filter(|&(x, y)| {
            boxes
                .iter()
                .all(|r| (r.llx - x).abs() < 1.0 && (r.lly - y).abs() < 1.0)
        });
        SheetSurvey {
            pages: (0..boxes.len()).collect(),
            boxes,
            drawn: None,
            unread: 0,
            common_origin,
        }
    }

    /// ★★★ **The overhang is the operator's own case, in his own numbers.**
    ///
    /// His A1 title block sits at x 1831–2207 pt (measured from
    /// `fixtures/a1-titleblock.pdf` with `extract-text --json`). A4 stops at
    /// 595.28. This is the arithmetic the window shows him before he presses
    /// anything, and getting it wrong in either direction is the difference
    /// between a warning he heeds and one he ignores.
    #[test]
    fn his_title_block_is_measured_as_running_off_the_right_edge() {
        let mut survey = survey_of(vec![Rect::from_corners(0.0, 0.0, 2383.94, 1683.78)]);
        survey.drawn = Some(Rect::from_corners(80.0, 60.0, 2231.54, 1620.0));

        let a4 = Rect::from_corners(0.0, 0.0, 595.2756, 841.8898);
        let (left, right, bottom, top) = survey.overhang(a4).expect("the extent was measured");
        assert!(
            (left - 0.0).abs() < 0.01,
            "nothing hangs off the left: {left}"
        );
        assert!(
            (bottom - 0.0).abs() < 0.01,
            "nothing hangs off the bottom: {bottom}"
        );
        assert!(
            (right - 1636.26).abs() < 0.1,
            "the title block runs 1,636 pt past A4's right edge, not {right}"
        );
        assert!(
            (top - 778.11).abs() < 0.1,
            "and 778 pt past its top, not {top}"
        );
    }

    /// ★★★ **"Could not measure" is not "nothing falls off".**
    ///
    /// The single most dangerous confusion available in this module, and the
    /// one the engine's own residual refuses to make. `None` must survive as
    /// `None` all the way to the caller, because the caller words the two
    /// differently and a `unwrap_or_default()` anywhere on this path would
    /// silently turn a stated boundary into a promise.
    #[test]
    fn an_unmeasured_extent_is_none_and_not_zero() {
        let survey = survey_of(vec![Rect::from_corners(0.0, 0.0, 2383.94, 1683.78)]);
        assert!(survey.drawn.is_none());
        assert_eq!(
            survey.overhang(Rect::from_corners(0.0, 0.0, 595.0, 842.0)),
            None,
            "an unmeasured extent must not report a zero overhang"
        );
    }

    /// ★★ **A drawing that fits reports zeros, not `None`.**
    ///
    /// The other half of the pair above. Without this, a build that returned
    /// `None` whenever the overhang was zero would pass the test above and
    /// never once show the operator the sentence that says he is safe.
    #[test]
    fn a_drawing_that_fits_reports_a_measured_zero() {
        let mut survey = survey_of(vec![Rect::from_corners(0.0, 0.0, 595.0, 842.0)]);
        survey.drawn = Some(Rect::from_corners(50.0, 50.0, 300.0, 400.0));
        assert_eq!(
            survey.overhang(Rect::from_corners(0.0, 0.0, 595.2756, 841.8898)),
            Some((0.0, 0.0, 0.0, 0.0))
        );
    }

    /// ★★ **A set is uniform to the producer's rounding, not to the bit.**
    ///
    /// A4 in points is 595.2755905511811, and producers write 595.276, 595.28
    /// and 595.32. An exact-equality check would call a set of ten identical A4
    /// sheets "mixed" and put a false sentence on screen for every real
    /// document.
    #[test]
    fn producer_rounding_does_not_make_a_uniform_set_mixed() {
        let survey = survey_of(vec![
            Rect::from_corners(0.0, 0.0, 595.276, 841.89),
            Rect::from_corners(0.0, 0.0, 595.28, 841.8898),
            Rect::from_corners(0.0, 0.0, 595.2755905511811, 841.8897637795276),
        ]);
        assert!(survey.uniform().is_some(), "these are all A4");
        assert_eq!(survey.distinct_sizes(), 1);
    }

    /// ★★ **A real mixed set is seen as mixed.**
    ///
    /// The falsifying direction of the test above: a tolerance wide enough to
    /// absorb producer rounding must not be wide enough to call an A3 detail
    /// sheet an A1. The nearest two distinct sizes in the engine's table differ
    /// by far more than a point, which is what makes 1 pt safe.
    #[test]
    fn a_drawing_set_with_a_detail_sheet_reads_as_two_sizes() {
        let survey = survey_of(vec![
            Rect::from_corners(0.0, 0.0, 2383.94, 1683.78),
            Rect::from_corners(0.0, 0.0, 2383.94, 1683.78),
            Rect::from_corners(0.0, 0.0, 1190.55, 841.89),
        ]);
        assert!(survey.uniform().is_none());
        assert_eq!(survey.distinct_sizes(), 2);
    }

    /// ★★★ **An offset sheet keeps its corner**, and a mixed-corner pick falls
    /// back to the origin.
    ///
    /// Both directions, because each is a single missing clause and each
    /// produces a silent shift of the paper relative to the drawing —
    /// invisible on screen and visible on a plot.
    #[test]
    fn the_new_sheet_keeps_the_corner_the_old_sheets_shared() {
        let offset = survey_of(vec![
            Rect::from_corners(100.0, 200.0, 2483.94, 1883.78),
            Rect::from_corners(100.0, 200.0, 2483.94, 1883.78),
        ]);
        let rect = offset.target_rect(595.2756, 841.8898);
        assert!((rect.llx - 100.0).abs() < 0.01, "{rect:?}");
        assert!((rect.lly - 200.0).abs() < 0.01, "{rect:?}");
        assert!((rect.width() - 595.2756).abs() < 0.01, "{rect:?}");

        let mixed = survey_of(vec![
            Rect::from_corners(100.0, 200.0, 2483.94, 1883.78),
            Rect::from_corners(0.0, 0.0, 2383.94, 1683.78),
        ]);
        assert!(mixed.common_origin.is_none());
        let rect = mixed.target_rect(595.2756, 841.8898);
        assert!(
            (rect.llx).abs() < 0.01 && (rect.lly).abs() < 0.01,
            "a mixed-corner pick anchors at the origin and says so: {rect:?}"
        );
    }

    /// ★★★ **The verb reaches the document, and the disclosure follows the
    /// DIRECTION of the change.**
    ///
    /// The only test in this module that calls [`set`] — everything above it
    /// exercises the arithmetic the window reads, and none of it would notice
    /// an engine call that was never made or a disclosure wired to the wrong
    /// field.
    ///
    /// Both directions, in one document, because that is what makes it a
    /// measurement rather than a coincidence:
    ///
    /// * **A4 → A1** grows the sheet. Nothing can fall off, so `lost_area` must
    ///   be false and the operator must be told **nothing** — a window that
    ///   warned about losing content every time it was used would be a window
    ///   nobody reads.
    /// * **A1 → A5** shrinks it, and the lost-area sentence must appear.
    ///
    /// A build with the two arms swapped, or with the disclosure raised
    /// unconditionally, passes neither half. A build that never calls the
    /// engine passes neither, because the assertion is on `session.pages()`.
    ///
    /// ⚠ R1 still applies: this is not a report of working software. It cannot
    /// see the ribbon, the window, the operand rule or the save. That is
    /// `ui-verify`'s `resizing_a_sheet_changes_the_paper_in_the_saved_file`,
    /// whose verdict is taken in a different process from a written file.
    #[test]
    fn the_verb_reaches_the_document_and_only_shrinking_is_disclosed() {
        let (doc, _pages) = crate::app::blank::document().expect("the template parses");
        let mut session = EditSession::new(doc);

        let a1 =
            pdfcer_core::paper::PaperSize::A1.rect_with(pdfcer_core::paper::Orientation::Portrait);
        let notes = set(&mut session, &[0], a1).expect("a blank page can be resized");
        let media = session.pages().expect("the page tree walks")[0].media_box;
        assert!(
            (media.width() - a1.width()).abs() < 0.01
                && (media.height() - a1.height()).abs() < 0.01,
            "the page must BE A1 afterwards, not merely have been asked to be: {media:?}"
        );
        assert!(
            notes.is_empty(),
            "growing a sheet loses nothing and owes the operator no sentence: {notes:?}"
        );

        let a5 =
            pdfcer_core::paper::PaperSize::A5.rect_with(pdfcer_core::paper::Orientation::Portrait);
        let notes = set(&mut session, &[0], a5).expect("and shrunk again");
        let media = session.pages().expect("the page tree walks")[0].media_box;
        assert!(
            (media.width() - a5.width()).abs() < 0.01,
            "the second change must land too: {media:?}"
        );
        assert!(
            notes.iter().any(|n| n.contains("lost area")),
            "shrinking a sheet must raise the lost-area disclosure: {notes:?}"
        );
    }

    /// ★★ **A certified document is refused, by name, with nothing written.**
    ///
    /// Measured against the engine on 2026-09-06 (`fixtures/certified-comments.pdf`
    /// → `CertificationForbidsChange`), and asserted here as the *shape* the
    /// shell relies on: [`set`] propagates the refusal rather than swallowing
    /// it, so `vector_edit` can trace it and
    /// [`crate::text::page_size::refused_certified`] can word it.
    ///
    /// ⓘ The blank template carries no certification, so what this can assert
    /// without a certified fixture in this crate is the **other** refusal on
    /// the same path: a degenerate rectangle, raised by `normalize_media_box`
    /// **before anything is touched**. The property is the same one and it is
    /// the one that matters — a refusal leaves the document exactly as it was,
    /// rather than half-resized.
    #[test]
    fn a_refused_change_leaves_the_document_exactly_as_it_was() {
        let (doc, _pages) = crate::app::blank::document().expect("the template parses");
        let mut session = EditSession::new(doc);
        let before = session.pages().expect("the page tree walks")[0].media_box;

        let degenerate = Rect::from_corners(0.0, 0.0, 0.0, 500.0);
        let refusal = set(&mut session, &[0], degenerate);
        assert!(refusal.is_err(), "a zero-area sheet must be refused");

        let after = session.pages().expect("the page tree walks")[0].media_box;
        assert_eq!(
            before, after,
            "a refused change must leave the page untouched, not half-resized"
        );
        assert!(
            !session.is_modified(),
            "and must record nothing on the undo stack"
        );
    }

    /// ★ **The ordinary sheet is byte-identical to the engine's own table.**
    ///
    /// What makes [`SheetSurvey::target_rect`]'s corner rule safe to apply
    /// unconditionally: on a page at `(0, 0)` — every CAD export in his corpus
    /// — it produces exactly what `PaperSize::rect_with` produces, so the
    /// offset case costs the common case nothing.
    #[test]
    fn an_origin_anchored_sheet_is_exactly_the_engines_rectangle() {
        let survey = survey_of(vec![Rect::from_corners(0.0, 0.0, 2383.94, 1683.78)]);
        let engine =
            pdfcer_core::paper::PaperSize::A4.rect_with(pdfcer_core::paper::Orientation::Portrait);
        let ours = survey.target_rect(engine.width(), engine.height());
        assert!((ours.llx - engine.llx).abs() < f64::EPSILON, "{ours:?}");
        assert!((ours.lly - engine.lly).abs() < f64::EPSILON, "{ours:?}");
        assert!((ours.urx - engine.urx).abs() < f64::EPSILON, "{ours:?}");
        assert!((ours.ury - engine.ury).abs() < f64::EPSILON, "{ours:?}");
    }
}
