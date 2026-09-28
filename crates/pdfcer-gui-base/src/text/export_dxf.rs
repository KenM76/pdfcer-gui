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
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/export_dxf.md`.

use pdfcer_core::export::dxf::{DxfOutcome, DxfUnits, DxfVersion};

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Export to DXF"
}

/// The paragraph under the title.
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

/// pdfcer inferred the scale, and this says **where from**.
#[must_use]
pub fn scale_from_group(scale: f64, group: &str, agreeing: usize) -> String {
    let head = format!("1 unit on paper is {scale} in the real drawing, from the group {group}");
    if agreeing > 1 {
        format!("{head} — and {agreeing} calibrated groups on this page agree.")
    } else {
        format!("{head}.")
    }
}

/// Nothing on the page carries a scale.
#[must_use]
pub const fn scale_uncalibrated() -> &'static str {
    "pdfcer cannot tell what this page is drawn at — none of the dimensions you \
     have drawn on it carries a scale. Exporting now writes it at paper size, \
     which is a choice rather than a measurement. Set a scale first, or type \
     the one you know below."
}

/// Calibrated groups disagree.
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

/// The label on the scale-group picker.
#[must_use]
pub const fn group_label() -> &'static str {
    "Take the scale from"
}

/// The picker's text when the scale was typed rather than taken from a group.
#[must_use]
pub const fn group_typed() -> &'static str {
    "the scale typed below"
}

/// One calibrated scale group in the picker: `plan — 1 Inches = 20 Feet`.
#[must_use]
pub fn group_choice(group: &str, paper_unit: &str, real: f64, real_unit: &str) -> String {
    let real = format!("{real:.4}");
    let real = real.trim_end_matches('0').trim_end_matches('.');
    format!("{group} — 1 {paper_unit} = {real} {real_unit}")
}

/// Hover on the scale-group picker.
#[must_use]
pub const fn group_hover() -> &'static str {
    "Every scale group in this document that has a scale set. Choosing one \
     writes its scale and its units into the row below."
}

/// The label in front of the ratio row.
#[must_use]
pub const fn ratio_label() -> &'static str {
    "Scale"
}

/// What the ratio row means.
#[must_use]
pub const fn ratio_hint() -> &'static str {
    "As the title block states it: 1 in on paper = 20 ft in reality. The same \
     unit on both sides is a plain ratio, 1 : 100. The DXF is written full size \
     in the units chosen below."
}

/// The heading over the unit choice.
#[must_use]
pub const fn units_heading() -> &'static str {
    "Units"
}

/// A DXF unit's name.
#[must_use]
pub const fn units_name(units: DxfUnits) -> &'static str {
    // No wildcard, and that is worth noting rather than assuming: `DxfUnits`
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

/// The label beside the DXF version choice.
#[must_use]
pub const fn version_label() -> &'static str {
    "DXF version"
}

/// A DXF version's name in the drop-down.
#[must_use]
pub const fn version_name(version: DxfVersion) -> &'static str {
    // `DxfVersion` is `#[non_exhaustive]`; a version added upstream shows its
    // `$ACADVER` string until it is given a name here.
    match version {
        DxfVersion::R12 => "R12 — any CAD program or plotter",
        DxfVersion::R2000 => "R2000",
        DxfVersion::R2004 => "R2004 — AutoCAD LT 2004 and later",
        other => other.acadver(),
    }
}

/// What the chosen version costs, under the drop-down.
#[must_use]
pub const fn version_hint(version: DxfVersion) -> &'static str {
    match version {
        DxfVersion::R12 => {
            "R12 has no splines, so curves that are not circles become polylines, and it cannot record units: the program opening it must be told."
        }
        _ => "Curves stay curves and the file records its units.",
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

/// Why arc fitting is on, in bytes.
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
#[must_use]
pub const fn no_geometry() -> &'static str {
    "pdfcer has not read this page's geometry yet, or could not. Nothing was \
     exported."
}

// ---------------------------------------------------------------------------
// After the fact
// ---------------------------------------------------------------------------

/// **What the export produced, and what it left behind.**
#[must_use]
pub fn exported(path: &str, outcome: &DxfOutcome, units: DxfUnits) -> Vec<String> {
    let DxfOutcome {
        polylines,
        circles,
        arcs,
        splines,
        skipped_text,
        skipped_images,
        unreadable_text,
        splines_flattened,
        units_undeclared,
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
    if splines_flattened > 0 {
        out.push(match splines_flattened {
            1 => "1 curve became a polyline, because R12 has no splines.".to_owned(),
            n => format!("{n} curves became polylines, because R12 has no splines."),
        });
    }
    if units_undeclared {
        out.push(format!(
            "R12 cannot record units. The drawing is in {}; set that when you open it.",
            units_name(units).to_lowercase()
        ));
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

    /// **An uncalibrated page is told that 1:1 is a CHOICE.**
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

    /// The two text counts stay apart.
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
            DxfUnits::Inches,
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
    #[test]
    fn a_skipped_picture_names_the_formats_limit_not_pdfcers() {
        let note = exported(
            "a.dxf",
            &DxfOutcome {
                polylines: 1,
                skipped_images: 2,
                ..DxfOutcome::default()
            },
            DxfUnits::Inches,
        );
        let joined = note.join(" | ");
        assert!(joined.contains("2 pictures"), "{joined}");
        assert!(joined.contains("no way to carry a raster"), "{joined}");
    }

    /// An R12 export says what the version could not carry, and in which units
    /// the drawing is.
    #[test]
    fn an_r12_export_discloses_flattened_curves_and_its_units() {
        let joined = exported(
            "a.dxf",
            &DxfOutcome {
                polylines: 1,
                splines_flattened: 3,
                units_undeclared: true,
                ..DxfOutcome::default()
            },
            DxfUnits::Millimetres,
        )
        .join(" | ");
        assert!(joined.contains("3 curves became polylines"), "{joined}");
        assert!(joined.contains("in millimetres"), "{joined}");
    }

    /// Every version the drop-down offers has a name of its own.
    #[test]
    fn every_offered_version_is_named() {
        for v in [DxfVersion::R12, DxfVersion::R2000, DxfVersion::R2004] {
            assert!(version_name(v).starts_with('R'), "{v:?}");
        }
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
                },
                DxfUnits::Inches,
            )
            .len(),
            1
        );
    }
}
