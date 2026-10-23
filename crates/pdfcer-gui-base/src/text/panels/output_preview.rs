//! `text::panels::output_preview` — the Output Preview panel's sentences and
//! its status-bar line.

/// The switch that turns the separation preview on.
#[must_use]
pub fn switch() -> &'static str {
    "Preview separations"
}

/// What the switch does, on hover.
#[must_use]
pub fn switch_hover() -> &'static str {
    "Draw the page as its printing inks, so you can hide any of them and see \
     what is on each plate. Only the screen changes; the file is untouched."
}

/// Shown while the preview is off.
#[must_use]
pub fn off_hint() -> &'static str {
    "Turn on to list this page's inks."
}

/// Shown while the first render under the preview is still being drawn.
#[must_use]
pub fn waiting() -> &'static str {
    "Reading this page's inks\u{2026}"
}

/// Shown when the preview is on and the page produced no ink list.
#[must_use]
pub fn no_inks() -> &'static str {
    "This page could not be split into inks: it needs more memory than the \
     preview is allowed. It is drawn as usual."
}

/// The button that shows every ink again.
#[must_use]
pub fn show_all() -> &'static str {
    "Show all inks"
}

/// One ink's checkbox hover.
#[must_use]
pub fn ink_hover(name: &str) -> String {
    format!("Show or hide the {name} plate.")
}

/// The page's blending space is RGB, and the preview composites it in ink.
#[must_use]
pub fn forced_subtractive() -> &'static str {
    "This page blends in RGB. To split it into inks it is blended as CMYK \
     instead, so transparent areas can look slightly different from the \
     normal view."
}

/// Hidden inks this page does not use.
#[must_use]
pub fn unmatched(names: &[String]) -> String {
    format!("Hidden but not used on this page: {}.", names.join(", "))
}

/// The status-bar line while the preview is on.
#[must_use]
pub fn status_line(hidden: usize) -> String {
    match hidden {
        0 => "Separation preview is on \u{2014} the page is drawn as its printing inks.".to_owned(),
        1 => "Separation preview is on \u{2014} 1 ink is hidden, so this is not how the page \
              will print."
            .to_owned(),
        n => format!(
            "Separation preview is on \u{2014} {n} inks are hidden, so this is not how the \
             page will print."
        ),
    }
}

/// Shown under the ink list until the pointer rests on the page.
#[must_use]
pub fn hover_hint() -> &'static str {
    "Rest the pointer on the page to read the inks under it."
}

/// Shown while the inks under the pointer are being read.
#[must_use]
pub fn hover_reading() -> &'static str {
    "Reading the inks under the pointer\u{2026}"
}

/// Where the reading was taken, in page points from the lower-left corner.
#[must_use]
pub fn hover_at(x: f64, y: f64) -> String {
    format!("Under the pointer, at {x:.0}, {y:.0} pt")
}

/// The sum of every plate's tint at the pointer.
#[must_use]
pub fn total_coverage(total: f32) -> String {
    format!("Total area coverage: {}", percent(total))
}

/// What the total means, on hover.
#[must_use]
pub fn total_coverage_hover() -> &'static str {
    "The sum of all inks at this point. Printers often set a limit, such as \
     300%, above which ink may not dry or may smear."
}

/// One plate's tint as a percentage.
#[must_use]
pub fn percent(tint: f32) -> String {
    format!("{:.0}%", tint * 100.0)
}

/// The pointer rested outside the page.
#[must_use]
pub fn hover_off_page() -> &'static str {
    "The pointer is off the page."
}

/// The point could not be read as inks.
#[must_use]
pub fn hover_no_inks() -> &'static str {
    "The inks at this point could not be read."
}

/// The probe render failed.
#[must_use]
pub fn hover_failed(why: &str) -> String {
    format!("The inks at this point could not be read: {why}")
}
