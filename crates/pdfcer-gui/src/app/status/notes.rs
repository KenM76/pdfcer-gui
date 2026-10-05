//! # `app::status::notes` — the narrator, demoted behind a disclosure triangle
//!
//!
//! > The left half carries four things, and only the first is the narrator.
//! > The others look similar and are governed by different rules.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/notes.md`.

use egui::{Align, Id, Layout, Vec2};

use crate::app::state::OpenDoc;
use crate::text::status as t;

use super::{NOTES_WIDTH_FRACTION, ROW_HEIGHT_PTS};

mod ink;

/// Named region: the disclosure triangle, plus its one line of render notes
/// when open.
///
/// Matched literally by `tools/ui-verify`, so renaming it silently un-aims
/// whatever check was measuring it.
const REGION_NOTES: &str = "status-group:notes"; // ui-text-exempt: trace region name, never displayed

/// Whether the render-notes disclosure is open.
pub(super) const NOTES_OPEN_ID: &str = "pdfcer-status-notes-open"; // ui-text-exempt: widget id, never displayed

/// The render-notes disclosure, and its one line when open.
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
type NoteEntry = (usize, fn(usize) -> String);

/// Turn the renderer's honesty report into the one line the disclosure shows.
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

/// **Annotations the file carries that the operator was shown nothing for.**
fn annotations_not_drawn(d: &pdfcer_render::Diagnostics) -> usize {
    if d.annotations_out_of_scope > 0 {
        return 0;
    }
    let without_ap: usize = d.annotations_without_ap.values().copied().sum();
    // `saturating_sub` rather than `-`: the subset relation above is the
    // engine's invariant, not this crate's, and a future engine that counted
    // an icon paint without the census entry would otherwise panic a release
    // build's status bar. Zero is the right answer to "how many were invisible"
    // when the two numbers disagree in that direction.
    without_ap.saturating_sub(d.annotations_icon_painted)
}

/// **The renderer's report as an ordered list of sentences**, one per finding
/// that actually occurred.
pub(crate) fn findings(d: &pdfcer_render::Diagnostics) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    // FIRST, and it is not in the table below because it is not a count —
    // it is a `bool`, and it is the CAUSE of several of the lines that can
    // appear under it.
    //
    //
    // ⚠ The reason this is disclosed at all, rather than treated as a silent
    // repair: §7.8.3 lets a form XObject or a Type 3 font omit its own
    // `/Resources` and inherit **the page's**, and the ISO 32000-2 erratum
    // extends that to **annotation appearance streams**. A page whose only
    // marks are annotations — exactly the operator's file's shape — can
    // therefore genuinely need the dictionary that was not there. So a font
    // reported missing on the lines below may be missing BECAUSE of this line,
    // and reading them the other way round sends him looking for the font.
    if d.page_resources_defaulted {
        out.push(t::diagnostics_resources_defaulted().to_owned());
    }

    let entries: [NoteEntry; 10] = [
        (
            d.contents_streams_unresolved,
            t::diagnostics_contents_missing,
        ),
        // Second, above every other absence, and the order is the argument.
        // A missing font leaves a hole an operator can SEE; an annotation with
        // no appearance leaves clean paper, and clean paper is what a drawing
        // nobody commented on looks like. It is the one finding in this table
        // the operator cannot discover by looking at the page.
        (
            annotations_not_drawn(d),
            t::diagnostics_annots_no_appearance,
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
    out.extend(
        entries
            .into_iter()
            .filter(|(n, _)| *n > 0)
            .map(|(n, render)| render(n)),
    );
    out.extend(ink::findings(d));
    out
}

/// The colour counters as `key=count` pairs, for the trace.
pub(crate) fn ink_trace(d: &pdfcer_render::Diagnostics) -> String {
    ink::trace_pairs(d)
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

    /// **A page pdfcer had to supply a resource dictionary for says so,
    /// and says it FIRST.**
    #[test]
    fn a_page_whose_resources_were_supplied_says_so_first() {
        let d = pdfcer_render::Diagnostics {
            page_resources_defaulted: true,
            fonts_unsupported: 1,
            ..Default::default()
        };

        let found = findings(&d);
        assert_eq!(
            found.len(),
            2,
            "the supplied dictionary is a finding in its own right, not a qualifier on the font \
             line: {found:?}"
        );
        assert_eq!(
            found[0],
            t::diagnostics_resources_defaulted(),
            "it must be read before anything it may have CAUSED: {found:?}"
        );
    }

    /// The other direction: a page that named its own resources is silent.
    #[test]
    fn a_page_that_names_its_own_resources_is_not_mentioned() {
        let d = pdfcer_render::Diagnostics::default();
        assert!(
            findings(&d).is_empty(),
            "a clean page has no findings at all"
        );
    }

    /// **An annotation the file carries and pdfcer drew nothing for is
    /// reported** — the finding an operator cannot make for himself.
    #[test]
    fn appearance_less_annotations_are_counted_as_not_drawn() {
        let mut d = pdfcer_render::Diagnostics::default();
        d.annotations_without_ap.insert("Square".to_owned(), 2);

        assert_eq!(annotations_not_drawn(&d), 2);
        let line = notes_line(&d);
        assert!(
            line.contains('2') && line.contains("not drawn"),
            "the appearance-less pair must reach the line: {line}"
        );
    }

    /// **The icon painter's rescues are subtracted, not counted twice.**
    #[test]
    fn an_icon_pdfcer_painted_is_not_reported_as_missing() {
        let mut d = pdfcer_render::Diagnostics::default();
        d.annotations_without_ap.insert("Text".to_owned(), 2);
        d.annotations_without_ap.insert("Square".to_owned(), 1);
        d.annotations_icon_painted = 2;

        assert_eq!(
            annotations_not_drawn(&d),
            1,
            "only the /Square is invisible; both sticky notes were painted"
        );
    }

    /// **Every appearance-less annotation was drawn ⇒ say nothing at all.**
    #[test]
    fn a_page_whose_icons_were_all_painted_reports_clean() {
        let mut d = pdfcer_render::Diagnostics::default();
        d.annotations_without_ap.insert("Text".to_owned(), 3);
        d.annotations_icon_painted = 3;

        assert_eq!(annotations_not_drawn(&d), 0);
        assert_eq!(notes_line(&d), t::diagnostics_clean());
    }

    /// **A narrowed scope suppresses the finding rather than shrinking it.**
    #[test]
    fn a_narrowed_scope_withholds_the_finding_instead_of_miscounting_it() {
        let mut d = pdfcer_render::Diagnostics::default();
        d.annotations_without_ap.insert("Text".to_owned(), 4);
        d.annotations_out_of_scope = 4;

        assert_eq!(
            annotations_not_drawn(&d),
            0,
            "withheld on request is not the same fact as tried and failed"
        );
        assert_eq!(notes_line(&d), t::diagnostics_clean());
    }

    /// **A count that disagrees with the engine's own subset invariant does not
    /// panic a release build's status bar.**
    #[test]
    fn an_impossible_engine_pair_saturates_rather_than_panicking() {
        let mut d = pdfcer_render::Diagnostics::default();
        d.annotations_without_ap.insert("Text".to_owned(), 1);
        d.annotations_icon_painted = 5;

        assert_eq!(annotations_not_drawn(&d), 0);
    }

    /// Every reported field reaches the line, and none of them wraps it.
    ///
    /// The second half is R128 again: the disclosure gets **one** line, so a
    /// page with every finding at once must still produce a single line.
    #[test]
    fn every_reported_finding_reaches_the_one_line() {
        let mut d = pdfcer_render::Diagnostics {
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
        // The tenth, and it cannot be set in the struct literal above
        // because it is a map rather than a count — which is precisely why it
        // was the one a `[NoteEntry; 9]` table could not hold. Ten so that a
        // finding silently dropped from the table is a MISSING NUMBER rather
        // than a number that happens to collide with a neighbour's.
        d.annotations_without_ap.insert("Square".to_owned(), 10);

        let line = notes_line(&d);
        for n in 1..=10 {
            assert!(
                line.contains(&n.to_string()),
                "finding {n} is missing from the line: {line}"
            );
        }
        assert!(!line.contains('\n'), "the disclosure gets one line: {line}");
        assert_ne!(line, t::diagnostics_clean());
    }

    /// The two counters that mean "nothing is wrong" stay out of the line.
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
