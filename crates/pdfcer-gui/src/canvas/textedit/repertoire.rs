//! # `canvas::textedit::repertoire` — the alphabet this run accepts, asked once
//! when the caret lands instead of discovered one refused keystroke at a time
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/repertoire.md`.

use std::sync::Arc;

use pdfcer_core::text_edit::RunRepertoire;

use crate::app::state::OpenDoc;

/// The `egui` temp-memory key the measurement is parked under.
const MEMORY_KEY: &str = "pdfcer.textedit.repertoire";

/// One measurement, with everything needed to know it is still the right one.
#[derive(Clone)]
struct Held {
    /// Which page the run is on.
    page: usize,
    /// Which run on it.
    run: usize,
    /// The document revision the measurement describes.
    epoch: u64,
    /// The answer — `None` when the attempt was made and failed. **Not** an
    /// absence of an attempt; see the module header.
    rep: Option<Arc<RunRepertoire>>,
}

/// **The repertoire for `run` on `page`, measured at most once per revision.**
#[must_use]
pub(crate) fn of_run(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page: usize,
    run: usize,
) -> Option<Arc<RunRepertoire>> {
    let epoch = doc.edit_epoch;
    if let Some(held) = read(ctx)
        && held.page == page
        && held.run == run
        && held.epoch == epoch
    {
        return held.rep;
    }
    let rep = measure(doc, page, run);
    // WRITTEN ON BOTH ARMS, never behind a `?`. See the module header: this
    // line, not the key's position, is what stops a failed measurement being
    // re-attempted on every frame the caret sits in the run.
    write(
        ctx,
        Held {
            page,
            run,
            epoch,
            rep: rep.clone(),
        },
    );
    rep
}

/// **What one keystroke's text survives as** — the characters the run will
/// take, and every one it will not.
pub(crate) struct Sieved {
    /// The typed characters the run will take, in the order they were typed.
    ///
    /// Empty when every character was refused, which is the ordinary case for a
    /// single keystroke that hit the wall.
    pub kept: String,
    /// Every character the run will not take, in the order typed, and the
    /// `/BaseFont` that will not take them. `None` when all were taken.
    ///
    /// The font travels with the character because the offer needs both:
    /// `panels::properties::refusedchar` names the face being replaced and
    /// keys its list of candidates on the character.
    pub refused: Option<(Vec<char>, String)>,
    /// Those refused characters the font draws with more than one code
    /// (`RunRepertoire::ambiguous`) rather than lacks.
    pub two_ways: Vec<char>,
}

/// **Split a keystroke's text into what this run can take and what it cannot.**
#[must_use]
pub(crate) fn sieve(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page: usize,
    run: usize,
    typed: &str,
) -> Sieved {
    let Some(rep) = of_run(ctx, doc, page, run) else {
        return Sieved {
            kept: typed.to_owned(),
            refused: None,
            two_ways: Vec::new(),
        };
    };
    let mut kept = String::with_capacity(typed.len());
    let mut missing = Vec::new();
    for c in typed.chars() {
        if rep.accepts(c) {
            kept.push(c);
        } else {
            missing.push(c);
        }
    }
    let two_ways = missing
        .iter()
        .copied()
        .filter(|c| rep.ambiguous.contains_key(c))
        .collect();
    let refused = (!missing.is_empty()).then(|| (missing, rep.base_font.clone()));
    Sieved {
        kept,
        refused,
        two_ways,
    }
}

/// Which of `chars` the run refuses now, measured afresh (no slot: the
/// commit has no `egui::Context`). Answers all of them when nothing could be
/// measured, so the caller keeps its plan.
/// The second list is those refused the font draws two ways.
pub(crate) fn refused_now(
    doc: &OpenDoc,
    page: usize,
    run: usize,
    chars: &[char],
) -> (Vec<char>, Vec<char>) {
    let Some(rep) = measure(doc, page, run) else {
        return (chars.to_vec(), Vec::new());
    };
    let refused: Vec<char> = chars.iter().copied().filter(|c| !rep.accepts(*c)).collect();
    let two_ways = refused
        .iter()
        .copied()
        .filter(|c| rep.ambiguous.contains_key(c))
        .collect();
    (refused, two_ways)
}

/// Drop the held measurement.
pub(crate) fn forget(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<Held>(egui::Id::new(MEMORY_KEY)));
}

/// Read the slot without measuring.
fn read(ctx: &egui::Context) -> Option<Held> {
    ctx.data(|d| d.get_temp::<Held>(egui::Id::new(MEMORY_KEY)))
}

/// Write the slot.
fn write(ctx: &egui::Context, held: Held) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(MEMORY_KEY), held));
}

/// Ask the engine, once.
fn measure(doc: &OpenDoc, page: usize, run: usize) -> Option<Arc<RunRepertoire>> {
    let pin = super::pin::resolve(doc, page, run)?;
    let started = std::time::Instant::now();
    let options =
        super::installed::augmented(doc, pdfcer_gui_base::editmodel::disposition::typing());
    let answer = doc
        .session
        .run_repertoire_with(page, "", Some(pin.span), &options);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        //
        // `reason` is the engine's own sentence and this is the ONE place
        // it is allowed to appear. `super::Refusal::NoUsableEncoding`'s docs
        // carry the rule — say where in HIS vocabulary, never in the engine's —
        // so the clause number goes here, where a reader wants it, and the
        // operator gets `crate::text::textedit::refusal`.
        let ms = started.elapsed().as_millis();
        match &answer {
            Ok(rep) => format!(
                "run-repertoire page={page} run={run} ms={ms} ok=1 accepted={} tested={} \
                 subset={} editable={} font={} reason={}",
                rep.accepted.len(),
                rep.candidates_tested,
                u8::from(rep.embedded_subset),
                u8::from(rep.is_editable()),
                rep.base_font,
                rep.reason.as_deref().unwrap_or("none"),
            ),
            Err(e) => format!("run-repertoire page={page} run={run} ms={ms} ok=0 said={e}"),
        }
    });
    answer.ok().map(Arc::new)
}

// ---------------------------------------------------------------------------
// Tests
//
// Two of these drive the ENGINE, not this module, and they are the reason
// the module is allowed to exist in the shape it has. `of_run` is a cache; a
// cache is easy to test and proves nothing about whether the answer it keeps is
// the right one to gate a keystroke on. The engine wrote down one direction —
// *"a character in `accepted` is one `edit_text` will not refuse for this
// run"* — and this module leans on the CONVERSE, which the engine did not write
// down. `the_engine_refuses_a_character_the_repertoire_omits` is that converse,
// measured rather than argued, against a font whose floor actually bites.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use pdfcer_core::document::Document;
    use pdfcer_core::edit::EditSession;
    use pdfcer_core::span::ByteSpan;
    use pdfcer_core::text_edit::{
        BlockRecognitionOptions, EditOptions, EditRequest, EditableTextModel,
    };

    use super::*;
    use crate::app::state::open_local_fixture;

    /// The one fixture in this repository whose font floor can actually refuse a
    /// keystroke.
    const FIXTURE: &str = "subset-font-floor.pdf";

    /// The three letters the fixture's single run prints, which are therefore
    /// three of the letters its subset embedding carries.
    const IN_THE_SUBSET: [char; 3] = ['A', 'B', 'C'];

    /// A character the subset font does not carry.
    const OUTSIDE_THE_SUBSET: char = 'q';

    /// A run index the fixture's one-run page does not have, used to reach
    /// [`measure`]'s failure arm without corrupting anything.
    const NO_SUCH_RUN: usize = 9_999;

    /// A bare context. `egui`'s memory is reachable outside a frame, which is
    /// what lets these run without a viewport.
    fn ctx() -> egui::Context {
        egui::Context::default()
    }

    /// The fixture's path, asserted to exist so a missing file fails as itself
    /// rather than as an unhelpful load error.
    fn fixture_path() -> std::path::PathBuf {
        let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(FIXTURE);
        assert!(
            p.exists(),
            "the fixture is missing at {}; see fixtures/subset-font-floor.PROVENANCE.md",
            p.display()
        );
        p
    }

    /// A fresh mutable session over the fixture.
    fn raw_session() -> EditSession {
        EditSession::new(Document::load(&fixture_path()).expect("the fixture loads"))
    }

    /// The pin for the fixture's run 0, measured from `session`'s **current**
    /// view.
    fn pin_of_run_zero(session: &EditSession) -> (ByteSpan, pdfcer_core::text_edit::EditTarget) {
        let view = session.view();
        let pages = pdfcer_core::page_tree::pages_in(&view).expect("a page tree");
        let extracted = pdfcer_core::text_extract::extract_page_view(
            &view,
            &pages[0],
            0,
            &pdfcer_core::text_extract::ExtractOptions::default().with_provenance(true),
        )
        .expect("the page's text extracts");
        let model = EditableTextModel::recognize(&extracted, &BlockRecognitionOptions::default());
        let p = super::super::pin::of_run(&model, 0).expect("run 0 carries provenance");
        (p.span, p.target)
    }

    /// Replace the fixture's run with `text`, the way the shell's commit does —
    /// a pinned whole-operator request carrying the target buffer.
    ///
    /// Returns the engine's own sentence on refusal, so an unexpected failure
    /// reports what the engine said rather than a bare `false`.
    fn type_into_run_zero(session: &mut EditSession, text: &str) -> Result<(), String> {
        let (span, target) = pin_of_run_zero(session);
        let mut req = EditRequest::whole_operator(0, span, text);
        req.target = target;
        session
            .edit_text(&req, &EditOptions::default())
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// **The fixture's run is editable, and its alphabet is the subset.**
    #[test]
    fn the_fixture_run_is_editable_and_its_alphabet_is_the_subset() {
        let doc = open_local_fixture(FIXTURE);
        let ctx = ctx();

        let rep = of_run(&ctx, &doc, 0, 0).expect("run 0 pins and the engine answers");

        assert!(
            rep.is_editable(),
            "the fixture's run must be editable or the gate below has nothing to gate; \
             the engine said {:?}",
            rep.reason
        );
        for c in IN_THE_SUBSET {
            assert!(
                rep.accepts(c),
                "'{c}' is printed by the run itself, so the embedded subset carries it"
            );
        }
        assert!(
            !rep.accepts(OUTSIDE_THE_SUBSET),
            "'{OUTSIDE_THE_SUBSET}' is outside the subset; a repertoire that accepts it \
             makes every check in this module unable to fail"
        );
    }

    /// **The engine keeps its half of the bargain: every character the
    /// repertoire names is one `edit_text` accepts.**
    #[test]
    fn the_engine_accepts_every_character_the_repertoire_names() {
        let doc = open_local_fixture(FIXTURE);
        let ctx = ctx();
        let rep = of_run(&ctx, &doc, 0, 0).expect("run 0 pins and the engine answers");

        for c in rep.accepted.iter().copied() {
            let mut session = raw_session();
            let text: String = std::iter::repeat_n(c, IN_THE_SUBSET.len()).collect();
            let outcome = type_into_run_zero(&mut session, &text);
            assert!(
                outcome.is_ok(),
                "the repertoire named '{c}' (U+{:04X}) as acceptable and the engine then \
                 refused it: {}",
                u32::from(c),
                outcome.unwrap_err()
            );
        }
    }

    /// **The converse, which the engine did NOT write down and this module
    /// depends on: a character absent from the repertoire is one `edit_text`
    /// refuses.**
    #[test]
    fn the_engine_refuses_a_character_the_repertoire_omits() {
        let mut session = raw_session();
        let text: String = format!("AB{OUTSIDE_THE_SUBSET}");

        let outcome = type_into_run_zero(&mut session, &text);

        assert!(
            outcome.is_err(),
            "the repertoire omits '{OUTSIDE_THE_SUBSET}', so the pre-commit gate refuses that \
             key; the engine accepting it here would mean the gate is refusing input this \
             document could have taken"
        );
    }

    /// **A run that cannot be measured records the ATTEMPT.**
    #[test]
    fn an_unmeasurable_run_records_the_attempt() {
        let doc = open_local_fixture(FIXTURE);
        let ctx = ctx();

        assert!(
            of_run(&ctx, &doc, 0, NO_SUCH_RUN).is_none(),
            "a run the page does not have cannot be pinned"
        );

        let held = read(&ctx).expect("the failed attempt is recorded, not skipped");
        assert_eq!(
            held.run, NO_SUCH_RUN,
            "recorded against the run that failed"
        );
        assert_eq!(held.page, 0);
        assert!(
            held.rep.is_none(),
            "and recorded as a failure, not as an absence of a record"
        );
    }

    /// **The caret moving to another run takes another measurement.**
    ///
    /// One slot, not a map: the caret is in one run at a time, so a second run's
    /// arrival replaces the first's answer rather than accumulating beside it.
    #[test]
    fn a_second_run_replaces_the_measurement() {
        let doc = open_local_fixture(FIXTURE);
        let ctx = ctx();

        let _ = of_run(&ctx, &doc, 0, 0);
        assert_eq!(read(&ctx).expect("measured").run, 0);

        let _ = of_run(&ctx, &doc, 0, NO_SUCH_RUN);
        assert_eq!(
            read(&ctx).expect("measured again").run,
            NO_SUCH_RUN,
            "the slot describes where the caret is now, never where it was"
        );
    }

    /// **An edit invalidates the measurement.**
    #[test]
    fn an_edit_invalidates_the_measurement() {
        let mut doc = open_local_fixture(FIXTURE);
        let ctx = ctx();

        let _ = of_run(&ctx, &doc, 0, 0);
        assert_eq!(
            read(&ctx).expect("measured").epoch,
            0,
            "a freshly opened document is at epoch zero"
        );

        doc.edit_epoch += 1;
        let _ = of_run(&ctx, &doc, 0, 0);

        assert_eq!(
            read(&ctx).expect("measured again").epoch,
            1,
            "the same run at a new revision is a different question"
        );
    }

    /// **The sieve keeps what the run takes and names what it does not.**
    #[test]
    fn the_sieve_splits_a_mixed_keystroke() {
        let doc = open_local_fixture(FIXTURE);
        let ctx = ctx();

        let out = sieve(&ctx, &doc, 0, 0, &format!("A{OUTSIDE_THE_SUBSET}B"));

        assert_eq!(out.kept, "AB", "the letters the subset carries survive");
        let (c, font) = out.refused.expect("and the one it does not is named");
        assert_eq!(c, vec![OUTSIDE_THE_SUBSET]);
        assert!(
            !font.is_empty(),
            "the face travels with the character, because the offer names the face it \
             is replacing"
        );
    }

    /// **Every refused character is reported, in the order typed.**
    #[test]
    fn the_sieve_names_every_refusal() {
        let doc = open_local_fixture(FIXTURE);
        let ctx = ctx();

        let out = sieve(&ctx, &doc, 0, 0, "qzA");

        assert_eq!(out.kept, "A");
        assert_eq!(
            out.refused.map(|(c, _)| c),
            Some(vec!['q', 'z']),
            "each one typed, in order"
        );
    }

    /// **An unmeasured run keeps every character the operator typed.**
    #[test]
    fn the_sieve_keeps_everything_when_nothing_was_measured() {
        let doc = open_local_fixture(FIXTURE);
        let ctx = ctx();

        let out = sieve(&ctx, &doc, 0, NO_SUCH_RUN, "q\u{20ac}Z");

        assert_eq!(
            out.kept, "q\u{20ac}Z",
            "not measured is permission to proceed"
        );
        assert!(out.refused.is_none(), "and nothing to say about it");
    }

    /// **Abandoning a draft clears the slot.**
    #[test]
    fn forgetting_clears_the_slot() {
        let doc = open_local_fixture(FIXTURE);
        let ctx = ctx();

        let _ = of_run(&ctx, &doc, 0, 0);
        assert!(read(&ctx).is_some(), "measured");

        forget(&ctx);

        assert!(read(&ctx).is_none(), "and gone with the draft");
    }
}
