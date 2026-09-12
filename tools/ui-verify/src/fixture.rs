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
/// # ★★★ Why this is a search and not a constant — 2026-09-05
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

// The test module goes LAST, and that is a lint rather than a taste: clippy
// refuses `items after a test module`, because an item below `mod tests` is
// easy to read as part of the tests and is not. It was moved here on
// 2026-09-05 when `OPERATOR_DIRS` was added above it during the driven
// sweep -- the const is production code and belongs with production code.

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
/// Eleven check modules carry a private copy of exactly this function:
/// `link_follow`, `ocr_progress`, `off_page_census`, `off_page_marquee`,
/// `off_page_press`, `off_page_toggle`, `off_page_visible`, `off_page_zoom`,
/// `page_display_pref`, `quit_unsaved` and `save_as`. None of them is wrong.
/// The problem is the twelfth: pinning a fixture is the standing repair for
/// the checks the 2026-09-12 sweep found aimed at the wrong document, and
/// there are about thirty of those.
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
/// # What it cost to learn - measured 2026-09-12
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
