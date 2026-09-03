//! # `canvas::clicking` — what a click MEANS
//!
//! ## The seam
//!
//! Split out of [`crate::canvas::interact`] on 2026-08-20 under R2, when the
//! Shift constraints took that file past the 1,500-line ceiling. It is the seam
//! the file was always going to split along rather than the cheapest one to
//! reach: `interact`'s subject is *one frame of canvas interaction* — read the
//! pointer, advance the gesture machine, decompose if a hit test needs it, route
//! the outcome, re-resolve, draw — and this is one of eleven outcomes it routes,
//! carrying a third of its lines.
//!
//! [`crate::canvas::pressing`] already owns the companion question, *what would
//! a press land on*. This owns *what does a completed click do about it*, and
//! the pair is easier to reason about than either was inside `interact`.
//!
//! ## ★★ The whole subject is a LADDER, and its order is the design
//!
//! A click is exactly one thing. Never two. The arms below are tried in order
//! and the first that answers consumes the click:
//!
//! | # | arm | why it is here and not one rung later |
//! |---|---|---|
//! | 1 | **the Node tool** | the most specific: the operator armed a tool whose entire subject is anchors |
//! | 2 | **the text caret** | an armed tool owns the press — this codebase's rule everywhere |
//! | 3 | **an annotation under the pointer** | below every armed tool, above the text fall-through |
//! | 4 | **a text sweep** | the fall-through for Read and Review |
//! | 5 | **a vertex markup** (PolyLine, Polygon) | a click-built shape; mutually exclusive with 4 by armed tool |
//! | 6 | **a sticky note** | the one text annotation placed by a click rather than a drag |
//! | 7 | **a measure pick** | the dimension tools |
//! | 8 | **content selection** | what a click meant before any of the above existed |
//!
//! Rung 3 is the one whose position was got wrong once and cost the operator
//! four reports — see its own comment, which is preserved verbatim below along
//! with the record of a diagnosis that was wrong and the test that caught it.
//!
//! ## conventions: click-selects
//!
//! Corpus: `ui-conventions/click-selects.md`.
//!
//! ★ **Most of that corpus is about a HIT TEST and this module is about
//! ROUTING**, so most rows below name where the rule actually lives rather than
//! claiming it. That is the honest answer and it is not a dodge: C8 — *click
//! priority is stated, not emergent* — is **this module's whole subject**, and
//! it is the row that had no single home until the ladder was extracted into a
//! file whose header could state it.
//!
//! - C1 ink-not-bounding-box: NOT THIS FILE — `ObjectModelProvider::subpath_hits`
//!   and `selection::annot::under_pointer` decide it, and the outstanding
//!   `/Square` case is `OPERATOR_REQUESTS.md` O14 row 8.
//! - C2 unfilled-interior-belongs-behind: NOT THIS FILE — same two places, same
//!   open row.
//! - C3 topmost-wins: `super::input::probe` returns the front-most target and
//!   rung 8 hands it to `SelectionState::click` unchanged; rung 3's
//!   `under_pointer` walks `/Annots` in paint order for the same reason.
//! - C4 tolerance-in-screen-units: `map.tolerance()` — the frame's one mapping,
//!   so it is constant at every zoom. Passed, never re-derived.
//! - C5 segments-clamp: NOT THIS FILE — the provider's distance tests.
//! - C6 empty-space-deselects: **answered here, twice.** Rung 8's
//!   `SelectionState::click` clears on a miss, and the `annot_hit.is_none()`
//!   guard above the ladder clears the annotation selection on a miss in the
//!   modes that have one — which had to be explicit, because an annotation
//!   selection is a second store that rung 8 cannot see.
//! - C7 drawn-outline-is-the-live-target: NOT THIS FILE — the painters and
//!   `dimdrag::vertex_at`/`handles::grip_at` own the drawn-vs-target pair, and
//!   that corpus row records the 2026-08-20 defect where they disagreed.
//! - C8 priority-is-stated: **this module.** The eight rungs above, in one
//!   place, in order, each with the reason it sits where it does — and each
//!   reachable only through this one function, so a press cannot mean different
//!   things depending on which path ran first.

use crate::app::actions::forms::FieldAction;
use egui::Pos2;

use crate::app::actions::Action;
use crate::app::modes::Capabilities;
use crate::app::state::OpenDoc;
use crate::canvas::input::probe;

/// ★★★ **How deep into the stack under one point the operator has asked to
/// go**, and where they asked it.
///
/// # What this closes
///
/// The operator, 2026-08-26: *"when I click on one of the objects all I get is
/// the page selected."* The engine already computed the whole front-to-back
/// list of what a click is over; this shell took the first entry and discarded
/// the rest, so anything underneath anything was unreachable at every point,
/// for ever.
///
/// `Alt`+click at the same place now steps one deeper each time and wraps —
/// which is Illustrator's *Select Behind* (`Ctrl`+click there) and Figma's
/// deep-select, the two conventions for exactly this.
///
/// # ★★ Why it resets on pointer travel, and why the threshold is generous
///
/// A depth is only meaningful *at a point*: three clicks in three different
/// places are three first clicks, not a walk into a stack. So the cursor
/// remembers where it was established and resets when the pointer has moved
/// away from there.
///
/// [`CYCLE_RESET_PTS`] is the radius. It is deliberately larger than a pixel:
/// an operator holding `Alt` and clicking repeatedly does not hold the mouse
/// perfectly still, and a one-pixel threshold would silently restart the cycle
/// on the second click and make the feature look broken in the most confusing
/// possible way — it would work, sometimes, depending on how steady their hand
/// was.
#[derive(Clone, Copy, Debug, Default)]
struct CycleCursor {
    /// Where the cycle was established, in canvas space.
    at: egui::Pos2,
    /// How many candidates to skip. `0` is a plain click.
    depth: usize,
}

/// How far the pointer may drift and still be "the same point" for cycling, in
/// canvas points. See [`CycleCursor`].
const CYCLE_RESET_PTS: f32 = 4.0;

/// The `egui::Memory` slot [`CycleCursor`] lives in.
const CYCLE_MEMORY_KEY: &str = "pdfcer-canvas-cycle"; // ui-text-exempt: internal memory id, never displayed

/// **What depth this click means**, advancing or resetting the cursor.
///
/// `alt` is the operator asking to go deeper. Without it the cursor is reset,
/// so an ordinary click always lands on the front-most candidate — which is
/// what makes this feature invisible to anyone not using it.
fn cycle_depth(ctx: &egui::Context, point: egui::Pos2, alt: bool) -> usize {
    let id = egui::Id::new(CYCLE_MEMORY_KEY);
    let previous = ctx
        .data_mut(|d| d.get_temp::<CycleCursor>(id))
        .unwrap_or_default();
    let same_place = previous.at.distance(point) <= CYCLE_RESET_PTS;
    let next = if alt && same_place {
        CycleCursor {
            at: previous.at,
            depth: previous.depth.saturating_add(1),
        }
    } else if alt {
        // First `Alt`+click at a new point: step past the front-most candidate,
        // because a plain click already offers that one and repeating it would
        // make the modifier look inert.
        CycleCursor {
            at: point,
            depth: 1,
        }
    } else {
        CycleCursor {
            at: point,
            depth: 0,
        }
    };
    ctx.data_mut(|d| d.insert_temp(id, next));
    next.depth
}
use crate::canvas::mapping::PageMapping;
use crate::canvas::pick::PickFilter;
use crate::canvas::selection::SelectionState;
use crate::canvas::textsel::TextSelection;
use crate::canvas::tool::CanvasTool;
use crate::panels::objects::provider::ObjectModelProvider;

/// Everything a completed click needs, gathered by the caller.
///
/// The `Frame` shape this codebase already uses for `resizing`, `handledrag`
/// and `dimdrag`, and for the reason those give: the members are read-only
/// facts about one frame, so grouping them says what they are and removes the
/// failure a long parameter list invites — three of the four `bool`s below
/// would compile in each other's places.
///
/// The two things that are **mutated** stay outside it, deliberately: a
/// `Frame` is what the frame knows, and a selection is what the document is.
pub struct Frame<'a> {
    /// The frame's context, for the caret and the pick, both of which store
    /// per-frame state in `egui::Memory`.
    pub ctx: &'a egui::Context,
    /// The open document. Read-only: everything that changes it goes through
    /// `actions`.
    pub doc: &'a OpenDoc,
    /// Which page the click is on.
    pub page_index: usize,
    /// The frame's one screen ⟷ canvas mapping.
    pub map: &'a PageMapping,
    /// The decomposition, if this frame asked for one. `None` is not an error:
    /// `interact` builds it only when the frame's outcome needs a hit test, and
    /// every rung below that consumes it degrades to "nothing was hit".
    pub targets: Option<&'a ObjectModelProvider>,
    /// Which tool is armed. The primary discriminator of the ladder.
    pub active_tool: CanvasTool,
    /// What this mode is allowed to do.
    pub caps: Capabilities,
    /// ★ What the OPERATOR is allowing clicks to land on.
    ///
    /// Beside `caps` because the two compose, and the composition is an
    /// `AND` in one direction only: a mode decides what may be authored,
    /// this decides what is worth pointing at, and switching a class on
    /// here can never grant a capability the mode withholds. See
    /// [`crate::canvas::pick`]'s header.
    ///
    /// Sampled once per frame for the same reason `caps` and `active_tool`
    /// are: a gesture means what it meant when it started.
    pub pick: PickFilter,
    /// The markup pen's current settings, for a vertex markup's click.
    pub pen: crate::canvas::markup::pen::Pen,
    /// Where the click landed, in canvas space.
    pub point: Pos2,
    /// Whether Shift was held **at the press** — extend rather than replace.
    pub shift: bool,
    /// The second click of a double.
    pub double: bool,
    /// The third click of a triple.
    pub triple: bool,
}

/// **Route one completed click.**
///
/// See the module header for the ladder and its order. Raises actions and
/// mutates the two selections; changes no document directly.
pub fn click(
    frame: Frame<'_>,
    selection: &mut SelectionState,
    text_selection: &mut Option<TextSelection>,
    actions: &mut Vec<Action>,
) {
    let Frame {
        ctx,
        doc,
        page_index,
        map,
        targets,
        active_tool,
        caps,
        pick,
        pen,
        point,
        shift,
        double,
        triple,
    } = frame;

    // ★★★ **This frame's Smart-Selector scope**, read once — O70.
    //
    // Once, and passed down, because every picking question below must resolve
    // identically: a press that grabbed a container and a click that selected
    // a leaf would be one gesture acting on two objects. `canvas::smart::Scope`
    // carries the argument for why the pick helpers take a value rather than
    // reading the context themselves.
    let scope = crate::canvas::smart::scope(ctx, page_index);

    // ★★ The annotation under the pointer, resolved BEFORE the ladder.
    //
    // Ahead of it rather than inside it because the arm that consumes
    // this has to be an `if let` — a click that hits nothing must fall
    // through and mean exactly what it meant before this feature
    // existed. See that arm for the full reasoning.
    //
    // # The guard, and why each half of it
    //
    // **`CanvasTool::Select`** — nothing armed. With a pen, a caret or
    // a measure tool armed the press belongs to that tool, which is
    // this codebase's rule everywhere else, so an annotation
    // underneath must not steal it.
    //
    // **`caps.author_markup`** — Review and Edit, not Read. Read is a
    // reader: it may fill a form and sweep text and may not change the
    // document, and selecting a stamp exists in order to act on it.
    //
    // # Cost
    //
    // One `/Annots` walk and one `/PieceInfo` read, **per click** —
    // not per frame. Both are bounded by the number of annotations
    // rather than by document size, and neither decomposes anything:
    // an annotation's geometry is its `/Rect`, four numbers in a
    // dictionary. A click on the 129,758-object benchmark sheet costs
    // the same as a click on a blank page.
    let annot_hit =
        if matches!(active_tool, crate::canvas::tool::CanvasTool::Select) && caps.author_markup {
            crate::canvas::selection::annot::under_pointer(doc, page_index, point, map)
        } else {
            None
        };
    // ★★ The LINK under the pointer, resolved beside `annot_hit` and for the
    // same reason: the arm that consumes it has to be an `if let`, so a click
    // that hits no link falls through and means exactly what it meant before.
    //
    // ★ Hoisted rather than written as a guard in the ladder, which is also how
    // this file's one previous compiler crash was avoided — a `let` chain
    // inside an `else if` at that depth put rustc 1.97 over its own stack. The
    // shape here matches `annot_hit` above, which is where it belonged anyway.
    //
    // The cost of computing it on a click an armed tool will take is one cache
    // comparison: see `crate::app::cache::LinkCache`.
    let link_hit = if caps.edit_content {
        None
    } else {
        crate::canvas::links::under_pointer(doc, page_index, point)
    };
    // A click that missed every annotation, in a mode that could have
    // hit one, **deselects**. Clicking away is the gesture every
    // operator tries first, and without this the outline would survive
    // a click on blank paper — which reads as the selection being
    // stuck rather than as the click having missed.
    if annot_hit.is_none() && caps.author_markup {
        selection.clear_annot();
    }
    // ★ A click is a measure pick, a **text** gesture, or a content
    // selection — never two of them.
    //
    // The text branch asks `super::textsel::takes_the_press` again rather than
    // inferring "the click must be text because the mode cannot select
    // content": `press_kind` reports `click: true` for *two* different
    // reasons, and inferring from the flag would be a second, quieter
    // statement of the rule, free to disagree with the one that decided
    // the drag. One function, two readers — the same shape
    // `active_tool.measure_kind()` already has one line above.
    // ★ …and the **caret** tool takes it before either, which is
    // `press_kind`'s own rung order restated where the click is routed.
    //
    // It has to be restated rather than inferred, for the reason the
    // paragraph below gives about the text branch: `press_kind` reports
    // `click: true` for three different reasons now, and reading the flag
    // would be a second, quieter statement of a rule that is already
    // written. Asking `text_edit_kind()` again is asking the same
    // question of the same value.
    //
    // A refusal is shown rather than swallowed. That is D4a's whole
    // lesson: the old shell's answer to a caret it could not place was a
    // boolean and a keyboard that stopped responding.
    // ★★ **The Node tool takes the click before anything else**, and
    // it is first because it is the most specific: the operator armed a
    // tool whose entire subject is anchors, so a click means an anchor
    // if one is there and "show me this shape's anchors" if not. See
    // `SelectionState::click_direct`.
    //
    // `hit` here already carries the object, the nearest part AND the
    // nearest node, because the probe that produced it is the one a
    // double-click descent uses. That is why this needed no new query:
    // the information the ladder made you perform two gestures to reach
    // was in the very first click all along.
    //
    // ★ **The text tool's kind is decided by the CLICK, not by which
    // ribbon command was pressed**, as of 2026-08-19. `CanvasTool::Text`
    // in a mode that can author now places a caret; `textedit::click`
    // falls back to a fresh origin when the point names no run. The
    // operator's report is why:
    //
    // > *"How do I edit text when on the canvas? I get a box and the I
    // > cursor, but I can't type anything. How do I make new text when I
    // > click on the canvas and expect to edit there? Same problem."*
    //
    // He was getting the I-beam because the text tool SWEPT text, and
    // the tool that types was a different tool reachable only through
    // `Edit ▸ Content ▸ Edit text` — four steps of ritual before a
    // character could be typed, and no surface anywhere saying so. One
    // tool now, click decides, which is Illustrator, Word, Inkscape and
    // every other program he has used.
    let text_kind = active_tool.text_edit_kind().or_else(|| {
        (active_tool.is_text() && caps.edit_content)
            .then_some(crate::canvas::textedit::TextEditKind::Edit)
    });
    if active_tool.is_node() && caps.edit_content {
        let hit = targets
            // Depth 0: the Node tool addresses anchors within an object it
            // has already entered, so "the object underneath" is not a
            // question it asks.
            .map(|t| probe(t, selection, page_index, point, map, pick, 0, scope))
            .unwrap_or_default();
        selection.click_direct(page_index, hit, shift);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "canvas-selection via=node-tool mod={shift} sel={} level={:?} node={:?}",
                selection.len(),
                selection.level(),
                hit.node
            )
        });
    } else if let Some(kind) = text_kind {
        match crate::canvas::textedit::click(
            ctx,
            &crate::canvas::textedit::Click {
                doc,
                page_index,
                kind,
                canvas_point: point,
            },
            actions,
        ) {
            Ok(()) => {}
            Err(refusal) => {
                crate::app::actions::record_note(
                    doc.edit_epoch,
                    crate::text::textedit::refusal(refusal).to_owned(),
                );
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("text-edit-declined reason={refusal:?}")
                });
            }
        }
    // ★★ **AN ANNOTATION UNDER THE POINTER TAKES THE CLICK.**
    //
    // The arm that closes `FEATURES.md`'s *"the canvas selection cannot
    // address an annotation"*, reported by the operator four ways:
    // *"How do I edit a stamp I've applied?"*, *"I still can't get to
    // edit dimension groups when I click on it."*
    //
    // # Why it sits HERE and not one arm earlier or later
    //
    // **Below every armed tool**, because this codebase's stated rule is
    // *"the press belongs to whichever tool is armed"* — an operator who
    // armed the caret, a pen or a measure tool asked for that gesture,
    // and a stamp underneath must not steal it.
    //
    // **Above the text-selection fall-through**, which is the arm that
    // was silently swallowing these clicks. `super::textsel::takes_the_press`
    // is true for the plain Select tool whenever `edit_content` is
    // false — i.e. in **Read and Review** — so in Review, the mode an
    // operator is in *because* they are working on markup, every click
    // on a stamp was being consumed as a text-selection click. Nothing
    // was broken downstream; the click never got there.
    //
    // ★ **My first diagnosis of this was wrong and a test caught it.**
    // I read `press_kind`'s `click: caps.edit_content || text` and
    // concluded Review produced no click event at all. It produces one
    // — `text` is true there for exactly the reason above — and
    // `review_mode_places_markup_but_refuses_content` failed against
    // the "fix", which is the second time this week a test has been the
    // thing that noticed. The predicate was not too coarse; the
    // ROUTING had no arm for annotations.
    //
    // # Why a miss falls through rather than swallowing the click
    //
    // Because this arm must be **additive**. A click that hits no
    // annotation has to mean exactly what it meant before — text in
    // Review, content in Edit — or adding annotation selection would
    // have taken away text selection in the same stroke. `annot_hit`
    // is therefore computed ahead of the ladder and this arm is an
    // `if let`, so a miss is not a branch at all.
    // ★★★ **A LINK UNDER THE POINTER IS FOLLOWED** — the operator's question,
    // 2026-09-01: *"does a clickable table of contents work?"*
    //
    // It did not. There was no link-following code path anywhere in this shell,
    // because until the engine shipped `outline::DestinationReader` a link's
    // destination **could not be read** — `Annotation::action_type` gives the
    // `/S` name and nothing else, deliberately. See `canvas::links`' header.
    //
    // # Why it sits HERE — above the annotation arm and above text
    //
    // **Above `annot_hit`**, because a `/Link` IS an annotation and would
    // otherwise be selected rather than followed in Review, where
    // `caps.author_markup` is true. An operator reading a drawing package in
    // Review expects a contents entry to take them to the sheet, not to put a
    // selection outline round the words.
    //
    // **Above the text fall-through** for the same reason the image arm is:
    // `takes_the_press` is true across the whole canvas in Read and Review, so
    // a rung below it is a rung that never runs.
    //
    // **Below every armed tool**, which is this codebase's standing rule — a
    // caret, a pen or a measure tool was armed on purpose and a link underneath
    // must not steal the press.
    //
    // # ★★ `!caps.edit_content` — Edit SELECTS a link, it does not follow it
    //
    // In Edit a `/Link` is an annotation like any other and the operator is
    // there to move, resize or delete it. A click that navigated away instead
    // would make a link the one annotation in the document that cannot be
    // edited. The same predicate, for the same reason, as `canvas::forms`'
    // fill-versus-author split.
    //
    // # Additive, like the two arms below it
    //
    // A click that hits no link falls through and means exactly what it meant
    // before — an annotation in Review, an image or text in Read.
    // `under_pointer` is an `Option`, so a miss is not a branch at all.
    } else if let Some(link) = link_hit.as_ref() {
        crate::canvas::links::follow(link, doc, actions);
    } else if let Some(hit) = annot_hit {
        selection.select_annot(hit.clone());
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "annot-select page={} id={:?} kind={:?} subtype={} locked={} rect={:?}",
                hit.target.page,
                hit.target.id,
                hit.target.kind,
                hit.target.subtype,
                hit.target.locked,
                hit.outline,
            )
        });
    // ★★★ **AN IMAGE, IN A MODE THAT CANNOT EDIT** — `OPERATOR_REQUESTS.md`
    // O71: *"In read mode the regular pointer should also allow us to select
    // images so we can copy and paste them … outside of the pdfcergui."*
    //
    // # Why it is HERE, one rung above the text fall-through
    //
    // Because it must be **additive**, exactly as the annotation arm two rungs
    // up is: a click that hits no image has to keep meaning what it meant
    // before — text in Read and Review — or adding this would have taken text
    // selection away in the same stroke. It is an `if let` over a hit computed
    // on the spot, so a miss is not a branch at all.
    //
    // Above the text arm rather than below it because `takes_the_press` is true
    // for the whole canvas in these modes: a rung below it is a rung that never
    // runs. That is the same mistake the annotation arm's comment records, and
    // this is the second feature to have been at risk from it.
    //
    // # ★★ IMAGES ONLY, and the narrowness is the decision
    //
    // The filter is one class. He asked for images, and the reason to hold the
    // line there is Read mode's whole promise: a click means *read*, and every
    // additional selectable class is a click that stops sweeping text. A path
    // on a CAD sheet is under the pointer almost everywhere, so allowing paths
    // here would make text selection unreachable on exactly the drawings this
    // program is for.
    //
    // ★ In **Edit** this arm is skipped and the ordinary content ladder below
    // handles images along with everything else — with grips, a marquee and the
    // rest. Two behaviours, one for each stance, rather than one behaviour that
    // is wrong in one of them.
    // ★★★ **…AND TEXT UNDER THE POINTER STILL WINS** — 2026-09-01, hours after
    // the arm above shipped, on the operator's report: *"I can't seem to copy
    // and paste text we have OCRed."*
    //
    // The narrowness chosen above — images only — was designed against a CAD
    // sheet, where a path is under the pointer almost everywhere and allowing
    // paths would have made text unreachable. It does nothing for the case that
    // actually broke: **a scanned page IS one image**, edge to edge, so every
    // click hits it, and an OCR layer is invisible text sitting exactly on top.
    // The one document class where text selection matters most is the one where
    // this arm swallowed it.
    //
    // ⇒ So the image only takes the press where there is **no text under the
    // pointer**. That is what every reader does — a cursor over a word, a
    // pointer over the picture — and it makes the arm safe on a scan by
    // construction rather than by choosing the right filter.
    //
    // ★ `word_at` is asked of the SAME `PageText` the sweep below would use, so
    // the two cannot disagree about whether there is a word here. A second
    // opinion computed differently would produce a click that selects the image
    // and a drag that selects text, from one pixel.
    } else if !caps.edit_content
        && !text_under(doc, page_index, point)
        && let Some(t) = targets
        && let Some(image) = crate::canvas::input::topmost(
            t,
            page_index,
            point,
            map,
            crate::canvas::pick::PickFilter::none()
                .with(crate::canvas::pick::PickClass::Image, true),
            scope,
        )
    {
        // ★ The text selection goes, because the operator has just said they
        // mean the picture. Leaving a swept range behind would make the next
        // `Ctrl+C` ambiguous — and `canvas::clipboard::text_owns_the_chord`
        // resolves that ambiguity in text's favour, so the copy would silently
        // be of the words rather than of the image they just clicked.
        *text_selection = None;
        selection.select_only(page_index, image, "read-image");
    } else if super::textsel::takes_the_press(active_tool, caps) {
        if let (Some(page_text), Some(page)) = (doc.page_text(), doc.pages.get(page_index)) {
            // ★ The SAME options the extraction ran with, through the funnel —
            // `textsel::PageContext::opts` documents why a bare
            // `ExtractOptions::default()` here would be a defect rather than a
            // shortcut.
            let text_ctx = super::textsel::PageContext {
                text: &page_text,
                page,
                index: page_index,
                epoch: doc.edit_epoch,
            };
            *text_selection = super::textsel::click(
                &text_ctx,
                text_selection.as_ref(),
                point,
                shift,
                double,
                triple,
            );
            // `via=` names the gesture rather than the result, so a
            // harness can tell a double-click that happened to cover
            // one word from a sweep that did.
            let via = if triple {
                "line"
            } else if double {
                "word"
            } else if shift {
                "extend"
            } else {
                "clear"
            };
            super::trace::text_selection(page_index, text_selection.as_ref(), via);
        }
    // ★ A **vertex markup** click — PolyLine and Polygon, whose whole
    // gesture is clicks.
    //
    // It sits beside the measure branch rather than inside the markup
    // wiring further down, because the thing being routed is a *click*
    // and this is the arm that routes clicks. The two are mutually
    // exclusive by construction — one armed tool per frame — so the
    // order between them is a statement rather than a tie-break, and
    // `gesture::press_kind` has already given this press a live click
    // and no drag, so there is no state in which one press both places a
    // vertex and replaces the selection.
    //
    // Note what it does NOT need: a decomposition. A vertex lands where
    // the operator clicked and hit-tests nothing, which is why
    // `needs_targets` above grew no term for it — a polygon drawn over
    // the 129,758-object benchmark sheet decomposes nothing.
    } else if let Some(kind) = active_tool.markup_kind().filter(|k| k.is_vertex()) {
        super::markup::vertex::click(
            pen,
            ctx,
            kind,
            page_index,
            point,
            double,
            doc.current_page(),
            actions,
        );
    } else if matches!(active_tool, crate::canvas::tool::CanvasTool::Place(_)) {
        // ★★ A CLICK places the corner and leaves the size to the window that
        // asked — `OPERATOR_REQUESTS.md` O66.
        //
        // Lower-left rather than centre, which is the Form arm's rule below and
        // is chosen for its reason: it matches what the DRAG does, so the two
        // gestures agree about what the pointer meant and an operator who
        // switches between them is not surprised.
        //
        // ★ Unlike every other arm here this commits nothing. The answer goes
        // to `egui::Memory` and the dialog reads it back — the operator has not
        // pressed Insert yet and may still change the numbers.
        //
        // ★★★ The page is passed because the conversion belongs INSIDE
        // `placing` — see its `click`. The first version of this line handed
        // over a canvas point and the placement came out mirrored in y.
        if let Some(page) = doc.current_page() {
            crate::canvas::placing::click(ctx, page, point);
        }
    } else if let crate::canvas::tool::CanvasTool::Form(kind) = active_tool {
        // ★★ A CLICK places a form control at its conventional size.
        //
        // The operator, 2026-08-26: *"I should be able to click on the canvas
        // to place the position or drag a box for size"*. Both, and this is the
        // first half.
        //
        // Unlike the sticky note one arm below, the size here is a REAL promise
        // about what is drawn: a `/Widget`'s `/Rect` is its extent, not a
        // discarded hint, so a 14 pt square really is a 14 pt check box. That is
        // why the numbers live on the kind with their reasoning
        // (`FormFieldKind::default_size_pt`) rather than being one shared
        // constant — a check box and a text field are not the same shape, and
        // sizing them alike would make every click need a resize afterwards.
        //
        // The click point is the LOWER-LEFT corner rather than the centre. Both
        // are defensible; lower-left is chosen because it matches what the drag
        // does — the press is one corner and the control grows from it — so the
        // two gestures agree about what the pointer meant.
        if let Some(page) = doc.current_page()
            && let Some((at, _)) = super::markup::band::endpoints(point, point, page)
        {
            let (w, h) = kind.default_size_pt();
            actions.push(
                FieldAction::Begin {
                    page: page_index,
                    kind,
                    rect: pdfcer_core::page_tree::Rect {
                        llx: at.0,
                        lly: at.1,
                        urx: at.0 + w,
                        ury: at.1 + h,
                    },
                }
                .into(),
            );
        }
    } else if let crate::canvas::tool::CanvasTool::TextAnnot(kind) = active_tool {
        // ★ The STICKY's whole placing gesture: one click, one point.
        //
        // The dragged kinds reach the dialog through
        // `GestureOutcome::TextAnnot` instead, so this arm is the
        // sticky's alone — but it is written for the family rather than
        // for the variant, and guarded by `is_dragged`, so a second
        // click-placed kind added later takes this path without an
        // edit and a kind that stops being click-placed leaves it.
        //
        // The rect is a small square around the point. A `/Text`
        // marker is fixed-size and `NoZoom` — the format discards the
        // rect's extent — so the size here is not a promise about what
        // is drawn; what matters is the LOWER-LEFT corner, which is
        // where the marker lands. `STICKY_PT` is documented at its
        // definition for exactly that reason.
        if !kind.is_dragged()
            && let Some(page) = doc.current_page()
            && let Some((at, _)) = super::markup::band::endpoints(point, point, page)
        {
            actions.push(Action::BeginTextAnnot {
                page: page_index,
                kind,
                rect: pdfcer_core::page_tree::Rect {
                    llx: at.0,
                    lly: at.1,
                    urx: at.0 + crate::canvas::textannot::STICKY_PT,
                    ury: at.1 + crate::canvas::textannot::STICKY_PT,
                },
            });
        }
    } else if let Some(kind) = active_tool.measure_kind() {
        super::measure::click(
            super::measure::Pick {
                ctx,
                doc,
                page_index,
                kind,
                canvas_point: point,
                // ★ The double-click travels to the pick rather than
                // being re-read there. It is the radius/diameter tool's
                // **ending** — the gesture has no natural one, so the
                // operator supplies it — and it is carried on the same
                // value as the click it belongs to for the reason every
                // other field is: one click, one complete statement.
                double,
                targets: targets.map(|t| t as &dyn super::target::CanvasTargetProvider),
                map,
            },
            actions,
        );
    } else {
        // ★★★ `Alt`+click reaches PAST whatever is on top. See [`CycleCursor`].
        //
        // The depth is computed here rather than inside `probe`, because it is
        // a fact about this gesture — how many times the operator has asked, at
        // this point — and `probe` is a pure question about a point. Keeping
        // the cursor at the gesture end means the node-tool branch above, which
        // asks the same question for a different purpose, is unaffected: it
        // passes `0` and behaves exactly as it always has.
        let alt = ctx.input(|i| i.modifiers.alt);
        let depth = cycle_depth(ctx, point, alt);
        let hit = targets
            .map(|t| probe(t, selection, page_index, point, map, pick, depth, scope))
            .unwrap_or_default();
        // ★ How many there were, so the status line can say *"2 of 5 here"*
        // rather than leaving the operator to discover a stack by cycling into
        // it. Computed only when something is under the pointer — the count is
        // a second walk of the same list and there is no reason to pay for it
        // on a click that hit nothing.
        let under = targets
            .filter(|_| hit.object.is_some())
            .map(|t| {
                crate::canvas::input::candidate_count(
                    t,
                    page_index,
                    point,
                    map.tolerance(),
                    pick,
                    scope,
                )
            })
            .unwrap_or(0);
        // ★★★ **A DOUBLE-CLICK ON A CONTAINER GOES INSIDE IT** —
        // `OPERATOR_REQUESTS.md` O70, and Inkscape's group context, which is
        // the convention the operator named.
        //
        // Placed before `selection.click` rather than inside it because the act
        // is not a selection change at all: it changes the SCOPE that the next
        // click resolves in, and then makes an ordinary selection under the new
        // scope. `SelectionState` has no business knowing what a form XObject
        // is, and `canvas::smart`'s header carries the argument for why the
        // container is a scope rather than a fourth `SelectionLevel`.
        //
        // ★ Re-probed with the entered scope rather than reusing `hit`, which
        // resolved to the container by construction. One rule, applied twice,
        // instead of a second path that reaches inside a form its own way.
        if double
            && scope.enabled
            && let Some(t) = targets
            && let Some(object) = hit.object
            && object.page_object_index().is_some()
            && scope.entered != Some(object.raw())
            && crate::canvas::target::CanvasTargetProvider::object_class(t, page_index, object)
                == Some(crate::canvas::pick::PickClass::FormXObject)
        {
            let slot = crate::pagedrag::active(ctx).unwrap_or_default().slot;
            crate::canvas::smart::enter(
                ctx,
                crate::canvas::smart::Entered {
                    page: page_index,
                    form: object.raw(),
                    slot,
                },
            );
            let inside = crate::canvas::smart::Scope {
                enabled: true,
                entered: Some(object.raw()),
            };
            let hit = probe(t, selection, page_index, point, map, pick, 0, inside);
            // ★ `false` for `double`: the operator's double-click was spent on
            // ENTERING. Passing it on would descend into whatever is inside on
            // the same gesture, which is two rungs for one act and is not what
            // Inkscape does either.
            selection.click(page_index, hit, shift, false);
            super::trace::selection_event(selection, "enter-form", true);
            return;
        }
        // ★★★ **A DOUBLE-CLICK ON TEXT EDITS THE TEXT** —
        // `OPERATOR_REQUESTS.md` O70: *"Selecting a text box or similar item
        // does the same thing, but double-clicking inside the bounding box
        // should edit the text."*
        //
        // Below the container arm, deliberately: a label inside a title block
        // is reached by entering the block first and editing on the next
        // double-click, which is the same chain the operator described rather
        // than a shortcut past it.
        //
        // ## ★★ It ARMS the caret tool, and that is the convention rather than
        // ## a side effect
        //
        // Inkscape's selector switches to the text tool on this gesture, and
        // Illustrator's does the same. Placing a caret without arming would
        // type correctly — `textedit::keys::typing` runs whatever tool is
        // selected — and would leave the operator in a state no other program
        // has: a caret in the page while the arrow is still the tool, so their
        // next click means *select* when everything about the screen says they
        // are typing.
        //
        // ★ `tool::select` rather than `arm_text_edit`, and the difference
        // matters here: the latter TOGGLES and calls `textedit::abandon`, which
        // would put away the caret this arm is about to place.
        if double
            && caps.edit_content
            && let Some(t) = targets
            && let Some(object) = hit.object
            && crate::canvas::target::CanvasTargetProvider::object_class(t, page_index, object)
                == Some(crate::canvas::pick::PickClass::Text)
        {
            crate::canvas::tool::select(
                ctx,
                crate::canvas::tool::CanvasTool::TextEdit(
                    crate::canvas::textedit::TextEditKind::Edit,
                ),
            );
            match crate::canvas::textedit::click(
                ctx,
                &crate::canvas::textedit::Click {
                    doc,
                    page_index,
                    kind: crate::canvas::textedit::TextEditKind::Edit,
                    canvas_point: point,
                },
                actions,
            ) {
                Ok(()) => crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "canvas-double-click-text via=descend".to_owned()
                }),
                Err(refusal) => {
                    // ★ The refusal is the operator's, not the trace's alone.
                    // A double-click that opened no caret and said nothing
                    // would read as a text object that cannot be edited, which
                    // is a different and more discouraging claim than the one
                    // `textedit` is actually making.
                    crate::app::actions::record_note(
                        doc.edit_epoch,
                        crate::text::textedit::refusal(refusal).to_owned(),
                    );
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        format!("text-edit-declined reason={refusal:?} via=double-click")
                    });
                }
            }
            return;
        }
        // ★★ **A CLICK OUTSIDE THE CONTAINER LEAVES IT** — O70, and Inkscape's
        // other way out.
        //
        // "Outside" is answered by the substitution that has already happened:
        // with a scope in force, anything inside the entered container resolves
        // to ITSELF and anything inside a different one resolves to that other
        // container. So a hit that is neither the entered form nor one of its
        // leaves is, by construction, elsewhere — and a click on blank paper
        // (no hit at all) is elsewhere too.
        //
        // ★ Written as a question about the RESOLVED target rather than as a
        // bounds test against the form's rectangle. A title block is usually a
        // hollow shape spanning the sheet, so "inside its bounding box" is true
        // of most of the drawing and would trap the operator in a container
        // they had visibly clicked out of.
        if let Some(form) = scope.entered {
            let inside = hit.object.is_some_and(|object| {
                object.raw() == form && !object.is_leaf()
                    || targets.is_some_and(|t| {
                        t.containing_form(page_index, object)
                            == Some(crate::panels::objects::provider::TargetId::Object(form))
                    })
            });
            if !inside {
                crate::canvas::smart::leave(ctx);
            }
        }
        selection.click(page_index, hit, shift, double);
        // ★ Recorded WITH the object it is about, so it cannot be claimed for
        // a selection that arrived some other way — see `canvas::depth::taken`.
        if let Some(object) = hit.object {
            crate::canvas::depth::remember(ctx, depth, under, page_index, object);
        }
        super::trace::selection_event(selection, "click", double);
        if under > 1 {
            crate::diag::trace(move || {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("canvas-pick-depth depth={depth} of={under} alt={alt}")
            });
        }
    }
}

/// **Is there a word under this point?**
///
/// # ★★★ The one question that keeps the Read-mode image arm off a scan
///
/// Added 2026-09-01 on the operator's report — *"I can't seem to copy and paste
/// text we have OCRed"* — hours after the image arm shipped. That arm was
/// narrowed to images because a CAD sheet has a path under the pointer almost
/// everywhere and allowing paths would have made text unreachable. The case it
/// did not anticipate is the one where the narrowing does not help at all: **a
/// scanned page IS one image**, edge to edge, so every click hits it, and an OCR
/// layer is invisible text lying exactly on top of it.
///
/// ⇒ The document class where selecting text matters most was the one where the
/// arm swallowed it.
///
/// ## ★★ Asked of the SAME `PageText` the sweep would use
///
/// Not of a second extraction and not of a cached copy taken elsewhere. Two
/// extractions under two configurations segment differently, so a second
/// opinion here would produce a click that takes the image and a drag that
/// takes text — from one pixel, with nothing on screen to explain it.
///
/// ## ★ Absent page text answers `false`, which yields to the image
///
/// That is the honest direction. `page_text` is `None` before the extraction has
/// run for this page; treating "I do not know yet" as "there is text here" would
/// make the image unclickable for the first frames after a page turn, which is
/// the flicker an operator reports as *"sometimes it does not work"*.
fn text_under(doc: &OpenDoc, page_index: usize, point: egui::Pos2) -> bool {
    let (Some(page_text), Some(page)) = (doc.page_text(), doc.pages.get(page_index)) else {
        return false;
    };
    let ctx = super::textsel::PageContext {
        text: &page_text,
        page,
        index: page_index,
        epoch: doc.edit_epoch,
    };
    super::textsel::word_at(&ctx, point).is_some()
}
