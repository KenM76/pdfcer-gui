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

/// Whether the page uses overprint. `partial` when only the part of the page
/// on screen was drawn, so the answer covers that part.
#[must_use]
pub fn overprint(set: bool, takes_effect: bool, partial: bool) -> String {
    let line = match (set, takes_effect) {
        (true, true) => "Overprint: used, and it changes how this page prints.",
        (true, false) => "Overprint: turned on, but it changes nothing on this page.",
        (false, _) => "Overprint: not used on this page.",
    };
    if partial {
        format!("{line} Only the part of the page on screen was checked.")
    } else {
        line.to_owned()
    }
}

/// What overprint means, on hover.
#[must_use]
pub fn overprint_hover() -> &'static str {
    "Overprint lets one ink print on top of another instead of cutting it out. \
     The preview always shows its effect, so what you see here is what the press \
     would do with it."
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

/// The status-bar line while the preview is on. `proofed` when the inks are
/// shown through a press simulation profile.
#[must_use]
pub fn status_line(hidden: usize, proofed: bool, warned: bool) -> String {
    let mut line = plates_line(hidden);
    if proofed {
        line.push_str(" Colours simulate a press profile.");
    }
    if warned {
        line.push_str(" Coverage warnings are painted over the page.");
    }
    line
}

fn plates_line(hidden: usize) -> String {
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

/// The simulation profile combo box's label.
#[must_use]
pub fn proof_label() -> &'static str {
    "Simulate"
}

/// What the simulation profile does, on hover.
#[must_use]
pub fn proof_hover() -> &'static str {
    "Show the inks as a chosen press and paper would print them, using that \
     press's colour profile. Only the screen changes; the file is untouched."
}

/// No simulation profile: the inks are shown through the plain ink model.
#[must_use]
pub fn proof_none() -> &'static str {
    "No press simulation"
}

/// The document's own output intent, offered as a simulation profile.
#[must_use]
pub fn proof_intent() -> &'static str {
    "This document's output intent"
}

/// A simulation profile this panel did not attach.
#[must_use]
pub fn proof_other() -> &'static str {
    "Another profile"
}

/// The profile file could not be read from disk.
#[must_use]
pub fn proof_unreadable(why: &str) -> String {
    format!("the file could not be read: {why}")
}

/// The output intent offered earlier is no longer there.
#[must_use]
pub fn proof_no_intent() -> &'static str {
    "this document has no output intent profile"
}

/// A profile that cannot be used, with the reason.
#[must_use]
pub fn proof_refused(why: &str) -> String {
    format!("That profile cannot be simulated: {why}.")
}

/// *Simulate paper colour*.
#[must_use]
pub fn proof_paper() -> &'static str {
    "Simulate paper colour"
}

/// What paper simulation shows, on hover.
#[must_use]
pub fn proof_paper_hover() -> &'static str {
    "Show the paper's white as the profile measured it, usually a little \
     darker or warmer than the screen's white. Also simulates black ink."
}

/// *Simulate black ink*.
#[must_use]
pub fn proof_black() -> &'static str {
    "Simulate black ink"
}

/// What black ink simulation shows, on hover.
#[must_use]
pub fn proof_black_hover() -> &'static str {
    "Show the press's darkest black as the profile measured it, usually a \
     dark grey, instead of the screen's black."
}

/// Why *Simulate black ink* is greyed.
#[must_use]
pub fn proof_black_implied() -> &'static str {
    "Paper colour simulation always simulates black ink too."
}

/// The profile has no black point compensation.
#[must_use]
pub fn proof_no_bpc() -> &'static str {
    "This profile's black cannot be matched to the screen's, so with both \
     boxes off it looks the same as Simulate black ink."
}

/// Spot inks are not simulated.
#[must_use]
pub fn proof_spots() -> &'static str {
    "Spot inks are shown as usual; the profile simulates only cyan, magenta, \
     yellow and black."
}

/// The coverage warnings' heading.
#[must_use]
pub fn warn_heading() -> &'static str {
    "Warnings"
}

/// *Total area coverage*.
#[must_use]
pub fn warn_total() -> &'static str {
    "Total area coverage over"
}

/// What the total area coverage warning flags, on hover.
#[must_use]
pub fn warn_total_hover() -> &'static str {
    "Paint over the page wherever all the inks together add up to more than \
     this percentage. Presses set a limit, often 300%, above which ink does \
     not dry cleanly."
}

/// *Rich black*.
#[must_use]
pub fn warn_rich() -> &'static str {
    "Rich black from black at"
}

/// What the rich black warning flags, on hover.
#[must_use]
pub fn warn_rich_hover() -> &'static str {
    "Paint over the page wherever black ink is at least this percentage and \
     cyan, magenta or yellow is mixed in with it."
}

/// The part of the page on screen is still being checked.
#[must_use]
pub fn warn_checking() -> &'static str {
    "checking…"
}

/// The engine could not check the page.
#[must_use]
pub fn warn_refused() -> &'static str {
    "could not be checked"
}

/// The rule flagged something on screen.
#[must_use]
pub fn warn_found() -> &'static str {
    "found on screen"
}

/// *Warning opacity*.
#[must_use]
pub fn warn_opacity() -> &'static str {
    "Warning opacity"
}

/// What warning opacity changes, on hover.
#[must_use]
pub fn warn_opacity_hover() -> &'static str {
    "How solidly the warning colours cover the page; lower lets the page show through."
}

/// The rule flagged nothing on screen.
#[must_use]
pub fn warn_none() -> &'static str {
    "none on screen"
}

#[cfg(test)]
mod tests {
    use super::overprint;

    #[test]
    fn overprint_tells_effect_from_no_effect() {
        assert_ne!(overprint(true, true, false), overprint(true, false, false));
        assert_eq!(
            overprint(false, true, false),
            overprint(false, false, false)
        );
    }

    #[test]
    fn the_status_line_names_a_simulation() {
        use super::status_line;
        assert_ne!(status_line(0, true, false), status_line(0, false, false));
        assert_ne!(status_line(0, false, true), status_line(0, false, false));
        assert!(status_line(1, true, true).starts_with(&status_line(1, false, false)));
    }

    #[test]
    fn a_partial_answer_says_so() {
        assert!(overprint(true, true, true).starts_with(&overprint(true, true, false)));
        assert!(overprint(true, true, true).len() > overprint(true, true, false).len());
    }
}
