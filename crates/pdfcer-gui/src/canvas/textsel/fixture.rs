//! # `canvas::textsel::fixture` — the rotated-text page these rules are tested on
//!
//! Test-only. Builds `fixtures/rotated-text.pdf`: one US-Letter page carrying
//! the same sentence set five times, at 0°, 90°, 180°, 270° and 30°.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textsel/fixture.md`.

#![cfg(test)]
// The INNER attribute, beside the `#[cfg(test)] pub mod fixture;` that
// declares this file, for the reason `tests.rs` gives at the same line: the
// string gates recognise it as "nothing here reaches the shipped binary", and
// the twelve string literals below are PDF SYNTAX — `/Type /Catalog`, `xref`,
// `%%EOF` — which are as far from operator-facing copy as a literal gets.

use std::path::PathBuf;

/// The strings, their text matrices, and what each is for.
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

    /// **Rewrite `fixtures/rotated-text.pdf`.**
    #[test]
    #[ignore = "writes into fixtures/; run deliberately"]
    fn regenerate_the_rotated_text_fixture() {
        let at = path();
        std::fs::write(&at, bytes()).expect("the fixtures directory is writable");
        eprintln!("wrote {}", at.display());
    }

    /// The committed fixture is what the generator above produces.
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
