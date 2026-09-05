//! # `canvas::notepopup` — reading a comment where the comment is
//!
//! The window that opens when an operator clicks a note on the page, and the
//! tooltip that appears when they hover one. **The canvas half of the review
//! surface**, and the half that was missing.
//!
//! ## ★★★ The report this closes, and what measuring it found
//!
//! The operator, 2026-09-05:
//!
//! > *"check how the review functions work. unless something has changed I
//! > could add a yellow sticky note but even in read mode I don't think I
//! > could figure out how to read it. the review features should look and act
//! > the same as they do in Acrobat Reader."*
//!
//! Three facts, each verified against this tree before a line was written:
//!
//! 1. **Nothing in `canvas/` displayed an annotation's `/Contents`.** A note
//!    could be placed, dragged, resized, rotated, styled, copied and deleted.
//!    It could not be *read*.
//! 2. **`pdfcer-core` had been writing a `/Popup` for every sticky note all
//!    along** — `annot_author.rs:3223`, with its `/Open` at `:3224` and a
//!    rectangle 150 pt wide beside the note at `:3217-3222`. The data was in
//!    his files. No shell had ever drawn it.
//! 3. **In Read mode there was no route to a comment at all.** The only one
//!    that existed was the Comments panel, whose command is `markup.comments`
//!    on the Markup tab, and `crate::app::modes::defaults`' `"read"` arm gives
//!    Read the tab list `["file", "view"]`.
//!
//! ★★★ Point 3 is the one to feel the weight of. **Acrobat *Reader* is a
//! read-only product and reading comments is its whole purpose.** A mode named
//! Read that cannot read the comments has the posture exactly backwards.
//!
//! ⇒ Which decides where this lives. A pop-up on the canvas is **canvas
//! behaviour, not a ribbon item**, so it is mode-independent *by
//! construction*: no future edit to a tab list, a manifest or a panel default
//! can take it away again. That property is the whole reason this was built
//! before the panel work, and it is why it is not a fourth panel.
//!
//! ## The interaction, and every part of it is the convention rather than an
//! invention
//!
//! This project's standing rule — *"use the conventional interaction, never
//! invent one — the convergence of the product class IS the spec"*:
//!
//! | gesture | what happens | why that |
//! |---|---|---|
//! | **hover** a comment | a tooltip with the author and the words | the cheap half of the same question, in every reader in the class |
//! | **click** a comment | its pop-up opens; clicking it again closes it | the convergent gesture. A *drag* moves the annotation and is a different gesture entirely — egui reports a click only when press and release land together — so the two cannot collide |
//! | **× on the pop-up** | closes it | every window in the class |
//! | a note the file marks `/Open` | **opens with the document**, no click | §12.5.6.4 Table 172 and §12.5.6.14 Table 183 both say so, and the state is in the file |
//!
//! ★ **A single click rather than a double.** In a reader, one click opens the
//! note; in an editor with the comment tool armed, one click selects and two
//! open. pdfcer has to serve both stances from one canvas, and a single click
//! serves both because opening a pop-up **does not consume the click**: in
//! Review and Edit the same press still selects the annotation, so the
//! selection outline, the grips and the Format tab all behave exactly as they
//! did. Nothing was taken away to add this.
//!
//! ## ★★★ Rule 4: the pop-up is CHROME, and the page is untouched
//!
//! *"Fuzzy never sneaky"*, and the one-line test this project uses for it:
//! **would a screenshot of the editing canvas differ from a screenshot of the
//! same document saved and reopened?**
//!
//! A pop-up is the same class of thing as a selection handle or a snap marker
//! — the cursor's own furniture. So:
//!
//! - It is drawn in an `egui::Area`, a **separate layer** above the page
//!   raster. Not one pixel of it is composited into anything that is saved.
//! - It is drawn at a **fixed size in screen points and does not scale with
//!   zoom**, which is what makes it unmistakably interface rather than
//!   content. Acrobat's pop-up behaves the same way and for the same reason.
//!   ★ Its *position* does follow the page, because it is about a particular
//!   note; its *size* does not, because it is about the operator's eyes.
//! - Nothing it shows is inferred. The words, the byline and the date are
//!   verbatim from the file; the open state is read from `/Open`, never
//!   defaulted (see [`model::read_open`], and the request filed beside it).
//! - **Closing a pop-up changes the screen, not the file** — and
//!   [`crate::text::annotpopup::popup_close_tooltip`] says so, because an
//!   operator has every reason to assume otherwise.
//!
//! ## What it can do, by mode
//!
//! | | Read | Review | Edit |
//! |---|---|---|---|
//! | hover tooltip | ✅ | ✅ | ✅ |
//! | open the pop-up and read the note | ✅ | ✅ | ✅ |
//! | read the thread of replies | ✅ | ✅ | ✅ |
//! | edit the note, remove it, delete the comment | — | ✅ | ✅ |
//!
//! ### ★★★ Read shows and does not edit, and the reason is on screen
//!
//! `MODES_AND_PANELS.md`'s stance for Read is *"the page content is not yours
//! to alter"*, and **reading is not editing** — which is the whole argument
//! for this module existing. So the editor is not drawn in Read.
//!
//! It is not drawn *greyed*, either. R9: *"an unavailable capability renders
//! nothing; a temporarily unavailable one may grey and must explain on
//! hover."* Read mode is the purest example of temporary this program has —
//! the operator chose a stance and a labelled three-position control changes
//! it — so the pop-up carries **one sentence naming the mode that can**
//! ([`crate::text::annotpopup::popup_read_only`]) and no dead text box. A
//! disabled `TextEdit` would be the half-built surface the no-placeholders
//! rule exists to forbid.
//!
//! ## ★★ What it CANNOT do, and both absences are engine gaps that are filed
//!
//! Audited against `pdfcer-core` v0.38.0 on 2026-09-05, at `file:line`, not
//! from a backlog row:
//!
//! - **No Reply.** `/IRT` and `/RT` are read-only. The crate's only write-side
//!   occurrences are *destructive*: the deletion cascade removes them
//!   (`edit.rs:24969-24970`) and the clipboard strips them
//!   (`edit.rs:10673`). There is no constructor, no `MarkupOptions` field and
//!   no verb. ⇒ the thread is **read** here and cannot be **added to**. Filed
//!   as `request_a_reply_can_be_read_and_never_written.md`.
//! - **No Accepted / Rejected / Completed.** `/State` and `/StateModel`
//!   (§12.5.6.4 Table 171) have **zero occurrences** in the crate — not read,
//!   not written, not modelled, not named in a doc comment. Filed as
//!   `request_review_status_is_not_modelled_at_all.md`.
//!
//! In both cases R9 governs and **nothing is drawn**: no greyed Reply, no
//! empty status row. A control that no state of the program could enable is
//! not an affordance, it is a promise.
//!
//! ## Where the pieces are
//!
//! | | |
//! |---|---|
//! | [`model`] | the pure read — what notes are here, where their windows go, what replies hang off them |
//! | [`open`] | which pop-ups are showing, and the override rule that lets the file speak first |
//! | this file | the drawing, the two hooks, and the trace |
//!
//! ## The two hooks, and how small they are
//!
//! [`show`] is called from `crate::app::surfaces` on the line after
//! `canvas::show` returns — **one statement** — because a floating layer does
//! not belong in the canvas paint order at all (`crate::canvas::painting`'s
//! header states that order and every position in it is an argument; a pop-up
//! has no position in it). [`clicked_on`] is called from
//! `crate::canvas::clicking` beside the annotation hit test — **one
//! statement**, consuming nothing.
//!
//! ★ [`show`] takes this frame's mapping from `crate::canvas::zoom::last_frame`
//! rather than being handed one. `canvas::present` publishes it through
//! `remember_frame` **before** it calls `interact`, so by the time this runs
//! it is this frame's map and not the previous one — which is what keeps the
//! window from lagging a pan by a frame.
//!
//! ## ★★★ A pop-up NEVER covers the annotation it belongs to — the invariant
//! this module gained on its second day
//!
//! It shipped 2026-09-05 without one, and within hours the first full driven
//! sweep filed *"an annotation can be ROTATED and cannot be MOVED or
//! RESIZED"* against the canvas. The canvas was fine. `Area::constrain_to`
//! had slid a pop-up that did not fit to the right of its note **back on top
//! of that note**, and an `egui::Area` at `Order::Middle` takes every press
//! inside it — so the move drag and the grip drag never reached the canvas at
//! all, while the rotate handle, which is drawn clear of the box, kept
//! working.
//!
//! [`popup_origin`] now flips rather than slides, and [`clear_of_anchor`]
//! carries the candidate order, the measurement and the one case that has no
//! answer. ⇒ **A window that describes a thing must not be laid over the
//! thing**, and on an immediate-mode canvas that is not a cosmetic rule: the
//! window is an input surface, and the thing underneath becomes unreachable.
//!
//! ## ⚠ Known limit, named rather than left to be found
//!
//! ★ **CLOSED 2026-09-05, the same day it was named.** A comment with no words
//! used to open an empty pop-up on a click, because [`model::under`] answers
//! for every annotation that *can* carry a note rather than for those that do —
//! noise on a shape the operator only meant to select. It was recorded here and
//! on `OPERATOR_REQUESTS.md` O133 as *"a question about WHEN a pop-up opens
//! rather than about where it goes"*, which was the right description and is
//! now answered: [`model::has_something_to_read`] is asked at the click site.
//!
//! The rule is **not** simply "has words" — a sticky note is a note whether or
//! not anybody has typed in it, and an operator who has just placed one needs
//! the window in order to write. Subtype decides for the two whose purpose is
//! the note; content decides for every mark that merely *may* carry one.
//!
//! Under a continuous or facing display mode, pop-ups are drawn for the
//! **acting page's** annotations only. That is not a decision of this module:
//! it inherits `crate::canvas::selection::annot::under_pointer`'s frame of
//! reference exactly — one `page_index`, one `PageMapping` — so a pop-up
//! appears wherever an annotation is *selectable*, and nowhere else. Fixing it
//! is the same piece of work as making annotation selection reach a second
//! visible page, and doing it here alone would put a window over a note the
//! canvas will not let you click.
//!
//! ## `PDFCER_DIAG` proves what this computed
//!
//! One `note-popup` line per frame with something to say: how many notes the
//! page carries, how many carry words, how many pop-ups are open, how many of
//! those the **file** asked for rather than the operator, and whether a
//! tooltip was shown. A screenshot cannot tell you that a pop-up opened
//! because `/Open` was true rather than because a click landed, and that
//! distinction is the whole of [`open`]'s contract.

/// Turning a document into notes and their windows — testable without a `Ui`.
pub mod model;

/// Which pop-ups are showing, and who decided.
pub mod open;

use egui::{Pos2, Rect};
use pdfcer_core::object::ObjId;

use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::app::modes::Capabilities;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::panels::comments::note::NoteDraft;
use crate::text::annotpopup as t;
use crate::text::panels::comments as tp;

use self::model::NoteView;

/// **The region an open pop-up publishes**, for the first open pop-up and only
/// that one.
///
/// # ★★ Why the first, when several can be open at once
///
/// A region name is a key, and publishing one name from four windows would
/// leave a driven check clicking whichever happened to be drawn last — a
/// coordinate nobody chose, which moves when an unrelated note is opened. The
/// first is the only deterministic choice available without inventing a
/// per-annotation naming scheme that nothing would consume.
///
/// ★ It is published with `ui_rect_visible` against the **canvas viewport**,
/// not with `ui_rect`. `REVIEW_TRIAGE.md` T4 records three panels shipping
/// **unreachable in real builds with every gate green**, because every driven
/// assertion about them proved *layout* rather than *visibility*. A pop-up
/// constrained off the edge of the canvas would lay out perfectly and be
/// invisible, which is precisely that failure in a new place.
pub const REGION_POPUP: &str = "notepopup.window"; // ui-text-exempt: trace region name, never displayed
/// The region an open pop-up's close control publishes.
pub const REGION_CLOSE: &str = "notepopup.close"; // ui-text-exempt: trace region name, never displayed
/// The region the *Add note* / *Edit note* control publishes.
pub const REGION_EDIT: &str = "notepopup.edit"; // ui-text-exempt: trace region name, never displayed
/// The region the open editor's text box publishes.
pub const REGION_BOX: &str = "notepopup.box"; // ui-text-exempt: trace region name, never displayed
/// The region the open editor's *Save note* publishes.
pub const REGION_SAVE: &str = "notepopup.save"; // ui-text-exempt: trace region name, never displayed
/// The region *Delete comment* publishes.
pub const REGION_DELETE: &str = "notepopup.delete"; // ui-text-exempt: trace region name, never displayed

/// The pop-up's width, in **screen points**.
///
/// # ★★ Screen points, not page points, and that is the rule-4 half
///
/// A pop-up sized in page space would grow to fill the sheet at 800 % and
/// vanish at 20 %, which is what *content* does. Chrome does not. 260 pt is
/// about forty characters of the shell's body face — wide enough for a
/// sentence of review prose without wrapping every third word, narrow enough
/// that four open pop-ups on a D-size sheet do not tile over the drawing.
///
/// ★ The `/Popup`'s own `/Rect` is still honoured **for position** (see
/// [`popup_origin`]). Its width is not, deliberately: `pdfcer-core` authors
/// 150 pt (`annot_author.rs:3217-3222`) which at 100 % zoom is under
/// twenty-five characters, and a producer's chosen width is a statement about
/// their reader's font rather than about ours.
const POPUP_WIDTH: f32 = 260.0;

/// The tallest a pop-up's body may grow before it scrolls, in screen points.
///
/// A note is arbitrary operator text and can be a page of it. Without a
/// ceiling one long comment would produce a window taller than the canvas,
/// whose Save button is off screen — the exact defect `RESUME.md` records the
/// Print dialog shipping four times over. The body scrolls; the title row, the
/// byline and the controls never do, so the two things an operator needs
/// (whose note is this, and how do I close it) are always in view.
const POPUP_MAX_BODY: f32 = 220.0;

/// How far a pop-up sits from its note when the file gives no `/Popup` rect.
///
/// To the right of the annotation's box, which is where `pdfcer-core`'s own
/// author places one and where every reader in the class puts it. Eight points
/// of gap so the window does not touch the mark it belongs to — a pop-up flush
/// against a cloud reads as part of the drawing.
const POPUP_GAP: f32 = 8.0;

/// **Draw every open pop-up, and the hover tooltip.**
///
/// The one entry point, called from `crate::app::surfaces` immediately after
/// `crate::canvas::show` returns. See the module header for why it is there
/// and not in the paint pass.
///
/// # Why it takes `caps`
///
/// To decide whether the editor is drawn at all — see the module header's mode
/// table. It is the frame's sampled value, passed in rather than read here,
/// for the reason every canvas sample is: two readings within one frame can
/// disagree, and a disagreement here would be an editor that appeared for one
/// frame.
///
/// # Why it takes `&OpenDoc` and `&mut Vec<Action>`
///
/// Actions, not mutations — the discipline every panel and every canvas
/// gesture in this crate follows. This function reads the document and pushes
/// intent; it never touches the session. The shared reference makes that a
/// compile-time fact rather than a convention.
pub fn show(ctx: &egui::Context, doc: &OpenDoc, caps: Capabilities, actions: &mut Vec<Action>) {
    // This frame's map and viewport, published by `canvas::present` before it
    // called `interact`. `None` on a frame with no canvas at all — no
    // document, or a render failure that replaced the strip with a sentence —
    // in which case there is nothing to hang a pop-up off and nothing to draw.
    let Some(frame) = crate::canvas::zoom::last_frame(ctx) else {
        return;
    };
    let page_index = doc.view.page_index;
    let Some(page) = doc.pages.get(page_index) else {
        return;
    };
    // Read the SESSION, not the file on disk — the same rule
    // `crate::panels::comments` states in its header. An operator who has just
    // typed a note must see it in the window they typed it in, without saving.
    let view = doc.session.view();
    let notes = model::notes_on(&view, page);
    if notes.is_empty() {
        return;
    }

    let overrides = open::load(ctx, &doc.path);
    let ce_dimensions = crate::panels::comments::model::ce_dimension_annots(&doc.session);
    let mut drawn = 0_usize;
    let mut from_file = 0_usize;
    // ★ The draft is loaded ONCE and stored back once, rather than read and
    // written per pop-up. Two pop-ups cannot be edited at the same time — the
    // draft names one annotation — and a load-per-window would let the second
    // one see the state the first had already written this frame.
    let mut draft = load_draft(ctx, &doc.path);
    draft.sync(doc.edit_epoch);

    for note in &notes {
        if !overrides.is_open(note.id, note.authored_open) {
            continue;
        }
        if note.authored_open && !overrides.touched(note.id) {
            from_file += 1;
        }
        let published = drawn == 0;
        drawn += 1;
        popup(
            ctx,
            &Ctx {
                doc,
                caps,
                page_index,
                map: &frame.map,
                clip: frame.viewport_rect,
                is_ce_dimension: ce_dimensions.contains(&note.id),
                published,
            },
            note,
            &mut draft,
            actions,
        );
    }
    store_draft(ctx, &doc.path, &draft);

    let tipped = tooltip(ctx, &notes, &overrides, &frame.map, frame.viewport_rect);

    trace(&notes, drawn, from_file, tipped);
}

/// Everything one pop-up needs that is the same for all of them.
///
/// A struct rather than eight parameters, and the grouping is a statement:
/// every member is a property of *the frame*, while the two arguments that
/// stay loose — the note and the draft — are what distinguishes one window
/// from the next.
struct Ctx<'a> {
    doc: &'a OpenDoc,
    caps: Capabilities,
    page_index: usize,
    map: &'a PageMapping,
    clip: Rect,
    /// Whether this note is a **ce dimension** — rule 15. Decided once per
    /// frame in [`show`] against the `/PieceInfo` sidecar, never per pop-up:
    /// `ce_dimension_annots` walks the catalog and deserializes it, and asking
    /// it per window would make the surface O(windows × sidecar).
    is_ce_dimension: bool,
    /// Whether this pop-up is the one that publishes [`REGION_POPUP`].
    published: bool,
}

/// Draw one pop-up.
fn popup(
    ctx: &egui::Context,
    f: &Ctx<'_>,
    note: &NoteView,
    draft: &mut NoteDraft,
    actions: &mut Vec<Action>,
) {
    let origin = popup_origin(note, f.map, f.clip);
    let area = egui::Area::new(egui::Id::new(("pdfcer-note-popup", note.id))) // ui-text-exempt: internal widget id, never displayed
        // ★ `Middle` is egui's own layer for windows, which puts this above
        // the page raster and below tooltips and menus. Not `Foreground`: a
        // context menu opened over a pop-up must still be on top of it, and
        // `Foreground` would invert that.
        .order(egui::Order::Middle)
        .fixed_pos(origin)
        // ★★ Constrained to the CANVAS viewport, not to the window. A pop-up
        // for a note near the right edge of a sheet would otherwise slide out
        // over the docked panels, where it would look like a panel with no
        // title. See `REGION_POPUP` on why this is also what makes the driven
        // check able to fail.
        .constrain_to(f.clip)
        .show(ctx, |ui| {
            ui.set_max_width(POPUP_WIDTH);
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_max_width(POPUP_WIDTH);
                body(ui, f, note, draft, actions);
            });
        });
    if f.published {
        crate::diag::ui_rect_visible(REGION_POPUP, area.response.rect, f.clip);
    }
}

/// The pop-up's **outer** width — the box `Area::constrain_to` has to fit —
/// as distinct from [`POPUP_WIDTH`], which is the width of its *contents*.
///
/// `popup` sets `ui.set_max_width(POPUP_WIDTH)` twice: once on the `Area`'s
/// own `Ui` and once inside `egui::Frame::popup`, whose inner margin and
/// stroke sit **outside** the contents. Measured on a real build the drawn
/// window is 274 pt for a 260 pt content width — fourteen points of frame.
/// Sixteen is used here rather than fourteen because the frame is a *style*
/// value and a theme with a fatter popup margin must not silently reintroduce
/// the overlap [`popup_origin`] exists to prevent. Erring wide costs at most a
/// two-point gap; erring narrow costs the gesture.
const POPUP_BOX_WIDTH: f32 = POPUP_WIDTH + 16.0;

/// **Where a pop-up's top-left corner goes**, in screen space.
///
/// Two sources of a *preferred* origin, in priority order:
///
/// 1. **The `/Popup`'s own `/Rect`**, when the file gives a usable one. The
///    producer said where the window belongs and honouring it is what makes a
///    document look here the way it looked in the reader that wrote it.
/// 2. **Beside the note**, to the right and top-aligned, when there is no
///    `/Popup` or its rectangle is unusable. Where `pdfcer-core`'s own author
///    puts one (`annot_author.rs:3217-3222`) and where every reader in the
///    class puts one.
///
/// …and then one **invariant that outranks both**, added 2026-09-05:
///
/// > ### ★★★ A pop-up must never be laid over the annotation it belongs to
///
/// # The defect this is the fix for, because the reasoning is not obvious
///
/// The first driven sweep (2026-09-05) reported *"an annotation can be ROTATED
/// and cannot be MOVED or RESIZED"* — `dragging_a_markup_moves_it` and
/// `the_line_weight_switch_reaches_the_resize` both FAILED with **no line
/// containing `drag` anywhere in the trace**, while `rotating_a_markup_turns_it`
/// PASSED. Three gestures on one shape, one working. The diagnosis those checks
/// offered — a fork in `canvas::interact` eating the gesture — named a real
/// mechanism and was about nothing.
///
/// What actually happened is in two lines of the trace:
///
/// ```text
/// ui-rect name=canvas.selection-outline rect=[[464.0 464.5] - [551.7 550.2]]
/// ui-rect name=notepopup.window         rect=[[498.0 465.0] - [772.0 565.0]]
/// ```
///
/// The window is **on top of the shape it describes**, and an `egui::Area` at
/// `Order::Middle` takes every press inside it: egui resolves interaction on the
/// topmost layer, so the canvas response never sees the press at all. The drag
/// was not consumed by a canvas fork — it never reached the canvas. Rotation
/// survived only because the rotate handle is drawn *above* the box's top edge,
/// clear of the window.
///
/// # …and the cause was the clamp, which read as harmless
///
/// `beside` puts the origin at `anchor.max.x + POPUP_GAP` = 559.7. The canvas
/// viewport ends at 772, so a 274 pt window does not fit; `Area::constrain_to`
/// then slid it **left** to 498 — back over the anchor. The clamp was doing
/// exactly what it was written to do, and *sliding is the wrong recovery*: the
/// one direction a pop-up must not be pushed is onto its own subject.
///
/// ⇒ **Flip, do not slide.** The candidates below are tried in order and the
/// first that clears the anchor wins:
///
/// | # | candidate | separation |
/// |---|---|---|
/// | 1 | the preferred origin (file, else right of the note) | taken as-is when the box it implies does not intersect the anchor |
/// | 2 | **left** of the note, right-aligned to its left edge | horizontal |
/// | 3 | **below** or **above**, whichever side of the anchor has more room, x pinned into the viewport | vertical |
///
/// ★ Candidates 1 and 2 separate on **x alone**, which makes them independent
/// of the window's height — and the height is the one dimension this function
/// cannot know, because it is decided by the note's own words during layout.
/// A placement that needed the height would have to read the *previous*
/// frame's measured rect, and `D:/dev/rag/egui/` records what that costs: a
/// surface whose position depends on its own size oscillates, and the
/// oscillation is invisible to unit tests and to screenshots alike. Candidate
/// 3 needs a vertical decision and takes it from the **room available**
/// (`clip.max.y - anchor.max.y` against `anchor.min.y - clip.min.y`) rather
/// than from the window's height, for the same reason: room is a property of
/// the page and the viewport, and nothing about it moves when the window does.
///
/// # ⚠ The case that has no answer, named rather than hidden
///
/// When the anchor is wider than the viewport minus a pop-up **and** taller
/// than half of it — an annotation zoomed until it fills the screen — no
/// candidate clears it, and the preferred origin is used unchanged. That is
/// honest: at that zoom every position covers part of the subject, and the
/// operator has the whole rest of the shape to press on. It is stated because
/// the alternative — refusing to draw the pop-up at all — would make a note
/// unreadable at exactly the zoom an operator uses to read one.
///
/// ★ Only the **origin** comes from the file; the size does not. See
/// [`POPUP_WIDTH`] for why, and note the consequence: a pop-up whose `/Rect`
/// is 150 pt wide is drawn 260 pt wide from the same top-left corner, so it
/// extends further right than the file's rectangle. That is correct — the
/// rectangle is where the window *is*, and how big a window needs to be is a
/// property of the reader's typeface.
///
/// ★★ The file's own rectangle is a *preference*, not a licence to overlap.
/// A producer that placed a `/Popup` over its own note is asking for a window
/// the annotation cannot be grabbed through, and honouring that would be
/// honouring a defect. Candidate 1 keeps the file's origin whenever it clears
/// the anchor, which is what every `/Popup` a real producer writes does.
///
/// # ★★ The clamp is still here, and still a fallback
///
/// Whatever candidate wins is clamped into a viewport grown by the window's
/// own size, so that an `Area` is never handed a position off in the millions:
/// at deep zoom a page point maps to a screen coordinate far outside any
/// viewport, and an `Area` positioned there is constrained back to the edge —
/// every pop-up on the sheet stacked in one corner. Clamping into a
/// slightly-grown viewport first means an off-screen note's window arrives at
/// the edge *nearest to it*, which is at least a direction.
fn popup_origin(note: &NoteView, map: &PageMapping, clip: Rect) -> Pos2 {
    let anchor = map.rect_to_screen(note.anchor);
    let preferred = note.popup.and_then(|p| p.rect).map_or_else(
        || Pos2::new(anchor.max.x + POPUP_GAP, anchor.min.y),
        |rect| map.rect_to_screen(rect).min,
    );
    let raw = clear_of_anchor(preferred, anchor, clip);
    // Grown by the width so a window whose origin is just off the right edge
    // is not slammed to the left edge. `constrain_to` does the real work.
    let room = clip.expand2(egui::vec2(POPUP_WIDTH, POPUP_MAX_BODY));
    Pos2::new(
        raw.x.clamp(room.min.x, room.max.x),
        raw.y.clamp(room.min.y, room.max.y),
    )
}

/// Move `preferred` off `anchor` if the window it implies would cover it.
///
/// Separated from [`popup_origin`] so it can be tested without a
/// `PageMapping` — the decision is pure rectangle arithmetic and every
/// interesting case is a specific arrangement of three rectangles, which is
/// exactly the shape a unit test can state and a driven check cannot.
///
/// See [`popup_origin`]'s header for the candidate order and for why the
/// first two separate on **x alone**. `width` is [`POPUP_BOX_WIDTH`]; the
/// height is deliberately not a parameter, because this function must not
/// depend on a quantity that is decided by the window's own contents.
fn clear_of_anchor(preferred: Pos2, anchor: Rect, clip: Rect) -> Pos2 {
    // ★★★ **An anchor that is not on screen cannot be covered**, and the
    // invariant is about a visible one. A note scrolled out of the viewport, or
    // one at 300,000 % zoom whose rect maps into the millions, gets its
    // preferred origin unchanged — because the placement that matters for it is
    // the *direction* it lies in, which `a_note_far_off_screen_is_clamped_
    // towards_itself` asserts and which pinning the window into the viewport
    // would destroy. Two rules, and this one comes first because the other has
    // nothing to protect here.
    //
    // ★ The intersection, not the anchor, is what the rest of this function
    // separates from: for a mark half off the left edge, the half an operator
    // can actually press on is the half inside the clip, and reserving room
    // beside the part they cannot see would push the window off the other side
    // for nothing.
    let anchor = anchor.intersect(clip);
    if !anchor.is_positive() {
        return preferred;
    }
    // A window's x-range, given its left edge. Two windows that do not
    // overlap horizontally cannot overlap at all, whatever their heights.
    let covers_x = |x: f32| x < anchor.max.x && x + POPUP_BOX_WIDTH > anchor.min.x;
    // ★★★ …and it is not enough to clear the anchor: the window must also FIT,
    // because a window that does not is one `Area::constrain_to` will SLIDE —
    // and sliding is the whole defect. `beside` puts the origin to the right of
    // the note, which clears it by construction and then, on any note within a
    // pop-up's width of the right edge, gets pushed straight back on top of it.
    // Testing `covers_x` alone accepted exactly those placements, which is what
    // the first two runs of `a_note_against_the_right_edge_puts_its_window_on_
    // the_left` measured: origin x = 768 in a viewport ending at 772.
    let fits = |x: f32| x >= clip.min.x && x + POPUP_BOX_WIDTH <= clip.max.x;
    let usable = |x: f32| !covers_x(x) && fits(x);
    // 1 — the preferred origin, when it already clears the anchor and fits.
    if usable(preferred.x) {
        return preferred;
    }
    // 2 — flip to the LEFT of the note, right edge against its left edge.
    let left = anchor.min.x - POPUP_GAP - POPUP_BOX_WIDTH;
    if usable(left) {
        return Pos2::new(left, preferred.y);
    }
    // …and the right, which candidate 1 reaches only when the file supplied an
    // origin of its own or when the right-hand placement did not fit. Written
    // out rather than assumed away: with no `/Popup` rect `preferred` IS this
    // position, so the arm is a no-op there and a real choice otherwise.
    let right = anchor.max.x + POPUP_GAP;
    if usable(right) {
        return Pos2::new(right, preferred.y);
    }
    // 3 — no horizontal separation is available. Go under the note, or over
    // it, whichever side of the anchor has more room. `x` is pinned into the
    // viewport so the window is not immediately slid back by `constrain_to`.
    let x = preferred
        .x
        .min(clip.max.x - POPUP_BOX_WIDTH)
        .max(clip.min.x);
    let below = clip.max.y - anchor.max.y;
    let above = anchor.min.y - clip.min.y;
    if below >= above {
        Pos2::new(x, anchor.max.y + POPUP_GAP)
    } else {
        // The **top of the viewport**, not "the anchor's top minus the window's
        // height", and the difference is the header's rule: the height is
        // decided by the note's own words and must not feed back into the
        // note's position. Starting at the top uses every point of the room
        // there is; when the window is shorter than that room it clears the
        // anchor, and when it is longer nothing could have.
        Pos2::new(x, clip.min.y)
    }
}

/// The pop-up's contents: the title row, the byline, the note, the thread and
/// the controls.
///
/// # The order, and why it is this one
///
/// Top to bottom in the order a reviewer needs them: **who and what** (so they
/// know whose comment they opened), **the words** (what they came for), **the
/// thread** (the rest of the conversation), then **the controls** — last,
/// because an operator scanning downward reads and stops when they reach a
/// button.
///
/// The close control is the exception and sits on the title row at the right,
/// which is where every window in the class puts it and where a hand reaches
/// for it without reading.
fn body(
    ui: &mut egui::Ui,
    f: &Ctx<'_>,
    note: &NoteView,
    draft: &mut NoteDraft,
    actions: &mut Vec<Action>,
) {
    // ---- the title row ------------------------------------------------
    let heading = if f.is_ce_dimension {
        t::popup_ce_dimension_heading(&note.subtype)
    } else {
        t::popup_heading(&note.subtype)
    };
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(heading));
        // Right-aligned close. `with_layout` rather than a spacer, because a
        // spacer sized from the heading's width would move when a subtype name
        // changed length.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let close = ui
                .button(t::popup_close())
                .on_hover_text(t::popup_close_tooltip());
            crate::diag::ui_rect_visible(REGION_CLOSE, close.rect, f.clip);
            if close.clicked() {
                open::set(ui.ctx(), &f.doc.path, note.id, false);
                draft.close();
            }
        });
    });

    // ---- who and when -------------------------------------------------
    //
    // ★ `comment_row_byline` is CALLED rather than copied, and that is the
    // point: it carries a settled ruling about `/M` being shown verbatim
    // (§12.5.2 makes it *"date or text string"* and requires a reader to
    // accept any format). Two surfaces showing one comment must not show two
    // different dates for it, and one function is the only way that cannot
    // happen.
    if let Some(byline) = tp::comment_row_byline(note.author.as_deref(), note.modified.as_deref()) {
        let resp = ui.label(egui::RichText::new(byline).small().weak());
        if note.modified.is_some() {
            resp.on_hover_text(tp::comment_row_modified_tooltip());
        }
    }
    ui.separator();

    // ---- the words -----------------------------------------------------
    let existing = note.contents.as_deref().unwrap_or_default();
    let editing = draft.editing(note.id, f.doc.edit_epoch);
    egui::ScrollArea::vertical()
        .id_salt(("pdfcer-note-popup-body", note.id)) // ui-text-exempt: internal widget id, never displayed
        .max_height(POPUP_MAX_BODY)
        .show(ui, |ui| {
            if editing {
                let response = ui.add(
                    egui::TextEdit::multiline(draft.text_mut())
                        .desired_rows(3)
                        .desired_width(f32::INFINITY),
                );
                crate::diag::ui_rect_visible(REGION_BOX, response.rect, f.clip);
                // Escape closes the editor, and it does so through egui rather
                // than by reading the keyboard: a `TextEdit` surrenders focus
                // on Escape, so `lost_focus()` plus the key is the idiomatic
                // test — and, importantly here, it asks nothing about whether
                // "the operator is typing". A canvas surface that read the raw
                // key would be a second claimant on a key the caret and the
                // tool arming both want, which is the class of bug
                // `tools/gates/check-typing-guard.sh` exists for.
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    draft.close();
                }
                ui.label(egui::RichText::new(t::popup_note_hint()).small().weak());
            } else if existing.trim().is_empty() {
                let caption = if f.is_ce_dimension {
                    t::popup_ce_dimension_note()
                } else {
                    t::popup_no_note()
                };
                ui.label(egui::RichText::new(caption).small().weak());
            } else {
                ui.label(t::popup_body(existing));
                if f.is_ce_dimension {
                    ui.label(
                        egui::RichText::new(t::popup_ce_dimension_note())
                            .small()
                            .weak(),
                    );
                }
            }
            thread(ui, f, note);
        });

    // ---- the controls ---------------------------------------------------
    ui.separator();
    controls(ui, f, note, draft, existing, actions);
}

/// The replies hanging off this comment, read from `/IRT`.
///
/// # ★★ Read-only, and that is an engine limit rather than a choice
///
/// `pdfcer_core::annot::Annotation` models `/IRT` and `/RT`, and
/// `EditSession` has no verb that writes either — audited against v0.38.0. So
/// a thread another product authored is legible here and this shell cannot add
/// to it. R9 forbids a greyed Reply button for a capability no state of the
/// program can reach, so **nothing is drawn** where one would go; the gap is
/// filed as `request_a_reply_can_be_read_and_never_written.md`.
///
/// # Cost
///
/// [`model::replies_to`] walks every page, because a reply may legally live on
/// a different page from the comment it replies to. It runs only for a pop-up
/// that is **open**, which is the gate that makes it affordable: closed notes
/// cost nothing at all. `crate::panels::comments` pays a comparable walk on
/// every frame it is visible and states so in its own header.
fn thread(ui: &mut egui::Ui, f: &Ctx<'_>, note: &NoteView) {
    let view = f.doc.session.view();
    let replies = model::replies_to(&view, &f.doc.pages, note.id);
    if replies.is_empty() {
        return;
    }
    ui.separator();
    ui.label(
        egui::RichText::new(t::popup_replies(replies.len()))
            .small()
            .weak(),
    );
    for reply in &replies {
        // Indented, because a thread is a hierarchy and the indent is what
        // says so without a glyph. `crate::panels::bookmarks` indents its
        // levels the same way and for the same reason.
        ui.indent(("pdfcer-note-reply", reply.id), |ui| {
            // ui-text-exempt: internal widget id, never displayed
            if let Some(byline) =
                tp::comment_row_byline(reply.author.as_deref(), reply.modified.as_deref())
            {
                ui.label(egui::RichText::new(byline).small().weak());
            }
            match reply.contents.as_deref().map(str::trim) {
                Some(text) if !text.is_empty() => {
                    ui.label(t::popup_body(text));
                }
                _ => {
                    ui.label(egui::RichText::new(t::popup_reply_no_note()).small().weak());
                }
            }
            // Rule 4: pdfcer is showing a string §12.5.6.2 tells a conforming
            // reader to ignore in favour of the group primary's. Another
            // reader legitimately shows something else, so this says so.
            if reply.group_member {
                ui.label(
                    egui::RichText::new(t::popup_reply_is_group_member())
                        .small()
                        .weak(),
                );
            }
        });
    }
}

/// The controls under the note: edit, save, remove, delete — or the sentence
/// that says why there are none.
///
/// # ★★★ Four states, and each is a fact about the document or the mode rather
/// than about what this build can do
///
/// | state | what is drawn |
/// |---|---|
/// | **Read mode** | one sentence naming the mode that can edit. R9's *temporarily* unavailable case — see the module header |
/// | **the file locks it** (§12.5.3 bit 8) | one sentence saying so. R83: the controls are omitted, not offered and refused |
/// | **a ce dimension** | one sentence saying where its text comes from. Rule 15; a note typed over it is regenerated away |
/// | anything else | *Add note* / *Edit note*, the editor when it is open, and *Delete comment* |
///
/// None of the three sentences is a greyed button, and none is temporary in a
/// way the operator cannot see: the first names its own remedy, and the other
/// two are properties of the file.
fn controls(
    ui: &mut egui::Ui,
    f: &Ctx<'_>,
    note: &NoteView,
    draft: &mut NoteDraft,
    existing: &str,
    actions: &mut Vec<Action>,
) {
    if !f.caps.author_markup {
        ui.label(egui::RichText::new(t::popup_read_only()).small().weak());
        return;
    }
    if note.locked {
        ui.label(egui::RichText::new(t::popup_locked()).small().weak());
        return;
    }
    if f.is_ce_dimension {
        // The caption is already under the body — a ce dimension's text is its
        // measurement — so nothing more is said here. What is withheld is the
        // whole control row, including Delete: `delete_annotation` would
        // remove the `/Line` and leave the `/PieceInfo` sidecar describing a
        // ce dimension that no longer exists. That is the Dimension groups
        // panel's subject, not this window's.
        return;
    }

    if draft.editing(note.id, f.doc.edit_epoch) {
        ui.horizontal(|ui| {
            let save = ui.button(t::popup_save());
            crate::diag::ui_rect_visible(REGION_SAVE, save.rect, f.clip);
            if save.clicked() {
                // ★★★ `keep_author` comes from the SAME function the Comments
                // panel uses, and that is not tidiness. `pdfcer-core` named
                // this mistake when it shipped the verb: *"an implementation
                // writing all three keys unconditionally would silently strip
                // the author and date on every correction, leaving a review
                // comment from nobody, dated never."* Two editors for one
                // note, two spellings of the rule, and one of them eventually
                // gets it wrong — so there is one spelling.
                actions.push(Action::Annot(AnnotAction::SetNote {
                    id: note.id,
                    text: draft.text().to_owned(),
                    keep_author: crate::panels::comments::keeps_author_name(note.author.as_deref()),
                }));
                draft.close();
            }
            if ui.button(t::popup_cancel()).clicked() {
                draft.close();
            }
            // Only when there is something to remove. `clear_markup_note` on
            // an annotation with no note is a call whose entire effect is an
            // undo entry, and R9's rule about a control that cannot do
            // anything applies to one that can only do nothing.
            if !existing.is_empty()
                && ui
                    .button(t::popup_remove())
                    .on_hover_text(t::popup_remove_tooltip())
                    .clicked()
            {
                actions.push(Action::Annot(AnnotAction::ClearNote { id: note.id }));
                draft.close();
            }
        });
        return;
    }

    ui.horizontal(|ui| {
        let label = if existing.is_empty() {
            t::popup_add()
        } else {
            t::popup_edit()
        };
        let edit = ui.button(label);
        crate::diag::ui_rect_visible(REGION_EDIT, edit.rect, f.clip);
        if edit.clicked() {
            draft.begin(note.id, f.doc.edit_epoch, existing);
        }
        delete(ui, f, note, actions);
    });
}

/// *Delete comment*, and the guard that decides whether it is drawn at all.
///
/// # ★★★ The Comments panel's *"this build has no Delete"* was true and is not
///
/// That paragraph was written on 2026-08-14 and its stated reason —
/// *"`crate::app::actions::Action` has no variant that could carry the
/// intent"* — stopped being true when `AnnotAction::Delete` landed. It is
/// corrected in place, dated, in that module's own header, along with the
/// `/TrapNet` reasoning that depended on it. **A limitation sentence is a
/// citation with an hours-long shelf life**, and this project has now paid for
/// that lesson six times.
///
/// # R83: the control is omitted when the engine would refuse
///
/// `EditSession::annotation_deletion_refusal` answers *"would
/// `delete_annotation` refuse right now?"* for the two document-wide reasons —
/// encryption and a certification signature — and asking it is what lets this
/// draw nothing instead of offering a button whose only outcome is a worded
/// decline. The per-annotation refusals (a locked annotation, a ce dimension)
/// are handled above by the same rule.
///
/// ⚠ It is **not a perfect oracle**, and `docs/core-api/03-capabilities.md`
/// §3.4 says so: the real call can still refuse. That is why the funnel's
/// worded decline stays the answer of record and this is only a filter on the
/// affordance.
fn delete(ui: &mut egui::Ui, f: &Ctx<'_>, note: &NoteView, actions: &mut Vec<Action>) {
    if f.doc.session.annotation_deletion_refusal().is_some() {
        return;
    }
    let button = ui
        .button(t::popup_delete())
        .on_hover_text(t::popup_delete_tooltip());
    crate::diag::ui_rect_visible(REGION_DELETE, button.rect, f.clip);
    if button.clicked() {
        // The pop-up is closed first. An open window describing an annotation
        // that no longer exists would draw for one more frame with an empty
        // body, which reads as the delete having failed.
        open::set(ui.ctx(), &f.doc.path, note.id, false);
        actions.push(Action::Annot(AnnotAction::Delete {
            page: f.page_index,
            id: note.id,
        }));
    }
}

/// **The hover tooltip** — the cheap half of the same affordance.
///
/// Returns whether one was shown, for the trace.
///
/// # ★ Why it is suppressed over an open pop-up
///
/// Because the answer is already on screen, three inches away and in full. A
/// tooltip repeating a truncated copy of it would be noise, and it would
/// appear *under the operator's pointer* at the moment they are reaching for
/// the window's own controls.
///
/// # ★★ Why it is drawn as an `Area` rather than through `Response::on_hover_text`
///
/// Because there is no `Response` to hang it off. The thing being hovered is a
/// rectangle inside a page raster, not an egui widget — the canvas is one
/// `Image` response covering the whole strip. An `Area` at `Order::Tooltip` is
/// what egui's own tooltip machinery resolves to anyway, and building it
/// directly means the offset, the constraint and the layer are this module's
/// to state rather than inherited from a widget that does not exist.
fn tooltip(
    ctx: &egui::Context,
    notes: &[NoteView],
    overrides: &open::Overrides,
    map: &PageMapping,
    clip: Rect,
) -> bool {
    let Some(screen) = ctx.pointer_latest_pos() else {
        return false;
    };
    if !clip.contains(screen) {
        return false;
    }
    // ★ Never over a pop-up. `layer_id_at` is egui's own answer to *"what is
    // under the pointer"*, so this cannot disagree with what will actually
    // receive the click — a hand-rolled rectangle test over the open windows
    // could, and would be a second claimant on the same question.
    if ctx
        .layer_id_at(screen)
        .is_some_and(|layer| layer.order > egui::Order::Background)
    {
        return false;
    }
    #[allow(clippy::cast_possible_truncation)]
    let tolerance = map.tolerance() as f32;
    let Some(note) = model::under(notes, map.to_page(screen), tolerance) else {
        return false;
    };
    if overrides.is_open(note.id, note.authored_open) {
        return false;
    }

    let text = t::popup_tooltip(note.author.as_deref(), note.contents.as_deref());
    egui::Area::new(egui::Id::new("pdfcer-note-tooltip")) // ui-text-exempt: internal widget id, never displayed
        .order(egui::Order::Tooltip)
        // Below and right of the pointer, which is where every tooltip in the
        // platform sits — above it would be under the operator's own hand on a
        // pen display, and the class convention exists for that reason.
        .fixed_pos(screen + egui::vec2(TOOLTIP_OFFSET, TOOLTIP_OFFSET))
        .constrain_to(clip)
        .interactable(false)
        .show(ctx, |ui| {
            ui.set_max_width(POPUP_WIDTH);
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_max_width(POPUP_WIDTH);
                ui.label(text);
            });
        });
    true
}

/// How far a tooltip sits from the pointer, in screen points.
///
/// Sixteen — clear of a standard cursor bitmap in both axes, so the tooltip
/// never appears *under* the arrow that summoned it.
const TOOLTIP_OFFSET: f32 = 16.0;

/// **A click landed on the canvas — open or close a note's pop-up.**
///
/// Called from `crate::canvas::clicking` beside the annotation hit test, and
/// **it consumes nothing**: the click goes on to mean exactly what it meant
/// before this module existed. That is the property that made a single click
/// the right gesture in all three modes — see the module header.
///
/// Returns the note it toggled, for the trace at the call site.
///
/// # ★★★ Why the hit test is repeated here rather than reusing `annot_hit`
///
/// Because `crate::canvas::clicking`'s `annot_hit` is gated on
/// `caps.author_markup` — Review and Edit only — and **that gate is the
/// operator's complaint**. In Read mode it is `None` on every click, so a rung
/// that consumed it would do nothing in the one mode this whole feature exists
/// for.
///
/// The two hit tests also exclude different things, deliberately, and
/// [`model::notes_on`]'s header states the difference and why each is right
/// for its surface. This one is a *reading* question and the other is a
/// *restyling* question.
///
/// # Cost
///
/// One `/Annots` walk per click — not per frame — bounded by
/// `pdfcer_core::annot::MAX_ANNOTS_PER_PAGE` and decomposing nothing. It is
/// the same walk the annotation hit test beside it already pays, so a click on
/// the 129,758-object benchmark sheet costs what it did.
pub fn clicked_on(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page_index: usize,
    point: Pos2,
    map: &PageMapping,
) -> Option<ObjId> {
    let page = doc.pages.get(page_index)?;
    let view = doc.session.view();
    let notes = model::notes_on(&view, page);
    #[allow(clippy::cast_possible_truncation)]
    let tolerance = map.tolerance() as f32;
    let note = model::under(&notes, point, tolerance)?;
    // ★★★ **A mark with nothing to say does not open a window.** The press
    // falls through to selection, which is what the operator meant by clicking
    // a cloud that carries no comment. `model::under` answers for every
    // annotation that *can* carry a note; `has_something_to_read` is the one
    // that asks whether this one *does*. See its doc comment for why a sticky
    // note is exempt (an empty one is still a note, and is how you write in it)
    // and why a byline alone is not enough.
    //
    // ⚠ Asked HERE and not inside `model::under`, deliberately. `under` is also
    // the painter's answer to *"which note is the pointer over"*, and the same
    // list feeds the hover tooltip; narrowing it there would make a commentless
    // cloud stop reporting itself to the trace as well, and the harness would
    // lose the only evidence that the pointer was over anything at all.
    if !model::has_something_to_read(note) {
        return None;
    }
    // ★ TOGGLE, not open. Clicking the icon again closes the window, which is
    // what every reader in the class does and what an operator who has just
    // opened one by accident will try.
    let overrides = open::load(ctx, &doc.path);
    let now = overrides.is_open(note.id, note.authored_open);
    open::set(ctx, &doc.path, note.id, !now);
    Some(note.id)
}

/// One `note-popup` line per frame that has something to say.
///
/// # Why this is more than a debug print
///
/// Because the two things that could be wrong here are both **invisible in a
/// screenshot**. A pop-up that is open because the file said `/Open` and one
/// that is open because the operator clicked look identical, and the whole of
/// [`open`]'s contract is the difference between them — an implementation that
/// silently defaulted `/Open` to `false` would look perfect until somebody
/// opened a file another product authored. `from_file` is the only oracle for
/// that available from outside the process.
///
/// `with_note` is the second: a page of markup pdfcer drew carries no
/// `/Contents` at all, so *"the pop-up showed nothing"* has two causes — the
/// note is empty, or the reader is broken — and they need opposite responses.
///
/// Silent when there is nothing open and nothing hovered, so a trace of a
/// reading session is not one line per frame of noise.
fn trace(notes: &[NoteView], open_count: usize, from_file: usize, tipped: bool) {
    if open_count == 0 && !tipped {
        return;
    }
    crate::diag::trace(|| {
        let with_note = notes
            .iter()
            .filter(|n| n.contents.as_deref().is_some_and(|c| !c.trim().is_empty()))
            .count();
        let authored_open = notes.iter().filter(|n| n.authored_open).count();
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "note-popup notes={} with_note={with_note} authored_open={authored_open} \
             open={open_count} from_file={from_file} tooltip={tipped}",
            notes.len(),
        )
    });
}

/// The egui id this document's canvas note draft is stored under.
///
/// Per document, exactly as [`open`]'s overrides are and for the same reason:
/// object ids collide freely between files, so a shared draft would put one
/// document's half-typed note into another's pop-up.
fn draft_key(path: &std::path::Path) -> egui::Id {
    egui::Id::new(("pdfcer-note-popup-draft", path)) // ui-text-exempt: internal widget id, never displayed
}

/// Read the canvas pop-up's note draft.
///
/// ★★ **A second draft from the Comments panel's, deliberately.** They are two
/// editors on two surfaces and an operator may legitimately have one open in
/// each; sharing one draft would mean typing in the panel silently rewriting
/// what is in the window. `NoteDraft`'s own `(annotation, edit epoch)` stamp
/// is what keeps *this* one honest — a draft stamped at an older epoch
/// describes a document that no longer exists, and [`show`] calls `sync`
/// before anything is drawn.
fn load_draft(ctx: &egui::Context, path: &std::path::Path) -> NoteDraft {
    ctx.data(|d| d.get_temp::<NoteDraft>(draft_key(path)).unwrap_or_default())
}

/// Store the canvas pop-up's note draft.
fn store_draft(ctx: &egui::Context, path: &std::path::Path, draft: &NoteDraft) {
    ctx.data_mut(|d| d.insert_temp(draft_key(path), draft.clone()));
}

#[cfg(test)]
mod placement_tests {
    use super::*;

    /// A canvas viewport of the shape the driven sweep measured: the central
    /// panel with a dock on the right, `[[288 174] - [772 758]]`.
    const CLIP: Rect = Rect {
        min: Pos2::new(288.0, 174.0),
        max: Pos2::new(772.0, 758.0),
    };

    /// The rectangle the pop-up would occupy, given the origin under test.
    ///
    /// Height is a stand-in: [`clear_of_anchor`] is specified to separate on
    /// **x alone** for its first two candidates, so a test that asserted with a
    /// real height would be asserting something weaker than the contract.
    fn window(origin: Pos2, height: f32) -> Rect {
        Rect::from_min_size(origin, egui::vec2(POPUP_BOX_WIDTH, height))
    }

    /// ★★★ **The 2026-09-05 defect, as an assertion.**
    ///
    /// The exact geometry from `dragging_a_markup_moves_it`'s trace: a markup
    /// selected at `[[464.0 464.5] - [551.7 550.2]]`, whose pop-up was drawn at
    /// `[[498.0 465.0] - [772.0 565.0]]` — on top of it, so every press meant
    /// for the shape went to the window instead and neither the move nor the
    /// resize ever reached the canvas.
    ///
    /// With the anchor at x 464–551.7 there is no room on the right
    /// (551.7 plus 8 plus 276 = 835.7, past the viewport's 772) and none on the
    /// left (464 minus 8 minus 276 = 180, before the viewport's 288), so this
    /// exercises candidate 3.
    #[test]
    fn the_popup_that_ate_the_drag_is_placed_clear_of_its_annotation() {
        let anchor = Rect::from_min_max(Pos2::new(464.0, 464.5), Pos2::new(551.7, 550.2));
        let preferred = Pos2::new(anchor.max.x + POPUP_GAP, anchor.min.y);
        let origin = clear_of_anchor(preferred, anchor, CLIP);
        // The measured window was 100 pt tall for a note with no words.
        let drawn = window(origin, 100.0);
        assert!(
            !drawn.intersects(anchor),
            "the pop-up at {drawn:?} still covers the annotation at {anchor:?} — this is the \
             state in which a markup could be rotated and could not be moved or resized"
        );
        // 290.5 pt above the note against 207.8 below it, so the rule the
        // header states — the side of the anchor with more room — puts the
        // window above. Asserted as the *rule*, not as "above": a viewport a
        // hundred points taller would legitimately answer the other way, and a
        // test that pinned the direction would fail on a resize rather than on
        // a regression.
        let (above, below) = (anchor.min.y - CLIP.min.y, CLIP.max.y - anchor.max.y);
        assert!(
            if below >= above {
                origin.y > anchor.max.y
            } else {
                origin.y == CLIP.min.y
            },
            "the window went to the side with LESS room: {above:.1} pt above the note, \
             {below:.1} pt below it, and the origin is {origin:?}"
        );
    }

    /// ★ **Candidate 1: a preference that already clears the anchor is kept.**
    ///
    /// This is the ordinary case and the one that must not move: a note near
    /// the left of the sheet has room on its right, and the window goes there,
    /// byte for byte where it went before this function existed.
    #[test]
    fn a_note_with_room_beside_it_keeps_the_placement_it_always_had() {
        let anchor = Rect::from_min_max(Pos2::new(300.0, 300.0), Pos2::new(360.0, 340.0));
        let preferred = Pos2::new(anchor.max.x + POPUP_GAP, anchor.min.y);
        assert_eq!(
            clear_of_anchor(preferred, anchor, CLIP),
            preferred,
            "the right-hand placement fits here, so nothing may be second-guessed"
        );
    }

    /// ★★ **Candidate 2: no room on the right, and the flip is to the LEFT —
    /// not a slide.**
    ///
    /// The anchor is pushed against the right edge of the viewport, where
    /// `Area::constrain_to` used to slide the window back over it. Room on the
    /// left is 700 − 8 − 276 = 416 ≥ 288, so the left-hand placement is
    /// available and must be taken.
    #[test]
    fn a_note_against_the_right_edge_puts_its_window_on_the_left() {
        let anchor = Rect::from_min_max(Pos2::new(700.0, 300.0), Pos2::new(760.0, 340.0));
        let preferred = Pos2::new(anchor.max.x + POPUP_GAP, anchor.min.y);
        let origin = clear_of_anchor(preferred, anchor, CLIP);
        assert!(
            origin.x + POPUP_BOX_WIDTH <= anchor.min.x,
            "the window at x={} is not clear of an anchor beginning at x={}",
            origin.x,
            anchor.min.x
        );
        assert!(
            origin.x >= CLIP.min.x,
            "…and it must still be inside the canvas viewport"
        );
        assert!(
            !window(origin, 400.0).intersects(anchor),
            "a horizontal separation has to hold for ANY height — that is why the first two \
             candidates are decided on x alone"
        );
    }

    /// ★★ **Candidate 3 the other way up: more room above than below.**
    ///
    /// A note low on the sheet, too wide for either side. The window goes to
    /// the top of the viewport, which is where every point of the available
    /// room is.
    #[test]
    fn a_wide_note_low_on_the_sheet_puts_its_window_above() {
        let anchor = Rect::from_min_max(Pos2::new(300.0, 700.0), Pos2::new(760.0, 740.0));
        let preferred = Pos2::new(anchor.max.x + POPUP_GAP, anchor.min.y);
        let origin = clear_of_anchor(preferred, anchor, CLIP);
        assert_eq!(
            origin.y,
            CLIP.min.y,
            "there are {} pt above this note and {} pt below it, so above is the side with room",
            anchor.min.y - CLIP.min.y,
            CLIP.max.y - anchor.max.y
        );
        assert!(
            origin.x >= CLIP.min.x && origin.x + POPUP_BOX_WIDTH <= CLIP.max.x,
            "the x is pinned into the viewport so `constrain_to` has nothing left to slide"
        );
    }

    /// ★★ **The file's `/Popup` rectangle is honoured — until it overlaps.**
    ///
    /// Two documents, one function. The first names a rectangle beside its
    /// note and gets exactly that; the second names one on top of its note and
    /// is overruled, because honouring it would be honouring a defect.
    #[test]
    fn a_producers_popup_rectangle_is_kept_unless_it_covers_the_note() {
        let anchor = Rect::from_min_max(Pos2::new(400.0, 300.0), Pos2::new(440.0, 340.0));
        let beside = Pos2::new(460.0, 290.0);
        assert_eq!(
            clear_of_anchor(beside, anchor, CLIP),
            beside,
            "a `/Popup` rect that clears its note is the producer's statement and stands"
        );
        let over = Pos2::new(410.0, 305.0);
        let moved = clear_of_anchor(over, anchor, CLIP);
        assert_ne!(
            over, moved,
            "a `/Popup` rect laid over its own note is overruled"
        );
        assert!(
            !window(moved, 100.0).intersects(anchor),
            "…and the replacement clears it"
        );
    }

    /// ★ **The outer box is wider than the contents, and the gap depends on it.**
    ///
    /// [`POPUP_BOX_WIDTH`] exists because `egui::Frame::popup`'s margin sits
    /// outside `ui.set_max_width(POPUP_WIDTH)`. If the two were ever collapsed
    /// into one constant the separation would be short by the frame and the
    /// overlap would come back at the margin — silently, on exactly the notes
    /// nearest the edge.
    /// ★ A `const` assertion rather than a runtime one, and the change is not
    /// cosmetic: clippy refuses `assertions_on_constants` because a runtime
    /// `assert!` over two constants **can never fail at runtime** — it is
    /// decided when the crate is compiled, and a test that cannot fail is not
    /// evidence. `const _: () = assert!(..)` states the same fact where it is
    /// actually checked: the build stops, with this message, and no test has to
    /// run at all.
    ///
    /// Kept as an assertion rather than deleted because the relationship is
    /// real and load-bearing — a pop-up's outer box is its contents plus
    /// `Frame::popup`'s margin and stroke, and the placement arithmetic that
    /// keeps a window clear of its own annotation is computed from the OUTER
    /// width. Were the two ever made equal, every placement would be short by
    /// the frame and the window would creep back over the mark it belongs to,
    /// which is the defect this module was rewritten to close on 2026-09-05.
    const _: () = assert!(
        POPUP_BOX_WIDTH > POPUP_WIDTH,
        "the box a pop-up occupies is its contents plus `Frame::popup`'s margin and stroke"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every region name this module publishes.
    ///
    /// Enumerated so the sweep below cannot drift from the constants: a name
    /// added without being added here would simply not be checked, which is
    /// the *"hand-written list inside a completeness sweep"* failure
    /// `RESUME.md` records shipping four defects in one gap.
    const REGIONS: &[&str] = &[
        REGION_POPUP,
        REGION_CLOSE,
        REGION_EDIT,
        REGION_BOX,
        REGION_SAVE,
        REGION_DELETE,
    ];

    /// ★ **Every region name is unique and namespaced to this module.**
    ///
    /// A region name is a key a driven check aims a real pointer at. Two
    /// controls publishing one name leaves the harness clicking whichever was
    /// drawn last — a coordinate nobody chose — and the failure looks like the
    /// feature being broken rather than like the check being blind.
    #[test]
    fn the_region_names_are_unique_and_namespaced() {
        let mut seen = std::collections::BTreeSet::new();
        for name in REGIONS {
            assert!(
                name.starts_with("notepopup."),
                "`{name}` is not namespaced to this module, so it can collide \
                 with a region published somewhere else"
            );
            assert!(seen.insert(*name), "`{name}` is published twice");
        }
    }

    /// A note occupying `anchor` in canvas space, carrying `popup` as its
    /// companion rectangle.
    fn note(anchor: Rect, popup: Option<Rect>) -> NoteView {
        NoteView {
            id: ObjId::new(7, 0),
            subtype: "Text".to_owned(),
            contents: Some("words".to_owned()),
            author: None,
            modified: None,
            anchor,
            popup: popup.map(|rect| model::PopupBox {
                id: ObjId::new(8, 0),
                rect: Some(rect),
            }),
            authored_open: false,
            locked: false,
            in_reply_to: None,
        }
    }

    /// A canvas 800 × 600 points at 100 %, with the page drawn at the origin —
    /// the identity mapping, so a canvas coordinate is a screen coordinate and
    /// the arithmetic under test is visible rather than buried in a projection.
    fn identity_map() -> PageMapping {
        let image = Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
        PageMapping::new(image, (800.0, 600.0), 1.0)
    }

    fn viewport() -> Rect {
        Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))
    }

    /// ★★★ **With no `/Popup` rect, the window goes BESIDE the note — never
    /// over it.**
    ///
    /// The one placement failure an operator would report as the feature being
    /// broken: a pop-up drawn on top of the icon that opened it hides the
    /// thing they just clicked, and the second click — which they will
    /// certainly try — lands on the window rather than on the note, so it does
    /// not close. Asserting *strictly* to the right of the anchor's right edge
    /// is what forbids the whole family of "close enough" placements.
    #[test]
    fn a_note_with_no_popup_rect_opens_beside_itself() {
        let anchor = Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(20.0, 20.0));
        let at = popup_origin(&note(anchor, None), &identity_map(), viewport());
        assert!(
            at.x > anchor.max.x,
            "the pop-up starts at x={} and the note ends at x={} — it would be \
             drawn over the mark that opened it",
            at.x,
            anchor.max.x
        );
        assert!(
            (at.y - anchor.min.y).abs() < 1.0,
            "top-aligned with the note, not floating above or below it: y={}",
            at.y
        );
    }

    /// ★★★ **The file's own `/Popup` rectangle wins.**
    ///
    /// §12.5.6.14 makes the pop-up a separate annotation *with its own
    /// placement*, and a producer who moved a note's window across the sheet
    /// meant it. Ignoring that and always placing beside would make every
    /// document laid out in Acrobat look rearranged here — and it is the
    /// mistake an implementation makes by default, because "beside" is the
    /// easier code path and it looks fine on a file pdfcer itself wrote.
    #[test]
    fn the_files_own_popup_rectangle_is_honoured() {
        let anchor = Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(20.0, 20.0));
        let placed = Rect::from_min_size(Pos2::new(400.0, 300.0), egui::vec2(150.0, 108.0));
        let at = popup_origin(&note(anchor, Some(placed)), &identity_map(), viewport());
        assert_eq!(
            at, placed.min,
            "the pop-up ignored the placement in the file"
        );
    }

    /// ★★ **A note far off the viewport does not put its window in the
    /// opposite corner.**
    ///
    /// The deep-zoom failure, and it is not hypothetical: at 300,000 % a page
    /// point maps to a screen coordinate in the millions, and an `Area` handed
    /// one is constrained back to the nearest edge — so every pop-up on the
    /// sheet stacks in one corner, all of them claiming to belong to marks
    /// nowhere near it. Clamping the origin first means an off-screen note's
    /// window arrives at the edge *nearest to it*, which is at least a
    /// direction.
    #[test]
    fn a_note_far_off_screen_is_clamped_towards_itself() {
        let far = Rect::from_min_size(Pos2::new(9.0e6, -9.0e6), egui::vec2(20.0, 20.0));
        let at = popup_origin(&note(far, None), &identity_map(), viewport());
        assert!(at.x.is_finite() && at.y.is_finite(), "{at:?}");
        // Right of the viewport (the note is far right) and above it (the note
        // is far above) — the direction is preserved, which is the whole point.
        assert!(at.x >= viewport().max.x, "{at:?}");
        assert!(at.y <= viewport().min.y, "{at:?}");
        // …and bounded, rather than the raw millions that would defeat
        // `constrain_to`.
        assert!(at.x <= viewport().max.x + POPUP_WIDTH, "{at:?}");
    }
}
