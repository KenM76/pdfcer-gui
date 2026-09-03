//! # `canvas::textedit::place` — where a press puts the caret
//!
//! ## The seam
//!
//! Split out of [`super`] on 2026-08-21 under R2, when the text box took that
//! file past the 1,500-line ceiling. It is the seam the file already drew with
//! its own banner — *"Starting a draft"* — and it is a real subject rather than
//! a size-driven cut: everything here answers **where does a press put the
//! caret**, and nothing here knows what typing does afterwards.
//!
//! ## The two gestures, and why they are two
//!
//! | gesture | anchor | what commits |
//! |---|---|---|
//! | **click** on existing text | [`Anchor::Run`] | `edit_text` — one show operator's text replaced |
//! | **click** on bare page | [`Anchor::Origin`] | `add_text` — one single-line run at a point |
//! | **drag** a rectangle | [`Anchor::Box`] | `add_text` boxed — a wrapped paragraph |
//!
//! ★★ The third arrived on 2026-08-21, on the operator's *"I should be able to
//! make it multi line."* It has to be a drag, and the reason is the file format
//! rather than a preference: **a PDF has no paragraph.** Each visual line is its
//! own show operator at its own absolute position, so something must decide
//! where the second line starts — a width to wrap against — and a width is a
//! rectangle somebody draws.
//!
//! ## ★ What this module refuses, and why each refusal is a sentence
//!
//! [`Refusal`]'s variants are shown on the status row, never dropped. That is
//! `DEFECTS.md` D4a's whole lesson: the old shell's answer to a caret it could
//! not place was a boolean and a keyboard that stopped responding, and the
//! operator reported the feature as broken for weeks.

use egui::Pos2;

use super::{Anchor, Draft, Refusal, TextEditKind, abandon, commit_into, read, store};
use crate::app::state::OpenDoc;
use pdfcer_core::text_edit::{BlockRecognitionOptions, EditableTextModel};

/// Everything resolving a click needs, gathered by the caller so this module
/// reads no globals.
pub struct Click<'a> {
    /// The document, for its page-text extraction.
    pub doc: &'a OpenDoc,
    /// Which page was clicked.
    pub page_index: usize,
    /// Which verb is armed.
    pub kind: TextEditKind,
    /// Where, in canvas space.
    pub canvas_point: Pos2,
}

/// **Start (or move) a draft from a click.**
///
/// Returns the refusal to show, if the click could not begin one. `Ok(())` means
/// a draft is now in flight and the next keystroke will reach it.
///
/// # Why an existing draft is committed rather than discarded
///
/// Clicking elsewhere while composing is the operator saying *"that word is
/// finished"*, not *"throw it away"* — every editor behaves this way, and the
/// old shell settled it under the name `commit_on_click`. So the caller is
/// handed the commit as an [`crate::app::actions::Action`] before the new draft
/// starts. Escape, and only Escape, discards.
pub fn click(
    ctx: &egui::Context,
    click: &Click<'_>,
    actions: &mut Vec<crate::app::actions::Action>,
) -> Result<(), Refusal> {
    // ★★★ A CLICK INSIDE THE EDITOR BOX BELONGS TO THE DRAFT, not to the page.
    //
    // Without this, clicking into the middle of what you are typing would
    // commit the draft and open a new one from whatever run the PAGE has at
    // that point — which after the first keystroke is not the text on screen,
    // because the editor box covers the original glyphs. The operator's caret
    // would land somewhere unrelated to where they clicked, and their draft
    // would be committed by the act of trying to correct it.
    //
    // `canvas::textedit::keys::pointer` has already placed the caret from the
    // galley that was actually drawn, so there is nothing to do here but stand
    // aside. Answering `Ok(())` rather than a refusal is the honest report: the
    // click was handled, just not by this function.
    if crate::canvas::textedit::hit::owns_canvas(ctx, click.canvas_point) {
        return Ok(());
    }
    if let Some(existing) = read(ctx) {
        commit_into(ctx, &existing, actions);
        abandon(ctx);
    }
    let anchor = match click.kind {
        TextEditKind::Add => {
            let page = click
                .doc
                .pages
                .get(click.page_index)
                .ok_or(Refusal::NoText)?;
            let pdf = crate::viewer::canvas_to_pdf_space(click.canvas_point, page)
                .ok_or(Refusal::NoText)?;
            Anchor::Origin {
                x: f64::from(pdf.x),
                y: f64::from(pdf.y),
            }
        }
        // ★★ **A click that names no run starts a new one**, as of 2026-08-19.
        //
        // This used to be `resolve_run(click)?` — a bare `?`, so a click on
        // blank paper with the caret armed refused, wrote a sentence to the
        // status row, and did nothing. Two separate tools were needed to type a
        // character in an empty spot versus in an existing word, and which one
        // you had was invisible.
        //
        // The operator, 2026-08-19:
        //
        // > *"How do I make new text when I click on the canvas and expect to
        // > edit there? Same problem as the previous."*
        //
        // Every editor he has used does this with **one** text tool: click in
        // text to edit it, click in space to start some. So a `NoRun` refusal
        // becomes an origin at the click point, and the two ribbon commands
        // (`edit.text`, `edit.add_text`) survive as two doors into one room.
        //
        // ★ Only `NoRun` falls through. Every other refusal — an encrypted
        // document, a page that will not decompose, a run the engine cannot
        // address — is still reported, because those say *this cannot be done
        // here* rather than *there is nothing here*. Swallowing them would put
        // a caret on a page that cannot take the edit, which is D4a's defect
        // with a nicer opening move.
        TextEditKind::Edit => match resolve_run(click) {
            Ok(anchor) => anchor,
            Err(Refusal::NoRun) => {
                let page = click
                    .doc
                    .pages
                    .get(click.page_index)
                    .ok_or(Refusal::NoText)?;
                let pdf = crate::viewer::canvas_to_pdf_space(click.canvas_point, page)
                    .ok_or(Refusal::NoText)?;
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "text-edit-became-add reason=no-run-under-the-click".to_owned()
                });
                Anchor::Origin {
                    x: f64::from(pdf.x),
                    y: f64::from(pdf.y),
                }
            }
            Err(other) => return Err(other),
        },
    };
    let text = match &anchor {
        Anchor::Run { original, .. } => original.clone(),
        // Both authoring anchors start empty. A box is not pre-filled with
        // anything: an operator who drags a rectangle has asked for somewhere to
        // type, not for a suggestion.
        Anchor::Origin { .. } | Anchor::Box { .. } => String::new(),
    };
    // ★★ The caret lands WHERE THE CLICK LANDED, not at the end.
    //
    // `caret_index_at` measures the click's x against the run's own glyph
    // advances - the same glyph boxes the caret is drawn from - so clicking
    // between the `1` and the ` ` of `SHEET 1 OF 4` puts the caret between
    // them. Before 2026-08-20 the draft had no caret index at all, so a click
    // anywhere in a run behaved as a click at its end.
    //
    // Falls back to the end of the text, which is the old behaviour, when the
    // run's glyphs cannot be read. That is the right fallback rather than the
    // start: appending is the less destructive of the two if the operator
    // types without looking.
    let caret = match &anchor {
        Anchor::Run { run, .. } => {
            caret_index_at(click, *run).unwrap_or_else(|| text.chars().count())
        }
        Anchor::Origin { .. } | Anchor::Box { .. } => 0,
    };
    store(
        ctx,
        Draft {
            page: click.page_index,
            kind: click.kind,
            anchor,
            text,
            caret,
            mark: None,
            seeded: false,
        },
    );
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // The whole reason this line exists: an armed text tool with a caret in
        // it and an armed text tool without one are the same screenshot, and the
        // caret blinks so even a captured frame cannot settle it. `DEFECTS.md`
        // D14's lesson — the freehand trail that authored two points — is that a
        // trace line must carry the number a wrong build would get wrong, so this
        // names the run and the length rather than only saying a caret exists.
        let d = read(ctx);
        let (anchor, len) = d.as_ref().map_or((String::new(), 0), |d| {
            (
                match &d.anchor {
                    Anchor::Run { run, .. } => format!("run={run}"),
                    Anchor::Origin { x, y } => format!("origin={x:.1},{y:.1}"),
                    Anchor::Box {
                        llx, lly, urx, ury, ..
                    } => format!("box={llx:.1},{lly:.1},{urx:.1},{ury:.1}"),
                },
                d.text.chars().count(),
            )
        });
        format!(
            "text-edit-caret kind={:?} page={} {anchor} len={len}",
            click.kind, click.page_index
        )
    });
    Ok(())
}

/// **Open a draft anchored to a dragged rectangle** — the multi-line entrance.
///
/// The operator, 2026-08-21: *"I should be able to make it multi line."*
///
/// # ★ The conversion is `markup::band::endpoints`, not a new one
///
/// That function is the canvas → page hop the markup band and the
/// text-annotation band already use, and reusing it is the standing rule rather
/// than convenience: a second conversion is how a preview and an authored box
/// come to disagree about where the operator dragged. It also normalises the
/// two raw endpoints exactly once, which is why the outcome carries them raw.
///
/// # A degenerate drag opens nothing
///
/// A box with no width has no width to wrap against, so it would accept
/// keystrokes and author a single line at an arbitrary place — a control that
/// takes input and does something else with it, which is this project's
/// defining defect class. The floor is deliberately generous: below it the
/// operator did not mean to draw a box, and a click is the gesture that places
/// a caret.
pub fn begin_box(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page_index: usize,
    from: egui::Pos2,
    to: egui::Pos2,
    page: &pdfcer_core::page_tree::Page,
) {
    /// The smallest box, in PDF points, that is worth typing into.
    ///
    /// Twelve is one default line's height: below that the box could not show a
    /// single line of text at the pen's default size, so it is not a box the
    /// operator can have meant.
    const MIN_PT: f64 = 12.0;

    // ★ A sweep that STARTS inside a live editor box is a text selection, not
    // a new box — see `keys::pointer`, which has already made it one. Without
    // this, dragging across what you are typing would commit that draft and
    // open a second box on top of it.
    if crate::canvas::textedit::hit::owns_canvas(ctx, from) {
        return;
    }
    let Some((a, b)) = crate::canvas::markup::band::endpoints(from, to, page) else {
        return;
    };
    let (llx, urx) = (a.0.min(b.0), a.0.max(b.0));
    let (lly, ury) = (a.1.min(b.1), a.1.max(b.1));
    if urx - llx < MIN_PT || ury - lly < MIN_PT {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "text-box-declined w={:.1} h={:.1} floor={MIN_PT:.1}",
                urx - llx,
                ury - lly
            )
        });
        return;
    }
    store(
        ctx,
        Draft {
            page: page_index,
            // ★ `Add`, not `Edit`, and it is not a choice: the box authors new
            // content through `add_text`. Carrying `Edit` here would send an
            // empty box down `edit_text`'s planning path, which pins a span in a
            // run that does not exist.
            kind: TextEditKind::Add,
            anchor: Anchor::Box { llx, lly, urx, ury },
            text: String::new(),
            caret: 0,
            mark: None,
            seeded: false,
        },
    );
    let _ = doc;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // ★ The box's own extent, because that is what a wrong build gets
        // wrong: a band converted through the wrong hop produces a plausible
        // rectangle somewhere else on the page, and the draft would accept
        // typing either way.
        format!(
            "text-box-open page={page_index} box={llx:.1},{lly:.1},{urx:.1},{ury:.1} \
             w={:.1} h={:.1}",
            urx - llx,
            ury - lly
        )
    });
}

/// **Does `run` have no show operator of its own?** `Some(true)` /
/// `Some(false)`, or `None` when the question could not be asked.
///
/// A thin forward to [`crate::app::state::OpenDoc::run_has_no_anchor`], which
/// owns the extraction and the cache. It is worth a named function here anyway:
/// this is the one place in the shell that asks *"can pdfcer-core edit this
/// run"*, so there is one line to change when the answer changes — which it
/// did, on 2026-08-20, when form editing landed and this stopped being about
/// forms at all.
///
/// # Why the answer is cached one level down and not here
///
/// Because the only way to ask is a **second extraction of the whole page with
/// provenance on** - `PageTextCache` deliberately leaves provenance off, since
/// the canvas, the find bar and the text sweep do not need it and it costs.
/// Measured on the benchmark CAD sheet: **336 ms**. Doing it inline froze the
/// UI for a third of a second on every click that landed on text, and made a
/// driven check flake because the trace had not been written by the time the
/// settle window closed - a performance defect that presented as harness
/// flakiness, which this project has been caught by before.
///
/// `None` means **not measured**, never "yes". See `FormRunCache::flags`.
fn has_no_anchor(c: &Click<'_>, run: usize) -> Option<bool> {
    c.doc.run_has_no_anchor(run)
}

/// **Which character boundary a click landed on, inside `run`.**
///
/// Returns a character index in `0..=glyphs.len()`, or `None` when the run or
/// the page cannot be read.
///
/// # How it decides
///
/// Every glyph `pdfcer-core` publishes carries its origin `x` and its `advance`,
/// so a run's character boundaries are `x[0]`, `x[0]+adv[0]`, `x[1]+adv[1]`, …
/// The click's x is compared against each glyph's MIDPOINT: past the midpoint
/// means the caret belongs after that glyph. That is the rule every text field
/// uses and it is what makes clicking "on" a character feel like clicking
/// *near* the boundary the operator was aiming at, rather than requiring them
/// to hit a one-pixel gap.
///
/// # Why the x axis alone
///
/// Because a run is one show operator, which is one baseline. The vertical
/// question - *which line?* - was already answered by `resolve_run`'s hit test
/// before this is called, and asking it again here with different arithmetic is
/// how a caret comes to land on a different line from the one that was clicked.
///
/// ★ Rotated runs. The comparison is done in **PDF user space** against the
/// glyph origins as published, which is the same space `resolve_run` works in.
/// For a run rotated off the horizontal this compares the wrong axis and the
/// caret will land at a boundary the operator did not aim at - it is still
/// inside the run, and still better than always landing at the end, but it is
/// not right. Fixing it properly means projecting the click onto the run's own
/// baseline direction, which needs the text matrix rather than the glyph
/// boxes. Recorded here rather than silently approximated.
fn caret_index_at(c: &Click<'_>, run: usize) -> Option<usize> {
    let page = c.doc.pages.get(c.page_index)?;
    let pdf = crate::viewer::canvas_to_pdf_space(c.canvas_point, page)?;
    let text = c.doc.page_text()?;
    let glyphs = &text.runs.get(run)?.glyphs;
    if glyphs.is_empty() {
        return None;
    }
    let x = pdf.x;
    let mut index = 0;
    for g in glyphs {
        if x < g.x + g.advance / 2.0 {
            return Some(index);
        }
        index += 1;
    }
    Some(index)
}

/// Resolve a click on existing page text to the run it landed in.
///
/// Two hops, and the first is the one `canvas::mapping`'s header calls *the
/// classic silent defect*: the canvas is Y-down from the page's top-left with
/// `/Rotate` applied, and every glyph position `pdfcer-core` publishes is in PDF
/// user space — Y-up from the un-rotated CropBox. `viewer::canvas_to_pdf_space`
/// is the single bridge, and it works by inverting the **renderer's own**
/// transform, so the geometry and the picture agree by construction. This is
/// deliberately the identical route `canvas::textsel::hit` takes, because a
/// second conversion here is how a caret comes to land on a different line from
/// the highlight.
fn resolve_run(c: &Click<'_>) -> Result<Anchor, Refusal> {
    let page = c.doc.pages.get(c.page_index).ok_or(Refusal::NoText)?;
    let pdf = crate::viewer::canvas_to_pdf_space(c.canvas_point, page).ok_or(Refusal::NoText)?;
    let text = c.doc.page_text().ok_or(Refusal::NoText)?;
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    let pos = model
        .hit_test(f64::from(pdf.x), f64::from(pdf.y))
        .ok_or(Refusal::NoRun)?;
    // ★★★ **D4a's boundary used to REFUSE here, and refusing was the defect.**
    //
    // The old code read: *if the caret's visual line begins and ends in
    // different runs, `return Err(Refusal::SpansRuns)`* — on the argument that
    // "the thing the operator is looking at is not a thing `pdfcer-core` can be
    // asked to replace".
    //
    // **That argument was about the LINE. The operator is editing a RUN.**
    // `EditSession::edit_text` replaces one show operator, which is exactly what
    // a click resolves; whether the *neighbours* on that line are separate
    // operators is a fact about what happens to **them**, not about whether this
    // edit is possible. The engine has answered the neighbour question since
    // before this shell existed — `FollowerDisposition::Pin` leaves every
    // follower `Tm` untouched — and this module has been *choosing* between the
    // two dispositions on every commit for days.
    //
    // ★ How expensive the refusal was, measured rather than guessed. A
    // SolidWorks-exported drawing sheet writes text as one show operator per
    // *cell*, so nearly every visual line on a title block or a parts table is
    // multi-run. `tools/ui-verify`'s `text_edit_on_a_real_drawing` armed the
    // caret on `SW41177.pdf` and clicked a point `pdfcer find-text` reported
    // as carrying the word `PART`: **`text-edit-declined reason=SpansRuns`**.
    //
    // So on this operator's own documents the feature refused essentially every
    // click, and his report — *"text editing on canvas still doesn't work"*,
    // twice, weeks apart — was **exactly accurate**. Two passing driven checks
    // said otherwise, and both drove fixtures this repository generated to
    // verify itself: a 924-byte three-line file and blank paper.
    //
    // What the refusal was right about is kept, and it is the *disclosure*: the
    // operator is editing one piece of something that looks like one line, and
    // the other pieces will not move. That is now said — before the commit and
    // after it — rather than used as a reason to do nothing. See
    // [`disposition::Reason::SharesTheLine`], and `crate::text::textedit`.
    //
    // The line is re-derived at commit rather than carried on the `Anchor`, for
    // the reason the `Anchor` docs already give: everything but the original
    // text is a pure function of `(page_text, run)` and a copy would go stale
    // when the page is rebuilt.
    let original = text
        .runs
        .get(pos.run)
        .map(|r| r.text.clone())
        .ok_or(Refusal::NoRun)?;
    if original.is_empty() {
        return Err(Refusal::NoRun);
    }
    // ★★ **Announced BEFORE the edit, not after it.**
    //
    // `MeasureState`'s derived-point rule in the operator's own vocabulary:
    // *a derived point is pdfcer's inference, so rule 4 requires it to be
    // announced before it is picked, not after.* The same is true of a layout
    // consequence — the operator is about to type into what looks like one line,
    // and the pieces beside it will not move. Telling them at commit time is
    // telling them after they have already chosen.
    //
    // It is a **note**, not a refusal: the caret is placed either way, and this
    // returns `Ok`. `crate::text::textedit::pinned_tail_disclosure` says the
    // same fact in the past tense when the edit lands, and both are wanted —
    // one is a warning and one is a receipt.
    if model
        .line_range_at(pos)
        .is_some_and(|(from, to)| from.run != to.run)
    {
        crate::app::actions::record_note(
            c.doc.edit_epoch,
            crate::text::textedit::shares_the_line_note().to_owned(),
        );
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // ★ Named so a driven check can tell the two shapes apart. Until
            // 2026-08-19 this case emitted `text-edit-declined reason=SpansRuns`
            // and placed no caret; it now emits this and a caret, and a harness
            // that could not distinguish "refused" from "allowed and disclosed"
            // would pass against either.
            format!("text-edit-shares-line run={}", pos.run)
        });
    }
    // ★★★ **THE EDITABILITY CHECK, and it is the last thing before the caret.**
    //
    // Added 2026-08-20 on the operator's *"Still no editing text on top of the
    // canvas."* Every stage of this module worked; the commit reached
    // `pdfcer-core` and was refused, **to the trace only**, so a caret took his
    // keystrokes and discarded them in silence.
    //
    // The cause is one field this shell was not reading. `GlyphProvenance`
    // carries a byte span AND the name of the buffer that span indexes:
    //
    // ```text
    // pub content_stream: ContentStreamRef,   // Page, or Form { object }
    // pub operator_span:  ByteSpan,           // …within THAT buffer
    // ```
    //
    // `commit` pins the request with `operator_span` alone. For page-stream text
    // that is right. For text drawn inside a `Do`-invoked form XObject the span
    // indexes the FORM's decoded bytes, while the engine's `find_anchor` walks
    // the PAGE's — so the pin matches nothing, and because a pinned request
    // skips the text search entirely, the loop exhausts and returns
    // `NoMatch(find)`. That error names the operator's text, which is why the
    // sentence reads *"text to edit ("p") was not found in an editable run"*
    // about text that is plainly there.
    //
    // ★★★ THE FORM REFUSAL IS GONE — `Pass 119.0`, 2026-08-20.
    //
    // Everything above this line describes a limit the engine no longer has.
    // It is kept because the mechanism it explains is still the mechanism, and
    // because the next reader needs to know that `content_stream` is a field
    // that MATTERS rather than one that used to.
    //
    // What is left is the one case that is genuinely unreachable and always
    // was: a run with **no show operator of its own**. An `/ActualText` run is
    // derived text — the producer supplied a replacement string for a span of
    // glyphs — so there is no operator for a pinned span to name. The engine
    // reports that as its own `Editability` variant rather than folding it into
    // "not editable", and the reason is worth keeping: this is not text that is
    // out of reach, it is text that has nothing to reach for.
    //
    // The answer is only populated when the extraction asked for provenance,
    // which `app::cache`'s read-only pass deliberately does not — so `None`
    // here means *"not measured"*, never "yes", and the caret is allowed.
    // Refusing on an unmeasured answer would block editing everywhere on a
    // guess, which is the exact failure the engine made `Editability` an enum
    // rather than a `bool` to prevent.
    if has_no_anchor(c, pos.run) == Some(true) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("text-edit-declined reason=NoAnchor run={}", pos.run)
        });
        return Err(Refusal::NoAnchor);
    }

    Ok(Anchor::Run {
        run: pos.run,
        original,
    })
}
