//! # `app::textoperand` — **which runs a Format command acts on**, from either
//! gesture
//!
//! `OPERATOR_REQUESTS.md` **O198**, the operator's words of 2026-09-14:
//!
//! > *"Also get the font selector and editing tools like old [bold] and italic
//! > working. That entire area is always greyed out in the menu, and the
//! > properties area is uneditable too. This is true even when I add a new line
//! > of text."*
//!
//! ## ★★★ The deadlock this module exists to break
//!
//! Every one of the five Format ▸ Font controls — the face chooser, the size
//! field, the colour swatch, Bold and Italic — was gated on **one** field:
//! [`OpenDoc::text_selection`], a range swept with the Text tool. The
//! Properties panel's font editor was gated on the same field. And that field
//! can only be written by a press the text-selection gate accepts:
//!
//! ```text
//! canvas::textsel::gate::takes_the_press(tool, caps)
//!     = tool.is_text() || (tool == Select && !caps.edit_content)
//! ```
//!
//! In **Edit** mode `caps.edit_content` is true, so the Select tool's press
//! yields an *object* selection and never a text range. In **Read** and
//! **Review** the Select tool does produce a range — but the Font group is only
//! shown at all when `mode.edit_content` holds, which is Edit.
//!
//! ⇒ **The only mode that shows the Font group is the only mode whose default
//! gesture cannot produce its operand.** Two correct halves; no way through.
//! That is the whole of *"that entire area is always greyed out"*, and no
//! amount of pressing the button would ever have helped.
//!
//! ## What this module does about it
//!
//! It gives the question *"which runs would a restyle act on?"* **one** answer
//! with **two** sources, in a fixed order:
//!
//! | rung | source | cost |
//! |---|---|---|
//! | 1 | a **live swept range** — [`OpenDoc::text_selection`] | free |
//! | 2 | a **single selected text object**, via [`crate::canvas::textedit::pin::object_text`] | one extraction, 392 ms on the benchmark sheet |
//!
//! Rung 1 first, always, because a sweep is the narrower and more deliberate
//! statement: an operator who swept three words and then happened to have an
//! object selected meant the three words.
//!
//! ## ★★ No new selection unit was invented, and that is the safety argument
//!
//! Rung 2's operand is `first_run..=last_run` derived from the object's
//! `BT`…`ET` **byte span**, joined to each glyph's provenance operator span in
//! the same buffer. That is byte-range containment — exact, total, and the same
//! fact the pinned edit path already stakes every restyle on. It is **not** a
//! bounding-box overlap, which is the geometric inference
//! [`crate::panels::properties::text`]'s header refuses by name because it
//! restyles text the operator did not select, silently, in a file they then
//! send to somebody.
//!
//! [`crate::panels::properties::textobject`] has shipped exactly this join
//! since O89, for the colour swatch alone. This module is that join lifted out
//! of one panel so the ribbon, the dispatch arm and the panel all read it from
//! one place — which is the rule [`crate::app::dispatch::format`]'s own note
//! states: *"which runs does a restyle act on?"* is a **rule**, and a rule
//! stated twice diverges.
//!
//! ## ★★★ Why the cheap half is a separate function
//!
//! [`selected_text_object`] answers *"is there an object-shaped operand?"*
//! without reading a single glyph, and it exists because **a condition is
//! evaluated every frame and an operand is resolved on a press**. Publishing
//! `selection.text_runs` from [`resolve`] would put a 392 ms extraction in the
//! per-frame path on the drawings this program is for, which is
//! `OPERATOR_REQUESTS.md` O74 at its most expensive point.
//!
//! ★ It reads the decomposition through `OpenDoc::page_objects`, which
//! **builds on first use** — 469 ms on the operator's benchmark sheet. That is
//! not a new cost here and cannot be: the function returns early unless exactly
//! one content object is selected on the current page, and an object selection
//! can only have been made by a hit test, which had to build the model to
//! answer. Every reachable call is therefore a cache hit.
//!
//! ## Rule 4
//!
//! Nothing here marks the canvas and nothing here can — it returns run
//! ordinals. Every disclosure a restyle causes is raised off-canvas by
//! [`crate::app::actions::textstyle`], and restyled text renders exactly as the
//! saved file will render it.

use crate::app::state::OpenDoc;

/// **Which gesture produced the runs**, kept beside them because callers need
/// to tell the two sources apart and cannot re-derive it.
///
/// [`crate::panels::properties::textobject`] draws nothing when a sweep is live
/// (the swept editor is drawing the same controls one row above), and
/// [`crate::app::fontband`] stamps its read-back cache on the object index when
/// there is one. A bare `(page, runs)` would make both of those guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Source {
    /// A range swept with the Text tool — [`OpenDoc::text_selection`], live
    /// against the current edit epoch.
    Swept,
    /// The single selected text object, by its index in the page's paint
    /// order.
    Object(usize),
}

/// The page, the runs, and where they came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Operand {
    /// The 0-based page the runs are ordinals into.
    pub page: usize,
    /// Extraction run ordinals, ascending — the operand
    /// [`crate::app::actions::Action::TextStyle`] takes and the engine's
    /// `format_text` acts on.
    pub runs: Vec<usize>,
    /// Which of the two rungs answered.
    pub source: Source,
}

/// **The cheap question: is there an object-shaped operand?**
///
/// Returns `(page, object index)` when exactly one page-content object is
/// selected on the current page and that object is text. `None` for every other
/// state, including the ones that are perfectly ordinary — nothing selected, an
/// annotation selected, two objects selected, a path selected.
///
/// # ★ Why an annotation is excluded first
///
/// An annotation is not page text. `format_text` addresses content-stream show
/// operators; a `/FreeText` annotation's appearance is a separate stream with
/// its own editor. Selecting one must not light the Font group, because
/// pressing Bold would then decline on an operand the control had promised —
/// which is R9 inverted: a control that is live must act.
///
/// # ★ Why exactly one, and not "the first of several"
///
/// The rule the geometry section states and this shares: a multi-object
/// selection has no single subject, and picking one of them is the shell
/// deciding which of the operator's shapes they meant.
#[must_use]
pub(crate) fn selected_text_object(doc: &OpenDoc) -> Option<(usize, usize)> {
    if doc.selection.annot().is_some() {
        return None;
    }
    let page = doc.view.page_index;
    let objects = doc.selection.object_indices_on(page);
    let [object] = objects.as_slice() else {
        return None;
    };
    let object = *object;
    is_text(doc, page, object).then_some((page, object))
}

/// Is the object at this index page text?
///
/// ★ Asked through [`crate::panels::objects::summary::object_kind`], which is
/// the same classification the Objects panel row and the read-only object
/// section use — so what this module calls text and what the panel beside it
/// calls text cannot disagree.
fn is_text(doc: &OpenDoc, page: usize, object: usize) -> bool {
    use crate::canvas::target::CanvasTargetProvider as _;
    doc.page_objects().is_some_and(|provider| {
        provider
            .page_objects_model(page)
            .and_then(|model| model.objects.get(object))
            .is_some_and(|o| {
                crate::panels::objects::summary::object_kind(o)
                    == crate::panels::objects::summary::ObjectKind::Text
            })
    })
}

/// **The live swept range**, or `None`.
///
/// Its own function because three callers ask it and one of them —
/// [`crate::panels::properties::textobject`] — asks it in order to *stand
/// down*. The staleness gate belongs with the data: a run ordinal resolved
/// against an older epoch restyles the **wrong text**, silently.
#[must_use]
pub(crate) fn swept(doc: &OpenDoc) -> Option<Operand> {
    let selection = doc.text_selection.as_ref()?;
    let runs = selection.runs(doc.edit_epoch);
    (!runs.is_empty()).then_some(Operand {
        page: selection.page,
        runs,
        source: Source::Swept,
    })
}

/// **The whole answer**, paying the extraction when the operand is an object.
///
/// # ★★★ Do NOT call this from a paint loop or a condition
///
/// Rung 2 runs one page extraction with provenance capture on, plus one block
/// recognition — **392 ms on the operator's benchmark sheet**. Every per-frame
/// caller holds it behind a `(page, object, edit epoch)` stamp, as
/// [`crate::panels::properties::textobject`] and
/// [`crate::panels::properties::text`] already do and for the same measured
/// reason. The per-press callers — [`crate::app::dispatch::format`]'s Font arm
/// — pay it once per operator gesture, which is what a gesture is for.
///
/// # Returns
///
/// `None` means *this verb has no operand*, which is the ordinary state and not
/// a defect. It is also the state the ribbon is greyed in, so an operator
/// cannot reach a `None` by clicking; a chord can, and a chord pressed with
/// nothing selected is a question rather than a mistake.
#[must_use]
pub(crate) fn resolve(doc: &OpenDoc) -> Option<Operand> {
    if let Some(swept) = swept(doc) {
        return Some(swept);
    }
    let (page, object) = selected_text_object(doc)?;
    let found = crate::canvas::textedit::pin::object_text(doc, page, object)?;
    Some(Operand {
        page,
        runs: (found.first_run..=found.last_run).collect(),
        source: Source::Object(object),
    })
}

/// **A stamped memory of the object rung**, so a surface that asks every frame
/// pays the extraction once per selection instead of once per frame.
///
/// # ★★★ Why this is a type and not a `HashMap` somewhere
///
/// [`resolve`] is honest about costing 392 ms on the operator's benchmark
/// sheet, and that sentence is only survivable because the callers who ask it
/// sixty times a second do not ask *it*. They ask this, which answers from a
/// three-part stamp and calls the resolver only when the stamp moves.
///
/// It is stored as a field of
/// [`crate::panels::properties::text::TextStyleDraft`] — the draft the ribbon
/// band and the Properties panel already **share one instance of** — so both
/// font surfaces are served by a single read. That was the deciding argument
/// for putting the storage there rather than giving this module a cache of its
/// own: a second instance would mean a second 392 ms on the same gesture, and
/// the two would be free to disagree about which runs the operator selected.
///
/// # The stamp is three parts and every one is load-bearing
///
/// * **page** — an object index means nothing without one;
/// * **object** — the operator clicked different text;
/// * **edit epoch** — the object is the same object and its content moved.
///   A restyle, a reflow or an added line all bump it, and a run ordinal
///   resolved before one of those addresses the WRONG text afterwards. This is
///   the term whose absence would be invisible in every test and wrong in the
///   only case that matters.
///
/// # A miss is remembered too
///
/// [`Self::runs`] is `Option<Vec<usize>>` inside an already-stamped record: the
/// outer stamp says *"this was asked"* and the inner `None` says *"and the
/// answer was nothing"*. Without the distinction a text object whose glyphs
/// cannot be placed would be re-extracted every frame at 392 ms apiece — a
/// 2.5 fps application, produced by a cache that was working exactly as
/// written.
#[derive(Default)]
pub(crate) struct Cache {
    /// `(page, object, edit epoch)` the runs below were resolved at.
    stamp: Option<(usize, usize, u64)>,
    /// The resolved run ordinals, or `None` when the resolution found nothing.
    runs: Option<Vec<usize>>,
}

impl Cache {
    /// [`resolve`], behind the stamp.
    ///
    /// — The swept rung is checked **first and unstamped**, because it is free
    /// and because it must win: an operator who swept three words and happens
    /// to also have an object selected meant the three words, and a cached
    /// object answer returned in preference to a live sweep would restyle the
    /// whole paragraph.
    pub(crate) fn resolve(&mut self, doc: &OpenDoc) -> Option<Operand> {
        if let Some(swept) = swept(doc) {
            return Some(swept);
        }
        let (page, object) = selected_text_object(doc)?;
        let stamp = (page, object, doc.edit_epoch);
        if self.stamp != Some(stamp) {
            self.stamp = Some(stamp);
            let started = std::time::Instant::now();
            self.runs = crate::canvas::textedit::pin::object_text(doc, page, object)
                .map(|found| (found.first_run..=found.last_run).collect::<Vec<_>>());
            let ms = started.elapsed().as_millis();
            let runs = self.runs.as_ref().map_or(0, Vec::len);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("text-operand-resolved page={page} object={object} runs={runs} ms={ms}")
            });
        }
        let runs = self.runs.clone()?;
        (!runs.is_empty()).then_some(Operand {
            page,
            runs,
            source: Source::Object(object),
        })
    }
}
