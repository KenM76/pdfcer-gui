//! The print preview's words: its strip of controls, its readouts, and what it
//! says about content lost off the printable area.

/// The word before the sheet-number box: which sheet of the **job** shows.
#[must_use]
pub const fn preview_sheet() -> &'static str {
    "Sheet"
}

/// After the sheet-number box: the job's sheet count.
#[must_use]
pub fn preview_sheet_of(total: usize) -> String {
    format!("of {total}")
}

/// The sheet-number box's hover.
#[must_use]
pub const fn preview_sheet_tooltip() -> &'static str {
    "Type a sheet number and press Enter to go to it."
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

/// Fit the sheet's width to the preview, top edge in view.
#[must_use]
pub const fn preview_fit_width() -> &'static str {
    "Fit width"
}

/// Hover text for Fit width.
#[must_use]
pub const fn preview_fit_width_tooltip() -> &'static str {
    "Fill the preview with the sheet's width, from its top edge."
}

/// Fit the sheet's height to the preview.
#[must_use]
pub const fn preview_fit_height() -> &'static str {
    "Fit height"
}

/// Hover text for Fit height.
#[must_use]
pub const fn preview_fit_height_tooltip() -> &'static str {
    "Fill the preview with the sheet's height, centred."
}

/// Hover text for Previous.
#[must_use]
pub const fn preview_previous_tooltip() -> &'static str {
    "Show the previous sheet of the job."
}

/// Hover text for Next.
#[must_use]
pub const fn preview_next_tooltip() -> &'static str {
    "Show the next sheet of the job."
}

/// Hover text for Zoom out.
#[must_use]
pub const fn preview_zoom_out_tooltip() -> &'static str {
    "Zoom out one step (Ctrl+wheel down over the sheet)."
}

/// Hover text for Zoom in.
#[must_use]
pub const fn preview_zoom_in_tooltip() -> &'static str {
    "Zoom in one step (Ctrl+wheel up over the sheet)."
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

/// The preview's switch between the page in colour and its plates.
#[must_use]
pub const fn preview_plates() -> &'static str {
    "Show plates"
}

/// What the plates switch does.
#[must_use]
pub const fn preview_plates_tooltip() -> &'static str {
    "Show each ink's sheet as it will print, in place of the page in colour."
}

/// Plates are on, but this sheet is n-up, a booklet or a poster.
#[must_use]
pub const fn preview_plates_imposed() -> &'static str {
    "This sheet prints in colour: n-up, booklet and poster sheets are not separated."
}

/// Plates are on, but the engine could not separate this page.
#[must_use]
pub const fn preview_plates_refused() -> &'static str {
    "This page is too large to separate in memory, so it shows and prints in colour."
}

/// The previous-ink arrow's hover.
#[must_use]
pub const fn preview_plate_previous() -> &'static str {
    "Previous ink"
}

/// The next-ink arrow's hover.
#[must_use]
pub const fn preview_plate_next() -> &'static str {
    "Next ink"
}

/// Which plate is showing: the ink as the file spells it, `i` of `n`.
#[must_use]
pub fn preview_plate_position(ink: &str, index: usize, total: usize) -> String {
    format!("{ink} ({index} of {total})")
}

/// Plates are on and this page's plates are still rendering.
#[must_use]
pub const fn preview_plates_separating() -> &'static str {
    "Separating inks\u{2026}"
}
