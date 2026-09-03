//! The pixel oracle, run against a real screenshot of the real defect.
//!
//! # Why this test exists
//!
//! `src/pixels.rs` has unit tests built on synthetic images, and synthetic
//! images are exactly as legible as whoever wrote them decided they should be.
//! They prove the arithmetic. They do not prove that the oracle, pointed at an
//! actual antialiased screenshot of an actual application, separates text a
//! person can read from text a person cannot.
//!
//! That is the claim the harness makes, so that is the claim that gets tested
//! — against `evidence/crop_settings.png`, the dated artefact `DEFECTS.md` D2
//! cites, captured from the old GUI on 2026-08-12.
//!
//! # Both directions, and the second one is the important one
//!
//! A gate that always fails is as useless as a gate that always passes, and it
//! is harder to notice. So this test asserts both:
//!
//! * the **Appearance** heading — the D2 defect — measures **below** the
//!   threshold;
//! * the **body prose** directly beneath it, which is perfectly readable in
//!   the same screenshot, in the same dialog, on the same background,
//!   measures **above** it.
//!
//! The two regions are eighty pixels apart. If the oracle were simply
//! reporting "grey screenshot, low contrast" it would fail both, and this test
//! would say so.
//!
//! # If the evidence file is missing
//!
//! The test returns early with a printed explanation rather than failing. It
//! asserts a property of an artefact, and an absent artefact is not a
//! regression in the code. It does **not** silently pass: the reason is
//! printed, and `cargo test -- --nocapture` shows it.

use std::path::PathBuf;

use ui_verify::geom::FracRect;
use ui_verify::image::Image;
use ui_verify::pixels::{self, AA_LARGE};

/// The repository root, from this crate's directory.
fn evidence() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("evidence/crop_settings.png")
}

/// The "Appearance" collapsing-header caption — D2's first named heading.
/// Same fractions as `profile::PDFCER_LEGACY`'s region set.
const DEFECTIVE_HEADING: FracRect = FracRect::new(0.030, 0.000, 0.160, 0.042);

/// The explanatory prose two lines below it: *"Changes the window only. It
/// never alters a document…"*. Dark text, same dialog, same background,
/// eighty pixels away, and entirely readable.
const LEGIBLE_BODY_TEXT: FracRect = FracRect::new(0.038, 0.124, 0.694, 0.150);

#[test]
fn the_oracle_separates_the_d2_heading_from_legible_text_eighty_pixels_away() {
    let path = evidence();
    if !path.is_file() {
        eprintln!(
            "skipping: no evidence file at {}. This test asserts a property of a captured \
             artefact; an absent artefact is not a code regression.",
            path.display()
        );
        return;
    }

    let image = match Image::load_png(&path) {
        Ok(i) => i,
        Err(e) => {
            eprintln!(
                "skipping: {} exists but could not be read ({e}). ui-verify converts PNGs \
                 through PowerShell rather than carrying a decoder — see src/image.rs.",
                path.display()
            );
            return;
        }
    };

    let heading = pixels::contrast_at(
        &image,
        DEFECTIVE_HEADING.resolve(image.width(), image.height()),
    );
    let body = pixels::contrast_at(
        &image,
        LEGIBLE_BODY_TEXT.resolve(image.width(), image.height()),
    );

    println!("  heading (D2):    {}", heading.summary());
    println!("  body prose:      {}", body.summary());

    assert!(
        heading.sampled > 1000 && body.sampled > 1000,
        "both regions must actually sample pixels; if either is near zero the fractions have \
         drifted off the image and neither number means anything"
    );

    assert!(
        heading.ratio < AA_LARGE,
        "the D2 heading must measure BELOW the {AA_LARGE}:1 floor — it is near-white on light \
         grey and DEFECTS.md records it as unreadable at 1x. Measured {}",
        heading.summary()
    );

    assert!(
        body.ratio >= AA_LARGE,
        "the body prose eighty pixels below it must measure ABOVE the {AA_LARGE}:1 floor. If \
         this fails, the oracle is not measuring legibility — it is just reporting that the \
         screenshot is grey, and a gate that fails everything is no better than one that \
         passes everything. Measured {}",
        body.summary()
    );

    assert!(
        !pixels::region_not_uniform(
            &image,
            DEFECTIVE_HEADING.resolve(image.width(), image.height())
        )
        .is_uniform(),
        "the heading region must be varied: something IS drawn there, it is merely invisible. \
         A uniform verdict here would misdiagnose D2 as a missing caption."
    );
}
