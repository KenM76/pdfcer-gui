//! # `text::export_dxf` — the words the Export-DXF window shows
//!
//! ## Rule 15, and it decides a sentence here
//!
//! The scale this window offers is inferred from the **ce dimensions** the
//! operator has drawn — the ones pdfcer authors. It is *not* read from **pdf
//! dimensions**, the CAD-exported page content pdfcer reads and must not alter,
//! and it could not be: those are anonymous vector geometry with no recorded
//! measurement. So the copy says *"the dimensions you have drawn on it"*, which
//! is both unambiguous and the honest boundary — an operator who has drawn none
//! is told pdfcer has no evidence rather than being given a number.
//!
//! ## ★ The one sentence this whole window exists for
//!
//! `pdfcer-core`'s own doc on `DxfOptions::scale`:
//!
//! > **This is the field the whole feature turns on.** Every generic PDF→DXF
//! > converter exports at paper scale and says nothing, so a **1:2 detail
//! > arrives at half size and looks plausible.**
//!
//! *Looks plausible* is the whole problem. A DXF at the wrong scale opens
//! cleanly, measures consistently, and is wrong — and the person who finds out
//! is whoever cuts from it. Everything below is arranged so that the scale is
//! either stated with its evidence, or stated as unknown.
//!
//! ## ★ Three answers, never two
//!
//! `DxfScaleSuggestion` is deliberately not an `Option<f64>`, and this catalog
//! keeps its three cases apart because they ask different things of an
//! operator:
//!
//! | | what it means | what the window does |
//! |---|---|---|
//! | `Calibrated` | every calibrated group agrees | states the number **and the group it came from** |
//! | `Uncalibrated` | nothing on this page carries a scale | says pdfcer has no evidence, and that 1:1 is a *choice* rather than a finding |
//! | `Conflicting` | calibrated groups disagree | lists every candidate and makes the operator pick |
//!
//! Collapsing `Uncalibrated` into "1.0" is exactly the silent paper-scale
//! export the feature exists to beat.

use pdfcer_core::export::dxf::{DxfOutcome, DxfUnits};

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Export to DXF"
}

/// The paragraph under the title.
///
/// Says what a DXF **is not**, first. A drafter opening one in SOLIDWORKS
/// expects the drawing; what arrives is its vector geometry, and the sentence
/// that saves a support question is the one about what was left behind.
#[must_use]
pub const fn intro() -> &'static str {
    "The page's vector geometry is written as a DXF. Pictures cannot be \
     carried at all, and text is carried only if you ask for it — this window \
     says how much of each was on the page."
}

/// The page-number line.
#[must_use]
pub fn page_line(page_number: usize) -> String {
    format!("Page {page_number}, the one on screen")
}

/// The heading over the scale controls.
#[must_use]
pub const fn scale_heading() -> &'static str {
    "Scale"
}

/// ★ pdfcer inferred the scale, and this says **where from**.
///
/// The group's name is in the sentence because the number alone is a claim the
/// operator cannot check. *"1 paper unit is 50 real units"* is unverifiable;
/// *"from the group Site plan"* points at something they set up themselves and
/// can go and look at.
///
/// `agreeing` above one is stated as corroboration rather than as a second
/// finding: two groups agreeing is the same answer, arrived at twice, and
/// saying so is worth a clause because it is the case an operator is most
/// entitled to trust.
#[must_use]
pub fn scale_from_group(scale: f64, group: &str, agreeing: usize) -> String {
    let head = format!("1 unit on paper is {scale} in the real drawing, from the group {group}");
    if agreeing > 1 {
        format!("{head} — and {agreeing} calibrated groups on this page agree.")
    } else {
        format!("{head}.")
    }
}

/// ★ Nothing on the page carries a scale.
///
/// **The most important string in this catalog.** The alternative — defaulting
/// to 1.0 and saying nothing — is precisely what `pdfcer-core` describes every
/// generic converter as doing, and its consequence is a detail arriving at the
/// wrong size while looking perfectly ordinary.
///
/// So it says three things: that pdfcer does not know, that 1:1 is therefore a
/// **choice** rather than a finding, and what the operator can do to make it a
/// finding instead.
#[must_use]
pub const fn scale_uncalibrated() -> &'static str {
    "pdfcer cannot tell what this page is drawn at — none of the dimensions you \
     have drawn on it carries a scale. Exporting now writes it at paper size, \
     which is a choice rather than a measurement. Set a scale first, or type \
     the one you know below."
}

/// ★ Calibrated groups disagree.
///
/// Not refused, and not resolved by pdfcer picking one. A sheet holding a 1:50
/// plan and a 1:5 detail is a **correct drawing**, and the disagreement is a
/// true statement about it: one DXF scale cannot serve both. The operator is
/// the only one who knows which half they are exporting for.
#[must_use]
pub fn scale_conflicting(count: usize) -> String {
    format!(
        "The dimensions on this page are calibrated at {count} different \
         scales, so one DXF cannot be right for all of them. Choose the one \
         this export is for."
    )
}

/// One candidate in the conflicting list.
#[must_use]
pub fn scale_candidate(scale: f64, group: &str) -> String {
    format!("{scale} — {group}")
}

/// The label on the scale field itself.
#[must_use]
pub const fn scale_label() -> &'static str {
    "Real units per paper unit"
}

/// The heading over the unit choice.
#[must_use]
pub const fn units_heading() -> &'static str {
    "Units"
}

/// A DXF unit's name.
///
/// ★ pdfcer writes only inches and millimetres, and `DxfUnits::for_unit` maps
/// feet onto inches and metres onto millimetres — *"the NUMBERS stay exact
/// either way … this choice affects only what the header declares"*. The
/// wording therefore names what the file will **say it is**, not what the
/// operator measured in, because those legitimately differ and only the first
/// is what this control sets.
#[must_use]
pub const fn units_name(units: DxfUnits) -> &'static str {
    // ★ No wildcard, and that is worth noting rather than assuming: `DxfUnits`
    // is NOT `#[non_exhaustive]`, unlike four of the five engine enums this
    // shell touched today — so this match really is exhaustive and a third
    // unit added upstream really would fail to compile here. The distinction
    // is invisible at the call site and decides whether a fallback arm is a
    // safety net or dead code; the compiler settled it by rejecting one.
    match units {
        DxfUnits::Millimetres => "Millimetres",
        DxfUnits::Inches => "Inches",
    }
}

/// The heading over the geometry options.
#[must_use]
pub const fn geometry_heading() -> &'static str {
    "Geometry"
}

/// The arc-fitting switch.
#[must_use]
pub const fn fit_arcs() -> &'static str {
    "Write circles and arcs where the curves are circular"
}

/// ★ Why arc fitting is on, in bytes.
///
/// The engine measured it: *"not recognising them is what produced a measured
/// **767 KB for forty washers**."* PDF has no arc primitive, so every hole and
/// fillet arrives as cubic Béziers, and a converter that emits them as splines
/// produces a file that is enormous and that no CAD package will let you snap
/// to a centre in.
#[must_use]
pub const fn fit_arcs_hint() -> &'static str {
    "PDF has no arcs — every hole and fillet is stored as curves — so without \
     this a drawing of forty washers becomes hundreds of kilobytes of splines \
     with no centres to snap to."
}

/// The text switch.
#[must_use]
pub const fn write_text() -> &'static str {
    "Write the page's text as DXF text"
}

/// What the text switch costs either way.
#[must_use]
pub const fn write_text_hint() -> &'static str {
    "Text arrives as separate entities on their own layer, not as part of the \
     geometry. Turn it off for a file you are going to cut from."
}

/// The commit button.
#[must_use]
pub const fn export_button() -> &'static str {
    "Export…"
}

/// The cancel button.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

/// The title of the native save dialog.
#[must_use]
pub const fn save_dialog_title() -> &'static str {
    "Save the DXF"
}

/// A page whose decomposition is not available.
///
/// Reachable while the page is still being read, and on a page whose content
/// streams could not be resolved. Says which, because *"not yet"* and *"not at
/// all"* are different situations and only the first is worth waiting for.
#[must_use]
pub const fn no_geometry() -> &'static str {
    "pdfcer has not read this page's geometry yet, or could not. Nothing was \
     exported."
}

// ---------------------------------------------------------------------------
// After the fact
// ---------------------------------------------------------------------------

/// ★ **What the export produced, and what it left behind.**
///
/// The disclosure half, and the `skipped` clauses are the reason it exists.
/// `pdfcer-core` states the case in its own field doc:
///
/// > an operator whose drawing was half annotation gets a DXF that looks like
/// > the geometry went missing, and *"the labels are not in this file"* is a
/// > sentence they need **before** they open it in SOLIDWORKS, not after.
///
/// ★ `skipped_text` and `unreadable_text` are kept apart, exactly as the engine
/// keeps them apart, because they ask different things:
///
/// - **skipped** — the operator turned text off. Nothing is wrong.
/// - **unreadable** — pdfcer *could not read it*: no font resolver in scope, or
///   an `Identity-H` encoding with no `/ToUnicode` whose codes map to nothing.
///   That is a fact about the source PDF and the reason a DXF is missing labels
///   the operator can plainly see on screen.
///
/// Rolling them together would let the second hide inside the first, which is
/// the failure mode the engine wrote a paragraph to prevent.
/// # Why it takes the ENGINE's outcome rather than eight counts
///
/// It was written as eight `usize` parameters, on this catalog's usual rule
/// that a wording function takes primitives so it can be tested without the
/// engine. Clippy's argument budget refused it, and the refusal was right for a
/// reason the rule does not cover: **these eight are not independent facts.**
/// They are one value — what the writer did — and a caller assembling them by
/// hand has eight chances to pass `skipped_text` where `unreadable_text`
/// belongs, which is exactly the pair whose confusion this function exists to
/// prevent.
///
/// `DxfOutcome` is `Default`, so the tests below still construct one in a line
/// without touching a document.
#[must_use]
pub fn exported(path: &str, outcome: &DxfOutcome) -> Vec<String> {
    let DxfOutcome {
        polylines,
        circles,
        arcs,
        splines,
        skipped_text,
        skipped_images,
        unreadable_text,
        ..
    } = *outcome;
    let mut out = vec![format!(
        "Exported to {path} — {polylines} lines, {circles} circles, {arcs} \
         arcs, {splines} splines."
    )];
    if skipped_images > 0 {
        out.push(match skipped_images {
            1 => "1 picture on this page is not in the DXF — the format has no \
                  way to carry a raster."
                .to_owned(),
            n => format!(
                "{n} pictures on this page are not in the DXF — the format has \
                 no way to carry a raster."
            ),
        });
    }
    if skipped_text > 0 {
        out.push(match skipped_text {
            1 => "1 piece of text was left out, as you asked.".to_owned(),
            n => format!("{n} pieces of text were left out, as you asked."),
        });
    }
    if unreadable_text > 0 {
        out.push(match unreadable_text {
            1 => "1 piece of text could not be read out of this PDF, so it is \
                  missing from the DXF even though you can see it on screen."
                .to_owned(),
            n => format!(
                "{n} pieces of text could not be read out of this PDF, so they \
                 are missing from the DXF even though you can see them on \
                 screen."
            ),
        });
    }
    out
}

/// The write failed.
#[must_use]
pub fn export_failed(detail: &str) -> String {
    format!("Nothing was written. {detail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **An uncalibrated page is told that 1:1 is a CHOICE.**
    ///
    /// The assertion this catalog exists for. `pdfcer-core` names the failure it
    /// prevents — every generic converter exports at paper scale and says
    /// nothing, so a 1:2 detail arrives at half size *looking plausible* — and
    /// the only defence is a sentence that refuses to present a default as a
    /// finding.
    #[test]
    fn an_uncalibrated_page_is_not_told_a_number() {
        let text = scale_uncalibrated();
        assert!(text.contains("cannot tell"), "{text}");
        assert!(text.contains("choice rather than a measurement"), "{text}");
        assert!(text.contains("Set a scale"), "it says what to do: {text}");
    }

    /// A calibrated scale is stated with the group it came from.
    ///
    /// The number alone is unverifiable. The group name points at something the
    /// operator set up themselves and can go and check.
    #[test]
    fn a_calibrated_scale_names_its_evidence() {
        let one = scale_from_group(50.0, "Site plan", 1);
        assert!(one.contains("Site plan"), "{one}");
        assert!(
            !one.contains("agree"),
            "one group is not corroboration: {one}"
        );

        let two = scale_from_group(50.0, "Site plan", 3);
        assert!(two.contains("3 calibrated groups"), "{two}");
    }

    /// ★ The two text counts stay apart.
    ///
    /// `skipped` is *you asked*; `unreadable` is *pdfcer could not read it*. The
    /// second is a fact about the source PDF and the reason labels the operator
    /// can see on screen are absent from the file — and rolling them together
    /// would let it hide inside a sentence about their own choice.
    #[test]
    fn skipped_text_and_unreadable_text_are_different_sentences() {
        let both = exported(
            "a.dxf",
            &DxfOutcome {
                polylines: 1,
                skipped_text: 4,
                unreadable_text: 2,
                ..DxfOutcome::default()
            },
        );
        let joined = both.join(" | ");
        assert!(joined.contains("as you asked"), "{joined}");
        assert!(joined.contains("could not be read"), "{joined}");
        assert!(
            joined.contains("even though you can see"),
            "the unreadable case must say why it is surprising: {joined}"
        );
    }

    /// A picture on the page is always mentioned, and says why.
    ///
    /// *"The format has no way to carry a raster"* rather than *"images are not
    /// supported"*: the first is a fact about DXF that no future version of
    /// pdfcer will change, and the second reads as a gap somebody might fix.
    #[test]
    fn a_skipped_picture_names_the_formats_limit_not_pdfcers() {
        let note = exported(
            "a.dxf",
            &DxfOutcome {
                polylines: 1,
                skipped_images: 2,
                ..DxfOutcome::default()
            },
        );
        let joined = note.join(" | ");
        assert!(joined.contains("2 pictures"), "{joined}");
        assert!(joined.contains("no way to carry a raster"), "{joined}");
    }

    /// A clean export says one thing.
    #[test]
    fn nothing_left_behind_produces_exactly_one_sentence() {
        assert_eq!(
            exported(
                "a.dxf",
                &DxfOutcome {
                    polylines: 10,
                    circles: 2,
                    arcs: 3,
                    ..DxfOutcome::default()
                }
            )
            .len(),
            1
        );
    }
}
