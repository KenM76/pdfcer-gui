//! # `text::page_size` — the words for **changing the paper an open drawing
//! sits on**
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/page_size.md`.

use pdfcer_core::paper::PaperSize;

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Sheet size"
}

/// The standing rule, first line of the window, before any choice.
#[must_use]
pub const fn intro() -> &'static str {
    "This changes the paper, not the drawing. Nothing on the page moves, and \
     nothing is scaled to fit — so a sheet made smaller keeps its drawing where \
     it is and the part that no longer fits stops being on the page."
}

/// Heading for the "what these sheets are now" line.
#[must_use]
pub const fn now_heading() -> &'static str {
    "These sheets now"
}

/// Every picked sheet is the same, named size.
///
/// `name` comes from [`size_name`]; `w_pt`/`h_pt` are the resolved media box.
#[must_use]
pub fn now_uniform_named(count: usize, name: &str, w_pt: f64, h_pt: f64) -> String {
    format!(
        "{count} {}, all {name} — {w_pt:.0} × {h_pt:.0} pt.",
        sheets(count)
    )
}

/// Every picked sheet is the same size and it is not one pdfcer has a name for.
#[must_use]
pub fn now_uniform_unnamed(count: usize, w_pt: f64, h_pt: f64) -> String {
    format!(
        "{count} {}, all {w_pt:.2} × {h_pt:.2} pt — not a size pdfcer has a name for.",
        sheets(count)
    )
}

/// The picked sheets are **not** all the same size.
#[must_use]
pub fn now_mixed(count: usize, distinct: usize) -> String {
    format!(
        "{count} {} in {distinct} different sizes. Applying a size makes every one of them \
         that size.",
        sheets(count)
    )
}

/// "sheet" / "sheets" — the only plural this module needs.
fn sheets(count: usize) -> &'static str {
    if count == 1 { "sheet" } else { "sheets" }
}

/// Heading above the size list.
#[must_use]
pub const fn size_heading() -> &'static str {
    "New size"
}

/// One entry in the size list: its name and its portrait dimensions.
#[must_use]
pub fn size_entry(name: &str, size_pt: (f64, f64)) -> String {
    let width_mm = crate::units::whole_mm_from_points(size_pt.0);
    let height_mm = crate::units::whole_mm_from_points(size_pt.1);
    format!("{name} — {width_mm} × {height_mm} mm")
}

/// The last entry in the size list.
#[must_use]
pub const fn size_custom() -> &'static str {
    "Custom…"
}

/// The name of a standard sheet size.
#[must_use]
pub fn size_name(size: PaperSize) -> String {
    match size {
        PaperSize::A0 => "A0".to_owned(),
        PaperSize::A1 => "A1".to_owned(),
        PaperSize::A2 => "A2".to_owned(),
        PaperSize::A3 => "A3".to_owned(),
        PaperSize::A4 => "A4".to_owned(),
        PaperSize::A5 => "A5".to_owned(),
        PaperSize::A6 => "A6".to_owned(),
        PaperSize::Letter => "Letter".to_owned(),
        PaperSize::Legal => "Legal".to_owned(),
        PaperSize::Tabloid => "Tabloid".to_owned(),
        PaperSize::Executive => "Executive".to_owned(),
        PaperSize::AnsiA => "ANSI A".to_owned(),
        PaperSize::AnsiB => "ANSI B".to_owned(),
        PaperSize::AnsiC => "ANSI C".to_owned(),
        PaperSize::AnsiD => "ANSI D".to_owned(),
        PaperSize::AnsiE => "ANSI E".to_owned(),
        other => other.id().to_owned(),
    }
}

/// A named size plus its orientation, for the "these sheets now" line.
#[must_use]
pub fn size_name_oriented(name: &str, landscape: bool) -> String {
    if landscape {
        format!("{name} landscape")
    } else {
        format!("{name} portrait")
    }
}

/// Heading above the orientation radios.
#[must_use]
pub const fn orientation_heading() -> &'static str {
    "Orientation"
}

/// The portrait radio.
#[must_use]
pub const fn orientation_portrait() -> &'static str {
    "Portrait"
}

/// The landscape radio.
#[must_use]
pub const fn orientation_landscape() -> &'static str {
    "Landscape"
}

/// Label for the custom width field.
#[must_use]
pub const fn custom_width() -> &'static str {
    "Width"
}

/// Label for the custom height field.
#[must_use]
pub const fn custom_height() -> &'static str {
    "Height"
}

/// The sheet that will land, in both units.
#[must_use]
pub fn sheet_summary(w_pt: f64, h_pt: f64) -> String {
    // Points keep two decimals deliberately — this line's whole job is letting
    // an operator check the exact sheet against a CAD page setup, and a whole
    // point is not enough resolution for that. Millimetres are whole, through
    // the one table, because that is the number matched against a ream label.
    let w_mm = crate::units::whole_mm_from_points(w_pt);
    let h_mm = crate::units::whole_mm_from_points(h_pt);
    format!("New sheet: {w_pt:.2} × {h_pt:.2} pt ({w_mm} × {h_mm} mm).")
}

/// A custom size outside the range this window will make.
#[must_use]
pub fn custom_refused(min_mm: i64, max_mm: i64) -> String {
    format!(
        "A sheet must be between {min_mm} and {max_mm} mm on each edge. ISO 32000-1 Annex C.2 \
         advises 3 to 14,400 points ({min_mm} mm is just over its floor, {max_mm} mm is exactly \
         its ceiling); PDF 2.0 dropped the advice, so this is portability, not validity."
    )
}

// ---------------------------------------------------------------------------
// The measured consequence — the two sentences that do the real work
// ---------------------------------------------------------------------------

/// Heading above the outcome line and the diagram.
#[must_use]
pub const fn outcome_heading() -> &'static str {
    "What happens to the drawing"
}

/// Nothing drawn on the picked sheets falls outside the new paper.
#[must_use]
pub const fn fits() -> &'static str {
    "Everything drawn on these sheets fits inside the new paper. Nothing will fall off."
}

/// The drawing runs past the new paper, by how much and on which edges.
#[must_use]
pub fn overhang(left: f64, right: f64, bottom: f64, top: f64) -> String {
    let mut edges: Vec<String> = Vec::new();
    if right > 0.0 {
        edges.push(format!("{right:.0} pt past the right edge"));
    }
    if top > 0.0 {
        edges.push(format!("{top:.0} pt past the top"));
    }
    if left > 0.0 {
        edges.push(format!("{left:.0} pt past the left edge"));
    }
    if bottom > 0.0 {
        edges.push(format!("{bottom:.0} pt past the bottom"));
    }
    format!(
        "The drawing runs {}. That part stops being on the page — it is not removed from the \
         file, but no reader will show it and any other tool is allowed to discard it. pdfcer \
         will not shrink the drawing to fit.",
        join_and(&edges)
    )
}

/// `["a", "b", "c"]` → `"a, b and c"`.
fn join_and(parts: &[String]) -> String {
    match parts {
        [] => String::new(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// The drawn extent of at least one picked sheet could not be read.
#[must_use]
pub fn overhang_unmeasurable(unread: usize, total: usize) -> String {
    format!(
        "pdfcer could not read what is drawn on {unread} of these {total} sheets, so it cannot \
         say whether anything falls outside the new paper. The paper will still change; check \
         those sheets afterwards."
    )
}

/// The boundary on what the fits/overhang measurement covers.
///
#[must_use]
pub const fn annots_not_counted() -> &'static str {
    "Measured on what is drawn on the page. Comments, form fields and ce dimensions keep their \
     positions too and are not counted here."
}

/// The picked sheets are already that size.
#[must_use]
pub const fn no_change() -> &'static str {
    "These sheets are already that size. Nothing will be written."
}

/// The picked sheets' lower-left corners disagree, so the new sheet is placed
/// at the origin.
#[must_use]
pub const fn origin_differs() -> &'static str {
    "These sheets do not all start at the same corner, so the new paper is placed at the \
     origin. On the sheets that started elsewhere, the paper moves relative to the drawing."
}

// -- the diagram's legend ---------------------------------------------------

/// Legend: the outline of the sheets as they are.
#[must_use]
pub const fn legend_now() -> &'static str {
    "now"
}

/// Legend: the outline of the sheet that will land.
#[must_use]
pub const fn legend_new() -> &'static str {
    "new"
}

/// Legend: the extent of what is drawn on the picked sheets.
#[must_use]
pub const fn legend_drawn() -> &'static str {
    "drawing"
}

// -- the two buttons --------------------------------------------------------

/// The Cancel button.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// The commit button.
#[must_use]
pub const fn apply() -> &'static str {
    "Change sheet size"
}

/// The commit button's tooltip.
#[must_use]
pub const fn apply_tooltip() -> &'static str {
    "Write the new sheet size onto the picked pages. One Undo reverses the whole set, however \
     many sheets it changed."
}

// ---------------------------------------------------------------------------
// Rule-4 disclosures, raised AFTER the commit
// ---------------------------------------------------------------------------

/// The sheet lost area on `n` pages.
#[must_use]
pub fn disclosure_lost_area(n: usize) -> String {
    format!(
        "{n} {} lost area. pdfcer removed nothing, but any other tool the file passes through \
         is allowed to discard what is now outside the paper — so Undo reverses this here, and \
         nothing reverses it after a round trip.",
        sheets(n)
    )
}

/// A `/CropBox` on `n` pages is no longer inside the new media box.
#[must_use]
pub fn disclosure_crop_outside(n: usize) -> String {
    format!(
        "{n} {} carry a crop box bigger than the new paper. pdfcer left it alone; every reader \
         shows the smaller of the two, so the visible area is the new paper.",
        sheets(n)
    )
}

/// `n` pages carry a crop box smaller than their new paper, so the page on
/// screen keeps the size it had.
#[must_use]
pub fn disclosure_crop_inside(n: usize) -> String {
    format!(
        "{n} {} carry a crop box smaller than the new paper, and every reader shows only the \
         crop box, so the page looks the size it was. The paper did change; pdfcer cannot \
         move a crop box yet.",
        sheets(n)
    )
}

/// `n` pages' own `/MediaBox` entry was removed because an ancestor already
/// says the same thing.
#[must_use]
pub fn disclosure_inherited(n: usize) -> String {
    format!(
        "{n} {} now take their size from the document instead of carrying their own, because \
         the document already said that size.",
        sheets(n)
    )
}

/// The new size is outside ISO 32000-1 Annex C.2's recommended range.
#[must_use]
pub fn disclosure_size_advisory(n: usize, below: bool) -> String {
    let which = if below {
        "smaller than the 3-point minimum"
    } else {
        "bigger than the 14,400-point maximum"
    };
    format!(
        "{n} {} are now {which} ISO 32000-1 Annex C.2 recommends. Written as asked — PDF 2.0 \
         dropped the advice entirely — but some readers may not handle it.",
        sheets(n)
    )
}

/// The engine refused the change because the document is certified.
#[must_use]
pub const fn refused_certified() -> &'static str {
    "This document carries a certification signature that does not permit structural page \
     changes, so its sheet size cannot be changed without breaking the certification."
}

/// The engine refused a degenerate rectangle.
#[must_use]
pub const fn refused_degenerate() -> &'static str {
    "That sheet has no area, so there is nothing to write."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The crop-not-scale rule is stated, in the operator's words, in
    /// the first line of the window.**
    #[test]
    fn the_intro_says_the_drawing_does_not_move_and_is_not_scaled() {
        let intro = intro();
        assert!(
            intro.contains("not the drawing"),
            "the intro must say the paper changes and the drawing does not: {intro}"
        );
        assert!(
            intro.contains("scaled"),
            "the intro must say nothing is scaled to fit — the belief every other page-size \
             control in the world installs: {intro}"
        );
    }

    /// **The overhang line names every edge it was given, and only those.**
    #[test]
    fn the_overhang_line_names_exactly_the_edges_that_overhang() {
        let right_only = overhang(0.0, 1636.0, 0.0, 0.0);
        assert!(
            right_only.contains("1636 pt past the right edge"),
            "{right_only}"
        );
        assert!(!right_only.contains("past the top"), "{right_only}");
        assert!(!right_only.contains("past the bottom"), "{right_only}");
        assert!(!right_only.contains("past the left edge"), "{right_only}");

        let two = overhang(0.0, 1636.0, 0.0, 45.0);
        assert!(two.contains("1636 pt past the right edge"), "{two}");
        assert!(two.contains("45 pt past the top"), "{two}");
        assert!(
            two.contains(" and "),
            "two edges must be joined with `and`: {two}"
        );
    }

    /// **The overhang line refuses the reading that pdfcer will shrink to
    /// fit**, because that is the operator's default expectation and the one
    /// this window exists to correct.
    #[test]
    fn the_overhang_line_says_pdfcer_will_not_shrink_the_drawing() {
        let line = overhang(0.0, 10.0, 0.0, 0.0);
        assert!(
            line.contains("not shrink the drawing to fit"),
            "the overhang line must say what pdfcer will NOT do: {line}"
        );
    }

    /// **The "it fits" line and the "not counted" line are separate, and
    /// both are needed.**
    ///
    #[test]
    fn the_promise_is_bounded_on_screen() {
        assert!(fits().contains("drawn"), "{}", fits());
        let bound = annots_not_counted();
        assert!(bound.contains("Comments"), "{bound}");
        assert!(bound.contains("ce dimensions"), "{bound}");
    }

    /// **R8b rule 15: this catalogue never writes a bare "dimension".**
    #[test]
    fn no_string_in_this_catalogue_writes_a_bare_dimension() {
        let strings: Vec<String> = vec![
            intro().to_owned(),
            fits().to_owned(),
            annots_not_counted().to_owned(),
            no_change().to_owned(),
            origin_differs().to_owned(),
            outcome_heading().to_owned(),
            now_heading().to_owned(),
            size_heading().to_owned(),
            apply().to_owned(),
            apply_tooltip().to_owned(),
            refused_certified().to_owned(),
            refused_degenerate().to_owned(),
            overhang(1.0, 2.0, 3.0, 4.0),
            overhang_unmeasurable(1, 2),
            disclosure_lost_area(2),
            disclosure_crop_outside(2),
            disclosure_inherited(2),
            disclosure_size_advisory(2, true),
            disclosure_size_advisory(2, false),
        ];
        for s in strings {
            let lower = s.to_lowercase();
            let mut from = 0;
            while let Some(at) = lower[from..].find("dimension") {
                let at = from + at;
                assert!(
                    lower[..at].ends_with("ce ") || lower[..at].ends_with("pdf "),
                    "R8b rule 15: a bare `dimension` at byte {at} of: {s}"
                );
                from = at + "dimension".len();
            }
        }
    }

    /// **A standard size reads back as its own millimetres**, through the
    /// engine's table rather than a hand-rounded copy of it.
    #[test]
    fn a_named_size_reads_back_as_its_own_millimetres() {
        let entry = size_entry(&size_name(PaperSize::A1), PaperSize::A1.size_pt());
        assert!(entry.contains("A1"), "{entry}");
        assert!(entry.contains("594"), "A1 is 594 mm wide: {entry}");
        assert!(entry.contains("841"), "A1 is 841 mm tall: {entry}");
    }

    /// **The lost-area disclosure states the asymmetry**, which is the whole
    /// reason it is a disclosure rather than a status count.
    #[test]
    fn the_lost_area_disclosure_says_undo_works_here_and_not_afterwards() {
        let note = disclosure_lost_area(3);
        assert!(note.contains("3 sheets"), "{note}");
        assert!(note.contains("Undo"), "{note}");
        assert!(
            note.contains("round trip"),
            "the point of the disclosure is that the loss becomes permanent elsewhere: {note}"
        );
    }

    /// **Singular and plural, because "1 sheets lost area" is the kind of
    /// thing that survives review forever.**
    #[test]
    fn one_sheet_is_singular() {
        assert!(
            disclosure_lost_area(1).contains("1 sheet lost"),
            "{}",
            disclosure_lost_area(1)
        );
        assert!(
            disclosure_lost_area(2).contains("2 sheets lost"),
            "{}",
            disclosure_lost_area(2)
        );
    }

    /// **No entry in the size list reads like an identifier.**
    #[test]
    fn no_shipped_size_falls_through_to_its_machine_id() {
        for size in PaperSize::ALL {
            let name = size_name(*size);
            assert!(
                !name.contains('-') || name.starts_with("ANSI"),
                "{name} looks like a machine id rather than a name"
            );
            assert_ne!(name, size.id(), "{name} fell through to the wildcard arm");
        }
    }

    /// **The certified refusal names the cause**, because a refusal an
    /// operator cannot act on is indistinguishable from a bug.
    #[test]
    fn the_certified_refusal_says_why() {
        let refusal = refused_certified();
        assert!(refusal.contains("certification"), "{refusal}");
        assert!(
            refusal.contains("cannot be changed"),
            "it must say what did not happen, not only what is true of the file: {refusal}"
        );
    }
}
