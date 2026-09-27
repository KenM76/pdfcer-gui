//! # `canvas::textedit::pin` — naming the exact show operator, and the exact
//! buffer it lives in
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/pin.md`.

use pdfcer_core::span::ByteSpan;
use pdfcer_core::text_edit::{BlockRecognitionOptions, EditTarget, EditableTextModel, GlyphRef};
use pdfcer_core::text_extract::TextColor;

use crate::app::state::OpenDoc;

/// Which content buffer a glyph's span indexes — the `EditTarget` half of a pin.
///
pub(super) fn target_of(
    p: &pdfcer_core::text_extract::GlyphProvenance,
) -> pdfcer_core::text_edit::EditTarget {
    match p.content_stream {
        pdfcer_core::text_extract::ContentStreamRef::Page => {
            pdfcer_core::text_edit::EditTarget::PageContents
        }
        pdfcer_core::text_extract::ContentStreamRef::Form { object } => {
            pdfcer_core::text_edit::EditTarget::Form { object }
        }
    }
}

/// Everything a pinned text verb needs in order to name one show operator.
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
#[must_use]
pub fn of_run(model: &EditableTextModel<'_>, run: usize) -> Option<Pinned> {
    let p = model.provenance(GlyphRef::new(run, 0))?;
    // NAME THE BUFFER THE PIN INDEXES. `Pass 119.0`, and this
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
#[must_use]
pub fn resolve(doc: &OpenDoc, page: usize, run: usize) -> Option<Pinned> {
    inspect(doc, page, run).map(|i| i.pin)
}

/// What a run currently **looks like** — the three facts a properties panel
/// shows and a restyle changes.
#[derive(Debug, Clone, PartialEq)]
pub struct RunStyle {
    /// The `Tf` size in points.
    pub size: f32,
    /// The `/Resources /Font` **key** in force — `F1`, not `Helvetica`.
    ///
    /// Not the `/BaseFont`, and the difference is why a caller showing this
    /// to an operator has to join it against the document's font inventory
    /// first. `GlyphProvenance` records what the content stream said, which is
    /// a resource key; the human-readable name lives in the font dictionary the
    /// key resolves to.
    pub font_resource: Option<String>,
    /// **The run's own characters** — and they are not decoration.
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
    /// It stays valid across a restyle: `format_text` changes how characters
    /// look and never which characters they are, so a text captured before a
    /// multi-run gesture is still the right `find` for the runs still to come.
    ///
    /// **It is NOT the show operator's decoded text**, and assuming it was
    /// cost this project a driven run. See [`Self::find`].
    pub text: String,
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
#[must_use]
pub fn operators(doc: &OpenDoc, page: usize, run: usize) -> Vec<Operator> {
    // The shared extraction — see `crate::app::cache::provenance`. `None`
    // covers both "no such page" and "the text could not be read", which are
    // one answer here: there are no operators to restyle either way.
    let Some(text) = doc.provenance_page_text(page) else {
        return Vec::new();
    };
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    operators_in_run(&model, &text, run)
}

/// **Which faces on this page `set_font` would actually ACCEPT for this run**,
/// and the string to pass for each.
#[must_use]
pub fn font_preflight(
    doc: &OpenDoc,
    page: usize,
    read: &Inspected,
    candidate: Option<&str>,
) -> Option<pdfcer_core::text_edit::FontPreflight> {
    // An EMPTY find, since `Pass 147.0`. The pre-flight resolves the
    // pinned operator's own characters through `effective_find` — the same
    // function the commit path calls — so the preview and the commit cannot
    // disagree about what was tested.
    //
    // It was `&read.style.find` for one day, and that field existed for one
    // day longer than it should have because of it. Passing `""` before `147.0`
    // would have tested **zero characters** and reported every face on the page
    // as accepted — silently, and worse than the superset it replaced. That was
    // filed as a trap rather than worked around; see `Reading`'s own note for
    // the whole story.
    //
    // An empty find with **no** pin is refused by name, which the engine
    // added in the same Pass after a test showed `s.text.contains("")` is true
    // of every string — so a caller who forgets to pin gets an error rather
    // than the first operator on the page.
    match candidate {
        // The engine's own two entry points, chosen here rather than by
        // passing `None` through one of them — so a reader of this function
        // sees which question was asked.
        None => doc
            .session
            .preview_font_resources(page, "", Some(read.pin.span))
            .ok(),
        Some(text) => doc
            .session
            .preview_font_resources_for(page, "", Some(read.pin.span), text)
            .ok(),
    }
}

/// The pin **and** the current style for `run` on `page`, in one extraction.
#[must_use]
pub fn inspect(doc: &OpenDoc, page: usize, run: usize) -> Option<Inspected> {
    let text = doc.provenance_page_text(page)?;
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

/// **Every show operator a run is made of**, in content order, each with
/// the `find` text that names all of it.
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
        // The per-operator `find` text was built HERE until 2026-08-27,
        // by walking the glyphs and extending a byte cursor over the run's
        // text — *"but only over bytes a glyph actually covers. A gap here is a
        // derived character and must not join the two halves."*
        //
        // That was a **second locator**, living beside the engine's, and it is
        // deleted rather than kept. `Pass 145.0` made a pinned request with an
        // empty `find` mean *the whole operator*, so the pin alone is the whole
        // address and there is nothing left to slice.
        //
        // The measurement that made deleting it safe rather than hopeful:
        // the engine probed 4,289 fixture files, 18,559 runs, 669,436 glyphs,
        // **29,246 distinct operator spans, zero non-contiguous groups and zero
        // groups whose slice did not index the run's text cleanly** — and
        // sabotage-checked the detector so a green result is not vacuous. The
        // invariant this walk was quietly relying on is now a documented
        // guarantee with a test that re-runs on every `cargo test`.
        //
        // The same probe settled the other question: **2,420 of 18,559 runs
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
#[derive(Debug, Clone, PartialEq)]
pub struct Operator {
    /// The locator.
    pub pin: Pinned,
}

// ===========================================================================
// FROM A CLICKED TEXT OBJECT TO THE RUNS A RESTYLE CAN ADDRESS
//
// `OPERATOR_REQUESTS.md` **O89**, and the half that had no route:
//
// > *"I don't see where I am able to edit the color of text, vectors, etc."*
//
// Text colour shipped, gated on a TEXT selection — an operator has to arm the
// Text tool and sweep the words. Clicking the text selects the *object*, which
// is a paint-order index into `PageObjects`, and
// `crate::panels::properties::text`'s header states correctly that the two
// index spaces are unrelated and that **an inference between them would
// restyle text the operator did not select**.
//
// THIS IS NOT AN INFERENCE. It is the one exact join the two models share:
// a `TextObject` carries the `BT`…`ET` **byte span** in the decoded content
// buffer (`pdfcer_core::vector::VectorObject::bytes`), and every glyph's
// `pdfcer_core::text_extract::GlyphProvenance` carries the byte span of the
// show operator that produced it, **in the same buffer**, together with the
// name of that buffer. A run belongs to the object exactly when its operator's
// span lies inside the object's span and both name the page's own stream. No
// geometry is compared, no bounding box is overlapped, no threshold is chosen.
//
// ⇒ The mapping is *containment of byte ranges*, which is the same kind of fact
// [`of_run`] already relies on to name an operand at all. If it were wrong, a
// pinned edit would already be editing the wrong text.
// ===========================================================================

/// **Which runs of the page's extraction a selected text OBJECT is made of**,
/// and what colour each of them is painted in.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectText {
    /// The first run of the object, in content order.
    pub first_run: usize,
    /// The last run of the object, in content order.
    ///
    /// A **range**, not a set, and the difference is the operand. The runs
    /// between `first_run` and `last_run` are exactly what a hand sweep from
    /// the object's first character to its last would cover —
    /// `TextSelection::runs` is literally `(start.run..=end.run)` — so taking
    /// the range makes the object route and the sweep route the *same gesture
    /// with the same operand*, rather than two things that usually agree.
    ///
    /// A set would be the more precise answer to *"which runs did this object
    /// produce"* and the wrong answer to *"what would he have swept"*. They can
    /// differ only where another object's show operators interleave with this
    /// one's inside its own `BT`…`ET`, which the content-stream grammar
    /// forbids: `Do` is not a permitted text-object operator (§9.4), so no
    /// form's runs can land in the middle of a page text object's.
    pub last_run: usize,
    /// The fill in force at each run of `first_run..=last_run`, in order.
    ///
    /// See [`RunFill`] — and in particular why *"no colour operator"* and *"no
    /// glyphs"* are two variants rather than one `None`.
    pub fills: Vec<RunFill>,
}

/// **What one run of a text object is painted in.**
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunFill {
    /// The run carries no glyphs, so it has no colour and cannot disagree with
    /// one. `crate::app::actions::textstyle::apply` skips these runs for the
    /// same reason: there is no show operator behind them to restyle.
    NoGlyphs,
    /// Glyphs, and **no colour operator in force** — §8.6.8's default, black.
    /// A real colour, stated by the file's silence rather than by an operator.
    DefaultBlack,
    /// Glyphs, painted in a colour the extraction modelled.
    Painted(TextColor),
}

/// Read [`ObjectText`] for the page object at `object`, or `None`.
#[must_use]
pub fn object_text(doc: &OpenDoc, page: usize, object: usize) -> Option<ObjectText> {
    // The object's own byte span, and the confirmation that it is text at all.
    // Read first and dropped before the extraction, so the provider's `Ref` is
    // not held across the expensive half.
    let span = {
        // The provider is asked through the trait rather than through its
        // inherent `page_objects()`, because the trait method is guarded on the
        // PAGE — the provider decomposes exactly one page, and its inherent
        // accessor would answer with a different sheet's model without saying
        // so. Same rule `super::super::target`'s own implementation states.
        use crate::canvas::target::CanvasTargetProvider as _;
        let provider = doc.page_objects()?;
        let model = provider.page_objects_model(page)?;
        match model.objects.get(object)? {
            pdfcer_core::vector::VectorObject::Text(t) => t.bytes,
            // A path has its own colour control (`panels::properties::paint`)
            // and an image has no text. Answering `None` rather than an empty
            // reading keeps *"this is not text"* distinguishable from *"this is
            // text nothing could be read from"*, which are two different
            // sentences on screen.
            _ => return None,
        }
    };

    // The shared extraction — see `crate::app::cache::provenance`.
    let text = doc.provenance_page_text(page)?;
    // Borrowed after the extraction, never across it (see the span above).
    use crate::canvas::target::CanvasTargetProvider as _;
    let provider = doc.page_objects()?;
    let objects = provider.page_objects_model(page)?;
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());

    let mut first: Option<usize> = None;
    let mut last: usize = 0;
    for run in 0..text.runs.len() {
        let Some(p) = model.provenance(GlyphRef::new(run, 0)) else {
            // No glyphs: a derived space or an `/ActualText` run. It carries no
            // operator, so it cannot be attributed to any object — and it must
            // not be, because attributing one would extend the range past the
            // object's own last character.
            continue;
        };
        if !operator_is_inside(p, span) || !engine_places_in(objects, p, object) {
            continue;
        }
        if first.is_none() {
            first = Some(run);
        }
        last = run;
    }
    let first_run = first?;

    // The fills for the whole inclusive range, INCLUDING the glyphless runs the
    // loop above skipped: the range is the operand, so the reading has to
    // describe the range. A glyphless run is neither a colour that agrees nor
    // one that disagrees — `crate::app::actions::textstyle::apply` skips it for
    // the same reason.
    let fills = (first_run..=last)
        .map(|run| match model.provenance(GlyphRef::new(run, 0)) {
            None => RunFill::NoGlyphs,
            // §8.6.8: no colour operator in force means black, and black is a
            // colour. See [`RunFill`] for the flattening this distinction
            // prevents.
            Some(p) => p.fill_color.map_or(RunFill::DefaultBlack, RunFill::Painted),
        })
        .collect();

    Some(ObjectText {
        first_run,
        last_run: last,
        fills,
    })
}

/// **Does the engine place this glyph in a run of object `object`?** The
/// span containment above is a cheap filter; this is the join's authority,
/// `vector::locate_text_run`, which answers `None` rather than guessing.
fn engine_places_in(
    objects: &pdfcer_core::vector::PageObjects,
    p: &pdfcer_core::text_extract::GlyphProvenance,
    object: usize,
) -> bool {
    matches!(
        pdfcer_core::vector::text_locate::locate_text_run(objects, p),
        Some(pdfcer_core::vector::text_locate::TextRunRef::Page { object_index, .. })
            if object_index == object
    )
}

/// **Does this glyph's show operator lie inside `object`'s `BT`…`ET` span?**
fn operator_is_inside(p: &pdfcer_core::text_extract::GlyphProvenance, object: ByteSpan) -> bool {
    matches!(
        p.content_stream,
        pdfcer_core::text_extract::ContentStreamRef::Page
    ) && p.operator_span.start >= object.start
        && p.operator_span.end() <= object.end()
}
