//! # `canvas::textedit::plan` — turning a caret and two strings into an
//! `EditRequest` the engine can answer
//!
//! **One function, called from one place**, and everything a text commit
//! decides is decided here: which show operator to address, whether to name it
//! by provenance pin or by its text, how the rest of the line is allowed to
//! move, and — since `OPERATOR_REQUESTS.md` **O142** — whether this shell is
//! entitled to let the engine choose an occurrence at all.
//!
//! ## The seam against [`super`]
//!
//! [`super`] is about a caret and a draft — where the operator clicked, what he
//! typed, when a draft opens and closes. This file is about **one request to
//! `pdfcer-core`**. [`plan`] and [`Plan`] are re-exported from [`super`], so
//! every caller looks for them in one place regardless.
//!
//! ## What a reader should take away before changing anything here
//!
//! The three things [`plan`] derives — all from `(page_text, run)` — are the
//! provenance pin, the matrices in force at the run's first glyph, and the
//! block's alignment. Each is re-derived from the page **as it now stands**
//! rather than carried on the `Anchor`, and that is not incidental: a value
//! sampled when the operator clicked goes stale the moment anything rebuilds
//! the page, which is `DEFECTS.md` D4b.
//!
//! ⚠ **The single most dangerous line in this file is the one that clears
//! `pinned_span`.** The pin is the only disambiguator `EditRequest` carries;
//! dropping it hands the choice of *which* occurrence to edit to the engine's
//! scan order. [`Plan::occurrences`] carries the whole argument for why that is
//! never permitted, and `super::glyphwall` holds it as tests over two fixtures
//! — one where the edit must land, one where it must be refused.

use pdfcer_core::text_edit::{
    BlockRecognitionOptions, EditOptions, EditRequest, EditableTextModel, ReflowEngine,
    TextPosition, reflow_recognition_options,
};

use crate::app::state::OpenDoc;

use super::disposition::{self, Reason};
use super::{Committing, LAST_COMMIT, pin};

/// A planned in-place edit: the request, the options, and the disclosure the
/// engine will not write for us.
pub struct Plan {
    /// The request, with its provenance pin.
    pub request: EditRequest,
    /// ★ The options, with the [`disposition`] this module exists to choose.
    pub options: EditOptions,
    /// Why that disposition, for the trace and the disclosure.
    pub reason: Reason,
    /// ★★★ **Whether the run being edited is ONE show operator** —
    /// `OPERATOR_REQUESTS.md` **O140**, and the only field here that exists to
    /// explain a *failure* rather than to shape a request.
    ///
    /// [`pin::spans_one_operator`]'s answer, already computed a few lines below
    /// to decide whether the `find` may be dropped, and until O140 it was traced
    /// and then thrown away. It is carried out because the apply arm cannot
    /// otherwise tell two identical engine refusals apart:
    ///
    /// * **one operator** — the page moved under the caret: *"pdfcer could not
    ///   find the text this edit named"*;
    /// * **several** — the reconstructed `find` could never have matched,
    ///   because no single operator holds it.
    ///
    /// The engine answers `RefusalKind::NotFound` for both, correctly and
    /// necessarily: from its side the request named text that is not in any one
    /// editable run, and it has no way to know the shell rebuilt that string
    /// from a run it had segmented itself. **This shell does know**, and this
    /// field is the whole of that knowledge.
    ///
    /// ★ `true` when there is no pin at all, which is the honest default: with
    /// no provenance the shell has measured nothing, and claiming a split it
    /// did not observe would put a confident wrong sentence in front of the
    /// operator — the one outcome `crate::text::textedit::EditRefusal`'s header
    /// argues is worse than the silence it replaces.
    pub one_operator: bool,

    /// ★★★ **How many times the text being edited appears on the page** — the
    /// count that would license dropping the provenance pin so that the
    /// engine's cross-operator matcher could reach a split run.
    /// `OPERATOR_REQUESTS.md` **O142**, the operator:
    ///
    /// > *"on page 2 there is a spelling mistake — clien instead of client. if I
    /// > try to edit the edit is not accepted."*
    ///
    /// **Always `None`.** Nothing in [`plan`] counts, because
    /// [`EditRequest::span_from_pin`](pdfcer_core::text_edit::EditRequest)
    /// disambiguates by the pin instead: the span search **starts at the pinned
    /// operator** rather than at the first operator on the page, with every
    /// other guard unchanged. `find` says *what*; the pin says *which one*.
    ///
    /// # ⚠ The count is NOT a fallback, and the difference is not a nuance
    ///
    /// `find_replace` does not edit "whichever occurrence comes first".
    /// `find_anchor` tries a **single-operator** match across the whole page
    /// before the spanning search runs at all, so a single-operator occurrence
    /// anywhere on the page beats a spanning one above it.
    ///
    /// ⇒ **A spanning run is unreachable by `find` alone whenever a
    /// single-operator twin exists anywhere on that page** — in a title block,
    /// in a note, however far away. Dropping the pin does not merely risk the
    /// wrong occurrence; on a BOM sheet it can make the right one impossible to
    /// reach. The engine's ruling: *"if you have a pin, use `spanning_from`. Do
    /// not drop the pin as a fallback; it is not a weaker version of the same
    /// thing."*
    ///
    /// # Why the field is kept wired
    ///
    /// A `None` that is always `None` is cheap, and [`super::glyphwall`] holds
    /// a fixture with two identical split runs on one page asserting that the
    /// edit **lands**. If a future engine withdraws `span_from_pin`, this field
    /// and the refusal it fed are the route that has to come back — and it must
    /// come back with the correction above, never as the count alone.
    pub occurrences: Option<usize>,
}

/// **Plan a commit against the page as it is now.**
///
/// Called from the apply arm rather than from the canvas, because it needs the
/// document and an `Action` is plain data. It is still one function in one place
/// — the arm routes to it and computes nothing itself.
///
/// The three things it derives, all from `(page_text, run)`:
///
/// 1. **the provenance pin** — `operator_span`, which is how the surgery finds
///    *this* show operator rather than the first one whose text matches. Without
///    it, editing the second `TITLE` on a title-block sheet edits the first.
/// 2. **the matrices** — `Tm` and the CTM in force at the run's first glyph,
///    which is what [`disposition::is_upright`] reads.
/// 3. **the block alignment** — through `ReflowEngine::detect_alignment` on a
///    model recognised with [`reflow_recognition_options`], i.e. the **relaxed**
///    recogniser. That is the old shell's own choice for its reflow target and
///    the reason carries here unchanged: the default recogniser splits on
///    indentation, so a right-aligned block whose lines start at different x —
///    which is what right alignment *is* — is exactly the shape it fragments,
///    and a fragmented block is a one-line block, and a one-line block reports
///    `SingleLineDefault`. Using the default model would make the alignment
///    fix unreachable on precisely the documents it is for.
#[must_use]
pub fn plan(doc: &OpenDoc, page: usize, run: usize, original: &str, replacement: &str) -> Plan {
    let mut request = EditRequest::find_replace(page, original, replacement);
    let mut matrices = (
        [1.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0],
        [1.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0],
    );
    let mut finding = None;
    // ★★ Whether the caret's visual line is made of more than one show
    // operator, re-derived here rather than carried on the `Anchor`.
    //
    // The `Anchor` docs give the rule and it applies unchanged: everything but
    // the original text is a pure function of `(page_text, run)`, and a copy
    // taken when the operator clicked would go stale when the page is rebuilt.
    //
    // Defaults to `false`, which is the *permissive* direction — `Reflow` — and
    // that is the honest default for the same reason the identity matrices below
    // are: it is what a page whose provenance could not be read gets, and a
    // shell that pinned on no evidence would be claiming to have measured
    // something it never saw. The single-run case is also the overwhelmingly
    // commoner one in ordinary prose documents.
    let mut shares_the_line = false;
    // ★★★ **Defaults to `true`, and the default is a claim about knowledge
    // rather than about the run** — see [`Plan::one_operator`]. If the
    // extraction fails, or the run carries no provenance, this shell has
    // measured nothing; answering `false` there would let the apply arm tell
    // the operator his line is written one letter at a time on the strength of
    // an extraction that never ran.
    let mut one_operator = true;
    // ★ Always `None` — see [`Plan::occurrences`], which carries the whole
    // argument. Nothing in this function counts.
    let occurrences = None;

    // ★★ **This extraction is its own, and it is NOT `doc.page_text()`.**
    //
    // `app::cache`'s extraction runs with `ExtractOptions::default()`, and the
    // engine's contract is that provenance is `None` unless the extraction set
    // `ExtractOptions::capture_provenance` — which **defaults to off**, so that
    // the default output stays byte-for-byte unchanged. Reading the shared
    // cache here would therefore make `model.provenance(..)` answer `None` for
    // every glyph, and this function would:
    //
    // * leave `pinned_span` at `None`, so the surgery locates the **first**
    //   operator whose text matches rather than the one the caret is in — which
    //   on a title-block sheet with two runs reading `REV A` edits the wrong
    //   one; and
    // * fall back to the identity matrices below, so **the rotation guard would
    //   never fire** and D4b case 2 would be unfixed while every unit test in
    //   `disposition` stayed green — a correct decision function wired to a
    //   value that is always the same.
    //
    // Widening the shared cache was the other option and is the worse one: every
    // caller of `page_text()` — Find, both copy verbs, the text sweep — would
    // then pay for provenance on every page, and `app::cache`'s own header
    // records that extraction is the expensive thing this shell does (392 ms on
    // the benchmark sheet). Paying it **once per commit**, here, is the whole
    // cost, and a commit is already an operation that saves and re-rasters.
    //
    // The run index is shared between the two extractions, which is safe and is
    // worth stating: `capture_provenance` populates a field and changes no
    // segmentation, so `runs[i]` names the same run under both options.
    {
        // ★ The funnel's output, MODIFIED — not a second construction.
        //
        // `with_provenance(true)` is the one thing no setting governs: it is the
        // substrate for editing text, and `app::cache`'s read-only extraction
        // deliberately leaves it off because it costs and it is not needed
        // there. Everything else — the word gap, the unmappable sentinel, the
        // replacement-text precedence — comes from the operator, so the runs
        // this editor addresses are segmented exactly as the runs the canvas
        // paints and the find bar searches. Two extractions of one page under
        // two configurations would put the glyph the operator clicked and the
        // glyph this code edits one step out of step.
        //
        // ⇒ All of which is now the shared cache's contract rather than this
        // function's — `crate::app::cache::provenance` — and on the commit
        // path that matters twice over: the operator has already clicked in
        // this run, so the extraction this needs was paid for at click time
        // and this reads it for free.
        if let Some(text) = doc.provenance_page_text(page) {
            let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
            // ★★ The pin, and the buffer it indexes — [`pin::of_run`].
            //
            // Both facts, from one call, over the model just recognised. The
            // measurement lives in [`pin`] rather than inline because
            // `format_text` needs exactly the same one, and the argument behind
            // the `EditTarget` choice is long enough that two paraphrases of it
            // would drift apart.
            if let Some(p) = pin::of_run(&model, run) {
                request.pinned_span = Some(p.span);
                matrices = (p.text_matrix, p.ctm);
                request.target = p.target;
                // ★★★ **The find string is DROPPED when the pin is exact.**
                //
                // `EditRequest::whole_operator`: an empty `find` beside a pin
                // means *"this whole show operator"*, which is precisely what a
                // caret in a run means and what a rebuilt `find` can only ever
                // approximate.
                //
                // ## Why the approximation is not good enough
                //
                // A run's `text` is not in 1:1 correspondence with its glyphs —
                // `/ToUnicode` may map one glyph to several characters — so a
                // reconstructed find fails invisibly on unligatured text and
                // routinely on real typeset copy. On CAD drawings it is worse:
                // `text_extract` synthesises inter-glyph spacing (one measured
                // title-block cell carried **twenty-one** of them), so the
                // string this shell holds contains characters no show operator
                // ever wrote and the match can never succeed.
                //
                // ## ★★ And why only when the run is one operator
                //
                // See [`pin::spans_one_operator`]. On a split run the whole
                // -operator form would replace one fragment's text with the
                // whole replacement and leave the other fragments painting their
                // old glyphs — visible corruption reported as success. The
                // find-based form fails cleanly there instead, which is the
                // right outcome for a case this shell cannot yet edit at all.
                // ★★★ THE DECISION IS TRACED, because without it the two
                // outcomes are indistinguishable from outside the process and
                // one of them is correct.
                //
                // A refusal reading `edit-text-refused … detail=text to edit
                // (…) was not found in an editable run` means a program defect
                // if the pin path was taken, and *"this run spans operators, the
                // find-based form was used deliberately, and it failed cleanly
                // as designed"* if it was not. Those need different responses —
                // one is a request to the engine, the other is nothing — and a
                // trace that cannot separate them turns a correct build into a
                // filed defect. `find_len` carries the number that made the
                // string unmatchable, because a reader seeing 30 characters for
                // a six-character cell has the whole story in one line.
                one_operator = pin::spans_one_operator(&model, run);
                // ★★★ **O142 — a typo inside a run written one glyph at a
                // time**, and the request shape that reaches it.
                //
                // A plain pin confines the match to the one operator it names,
                // which on a per-glyph CAD run holds a single character — so a
                // 36-character `find` cannot fit and the engine answers
                // `NotFound`, correctly, to the question it was asked.
                // `span_from_pin` lets the match **begin** at the pinned
                // operator and continue across the following ones, and the pin
                // therefore stays on: the shell never has to choose between
                // *addressing this occurrence* and *reaching a split run*. See
                // [`Plan::occurrences`] for why taking the pin off is not the
                // weaker version of that, but a different and unsafe thing.
                //
                // ★★ Measured on the operator's own file, one `EditSession`
                // per shape, page 2, the run at doc-point `1,200.4,537.1`:
                //
                // | request | result |
                // |---|---|
                // | whole-run `find` + plain pin | `NotFound` |
                // | whole-run `find`, no pin | **OK**, `operators_spanned=36` |
                // | `"clien"` + plain pin | `NotFound` |
                // | `"clien"`, no pin | **OK**, `operators_spanned=5` |
                //
                // ★★★ The second row is what settles the choice of `find`: the
                // whole-run string — synthesised spaces and all — matches
                // perfectly once the match is allowed to span, 36 characters
                // over 36 operators. The synthesised-space case documented
                // above is real, but it belongs to the CAD drawings and is not
                // what blocks this typo. A narrower `find` is strictly worse:
                // the changed span alone (`"n"` → `"nt"`) occurs **33 times**
                // on that page, where the whole run occurs once.
                //
                // ★ Spanning puts the replacement into the operator holding the
                // match's end and empties the ones before it (each kept as `() Tj`);
                // the engine keeps the line where the match began, and narrows the
                // rewrite to the part that differs by itself.
                //
                // ⚠ **Do NOT reintroduce "drop the pin when the text is
                // unique" as a fallback**; see [`Plan::occurrences`].
                if !one_operator {
                    request.span_from_pin = true;
                }
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // ★ `span_from_pin` is the field that says how the request
                    // is addressed. It is spelled as `0`/`1` rather than
                    // `{:?}` on a `bool` for the reason that bans `{:?}` from
                    // every parsed field in this tree: a `Debug` spelling is a
                    // Rust rendering of a domain value, and a parser reading it
                    // is reading the language rather than the program.
                    format!(
                        "edit-text-pin page={page} run={run} one_operator={one_operator} \
                         find_len={} span_from_pin={} pinned={}",
                        request.find.chars().count(),
                        u8::from(request.span_from_pin),
                        request.pinned_span.is_some()
                    )
                });
                if one_operator {
                    request.find.clear();
                }
            }
            // ★ The SAME model the caret's hit test used, with the same
            // options — `BlockRecognitionOptions::default()` — because the
            // question is *how did the thing the operator clicked get
            // segmented*, and asking it of a differently-recognised model would
            // answer about a different segmentation. The relaxed model below is
            // for alignment detection, which is a different question about the
            // same page.
            if let Some((from, to)) = model.line_range_at(TextPosition::new(run, 0)) {
                shares_the_line = from.run != to.run;
            }
            let relaxed = EditableTextModel::recognize(&text, &reflow_recognition_options());
            finding = relaxed
                .block_at(TextPosition::new(run, 0))
                .and_then(|b| ReflowEngine::new(&relaxed).detect_alignment(b).ok())
                .map(disposition::from_detection);
        }
    }

    let reason = disposition::choose(matrices.0, matrices.1, shares_the_line, finding);
    // ★★★ **The words the operator typed, kept where a refusal can find them**
    // — `OPERATOR_REQUESTS.md` O141. See [`LAST_COMMIT`].
    LAST_COMMIT.with_borrow_mut(|slot| {
        *slot = Some(Committing {
            page,
            run,
            original: original.to_owned(),
            replacement: replacement.to_owned(),
        });
    });
    Plan {
        request,
        options: disposition::options(reason),
        reason,
        one_operator,
        occurrences,
    }
}
