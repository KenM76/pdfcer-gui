//! **A probe, not a test** — measuring the operator's 2026-09-08 report about
//! the Text box markup tool.
//!
//! > *"when I use the 'Text box' Markup tool, making new lines by pressing
//! > enter just has the items show up as one line with a `?` for each new line
//! > instead."*
//!
//! # The question, and why it needs measuring rather than reading
//!
//! `canvas::textedit::keys` already makes a plain Enter mean **line break** in
//! a box draft — that half is not in doubt, it has its own contract table and
//! its own tests. So the newline reaches the engine inside
//! `AddTextRequest::text`, and what happens after that is the question.
//!
//! Reading `linebreak.rs` suggests an answer: its header says the greedy packer
//! breaks on **U+0020 only** (*"no hyphenation — whitespace-U+0020 breaks
//! only"*). If that is the whole story, a `\n` is an ordinary character inside
//! a word, gets encoded against a Standard-14 face that has no glyph for
//! U+000A, and renders as the notdef box the operator is seeing.
//!
//! ⚠ **That is a reading, not a measurement**, and this session has already
//! produced three precisely-located wrong diagnoses from exactly that. So this
//! performs the edit and reads the page back.
//!
//! Run with:
//!
//! ```text
//! cargo test --test ken_textbox_newline_probe -- --ignored --nocapture
//! ```

use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;

/// A fixture from this repository, so this probe can run anywhere.
const FIXTURE: &str = "fixtures/a1-titleblock.pdf";

/// Exactly what he types: two lines, separated by one Enter.
const TWO_LINES: &str = "FIRST LINE\nSECOND LINE";

fn session() -> EditSession {
    let path = format!("{}/../../{FIXTURE}", env!("CARGO_MANIFEST_DIR"));
    EditSession::new(Document::load(std::path::Path::new(&path)).expect("the fixture loads"))
}

/// The page's text as the session now sees it — the overlay, not the file.
fn page_text(session: &EditSession) -> Vec<String> {
    let view = session.view();
    let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
    pdfcer_core::text_extract::extract_page_view(
        &view,
        &pages[0],
        0,
        &pdfcer_core::text_extract::ExtractOptions::default(),
    )
    .expect("the page's text extracts")
    .runs
    .iter()
    .map(|r| r.text.clone())
    .collect()
}

/// ★★★ **What does a `\n` inside a BOXED add-text become?**
///
/// Three outcomes are possible and they want three different responses from
/// this shell, which is why the probe prints rather than asserts:
///
/// 1. **Two lines.** The engine honours it; the defect is somewhere in this
///    shell between the draft and the request, and it is ours to find.
/// 2. **One line with a notdef.** The engine encodes U+000A as a glyph. That
///    is an engine gap, and the request writes itself.
/// 3. **A refusal.** The engine already declines control characters, and this
///    shell is not surfacing the refusal — also ours.
#[test]
#[ignore = "a measurement, not an assertion — run it and read the output"]
fn what_does_a_newline_become_in_a_boxed_add_text() {
    use pdfcer_core::text_edit::AddTextRequest;

    let mut session = session();
    let before = page_text(&session).len();

    // The same shape `app::actions::addtext` builds for a box: an origin, a
    // size, and `with_box` for the rectangle the operator swept.
    let req = AddTextRequest::new(0, (100.0, 700.0), TWO_LINES)
        .with_size(12.0)
        .with_box(100.0, 640.0, 300.0, 60.0);

    match session.add_text(&req) {
        Ok(report) => {
            println!("ACCEPTED");
            println!("  {report:?}");
            let after = page_text(&session);
            println!("\nruns before: {before}, after: {}", after.len());
            for run in after.iter().skip(before) {
                println!("  new run: {run:?}");
                for c in run.chars() {
                    if !c.is_ascii_graphic() && c != ' ' {
                        println!("    ★ non-graphic char: U+{:04X}", c as u32);
                    }
                }
            }
            let joined: String = after.iter().skip(before).cloned().collect();
            println!("\n★ contains a literal newline : {}", joined.contains('\n'));
            println!("★ contains '?'                : {}", joined.contains('?'));
            println!(
                "★ new runs added              : {} (two lines should be >= 2)",
                after.len() - before
            );
        }
        Err(e) => {
            println!("REFUSED — {e}");
            println!("  debug: {e:?}");
        }
    }
}

/// The control: the same box with a **space** where the newline was.
///
/// ★ Without it, a failure above could be the fixture, the box, the origin or
/// the size rather than the newline — and the report would name the wrong
/// subject. This is the same control `glyphwall` and `facewall` both carry,
/// for the same reason.
#[test]
#[ignore = "a measurement, not an assertion — run it and read the output"]
fn the_same_box_with_a_space_instead_of_a_newline() {
    use pdfcer_core::text_edit::AddTextRequest;

    let mut session = session();
    let before = page_text(&session).len();
    let req = AddTextRequest::new(0, (100.0, 700.0), "FIRST LINE SECOND LINE")
        .with_size(12.0)
        .with_box(100.0, 640.0, 300.0, 60.0);

    match session.add_text(&req) {
        Ok(_) => {
            let after = page_text(&session);
            println!("ACCEPTED — {} new run(s)", after.len() - before);
            for run in after.iter().skip(before) {
                println!("  {run:?}");
            }
            println!(
                "⇒ if this wraps to two lines and the newline case does not, the box and \
                      the origin are fine and the newline is the whole subject"
            );
        }
        Err(e) => println!("REFUSED — {e} — the control failed, so the fixture or box is wrong"),
    }
}

/// ★★★ **THE RIGHT VERB — and the two probes above were aimed at the wrong one.**
///
/// He said the **Markup** tool's Text box. That is not `add_text`, which
/// appends a page-content run; it is `add_text_annotation` with a
/// `TextAnnotSpec::FreeText`, which authors a `/FreeText` **annotation** and
/// bakes its own appearance stream. Different verb, different layout code,
/// different font path.
///
/// ⚠ The probes above are kept rather than deleted, and they are the reason
/// this one is right: they measured `add_text` honouring the newline perfectly
/// — `wrapped_lines: Some(2)`, no `?` — which is exactly the sort of clean
/// result that would have been written up as *"cannot reproduce"*. What it
/// actually proved is that **the Edit-tab text box is fine and the Markup-tab
/// one is not**, which narrows the subject rather than clearing it.
///
/// ★ It also shows the extraction trap: a `"\n"` run appears between lines in
/// BOTH cases, including the control that contains no newline at all. That is
/// extraction marking a line boundary, not a glyph. A probe that had only run
/// the newline case would have read those as the defect.
#[test]
#[ignore = "a measurement, not an assertion — run it and read the output"]
fn what_does_a_newline_become_in_a_freetext_annotation() {
    for (label, text) in [
        ("newline", TWO_LINES),
        ("space (control)", "FIRST LINE SECOND LINE"),
    ] {
        let mut session = session();
        let spec = pdfcer_core::annot_author::TextAnnotSpec::FreeText {
            rect: pdfcer_core::page_tree::Rect {
                llx: 100.0,
                lly: 640.0,
                urx: 400.0,
                ury: 700.0,
            },
            text: text.to_owned(),
            font: pdfcer_core::fontdata::Std14::Helvetica,
            font_size: 12.0,
            color: pdfcer_core::vartext::TextColor::Gray(0.0),
            quadding: pdfcer_core::vartext::Quadding::Left,
            multiline: true,
            border: None,
            border_width: 0.0,
        };
        match session.add_text_annotation(0, &spec) {
            Ok(id) => {
                println!("\n=== {label}: ACCEPTED as {id:?} ===");
                // ★★★ **The APPEARANCE stream, not the page text.**
                //
                // A `/FreeText`'s words are drawn by its own `/AP`; the page's
                // content stream never mentions them. The first draft of this
                // probe called `page_text` and printed the FIXTURE's own title
                // block for both cases — "no difference", arrived at by
                // measuring nothing. Same trap as the `Z` in the font probe:
                // an instrument pointed at the wrong subject reports calmly.
                use pdfcer_core::graph::ObjectGraph as _;
                let graph = session.graph();
                let Some(pdfcer_core::object::Object::Dict(dict)) = session.value(id).cloned()
                else {
                    println!("  not a dictionary");
                    continue;
                };
                println!("  /Contents : {:?}", dict.get(b"Contents"));
                println!(
                    "  /AP present : {}",
                    dict.get(b"AP").map(|o| graph.resolve(o)).is_some()
                );
                // ⚠ **The appearance stream is deliberately NOT decoded here.**
                //
                // It is where the `?` is, and reading it needs the raw bytes
                // behind a `data_span` — which for a session-authored object
                // is not simply "slice the file". Two attempts at that produced
                // a probe that did not compile and one that printed the
                // FIXTURE's own title block for both cases, which is "no
                // difference" arrived at by measuring nothing.
                //
                // ⇒ The cause was established instead by reading the two
                // functions and their ORDER, which is where the fault is:
                //
                //   `vartext::encode_winansi` maps every char with no WinAnsi
                //   code to `b'?'`, and `winansi_code` covers U+0020..=U+007E
                //   plus 0x80..=0xFF — nothing in the C0 range. It runs FIRST.
                //   `vartext::wrap_lines` then splits on `b'\n'` and finds
                //   none, because the separator is now a question mark.
                //
                // That is a complete account of *"one line with a `?` for each
                // new line"*, and the remaining doubt is answerable by driving
                // the binary and looking at the box — which is a screenshot,
                // not a stream dump.
            }
            Err(e) => println!("\n=== {label}: REFUSED — {e} ==="),
        }
    }
}
