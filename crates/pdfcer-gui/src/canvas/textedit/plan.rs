//! # `canvas::textedit::plan` — turning a caret and two strings into an
//! `EditRequest` the engine can answer
//!
//! **One function, called from one place**, and everything a text commit
//! decides is decided here: which show operator to address, whether to name it
//! by provenance pin or by its text, how the rest of the line is allowed to
//! move, and — since `OPERATOR_REQUESTS.md` **O142** — whether this shell is
//! entitled to let the engine choose an occurrence at all.
//!
//! ## Why this is its own file
//!
//! It was the bottom half of [`super`] until 2026-09-06, under a section banner
//! reading *"Planning the commit — where D4b's two fixes actually take effect"*.
//! O142's occurrence guard pushed that file to 1,512 lines, twelve over R2, and
//! the banner was already naming the seam: everything above it is about a caret
//! and a draft — where the operator clicked, what he typed, when a draft opens
//! and closes — and everything below it is about **one request to
//! `pdfcer-core`**. Two subjects, one boundary, already drawn.
//!
//! ★ R2's own text says to split along the seams rather than raise the limit,
//! and this is what that looks like when the seam is honest: no call site
//! changed, because [`plan`] and [`Plan`] are re-exported from [`super`] where
//! every caller already looks for them.
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
//! scan order. [`Plan::occurrences`] and [`page_occurrences`] carry the whole
//! argument for when that is permitted, and `super::glyphwall` holds it as
//! tests over two fixtures — one where the edit must land, one where it must be
//! refused.

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
    /// | run | what `EditError::NoMatch` means |
    /// |---|---|
    /// | one operator | the page moved under the caret — *"pdfcer could not find the text this edit named"* |
    /// | several | **the reconstructed `find` could never have matched**, because no single operator holds it |
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

    /// ★★★ **How many times the text being edited appears on the page** — and
    /// therefore whether the pin could safely be dropped so that the engine's
    /// cross-operator matcher could reach it. `OPERATOR_REQUESTS.md` **O142**,
    /// the operator's own report of 2026-09-05:
    ///
    /// > *"on page 2 there is a spelling mistake — clien instead of client. if I
    /// > try to edit the edit is not accepted."*
    ///
    /// `None` when the pin was exact — [`Self::one_operator`] is `true`, the
    /// `find` was dropped, and the request names an operator rather than a
    /// string, so there is nothing to be ambiguous about. `Some(n)` when the run
    /// spans operators and the `find` is therefore the only route: `n` is how
    /// many times that string occurs in the page's text.
    ///
    /// # ★★★ Why a count decides whether the PIN is sent
    ///
    /// `Pass 256.0` taught `edit_text` to match a `find` across consecutive show
    /// operators of one text object — which is exactly the shape of the
    /// operator's file, where the producer writes one glyph per operator. Its
    /// contract carries one clause that governs everything here:
    ///
    /// > *"A pinned request never spans."*
    ///
    /// So a request carrying **both** a `find` and a `pinned_span` is confined
    /// to the single operator the pin names, and on his line that operator holds
    /// one character. A 36-character `find` cannot match inside it, and the
    /// engine answers `NotFound` — which is the refusal he actually saw.
    ///
    /// ⇒ To reach his typo the pin must come **off**. But the pin is what makes
    /// an edit address *this* occurrence rather than the first one that matches,
    /// and without it `EditRequest` has no way to choose: it carries no
    /// occurrence index, and `pinned_span` is its only disambiguator. Dropping
    /// the pin on a page where the text occurs twice would edit whichever the
    /// engine reached first — **on a signed quotation, silently.**
    ///
    /// So the pin comes off only when this count is exactly 1, and the count is
    /// what licenses it.
    ///
    /// # ★★ The count is a SAFE proxy, and the direction of its error is the
    /// whole argument
    ///
    /// It is taken over the page's extracted text ([`page_occurrences`]), while
    /// the engine matches over decoded **operator** text. The two can differ,
    /// and every way they differ pushes this count **up**:
    ///
    /// * extraction may synthesise inter-glyph spacing that no operator wrote
    ///   (on this operator's CAD drawings a title-block cell showed twenty-one
    ///   such spaces), and `/ToUnicode` may map one glyph to several characters
    ///   — both make the extracted string a **superset** of what any operator
    ///   holds;
    /// * the count is taken over the runs concatenated, so it also sees matches
    ///   straddling a run boundary that no single text object could hold.
    ///
    /// ⇒ Every location the engine could match is a location this count already
    /// counted, so `n == 1` means the engine has **at most one** candidate.
    /// The proxy can only ever refuse an edit that would have been safe; it
    /// cannot license one that is not. That asymmetry is the property, not an
    /// accident of the implementation, and [`page_occurrences`] states it again
    /// at the point where a future change could break it.
    ///
    /// ★ And where the strings genuinely disagree — the CAD case, where the
    /// `find` carries synthesised spaces — the engine answers `NotFound` and the
    /// edit is refused cleanly. It is never applied somewhere else.
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
    // ★★ `None` until the plan finds it needs the `find` at all — see
    // [`Plan::occurrences`]. It stays `None` on the exact-pin path and on every
    // path where the extraction did not run, which is the honest spelling of
    // *"this shell did not have to choose an occurrence"* as distinct from
    // *"it chose and found one"*.
    let mut occurrences = None;

    // ★★ **This extraction is its own, and it is NOT `doc.page_text()`.**
    //
    // That was the first shape of this function and it was silently broken.
    // `app::cache`'s extraction runs with `ExtractOptions::default()`, and
    // `capture_provenance` **defaults to off** — the engine says so in terms:
    // *"`None` unless the extraction set `ExtractOptions::capture_provenance`;
    // this keeps the default Pass 4 output byte-for-byte unchanged."* With it
    // off, `model.provenance(..)` answers `None` for every glyph, and this
    // function would have:
    //
    // * left `pinned_span` at `None`, so the surgery would locate the **first**
    //   operator whose text matches rather than the one the caret is in — which
    //   on a title-block sheet with two runs reading `REV A` edits the wrong
    //   one; and
    // * fallen back to the identity matrices below, so **the rotation guard
    //   would never fire** and D4b case 2 would be unfixed while every unit test
    //   in `disposition` stayed green. That is precisely `HANDOFF.md` §2's
    //   shape: a correct decision function, wired to a value that is always the
    //   same.
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
    if let Some(page_ref) = doc.pages.get(page) {
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
        use crate::app::settings::SettingsExt;
        let opts = doc.settings.extract_options().with_provenance(true);
        if let Ok(text) =
            pdfcer_core::text_extract::extract_page_view(&doc.session.view(), page_ref, page, &opts)
        {
            let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
            // ★★ The pin, and the buffer it indexes — [`pin::of_run`].
            //
            // Both facts, from one call, over the model just recognised. They
            // lived here inline until 2026-08-27; `format_text` became the
            // second verb that needs exactly the same measurement, and the
            // sixty lines of argument behind the `EditTarget` choice are what
            // makes it right, so they moved somewhere both callers reach them
            // rather than being paraphrased twice.
            if let Some(p) = pin::of_run(&model, run) {
                request.pinned_span = Some(p.span);
                matrices = (p.text_matrix, p.ctm);
                request.target = p.target;
                // ★★★ **The find string is DROPPED when the pin is exact.**
                //
                // `EditRequest::whole_operator` (`Pass 152.0`): an empty `find`
                // beside a pin means *"this whole show operator"*, which is
                // precisely what a caret in a run means and what a rebuilt
                // `find` was only ever an approximation of.
                //
                // ## Why the approximation had to go
                //
                // A run's `text` is not in 1:1 correspondence with its glyphs —
                // `/ToUnicode` may map one glyph to several characters — so the
                // reconstructed find *"fails invisibly on unligatured test text
                // and routinely on real typeset copy"*, in the engine's words.
                // On this operator's own CAD drawings it is worse than that:
                // `text_extract` synthesises inter-glyph spacing (a trace of one
                // of his title-block cells showed **twenty-one** spaces), so the
                // string this shell holds contains characters no show operator
                // ever wrote and the match can never succeed.
                //
                // ⇒ That was reported as *"text editing is weird"*, filed as a
                // defect against the engine, and answered by naming a capability
                // that already existed. The workaround is deleted rather than
                // kept beside the fix.
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
                // A driven run on the operator's own drawing produced
                // `edit-text-refused … detail=text to edit ("0.00[21 spaces]0.030")
                // was not found in an editable run`, and **nothing anywhere said
                // whether the pin path had been taken.** So the check reported a
                // program defect and aimed the reader at `pdfcer-core`, when the
                // honest reading might have been *"this run spans two operators,
                // the find-based form was used deliberately, and it failed
                // cleanly as designed"*.
                //
                // ⇒ Those two need different responses — one is a request to the
                // engine, the other is the shell working — and a trace that
                // cannot separate them turns a correct build into a filed
                // defect. `find_len` carries the number that made the string
                // unmatchable, because a reader seeing 30 characters for a
                // six-character cell has the whole story in one line.
                one_operator = pin::spans_one_operator(&model, run);
                // ★★★ **O142 — HIS TYPO, and the one line that was stopping it.**
                //
                // Until 2026-09-06 this block did exactly two things: trace, and
                // clear the `find` when the pin was exact. On a split run it
                // left BOTH the `find` and the `pinned_span` set, and that
                // combination is unmatchable by construction — `Pass 256.0`'s
                // contract says *"a pinned request never spans"*, so the engine
                // looks for a 36-character string inside the one operator the
                // pin names, which on his file holds a single character.
                //
                // He reported it as *"if I try to edit the edit is not
                // accepted"*, and the refusal he got —
                // `text to edit ("Final quality walkthrough with clien") was not
                // found in an editable run on the page` — was the engine
                // answering the question it had been asked, correctly.
                //
                // ⇒ The remedy is to take the pin OFF, which is what lets the
                // cross-operator matcher run. The count is what makes that safe;
                // see [`Plan::occurrences`] for why the pin cannot simply be
                // dropped unconditionally, and [`page_occurrences`] for why a
                // count over extracted text is a conservative stand-in for a
                // count over operator text.
                //
                // ★★ Measured on his own file, one `EditSession` per shape,
                // page 2, the run at doc-point `1,200.4,537.1`:
                //
                // | request | result |
                // |---|---|
                // | whole-run `find` + pin — what this shell sent | `NotFound` |
                // | whole-run `find`, no pin | **OK**, `operators_spanned=36` |
                // | `"clien"` + pin | `NotFound` |
                // | `"clien"`, no pin | **OK**, `operators_spanned=5` |
                //
                // ★★★ Note what the second row falsifies: the whole-run `find`
                // — spaces and all — matches perfectly once the pin is off. The
                // standing diagnosis had been that extraction synthesises the
                // spaces in that string and no operator holds them, so no
                // widening of the matcher could ever reach it. **On this file
                // that is not true**: 36 characters, 36 operators, spaces
                // included. The synthesised-space case is real and is documented
                // above, but it belongs to his CAD drawings and was not what
                // blocked his typo. The narrower `find` was therefore never
                // needed, and it would have been strictly worse — the changed
                // span alone (`"n"` → `"nt"`) occurs **33 times** on that page,
                // where the whole run occurs once.
                //
                // ## ★★★ The one worry about the whole-run form, MEASURED
                //
                // Spanning thirty-six operators means the engine puts the
                // replacement into the operator holding the match's **end** and
                // empties the thirty-five before it (each kept as `() Tj` so the
                // producer's positioning chain survives). Its disclosure reports
                // the tail re-spaced by a net advance of **-437.080 pt**, which
                // reads alarmingly like the line being flung across the page.
                //
                // ⇒ **It is not.** That number is bookkeeping over the collapsed
                // operators, not a visual displacement, and the rendered result
                // was measured rather than reasoned about — extracted run boxes
                // on his own file, before and after:
                //
                // | | llx | urx |
                // |---|---|---|
                // | before | 33.4709 | 367.2634 |
                // | after, whole-run `find` | 33.4289 | 373.2873 |
                // | after, narrow `find` | 33.4709 | 373.2874 |
                //
                // The line's left edge moves by **0.042 pt** — about fifteen
                // microns, and under a thousandth of the line's own width — while
                // the right edge grows by 6.02 pt, which is the `t` he asked for.
                //
                // ★★ The narrow form is exact to the last decimal and the
                // whole-run form is not, and that difference is **deliberately
                // not** the deciding one: 0.042 pt is invisible, whereas the
                // narrow form's ambiguity is a wrong edit on a signed document.
                // `glyphwall::a_typo_in_a_run_written_one_glyph_at_a_time_can_be
                // _corrected` holds the bound so the day it stops being invisible
                // is a test failure and not a report from him.
                if !one_operator {
                    let n = page_occurrences(&text, original);
                    occurrences = Some(n);
                    if n == 1 {
                        request.pinned_span = None;
                    }
                }
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // ★ `occurrences` is spelled as a bare number, or `none`
                    // when the pin was exact — never `{:?}` on the `Option`.
                    // `{:?}` on a domain value in a field a check parses is
                    // banned in this tree and has already produced two false
                    // failure reports; `Some(1)` would also put a bracket into
                    // a `key=value` line.
                    let occurrences =
                        occurrences.map_or_else(|| "none".to_owned(), |n| n.to_string());
                    format!(
                        "edit-text-pin page={page} run={run} one_operator={one_operator} \
                         find_len={} occurrences={occurrences} pinned={}",
                        request.find.chars().count(),
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
    // — `OPERATOR_REQUESTS.md` O141, 2026-09-05. See [`LAST_COMMIT`].
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

/// **How many times `needle` occurs in this page's extracted text**, for
/// [`Plan::occurrences`] — the count that decides whether the provenance pin may
/// be dropped so the engine's cross-operator matcher can reach a run the
/// producer wrote one glyph at a time.
///
/// # ★★★ THE ONE PROPERTY THIS FUNCTION MUST KEEP: IT MAY OVER-COUNT, NEVER
/// UNDER-COUNT
///
/// Its answer is used for exactly one decision — *"is it safe to send this edit
/// without a pin?"* — and the two errors it could make are not symmetric:
///
/// * **over-count** → the shell refuses an edit the engine would have applied
///   correctly. The operator is told pdfcer cannot tell which occurrence he
///   meant. Annoying, honest, and reversible.
/// * **under-count** → the shell drops the pin on a page holding two matches and
///   the engine edits whichever it reaches first. **On a signed quotation that
///   is a silent wrong edit**, and it is the defect that ends trust in the
///   program.
///
/// So the implementation is deliberately the loosest one that still answers the
/// question, and the two ways it is loose are both in the safe direction:
///
/// 1. **It counts over EXTRACTED text, while the engine matches over decoded
///    operator text.** Extraction can only add characters relative to what an
///    operator holds — it synthesises inter-glyph spacing, and `/ToUnicode` can
///    map one glyph to several characters. So every substring an operator could
///    match is a substring of the extraction, and every location the engine
///    could match is a location counted here. `n == 1` therefore means the
///    engine has **at most one** candidate.
/// 2. **It counts over the runs concatenated**, so a match straddling a run
///    boundary — which no single text object could hold, and the engine could
///    never make — is still counted. That inflates the count and refuses; it
///    cannot license anything.
///
/// ⚠ **Do not "tighten" this by counting per run, or by reconstructing operator
/// text.** Both would move the error into the under-counting direction, which is
/// the one that writes to his document. If it ever needs to be more precise, the
/// precision must come from the engine gaining an occurrence selector on
/// `EditRequest` — filed rather than approximated.
///
/// ★ Matches are counted **non-overlapping**, which is what `str::matches` does
/// and what the engine's own left-to-right scan does. For a `needle` that can
/// overlap itself the two agree on which locations exist even where they
/// disagree on how many; and since the only value that matters here is *"exactly
/// one or more than one"*, an overlapping pair counts as at least two either
/// way and refuses.
///
/// ★ An empty `needle` answers `0`, which routes to the ambiguous arm rather
/// than to the permissive one. `str::matches("")` yields a match at every
/// boundary, so returning its length would be a large number that happens to
/// refuse for the wrong reason; `0` refuses for the right one. The case is not
/// reachable from [`plan`] — a run with no text has no caret in it — and is
/// handled here so that it cannot become reachable silently.
#[must_use]
fn page_occurrences(text: &pdfcer_core::text_extract::PageText, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let whole: String = text.runs.iter().map(|r| r.text.as_str()).collect();
    whole.matches(needle).count()
}
