//! **Why a click on his text selects a path** — the measurement behind
//! `OPERATOR_REQUESTS.md` O198's first sentence, taken headlessly against his
//! own drawing.
//!
//! # The report, and the thing that was measured before this file existed
//!
//! > *"I really need you to focus on finding ways to make all text editable on
//! > the sw drawing that is in pdftests folder. Find a way to make it happen."*
//!
//!
//! ```text
//! pdfcer-diag properties-panel object=5899 kind=Path notes=0
//! pdfcer-diag canvas-selection via=click mod=false sel=1 level=Object first=object:5899
//! pdfcer-diag canvas-pick-depth depth=0 of=2 alt=false
//! ```
//!
//! Object **5899** — the same object at every one of the eight points, and the
//! sheet has 5,903 page objects, so it is very nearly the last thing painted.
//! `of=2` says the text WAS a candidate. It was second.
//!
//! ★★★ That is the whole of his first sentence, and it is not about text
//! editing at all. Every font control, every Properties field and every restyle
//! verb in this program is reached through a text selection, so a hit test that
//! cannot produce one makes all of them unreachable at once — which is exactly
//! how a capability that is present, tested and green on a fixture presents to
//! an operator as *"that entire area is always greyed out"*.
//!
//! # What this file asks, and why it asks the ENGINE rather than the shell
//!
//! Three questions, in order, because each one only makes sense if the one
//! before it answered the way it did:
//!
//! 1. **Is the path hit real ink, or slack?** `pdfcer_core::vector::hit_test_*`
//!    treats a path as *fill interior under its winding rule, or stroke
//!    proximity within half the scaled line width **plus the tolerance***, and
//!    treats a text object as *its bounding box inflated by the tolerance*. So
//!    a click can hit a path it is merely NEAR and a text run it is genuinely
//!    INSIDE, and the two are indistinguishable in the returned list.
//! 2. **Does the tolerance decide it?** Re-ask at tolerance zero. Anything that
//!    drops out was a near-miss; anything that survives is ink under the
//!    pointer.
//! 3. **Would preferring the exact hit actually reach his text?** If the
//!    zero-tolerance answer at these nine points is the text object, then the
//!    remedy is a re-rank the shell can do with the API it already calls, and
//!    no engine change is needed for it.
//!
//! ★★ Asked of the engine and not of the running shell because the shell's
//! answer is one number — `depth=0 of=2` — and one number cannot distinguish a
//! wrong ORDER from a wrong CANDIDATE SET. This project has filed a wrong
//! engine request from exactly that confusion once already.
//!
//! # ★ A probe, not a test, and `#[ignore]`d for the same reason as its sibling
//!
//! It reads a file that is **not in this repository** — a copy of his drawing
//! under `target/scratch/docs/` — so it cannot run in CI and must never fail a
//! build. `ken_sw41177_probe.rs` is the sibling and carries the text-editing
//! measurements; this one carries the picking measurements, in its own file
//! rather than appended to that one because that file is at 1,349 lines and R2
//! stops at 1,500. A probe that forces a split of the file it lands in is a
//! probe in the wrong file.
//!
//! Run with:
//!
//! ```text
//! cargo test --test ken_sw41177_pick -- --ignored --nocapture
//! ```

use pdfcer_core::document::Document;
use pdfcer_core::page_tree;
use pdfcer_core::vector::{HitTarget, Matrix, PageObjects, Point, decompose_page};

/// His drawing, copied out of `D:\dev\pdfTests\SW41177\` by hand.
///
/// Two names for one file are tolerated here: the sibling probe pins
/// `SW41177-ken.pdf` and the `ui-verify` runs pin `SW41177.pdf`. Both are
/// looked for, in that order, so whichever a session happened to stage is
/// found.
const FIXTURES: [&str; 2] = [
    "target/scratch/docs/SW41177-ken.pdf",
    "target/scratch/docs/SW41177.pdf",
];

/// The nine aims the driven check was pointed at, in PDF user space.
///
/// ★★ One per distinct font size on page 1, and the choice of ONE PER SIZE is
/// the point rather than a convenience. Two points either side of a transition
/// look exactly like no transition, and a series picked by eye is a series
/// picked to agree with whoever picked it. These came out of
/// `pdfcer extract-text --pages 1 --json`: group every glyph run by its size,
/// take the longest run at each size, use its first glyph's origin.
///
/// The `+2, +2` the driven check adds is applied here too, so the two
/// measurements are about the same point. That offset exists because
/// `--doc-point` names a glyph ORIGIN, which is the bottom-left corner of the
/// first character and therefore on the very edge of the ink.
const AIMS: [(&str, f64, f64); 9] = [
    ("5pt INTERPRET THIS DRAWING", 1135.7, 84.6),
    ("6pt UNLESS OTHERWISE SPEC", 1118.3, 96.0),
    ("8pt P.ENG REVIEWED", 786.6, 98.3),
    ("9pt CHANGES TO SW41177-09", 325.6, 44.9),
    ("10pt ITEM", 1082.5, 1181.6),
    ("11.8pt SW41177 ASSEMBLY", 1373.4, 35.5),
    ("12pt Valley East Industrial", 865.3, 84.0),
    ("13.2pt #5 FIT ITEM 12 FRAME", 1047.3, 642.7),
    ("16pt WEIGHT: 683.33LBS", 1249.7, 138.2),
];

/// The offset the driven check applies to `--doc-point`. See [`AIMS`].
const NUDGE: f64 = 2.0;

/// Tolerances asked at each aim, in PDF user space points.
///
/// ★ `0.0` is the control and the other three are the series. The shell derives
/// its tolerance from the zoom — a few screen pixels converted into page units
/// — so on a 1,584 pt sheet fitted into roughly a thousand pixels one pixel is
/// about 1.6 pt and the working tolerance is several points. Walking four rungs
/// says WHERE the answer changes, which is the thing a single sample cannot.
const TOLERANCES: [f64; 4] = [0.0, 1.0, 4.0, 8.0];

/// Load page 1's object model, or print why not and return `None`.
///
/// Returns the model rather than the session because every question below is
/// about geometry, and `decompose_page` is the same call
/// `panels::objects::provider` makes — asking a different decomposition would
/// be measuring a different program.
fn page_model() -> Option<PageObjects> {
    let root = env!("CARGO_MANIFEST_DIR");
    let found = FIXTURES.iter().find_map(|rel| {
        let path = format!("{root}/../../{rel}");
        let p = std::path::PathBuf::from(&path);
        p.exists().then_some(p)
    });
    let Some(path) = found else {
        println!("SKIP: none of {FIXTURES:?} is present - copy his file there first");
        return None;
    };
    println!("fixture: {}", path.display());
    let doc = Document::load(&path).expect("his file loads");
    let view = doc.view();
    let pages = page_tree::pages_in(&view).expect("his file has a page tree");
    let model = decompose_page(&view, pages.first()?, Matrix::IDENTITY).expect("page 1 decomposes");
    println!(
        "page 1: {} objects, {} leaves",
        model.objects.len(),
        model.leaves.len()
    );
    Some(model)
}

/// A one-word class for a hit, matching the vocabulary `canvas::pick` uses.
///
/// Deliberately coarse. The question here is *text or not text*, and a richer
/// classification would invite reading a distinction this probe has not
/// measured.
fn class_of(model: &PageObjects, hit: HitTarget) -> String {
    let object = match hit {
        HitTarget::Object(i) => model.objects.get(i),
        HitTarget::Leaf(i) => model.leaves.get(i).map(|leaf| &leaf.object),
    };
    let name = match object {
        Some(pdfcer_core::vector::VectorObject::Text(_)) => "Text",
        Some(pdfcer_core::vector::VectorObject::Path(_)) => "Path",
        Some(pdfcer_core::vector::VectorObject::Image(_)) => "Image",
        // ★ No catch-all arm, and its absence is deliberate. `VectorObject`
        // has exactly these three variants as of engine 0.53.0, so `Some(_)`
        // here is unreachable and clippy says so. Leaving it out turns the
        // engine growing a fourth variant into a COMPILE ERROR in this file
        // rather than a silent "Other" in a report — which is the outcome
        // worth having, because this probe's whole job is to say what class
        // won a hit test, and a class it cannot name is the finding.
        None => "MISSING",
    };
    match hit {
        HitTarget::Object(i) => format!("{name}#{i}"),
        HitTarget::Leaf(i) => format!("{name}~leaf{i}"),
    }
}

/// Is this hit a text object?
fn is_text(model: &PageObjects, hit: HitTarget) -> bool {
    let object = match hit {
        HitTarget::Object(i) => model.objects.get(i),
        HitTarget::Leaf(i) => model.leaves.get(i).map(|leaf| &leaf.object),
    };
    matches!(object, Some(pdfcer_core::vector::VectorObject::Text(_)))
}

/// **Question 1 and 2 together: what is under each aim, at four tolerances.**
///
/// Prints the full front-to-back candidate list per aim per tolerance, so the
/// reader can see both the ORDER and the SET change as the slack grows. The
/// two are separate failure modes with separate fixes and this project has
/// already confused them once.
#[test]
#[ignore = "reads a file outside the repository; run by hand"]
fn what_is_under_his_text_at_each_tolerance() {
    let Some(model) = page_model() else { return };
    let mut exact_is_text = 0usize;
    let mut working_is_text = 0usize;

    for (label, x, y) in AIMS {
        let point = Point::new(x + NUDGE, y + NUDGE);
        println!("\n=== {label}  at ({:.1}, {:.1})", point.x, point.y);
        for tolerance in TOLERANCES {
            let hits = pdfcer_core::vector::hit_test_point_deep(&model, point, tolerance);
            let rendered: Vec<String> = hits.iter().map(|h| class_of(&model, *h)).collect();
            println!(
                "  tol {tolerance:>4.1}: {} candidate(s)  [{}]",
                hits.len(),
                rendered.join(", ")
            );
            let top_is_text = hits.first().is_some_and(|h| is_text(&model, *h));
            if tolerance == 0.0 && top_is_text {
                exact_is_text += 1;
            }
            if (tolerance - 8.0).abs() < f64::EPSILON && top_is_text {
                working_is_text += 1;
            }
        }
    }

    println!("\n---------------------------------------------------------------");
    println!(
        "at tolerance 0.0 the FRONTMOST candidate is text at {exact_is_text} of {} aims",
        AIMS.len()
    );
    println!(
        "at tolerance 8.0 the FRONTMOST candidate is text at {working_is_text} of {} aims",
        AIMS.len()
    );
    println!(
        "\nRead the two numbers together. If the first is high and the second is low, the\n\
         path is winning on SLACK and the remedy is a re-rank the shell can do with the\n\
         engine API it already calls: ask at zero tolerance first, put those hits at the\n\
         front, and append the tolerance hits behind them. If BOTH are low, the text is\n\
         not a candidate at these points at all and the remedy is somewhere else\n\
         entirely - `decompose_page`, or the aim."
    );
}

/// **Question 3: is there a text candidate at all, and where in the list?**
///
/// The complement of the test above, stated as the thing a fix would have to
/// achieve rather than as a description of today. A re-rank can only help if
/// the text is IN the list; if it is absent the whole approach is wrong, and
/// that is worth failing loudly about rather than discovering later.
#[test]
#[ignore = "reads a file outside the repository; run by hand"]
fn how_deep_is_his_text_in_the_candidate_list() {
    let Some(model) = page_model() else { return };
    let tolerance = 8.0;
    let mut reachable = 0usize;
    let mut absent: Vec<&str> = Vec::new();

    for (label, x, y) in AIMS {
        let point = Point::new(x + NUDGE, y + NUDGE);
        let hits = pdfcer_core::vector::hit_test_point_deep(&model, point, tolerance);
        match hits.iter().position(|h| is_text(&model, *h)) {
            Some(depth) => {
                reachable += 1;
                println!(
                    "{label:<32} text at depth {depth} of {} - {}",
                    hits.len(),
                    if depth == 0 {
                        "a plain click reaches it"
                    } else {
                        "only an Alt-click reaches it"
                    }
                );
            }
            None => {
                absent.push(label);
                println!("{label:<32} NO TEXT CANDIDATE in {} hit(s)", hits.len());
            }
        }
    }

    println!("\n---------------------------------------------------------------");
    println!(
        "text is somewhere in the list at {reachable} of {} aims; absent at {}",
        AIMS.len(),
        if absent.is_empty() {
            "none".to_owned()
        } else {
            absent.join(", ")
        }
    );
}
