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
