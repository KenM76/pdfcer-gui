//! # `app::status::notes` — the narrator, demoted behind a disclosure triangle
//!
//! The status bar's left-most line: *what pdfcer had to substitute or leave out
//! when it drew this page*. Split out of [`crate::app::status`] under standing
//! rule **R2** (no `.rs` file over 1,500 lines), and the seam is the one the
//! parent's own header already draws in prose:
//!
//! > The left half carries four things, and only the first is the narrator.
//! > The others look similar and are governed by different rules.
//!
//! - Everything left in the parent answers *"how is the bar laid out, and what
//!   does each group show?"* — a fixed row, two rule-4 disclosure lines, a
//!   worded decline, and two clusters of stateless controls.
//! - Everything here answers *"what did the renderer have to compromise on,
//!   and how is that said in one line?"* — which is a different question with
//!   its own widget state (an open/closed flag in `egui::Memory`), its own
//!   pure decision function ([`notes_line`]), and its own editorial rule about
//!   which of the renderer's counters an operator can act on.
//!
//! ## ★ Why this line is *narration* and the three below it are not
//!
//! `DEFECTS.md`'s "Not defects" table records the old shell opening with a
//! substitute-glyph census:
//!
//! > The first thing a user reads is the app talking about itself. Excellent
//! > information, wrong prominence — put it behind the disclosure triangle
//! > that is already there.
//!
//! So the report is complete, still here, and **closed by default**. The words
//! "Render notes" stay visible so it is discoverable; only the report itself
//! is one click away.
//!
//! That prominence argument is exactly what does **not** apply to the lines
//! beside it. A rule-4 disclosure and a worded decline are facts about the
//! operator's own document and their own gesture, and a disclosure the
//! operator has to *open something* to find is a disclosure that did not
//! happen — the opposite failure. Hence: this one is demoted, those are not.
//!
//! ## ★ Opening it does not make the bar taller (R128)
//!
//! The parent's header carries the measurement — a content-driven status
//! panel takes space from the central panel, an active `FitMode` recomputes
//! its zoom from the canvas viewport every frame, and the result on pdfcer was
//! a page that shrank 230 % → 224 % → 215 % across three frames with no zoom
//! input. This module is the half of that defence that lives in the widget:
//! the line is drawn **beside** the triangle, inside the parent's single
//! allocated row, elided at [`super::NOTES_WIDTH_FRACTION`] of the bar with
//! the whole text on hover.
//!
//! It is also why there is no [`egui::CollapsingHeader`] anywhere in this
//! file. Changing its own height is that widget's entire behaviour, which is
//! the one thing this surface may not do.
//! [`super::tests::the_bar_is_exactly_as_tall_open_as_closed`] pins it, from
//! the parent, where the whole bar can be measured at once.

use egui::{Align, Id, Layout, Vec2};

use crate::app::state::OpenDoc;
use crate::text::status as t;

use super::{NOTES_WIDTH_FRACTION, ROW_HEIGHT_PTS};

/// Named region: the disclosure triangle, plus its one line of render notes
/// when open.
///
/// Matched literally by `tools/ui-verify`, so renaming it silently un-aims
/// whatever check was measuring it.
const REGION_NOTES: &str = "status-group:notes"; // ui-text-exempt: trace region name, never displayed

/// Whether the render-notes disclosure is open.
///
/// `pub(super)` because the parent's R128 test drives the flag directly — it
/// measures the bar open and closed, and it can only do that by writing the
/// same key this module reads.
pub(super) const NOTES_OPEN_ID: &str = "pdfcer-status-notes-open"; // ui-text-exempt: widget id, never displayed

/// The render-notes disclosure, and its one line when open.
///
/// Drawn only when a page has actually been rasterized: the notes describe a
/// raster, and `page_texture` is `None` only before the first render and
/// after a failure the canvas already reports in words.
///
/// **Opening this does not make the bar taller.** The line is drawn beside
/// the triangle, inside the same row, elided at [`NOTES_WIDTH_FRACTION`] of
/// the bar with the whole text on hover. See the ★ R128 section of this
/// module's header for why that is a requirement rather than a layout
/// preference.
pub(super) fn show(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(texture) = doc.page_texture.as_ref() else {
        return;
    };
    let id = Id::new(NOTES_OPEN_ID);
    let mut open = ui
        .ctx()
        .data_mut(|d| d.get_temp::<bool>(id))
        .unwrap_or(false);

    let rect = ui
        .scope(|ui| {
            let toggle = ui
                .selectable_label(open, t::diagnostics_toggle(open))
                .on_hover_text(t::diagnostics_tooltip());
            if toggle.clicked() {
                open = !open;
            }
            if !open {
                return;
            }
            let line = notes_line(&texture.diagnostics);
            // A bounded sub-region, so a page with eight findings cannot
            // squeeze the navigation controls off the right of the bar.
            let width = (ui.available_width() * NOTES_WIDTH_FRACTION).max(0.0);
            ui.allocate_ui_with_layout(
                Vec2::new(width, ROW_HEIGHT_PTS),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.add(
                        egui::Label::new(egui::RichText::new(&line).small().weak())
                            // Elide rather than wrap: wrapping is how a
                            // one-row bar becomes a two-row bar, which is
                            // the R128 loop with extra steps.
                            .truncate(),
                    )
                    .on_hover_text(line.clone());
                },
            );
        })
        .response
        .rect;

    crate::diag::ui_rect(REGION_NOTES, rect);
    ui.ctx().data_mut(|d| d.insert_temp(id, open));
}

/// One count from the renderer's report, paired with the catalog entry that
/// puts it into words.
///
/// A named type rather than an inline tuple so [`findings`]' table reads
/// as a table. The `fn(usize) -> String` half is a plain function pointer
/// rather than a closure on purpose: it names a *catalog entry*, so the
/// pairing is a lookup that can be read down the page, and a reviewer
/// checking that every reported field has a sentence has one place to look.
type NoteEntry = (usize, fn(usize) -> String);

/// Turn the renderer's honesty report into the one line the disclosure shows.
///
/// The **join and the empty case**, and nothing else. Which findings exist, in
/// what order, and which two counters are excluded are all [`findings`]' — see
/// there. This function's whole subject is that the bar gets *one* line
/// (**R128**) and that an empty one would be indistinguishable from a
/// disclosure that failed to fill itself.
fn notes_line(d: &pdfcer_render::Diagnostics) -> String {
    let parts = findings(d);
    if parts.is_empty() {
        // Stated positively. An empty disclosure is indistinguishable from
        // one that failed to fill itself, and the operator who opened it
        // wanted an answer either way.
        t::diagnostics_clean().to_owned()
    } else {
        t::diagnostics_join(&parts)
    }
}

/// **The renderer's report as an ordered list of sentences**, one per finding
/// that actually occurred.
///
/// ★ Split out of [`notes_line`] on 2026-08-15 so that the status bar's one
/// line and the Render-diagnostics dialog's list are the *same nine decisions*
/// — which counters are reported, which two are not, and in what order — made
/// once. Two tables would agree on the day they were written and disagree the
/// first time a tenth counter was added to one of them, and the symptom would
/// be two surfaces describing one raster differently, which is the worst
/// available outcome for a *diagnostic*.
///
/// Empty is a real answer and means the page drew clean. Callers word that
/// themselves, because the bar and the dialog have different room for it.
///
/// # What is reported, and what is deliberately not
///
/// Every field here changes **what the operator can see on the page**: text
/// that was not drawn, images that were not drawn, glyphs whose shapes are
/// not the document's, layers that were hidden, content the file does not
/// actually contain. Those are facts an operator can act on — supply a font,
/// turn a layer back on, go and find the missing stream.
///
/// `Diagnostics::tolerated` and `Diagnostics::compat_skipped` are **not**
/// reported. Both count divergences that leave the picture correct: a
/// tolerated structural oddity (an unbalanced `Q`, a mid-path `cm`) is
/// something the renderer absorbed and drew right anyway, and a `BX`/`EX`
/// skip is spec-sanctioned (§7.8.2 Table 32) — the file is *telling* readers
/// to skip it. Listing them **here** would put two numbers that mean "nothing
/// is wrong" in front of the six that mean something is.
///
/// ★ They are not lost: the Render-diagnostics dialog shows them, separately
/// and with a sentence saying they are not faults
/// ([`crate::text::diagnostics::absorbed`]). That is the distinction this
/// exclusion has always rested on — *"this is a status bar, not a report"* —
/// finally having a report to be distinguished from.
///
/// # Order
///
/// Most consequential first: content the file is missing, then whole
/// surfaces that were not drawn, then glyph-level substitution, then the
/// operator's own hidden layers, then operators pdfcer has not implemented. A
/// line that opens with "3 unrecognised drawing operators" and buries "text
/// from 2 fonts not drawn" is sorted by the renderer's interest rather than
/// by the reader's. The dialog lists them top-down in the same order, for the
/// same reason.
pub(crate) fn findings(d: &pdfcer_render::Diagnostics) -> Vec<String> {
    let entries: [NoteEntry; 9] = [
        (
            d.contents_streams_unresolved,
            t::diagnostics_contents_missing,
        ),
        (d.fonts_unsupported, t::diagnostics_fonts_skipped),
        (d.images_unsupported, t::diagnostics_images_skipped),
        (d.glyphs_notdef, t::diagnostics_glyphs_notdef),
        (d.glyphs_substituted, t::diagnostics_glyphs_substituted),
        (d.glyphs_supplied, t::diagnostics_glyphs_supplied),
        (d.oc_sections_hidden, t::diagnostics_layers_hidden),
        (d.deferred_ops, t::diagnostics_ops_deferred),
        (d.unknown_ops, t::diagnostics_ops_unknown),
    ];
    entries
        .into_iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, render)| render(n))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A clean render still says something.
    #[test]
    fn a_clean_render_reports_that_it_is_clean() {
        let d = pdfcer_render::Diagnostics::default();
        assert_eq!(notes_line(&d), t::diagnostics_clean());
    }

    /// Every reported field reaches the line, and none of them wraps it.
    ///
    /// The second half is R128 again: the disclosure gets **one** line, so a
    /// page with every finding at once must still produce a single line.
    #[test]
    fn every_reported_finding_reaches_the_one_line() {
        let d = pdfcer_render::Diagnostics {
            contents_streams_unresolved: 1,
            fonts_unsupported: 2,
            images_unsupported: 3,
            glyphs_notdef: 4,
            glyphs_substituted: 5,
            glyphs_supplied: 6,
            oc_sections_hidden: 7,
            deferred_ops: 8,
            unknown_ops: 9,
            ..Default::default()
        };

        let line = notes_line(&d);
        for n in 1..=9 {
            assert!(
                line.contains(&n.to_string()),
                "finding {n} is missing from the line: {line}"
            );
        }
        assert!(!line.contains('\n'), "the disclosure gets one line: {line}");
        assert_ne!(line, t::diagnostics_clean());
    }

    /// The two counters that mean "nothing is wrong" stay out of the line.
    ///
    /// A tolerated structural oddity was absorbed and drawn correctly, and a
    /// `BX`/`EX` skip is the file telling readers to skip it (§7.8.2 Table
    /// 32). Reporting either would put reassurance in front of the findings
    /// that need reading.
    #[test]
    fn tolerated_and_compat_skipped_are_not_reported() {
        let d = pdfcer_render::Diagnostics {
            tolerated: 11,
            compat_skipped: 13,
            ..Default::default()
        };
        assert_eq!(
            notes_line(&d),
            t::diagnostics_clean(),
            "neither counter describes anything the operator can see or act on"
        );
    }
}
