//! # `app::actions::pagesize` — changing the **paper** an open drawing sits on
//!
//! The body of [`PageAction::SetPageSize`], and the pre-commit
//! [`survey`] the sheet-size window reads to tell the operator, *before* he
//! commits, which of two very different things he is about to get.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/pagesize.md`.

use pdfcer_core::edit::{
    CropBoxChange, CropBoxEdit, CropBoxEntry, CropFollow, EditError, EditSession, MediaBoxChange,
    MediaBoxEntry, PageResize,
};
use pdfcer_core::page_tree::Rect;
use pdfcer_core::pageops::{OrientationPolicy, ScaleMode, ScaleReport, ScaleRequest};
use pdfcer_core::vector::VectorObject;

use crate::app::state::OpenDoc;
use crate::text::page_size as t;

/// **What the picked sheets are, and what is drawn on them** — everything the
/// sheet-size window needs to answer *"what will this do to my drawing?"*
/// before the operator commits.
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
    /// Non-zero is the ordinary case for a multi-sheet pick, and it is a
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
    /// Each operand's visible region (its effective crop box) and `/Rotate`,
    /// in `pages` order: what `scale_pages` maps onto the new sheet.
    pub sources: Vec<(Rect, u16)>,
}

impl SheetSurvey {
    /// The one size every operand already is, when they are all the same.
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
    #[must_use]
    pub fn target_rect(&self, w_pt: f64, h_pt: f64) -> Rect {
        let (llx, lly) = self.common_origin.unwrap_or((0.0, 0.0));
        Rect::from_corners(llx, lly, llx + w_pt, lly + h_pt)
    }

    /// **The request that scales the operands onto a `w_pt` × `h_pt` sheet.**
    ///
    /// The window states sizes as the media box is written, and
    /// `ScaleRequest` states them as the page is displayed, so a set whose
    /// every sheet is turned a quarter has its size transposed. A set mixing
    /// turned and unturned sheets cannot be one exact request, so it uses
    /// `OrientationPolicy::Match` and each sheet keeps its own orientation;
    /// [`Self::orientation_mixed`] lets the window say so.
    #[must_use]
    pub fn scale_request(&self, w_pt: f64, h_pt: f64, mode: ScaleMode) -> ScaleRequest {
        let turned = self.turned();
        let request = if turned == self.sources.len() && turned > 0 {
            ScaleRequest::new(h_pt, w_pt)
        } else {
            ScaleRequest::new(w_pt, h_pt)
        };
        let orientation = if self.orientation_mixed() {
            OrientationPolicy::Match
        } else {
            OrientationPolicy::Exact
        };
        request.with_mode(mode).with_orientation(orientation)
    }

    /// Whether the operands mix quarter-turned and unturned sheets.
    #[must_use]
    pub fn orientation_mixed(&self) -> bool {
        let turned = self.turned();
        turned > 0 && turned < self.sources.len()
    }

    fn turned(&self) -> usize {
        self.sources.iter().filter(|(_, r)| r % 180 == 90).count()
    }

    /// The smallest and largest factor `request` scales the operands by, from
    /// the engine's own placement plan. `None` when no sheet can be planned.
    #[must_use]
    pub fn scale_span(&self, request: &ScaleRequest) -> Option<(f64, f64)> {
        self.sources
            .iter()
            .filter_map(|&(r, rot)| pdfcer_core::pageops::scale::plan_placement(r, rot, request))
            .map(|p| p.scale)
            .fold(None, |span, s| {
                Some(span.map_or((s, s), |(lo, hi): (f64, f64)| (lo.min(s), hi.max(s))))
            })
    }
}

/// Read the picked sheets and what is drawn on them.
#[must_use]
pub fn survey(doc: &OpenDoc, pages: &[usize]) -> SheetSurvey {
    let boxes: Vec<Rect> = pages
        .iter()
        .filter_map(|&i| doc.pages.get(i).map(|page| page.media_box))
        .collect();
    let sources: Vec<(Rect, u16)> = pages
        .iter()
        .filter_map(|&i| doc.pages.get(i).map(|page| (page.crop_box, page.rotate)))
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
        // Not `PageObjects::page_bbox`: it counts paths that paint nothing,
        // so a scaled page's `re W n` clip read as drawing (request G109).
        let bounds = provider
            .page_objects()
            .objects
            .iter()
            .filter(|o| !matches!(o, VectorObject::Path(p) if p.style.is_invisible()))
            .fold(pdfcer_core::vector::Bounds::EMPTY, |acc, o| {
                acc.union(o.page_bbox())
            });
        if !bounds.is_empty() {
            drawn = Some(Rect::from_corners(
                bounds.min.x,
                bounds.min.y,
                bounds.max.x,
                bounds.max.y,
            ));
        }
        // Counted as measured even when the page draws NOTHING. An empty page
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
        sources,
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
/// * `crop_box_outside` — a `/CropBox` the new sheet no longer contains. The
///   engine reports it after the follow below, so it names only a crop box
///   that was cropped to a region and kept; the visible region is now the
///   smaller of the two.
/// * `size_advisory` — outside Annex C.2's recommended range. Advice, worded as
///   advice; ISO 32000-2 dropped the range entirely.
/// * `entry == InheritedSoOwnEntryRemoved` — the page's own `/MediaBox` was
///   **removed** because an ancestor already said that size, so the page is now
///   sized by inheritance. That changes what a *later* edit does to it, which
///   is exactly the kind of fact nothing else will ever tell him.
///
/// **What is NOT disclosed, because the engine does not report it.**
/// `/BleedBox`, `/TrimBox` and `/ArtBox` are left byte-identical (measured — a
/// `/BleedBox [10 10 1000 1000]` survives a resize to 595 × 842 untouched),
/// and `MediaBoxChange` carries **no field for them**: only
/// `crop_box_outside`. A CAD or press export that carries a bleed box therefore
/// gets one overhang disclosed and three not. Filed for the engine; nothing is
/// faked here in the meantime, because a disclosure this shell computed from a
/// walk the engine did not do would be a fifth source of truth about the same
/// page dictionary.
///
/// # Errors
///
/// The operator's sentence for the engine's refusal ([`refusal`]):
/// `CertificationForbidsChange` is the one an operator will actually meet;
/// `MediaBoxDegenerate` is unreachable from the window, which bounds its own
/// custom fields.
///
/// [`PageAction::SetPageSize`]: super::pages::PageAction::SetPageSize
pub(super) fn set(
    session: &mut EditSession,
    pages: &[usize],
    rect: Rect,
) -> Result<Vec<String>, String> {
    // `WhenItMatched`: a crop box that showed the whole old sheet becomes the
    // new sheet, so a resize grows what is seen — the Word, Acrobat and CAD
    // shape. One cropped to a smaller region keeps its crop, and says so below.
    let resized = session
        .resize_pages(pages, rect, CropFollow::WhenItMatched)
        .map_err(refusal)?;
    let followed = resized
        .iter()
        .filter(|&&PageResize { crop, .. }| crop.is_some())
        .count();
    let changes: Vec<MediaBoxChange> = resized
        .iter()
        .map(|&PageResize { media, .. }| media)
        .collect();

    // Traced from the SESSION's own page tree, re-walked after the commit —
    // not from `rect`, and not from `change.after`.
    //
    // `crate::app::blank`'s `document_sized` makes the identical argument for
    // the identical reason, and `ui-verify` reads that line for it: *a trace of
    // the request says what this function was told; a trace of the page tree
    // says what a reader of the resulting file will see.* A build that recorded
    // the request and dropped the write would have a perfect `w=`/`h=` here and
    // an unchanged document — which is the whole class of defect this project
    // is named after.
    let after = trace_sheets(session, pages);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "page-size-applied n={} asked_w={:.2} asked_h={:.2} lost_area={} crop_outside={} \
             advisories={} explicit={} inherited_removed={} base_kept={} crop_followed={followed}",
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

    let mut notes = disclosures(&changes);
    let hidden = pages
        .iter()
        .filter(|&&i| {
            after
                .get(i)
                .is_some_and(pdfcer_gui_base::pagebox::crop_hides_sheet)
        })
        .count();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("page-size-crop-hides n={hidden}")
    });
    if hidden > 0 {
        notes.push(t::disclosure_crop_inside(hidden));
    }
    Ok(notes)
}

/// The body of `PageAction::SetCropBox`: set or remove the visible area of
/// `pages` as one `CommandKind::SetCropBoxes` undo step, returning the
/// off-canvas sentences its outcome owes (R8b).
///
/// # Errors
///
/// `EditError::CropBoxEmpty` for a rectangle that leaves a sheet nothing, raised
/// before any sheet is touched.
pub fn crop(
    session: &mut EditSession,
    pages: &[usize],
    edit: CropBoxEdit,
) -> Result<Vec<String>, EditError> {
    let changes = session.set_crop_boxes(pages, edit)?;
    let overhang = changes
        .iter()
        .filter(
            |CropBoxChange {
                 overhangs_media_box,
                 ..
             }| *overhangs_media_box,
        )
        .count();
    let entries = |want: CropBoxEntry| {
        changes
            .iter()
            .filter(|CropBoxChange { entry, .. }| *entry == want)
            .count()
    };
    let inherited = entries(CropBoxEntry::InheritedSoOwnEntryRemoved);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "page-crop-applied n={} overhang={overhang} explicit={} inherited_removed={inherited} \
             base_kept={} absent={}",
            changes.len(),
            entries(CropBoxEntry::ExplicitWritten),
            entries(CropBoxEntry::BaseSpellingKept),
            entries(CropBoxEntry::Absent),
        )
    });
    let mut notes = Vec::new();
    if overhang > 0 {
        notes.push(crate::text::page_crop::disclosure_overhang(overhang));
    }
    if inherited > 0 {
        notes.push(crate::text::page_crop::disclosure_inherited(inherited));
    }
    Ok(notes)
}

/// The body of `PageAction::StampBates`: stamp the labels as one
/// `CommandKind::StampBates` undo step and return the receipt, which names the
/// labels written and the number the next document of a batch starts at.
///
/// # Errors
///
/// The engine's refusals, raised before anything is written.
pub fn bates(
    session: &mut EditSession,
    stamp: &pdfcer_core::bates::BatesStamp,
    first: u64,
) -> Result<Vec<String>, EditError> {
    let pdfcer_core::bates::BatesOutcome {
        pages,
        first_label,
        last_label,
        next,
        ..
    } = session.stamp_bates(stamp, first)?;
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "bates-applied n={} first_label={first_label} last_label={last_label} next={next}",
            pages.len(),
        )
    });
    Ok(vec![crate::text::bates::receipt(
        pages.len(),
        &first_label,
        &last_label,
        next,
    )])
}

/// The body of `PageAction::RemoveBates`: one `CommandKind::RemoveBates` undo
/// step, or none when nothing in scope carries a label, and its receipt.
///
/// # Errors
///
/// The engine's refusals, raised before anything is written.
pub fn remove_bates(
    session: &mut EditSession,
    pages: Option<&[usize]>,
) -> Result<Vec<String>, EditError> {
    let pdfcer_core::bates::BatesRemoval { pages, labels, .. } = session.remove_bates(pages)?;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "bates-removal pages={} labels={}",
            pages.len(),
            labels.len()
        )
    });
    Ok(vec![if labels.is_empty() {
        crate::text::bates::none_removed().to_owned()
    } else {
        crate::text::bates::removed(labels.len(), pages.len())
    }])
}

/// How many changes ended in `want`.
/// Publish `page-size-sheet` for each of `pages` from the session's page tree
/// as it now is, and return that tree.
fn trace_sheets(session: &EditSession, pages: &[usize]) -> Vec<pdfcer_core::page_tree::Page> {
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
    after
}

/// **The body of `PageAction::ScalePages`**: scale the drawing on `pages`
/// onto `request`'s sheet as one undo step, returning the off-canvas
/// sentences its outcome owes (R8b).
///
/// # Errors
///
/// The operator's sentence for the engine's refusal; see [`refusal`].
pub fn scale(
    session: &mut EditSession,
    pages: &[usize],
    request: &ScaleRequest,
) -> Result<Vec<String>, String> {
    let report = session.scale_pages(pages, request).map_err(refusal)?;
    trace_sheets(session, pages);
    let hidden = fill_overflow(&report);
    let (lo, hi) = report
        .pages
        .iter()
        .map(|p| p.placement.scale)
        .fold((f64::INFINITY, 0.0_f64), |(lo, hi), s| {
            (lo.min(s), hi.max(s))
        });
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "page-scale-applied n={} mode={:?} scale_min={lo:.4} scale_max={hi:.4} flipped={} annotations={} measures={} destinations={} geo_unchanged={} overflow={hidden}",
            report.pages.len(),
            request.mode,
            report
                .pages
                .iter()
                .filter(|p| p.placement.orientation_flipped)
                .count(),
            report.pages.iter().map(|p| p.annotations).sum::<usize>(),
            report.pages.iter().map(|p| p.measures).sum::<usize>(),
            report.destinations,
            report.geo_measures_unchanged,
        )
    });
    let mut notes = Vec::new();
    if hidden > 0 {
        notes.push(t::disclosure_fill_hidden(hidden));
    }
    if report.geo_measures_unchanged > 0 {
        notes.push(t::disclosure_geo_unchanged(report.geo_measures_unchanged));
    }
    Ok(notes)
}

/// How many scaled sheets now carry drawing past their edges: a `Fill` whose
/// scaled source is larger than the target on either axis.
fn fill_overflow(report: &ScaleReport) -> usize {
    const SLACK_PT: f64 = 0.01;
    report
        .pages
        .iter()
        .filter(|p| {
            let s = p.placement.scale;
            let target = p.placement.target;
            s * p.source.width() > target.width() + SLACK_PT
                || s * p.source.height() > target.height() + SLACK_PT
        })
        .count()
}

/// The operator's sentence for a refused page-geometry edit. Pages are
/// numbered from 1 here; the engine's own message numbers them from 0.
fn refusal(err: EditError) -> String {
    match err {
        EditError::ScaleRefusedCeDimensions { page_index, count } => {
            t::refused_ce_dimensions(page_index + 1, count)
        }
        EditError::CertificationForbidsChange { .. } => t::refused_certified().to_owned(),
        EditError::MediaBoxDegenerate { .. } => t::refused_degenerate().to_owned(),
        other => other.to_string(),
    }
}

fn count_entry(changes: &[MediaBoxChange], want: MediaBoxEntry) -> usize {
    changes.iter().filter(|c| c.entry == want).count()
}

/// The rule-4 sentences for a set of [`MediaBoxChange`]s.
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

    // The two Annex C.2 directions are separate sentences, because
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
            sources: boxes.iter().map(|&r| (r, 0)).collect(),
            boxes,
            drawn: None,
            unread: 0,
            common_origin,
        }
    }

    /// **A quarter-turned set has its size transposed; a mixed set matches
    /// each sheet's orientation instead.**
    #[test]
    fn the_scale_request_speaks_in_displayed_orientation() {
        let a1 = Rect::from_corners(0.0, 0.0, 2383.94, 1683.78);
        let mut survey = survey_of(vec![a1, a1]);
        let plain = survey.scale_request(595.0, 842.0, ScaleMode::Fit);
        assert_eq!((plain.width, plain.height), (595.0, 842.0));
        assert_eq!(plain.orientation, OrientationPolicy::Exact);

        survey.sources = vec![(a1, 90), (a1, 270)];
        let turned = survey.scale_request(595.0, 842.0, ScaleMode::Fill);
        assert_eq!((turned.width, turned.height), (842.0, 595.0));
        assert_eq!(turned.mode, ScaleMode::Fill);
        assert!(!survey.orientation_mixed());

        survey.sources = vec![(a1, 90), (a1, 0)];
        assert!(survey.orientation_mixed());
        assert_eq!(
            survey
                .scale_request(595.0, 842.0, ScaleMode::Fit)
                .orientation,
            OrientationPolicy::Match
        );
    }

    /// **The pre-commit factor is the engine's: A1 landscape onto A3
    /// landscape is half size, fit or fill.**
    #[test]
    fn the_quoted_factor_is_the_engines_plan() {
        let survey = survey_of(vec![Rect::from_corners(0.0, 0.0, 2383.94, 1683.78)]);
        let request = survey.scale_request(1190.55, 841.89, ScaleMode::Fit);
        let (lo, hi) = survey.scale_span(&request).expect("a real sheet plans");
        assert!(
            (lo - 0.5).abs() < 0.001 && (hi - lo).abs() < 1e-9,
            "{lo} {hi}"
        );
    }

    /// **Scaling lands the new sheet and moves the content; a ce-dimension
    /// refusal is worded with a 1-based page.**
    #[test]
    fn scaling_lands_the_sheet_and_words_its_refusal() {
        let (doc, _pages) = crate::app::blank::document().expect("the template parses");
        let mut session = EditSession::new(doc);
        let before = session.pages().expect("the page tree walks")[0].media_box;
        let half = ScaleRequest::new(before.width() / 2.0, before.height() / 2.0);
        let notes = scale(&mut session, &[0], &half).expect("a blank page scales");
        let media = session.pages().expect("the page tree walks")[0].media_box;
        assert!(
            (media.width() - before.width() / 2.0).abs() < 0.01,
            "{media:?}"
        );
        assert!(
            notes.is_empty(),
            "a same-shape fit hides nothing: {notes:?}"
        );

        let worded = refusal(EditError::ScaleRefusedCeDimensions {
            page_index: 2,
            count: 4,
        });
        assert!(
            worded.starts_with("Page 3 carries 4 ce dimensions"),
            "{worded}"
        );
    }

    /// **The overhang is the operator's own case, in his own numbers.**
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

    /// **"Could not measure" is not "nothing falls off".**
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

    /// **A drawing that fits reports zeros, not `None`.**
    #[test]
    fn a_drawing_that_fits_reports_a_measured_zero() {
        let mut survey = survey_of(vec![Rect::from_corners(0.0, 0.0, 595.0, 842.0)]);
        survey.drawn = Some(Rect::from_corners(50.0, 50.0, 300.0, 400.0));
        assert_eq!(
            survey.overhang(Rect::from_corners(0.0, 0.0, 595.2756, 841.8898)),
            Some((0.0, 0.0, 0.0, 0.0))
        );
    }

    /// **A set is uniform to the producer's rounding, not to the bit.**
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

    /// **A real mixed set is seen as mixed.**
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

    /// **An offset sheet keeps its corner**, and a mixed-corner pick falls
    /// back to the origin.
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

    /// **The verb reaches the document, and the disclosure follows the
    /// DIRECTION of the change.**
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

    /// **A certified document is refused, by name, with nothing written.**
    #[test]
    fn a_refused_change_leaves_the_document_exactly_as_it_was() {
        let (doc, _pages) = crate::app::blank::document().expect("the template parses");
        let mut session = EditSession::new(doc);
        let before = session.pages().expect("the page tree walks")[0].media_box;

        let degenerate = Rect::from_corners(0.0, 0.0, 0.0, 500.0);
        let refusal = set(&mut session, &[0], degenerate);
        assert_eq!(
            refusal.err().as_deref(),
            Some(t::refused_degenerate()),
            "a zero-area sheet must be refused in the operator's words"
        );

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

    /// **The ordinary sheet is byte-identical to the engine's own table.**
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
