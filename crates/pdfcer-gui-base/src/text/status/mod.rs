//! # `text::status` — every string the status bar shows
//!
//! One area of the catalog described in [`crate::text`]'s header, and the
//! sole consumer is `pdfcer_gui::app::status`. Nothing here is read by the
//! ribbon: the status bar **mirrors** three View-tab commands under
//! amendment P1a (`RIBBON_IA.md` §2), and a mirror is a second surface for
//! one command, not a second command.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/status/mod.md`.

/// **The Render-notes vocabulary** — every sentence the renderer's honesty
/// report can produce.
mod diagnostics;
pub use diagnostics::{
    diagnostics_annots_no_appearance, diagnostics_clean, diagnostics_contents_missing,
    diagnostics_fonts_skipped, diagnostics_glyphs_notdef, diagnostics_glyphs_substituted,
    diagnostics_glyphs_supplied, diagnostics_images_skipped, diagnostics_join,
    diagnostics_layers_hidden, diagnostics_ops_deferred, diagnostics_ops_unknown,
    diagnostics_resources_defaulted, diagnostics_toggle, diagnostics_tooltip,
};

mod formdelete;
mod inknotes;
pub use inknotes::*;
mod refused;
mod selection;

/// Re-exported for the reason stated on [`selection`]'s block below, and it
/// earns a file of its own rather than a place in this catalog because its
/// argument is long: a **decline** that repeats a panel's **standing
/// description** almost word for word has to justify every word it does not
/// share with it, or the two surfaces become two paraphrases of one fact.
pub use formdelete::field_delete_declined_structural;

/// Re-exported on [`formdelete`]'s precedent, with one addition that is this
/// entry's whole difficulty: **it is the only sentence in this catalog
/// that names no cause**, because the engine exposes none this shell may
/// switch on. Its file carries the argument for every word it does not say,
/// including why it does not point the operator at Render diagnostics.
pub use refused::{edit_declined_by_engine, session_busy};
/// **Resize refusals** — the un-rebuildable appearance and the fixed-size
/// marker.
mod resize;
pub use resize::{resize_fixed_size_marker, resize_not_rebuildable};

/// Re-exported rather than moved-and-repathed.
pub mod waiting;
pub use waiting::{
    line_weights_no_effect, line_weights_off, page_catching_up, tiny_details_skipped,
};

pub use selection::{
    // A type rather than a flat function, because the refusal has more than
    // one shape and a caller must say which it means. No flat alias is exported
    // beside it: an alias would let a call site ask for a sentence that is
    // wrong about a shipped capability, by name, and compile.
    InsideFormRefusal,
    TextStyleRefusal,
    inside_container,
    selection_many,
    selection_one,
    selection_one_in_form,
    selection_one_in_form_unsized,
    selection_one_unsized,
    // The Part rung's four — O188(A). Two clauses and two hovers, and they
    // come in pairs because the clause is the readout and the hover is the
    // verbs; a call site that took one without the other would be stating a
    // rung with no way out of it.
    selection_part_of_path,
    selection_part_of_path_hint,
    selection_part_of_text,
    selection_part_of_text_hint,
    selection_with_depth,
    text_render_mode_invisible,
    text_run_width_held,
    // The six ladder sentences, one per outcome the engine can report
    // (`StyleLadder::rung`, split at rung 1 by `StyleLadder::same_family`),
    // plus the `Warn`-posture one.
    //
    // There is deliberately no single sentence naming both `/BaseFont`s: a
    // substitution within the family and one across families are different
    // outcomes, the engine distinguishes them, and one sentence covering both
    // would tell the operator less than the engine already knows.
    text_style_already_that_way,
    text_style_already_without,
    text_style_faked,
    text_style_faked_warning,
    text_style_multi,
    text_style_off_face,
    text_style_off_unsynthesised,
    text_style_used_other_family,
    text_style_used_sibling_face,
    text_style_used_standard_face,
    // O69's sibling of `too_many_anchors` below. It lives in `selection`
    // rather than in this file because this file is at 1,482 lines against
    // R2's 1,500, and it is re-exported here so a caller says
    // `text::status::too_many_anchors_in_part` beside
    // `text::status::too_many_anchors` — two sentences about one limit, named
    // the same way, which is what stops the second being missed.
    too_many_anchors_in_part,
};

// ---------------------------------------------------------------------------
// The edit disclosure — rule 4's surviving half for the vector verbs
//
// Almost every word an operator reads here was written by `pdfcer-core`,
// and that is the point of this section rather than a shortcut through it.
//
// `EditSession`'s vector verbs return `Result<Vec<String>, EditError>`, and
// the `Vec<String>` is a **disclosure list**: sentences the surgery owes the
// operator because it had to change an operator's *form* to express the
// request — an `re` rectangle rewritten as four lines so one corner could
// move, an implicitly-started subpath's `m` materialised, a curve dropped
// with the point it ran into. They are already finished English prose,
// written where the fact is known, and they are passed through **verbatim**.
//
// So what belongs in this catalog is only the *framing this shell adds*:
// the warning mark, the lead-in that ties the sentences to the gesture that
// produced them, and the separator between two of them. Re-wording core's
// sentences here would put two descriptions of one surgery in the product,
// and the one further from the code would be the one on screen.
// ---------------------------------------------------------------------------

/// One line for the disclosures the last vector edit returned.
#[must_use]
pub fn edit_disclosure_line(notes: &[String]) -> String {
    format!("⚑ About your last edit: {}", notes.join(" "))
}

// ---------------------------------------------------------------------------
// The worded decline — a framing zoom that had nothing to frame
//
// A DECLINE IS NOT A DISCLOSURE, AND THE COPY HAS TO SAY SO
//
// The section above frames sentences `pdfcer-core` wrote about work that
// *happened*: a rectangle really was rewritten as four lines, and the operator
// is owed the part they cannot see. These two strings are the opposite speech
// act. Nothing happened. The command was invoked, it looked at what it had to
// work with, and it declined.
//
// One slot and one wording for both would make a completed gesture and a
// refused one wear the same sentence in the same place, which is worse than
// the trace-only state these strings replace — an operator who reads
// "About your last edit" after a gesture that did nothing has been told a
// small lie confidently. So the lead-in diverges (*"Nothing to zoom to"*, not
// *"About your last edit"*) and the mark diverges with it: `⊗` (U+2297) rather
// than `⚑` (U+2691).
//
// `⊗` was chosen because it reads as *"this did not happen"* rather than as
// *"look at this"*, and because it is drawable. That second half is measured,
// not assumed: `crate::text::glyphs`' header records which codepoints egui's
// bundled proportional chain (Ubuntu-Light → NotoEmoji-Regular →
// emoji-icon-font) actually supplies, and `⊗` is among those confirmed
// present. It is checked twice on every run anyway — by
// `pdfcer_gui::app::status::tests::every_glyph_the_status_bar_draws_has_a_glyph`,
// which lists this bar's labels by hand, and by
// `crate::text::glyphs::tests::every_glyph_the_catalog_draws_has_a_glyph`,
// which reads every literal in this directory from source. A tofu box on a
// decline would read as a rendering failure, which is exactly how an operator
// decides a surface is broken and stops reading it.
//
// # What is deliberately NOT worded here
//
// **The raster-ceiling-clamped region zoom.** A framing zoom that asked for
// more magnification than the page's raster allows still zooms, still centres
// what was asked for, and raises `Action::ZoomTo` carrying the **clamped**
// scale — so the zoom readout three controls to the right states the scale
// actually pinned, on the same frame. That is a partial grant that already
// reports itself, and a sentence saying so would word a non-event. See
// `pdfcer_gui::canvas::zoom::ZoomOutcome::ceiling_changed_the_answer`, which
// carries the argument in full.
//
// # No trailing full stop
//
// These sit in the same slot as `page_clamped_note` and `page_rejected_note`
// — a short note beside a control, read at a glance in a small weak face —
// and they follow that precedent rather than the prose one. The tooltips in
// this file are prose and end in a full stop; these are notes and do not.
// ---------------------------------------------------------------------------

/// Shown when zoom-to-selection was invoked with nothing it could frame.
#[must_use]
pub fn zoom_declined_no_selection() -> &'static str {
    "⊗ Nothing to zoom to — nothing on this page is selected right now"
}

/// Shown when a framing zoom was invoked before the canvas had drawn a page.
#[must_use]
pub fn zoom_declined_not_drawn() -> &'static str {
    "⊗ Nothing to zoom to yet — the page has not finished drawing"
}

/// Shown when `file.save_copy` asked where to write, was told, and could not.
#[must_use]
pub fn save_copy_failed() -> &'static str {
    "⊗ The copy was not written — check that the folder exists and can be written to"
}

/// Shown when the Settings window's Save reached no disk.
#[must_use]
pub fn settings_not_saved() -> &'static str {
    "⊗ Your choices are in use now but could not be written down — they will be \
     gone when pdfcer restarts"
}

/// Shown when `edit.undo` was invoked and the command log was empty.
#[must_use]
pub fn undo_declined_empty() -> &'static str {
    "⊗ Nothing to undo — this document has no changes to take back"
}

/// Shown when `edit.redo` was invoked and the redo stack was empty.
#[must_use]
pub fn redo_declined_empty() -> &'static str {
    "⊗ Nothing to redo — nothing has been undone, or a new change replaced it"
}

// ---------------------------------------------------------------------------
// Zoom
// ---------------------------------------------------------------------------

/// The zoom-out button's label — `−` (U+2212 MINUS SIGN).
///
/// Not the ASCII hyphen. A hyphen next to a `+` reads as a dash rather than
/// as an operator, and the two controls are meant to be seen as a pair.
#[must_use]
pub fn zoom_out() -> &'static str {
    "−"
}

/// Hover text for zoom out.
#[must_use]
pub fn zoom_out_tooltip() -> &'static str {
    "Zoom out one step (Ctrl+Minus)."
}

/// The zoom-in button's label.
#[must_use]
pub fn zoom_in() -> &'static str {
    "+"
}

/// Hover text for zoom in.
#[must_use]
pub fn zoom_in_tooltip() -> &'static str {
    "Zoom in one step (Ctrl+Plus)."
}

/// The current zoom, as a whole percentage.
#[must_use]
pub fn zoom_percent(percent: f64) -> String {
    // `{:.0}` rather than an integer cast — O24j. The value now spans 10 % to
    // a trillion percent and no integer type covers it without either
    // saturating or being wider than the thing it describes. Rounding at the
    // formatter keeps the readout an exact whole number of percent at every
    // magnitude, which is what it always showed.
    format!("{percent:.0}%")
}

/// Hover text for the zoom readout.
#[must_use]
pub fn zoom_percent_tooltip() -> &'static str {
    "The current zoom. The − and + buttons step a fixed ladder of familiar \
     percentages, so zooming in and back out returns to exactly where you \
     started."
}

// ---------------------------------------------------------------------------
// Fit — the three View-tab mirrors (amendment P1a)
// ---------------------------------------------------------------------------

/// The Actual size button's label.
#[must_use]
pub fn fit_actual_size() -> &'static str {
    "Actual size"
}

/// Hover text for Actual size.
#[must_use]
pub fn fit_actual_size_tooltip() -> &'static str {
    "Show the page at actual size — one PDF point per screen point (Ctrl+0)."
}

/// The Fit width button's label.
#[must_use]
pub fn fit_width() -> &'static str {
    "Fit width"
}

/// Hover text for Fit width.
#[must_use]
pub fn fit_width_tooltip() -> &'static str {
    "Scale the page so its full width is visible, and keep it fitted as the \
     window resizes."
}

/// The Fit page button's label.
#[must_use]
pub fn fit_page() -> &'static str {
    "Fit page"
}

/// Hover text for Fit page.
#[must_use]
pub fn fit_page_tooltip() -> &'static str {
    "Scale the page so all of it is visible, and keep it fitted as the \
     window resizes."
}

/// The Fit height button's label.
#[must_use]
pub fn fit_height() -> &'static str {
    "Fit height"
}

/// Hover text for Fit height.
#[must_use]
pub fn fit_height_tooltip() -> &'static str {
    "Scale the page so its full height is visible, and keep it fitted as the window resizes."
}

// ---------------------------------------------------------------------------
// Page navigation, and the editable page box
// ---------------------------------------------------------------------------

/// The wheel-paging toggle's label — `OPERATOR_REQUESTS.md` O30.
#[must_use]
pub fn wheel_flip_pages() -> &'static str {
    "Flip pages"
}

/// Hover text for the wheel-paging toggle.
#[must_use]
pub fn wheel_flip_pages_tooltip() -> &'static str {
    "Turn the mouse wheel into a page turn: one notch, one sheet. Switch it off and the wheel scrolls within the page instead. Ctrl+wheel always zooms, and a continuous page display always scrolls."
}

/// The previous-page button's label — `⏴` (U+23F4).
#[must_use]
pub fn prev_page() -> &'static str {
    "⏴"
}

/// Hover text for the previous-page button.
#[must_use]
pub fn prev_page_tooltip() -> &'static str {
    "Previous page (Page Up)."
}

/// The next-page button's label — `⏵` (U+23F5). See [`prev_page`].
#[must_use]
pub fn next_page() -> &'static str {
    "⏵"
}

/// Hover text for the next-page button.
#[must_use]
pub fn next_page_tooltip() -> &'static str {
    "Next page (Page Down)."
}

/// The page number, as the editable box shows it.
#[must_use]
pub fn page_number(page_1_based: usize) -> String {
    format!("{page_1_based}")
}

/// The total, shown to the right of the editable box.
#[must_use]
pub fn page_of_total(total: usize) -> String {
    format!("/ {total}")
}

/// The position shown right of the box when the box shows a page label.
#[must_use]
pub fn page_of_total_labelled(page_1_based: usize, total: usize) -> String {
    format!("({page_1_based} / {total})")
}

/// Hover text for the page box on a document with page labels.
#[must_use]
pub fn page_box_tooltip_labelled() -> &'static str {
    "This document labels its pages; the box shows the label. Type a label, \
     or a page's position counted from 1, and press Enter."
}

/// Hover text for the editable page box.
#[must_use]
pub fn page_box_tooltip() -> &'static str {
    "Type a page number and press Enter. Nothing moves while you type, and \
     a number past the end of the document goes to the nearest page and \
     says so."
}

/// Shown beside the box when a committed number was outside the document.
#[must_use]
pub fn page_clamped_note(asked: usize, landed: usize, total: usize) -> String {
    format!("No page {asked} — went to {landed} of {total}")
}

/// Shown beside the box when the committed text was neither a page's label
/// nor a number.
#[must_use]
pub fn page_rejected_note_labelled() -> &'static str {
    "No page has that label — type a label or a number, then Enter"
}

/// Shown beside the box when the committed text was not a page number.
#[must_use]
pub fn page_rejected_note() -> &'static str {
    "Not a page number — type digits, then Enter"
}

// --- Registering an unclaimed form control ---------------------------------

/// `adopt_widget` refused: the name is already another field's.
#[must_use]
pub const fn adopt_declined_name_taken() -> &'static str {
    "Another field in this document already uses that name. In a PDF, two fields with the same \
     name are one field with two boxes — filling either would fill both — so pdfcer needs a \
     different name."
}

/// `adopt_widget` refused: the widget carries no name and none was typed.
#[must_use]
pub const fn adopt_declined_no_name() -> &'static str {
    "This box carries no name of its own, so pdfcer has nothing to register it under. Type a name \
     to make it a new, empty field — its original name, type and any value it had are not in \
     this file. To get those back, insert the pages again from the document they came from."
}

/// The disclosure after a widget was registered.
#[must_use]
pub fn adopted(name: &str, typed: bool, acroform_created: bool) -> String {
    let mut line = format!("Registered as \u{201c}{name}\u{201d}.");
    if !typed {
        line.push_str(
            " It has no field type, so no viewer knows how to fill it — pdfcer cannot give it one \
             without the field definition it lost.",
        );
    }
    if acroform_created {
        line.push_str(" This document had no interactive form before; it has one now.");
    }
    line
}
/// **`edit.form_flatten` was invoked on a document whose certification forbids
/// it.**
#[must_use]
pub const fn flatten_declined_certified() -> &'static str {
    "This document is certified, and flattening its fields would break the signature. Filling \
     is still allowed; turning the values into page content is not."
}

/// Shown when the renderer reports `cmyk_buffer_refused` — the page's raster
/// grew past the size the engine will composite in subtractive CMYK, so
/// blending fell back to sRGB and the colours moved.
#[must_use]
pub fn blend_space_status_line() -> String {
    "Colours are approximate at this zoom \u{2014} the page is too large to blend in print \
     colours here. Zoom out to see the exact colours."
        .to_owned()
}

/// Shown when the renderer reports `cmyk_spots_flattened`: `inks` distinct spot
/// inks on the page beyond what it keeps as separate colours.
#[must_use]
pub fn spots_flattened_status_line(inks: u64) -> String {
    let (count, verb) = if inks == 1 {
        ("1 spot colour".to_owned(), "is")
    } else {
        (format!("{inks} spot colours"), "are")
    };
    format!(
        "{count} on this page {verb} shown as ordinary print colours \u{2014} there are more \
         than can be kept separate, so where they overlap or blend the colours are approximate."
    )
}

/// The status-bar line for a document whose index pdfcer had to rebuild.
#[must_use]
pub const fn recovered_status_line() -> &'static str {
    "This file's index was damaged — pdfcer rebuilt it to open the document. The Properties panel says what was recovered."
}

/// **Why zooming in stopped**, on the bottom bar — `OPERATOR_REQUESTS.md`
/// O186's fourth clause, in the operator's own words:
#[must_use]
pub const fn raster_stop_status_line() -> &'static str {
    "Zoom stopped here — this page cannot be drawn any larger. The limit was measured on this sheet; other pages may go further."
}

/// **Show points was switched on and the object has more anchors than the
/// canvas will draw.**
#[must_use]
pub fn too_many_anchors(count: usize, cap: usize) -> String {
    format!(
        "This object has {count} points and pdfcer draws at most {cap} at once, so none are \
         shown. Double-click into a part of it, or use the Points tool, to see that part's."
    )
}

/// **The chunk boxes were asked for and none were drawn** — `OPERATOR_REQUESTS.md`
/// O215, and `pdfcer_gui::canvas::chunks::MAX_CHUNK_BOXES`'s disclosure.
#[must_use]
pub fn too_many_text_chunks(count: usize, cap: usize) -> String {
    format!(
        "What you have selected holds {count} chunks of text and pdfcer outlines at most {cap} \
         at once, so none are shown. Select one text block on its own to see its chunks."
    )
}

/// **The OCR text layer is switched on** — `OPERATOR_REQUESTS.md` O226.
#[must_use]
pub fn ocr_layer_line(percent: u32, blocks: usize) -> String {
    if blocks == 0 {
        "OCR text layer is on, and this page carries no recognised text — there is nothing to \
         draw over it."
            .to_owned()
    } else {
        format!("OCR text layer at {percent}% — {blocks} blocks of recognised text on this page.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spots_flattened_count_agrees_with_its_verb() {
        assert!(spots_flattened_status_line(1).starts_with("1 spot colour on this page is "));
        assert!(spots_flattened_status_line(3).starts_with("3 spot colours on this page are "));
    }

    /// **The absent case is a different sentence, not the same one with a
    /// zero in it.**
    #[test]
    fn a_page_with_no_recognised_text_says_so_instead_of_counting_to_zero() {
        let none = ocr_layer_line(65, 0);
        assert!(
            !none.contains('0'),
            "the empty case must not read as a count: {none}"
        );
        assert!(none.contains("no recognised text"), "{none}");
        assert_ne!(none, ocr_layer_line(65, 412));
    }

    /// The slider position reaches the sentence, because it reaches nowhere
    /// else the operator can read it.
    #[test]
    fn the_blend_is_named_when_there_is_something_to_blend() {
        let line = ocr_layer_line(30, 7);
        assert!(line.contains("30%"), "{line}");
        assert!(line.contains('7'), "{line}");
    }

    /// The disclosure must say which state it is in.
    #[test]
    fn the_disclosure_reads_differently_open_and_closed() {
        assert_ne!(diagnostics_toggle(false), diagnostics_toggle(true));
    }

    /// **`pdfcer-core`'s sentences reach the operator unaltered, and on one
    /// line.**
    #[test]
    fn the_edit_disclosure_frames_cores_sentences_without_altering_them() {
        let notes = vec![
            "This shape was stored as a rectangle, so it has been rewritten as four lines."
                .to_owned(),
            "This point had no coordinates of its own — the file re-used an earlier start."
                .to_owned(),
        ];
        let line = edit_disclosure_line(&notes);

        for note in &notes {
            assert!(
                line.contains(note.as_str()),
                "core's sentence was altered on the way to the bar: {line}"
            );
        }
        assert!(
            !line.contains('\n'),
            "the bar gets one line, and a newline in the string wraps the label \
             regardless of what the layout asks for: {line}"
        );
        assert!(
            line.starts_with('⚑'),
            "the mark is what tells a disclosure apart from the narration beside it: {line}"
        );
    }

    /// **A decline does not read like a disclosure.**
    #[test]
    fn a_decline_does_not_read_like_a_disclosure() {
        let declines = [
            zoom_declined_no_selection(),
            zoom_declined_not_drawn(),
            save_copy_failed(),
        ];
        let disclosure = edit_disclosure_line(&["x".to_owned()]);

        for line in declines {
            assert!(
                line.starts_with('⊗'),
                "a decline carries the mark that says nothing happened: {line}"
            );
            assert!(
                !line.starts_with('⚑'),
                "`⚑` is the disclosure's mark; a decline wearing it is a \
                 refused gesture dressed as a completed one: {line}"
            );
            assert!(
                !line.contains("About your last edit"),
                "a decline must not borrow the disclosure's lead-in — nothing \
                 happened, so there is no last edit to be about: {line}"
            );
            assert_ne!(line, disclosure);
            assert!(!line.contains('\n'), "the bar gets one line: {line}");
        }
        // Pairwise rather than one comparison, because the operator gets ONE
        // line to tell these apart and each has a different remedy: select
        // something, wait, or fix the destination. A third sentence that
        // duplicated either of the first two would be a decline that says
        // nothing about which command declined.
        for (i, a) in declines.iter().enumerate() {
            for b in &declines[i + 1..] {
                assert_ne!(
                    a, b,
                    "two declines with different remedies must not share a sentence"
                );
            }
        }
    }

    /// **The decline does not tell the operator they did something wrong.**
    #[test]
    fn the_decline_reports_the_state_rather_than_instructing_the_operator() {
        let line = zoom_declined_no_selection();
        for imperative in ["Select ", "select something", "first", "try again"] {
            assert!(
                !line.contains(imperative),
                "the control was enabled and then declined; {imperative:?} \
                 blames the operator for a race they cannot see: {line}"
            );
        }
        assert!(
            line.contains("right now"),
            "the sentence has to date its claim to the gesture rather than \
             assert a standing fact about a selection the operator may \
             already have made: {line}"
        );
    }

    /// Every counted note in this module, so a new one cannot be added
    /// without inheriting the checks below.
    const COUNTERS: [fn(usize) -> String; 9] = [
        diagnostics_contents_missing,
        diagnostics_fonts_skipped,
        diagnostics_images_skipped,
        diagnostics_glyphs_notdef,
        diagnostics_glyphs_substituted,
        diagnostics_glyphs_supplied,
        diagnostics_layers_hidden,
        diagnostics_ops_deferred,
        diagnostics_ops_unknown,
    ];

    /// **One is singular everywhere it can be.**
    #[test]
    fn every_counted_note_is_singular_at_one() {
        for f in COUNTERS {
            let one = f(1);
            let two = f(2);
            assert!(one.contains('1'), "a count must show its number: {one}");
            assert!(two.contains('2'), "a count must show its number: {two}");
            assert_ne!(
                one,
                two.replacen('2', "1", 1),
                "the singular form is missing — this note reads as a plural at \
                 a count of one: {one}"
            );
        }
    }

    /// The join is what makes several notes one line.
    #[test]
    fn notes_join_into_one_line() {
        let joined = diagnostics_join(&[
            diagnostics_glyphs_substituted(3),
            diagnostics_images_skipped(1),
        ]);
        assert!(joined.contains('·'));
        assert!(
            !joined.contains('\n'),
            "the bar has exactly one row: {joined}"
        );
    }

    /// **The clamp note names both numbers.**
    #[test]
    fn the_clamp_note_names_what_was_asked_and_what_was_given() {
        let note = page_clamped_note(99, 42, 42);
        assert!(note.contains("99"), "{note}");
        assert!(note.contains("42"), "{note}");
    }

    /// The two failure notes must not read alike.
    #[test]
    fn the_two_page_box_notes_read_differently() {
        assert_ne!(page_clamped_note(99, 42, 42), page_rejected_note());
    }

    /// The page number shown is 1-based, and the total reads as a total.
    #[test]
    fn the_page_number_is_one_based_and_the_total_is_labelled() {
        assert_eq!(page_number(1), "1");
        assert_eq!(page_number(42), "42");
        assert!(page_of_total(42).contains("42"));
        assert!(
            page_of_total(42).starts_with('/'),
            "the total must read as a denominator, not as a second page number"
        );
    }

    /// A zoom readout is a percentage.
    #[test]
    fn the_zoom_readout_carries_its_unit() {
        assert_eq!(zoom_percent(100.0), "100%");
        assert_eq!(zoom_percent(8.0), "8%");
    }

    /// **The three fit labels are distinct, and so are their tooltips.**
    #[test]
    fn the_three_fit_controls_are_distinguishable() {
        let labels = [fit_actual_size(), fit_width(), fit_page()];
        let tooltips = [
            fit_actual_size_tooltip(),
            fit_width_tooltip(),
            fit_page_tooltip(),
        ];
        for i in 0..3 {
            for j in (i + 1)..3 {
                assert_ne!(labels[i], labels[j]);
                assert_ne!(tooltips[i], tooltips[j]);
            }
        }
    }

    /// **Each fit tooltip names exactly the chord that reaches it.**
    #[test]
    fn each_fit_tooltip_names_exactly_the_chord_that_reaches_it() {
        assert!(
            fit_actual_size_tooltip().contains("Ctrl+0"),
            "the manifest binds Ctrl+0 to view.zoom_actual and the keyboard enacts it: {}",
            fit_actual_size_tooltip()
        );
        assert!(
            !fit_page_tooltip().contains("Ctrl"),
            "no chord reaches Fit page in this build: {}",
            fit_page_tooltip()
        );
        assert!(
            !fit_width_tooltip().contains("Ctrl"),
            "Ctrl+2 belongs to mode.review, not to Fit width: {}",
            fit_width_tooltip()
        );
    }

    /// **The mirrors say exactly what the ribbon says.**
    #[test]
    fn the_fit_mirrors_repeat_the_ribbon_word_for_word() {
        use crate::text::commands as c;
        assert_eq!(fit_actual_size(), c::view_zoom_actual().label);
        assert_eq!(fit_actual_size_tooltip(), c::view_zoom_actual().tooltip);
        assert_eq!(fit_page(), c::view_zoom_fit_page().label);
        assert_eq!(fit_page_tooltip(), c::view_zoom_fit_page().tooltip);
        assert_eq!(fit_width(), c::view_zoom_fit_width().label);
        assert_eq!(fit_width_tooltip(), c::view_zoom_fit_width().tooltip);
    }

    /// The page-box tooltip must state the commit rule.
    #[test]
    fn the_page_box_tooltip_states_the_commit_rule() {
        let t = page_box_tooltip();
        assert!(t.contains("Enter"), "{t}");
        assert!(t.contains("while you type"), "{t}");
    }

    #[test]
    fn zoom_percent_rounds_rather_than_truncating() {
        let mut v = crate::viewer::ViewState::default();
        v.set_zoom(0.999_97, crate::viewer::MAX_ZOOM);
        assert_eq!(zoom_percent(v.zoom_percent()), "100%");
        v.set_zoom(0.335, crate::viewer::MAX_ZOOM);
        assert_eq!(zoom_percent(v.zoom_percent()), "34%");
    }

    /// O24j — **the readout must survive the whole ceiling the zoom offers.**
    #[test]
    fn the_readout_survives_the_whole_configured_range() {
        let mut v = crate::viewer::ViewState::default();
        for (zoom, want) in [
            (1.0_f32, "100%"),
            (8.0, "800%"),
            (1.0e6, "100000000%"),
            // Not "1000000000000%", and the difference is not a defect.
            // `ViewState::zoom` is an `f32`, so the nearest representable
            // value to 10¹⁰ is 9,999,999,827,968 / 1000 — and the readout
            // shows what the view IS rather than what was asked for. Pinned
            // exactly, so a future change that starts rounding the display
            // instead of reporting it has to be a deliberate one.
            (1.0e10, "999999995904%"),
        ] {
            v.set_zoom(zoom, f32::MAX);
            let shown = zoom_percent(v.zoom_percent());
            assert_eq!(shown, want, "zoom {zoom} showed {shown}");
            assert!(
                !shown.contains("4294967295"),
                "the readout saturated at u32::MAX"
            );
        }
    }
}
