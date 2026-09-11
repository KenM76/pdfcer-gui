//! # `text::diagnostics` — every word the Render-diagnostics dialog shows
//!
//! The copy for `tools.render_diagnostics`, on **Tools ▸ Diagnostics**, drawn
//! by [`crate::dialogs::diagnostics`].
//!
//! ## ★ What is deliberately NOT here: the findings themselves
//!
//! The nine sentences that name what the renderer substituted or skipped —
//! *"3 glyphs drawn with a bundled substitute face"*, *"1 content stream
//! missing from the file"* — live in [`crate::text::status`] and are used from
//! there unchanged. They were written for the status bar's disclosure and they
//! are the same facts said to the same operator; a second wording here would be
//! `DEFECTS.md` D5's shape in a catalog rather than in a list, and the two
//! copies would drift the first time one of them was improved.
//!
//! So this file holds only what the **dialog** adds and the bar has nowhere to
//! put: a title, the three measurements of the render itself, the headings that
//! separate them from the findings, and the sentence shown when there is no
//! raster to describe at all.
//!
//! ## Why the measurements are worded as a pair
//!
//! `HANDOFF.md` §10 records the fact that makes a bare duration misleading on
//! this project's own documents:
//!
//! > **~99 % of render cost is resolution-independent** on dense CAD. A small
//! > thumbnail is not a cheap thumbnail. A 1×1 *point* region costs 691 ms.
//!
//! An operator reading "1,240 ms" alone will reach for the zoom, and on a CAD
//! sheet that will not help. So the scale and the pixel size are shown beside
//! the duration rather than under a separate heading, and the tooltip on the
//! group says what the relationship actually is. This is the same editorial
//! rule [`crate::text::status::diagnostics_layers_hidden`] follows one surface
//! over: **name the cause, or the number reads as a fault.**

use pdfcer_render::interpret::BlendSpaceFrom;

/// The dialog's title.
///
/// The command's own label, so an operator who pressed *Render diagnostics*
/// arrives at a window called *Render diagnostics*. A title that paraphrases
/// its command is a title that makes the operator wonder whether they opened
/// the right thing.
#[must_use]
pub fn title() -> &'static str {
    "Render diagnostics"
}

/// The lead-in above the measurements.
///
/// Says **which** render is being described, because it is not the document
/// and not "the last thing that happened" — it is the raster currently on the
/// canvas. An operator who has scrolled since would otherwise read these
/// numbers as being about the page they are looking at.
#[must_use]
pub fn subject(page_number: usize) -> String {
    format!("The picture currently on the canvas — page {page_number}")
}

/// How long the rasterization took.
///
/// Milliseconds, whole. Sub-millisecond precision would be false confidence:
/// the measurement is one wall-clock read around a call that competes with
/// whatever else the machine is doing, and the useful distinction on these
/// documents is between *tens* and *thousands*.
#[must_use]
pub fn took(millis: u128) -> String {
    format!("Drawn in {millis} ms")
}

/// The raster scale and the pixel size it produced.
///
/// Both, on one line, for the reason this module's header gives: a duration
/// with no scale beside it invites the operator to zoom out and expect it to
/// get cheaper.
///
/// The scale is **device pixels per PDF user-space unit** — the zoom already
/// multiplied by the display's density — which is why it is not the percentage
/// the status bar shows, and why the word is "scale" rather than "zoom".
#[must_use]
pub fn raster(scale: f32, width: usize, height: usize) -> String {
    format!("Rasterized at {scale:.2}× — {width} × {height} pixels")
}

/// **What colour space this page was BLENDED in** - one short line, beside the
/// other two measurements.
///
/// # Why an operator is owed this at all
///
/// It is the fact that explains the line above it. A page that composites in
/// four colorant planes costs more to draw than the same geometry composited in
/// three, and a CAD sheet exported for print routinely does - so a duration
/// that looks wrong on one sheet and fine on the next is very often this, and
/// nothing else in this shell says so.
///
/// It is also the precondition of `max_cmyk_buffer_bytes` meaning anything: the
/// operator can raise that ceiling in Settings > Colour and see no change
/// whatever, because the page never asked for ink in the first place.
///
/// # Why it names CMYK rather than "subtractive"
///
/// The engine's own vocabulary is *subtractive*, which is correct and is the
/// word its documentation uses. The operator's vocabulary is CMYK, and this is
/// the surface where his word wins - the same ruling the markup and text colour
/// disclosures already took.
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
///
/// # The three cases, and why the middle one is not a failure
///
/// * [`BlendSpaceFrom::PageGroup`] - the page's own `/Group` dictionary named a
///   space (ISO 32000-1 Table 147). The file said so; nothing was inferred, and
///   no setting in this shell can change the answer.
/// * [`BlendSpaceFrom::DeviceNative`] - the page group named nothing, so the
///   output device's own space stands, which for pdfcer is sRGB. This is the
///   ordinary case for almost every PDF ever made and reads as a non-event; it
///   is stated anyway, because an operator comparing two sheets needs to see
///   which of them declared something and which did not.
/// * [`BlendSpaceFrom::OutputIntent`] - the page group named nothing AND the
///   document carries an `/OutputIntents` entry pdfcer could resolve, so the
///   intent's own colorant count decided it. **This is the only case
///   `page_blend_space_source` governs**, and naming it is how an operator
///   learns which setting would change this page.
///
/// # Why the sentence names the setting in the third case only
///
/// R9's rule applied to prose rather than to a widget: pointing at a control
/// that cannot change the answer is the same defect as drawing a disabled one.
/// The first two cases say what happened and stop.
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
///
/// The same positive statement the status bar's disclosure makes, and
/// deliberately the same words: an operator who opened the disclosure and then
/// opened this dialog must not be told two different things about one raster.
/// Delegated rather than copied, so improving one improves both.
#[must_use]
pub fn clean() -> &'static str {
    super::status::diagnostics_clean()
}

/// Shown when there is no raster to describe.
///
/// Reachable, and not only in theory: the dialog is gated on `doc.open`, and
/// a document can be open with nothing yet drawn — before the first render, and
/// after a render failure, which is the state `page_texture` is `None` in. A
/// window that opened empty would read as the command being broken, so it says
/// which of the two nothings this is.
#[must_use]
pub fn nothing_drawn() -> &'static str {
    "This page has not been drawn yet, so there is nothing to report. The \
     canvas says so in its own words if a render failed."
}

/// The two counters that are deliberately **not** listed, said once.
///
/// [`crate::app::status::notes`]' editorial rule excludes `tolerated` and
/// `compat_skipped` from the one-line summary because both count divergences
/// that leave the picture correct, and *"listing them would put two numbers
/// that mean 'nothing is wrong' in front of the six that mean something is"*.
///
/// ★ The dialog is the surface that argument does **not** apply to. It has
/// room, it is a place an operator goes deliberately when something looks
/// wrong, and the numbers are exactly what someone diagnosing a file wants. So
/// they are shown here and nowhere else — with a sentence saying why they are
/// not faults, because a bare count of "tolerated" oddities beside a list of
/// real findings would otherwise read as nine problems instead of seven.
///
/// ★ **Written out in full for each count rather than with `(s)`.** The first
/// draft read *"0 structural oddity/oddities … and 0 section(s)"*, which was
/// seen in the running window and is the shape every `diagnostics_*` entry in
/// [`crate::text::status`] already refuses: each of those spells the singular
/// and the plural. A slash or a parenthesised `s` is a catalog telling the
/// operator that nobody read the sentence they are reading.
///
/// The **both-zero** case gets a sentence of its own for the same reason
/// [`crate::text::status::diagnostics_clean`] exists: "0 and 0" is a true
/// answer that reads as an unfilled template.
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
///
/// Its own function rather than borrowing [`crate::text::about::close`]: two
/// surfaces sharing a word is not the same as two surfaces sharing a *string*,
/// and a catalog that reaches sideways for a label is a catalog whose entries
/// cannot be changed independently.
#[must_use]
pub fn close() -> &'static str {
    "Close"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The measurements say what they are measuring.
    ///
    /// Not a spelling test — a **units** test. A duration with no unit and a
    /// scale with no multiplication sign are the two ways this surface could
    /// present a number the operator cannot interpret, and both are exactly the
    /// sort of thing a later edit trims for width.
    #[test]
    fn every_measurement_carries_its_unit() {
        assert!(took(1240).contains("ms"), "a duration needs its unit");
        let line = raster(2.0, 1684, 1190);
        assert!(line.contains('×'), "a scale needs its multiplication sign");
        assert!(line.contains("1684") && line.contains("1190"));
        assert!(line.contains("pixels"), "a size needs its unit");
    }

    /// The scale is shown to two places, so 1.0 and 1.25 are distinguishable.
    ///
    /// A rounded-to-integer scale would print `1×` for every zoom between 50 %
    /// and 150 % on a 1.0 density display, which is a readout that changes
    /// nothing while the thing it reports changes constantly.
    #[test]
    fn the_scale_is_not_rounded_to_a_whole_number() {
        assert_ne!(raster(1.0, 1, 1), raster(1.25, 1, 1));
        assert!(raster(1.25, 1, 1).contains("1.25"));
    }

    /// **The clean sentence is the status bar's, not a second one.**
    ///
    /// The property this module's header is about, asserted rather than
    /// promised: two surfaces describing one raster must not be able to say
    /// different things about it.
    #[test]
    fn the_clean_sentence_is_the_one_the_status_bar_uses() {
        assert_eq!(clean(), crate::text::status::diagnostics_clean());
    }

    /// **No `(s)` and no `oddity/oddities`**, in any of the four cases.
    ///
    /// The defect this pins was in the shipped window for one run before it was
    /// read: *"0 structural oddity/oddities … and 0 section(s)"*. Every
    /// `diagnostics_*` entry in [`crate::text::status`] already spells both
    /// forms, so this is the catalog's own convention being kept rather than a
    /// new rule — and the assertion is on the *characters*, because that is
    /// what an operator sees and what a later "just make it shorter" edit would
    /// reintroduce.
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
    ///
    /// [`crate::text::status::diagnostics_clean`]'s argument, applied to the
    /// line beneath it: a true answer that reads as an unfilled template is
    /// worse than no line at all, because the operator cannot tell which it is.
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
    ///
    /// A `match` over a unit-variant enum is the shape that reads as obviously
    /// correct and is the shape a copy-paste edit silently collapses: two arms
    /// returning the same string still compiles, still passes a test that only
    /// checks "the answer is non-empty", and hands the operator a sentence
    /// about the wrong document. Pairwise inequality is the only assertion that
    /// can fail for that.
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
    ///
    /// R9's rule applied to prose. `page_blend_space_source` decides what to do
    /// when the page group declares nothing AND the document carries a
    /// resolvable output intent; on a page whose own `/Group` named a space it
    /// is not consulted at all. A sentence sending the operator to Settings for
    /// a page Settings cannot affect is the prose form of a disabled button,
    /// and it costs more than a disabled button because he goes and looks.
    ///
    /// ★ Asserted BOTH ways. The positive half alone would pass a catalog that
    /// named the setting in all three; the negative half alone would pass one
    /// that named it in none.
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
    ///
    /// The failure this pins is the one-word readout - `CMYK` / `RGB` - which
    /// looks tidy in a report and is unreadable next to a duration, because
    /// nothing on the line says what the acronym is a property OF.
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
