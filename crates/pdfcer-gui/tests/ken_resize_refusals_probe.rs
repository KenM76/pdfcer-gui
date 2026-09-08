//! **A probe, not a test** — which markup kinds does `resize_annotation`
//! actually refuse, and why?
//!
//! The operator, 2026-09-08:
//!
//! > *"Also when will being able to drag on the canvas be able to resize the
//! > Text Box and Stamp."*
//!
//! # Why this is measured rather than read
//!
//! `canvas::pressing` offers **all nine handles** on every unlocked markup, so
//! the shell believes every kind is resizable. `app::actions::annots::resize`
//! catches one refusal and words it, and its comment names the case:
//! *"`resize_annotation` has to refuse artwork pdfcer did not draw"*.
//!
//! ⇒ So the shell's own account is *"they all resize, except sometimes"*, and
//! *which* sometimes is a property of the engine and of the annotation, not
//! something this repository states anywhere. He asked which; nobody knows.
//!
//! ⚠ **And "read the engine and reason" is exactly what produced four wrong
//! claims on 2026-09-08 alone** — two wrong diagnoses of his BOM report, a font
//! coverage number sampled from one cell, and a request that asserted a code
//! branch was fine when it had never been exercised. This authors each kind and
//! asks.
//!
//! Run with:
//!
//! ```text
//! cargo test --test ken_resize_refusals_probe -- --ignored --nocapture
//! ```

use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;

const FIXTURE: &str = "fixtures/a1-titleblock.pdf";

fn session() -> EditSession {
    let path = format!("{}/../../{FIXTURE}", env!("CARGO_MANIFEST_DIR"));
    EditSession::new(Document::load(std::path::Path::new(&path)).expect("the fixture loads"))
}

fn rect() -> pdfcer_core::page_tree::Rect {
    pdfcer_core::page_tree::Rect {
        llx: 100.0,
        lly: 600.0,
        urx: 400.0,
        ury: 700.0,
    }
}

/// ★★★ **Author each kind pdfcer can draw, then try to resize it, and print
/// the verdict.**
///
/// Two resizes per kind, because they are refused by different rules:
///
/// * **uniform** (1.5, 1.5) — the ordinary corner drag with Shift, and the one
///   with the best chance of being accepted on foreign artwork;
/// * **non-uniform** (1.5, 0.8) — the ordinary corner drag *without* Shift,
///   which on an appearance pdfcer did not author produces an anisotropic
///   border by arithmetic, since neither PDF nor SVG has a per-axis stroke
///   width.
///
/// ⚠ A fresh session per attempt. A shared one measures the Nth resize on an
/// annotation already resized N−1 times, and `PageEditedThisSession` is a real
/// refusal this probe would then attribute to the kind.
#[test]
#[ignore = "a measurement, not an assertion — run it and read the output"]
fn which_markup_kinds_does_a_corner_drag_actually_resize() {
    use pdfcer_core::annot_author::TextAnnotSpec;
    use pdfcer_core::edit::ResizeOptions;

    // The three kinds his question names, plus a shape as the control: a
    // `/Square` is pdfcer's own artwork end to end and must resize, so a
    // refusal there would say the probe is wrong rather than the kind.
    /// One row of the table: a label and the verb that authors that kind.
    ///
    /// ★ Named rather than written inline because the inline form is a type
    /// clippy calls "very complex", and it is right — a reader meeting
    /// `Vec<(&str, Box<dyn Fn(&mut EditSession) -> Option<ObjId>>)>` has to
    /// decode it before learning that it means "a name and a way to make one".
    type Author = Box<dyn Fn(&mut EditSession) -> Option<pdfcer_core::object::ObjId>>;

    let kinds: Vec<(&str, Author)> = vec![
        (
            "FreeText (Markup > Text box)",
            Box::new(|s: &mut EditSession| {
                let spec = TextAnnotSpec::FreeText {
                    rect: rect(),
                    text: "TWO WORDS".to_owned(),
                    font: pdfcer_core::fontdata::Std14::Helvetica,
                    font_size: 12.0,
                    color: pdfcer_core::vartext::TextColor::Gray(0.0),
                    quadding: pdfcer_core::vartext::Quadding::Left,
                    multiline: true,
                    border: None,
                    border_width: 0.0,
                };
                s.add_text_annotation(0, &spec).ok()
            }),
        ),
        (
            "Sticky note",
            Box::new(|s: &mut EditSession| {
                let spec = TextAnnotSpec::Sticky {
                    rect: rect(),
                    contents: "a note".to_owned(),
                    icon: pdfcer_core::annot_author::StickyIcon::Note,
                    color: pdfcer_core::annot_author::Color::Rgb(1.0, 0.9, 0.2),
                    open: false,
                };
                s.add_text_annotation(0, &spec).ok()
            }),
        ),
        // ★★★ **THE CONTROL, and the doc comment above promised it while the
        // first draft of this list did not contain it.**
        //
        // A `/Square` is pdfcer's own artwork end to end, authored through
        // `add_markup` seconds earlier in the same session. If it resizes and
        // the two above do not, the difference is real and is about those
        // kinds. If **nothing** resizes, the refusal is not about the kind at
        // all — it is the engine failing to recognise its own work — and a
        // report naming the Text box would be naming the wrong subject.
        //
        // ⚠ Without this row the first run's output reads as *"text boxes and
        // stickies cannot be resized"*, which is a confident, precise and
        // possibly wrong sentence. That is the shape of every wrong claim made
        // on 2026-09-08.
        (
            "Square (the CONTROL)",
            Box::new(|s: &mut EditSession| {
                let spec = pdfcer_core::annot_author::MarkupSpec::Square {
                    rect: rect(),
                    border: Some(pdfcer_core::annot_author::Color::Rgb(1.0, 0.0, 0.0)),
                    interior: None,
                    border_width: 2.0,
                    border_effect: None,
                };
                s.add_markup(0, &spec).ok()
            }),
        ),
    ];

    for (label, author) in kinds {
        for (what, sx, sy) in [("uniform  ", 1.5, 1.5), ("non-unif ", 1.5, 0.8)] {
            let mut session = session();
            let Some(id) = author(&mut session) else {
                println!("{label:<30} {what} AUTHORING FAILED — nothing to resize");
                continue;
            };
            let anchor = (100.0, 600.0);
            match session.resize_annotation(id, anchor, sx, sy, &ResizeOptions::default()) {
                Ok(report) => println!("{label:<30} {what} ✅ ACCEPTED  {report:?}"),
                Err(e) => println!("{label:<30} {what} ❌ REFUSED — {e}"),
            }
        }
    }
}
