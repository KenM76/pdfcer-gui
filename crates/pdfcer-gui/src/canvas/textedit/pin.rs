//! # `canvas::textedit::pin` — naming the exact show operator, and the exact
//! buffer it lives in
//!
//! ## What a pin is, and why nothing that edits text may go without one
//!
//! `pdfcer-core`'s text verbs — [`EditSession::edit_text`] and
//! [`EditSession::format_text`] — locate their operand two ways. Given only a
//! search string they find the **first** show operator on the page whose
//! decoded text matches. Given a `pinned_span` they find the one whose byte
//! span in the decoded content buffer is exactly that.
//!
//! The difference is not an optimisation. On a title-block sheet with two runs
//! reading `REV A`, the unpinned form edits the wrong one, silently, with no
//! error anywhere. This operator's own benchmark drawing carries **3,007
//! single-character show operators** in one page stream, so "the first match is
//! the one the operator meant" is false on the documents this program exists
//! for.
//!
//! ## Why this is its own module rather than a private detail of the caret
//!
//! It was a private detail of the caret until 2026-08-27, inline in
//! [`super::plan`], and that was correct while exactly one thing edited text.
//! `format_text` is the second: restyling an existing run takes **the same
//! `pinned_span` and the same `EditTarget`** as replacing its text — the engine
//! shaped the two verbs that way deliberately, *"so a shell that has decided
//! which stream a caret is in does not have to translate that decision between
//! two verbs."*
//!
//! A second copy of that decision is the thing to avoid. The `EditTarget` arm
//! below is nine lines of code and sixty of argument, and the argument is what
//! makes it right; a paraphrase of it beside the restyle verb would compile,
//! would look correct, and would drift.
//!
//! ## ★★ The extraction here is NOT the shared page-text cache
//!
//! `crate::app::cache`'s extraction runs with `ExtractOptions::default()`, and
//! `capture_provenance` **defaults to off** — the engine's own words:
//! *"`None` unless the extraction set `ExtractOptions::capture_provenance`;
//! this keeps the default Pass 4 output byte-for-byte unchanged."*
//!
//! With it off, `provenance()` answers `None` for every glyph, and a caller
//! built on the shared cache would get **no pin at all** while every line of it
//! kept compiling. That is this project's canonical failure shape: a correct
//! decision function wired to a value that is always the same.
//!
//! Widening the shared cache is the other option and is worse. Every consumer
//! of `page_text()` — Find, both copy verbs, the text sweep — would then pay
//! for provenance on every page, and extraction is the expensive thing this
//! shell does (392 ms on the benchmark sheet). Paying it **once per edit**, in
//! [`resolve`], is the whole cost, and an edit is already an operation that
//! saves and re-rasters.
//!
//! ★ The run index is shared between the two extractions, which is safe and is
//! worth stating: `capture_provenance` populates a field and changes no
//! segmentation, so `runs[i]` names the same run under both options.

use pdfcer_core::span::ByteSpan;
use pdfcer_core::text_edit::{BlockRecognitionOptions, EditTarget, EditableTextModel, GlyphRef};
use pdfcer_core::text_extract::TextColor;

use crate::app::state::OpenDoc;

/// Which content buffer a glyph's span indexes — the `EditTarget` half of a pin.
///
/// Its own function since 2026-08-27, when a second caller appeared
/// ([`operators_in_run`]). The sixty lines of argument below are the reason it
/// is a function rather than two copies: a paraphrase of them beside the
/// restyle verb would compile, would look correct, and would drift.
fn target_of(p: &pdfcer_core::text_extract::GlyphProvenance) -> pdfcer_core::text_edit::EditTarget {
    match p.content_stream {
        pdfcer_core::text_extract::ContentStreamRef::Page => {
            pdfcer_core::text_edit::EditTarget::PageContents
        }
        pdfcer_core::text_extract::ContentStreamRef::Form { object } => {
            pdfcer_core::text_edit::EditTarget::Form { object }
        }
        // ★ `ContentStreamRef` is `#[non_exhaustive]`, so a buffer
        // kind added later lands here. `Auto` is the right fallback
        // and not merely the compiling one: it is the engine's own
        // default, it searches everywhere including whatever the new
        // kind is, and it degrades to the pre-`119.0` behaviour
        // rather than to a refusal. A `PageContents` fallback would
        // silently narrow the search for a stream nobody here has
        // heard of, which is the worse direction.
        _ => pdfcer_core::text_edit::EditTarget::Auto,
    }
}

/// Everything a pinned text verb needs in order to name one show operator.
///
/// The three fields travel together because they are **one measurement**. The
/// span alone is the defect this shell shipped first: it pinned the offset and
/// discarded the stream, and the engine correctly reported *"text not found"*
/// about text that was plainly there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pinned {
    /// The show operator's byte span within its own decoded content buffer.
    pub span: ByteSpan,
    /// **Which buffer that span indexes.** Never `Auto` when the provenance
    /// was read; see the argument on [`resolve`].
    pub target: EditTarget,
    /// `Tm` in force at the run's first glyph — read by
    /// [`super::disposition::is_upright`].
    pub text_matrix: [f32; 6],
    /// The CTM in force at the run's first glyph.
    pub ctm: [f32; 6],
}

/// The pin for `run`, from a model **already recognised over a
/// provenance-carrying extraction**.
///
/// `None` when the extraction did not capture provenance, or when the run has
/// no glyphs. Both mean the same thing to a caller — *this run cannot be
/// pinned* — and both must be treated as a refusal rather than as permission to
/// fall back to an unpinned request, for the reason the module header gives.
#[must_use]
pub fn of_run(model: &EditableTextModel<'_>, run: usize) -> Option<Pinned> {
    let p = model.provenance(GlyphRef::new(run, 0))?;
    // ★★★ NAME THE BUFFER THE PIN INDEXES. `Pass 119.0`, and this
    // is the line that makes form editing SAFE rather than merely
    // possible.
    //
    // `EditTarget::Auto` is the engine's default and is right for a
    // caller that has only a search string: it tries the page's own
    // `/Contents` first, then each form in `Do` order, and edits the
    // first stream that matches.
    //
    // **It is the wrong default for a PINNED request.** A pin is a
    // byte span into ONE decoded buffer, and `GlyphProvenance`
    // carries the name of that buffer beside it — the two fields are
    // one fact, and reading half of it is the defect this shell
    // shipped in the first place (the span was pinned, the stream
    // was discarded, and the engine reported "text not found" about
    // text that was plainly there).
    //
    // Under `Auto`, a span that indexes a form's bytes is offered to
    // the page's stream first. On this operator's own benchmark
    // sheet that stream holds **3,007 single-character show
    // operators**, so "an arbitrary offset happens to name a
    // matching operator in the wrong buffer" is not a theoretical
    // collision — it is a dense field of near-misses, and the result
    // would be an edit that succeeded on the wrong glyph with no
    // error anywhere.
    //
    // So: the shell knows exactly which stream it measured, and it
    // says so. `Form { object }` is an error if the page does not
    // paint that form, which is the answer we want — a loud refusal
    // beats a widened search when the caller had a measurement.
    let target = target_of(p);
    Some(Pinned {
        span: p.operator_span,
        target,
        text_matrix: p.text_matrix,
        ctm: p.ctm,
    })
}

/// **Do all of this run's glyphs come from ONE show operator?**
///
/// ## ★★★ Why this question is worth its own function
///
/// Because the answer decides whether the run can be addressed as a *whole
/// operator* — `EditRequest::whole_operator`, `Pass 152.0` — and getting it
/// wrong is the difference between an edit that refuses and an edit that
/// duplicates text on the page.
///
/// [`of_run`] pins on **glyph 0's** operator. The engine's own measurement is
/// that **13% of runs over its corpus carry glyphs from more than one show
/// operator**, and on those runs:
///
/// | request | what happens |
/// |---|---|
/// | `find` = the run's text, pinned | `NoMatch` — the pinned operator holds only part of it |
/// | whole-operator (empty find), pinned | the pinned operator's text is replaced **with the whole replacement**, and the run's other operators keep their glyphs |
///
/// ⇒ The second is worse. A refusal costs an operator a puzzled moment; the
/// other writes `Rev BEV A` onto a drawing and reports success. So the
/// whole-operator form is taken **only** when this answers `true`, and the
/// find-based form — with its clean refusal — is what a split run keeps.
///
/// ★ It walks the glyphs rather than trusting a count, because
/// [`EditableTextModel::provenance`] answers `None` one past the end and that
/// is the same termination a length would give with one fewer thing to keep in
/// step.
///
/// ★★ `false` when there is no provenance at all. A caller with no pin is not
/// entitled to the whole-operator form in the first place — the engine refuses
/// an empty `find` without a pin, by name — and answering `true` here would
/// build a request it would then reject.
#[must_use]
pub fn spans_one_operator(model: &EditableTextModel<'_>, run: usize) -> bool {
    let Some(first) = model.provenance(GlyphRef::new(run, 0)) else {
        return false;
    };
    let mut glyph = 1;
    while let Some(p) = model.provenance(GlyphRef::new(run, glyph)) {
        if p.operator_span != first.operator_span {
            return false;
        }
        glyph += 1;
    }
    true
}

/// The pin for `run` on `page`, extracting the page with provenance on.
///
/// The convenience form for a caller that does not already hold a model — the
/// restyle verbs, which start from a selection rather than from a caret. A
/// caller that has just recognised a model should use [`of_run`] and not pay
/// for a second extraction.
///
/// `None` when the page is absent, when the extraction fails, or when
/// [`of_run`] answers `None`.
#[must_use]
pub fn resolve(doc: &OpenDoc, page: usize, run: usize) -> Option<Pinned> {
    inspect(doc, page, run).map(|i| i.pin)
}

/// What a run currently **looks like** — the three facts a properties panel
/// shows and a restyle changes.
///
/// # ★ Why this is separate from [`Pinned`] and returned beside it
///
/// [`Pinned`] is a *locator*: it names an operand, and every field on it is
/// consumed by the engine. This is a *reading*: every field on it is consumed
/// by a human. Merging them would mean the restyle verb carrying three fields
/// it never looks at, and — the part that matters — would make it possible to
/// pass a stale reading into an edit by passing the struct that also carries
/// the pin.
///
/// They come back from one call because they come from one `GlyphProvenance`,
/// and the extraction that produces it costs 392 ms on this operator's
/// benchmark sheet. Two calls would be two extractions for one question.
#[derive(Debug, Clone, PartialEq)]
pub struct RunStyle {
    /// The `Tf` size in points.
    pub size: f32,
    /// The `/Resources /Font` **key** in force — `F1`, not `Helvetica`.
    ///
    /// ★ Not the `/BaseFont`, and the difference is why a caller showing this
    /// to an operator has to join it against the document's font inventory
    /// first. `GlyphProvenance` records what the content stream said, which is
    /// a resource key; the human-readable name lives in the font dictionary the
    /// key resolves to.
    pub font_resource: Option<String>,
    /// ★★ **The run's own characters** — and they are not decoration.
    ///
    /// `format_text` needs a non-empty `find` **even on a pinned request**, and
    /// that surprised this shell: the pin names the show OPERATOR, and `find`
    /// then names a contiguous sub-range *within* it (`match_run`, which
    /// refuses an empty one by name). Restyling a whole run therefore means
    /// passing the whole run's text.
    ///
    /// Published here rather than re-read by the caller because the caller
    /// would have to re-extract to get it, and this function has just paid for
    /// an extraction.
    ///
    /// ★ It stays valid across a restyle: `format_text` changes how characters
    /// look and never which characters they are, so a text captured before a
    /// multi-run gesture is still the right `find` for the runs still to come.
    ///
    /// ★★★ **It is NOT the show operator's decoded text**, and assuming it was
    /// cost this project a driven run. See [`Self::find`].
    pub text: String,
    // ★★★ `Reading::find` was HERE until 2026-08-28, and its deletion is the
    // end of a three-act story worth keeping in one place.
    //
    // **Act 1 — it was built for a reason that turned out to be false.** It
    // computed *"the longest stretch of the run's text whose glyph byte-ranges
    // are contiguous"*, on the stated grounds that the extraction synthesises a
    // space wherever a `TJ` offset exceeds the word-gap threshold. `pdfcer-core`
    // measured 256 fixtures and found **zero** glyph runs containing one. The
    // symptom was real and the mechanism was invented — a hypothesis wearing a
    // fact's clothes, written into a doc comment, a handover and a resume file
    // without one measurement behind it.
    //
    // **Act 2 — it turned out to be correct anyway, and that was the
    // uncomfortable part.** The question it rested on — *do the glyphs sharing
    // one `operator_span` always slice a contiguous, matchable range out of the
    // run's text?* — was filed, and answered by a probe over **4,289 files,
    // 18,559 runs, 669,436 glyphs and 29,246 operator spans: zero exceptions**,
    // sabotage-checked. So the walk was sound and its justification was void,
    // which is the least comfortable of the four possible combinations.
    //
    // **Act 3 — the last thing keeping it alive was fixed.** `Pass 145.0` gave
    // every FORMAT path `FormatRequest::whole_operator`, and this field survived
    // one more day only to feed `preview_font_resources`, whose coverage gate
    // took the text as a **parameter** — so an empty `find` there tested zero
    // characters and reported every face as accepted. That was filed as a trap
    // rather than absorbed, and `Pass 147.0` made the pre-flight resolve the pin
    // itself through the same `effective_find` the commit path uses.
    //
    // ⇒ Nothing feeds it now, so it is gone rather than kept "in case". A
    // mechanism with no caller rots, and the next reader cannot tell a
    // deliberate fallback from a forgotten one.
    /// The fill colour in force, in whatever space the file set it.
    ///
    /// `TextColor::Other` is a real and important answer: the run is painted in
    /// a space this Pass does not decode, and a caller that renders it as its
    /// nearest RGB — and then writes that RGB back — has converted the
    /// operator's ink without being asked.
    pub fill: Option<TextColor>,
}

/// Everything one provenance read yields: the operand, and what it looks like.
#[derive(Debug, Clone, PartialEq)]
pub struct Inspected {
    /// The locator, for a verb.
    pub pin: Pinned,
    /// The reading, for a panel.
    pub style: RunStyle,
}

/// Every show operator of `run` on `page`, in ONE extraction.
///
/// The form `crate::app::actions::textstyle` wants: a restyle acts on operators
/// and a selection names runs, and this is the hop between them. See
/// [`operators_in_run`] for why the two are not the same thing.
#[must_use]
pub fn operators(doc: &OpenDoc, page: usize, run: usize) -> Vec<Operator> {
    let Some(page_ref) = doc.pages.get(page) else {
        return Vec::new();
    };
    use crate::app::settings::SettingsExt;
    let opts = doc.settings.extract_options().with_provenance(true);
    let Ok(text) =
        pdfcer_core::text_extract::extract_page_view(&doc.session.view(), page_ref, page, &opts)
    else {
        return Vec::new();
    };
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    operators_in_run(&model, &text, run)
}

/// **Which faces on this page `set_font` would actually ACCEPT for this run**,
/// and the string to pass for each.
///
/// `Pass 142.1`, consumed 2026-08-27. This project asked for it by name after
/// shipping a face chooser built the only way that was then possible.
///
/// # ★★★ What the list used to be, and the two ways it was wrong
///
/// The face combo was built from `fontinfo::FontInventory`, filtered to the
/// records naming this page, showing each `/BaseFont` with its §9.6.4 subset
/// tag stripped. The engine's own summary of that arrangement: *"a list built
/// from the first key is a superset of the second that is usually right, and
/// when it is wrong the operator finds out by pressing a button and getting a
/// refusal."*
///
/// Two failures, and the second is much worse than the first:
///
/// **1. Entries that cannot work.** `fontinfo` is keyed on the font
/// **dictionary**; `set_font` matches on `/BaseFont` and then asks whether the
/// face can encode *this run's characters*. A face that cannot — `Times-Bold`
/// with no code for `o` — was offered, pressed, and refused. A control whose
/// entries may not work is what this project spends its time removing.
///
/// **2. ★★ The wrong twin, silently.** One page can carry **two font
/// dictionaries sharing one `/BaseFont`** — two subsets of one face — which the
/// survey behind the Fonts panel found in **87 % of embedding files**. A name
/// match reaches exactly one of them, arbitrarily, and the operator is given no
/// hint that a choice was made on their behalf. That is not a refusal an
/// operator can see; it is the wrong font, applied.
///
/// `FontResourceEntry::selector` is the fix for both: it is *the string to pass
/// to `set_font` to reach THIS resource* — normally the stripped `/BaseFont`,
/// and the **resource key** instead when the page carries twins, with
/// `base_font_ambiguous` set so a chooser can say so.
///
/// # Why it takes the run and not just the page
///
/// Because acceptance is per-run. The same face is accepted for one line of a
/// page and refused for another, depending on which characters each contains —
/// so a page-scoped list would be back to being a superset. The `find` and the
/// pinned span are the same operands `format_text` takes, which is what makes
/// the preview and the commit incapable of disagreeing (the engine moved the
/// four conditions into one `accept_font_target` for exactly that reason, R221).
///
/// ## Cost, and why this is not called per frame
///
/// It is `&self` and side-effect-free, and it runs one extraction plus one
/// acceptance test per page `/Font` resource. The callers hold it behind the
/// same `(page, run, epoch)` stamp their style read-back uses, so it is paid
/// once per selection change rather than sixty times a second.
///
/// `None` when the run does not pin or the preview refuses — a chooser then
/// falls back to the inventory list, which is the behaviour that shipped and is
/// wrong only in the two ways above, rather than to an empty combo.
/// ## ★★★ It takes the ALREADY-INSPECTED reading, and the first draft did not
///
/// The first version of this function called [`inspect`] itself, which reads
/// well and is a **doubling of the most expensive thing this shell does**: its
/// only caller is the properties draft's `sync`, which had just run `inspect`
/// to get the face, size and colour. Two extractions with provenance capture on
/// is **784 ms** on the operator's benchmark sheet where one is 392 — paid on
/// every selection change, to answer two halves of one question.
///
/// Caught by asking what the caller already had rather than by measuring a slow
/// build, which is the cheap direction to catch it in. The signature is now the
/// honest one: this function's job is the **preview**, and the extraction is the
/// caller's.
#[must_use]
pub fn font_preflight(
    doc: &OpenDoc,
    page: usize,
    read: &Inspected,
) -> Option<pdfcer_core::text_edit::FontPreflight> {
    // ★★★ An EMPTY find, since `Pass 147.0`. The pre-flight resolves the
    // pinned operator's own characters through `effective_find` — the same
    // function the commit path calls — so the preview and the commit cannot
    // disagree about what was tested.
    //
    // ★★ It was `&read.style.find` for one day, and that field existed for one
    // day longer than it should have because of it. Passing `""` before `147.0`
    // would have tested **zero characters** and reported every face on the page
    // as accepted — silently, and worse than the superset it replaced. That was
    // filed as a trap rather than worked around; see `Reading`'s own note for
    // the whole story.
    //
    // ★ An empty find with **no** pin is refused by name, which the engine
    // added in the same Pass after a test showed `s.text.contains("")` is true
    // of every string — so a caller who forgets to pin gets an error rather
    // than the first operator on the page.
    doc.session
        .preview_font_resources(page, "", Some(read.pin.span))
        .ok()
}

/// The pin **and** the current style for `run` on `page`, in one extraction.
///
/// The form a properties panel wants. [`resolve`] is this with the reading
/// dropped, kept as its own entry point so an edit path cannot accidentally
/// hold a stale style struct alongside a fresh pin.
#[must_use]
pub fn inspect(doc: &OpenDoc, page: usize, run: usize) -> Option<Inspected> {
    let page_ref = doc.pages.get(page)?;
    use crate::app::settings::SettingsExt;
    let opts = doc.settings.extract_options().with_provenance(true);
    let text =
        pdfcer_core::text_extract::extract_page_view(&doc.session.view(), page_ref, page, &opts)
            .ok()?;
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    let pin = of_run(&model, run)?;
    let p = model.provenance(GlyphRef::new(run, 0))?;
    Some(Inspected {
        pin,
        style: RunStyle {
            size: p.tf_size,
            text: text
                .runs
                .get(run)
                .map(|r| r.text.clone())
                .unwrap_or_default(),
            // Lossy rather than strict: a resource key is a PDF name, which is
            // bytes, and a name that is not UTF-8 is legal. Losing a byte in a
            // label is better than showing no label at all, and nothing acts on
            // this string — the edit uses the pin.
            font_resource: p
                .font_resource
                .as_ref()
                .map(|k| String::from_utf8_lossy(k).into_owned()),
            fill: p.fill_color,
        },
    })
}

/// ★★★ **Every show operator a run is made of**, in content order, each with
/// the `find` text that names all of it.
///
/// # Why a run is not an operator, which is the thing this function exists to say
///
/// It is tempting — and this shell did it for one afternoon — to treat a
/// `TextRun` as a show operator: pin the first glyph's operator, pass the run's
/// text as `find`, and restyle. It works on most runs and fails on real
/// drawings, because `layout` closes a run on *geometry* and a producer closes a
/// show operator on *whatever its writer felt like*. A title-block cell reading
/// `FINISH` came back as one run spanning several `Tj`s, so the pin named the
/// first and the `find` named all of them, and `format_text` refused with *"text
/// to format ("FINISH ") was not found in an editable run on the page"* — on a
/// page where the very same string is found instantly by an UNpinned search.
///
/// That refusal is correct and is not a bug: `find` selects a contiguous code
/// range **within one string element**, and the shell was asking for a range
/// that spans several.
///
/// ⇒ **The operator is the unit of a restyle**, so the operator is what this
/// answers with. A run of three `Tj`s is three entries, three `format_text`
/// calls and three undo entries, and every one of them restyles exactly what it
/// names.
///
/// # The `find` per entry
///
/// The glyphs that share that operator's span, sliced out of the run's text by
/// their own `text_start`/`text_len`. Every byte comes from a glyph, so no
/// **derived** character — a space the extraction synthesised from a `TJ` offset
/// — can get in, which is the second way the naive version failed.
///
/// # Order
///
/// Content order, ascending. A caller wanting the descending order that keeps
/// byte offsets stable across edits reverses it, and
/// `crate::app::actions::textstyle` does, with the argument.
#[must_use]
pub fn operators_in_run(
    model: &EditableTextModel<'_>,
    page_text: &pdfcer_core::text_extract::PageText,
    run: usize,
) -> Vec<Operator> {
    let mut out: Vec<Operator> = Vec::new();
    let Some(text) = page_text.runs.get(run) else {
        return out;
    };
    for (index, glyph) in text.glyphs.iter().enumerate() {
        let Some(p) = model.provenance(GlyphRef::new(run, index)) else {
            continue;
        };
        let (gs, ge) = (
            glyph.text_start as usize,
            glyph.text_start as usize + glyph.text_len as usize,
        );
        // ★★★ The per-operator `find` text was built HERE until 2026-08-27,
        // by walking the glyphs and extending a byte cursor over the run's
        // text — *"but only over bytes a glyph actually covers. A gap here is a
        // derived character and must not join the two halves."*
        //
        // That was a **second locator**, living beside the engine's, and it is
        // deleted rather than kept. `Pass 145.0` made a pinned request with an
        // empty `find` mean *the whole operator*, so the pin alone is the whole
        // address and there is nothing left to slice.
        //
        // ★★ The measurement that made deleting it safe rather than hopeful:
        // the engine probed 4,289 fixture files, 18,559 runs, 669,436 glyphs,
        // **29,246 distinct operator spans, zero non-contiguous groups and zero
        // groups whose slice did not index the run's text cleanly** — and
        // sabotage-checked the detector so a green result is not vacuous. The
        // invariant this walk was quietly relying on is now a documented
        // guarantee with a test that re-runs on every `cargo test`.
        //
        // ★ The same probe settled the other question: **2,420 of 18,559 runs
        // (13 %) carry glyphs from more than one show operator.** This function
        // is not an edge case; it is the ordinary shape of real typeset text.
        if out
            .last()
            .is_none_or(|last| last.pin.span != p.operator_span)
        {
            out.push(Operator {
                pin: Pinned {
                    span: p.operator_span,
                    target: target_of(p),
                    text_matrix: p.text_matrix,
                    ctm: p.ctm,
                },
            });
        }
        let _ = (gs, ge);
    }
    out
}

/// One show operator inside a run: how to name it.
///
/// ★ It carried a `find` and a byte cursor until 2026-08-27. Both are gone —
/// `Pass 145.0` made a pinned request with an empty `find` mean *the whole
/// operator*, so the pin is the whole address. See [`operators_in_run`] for the
/// measurement that made the deletion safe rather than hopeful.
#[derive(Debug, Clone, PartialEq)]
pub struct Operator {
    /// The locator.
    pub pin: Pinned,
}
