//! # `text::print` — every word the print dialog shows
//!
//! The catalog area for `pdfcer_gui::dialogs::print`. One module per surface is
//! the rule this directory's `mod.rs` states; the print dialog is a surface,
//! and it is a large one — three tabs, a preview, a device selector and a
//! commit button whose label is itself a disclosure.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/print/mod.md`.

/// The dialog's window title.
#[must_use]
pub const fn dialog_title() -> &'static str {
    "Print"
}

// ---------------------------------------------------------------------------
// The device, and the three ways there is not one
// ---------------------------------------------------------------------------

/// Label for the printer selector.
#[must_use]
pub const fn printer_label() -> &'static str {
    "Printer"
}

/// Label for the button that opens the **driver's own** properties dialog.
#[must_use]
pub const fn properties() -> &'static str {
    "Properties…"
}

/// Hover text for [`properties`].
#[must_use]
pub const fn properties_tooltip() -> &'static str {
    "Opens this printer's own settings — media type, quality, finishing and anything else the driver offers. pdfcer keeps the orientation chosen here in the Print dialog; a paper size chosen there is picked up by the Paper list."
}

/// Disclosure that the driver's settings are being carried with the job.
#[must_use]
pub const fn properties_held() -> &'static str {
    "This printer's own settings are being sent with the job."
}

/// The driver's properties dialog could not be opened, and why.
#[must_use]
pub fn properties_failed(detail: &str) -> String {
    format!("This printer's own settings could not be opened. {detail}")
}

/// **pdfcer could not ask this system about its printers at all.**
#[must_use]
pub const fn spooler_unavailable() -> &'static str {
    "pdfcer could not ask this system about its printers, so there is nothing to \
     print to. Nothing has been sent."
}

/// The engine's own account of a spooler failure, shown beneath
/// [`spooler_unavailable`].
#[must_use]
pub fn spooler_detail(detail: &str) -> String {
    format!("The print system reported: {detail}")
}

/// **The spooler answered, and this system has no printers installed.**
///
/// The second sentence. A true statement about the machine, which is why it
/// may not be used for the case above.
#[must_use]
pub const fn no_printers() -> &'static str {
    "This system reports no printers. Install one, then reopen this dialog."
}

/// **This particular printer's driver would not describe itself.**
#[must_use]
pub const fn device_unavailable() -> &'static str {
    "This printer's driver did not report its paper size, so pdfcer cannot show \
     what the sheet will look like. Choose another printer."
}

/// The dialog was asked to draw with no document open.
#[must_use]
pub const fn no_document() -> &'static str {
    "No document is open, so there is nothing to print."
}

// ---------------------------------------------------------------------------
// The tab strip
// ---------------------------------------------------------------------------

/// Tab 1's label.
#[must_use]
pub const fn tab_pages_layout() -> &'static str {
    "Pages"
}

/// Tab 1's hover text — the question the tab answers.
#[must_use]
pub const fn tab_pages_layout_tooltip() -> &'static str {
    "Which pages print, and how each one lands on the sheet."
}

/// Tab 2's label.
#[must_use]
pub const fn tab_copies_finishing() -> &'static str {
    "Copies"
}

/// Tab 2's hover text.
#[must_use]
pub const fn tab_copies_finishing_tooltip() -> &'static str {
    "How many sheets come out, in what order, and on how many sides."
}

/// Tab 3's label.
#[must_use]
pub const fn tab_comments() -> &'static str {
    "Comments"
}

/// Tab 3's hover text.
#[must_use]
pub const fn tab_comments_tooltip() -> &'static str {
    "Which comments, markup and form fields are painted onto each page."
}

/// Tab 4's label.
#[must_use]
pub const fn tab_position() -> &'static str {
    "Position"
}

/// Tab 4's hover text.
#[must_use]
pub const fn tab_position_tooltip() -> &'static str {
    "Where each page sits on the sheet, and therefore what gets cropped."
}

// ---------------------------------------------------------------------------
// Tab 1 — Pages & Layout
// ---------------------------------------------------------------------------

/// Heading over the page-range radios.
#[must_use]
pub const fn pages_heading() -> &'static str {
    "Pages"
}

/// "All N pages" — the count is in the label so the operator can see what
/// "all" costs before choosing it.
#[must_use]
pub fn range_all(pages: usize) -> String {
    if pages == 1 {
        "All 1 page".to_owned()
    } else {
        format!("All {pages} pages")
    }
}

/// The page currently on the canvas.
#[must_use]
pub const fn range_current() -> &'static str {
    "Current page"
}

/// The typed-range radio.
#[must_use]
pub const fn range_custom() -> &'static str {
    "Pages"
}

/// Hover text for the range box.
#[must_use]
pub const fn range_hint() -> &'static str {
    "Page numbers or ranges, for example 3 or 1-4 or 5,1-2. \
     Numbers are the ones printed on the page, starting at 1."
}

/// The typed range names no page in this document.
#[must_use]
pub const fn range_unparsable() -> &'static str {
    "That range does not name any page in this document."
}

/// Label in front of the odd/even radios.
#[must_use]
pub const fn subset_label() -> &'static str {
    "Subset"
}

/// No odd/even filtering.
#[must_use]
pub const fn subset_all() -> &'static str {
    "Every page"
}

/// Odd document pages only.
#[must_use]
pub const fn subset_odd() -> &'static str {
    "Odd only"
}

/// Even document pages only.
#[must_use]
pub const fn subset_even() -> &'static str {
    "Even only"
}

/// Hover text over the subset row.
#[must_use]
pub const fn subset_tooltip() -> &'static str {
    "Odd and even mean the page numbers printed on the paper, not positions \
     within the range above. The subset narrows the range; the two combine."
}

/// Heading over the sizing radios.
#[must_use]
pub const fn sizing_heading() -> &'static str {
    "Sizing"
}

/// Scale up or down to fill the printable area.
#[must_use]
pub const fn scale_fit() -> &'static str {
    "Fit to the printable area"
}

/// One PDF point to one point of paper.
#[must_use]
pub const fn scale_actual() -> &'static str {
    "Actual size"
}

/// Reduce an oversized page; never enlarge a small one.
#[must_use]
pub const fn scale_shrink() -> &'static str {
    "Shrink oversized pages only"
}

/// An explicit percentage.
#[must_use]
pub const fn scale_custom() -> &'static str {
    "Custom scale"
}

/// Hover text over the sizing group.
#[must_use]
pub const fn sizing_tooltip() -> &'static str {
    "Fit scales in both directions, so a small page is enlarged to fill the \
     sheet. Shrink oversized pages only ever reduces."
}

/// Suffix on the custom-scale spinner.
#[must_use]
pub const fn percent_suffix() -> &'static str {
    " %"
}

/// Heading over the orientation radios.
#[must_use]
pub const fn orientation_heading() -> &'static str {
    "Orientation"
}

/// Decide per page from its own shape.
#[must_use]
pub const fn orientation_auto() -> &'static str {
    "Auto, from each page's shape"
}

/// Force portrait.
#[must_use]
pub const fn orientation_portrait() -> &'static str {
    "Portrait"
}

/// Force landscape.
#[must_use]
pub const fn orientation_landscape() -> &'static str {
    "Landscape"
}

// ---------------------------------------------------------------------------
// Tab 2 — Copies & Finishing
// ---------------------------------------------------------------------------

/// Label in front of the copy-count spinner.
#[must_use]
pub const fn copies_label() -> &'static str {
    "Copies"
}

/// The collation checkbox, phrased as the *un*-collated option.
#[must_use]
pub const fn uncollated() -> &'static str {
    "Group each page's copies together, rather than repeating the whole set"
}

/// Print the sequence back to front.
#[must_use]
pub const fn reverse() -> &'static str {
    "Print back to front"
}

/// Hover text for reverse — names the reason it exists.
#[must_use]
pub const fn reverse_tooltip() -> &'static str {
    "For a printer that stacks face-up, so the finished pile is in order."
}

/// Heading over the duplex radios.
#[must_use]
pub const fn duplex_heading() -> &'static str {
    "Two-sided"
}

/// One side only.
#[must_use]
pub const fn duplex_off() -> &'static str {
    "One-sided"
}

/// Two-sided, flipped on the long edge — the usual book binding.
#[must_use]
pub const fn duplex_long() -> &'static str {
    "Two-sided, long-edge binding"
}

/// Two-sided, flipped on the short edge — notepad binding.
#[must_use]
pub const fn duplex_short() -> &'static str {
    "Two-sided, short-edge binding"
}

/// Label for the tray-by-sheet-size checkbox.
#[must_use]
pub const fn tray_by_size() -> &'static str {
    "Let the printer choose the tray from each page's size"
}

/// Hover text for [`tray_by_size`].
#[must_use]
pub const fn tray_tooltip() -> &'static str {
    "Useful when a document mixes sheet sizes and the printer has a tray loaded for each. Off, every sheet is fed from the printer's usual tray."
}

/// Disclosure for a driver that did not advertise tray-by-size.
#[must_use]
pub const fn tray_not_advertised() -> &'static str {
    "This printer does not advertise tray selection by sheet size. The request is still sent; the driver may ignore it."
}

// ---------------------------------------------------------------------------
// Paper — the sheet, and the fact that asking for one is only asking
// ---------------------------------------------------------------------------

/// Heading over the paper selector.
#[must_use]
pub const fn paper_heading() -> &'static str {
    "Paper"
}

/// The paper entry meaning "say nothing; use whatever this printer is set to".
#[must_use]
pub const fn paper_device_default() -> &'static str {
    "From the printer's own settings"
}

/// One entry in the paper list: the driver's name for it, and its size.
#[must_use]
pub fn paper_form(name: &str, size_pt: (f64, f64)) -> String {
    use crate::units::whole_mm_from_points as mm;
    format!("{name} — {} × {} mm", mm(size_pt.0), mm(size_pt.1))
}

/// The paper entry meaning "look at the pages and pick the sheet yourself" —
/// operator request O167, 2026-09-10.
#[must_use]
pub const fn paper_auto() -> &'static str {
    "Match the pages in this document"
}

/// What auto selection chose, when every page fits on it.
#[must_use]
pub fn paper_auto_matched(name: &str, sheet_pt: (f64, f64), page_pt: (f64, f64)) -> String {
    use crate::units::whole_mm_from_points as mm;
    format!(
        "Largest page in this job: {} × {} mm. Closest sheet this printer offers: {name} ({} × {} mm). pdfcer asks the printer for it; a driver may ignore the request without reporting it, so check the first sheet that comes out.",
        mm(page_pt.0),
        mm(page_pt.1),
        mm(sheet_pt.0),
        mm(sheet_pt.1),
    )
}

/// The extra sentence when the job does not have one page size throughout.
#[must_use]
pub const fn paper_auto_mixed() -> &'static str {
    "This job has more than one page size. Smaller pages will be scaled onto the chosen sheet — tick Choose tray by sheet size below if this printer has more than one tray or roll."
}

/// What auto selection chose when **nothing** the printer offers is big enough.
#[must_use]
pub fn paper_auto_too_big(name: &str, sheet_pt: (f64, f64), page_pt: (f64, f64)) -> String {
    use crate::units::whole_mm_from_points as mm;
    format!(
        "Largest page in this job: {} × {} mm — bigger than any sheet this printer offers. Using its largest, {name} ({} × {} mm). pdfcer asks the printer for it; a driver may ignore the request without reporting it, so check the first sheet that comes out.",
        mm(page_pt.0),
        mm(page_pt.1),
        mm(sheet_pt.0),
        mm(sheet_pt.1),
    )
}

/// Shown when auto selection is chosen but there is nothing to measure.
#[must_use]
pub const fn paper_auto_no_basis() -> &'static str {
    "There are no pages to measure, so no sheet was chosen. The job will use whatever the printer's own settings name."
}

/// Shown in place of the paper list when the driver enumerated none.
#[must_use]
pub const fn paper_not_listed() -> &'static str {
    "This printer did not list any paper sizes. The job will use whatever the printer's own settings name."
}

/// **The sheet a chosen paper actually means: a request, not a setting.**
#[must_use]
pub fn paper_is_a_request(sheet: Option<(f64, f64)>) -> String {
    let Some((w_pt, h_pt)) = sheet else {
        return "pdfcer asks the printer for this sheet. A driver may ignore the request without reporting it, so check the first sheet that comes out.".to_owned();
    };
    use crate::units::whole_mm_from_points as mm;
    format!(
        "Planned for {} × {} mm. pdfcer asks the printer for this sheet; a driver may ignore the request without reporting it, so check the first sheet that comes out.",
        mm(w_pt),
        mm(h_pt),
    )
}

/// The sheet the job was laid out on, when the operator asked for no
/// particular one.
#[must_use]
pub fn sheet_from_driver(sheet: Option<(f64, f64)>) -> String {
    let Some((w_pt, h_pt)) = sheet else {
        return "Paper comes from this printer's own settings in Windows.".to_owned();
    };
    // Whole millimetres because the operator is matching this against a ream
    // label, and a tenth of a millimetre of driver rounding is noise that
    // makes a familiar size look unfamiliar. The conversion and the rounding
    // rule are both `units`'; see that module's header for why half away from
    // zero rather than Rust's `{:.0}` default.
    use crate::units::whole_mm_from_points as mm;
    format!(
        "Planned for {} × {} pt ({} × {} mm), from this printer's own settings in Windows.",
        w_pt.round() as i64,
        h_pt.round() as i64,
        mm(w_pt),
        mm(h_pt),
    )
}

// ---------------------------------------------------------------------------
// Tab 3 — Comments, and the resolution block on Tab 1
// ---------------------------------------------------------------------------

/// Heading over the annotation-scope radios.
#[must_use]
pub const fn comments_heading() -> &'static str {
    "Comments and forms"
}

/// Page content, links and form-field widgets — no review markup.
#[must_use]
pub const fn scope_document() -> &'static str {
    "Document"
}

/// Everything above, plus review markup.
#[must_use]
pub const fn scope_markups() -> &'static str {
    "Document and markups"
}

/// Everything above, restricted to stamps.
#[must_use]
pub const fn scope_stamps() -> &'static str {
    "Document and stamps"
}

/// Form-field widgets only, over blank page content.
#[must_use]
pub const fn scope_fields_only() -> &'static str {
    "Form fields only"
}

/// The standing note that pdfcer prints rasters, not vectors.
#[must_use]
pub const fn raster_note() -> &'static str {
    "pdfcer renders each page to an image at the resolution below and sends \
     that image. Text and lines are not sent as vectors."
}

/// pdfcer chose a resolution the operator did not.
#[must_use]
pub fn dpi_capped(dpi: u32, device_dpi: u32, uncapped_page_mb: u64) -> String {
    format!(
        "Printing at {dpi} DPI. This printer can do {device_dpi} DPI, but one page \
         at that resolution costs pdfcer about {uncapped_page_mb} MB of memory, so \
         pdfcer capped it. Raise the cap if you need the detail."
    )
}

/// Label before the resolution limit field.
#[must_use]
pub const fn dpi_limit_label() -> &'static str {
    "Highest resolution"
}

/// The resolution in use when pdfcer did not cap it.
#[must_use]
pub fn dpi_in_use(dpi: u32, device_dpi: u32) -> String {
    if dpi == device_dpi {
        format!("Printing at {dpi} DPI, the printer's own resolution.")
    } else {
        format!("Printing at {dpi} DPI; this printer can do {device_dpi} DPI.")
    }
}

/// Suffix on the DPI spinner.
#[must_use]
pub const fn dpi_suffix() -> &'static str {
    " DPI"
}

// ---------------------------------------------------------------------------
// The preview
// ---------------------------------------------------------------------------

/// "Sheet i of n" — which sheet of the **job** is showing.
#[must_use]
pub fn preview_position(index: usize, total: usize) -> String {
    format!("Sheet {index} of {total}")
}

/// Step to the previous sheet of the job.
#[must_use]
pub const fn preview_previous() -> &'static str {
    "Previous"
}

/// Step to the next sheet of the job.
#[must_use]
pub const fn preview_next() -> &'static str {
    "Next"
}

/// Put the preview back to fit, centred.
#[must_use]
pub const fn preview_zoom_fit() -> &'static str {
    "Fit"
}

/// Hover text for Fit.
#[must_use]
pub const fn preview_zoom_fit_tooltip() -> &'static str {
    "Show the whole sheet, centred."
}

/// Zoom the preview out one step.
#[must_use]
pub const fn preview_zoom_out() -> &'static str {
    "Zoom out"
}

/// Zoom the preview in one step.
#[must_use]
pub const fn preview_zoom_in() -> &'static str {
    "Zoom in"
}

/// Draw one PDF point as one screen point.
#[must_use]
pub const fn preview_zoom_actual() -> &'static str {
    "100%"
}

/// Hover text for the actual-size button.
#[must_use]
pub const fn preview_zoom_actual_tooltip() -> &'static str {
    "Draw the sheet at its true size on this screen."
}

/// The magnification readout.
#[must_use]
pub fn preview_zoom_percent(percent: u32) -> String {
    format!("{percent}% of actual size")
}

/// The gesture hint under the preview.
#[must_use]
pub const fn preview_pan_hint() -> &'static str {
    "Drag the page to move it on the sheet, drag the paper to pan, Ctrl+wheel to zoom"
}

/// Move the preview into a window of its own — operator request O112.
#[must_use]
pub const fn preview_pop_out() -> &'static str {
    "Pop out"
}

/// Hover text for the pop-out button.
#[must_use]
pub const fn preview_pop_out_tooltip() -> &'static str {
    "Show the preview in its own window, which you can resize and move to another screen. \
     Closing that window puts the preview back here."
}

/// The pop-out window's title bar.
#[must_use]
pub const fn preview_window_title() -> &'static str {
    "Print preview"
}

/// The job selects no pages, so there is nothing to preview.
#[must_use]
pub const fn no_pages_selected() -> &'static str {
    "This job selects no pages, so there is nothing to preview and nothing to print."
}

/// **Content will be lost off the edge of the printable area.**
#[must_use]
pub fn clip_summary(clipped: usize, total: usize) -> String {
    if clipped == 1 {
        format!("1 of these {total} sheets will lose content outside the printable area.")
    } else {
        format!("{clipped} of these {total} sheets will lose content outside the printable area.")
    }
}

/// **A CEILING on how many sheets will lose content**, for the state where
/// some have been examined and some have not — operator request O113,
/// 2026-09-04.
#[must_use]
pub fn clip_summary_at_most(clipped: usize, total: usize) -> String {
    if clipped == 1 {
        format!("Up to 1 of these {total} sheets may lose content outside the printable area.")
    } else {
        format!(
            "Up to {clipped} of these {total} sheets may lose content outside the printable area."
        )
    }
}

/// **The sheet on screen overhangs the printable area, and the overhang is
/// empty paper** — operator request O113, 2026-09-03.
#[must_use]
pub const fn overhang_is_blank() -> &'static str {
    "This sheet hangs over the printable area, but nothing is printed there — the overhang is \
     blank."
}

// ---------------------------------------------------------------------------
// The footer — the one irreversible control in the application
// ---------------------------------------------------------------------------
//

/// The three ways out of the print window, and what each does to the
/// remembered settings. See the module's own header for the argument.
mod footer;

// A glob re-export, deliberately. The alternative — naming five functions
// here — is a hand-written list inside the mechanism that exists to make the
// split invisible, and this project's standing lesson is that a hand-written
// list is exactly where the next addition goes missing. A sixth footer string
// added next door is reachable as `t::…` the moment it is written.
pub use footer::*;

/// **Where the page sits on the paper** — operator request O208.
///
/// Its own file because every sentence in it is about one quantity and the two
/// conventions that quantity needs stated; the module header carries both.
mod position;

// Glob re-exported, for the reason argued over `footer` above.
pub use position::*;

/// **Poster printing** — operator request O238. One page across many sheets.
mod poster;

// Glob re-exported, for the reason argued over `footer` above.
pub use poster::*;

/// **Fixed line width** — operator request O233.
mod lines;

// Glob re-exported, for the reason argued over `footer` above.
pub use lines::*;

#[cfg(test)]
mod tests {
    use super::*;

    /// The three no-printer sentences must be genuinely different.
    #[test]
    fn the_three_no_printer_sentences_read_differently() {
        let a = spooler_unavailable();
        let b = no_printers();
        let c = device_unavailable();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    /// The commit label carries the count, and the count is visible in it.
    #[test]
    fn the_commit_label_states_the_clip_count() {
        assert!(commit_with_clipping(7).contains('7'));
        assert!(commit_with_clipping(1).contains('1'));
        // And it must not read like the plain label, or the disclosure is
        // invisible at a glance.
        assert_ne!(commit_with_clipping(1), commit());
    }

    /// Singular and plural are both grammatical.
    #[test]
    fn the_counted_sentences_are_grammatical_at_one() {
        assert!(commit_with_clipping(1).contains("1 sheet will"));
        assert!(clip_summary(1, 4).contains("1 of these 4 sheets"));
        assert!(commit_losing_content(1).contains("1 sheet will"));
        assert!(commit_may_lose_content(1).contains("1 sheet may"));
        assert!(clip_summary_at_most(1, 4).contains("1 of these 4 sheets"));
        assert!(sent(1).contains("1 page to"));
        assert!(range_all(1).contains("1 page"));
        assert!(!range_all(1).contains("1 pages"));
    }

    /// **The three commit labels are three different claims**, and an
    /// operator must be able to tell which one they are being shown from the
    /// words alone — operator request O113, 2026-09-04.
    #[test]
    fn the_three_commit_labels_are_distinguishable_claims() {
        let geometric = commit_with_clipping(3);
        let measured = commit_losing_content(3);
        let bounded = commit_may_lose_content(3);
        assert_ne!(geometric, measured);
        assert_ne!(measured, bounded);
        assert_ne!(geometric, bounded);

        assert!(
            bounded.contains("may") && bounded.contains("up to"),
            "the ceiling must hedge, or a number nobody measured reads as one that was: {bounded}"
        );
        for measured_claim in [&geometric, &measured] {
            assert!(
                !measured_claim.contains("may"),
                "a measured count must NOT hedge — softening a true statement to match a \
                 better one is how the next defect gets built: {measured_claim}"
            );
        }
        // And all three still carry the number, which is the whole
        // disclosure mechanism.
        for label in [&geometric, &measured, &bounded] {
            assert!(label.contains('3'), "the count vanished from {label}");
        }
    }

    /// The bounded job-wide sentence hedges where its measured twin does not.
    #[test]
    fn only_the_bounded_summary_hedges() {
        assert!(!clip_summary(2, 5).contains("may"));
        assert!(clip_summary_at_most(2, 5).contains("may"));
        assert!(clip_summary_at_most(2, 5).contains("Up to 2"));
    }

    /// The capped-resolution disclosure names all three numbers.
    #[test]
    fn the_dpi_disclosure_names_what_it_costs() {
        let message = dpi_capped(300, 1200, 139);
        for number in ["300", "1200", "139"] {
            assert!(message.contains(number), "missing {number} in {message}");
        }
    }
}
