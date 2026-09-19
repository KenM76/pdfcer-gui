//! **A probe, not a test** — measuring the operator's O214 ask against his own
//! file, headlessly.
//!
//! > *"I'd also like to be able to take a line like this that is part of a
//! > larger block and be able to relocate it by dragging and moving as if it
//! > wasn't part of a larger block."*
//!
//! The shell already reaches `EditSession::move_text_run`, whose unit is **one
//! show operator**. His unit is **one visual line**, and on this sheet those are
//! not the same thing. Two questions decide which route answers him, and
//! neither can be reasoned about from the engine's documentation:
//!
//! 1. **How is the line positioned?** If fragments 2..n inherit their position
//!    from fragment 1, then moving fragment 1 already carries the rest — and the
//!    engine's `MoveWouldMoveNextRun` guard refuses exactly the move he wants.
//!    If every fragment states its own position, nothing inherits and the line
//!    can only be moved by moving all of them.
//! 2. **Where does `SplitGranularity::Line` cut?** It groups *consecutive* runs
//!    sharing a baseline, in stream order. Whether that reconstructs his visual
//!    line, or something coarser or finer, is a property of his producer.
//!
//! ★ Written as a file rather than as a shell one-liner because the question is
//! *what does the engine say about each run*, and that needs the engine, not a
//! byte grep — which is what produced two wrong diagnoses of his last text
//! report.
//!
//! # What it answered
//!
//! Page 0, text object **5871** — the notes column, **237 show operators in
//! one `BT`…`ET`**, **144 lines**.
//!
//! 1. **Nothing inherits.** All nine fragments of
//!    `#2 USE SPACERS 8 9 10 11 IF REQUIRED.` are `RunPositioning::Explicit`
//!    and `text_run_move_refusal` returns `None` for every one. So the line can
//!    only be moved by moving all nine, and all nine moves are planned.
//! 2. **`SplitGranularity::Line` reconstructs his line exactly** — 143 cuts,
//!    144 pieces, his line is piece 49 = runs 50..59.
//!
//! And two things that are not in the questions but decide the design:
//!
//! - **`move_objects` refuses a text object.** Splitting the line into its own
//!   object works and the index arithmetic is `object + piece`, verified by
//!   reading object 5920's own text back — but nothing will then move it.
//!   Request `G029`; the split route is dead for moving.
//! - **A click on the words lands on run 53**, a 19 pt fragment in the middle
//!   of them. That is his complaint made concrete: the run-unit drag moves
//!   those glyphs and leaves the other eight fragments behind.
//!
//! ⚠ **The object model cannot tell you what a run says.**
//! `TextPreview::Decoded` is capped with no knob and every run of a CAD text
//! object comes back empty, so [`locate`] bridges from `text_extract` — which
//! has no cap — through a **point**, the only currency that crosses. Request
//! `G031`.
//!
//! It reads a file that is **not in this repository** — a copy of his drawing
//! under `target/scratch/` — so it is `#[ignore]`d and can never fail a build.
//!
//! Run with:
//!
//! ```text
//! cargo test --test ken_sw41177_line_move_probe -- --ignored --nocapture
//! ```

use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;
use pdfcer_core::vector::{Point, SplitGranularity, TextObject, TextPreview, VectorObject};

/// The pick tolerance, in page units. The shell derives its own from the
/// zoom; on a 1,584 pt sheet fitted to about a thousand pixels one pixel is
/// roughly 1.6 pt, so this is about two pixels' worth.
const TOLERANCE: f64 = 3.0;

const FIXTURE: &str = "target/scratch/docs/SW41177.pdf";

/// The line he named.
const HIS_TEXT: &str = "USE SPACERS";

fn session() -> Option<EditSession> {
    let path = format!("{}/../../{FIXTURE}", env!("CARGO_MANIFEST_DIR"));
    let p = std::path::Path::new(&path);
    if !p.exists() {
        println!("SKIP: {path} is not present — copy his file there first");
        return None;
    }
    Some(EditSession::new(Document::load(p).expect("his file loads")))
}

/// Everything the object shows, as one string — the concatenation the runs
/// index into.
fn whole_text(text: &TextObject) -> &str {
    match &text.preview {
        TextPreview::Decoded { text, .. } => text.as_str(),
        _ => "",
    }
}

/// One run's own characters, sliced out of the object's preview by the byte
/// range the run carries.
///
/// ⚠ `text_start`/`text_end` are indices into [`whole_text`], which is a
/// **truncated** preview when the object is long. A run past the truncation
/// point has a range the string cannot satisfy, and that is a fact about the
/// preview rather than about the run — so it is reported as `…` rather than
/// panicking and losing every later measurement.
fn run_text(text: &TextObject, run: usize) -> &str {
    let whole = whole_text(text);
    let r = text.runs[run].text_range();
    whole.get(r).unwrap_or("…")
}

/// Where his line is, in page space — found the way he finds it, by reading
/// the page's text rather than the object model's preview.
///
/// ⚠ **`TextPreview::Decoded` is capped and every object that matters here is
/// `truncated`.** His notes column is one text object of 119 show operators and
/// the preview stops long before `USE SPACERS`, so a `contains` over the model
/// reports NOT FOUND on a page that plainly holds the words. Extraction has no
/// such cap, so the phrase is found there and carried into the model as a
/// POINT — which is also the only currency a click has.
fn aim(session: &EditSession) -> Option<(usize, Point)> {
    let view = session.view();
    let pages = pdfcer_core::page_tree::pages_in(&view).ok()?;
    for (index, page) in pages.iter().enumerate() {
        let Ok(text) = pdfcer_core::text_extract::extract_page_view(
            &view,
            page,
            index,
            &pdfcer_core::text_extract::ExtractOptions::default(),
        ) else {
            continue;
        };
        for run in &text.runs {
            if !run.text.contains(HIS_TEXT) {
                continue;
            }
            let b = run.bbox?;
            // The middle of the run's box, which is where a pointer lands when
            // he clicks the words rather than their corner.
            return Some((
                index,
                Point {
                    x: f64::midpoint(b.llx, b.urx),
                    y: f64::midpoint(b.lly, b.ury),
                },
            ));
        }
    }
    None
}

/// The page, the paint-order object index and the run index his click reaches
/// — through `hit_test_point_all` and `hit_test_text_runs`, the two calls
/// `canvas::picking` makes, so this measures the shell's route and not a
/// shortcut past it.
fn locate(session: &mut EditSession) -> Option<(usize, usize, usize, Point)> {
    let (page, point) = aim(session)?;
    let model = session.page_objects(page).ok()?;
    for object in pdfcer_core::vector::hit_test_point_all(&model, point, TOLERANCE) {
        if !matches!(model.objects.get(object), Some(VectorObject::Text(_))) {
            continue;
        }
        let runs = pdfcer_core::vector::hit_test_text_runs(&model, object, point, TOLERANCE);
        if let Some(&run) = runs.first() {
            return Some((page, object, run, point));
        }
    }
    None
}

#[test]
#[ignore = "reads his own drawing from target/scratch — not in this repository"]
fn how_his_line_is_positioned_and_where_a_line_split_would_cut() {
    let Some(mut session) = session() else {
        return;
    };
    let Some((page, object, run, point)) = locate(&mut session) else {
        println!(
            "NOT FOUND: nothing on any page extracts `{HIS_TEXT}`, or the point it gave misses every text object"
        );
        return;
    };
    println!(
        "page {page}, object {object}, run {run} — the click lands at ({:.1}, {:.1})",
        point.x, point.y
    );

    let model = session.page_objects(page).expect("the page decomposes");
    let VectorObject::Text(text) = &model.objects[object] else {
        unreachable!("located as a text object")
    };
    println!("the object holds {} show operators", text.runs.len());

    // --- 1: how each run is positioned, and what the engine says about moving
    //        it on its own ---------------------------------------------------
    //
    // `Inherited` is the interesting value: a run positioned that way has no
    // number of its own to change, and the run BEFORE it cannot move without
    // dragging it along. Both are refusals the shell already words.
    let first = run;
    let lo = first.saturating_sub(6);
    let hi = (first + 7).min(text.runs.len());
    println!("\n  run | positioned_by | move refusal | llx      | lly      | text");
    for r in lo..hi {
        let refusal = pdfcer_core::vector::edit::text_run_move_refusal(text, r)
            .map_or_else(|| "—  (would be planned)".to_owned(), |e| format!("{e:?}"));
        let b = text.runs[r].bounds;
        let mark = if r == run { '*' } else { ' ' };
        println!(
            "{mark} {r:>3} | {:<13} | {refusal:<40} | {:>8.2} | {:>8.2} | {:?}",
            format!("{:?}", text.runs[r].positioned_by),
            b.min.x,
            b.min.y,
            run_text(text, r)
        );
    }

    // --- 2: where a Line split would cut -----------------------------------
    let (points, disclosures) = session
        .text_object_split_plan(page, object, SplitGranularity::Line)
        .expect("the split plan is computable on a text object");
    println!(
        "\nLine granularity cuts before runs {points:?} — {} pieces from {} operators",
        points.len() + 1,
        text.runs.len()
    );
    for d in &disclosures {
        println!("  disclosure: {d}");
    }

    // Which piece would hold his line, and what else would ride with it.
    let piece = points.partition_point(|&p| p <= first);
    let start = if piece == 0 { 0 } else { points[piece - 1] };
    let end = points.get(piece).copied().unwrap_or(text.runs.len());
    println!(
        "his line falls in piece {piece} of {} — runs {start}..{end}, which shows: {:?}",
        points.len() + 1,
        (start..end).map(|r| run_text(text, r)).collect::<String>()
    );

    // --- 3: what a Run split would cut, for comparison ----------------------
    let (run_points, _) = session
        .text_object_split_plan(page, object, SplitGranularity::Run)
        .expect("the split plan is computable at Run granularity too");
    println!(
        "Run granularity cuts before {} runs — {} pieces",
        run_points.len(),
        run_points.len() + 1
    );
}

#[test]
#[ignore = "reads his own drawing from target/scratch — not in this repository"]
fn splitting_at_line_granularity_then_moving_the_piece_relocates_his_line() {
    let Some(mut session) = session() else {
        return;
    };
    let Some((page, object, first, _)) = locate(&mut session) else {
        println!(
            "NOT FOUND: nothing on any page extracts `{HIS_TEXT}`, or the point it gave misses every text object"
        );
        return;
    };

    let before = {
        let model = session.page_objects(page).expect("the page decomposes");
        let VectorObject::Text(text) = &model.objects[object] else {
            unreachable!()
        };
        text.runs[first].bounds
    };
    let objects_before = session
        .page_objects(page)
        .expect("the page decomposes")
        .objects
        .len();

    let (points, _) = session
        .text_object_split_plan(page, object, SplitGranularity::Line)
        .expect("the split plan is computable");
    let piece = points.partition_point(|&p| p <= first);

    match session.split_text_object(page, object, &points) {
        Ok(notes) => {
            println!("split: ok, {} note(s)", notes.len());
            for n in &notes {
                println!("  {n}");
            }
        }
        Err(e) => {
            println!("SPLIT REFUSED: {e}");
            println!("⇒ this is the answer the shell would have to word. Stop here.");
            return;
        }
    }

    let objects_after = session
        .page_objects(page)
        .expect("the page decomposes")
        .objects
        .len();
    println!(
        "objects on the page: {objects_before} → {objects_after} (+{})",
        objects_after - objects_before
    );

    // ★ The pieces land where the original was, in order, so the piece holding
    // his line is `object + piece`. That is the arithmetic the shell will have
    // to do, so the probe does it the same way and then CHECKS it rather than
    // trusting it.
    let moved_index = object + piece;
    let is_his = {
        let model = session.page_objects(page).expect("the page decomposes");
        match model.objects.get(moved_index) {
            Some(VectorObject::Text(t)) => {
                let shown = whole_text(t).to_owned();
                println!("object {moved_index} after the split shows: {shown:?}");
                shown.contains(HIS_TEXT)
            }
            other => {
                println!("object {moved_index} is not a text object: {other:?}");
                false
            }
        }
    };
    if !is_his {
        println!("⇒ `object + piece` is NOT the arithmetic. Find the real rule before wiring it.");
        return;
    }

    match session.move_objects(page, &[moved_index], 0.0, -30.0) {
        Ok(notes) => println!("move: ok, {} note(s)", notes.len()),
        Err(e) => {
            println!("MOVE REFUSED: {e}");
            return;
        }
    }

    let after = {
        let model = session.page_objects(page).expect("the page decomposes");
        let VectorObject::Text(text) = &model.objects[moved_index] else {
            unreachable!()
        };
        text.runs[0].bounds
    };
    println!(
        "his line's first fragment: ({:.2}, {:.2}) → ({:.2}, {:.2}) — dx {:+.2}, dy {:+.2}",
        before.min.x,
        before.min.y,
        after.min.x,
        after.min.y,
        after.min.x - before.min.x,
        after.min.y - before.min.y
    );
}

/// What the page actually holds — run this when [`locate`] says NOT FOUND, to
/// find out whether the phrase is missing, truncated out of a preview, or
/// inside a form XObject the page-level decomposition does not descend into.
#[test]
#[ignore = "reads his own drawing from target/scratch — not in this repository"]
fn survey_the_text_objects() {
    let Some(mut session) = session() else {
        return;
    };
    let page_count = pdfcer_core::page_tree::pages_in(&session.view()).map_or(0, |p| p.len());
    println!("{page_count} page(s)");
    for page in 0..page_count {
        let Ok(model) = session.page_objects(page) else {
            println!("page {page}: does not decompose");
            continue;
        };
        let texts = model
            .objects
            .iter()
            .filter(|o| matches!(o, VectorObject::Text(_)))
            .count();
        println!(
            "page {page}: {} object(s), {texts} of them text",
            model.objects.len()
        );
        for (index, object) in model.objects.iter().enumerate() {
            let VectorObject::Text(text) = object else {
                continue;
            };
            let (kind, shown, trunc) = match &text.preview {
                TextPreview::Decoded {
                    text, truncated, ..
                } => ("Decoded", text.clone(), *truncated),
                other => (
                    match other {
                        TextPreview::Unavailable => "Unavailable",
                        _ => "other",
                    },
                    String::new(),
                    false,
                ),
            };
            let head = shown.chars().take(70).collect::<String>();
            println!(
                "  obj {index:>4}: {:>4} run(s) {kind}{} {head:?}",
                text.runs.len(),
                if trunc { " TRUNCATED" } else { "" }
            );
        }
    }
}

/// **The route that needs no split**: move every show operator the line is
/// written in, one `move_text_run` per fragment.
///
/// Its cost is that the engine has no plural form, so nine fragments are nine
/// commands and therefore nine undo entries unless something coalesces them.
#[test]
#[ignore = "reads his own drawing from target/scratch — not in this repository"]
fn moving_every_fragment_of_the_line_relocates_it_without_restructuring_anything() {
    let Some(mut session) = session() else {
        return;
    };
    let Some((page, object, run, _)) = locate(&mut session) else {
        println!("NOT FOUND");
        return;
    };
    let (points, _) = session
        .text_object_split_plan(page, object, SplitGranularity::Line)
        .expect("the split plan is computable");
    // The same grouping the split would make, used as a READING of where the
    // line starts and stops rather than as a mutation.
    let piece = points.partition_point(|&p| p <= run);
    let start = if piece == 0 { 0 } else { points[piece - 1] };
    let end = {
        let model = session.page_objects(page).expect("the page decomposes");
        let VectorObject::Text(text) = &model.objects[object] else {
            unreachable!()
        };
        points.get(piece).copied().unwrap_or(text.runs.len())
    };
    println!(
        "his line is runs {start}..{end} — {} fragments",
        end - start
    );

    let before: Vec<_> = {
        let model = session.page_objects(page).expect("the page decomposes");
        let VectorObject::Text(text) = &model.objects[object] else {
            unreachable!()
        };
        (start..end).map(|r| text.runs[r].bounds.min).collect()
    };

    let mut notes = 0usize;
    for r in start..end {
        match session.move_text_run(page, object, r, 0.0, -30.0) {
            Ok(d) => notes += d.len(),
            Err(e) => {
                println!("run {r} REFUSED: {e}");
                return;
            }
        }
    }
    println!("all {} fragments moved, {notes} disclosure(s)", end - start);

    let after: Vec<_> = {
        let model = session.page_objects(page).expect("the page decomposes");
        let VectorObject::Text(text) = &model.objects[object] else {
            unreachable!()
        };
        (start..end).map(|r| text.runs[r].bounds.min).collect()
    };
    for (i, (b, a)) in before.iter().zip(&after).enumerate() {
        println!(
            "  run {:>3}: ({:>8.2}, {:>8.2}) -> ({:>8.2}, {:>8.2})  dx {:+.2} dy {:+.2}",
            start + i,
            b.x,
            b.y,
            a.x,
            a.y,
            a.x - b.x,
            a.y - b.y
        );
    }

    // And nothing else on the page moved: the line above and the line below.
    let model = session.page_objects(page).expect("the page decomposes");
    let VectorObject::Text(text) = &model.objects[object] else {
        unreachable!()
    };
    for r in [start.saturating_sub(1), end] {
        if r < text.runs.len() {
            let b = text.runs[r].bounds.min;
            println!("  neighbour run {r}: ({:.2}, {:.2})", b.x, b.y);
        }
    }
}

/// **Does `move_objects` move a whole text object?**
///
/// `crate::text::arrange`'s two refusal sentences both end *"press Escape to
/// select the whole block of text and drag that"*, and that remedy is only
/// honest if the Object rung's verb covers text. `G029` found it refusing a
/// text object produced by a split; this asks the same question of a text
/// object the file itself contains, because a wrong remedy sentence sends the
/// operator to a gesture that does nothing and reports nothing.
#[test]
#[ignore = "reads his own drawing from target/scratch — not in this repository"]
fn whether_the_escape_remedy_reaches_a_verb_that_moves_a_whole_text_object() {
    let Some(mut session) = session() else {
        return;
    };
    let Some((page, object, _, _)) = locate(&mut session) else {
        println!("NOT FOUND");
        return;
    };
    match session.move_objects(page, &[object], 0.0, -10.0) {
        Ok(notes) => println!(
            "move_objects on text object {object}: OK, {} note(s)",
            notes.len()
        ),
        Err(e) => println!("move_objects on text object {object}: REFUSED — {e}"),
    }
    match session.transform_objects(
        page,
        &[object],
        pdfcer_core::vector::Matrix::IDENTITY,
        pdfcer_core::vector::TransformOptions::default(),
    ) {
        Ok(_) => println!("transform_objects on text object {object}: OK"),
        Err(e) => println!("transform_objects on text object {object}: REFUSED — {e}"),
    }
}

/// **How often does one "line" hold pieces that are not one line?**
///
/// `SplitGranularity::Line` groups consecutive show operators whose text
/// matrices agree on `a`, `b`, `c`, `d` and agree on `f` within a scaled
/// tolerance. It never reads `e` — the horizontal translation — so there is no
/// gap criterion, and a table row whose cells were emitted consecutively
/// becomes one movable, deletable, redactable unit.
///
/// The probe reports, per text object: how many groups hold more than one
/// operator, and for the widest of them the x-extent of each piece and the
/// clear gap between consecutive pieces. A gap of tens of points between two
/// pieces of one "line" is the defect, stated in the producer's own geometry
/// rather than in the shell's opinion of it.
#[test]
#[ignore = "reads his own drawing from target/scratch — not in this repository"]
fn how_many_lines_weld_pieces_that_are_separated_by_clear_space() {
    let Some(mut session) = session() else {
        return;
    };
    let page_count = pdfcer_core::page_tree::pages_in(&session.view()).map_or(0, |p| p.len());
    let mut widest: Vec<(f64, String)> = Vec::new();
    for page in 0..page_count {
        let Ok(model) = session.page_objects(page) else {
            continue;
        };
        for (index, object) in model.objects.iter().enumerate() {
            let VectorObject::Text(text) = object else {
                continue;
            };
            let cuts =
                pdfcer_core::vector::edit::text_object_split_points(text, SplitGranularity::Line);
            let mut starts = vec![0usize];
            starts.extend_from_slice(&cuts);
            let mut multi = 0usize;
            for (piece, &start) in starts.iter().enumerate() {
                let end = starts.get(piece + 1).copied().unwrap_or(text.runs.len());
                if end - start < 2 {
                    continue;
                }
                multi += 1;
                // The widest clear space inside this group, and the whole
                // group's geometry alongside it so the number can be checked.
                let mut gap = 0.0f64;
                let mut shape = String::new();
                for r in start..end {
                    let b = text.runs[r].bounds;
                    if r > start {
                        let clear = b.min.x - text.runs[r - 1].bounds.max.x;
                        gap = gap.max(clear);
                        shape.push_str(&format!(" |{clear:+.1}| "));
                    }
                    shape.push_str(&format!("[{:.1}..{:.1}]", b.min.x, b.max.x));
                }
                widest.push((
                    gap,
                    format!("page {page} obj {index} piece {piece} (runs {start}..{end}): {shape}"),
                ));
            }
            println!(
                "page {page} obj {index:>5}: {:>4} operator(s), {:>4} line(s), {multi:>4} of them hold more than one",
                text.runs.len(),
                starts.len()
            );
        }
    }
    widest.sort_by(|a, b| b.0.total_cmp(&a.0));
    println!("\nthe twelve widest clear gaps welded inside one line:");
    for (gap, what) in widest.iter().take(12) {
        println!("  {gap:8.1}pt  {what}");
    }
    println!(
        "\n{} line(s) across the document hold more than one operator",
        widest.len()
    );
}
