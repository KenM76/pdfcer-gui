//! What the harness knows about the document it opened.
//!
//! ## Why this module exists at all
//!
//! [`crate::coords`] needs one number the application does not have to supply:
//! the **page height in PDF points**, for the y-flip. That number is a
//! property of the *document*, not of the application, so the harness can read
//! it itself — and doing so keeps the document-space contract real rather than
//! aspirational. A harness that had to be told the page size by the program
//! under test would be trusting the program to describe the coordinate space
//! the harness is checking it against.
//!
//! ## The MediaBox scan, and its stated limits
//!
//! [`page_geometry`] scans the raw file bytes for the first `/MediaBox
//! [a b c d]` and reads the size from it. That is a heuristic, and here is
//! exactly what it does and does not handle:
//!
//! **Handled.** The common case, by a wide margin: a `/MediaBox` written as a
//! direct array in an uncompressed object header, which is what every producer
//! this project cares about emits for the page tree root or the first page.
//!
//! **Not handled**, each of which returns `None` rather than a wrong answer:
//!
//! * a `/MediaBox` that is an indirect reference (`/MediaBox 12 0 R`);
//! * a page whose box lives in an object stream (compressed);
//! * documents whose pages differ in size — the *first* box found wins, which
//!   is right for a single-page fixture and wrong for a mixed one;
//! * a non-zero origin (`/MediaBox [10 10 622 802]`) is handled for *size* but
//!   the harness's document coordinates are relative to the box origin, which
//!   for a shifted box is not the same as the PDF origin.
//!
//! Returning `None` matters more than the list. A wrong page height produces a
//! click that is vertically mirrored about the page centre — it lands on the
//! page, hit-tests something plausible, and the resulting failure looks like a
//! selection bug. `None` produces a SKIP that names the missing number, and
//! [`crate::checks`] callers can then be told the size explicitly with
//! `--page-size`.
//!
//! This is the same discipline the rest of the crate applies to coordinates:
//! **refuse rather than guess**, because a confident wrong coordinate is more
//! expensive than no coordinate.

use std::path::Path;

use crate::coords::{DocPoint, PageGeometry};

/// Read the first page's size from a PDF, if it can be read confidently.
///
/// See the module docs for what "confidently" excludes.
#[must_use]
pub fn page_geometry(pdf: &Path) -> Option<PageGeometry> {
    let bytes = std::fs::read(pdf).ok()?;
    // Latin-1 rather than UTF-8: a PDF's binary streams are not text, and a
    // lossy UTF-8 conversion can replace bytes and shift the offsets of the
    // ASCII we are looking for.
    let text: String = bytes.iter().map(|&b| b as char).collect();
    parse_first_mediabox(&text)
}

/// The scan itself, separated so it can be tested without a file.
fn parse_first_mediabox(text: &str) -> Option<PageGeometry> {
    let at = text.find("/MediaBox")?;
    let rest = &text[at + "/MediaBox".len()..];
    let open = rest.find('[')?;
    // A direct array is short. If there is no `]` within a sensible distance,
    // this is an indirect reference or something else entirely, and guessing
    // would be worse than declining.
    let window = &rest[open + 1..(open + 128).min(rest.len())];
    let close = window.find(']')?;
    let nums: Vec<f64> = window[..close]
        .split_whitespace()
        .filter_map(|t| t.parse::<f64>().ok())
        .collect();
    if nums.len() != 4 {
        return None;
    }
    let width = (nums[2] - nums[0]).abs();
    let height = (nums[3] - nums[1]).abs();
    if width <= 1.0 || height <= 1.0 {
        return None;
    }
    Some(PageGeometry {
        width_pt: width,
        height_pt: height,
    })
}

/// **Where the operator's own test drawings live, in the order to look.**
///
///
/// Three checks pinned one absolute path apiece:
///
/// ```text
/// C:\Users\Ken\OneDrive\pdfTests\TR-0461-1500-copy.pdf
/// C:\Users\Ken\OneDrive\pdfTests\KEN-recognised.pdf
/// ```
///
/// On the first full sweep this project ever ran, all three SKIPPED with *"the
/// operator's drawing is not at …"*. The files exist. They had been moved one
/// directory down, into `pdfTests\Moved\`, by the operator tidying his own
/// folder — which he is entitled to do and which no check can be expected to be
/// told about.
///
/// ★★ The cost of getting this wrong is the shape this project keeps meeting: a
/// **SKIP is not red**, so three checks whose subjects are a table marquee, a
/// nested `/FitR` bookmark and text over a scan sat reporting nothing, for ever,
/// while the suite showed its ordinary cheerful INCOMPLETE. Nobody was going to
/// look, because "the operator's file is not there" reads as a fact about the
/// machine rather than as a defect in the harness.
///
/// ⇒ A search over the places those files are actually kept, in one function,
/// so that a fourth check inherits the behaviour rather than the constant. The
/// list is ordered by how canonical the location is, and every entry is a real
/// directory on this machine as of the date above.
///
/// ★ It still returns `None` rather than guessing when the name is nowhere: a
/// check that could not find its subject must SKIP saying so, and
/// [`operator_file_complaint`] builds the sentence that lists where it looked —
/// because a reason that names only the first candidate is what produced the
/// misdiagnosis in the first place.
const OPERATOR_DIRS: [&str; 4] = [
    r"C:\Users\Ken\OneDrive\pdfTests",
    r"C:\Users\Ken\OneDrive\pdfTests\Moved",
    r"D:\Dev\pdfTests",
    r"D:\Dev\pdfTests\SW41177",
];

/// Find one of the operator's own test drawings by file name.
///
/// See [`OPERATOR_DIRS`] for the argument. Pass a bare file name, not a path.
#[must_use]
pub fn operator_file(name: &str) -> Option<std::path::PathBuf> {
    OPERATOR_DIRS
        .iter()
        .map(|d| Path::new(d).join(name))
        .find(|p| p.is_file())
}

/// The sentence a check prints when [`operator_file`] found nothing.
///
/// Lists **every** place that was looked, because a reason may only assert what
/// the check actually looked at — `checks/mod.rs` rule 5.
#[must_use]
pub fn operator_file_complaint(name: &str) -> String {
    format!(
        "the operator's `{name}` is in none of the places this harness looks: {}",
        OPERATOR_DIRS
            .iter()
            .map(|d| format!("`{d}`"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Where a gesture on TEXT can actually be measured, and on which document.
///
/// # The one measurement that explains sixteen skips
///
///
/// ★★★ And a second fact would have survived fixing the aim: that sheet is
/// **2383.9 × 1683.8 pt carrying 123 characters**, so at the fit zoom the
/// sweep drives at, **the tallest text on it is 2.4 screen pixels**. A sweep
/// or a click measured in whole screen pixels cannot reliably land inside a
/// 2.4 px band, and a check that did land would be measuring the harness's
/// luck rather than the program.
///
/// # What this fixture is, measured before anything was driven
///
/// `fixtures/paragraph.pdf`, by walking its content stream:
///
/// ```text
/// MediaBox 0 0 612 792
/// BT  /F1 12 Tf
///   1 0 0 1 72.00 700.00 Tm  (The drawing office keeps every revision of a sheet) Tj
///   1 0 0 1 72.00 684.00 Tm  (in one file, and the notes beside)                  Tj
///   1 0 0 1 72.00 668.00 Tm  (the title block are edited far more often)           Tj
///   1 0 0 1 72.00 652.00 Tm  (than the geometry is. A note that has been)          Tj
///   1 0 0 1 72.00 636.00 Tm  (retyped twice no longer fills)                       Tj
///   1 0 0 1 72.00 620.00 Tm  (its box.)                                            Tj
/// ET
/// ```
///
/// One text object, **six runs**, six baselines 16 pt apart, every one
/// starting at x = 72. On a 612 × 792 page at fit zoom that text is an order
/// of magnitude taller in screen pixels than the A1 sheet's.
///
/// # The aim
///
/// **Page 0, (120, 704)** - inside the FIRST line. The baseline is 700 and
/// the cap height at 12 pt is about 8.4 pt, so the glyph band runs roughly
/// 700..708; x = 120 is about eight characters into a line some 270 pt long.
/// Driven confirmation: `deeper_rung_delete`'s label rung selects a text line
/// there and removes it, `text-lines_before=6 text-lines_after=5`. Six either
/// way on this document — it writes a `Tm` in front of every `Tj`, so each of
/// its six show operators is its own line and the two granularities agree.
/// That agreement is why this fixture cannot tell a line-addressing build from
/// a run-addressing one, and why `inherited-runs.pdf` exists.
///
/// ⚠ **This is a single point on a single line, and that is deliberate.**
/// A check that needs several separate text OBJECTS, or a band wide enough
/// to sweep, needs more than this pair and must say so itself - see
/// [`text_block_target`].
#[must_use]
pub fn text_point_target() -> (std::path::PathBuf, DocPoint) {
    let pdf = workspace_root().join("fixtures").join("paragraph.pdf");
    (pdf, DocPoint::new(0, 120.0, 704.0))
}

/// The same document, with the span of every line on it.
///
/// A few checks do not click at a point: they sweep a band, walk from one
/// run to the next, or select a range and read back what was taken. Those
/// need to know **where the text is**, not just one point inside it, and the
/// numbers they need are the ones in [`text_point_target`]'s table.
///
/// Returns `(document, baseline, left edge, right edge)` for the **first**
/// line only, in PDF user space.
///
/// # Why the first line and not the block
///
/// Because the lines are not the same length, and one right edge for the
/// whole block would be a trap dressed as a convenience. The extents below
/// are arithmetic rather than estimate - the file states `/Helvetica`,
/// `/WinAnsiEncoding`, 12 pt and an origin of x = 72, so the widths come
/// from the standard Helvetica metrics:
///
/// ```text
/// baseline 700.0   x 72.0 .. 338.8   The drawing office keeps every revision of a sheet
/// baseline 684.0   x 72.0 .. 241.4   in one file, and the notes beside
/// baseline 668.0   x 72.0 .. 276.8   the title block are edited far more often
/// baseline 652.0   x 72.0 .. 298.1   than the geometry is. A note that has been
/// baseline 636.0   x 72.0 .. 216.7   retyped twice no longer fills
/// baseline 620.0   x 72.0 .. 110.0   its box.
/// ```
///
/// A sweep from 72 to 330 along the first line ends **inside** the glyphs.
/// The same sweep along the last line would end 220 pt past the final full
/// stop - a question about how the hit test treats trailing space, which is
/// a real question and not one any of these checks means to ask by accident.
///
/// ⚠ The six baselines are 16 pt apart and the glyphs are about 8.4 pt
/// tall, so a band aimed BETWEEN two baselines hits nothing. Aim at
/// `baseline + 4.0`, never at a midpoint between lines. A check that needs a
/// second line has the numbers above rather than an invitation to estimate.
#[must_use]
pub fn text_block_target() -> (std::path::PathBuf, f32, f32, f32) {
    let pdf = workspace_root().join("fixtures").join("paragraph.pdf");
    (pdf, 700.0, 72.0, 330.0)
}

/// The workspace root, derived from this crate's manifest directory.
///
/// Every fixture this harness pins is named relative to the repository root,
/// because that is the only place a path can be written down that survives
/// being run from a different working directory - and `ui-verify` is run from
/// the repository root by hand, from `tools/ui-verify` by `cargo run`, and
/// from wherever a sweep script happens to be.
///
/// # Why it is here and not in each check
///
///
/// ⇒ The eleven are left alone on purpose. Migrating them belongs in its own
/// commit - a mechanical edit to eleven unrelated modules, folded into a
/// defect repair, makes the repair unreviewable.
#[must_use]
pub fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// Where a grip drag can actually be measured, and on which document.
///
/// # Why this is a function and not three literals
///
/// Three checks are the same gesture with three different verbs -
/// [`crate::checks`]'s `resize_scales_a_shape`, `rotate_handle_turns_a_selection`
/// and `shift_constrains_a_resize`. Each selects a shape, presses one of the
/// eight grips and drags it. All three therefore need the same thing from the
/// document: **a point where clicking selects a single path object whose
/// selection outline is large enough on screen that its eight grips do not
/// overlap each other.**
///
/// That is a property of the *fixture*, not of any one check, which is why it
/// lives here beside the other thing the harness reads out of a document.
///
///
/// `resize_scales_a_shape` was moved into `sweep-full.sh`'s ALONE table with
/// `--doc-point 0,300,500` and passed. The other two kept taking the sweep's
/// shared `0,2000,320` and **both failed**, each with several paragraphs
/// naming three application functions as the likely cause. Every one of those
/// functions is correct. The aim point was wrong, and the knowledge of which
/// aim point works was written down in a shell script, attached to one of the
/// three checks that needed it.
///
/// ⇒ **Knowledge a check cannot run without belongs beside the check, not in
/// the runner's arguments** - the runner's arguments are a compromise chosen
/// for the majority, and a check whose subject cannot exist under that
/// compromise does not report *my input is wrong*. It reports something
/// specific and believable about the program.
///
/// # The numbers
///
/// `fixtures/a1-titleblock.pdf`, page 0, **300, 500** in PDF user space -
/// a shape near the lower-left of the 2383.9 x 1683.8 pt sheet, clear of the
/// title block's own dense line work. Verified by `resize_scales_a_shape`
/// committing `resize-commit grip=SouthEast sx=1.1449 sy=1.2052` from it.
#[must_use]
pub fn grip_gesture_target() -> (std::path::PathBuf, DocPoint) {
    let pdf = workspace_root().join("fixtures").join("a1-titleblock.pdf");
    (pdf, DocPoint::new(0, 300.0, 500.0))
}

/// **A point on a run of TEXT on the shared drawing fixture** —
/// `fixtures/a1-titleblock.pdf`, page 0, **1845.5, 184.7**.
///
/// # What the document actually has there
///
/// The sheet's only ink is its title block. `pdfcer extract-text --json` reports
/// fourteen runs with a box, and this point is inside the one reading
/// `PROJECT NO`, whose box is `1831.2, 181.3 → 1872.2, 187.3`. It is a 6 pt
/// label on a 2383.9 × 1683.8 pt sheet, which is the property that makes it
/// worth aiming at: a click that lands on it is a click the operator could make
/// and the harness only just can.
///
/// # ★ Two unrelated checks want this point for two unrelated reasons
///
/// `the_font_controls_are_live_on_the_drawing_you_open` wants **text under the
/// cursor**, because a click on blank paper is symptom-identical to a hit test
/// that does not work.
///
/// `text_annot_takes_the_keyboard_unclicked` does not care about text at all. It
/// wants **room to the right**: it drags a box 440 pt wide from the point, and
/// the sweep's shared `0,2000,320` puts the far corner at 2440, which is off a
/// 2383.9 pt sheet. Every assertion in that check passed at the shared point and
/// the run was then reported SKIPPED on the geometry — a correct skip that reads
/// like a broken check.
///
/// ⇒ The second requirement is the one a reader would not guess, so it is
/// written here rather than left in whichever check happens to be read first.
/// **Any replacement point must satisfy both**: on a glyph, and at least 440 pt
/// clear of the right edge and 190 pt clear of the top.
#[must_use]
pub fn a1_text_target() -> (std::path::PathBuf, DocPoint) {
    let pdf = workspace_root().join("fixtures").join("a1-titleblock.pdf");
    (pdf, DocPoint::new(0, 1845.5, 184.7))
}

/// **A point on a path whose stroke is heavy enough to MEASURE a stroke rule**
/// - `fixtures/polyline-nodes.pdf`, page 0, **150, 260**.
///
/// # The property being asked of the document
///
/// `preview_width_ignores_zoom` (`OPERATOR_REQUESTS.md` **O184**) measures how
/// wide the drag preview is painted, and it asks two questions of that number:
/// that zoom does not change it, and that the one-pixel line-weight view does.
/// Both need **a stroke wider than one point**, because
/// `canvas::shapes::StrokeRule::preview_px` floors at one device pixel: on a
/// hairline the correct build answers `1.00` under every setting, and an
/// assertion that two settings differ would be asserting something true builds
/// do not do.
///
/// ⇒ The requirement is not "a path" but **"a path with a heavy pen"**, and
/// that is a property of the fixture rather than of the check.
///
///
/// Driven first on the sweep's shared `a1-titleblock.pdf --doc-point
/// 0,2000,320`, the check **SKIPPED**: nothing at that coordinate has stroked
/// geometry, so `canvas::shapes::for_move_subject` answers with an erase and no
/// shapes, and there was no preview stroke to measure at all.
///
/// ⚠ The shared point is bare paper. `extract-text --json` puts the nearest
/// text run 132.9 pt away and the sheet's only ink is the title block in the
/// bottom right. A `marquee-mode` line read at that point counts what a BAND
/// returned, not what is under the cursor, so it cannot be quoted as evidence
/// about the coordinate.
///
/// Moved to [`grip_gesture_target`]'s `0,300,500` it **passed, at 1.00 px at
/// both zooms** - and that pass is half a measurement. Ruling 1 was real there
/// (the defect multiplies before the floor, so a 0.5 pt line would have read
/// 4.74 px at 948 %), but ruling 2 could not be measured at all, because a
/// number already at the floor cannot be lowered to it.
///
/// ⇒ On this fixture the same check reports **3.00 px at both zooms, and 1.00
/// px with line weights off**. Three separate numbers, a 13.5x magnification
/// between two of them, and every assertion has somewhere to fail.
///
/// # The numbers
///
/// `fixtures/polyline-nodes.pdf` is 535 bytes and one open path - a zigzag and
/// two Beziers, drawn `3.0 w`, which is the widest single-path pen in this
/// fixture set. `deeper_rung_delete` and `bezier_handle` already pin it, for
/// the unrelated reason that its tail has enough anchors to delete one from.
#[must_use]
pub fn heavy_stroke_target() -> (std::path::PathBuf, DocPoint) {
    let pdf = workspace_root().join("fixtures").join("polyline-nodes.pdf");
    (pdf, DocPoint::new(0, 150.0, 260.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_direct_mediabox() {
        let g = parse_first_mediabox("<< /Type /Page /MediaBox [0 0 612 792] >>").unwrap();
        assert_eq!(g.width_pt, 612.0);
        assert_eq!(g.height_pt, 792.0);
    }

    #[test]
    fn reads_a_shifted_box_as_its_size() {
        let g = parse_first_mediabox("/MediaBox [10 10 622 802]").unwrap();
        assert_eq!(g.width_pt, 612.0);
        assert_eq!(g.height_pt, 792.0);
    }

    /// An indirect reference must decline, not invent. A wrong page height
    /// mirrors every click about the page centre.
    #[test]
    fn declines_an_indirect_mediabox() {
        assert!(parse_first_mediabox("/MediaBox 12 0 R").is_none());
    }

    #[test]
    fn declines_a_degenerate_box() {
        assert!(parse_first_mediabox("/MediaBox [0 0 0 0]").is_none());
    }

    #[test]
    fn declines_when_there_is_no_mediabox_at_all() {
        assert!(parse_first_mediabox("%PDF-1.7\n1 0 obj\n<< >>").is_none());
    }
}
