//! # `text::diagnostics` — every word the Render-diagnostics dialog shows
//!
//! The copy for `tools.render_diagnostics`, on **Tools ▸ Diagnostics**, drawn
//! by `pdfcer_gui::dialogs::diagnostics`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/diagnostics.md`.

use pdfcer_render::interpret::BlendSpaceFrom;

/// The dialog's title.
#[must_use]
pub fn title() -> &'static str {
    "Render diagnostics"
}

/// The lead-in above the measurements.
#[must_use]
pub fn subject(page_number: usize) -> String {
    format!("The picture currently on the canvas — page {page_number}")
}

/// How long the rasterization took.
#[must_use]
pub fn took(millis: u128) -> String {
    format!("Drawn in {millis} ms")
}

/// The raster scale and the pixel size it produced.
#[must_use]
pub fn raster(scale: f32, width: usize, height: usize) -> String {
    format!("Rasterized at {scale:.2}× — {width} × {height} pixels")
}

/// **What colour space this page was BLENDED in** - one short line, beside the
/// other two measurements.
#[must_use]
pub const fn blended_in(composites_in_ink: bool) -> &'static str {
    if composites_in_ink {
        "Blended in CMYK ink"
    } else {
        "Blended in screen colour (RGB)"
    }
}

/// **Where that blending space was decided**, which is a different fact from
/// what it is, and the only one of the two an operator can act on.
#[must_use]
pub const fn blend_space_from(source: BlendSpaceFrom) -> &'static str {
    match source {
        BlendSpaceFrom::PageGroup => "The page declares that space itself",
        BlendSpaceFrom::DeviceNative => {
            "The page declares no space, so the screen's own colour stands"
        }
        BlendSpaceFrom::OutputIntent => {
            "The page declares no space, so the file's print output intent decided it - Settings > Colour is where that rule lives"
        }
    }
}

/// Hover text for the measurement group.
///
/// Carries the one fact that stops the duration being misread, in the
/// operator's terms rather than as a percentage.
#[must_use]
pub fn raster_tooltip() -> &'static str {
    "On a dense drawing almost all of the cost is in the content rather than \
     in the pixel count, so a smaller raster is usually not a faster one."
}

/// Heading above the list of findings.
#[must_use]
pub fn findings_heading() -> &'static str {
    "What the renderer had to substitute or leave out"
}

/// Shown in place of the list when the page drew with nothing substituted and
/// nothing skipped.
#[must_use]
pub fn clean() -> &'static str {
    super::status::diagnostics_clean()
}

/// Shown when there is no raster to describe.
#[must_use]
pub fn nothing_drawn() -> &'static str {
    "This page has not been drawn yet, so there is nothing to report. The \
     canvas says so in its own words if a render failed."
}

/// The two counters that are deliberately **not** listed, said once.
#[must_use]
pub fn absorbed(tolerated: usize, compat_skipped: usize) -> String {
    if tolerated == 0 && compat_skipped == 0 {
        return "Nothing was absorbed: the renderer met no structural oddity and the file \
                asked it to skip nothing."
            .to_owned();
    }
    let oddities = if tolerated == 1 {
        "1 structural oddity the renderer drew correctly anyway".to_owned()
    } else {
        format!("{tolerated} structural oddities the renderer drew correctly anyway")
    };
    let skipped = if compat_skipped == 1 {
        "1 section the file itself asks readers to skip".to_owned()
    } else {
        format!("{compat_skipped} sections the file itself asks readers to skip")
    };
    format!("Absorbed without affecting the picture: {oddities}, and {skipped}.")
}

/// The dialog's Close button.
#[must_use]
pub fn close() -> &'static str {
    "Close"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The measurements say what they are measuring.
    #[test]
    fn every_measurement_carries_its_unit() {
        assert!(took(1240).contains("ms"), "a duration needs its unit");
        let line = raster(2.0, 1684, 1190);
        assert!(line.contains('×'), "a scale needs its multiplication sign");
        assert!(line.contains("1684") && line.contains("1190"));
        assert!(line.contains("pixels"), "a size needs its unit");
    }

    /// The scale is shown to two places, so 1.0 and 1.25 are distinguishable.
    #[test]
    fn the_scale_is_not_rounded_to_a_whole_number() {
        assert_ne!(raster(1.0, 1, 1), raster(1.25, 1, 1));
        assert!(raster(1.25, 1, 1).contains("1.25"));
    }

    /// **The clean sentence is the status bar's, not a second one.**
    #[test]
    fn the_clean_sentence_is_the_one_the_status_bar_uses() {
        assert_eq!(clean(), crate::text::status::diagnostics_clean());
    }

    /// **No `(s)` and no `oddity/oddities`**, in any of the four cases.
    #[test]
    fn the_absorbed_line_is_never_written_with_a_slash_or_a_parenthesised_s() {
        for (t, c) in [(0, 0), (1, 0), (0, 1), (3, 5)] {
            let line = absorbed(t, c);
            assert!(!line.contains("(s)"), "parenthesised plural: {line}");
            assert!(!line.contains('/'), "slashed plural: {line}");
        }
        assert!(absorbed(1, 1).contains("1 structural oddity "));
        assert!(absorbed(2, 2).contains("2 structural oddities"));
        assert!(absorbed(1, 1).contains("1 section "));
        assert!(absorbed(2, 2).contains("2 sections"));
    }

    /// Both counters at zero gets a positive sentence, not "0 and 0".
    #[test]
    fn nothing_absorbed_is_stated_positively() {
        let none = absorbed(0, 0);
        assert!(
            !none.contains('0'),
            "a zero count reads as a template: {none}"
        );
        assert_ne!(none, absorbed(1, 0));
    }

    /// **The three blend-space origins are three different sentences**, and the
    /// falsification is per-variant rather than one combined assertion.
    #[test]
    fn every_blend_space_origin_says_something_different() {
        let all = [
            BlendSpaceFrom::PageGroup,
            BlendSpaceFrom::DeviceNative,
            BlendSpaceFrom::OutputIntent,
        ];
        for (i, a) in all.iter().enumerate() {
            assert!(
                !blend_space_from(*a).is_empty(),
                "an origin with no sentence is a silence, not a short answer"
            );
            for b in &all[i + 1..] {
                assert_ne!(
                    blend_space_from(*a),
                    blend_space_from(*b),
                    "two origins share one sentence, so one of them is wrong"
                );
            }
        }
    }

    /// **Only the output-intent origin points at a setting**, because it is the
    /// only one a setting can change.
    #[test]
    fn only_the_output_intent_origin_names_the_setting() {
        assert!(
            blend_space_from(BlendSpaceFrom::OutputIntent).contains("Settings"),
            "the one origin a control governs must name that control"
        );
        for quiet in [BlendSpaceFrom::PageGroup, BlendSpaceFrom::DeviceNative] {
            let line = blend_space_from(quiet);
            assert!(
                !line.contains("Settings"),
                "this origin cannot be changed by a setting, so naming one sends \
                 the operator somewhere that will not help: {line}"
            );
        }
    }

    /// **Ink and screen are different sentences, and neither is a bare word.**
    #[test]
    fn the_blend_line_says_what_was_blended_and_not_just_a_colour_model() {
        let ink = blended_in(true);
        let screen = blended_in(false);
        assert_ne!(ink, screen);
        assert!(
            ink.contains("CMYK"),
            "the operator's word, not 'subtractive'"
        );
        assert!(
            ink.to_lowercase().contains("blend") && screen.to_lowercase().contains("blend"),
            "the line must name the operation, or the colour model has no subject"
        );
    }

    /// The subject line names a page **number**, not an index.
    ///
    /// The caller adds one; this asserts the catalog does not add a second, and
    /// that the number reaches the string at all.
    #[test]
    fn the_subject_names_the_page_it_was_given() {
        assert!(subject(7).contains('7'));
        assert!(!subject(7).contains('8'), "the catalog must not renumber");
    }
}
