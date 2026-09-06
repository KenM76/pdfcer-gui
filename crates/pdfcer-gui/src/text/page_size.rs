//! # `text::page_size` — the words for **changing the paper an open drawing
//! sits on**
//!
//! The catalogue for [`crate::dialogs::page_size`] and for the disclosures
//! [`crate::app::actions::pagesize`] raises after the commit. R1: every string
//! a human can read is defined here and nowhere else.
//!
//! ## ★★★ The one thing this whole catalogue exists to say
//!
//! **Changing a `/MediaBox` changes the paper. It does not move, scale or
//! shrink anything drawn on the page.**
//!
//! That sentence is the difference between the operator getting what he
//! expected and losing his title block, and it is *not* obvious — every office
//! application in the world has a "page size" control that reflows, and the
//! only one that behaves like this is a drawing program. Measured on
//! `fixtures/a1-titleblock.pdf` on 2026-09-06, through the engine's own
//! `set-page-size`: every glyph run's coordinate is **byte-identical** before
//! and after an A1 → A4 resize. The title block sits at x 1831–2207 pt, and an
//! A4 sheet stops at 595.28. It is *entirely off the paper*, still in the
//! content stream, invisible to every reader.
//!
//! So this catalogue states the rule three times over, in three registers, and
//! that repetition is deliberate rather than sloppy:
//!
//! 1. [`intro`] — the standing rule, at the top of the window, before any
//!    choice is made.
//! 2. [`fits`] / [`overhang`] — the **measured consequence for these sheets and
//!    this size**, recomputed as the operator changes the size. This is the one
//!    that does the work: a rule is a thing to agree with, a measurement is a
//!    thing to act on.
//! 3. [`disclosure_lost_area`] — after the commit, in the status bar, because
//!    rule 4 says a consequence the operator cannot see on the page is owed a
//!    sentence off it.
//!
//! ## ★★ Rule 15 — "dimension" is never written bare here
//!
//! Two different things on a CAD sheet are called dimensions and a paper change
//! affects them differently, so R8b rule 15 forbids the bare word:
//!
//! | | what it is | what a paper change does to it |
//! |---|---|---|
//! | **pdf dimension** | the printed measurement the CAD exporter drew — page content pdfcer reads and must not silently alter | **nothing.** It is content; it does not move. It can end up off the sheet. |
//! | **ce dimension** | the measurement pdfcer itself authored — an annotation plus a `/PieceInfo` sidecar | **nothing.** Measured 2026-09-06: `/Rect`, sidecar and the printed value (`400.00 pt`) are identical across an A1 → A4 resize. |
//!
//! ★ The second row is *why the honest answer is "nothing moves"* rather than a
//! hedge. A verb that scaled the drawing to fit would have to rescale every ce
//! dimension group's calibration to keep its numbers true, and would silently
//! falsify every one it missed. This verb cannot get that wrong because it does
//! not touch them — and [`intro`] says so in the operator's own terms rather
//! than in that argument.
//!
//! ## Register
//!
//! Short, and assumes competence, matching [`crate::text::new_document`]. He
//! drafts in SolidWorks; he knows what A1 is. What he does not know — because
//! no other application behaves this way — is which of *crop* and *shrink* he
//! is about to get, so that is the sentence that gets the words.

use pdfcer_core::paper::PaperSize;

/// Points per millimetre — 72 points per inch ÷ 25.4 mm per inch.
///
/// A definition, restated here for the same reason
/// [`crate::text::new_document`] restates it: a four-character constant hoisted
/// into a public API to give two unrelated callers a common dependency is not a
/// saving, and the value cannot drift because it is not a measurement.
const PT_PER_MM: f64 = 72.0 / 25.4;

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Sheet size"
}

/// ★★★ The standing rule, first line of the window, before any choice.
///
/// # Why this is the first thing on screen and not a footnote
///
/// Because it is the one belief the operator arrives with that is wrong. Every
/// "page size" control he has ever used — Word, LibreOffice, a print dialog —
/// reflows or scales. This one does not, and a window that let him discover
/// that from the result would be the *"fuzzy, never sneaky"* failure in its
/// purest form: he would get a file that is exactly what he asked for and
/// nothing like what he wanted.
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
///
/// ★ Said out loud rather than shown as bare numbers with no comment. A CAD
/// exporter that writes 2,381.10 × 1,683.78 has produced an A1 sheet rounded to
/// two decimals by a units conversion, and an operator who sees "not a standard
/// size" learns something true about his own export pipeline.
#[must_use]
pub fn now_uniform_unnamed(count: usize, w_pt: f64, h_pt: f64) -> String {
    format!(
        "{count} {}, all {w_pt:.2} × {h_pt:.2} pt — not a size pdfcer has a name for.",
        sheets(count)
    )
}

/// The picked sheets are **not** all the same size.
///
/// ★ This is the state that makes a single "current size" readout a lie, and
/// the reason the dialog reads the operands rather than the current page. A
/// drawing set with one A3 detail sheet among nine A1s is the ordinary case,
/// not the exotic one.
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
    format!(
        "{name} — {:.0} × {:.0} mm",
        size_pt.0 / PT_PER_MM,
        size_pt.1 / PT_PER_MM
    )
}

/// The last entry in the size list.
#[must_use]
pub const fn size_custom() -> &'static str {
    "Custom…"
}

/// The name of a standard sheet size.
///
/// ★ The wildcard arm is load-bearing. `PaperSize` is `#[non_exhaustive]` and
/// its own docs say the table will grow (ARCH, JIS B, ISO B/C); a size the
/// engine adds must appear in the list with its machine id rather than making
/// this module fail to compile or, worse, silently vanish from the picker.
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
    "Width (mm)"
}

/// Label for the custom height field.
#[must_use]
pub const fn custom_height() -> &'static str {
    "Height (mm)"
}

/// The sheet that will land, in both units.
#[must_use]
pub fn sheet_summary(w_pt: f64, h_pt: f64) -> String {
    format!(
        "New sheet: {w_pt:.2} × {h_pt:.2} pt ({:.0} × {:.0} mm).",
        w_pt / PT_PER_MM,
        h_pt / PT_PER_MM
    )
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
// ★★★ The measured consequence — the two sentences that do the real work
// ---------------------------------------------------------------------------

/// Heading above the outcome line and the diagram.
#[must_use]
pub const fn outcome_heading() -> &'static str {
    "What happens to the drawing"
}

/// Nothing drawn on the picked sheets falls outside the new paper.
///
/// ★ This is the *only* wording in the window that promises anything, so it is
/// bounded exactly to what was measured — the union of the picked pages' drawn
/// vector extents, which is what `PageObjects::page_bbox` computes. See
/// [`overhang_unmeasurable`] for the case where even that could not be read,
/// and [`annots_not_counted`] for the boundary this sentence does not cover.
#[must_use]
pub const fn fits() -> &'static str {
    "Everything drawn on these sheets fits inside the new paper. Nothing will fall off."
}

/// The drawing runs past the new paper, by how much and on which edges.
///
/// # ★★★ Why this names the EDGES and the AMOUNT
///
/// Because "content will be cropped" is a warning and this is a
/// **measurement**, and the operator's decision turns on the difference. On his
/// own A1 title-block sheet the overhang is 1,636 pt off the right edge — the
/// width of the whole title block — and a number that large tells him
/// immediately that he has picked the wrong size. A generic warning would read
/// identically for a 2 pt overhang he does not care about.
///
/// The four amounts are in points because that is the unit the sheet is in and
/// the unit the numbers beside it are in; a millimetre conversion here would
/// make the reader do arithmetic to compare this line with the one above it.
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
///
/// ★ A stated boundary rather than a cheerful silence. A page whose content
/// stream will not decompose is exactly the page most likely to be a strange
/// export, and reporting "nothing falls off" for it would be a false negative
/// dressed as a measurement — which is the failure the engine's own
/// `MediaBoxChange` names as its residual and refuses to commit.
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
/// ★★ Stated on screen, permanently, beside the measurement — not buried in a
/// doc comment. `PageObjects::page_bbox` is the union of the **drawn** objects'
/// bounding boxes. Comments, form fields, stamps and ce dimensions are
/// annotations: separate objects with their own `/Rect`, which this sweep does
/// not walk. They keep their coordinates across a resize (measured 2026-09-06 —
/// a `Square` at 120,560–320,700 was still at 120,560–320,700 on an A6 sheet
/// 297 × 420, i.e. entirely off it), so they can fall off exactly as content
/// can, and the measurement above would not have said so.
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
///
/// # ★ Why this exists at all
///
/// §7.7.3.3 does not require a media box to start at `(0, 0)`, and imposition
/// output and cropped scans really do carry offset ones. `set_media_boxes`
/// takes **one** rectangle for the whole selection, so when the picked sheets
/// sit at different corners no single rectangle can preserve all of them; the
/// dialog anchors at the origin and says so, rather than moving the paper
/// relative to the drawing without a word.
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
///
/// ★★ The asymmetry is the part worth reporting and is quoted from the engine's
/// own reasoning: §14.11.2.1 says content outside the media box *"may safely be
/// discarded without affecting the meaning of the PDF file"* — an unusually
/// strong permission, because it asserts the discard is meaning-preserving. So
/// shrinking is reversible **in pdfcer**, by Undo, and is not reversible **in
/// the ecosystem**, once the file has been through any other tool.
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
///
/// ★ Disclosed, not repaired, and the engine's argument for that is quoted
/// because the operator would otherwise reasonably expect a fix: a conforming
/// reader *"shall treat the box as its intersection with the media box"*, so
/// clamping the entry would change no reader's output while rewriting an entry
/// the operator did not name.
#[must_use]
pub fn disclosure_crop_outside(n: usize) -> String {
    format!(
        "{n} {} carry a crop box bigger than the new paper. pdfcer left it alone; every reader \
         shows the smaller of the two, so the visible area is the new paper.",
        sheets(n)
    )
}

/// `n` pages' own `/MediaBox` entry was removed because an ancestor already
/// says the same thing.
///
/// ★ Worth a sentence rather than silence: the page is now sized by
/// inheritance, so a later change to the document's default size will move it
/// and a sibling's will not. That is a real, invisible difference in what the
/// next edit does.
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
///
/// ★★ Worded rather than traced, because `RESUME.md`'s standing cross-cutting
/// defect is that *"every engine refusal reaches the operator as SILENCE"*.
/// Measured 2026-09-06 on `fixtures/certified-comments.pdf`: the engine refuses
/// with `CertificationForbidsChange` and this is what that has to read as.
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

    /// ★★★ **The crop-not-scale rule is stated, in the operator's words, in
    /// the first line of the window.**
    ///
    /// The single most important property of this whole catalogue, and the one
    /// a reword could quietly destroy: a future editor tidying [`intro`] for
    /// length could drop the clause that says nothing is scaled, and every
    /// test in the workspace would stay green while the window started lying
    /// by omission. This fails instead.
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

    /// ★★ **The overhang line names every edge it was given, and only those.**
    ///
    /// Both directions. A line that named all four edges regardless would read
    /// as alarming nonsense on a sheet that overhangs only to the right; one
    /// that named the first would silently under-report a drawing hanging off
    /// two edges, which is what a wrongly-oriented sheet produces.
    ///
    /// ⚠ **The absence assertions match `"past the top"`, not `"top"`**, and
    /// the reason is recorded because it cost a red run: the sentence contains
    /// the word *"s-top-s"*, so a bare substring test reported a line naming
    /// one edge as naming two. A negative assertion over prose has to match the
    /// **phrase the code emits**, or it is asserting something about English
    /// rather than about the program.
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

    /// ★ **The overhang line refuses the reading that pdfcer will shrink to
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

    /// ★★ **The "it fits" line and the "not counted" line are separate, and
    /// both are needed.**
    ///
    /// `fits()` is the only promise in this window. Measured 2026-09-06, an
    /// annotation keeps its `/Rect` across a resize exactly as content keeps
    /// its coordinates — so a sheet whose *drawing* fits can still lose a
    /// sticky note. The promise is therefore bounded on screen and not only in
    /// a doc comment: a bound nobody can read is not a bound.
    #[test]
    fn the_promise_is_bounded_on_screen() {
        assert!(fits().contains("drawn"), "{}", fits());
        let bound = annots_not_counted();
        assert!(bound.contains("Comments"), "{bound}");
        assert!(bound.contains("ce dimensions"), "{bound}");
    }

    /// ★★★ **R8b rule 15: this catalogue never writes a bare "dimension".**
    ///
    /// A CAD sheet has two kinds and a paper change affects them differently:
    /// a **pdf dimension** is the printed measurement the exporter drew — page
    /// content pdfcer reads and must not silently alter — and a **ce
    /// dimension** is the one pdfcer authored. Both are unaffected here, for
    /// different reasons, and a sentence that said "dimensions" would be
    /// telling the operator something about a category that does not exist.
    ///
    /// Swept over **every** string this module can produce rather than over the
    /// one that happens to use the word today, because the rule binds the
    /// catalogue and not a line of it — and a future sentence added by someone
    /// who has not read R8b is exactly what this is for.
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

    /// ★ **A standard size reads back as its own millimetres**, through the
    /// engine's table rather than a hand-rounded copy of it.
    #[test]
    fn a_named_size_reads_back_as_its_own_millimetres() {
        let entry = size_entry(&size_name(PaperSize::A1), PaperSize::A1.size_pt());
        assert!(entry.contains("A1"), "{entry}");
        assert!(entry.contains("594"), "A1 is 594 mm wide: {entry}");
        assert!(entry.contains("841"), "A1 is 841 mm tall: {entry}");
    }

    /// ★ **The lost-area disclosure states the asymmetry**, which is the whole
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

    /// ★ **Singular and plural, because "1 sheets lost area" is the kind of
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

    /// ★ **No entry in the size list reads like an identifier.**
    ///
    /// The wildcard arm of [`size_name`] falls back to `PaperSize::id`, which is
    /// machine-facing (`ansi-d`). Every size the engine ships today must have a
    /// real name here; the fallback is for one it adds tomorrow, and this is
    /// what makes that distinction hold rather than drift.
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

    /// ★★ **The certified refusal names the cause**, because a refusal an
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
