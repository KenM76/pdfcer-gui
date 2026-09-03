//! # `canvas::textsel::fixture` — the rotated-text page these rules are tested on
//!
//! Test-only. Builds `fixtures/rotated-text.pdf`: one US-Letter page carrying
//! the same sentence set five times, at 0°, 90°, 180°, 270° and 30°.
//!
//! ## ★★ Why a synthetic page and not the operator's own drawing
//!
//! The report that started this work names a real file — `SW41177.pdf`, whose
//! title block carries a vertical SolidWorks path stamp — and that file is what
//! every claim here was *measured* against. It is deliberately **not committed**
//! as a fixture: it is a customer drawing, and the standing rule about
//! SolidWorks-derived work product is that it does not enter a repository that
//! could be published. It stays at `D:\Dev\temp\pdfcer\SW41177.pdf` and is used
//! by hand and by `ui-verify --pdf`.
//!
//! So the committed fixture has to stand in for it, and standing in means
//! reproducing the *mechanism* rather than the appearance:
//!
//! * **Every rotated string is set in wide capitals.** That is not cosmetic. At
//!   12 pt Helvetica every capital's advance exceeds `line_gap_ratio × size` =
//!   3.6 pt, so the extraction splits the string at **every letter** and no run
//!   holds two glyphs. This is precisely the case the first draft of
//!   [`super::writing`] could not see — see its §2.1 — and a fixture full of
//!   narrow letters would have passed that draft and proved nothing.
//! * **Four quadrant rotations, not one.** 90° and 270° break on the baseline
//!   clause; 180° breaks on the *backward-jump* clause instead, which is a
//!   different line of `classify` reached for a different reason and would not
//!   have been exercised by the vertical case alone.
//! * **One 30° string**, because the band this produces is a genuine
//!   parallelogram rather than an axis-aligned rectangle, which is the one place
//!   the canvas wash is documented to over-cover.
//! * **One horizontal string**, which is the regression guard: whatever else
//!   changes, selecting it must produce exactly what it produced before any of
//!   this existed.
//!
//! ## The generator is a test, and the fixture is committed
//!
//! [`regenerate`] is `#[ignore]`d and writes the file; the fixture is checked
//! in beside it. That is the convention `crate::ocr::fixture` established and
//! the reasons are its reasons: a fixture generated at test time is a fixture
//! whose bytes no one has ever looked at, and `ui-verify` needs a path on disk
//! that exists before `cargo test` has run.
//!
//! ## Why the bytes are written by hand
//!
//! `pdfcer-core` can author text, and using it here would make the fixture a
//! product of the same engine whose extraction is under test — so a change that
//! moved both would still pass. Hand-written content streams state the text
//! matrices literally, which is the one thing this fixture is *for*: `0 1 -1 0
//! 100 300 Tm` is the input, in the file, readable, and not the output of
//! anything.

#![cfg(test)]
// ★ The INNER attribute, beside the `#[cfg(test)] pub mod fixture;` that
// declares this file, for the reason `tests.rs` gives at the same line: the
// string gates recognise it as "nothing here reaches the shipped binary", and
// the twelve string literals below are PDF SYNTAX — `/Type /Catalog`, `xref`,
// `%%EOF` — which are as far from operator-facing copy as a literal gets.

use std::path::PathBuf;

/// The strings, their text matrices, and what each is for.
///
/// `[a b c d e f]` is §9.4.2's `Tm`. The rotation lives in `a b c d`; `e f` is
/// where the string starts. Every rotated entry is **capitals**, for the reason
/// in the module header.
const LINES: &[(&str, [f32; 6])] = &[
    // 0° — the regression guard. Nothing about it may change.
    ("HORIZONTAL", [1.0, 0.0, 0.0, 1.0, 72.0, 700.0]),
    // 90°, advancing up the page: the operator's case.
    ("UPWARD", [0.0, 1.0, -1.0, 0.0, 100.0, 300.0]),
    // 180°, advancing left: breaks on the backward-jump clause, not the
    // baseline clause.
    ("INVERTED", [-1.0, 0.0, 0.0, -1.0, 520.0, 200.0]),
    // 270°, advancing down the page.
    ("DOWNWARD", [0.0, -1.0, 1.0, 0.0, 300.0, 700.0]),
    // 30°, where the band is a parallelogram and the wash is its bounds.
    ("SKEWED", [0.866_025, 0.5, -0.5, 0.866_025, 200.0, 450.0]),
];

/// Where the fixture lives, relative to this crate.
///
/// `../../fixtures/` — **this** repository's, not the engine's. The engine's
/// tree is read-only for this project, and a fixture written into it would be
/// the one kind of write the governing rule forbids outright.
#[must_use]
pub fn path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/rotated-text.pdf")
}

/// The whole file, as bytes.
///
/// Split out from [`regenerate`] so the assertion that the fixture on disk
/// still matches the generator is a byte comparison rather than a rerun — see
/// [`tests::the_committed_fixture_matches_its_generator`].
#[must_use]
pub fn bytes() -> Vec<u8> {
    let mut content = String::new();
    for (text, tm) in LINES {
        // `Tf` inside each `BT`/`ET` rather than once outside: §9.3.1 makes the
        // text state part of the graphics state, so hoisting it would work and
        // would also make each block depend on the one before it, which is
        // exactly the coupling a fixture should not have.
        content.push_str("BT /F1 12 Tf ");
        for n in tm {
            content.push_str(&format!("{n} "));
        }
        content.push_str(&format!("Tm ({text}) Tj ET\n"));
    }

    let objects: Vec<String> = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
         /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>"
            .to_owned(),
        format!(
            "<< /Length {} >>\nstream\n{content}endstream",
            content.len()
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
    ];

    let mut out: Vec<u8> = b"%PDF-1.4\n".to_vec();
    // A binary comment line, per §7.5.2's recommendation, so a transfer that
    // guesses at the file's type guesses binary.
    out.extend_from_slice(b"%\xE2\xE3\xCF\xD3\n");
    let mut offsets = Vec::with_capacity(objects.len());
    for (index, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", index + 1).as_bytes());
    }

    let xref_at = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for offset in &offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_at}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **Rewrite `fixtures/rotated-text.pdf`.**
    ///
    /// `#[ignore]`d: it writes into the repository, and a test that edits the
    /// tree it is run from should be an act rather than a side effect. Run it
    /// deliberately —
    /// `cargo test -p pdfcer-gui regenerate_the_rotated_text_fixture -- --ignored`
    /// — and commit what it produces.
    #[test]
    #[ignore = "writes into fixtures/; run deliberately"]
    fn regenerate_the_rotated_text_fixture() {
        let at = path();
        std::fs::write(&at, bytes()).expect("the fixtures directory is writable");
        eprintln!("wrote {}", at.display());
    }

    /// The committed fixture is what the generator above produces.
    ///
    /// Without this, the generator and the file could drift and every test that
    /// reads the fixture would still pass — while the header above, which
    /// explains the fixture in terms of the generator's `LINES` table, would
    /// have quietly become fiction.
    #[test]
    fn the_committed_fixture_matches_its_generator() {
        let on_disk = std::fs::read(path()).expect(
            "fixtures/rotated-text.pdf is committed; run \
             `regenerate_the_rotated_text_fixture -- --ignored` if it is missing",
        );
        assert_eq!(
            on_disk,
            bytes(),
            "the committed fixture has drifted from its generator"
        );
    }
}
