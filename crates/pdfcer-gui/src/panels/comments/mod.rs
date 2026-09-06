//! # `panels::comments` — every annotation on this document, listed
//!
//! The comment list a reviewer works through. Salvaged from the old shell's
//! `main.rs:7028-7060` (`fn comments_panel`), whose **exclusion reasoning is
//! settled law** and is carried across below with its argument rather than as
//! a code snippet.
//!
//! The classification lives in [`model`]; this file is the drawing, the
//! disclosures and the one action the panel can raise.
//!
//! ## ★ What it deliberately excludes, decided by exclusion first
//!
//! Straight from the old shell, and the wording is kept because the wording is
//! the decision:
//!
//! - **`/Widget`** — form fields have their own first-class surface (the Forms
//!   panel). `Annotation::is_widget()` already exists as the exact predicate;
//!   a second one would be a divergence waiting to happen.
//! - **`/Popup`** — a reader-UI window attached to a `Text`/`FreeText`
//!   annotation, never independent content. One row per real annotation; its
//!   pop-up is implementation detail. §12.5.6.14 is a `shall`: a pop-up
//!   *"shall not appear alone but is associated with a markup annotation, its
//!   parent annotation."*
//! - **ce dimensions are NOT excluded by type**, and that is worth stating:
//!   they are `/Line` annotations and so they appear here. The spec excludes
//!   them conceptually because they have their own home, but excluding them by
//!   subtype would also hide a genuine `/Line` markup an operator drew.
//!   Showing them is the lesser wrong, and it is honest — they ARE annotations
//!   on the document.
//!
//! ### `/TrapNet`, and ★★★ the argument that came BACK on 2026-09-05
//!
//! The old shell also excluded **`/TrapNet`**, and its reason was
//! *delete-shaped*: core refuses a `/TrapNet` deletion by name, so listing one
//! *"would put a row here whose only possible action is a refusal, which is the
//! affordance R83 forbids."*
//!
//! From 2026-08-14 this header said *"This build has no Delete, so that reason
//! does not reach"*, and excluded `/TrapNet` on the surviving half of the
//! argument instead: it is **prepress output state** — the trapping a RIP
//! applied to the page — so it is not a comment, nobody wrote it, and there is
//! nothing in it for a reviewer to work through. That is the same shape as the
//! `/Widget` exclusion: not *"we cannot act on it"* but *"this surface is not
//! about it."*
//!
//! **The panel now HAS a Delete** (see below), so the old shell's reasoning is
//! live again — and, exactly as the paragraph it replaces predicted, *"nothing
//! needs to change: the row is already absent, for a reason that does not
//! depend on the button."* Both arguments now hold, independently, for the
//! same exclusion. Recorded rather than tidied because a prediction that came
//! true is the cheapest evidence available that the reasoning was sound.
//!
//! What the departure buys instead is that **nothing is silently omitted**.
//! Every exclusion is counted and disclosed by
//! [`crate::text::panels::comments::comments_excluded`], so a reviewer looking
//! at six rows on a drawing they know carries forty annotations is told the
//! arithmetic and where each missing kind went. The old shell stated the rule
//! only on the empty case; this states the numbers on every case.
//!
//! ★ **A filter is a FOURTH kind of omission**, added 2026-09-05, and it is
//! disclosed by the same rule — see [`filter`]'s header and
//! [`crate::text::panels::comments::comments_filtered`]. It is the only one
//! the operator caused, which makes stating it more important rather than
//! less: an exclusion is a property of the document a reviewer learns once, a
//! filter is a switch they set an hour ago and have forgotten.
//!
//! ## Ordering: page order, then `/Annots` order
//!
//! The ordering `pdfcer list-annotations` already produces, **reused by
//! name** rather than a second GUI-only rule that could disagree with it. See
//! [`model`]'s header for what that means concretely, and for why there is no
//! sort by date.
//!
//! ## ★ Read the SESSION, not the file on disk
//!
//! [`body`] hands [`model::collect`] `doc.session.view()` — the base revision
//! with **every unsaved edit applied**, which is the same thing the canvas
//! rasterizes. An operator who has just drawn three shapes must see three rows
//! without saving first. `crate::panels::forms`' body is the worked example
//! and carries the same sentence.
//!
//! ## Actions, not mutations — and every one of them is an `Action`
//!
//! [`Action::GoToPage`], from a row's **Go to** control, exactly as
//! `crate::panels::bookmarks` does; and [`Action::Annot`] carrying
//! [`AnnotAction::SetNote`], [`AnnotAction::ClearNote`] and — since
//! 2026-09-05 — [`AnnotAction::Delete`]. The body is handed `&OpenDoc` — a
//! **shared** reference, so this is a compile-time fact and not a convention —
//! it reads, and it pushes. It never touches the document.
//!
//! ⚠ **One thing this panel does write directly, and it is not the document.**
//! A row's **Go to** also opens that comment's canvas pop-up, through
//! `crate::canvas::notepopup::open::set`, which is `egui::Memory` — interface
//! state, per document, never saved. It is here rather than behind an `Action`
//! because an `Action` is drained *after* the frame and the pop-up must be
//! open on the frame the page arrives, and because the actions-not-mutations
//! rule is about the **document**: the thing it protects is the undo stack and
//! the edit epoch, neither of which a floating window touches.
//!
//! ★★ Why it does it at all: jumping to page 14 and leaving the reviewer to
//! find which of its six clouds the row meant is half a navigation. Acrobat's
//! Comment pane opens the comment it takes you to, and that is the gesture
//! being matched.
//!
//! ### ★★★ THERE IS A DELETE — 2026-09-05, and the paragraph it replaces was
//! true when written
//!
//! This header said, from 2026-08-14:
//!
//! > **There is no Delete, and its absence is a decision rather than a gap.**
//! > […] `crate::app::actions::Action` has no variant that could carry the
//! > intent and `app/actions.rs` is not this work's to extend. […] What the
//! > day it lands needs, so nobody re-derives it: one `Action` variant, one
//! > dispatch arm calling `EditSession::delete_annotation`, and the three
//! > disclosures `docs/core-api/03-capabilities.md` §3.4 requires.
//!
//! **Every one of those existed by the time anybody looked again.**
//! `crate::app::actions::annot::AnnotAction::Delete { page, id }` is the
//! variant, `crate::app::actions::apply` is the arm, and
//! `crate::app::actions::annots::delete` is the body — already reporting the
//! engine's collateral through `crate::text::markup::deleted_collateral`. The
//! canvas Delete key and the Format tab have both been deleting annotations
//! through it for weeks. **The reviewer's own work list was the last surface
//! that could not**, which is precisely backwards.
//!
//! It took the operator's report of 2026-09-05 — *"the review features should
//! look and act the same as they do in Acrobat Reader"* — to send somebody to
//! re-derive the reason rather than re-read the sentence. ⇒ **The sixth
//! recurrence in this project of a limitation outliving its cause.** Corrected
//! in place and dated, here and in `crate::text::panels::comments`' header,
//! rather than left as two answers.
//!
//! #### What the Delete carries, and where each piece comes from
//!
//! - **R83 — the control is omitted where the engine would refuse.**
//!   `EditSession::annotation_deletion_refusal` answers the two document-wide
//!   cases (encrypted, certified) and [`delete_control`] asks it. A locked
//!   annotation and a ce dimension are withheld by the row itself, for reasons
//!   the row already states.
//! - **"Delete is not redaction"**, in the tooltip, per §3.4 — an incremental
//!   save leaves the previous revision in the file.
//! - **The collateral is reported after the call, not predicted before it.**
//!   The old shell computed a hover preview; this does not, and the reason is
//!   §3.4's own: the preview *"is not a perfect oracle"* and the real call can
//!   still refuse. `delete_annotation`'s report names what actually went — a
//!   `/Popup` removed, replies orphaned, group members promoted — and
//!   `annots::delete` already surfaces it. One statement of record beats a
//!   guess before and a fact after that can disagree.
//!
//! ## Rule 4: everything here is disclosure, and none of it is on the page
//!
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`'s first
//! non-negotiable: *"Disclosure lives off-canvas."* **A panel is the right
//! home**, and this one draws not a single pixel on the canvas — no badge on a
//! hidden annotation, no tint on an unresolved appearance, no outline round the
//! row under the pointer. It must not start to. The one-line test is *would a
//! screenshot of the editing canvas differ from a screenshot of the same
//! document saved and reopened?*
//!
//! Three of this panel's row captions exist **only** because of that rule, and
//! each names the inference it discloses:
//!
//! | Caption | The inference |
//! |---|---|
//! | `comment_row_hidden` | none — this one is a *document fact* the file states and the page therefore cannot show. §3.4.5: *"list it and mark it hidden."* |
//! | `comment_row_appearance_unresolved` | pdfcer chose to paint nothing, under a default core documents as **evidence tier (d), a reasoned guess** |
//! | `comment_row_is_group_member` | pdfcer shows the raw `/Contents` where §12.5.6.2 says a reader should show the group primary's — so another viewer legitimately disagrees |
//!
//! The old shell's Forms panel highlighted a field's rectangle on the page on
//! hover, and rule 4's fourth clause would permit the equivalent here (a hover
//! highlight *"is the cursor"*). It is not built, for the same reason that one
//! was not carried: the mechanism — a channel from a panel to the canvas
//! overlay — does not exist in this build, and `crate::canvas` is not this
//! module's to extend. Named rather than silently dropped, and named as a
//! *permitted* affordance so nobody later reads its absence as a rule.
//!
//! ## The two layout rules, and which one applies
//!
//! 1. **Scrollbars must be visible.** `crate::panels::scroll_style` is applied
//!    by [`crate::panels::Panel::show`] before any body runs, so this panel
//!    inherits it. egui's default `floating()` bar allocates zero space and is
//!    fully transparent when the pointer is elsewhere, which makes a scrolling
//!    area indistinguishable in a capture from content clipped at the
//!    container edge.
//!
//! 2. **A fixed-size child inside a scroll area needs the container's width
//!    stated.** ★ **This panel has no fixed-size child**, so
//!    [`crate::panels::content_width`] is deliberately not called — and that
//!    is stated here rather than left to look like an omission, because it is
//!    the second layout rule and skipping it silently is exactly how the
//!    Objects panel shipped clipped rows.
//!
//!    Every child is a `Label`, which wraps to whatever width it is given, so
//!    the clamping defect cannot arise: there is nothing whose *requested*
//!    size could exceed the pane and be silently squeezed. The one fixed-width
//!    child is the **Go to** button, at a couple of dozen points against a
//!    dock that opens at 320. A note body is arbitrary operator text and can
//!    be a paragraph; stating a container width computed from it would either
//!    scroll a 4,000 pt row sideways or defeat its own wrapping. Vertical-only
//!    scrolling with wrapping labels is the correct shape here, and
//!    `crate::panels::forms` — whose rows have the same character — takes it
//!    too.
//!
//! ## Cost, stated rather than discovered
//!
//! [`body`] walks **every page's `/Annots`** every frame, and lays out every
//! row it finds. Both are the old shell's behaviour and both are bounded —
//! `pdfcer_core::annot::MAX_ANNOTS_PER_PAGE` caps the walk, and the walk reads
//! one array plus one dictionary per annotation rather than decomposing any
//! content — so on the documents this project measures against
//! (`SW41177.pdf`, 36 sheets) it is nothing beside a raster.
//!
//! The one thing that would change the picture is a document with thousands of
//! comments, where the *layout* rather than the walk becomes the cost. The fix
//! is `ScrollArea::show_rows`, which needs a uniform row height, which these
//! rows do not have — a row is one to six labels depending on what the
//! annotation carries. Named here so the next hand does not have to measure it
//! twice; `crate::panels::forms` carries the same note for the same reason.
//!
//! ## `PDFCER_DIAG` proves what the panel computed
//!
//! One `comments-panel` line per frame carrying the counts: rows found, rows
//! with note text, rows with an author, ce dimensions, suppressed rows,
//! unresolved appearances, relations — and the three exclusion counts
//! separately, so *how many it excluded and why* is answerable from the trace
//! alone. That is the founding rule of this project applied to a surface whose
//! correctness is entirely arithmetic: a screenshot of this panel cannot tell
//! you that a widget was excluded, and the trace can.

/// ★ **Narrowing and ordering the work list** — the filter, the sort, and the
/// disclosure a filtered list owes. Added 2026-09-05 against Acrobat's Comment
/// pane; its header carries the four things Acrobat offers that this cannot,
/// and which engine gap each is filed under.
pub mod filter;

/// Turning a document into a comment list — the classification, testable
/// without a `Ui`.
pub mod model;

/// The note being typed, and the `(annotation, edit epoch)` stamp that keeps
/// it honest.
pub mod note;

use pdfcer_core::object::ObjId;

use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::panels::comments as t;

use self::model::{CommentRow, Listing, Note, Relation};
use self::note::NoteDraft;

/// The ribbon command that opens this panel.
///
/// Named here as well as on [`crate::panels::Panel::command_id`] so this
/// module's own reachability test can assert it, exactly as
/// `crate::panels::forms` does.
///
/// # ★ Why `markup.comments` and not a `view.panel_*` id
///
/// `RIBBON_IA.md` names Comments **twice** and the two placements cannot both
/// be honoured, because P1 gives a command one tab:
///
/// - **§5.2** lists `Comments` among View ▸ Panels, beside Pages, Objects,
///   Bookmarks, Layers, Signatures and Forms.
/// - **§5.5** gives the Markup tab its own `Comments` group, with `Comments
///   panel` in it.
/// - **§7's migration map** then settles it explicitly, naming the source and
///   the destination: `Review ▸ Comments ▸ Comments` → `Markup ▸ Comments`.
///
/// The migration map is the more specific statement — §5.2's row is a list of
/// panel names, while §7 is a per-control ruling on this control — so Markup ▸
/// Comments it is. `crate::shell::manifest::markup` already reached the same
/// conclusion in the same words when the tab was built, and the command is
/// already registered and already on the ribbon; only the panel behind it was
/// missing.
///
/// **The mode taxonomy agrees, which is what makes this safe.** The Forms
/// panel had to move *off* the Edit tab because Read is shown `file` and
/// `view` alone and Read mounts Forms. Comments is mounted by **Review and
/// Edit only** (`crate::app::modes::defaults`), and both of those are shown
/// the `markup` tab. So no mode can mount this panel without also being able
/// to reopen it — which is the failure the Forms move existed to prevent.
pub const COMMAND_ID: &str = "markup.comments";

/// **The region the *Add note* / *Edit note* control publishes** — on the
/// FIRST row that offers one, and only that row.
///
/// # ★★ Why only the first, when every row draws the control
///
/// A region name is a key. Publishing the same name from thirty rows would
/// leave the harness clicking whichever one happened to be drawn last, which is
/// a coordinate nobody chose and which moves when a row above it grows a
/// caption. The first row is the one deterministic choice available without
/// inventing a per-row naming scheme that nothing would consume.
///
/// ⇒ ★★★ This matters because of a finding this project has now made twice:
/// **a gesture with no driver is a gesture R1 cannot reach**, and the gap
/// leaves no failing test behind. The canvas context menus went the whole life
/// of the project unopened by any check because `Driver` had no right-click.
/// A panel control that published no rect would be the same hole in a quieter
/// place.
pub const REGION_EDIT: &str = "comments.note_edit"; // ui-text-exempt: trace region name, never displayed
/// The region the open editor's text box publishes. Unique by construction —
/// one draft, one editor, one box.
pub const REGION_BOX: &str = "comments.note_box"; // ui-text-exempt: trace region name, never displayed
/// The region the open editor's *Save note* publishes.
pub const REGION_SAVE: &str = "comments.note_save"; // ui-text-exempt: trace region name, never displayed
/// The region the open editor's *Remove note* publishes, when there is a note
/// to remove.
pub const REGION_REMOVE: &str = "comments.note_remove"; // ui-text-exempt: trace region name, never displayed
/// The region the FIRST row's *Delete comment* publishes — one name, one row,
/// for [`REGION_EDIT`]'s stated reason.
pub const REGION_DELETE: &str = "comments.delete"; // ui-text-exempt: trace region name, never displayed
/// The region the filter strip's *Show all* publishes, when a filter is set.
pub const REGION_FILTER_CLEAR: &str = "comments.filter_clear"; // ui-text-exempt: trace region name, never displayed

/// Draw the Comments panel.
///
/// The one entry point. Shape and signature match every other panel body — see
/// [`crate::panels::Panel::show`].
///
/// ## ★ `state` carries one thing, and it is not a selection
///
/// Until 2026-08-28 this paragraph read *"`state` is unused, and that is a
/// property of the panel rather than an oversight: it is a pure function of the
/// document."* That was true for as long as the panel could not write anything.
///
/// It now holds a [`note::NoteDraft`] — one annotation's `/Contents` while the
/// operator is typing it — and **nothing else**. It is emphatically not a
/// "selected comment": the draft names one annotation by `ObjId` for the
/// duration of one edit, and it decides nothing about what the canvas outlines,
/// what the Format tab describes or what Delete acts on. That distinction is
/// the one [`crate::panels::ObjectTreeUi::focus`]' docs refuse to blur, and it
/// is what stops this panel growing a second, weaker selection that the canvas
/// would then have to be kept in step with.
pub fn body(ui: &mut egui::Ui, doc: &OpenDoc, state: &mut PanelsState, actions: &mut Vec<Action>) {
    // ★ FIRST, before anything is drawn: drop the operator's half-typed note if
    // the document has moved under it. `NoteDraft`'s header carries the whole
    // argument — the short form is that a draft stamped at an older epoch
    // describes a document that no longer exists, and an editor showing words
    // beside a shape that no longer has them is lying for as long as it is on
    // screen.
    state.comments_mut().draft.sync(doc.edit_epoch);
    // Reset before anything is drawn — see `CommentsUi::writing_controls_drawn`.
    state.comments_mut().writing_controls_drawn = 0;
    // Asked ONCE per frame, never per row. `dimension_model` walks the catalog
    // to the `/PieceInfo` sidecar and deserializes it — cheap, and bounded by
    // the number of ce dimensions rather than by the document — but calling it
    // per row would make the panel O(rows x sidecar), which is the shape of
    // defect the old shell's hover-gated deletion preview was fixing.
    let ce_dimensions = model::ce_dimension_annots(&doc.session);
    // Read the SESSION, not the file on disk — see the module header.
    let view = doc.session.view();
    let listing = model::collect(&view, &doc.pages, &ce_dimensions);

    // Read BEFORE the strip is drawn, so the trace can state whether a filter
    // is narrowing the list — see [`trace`] for why that field exists and for
    // the one-frame lag this read implies. Cloned rather than borrowed because
    // `state` is `&mut` for the rest of the draw.
    let filter_now = state.comments_mut().filter.clone();
    trace(doc, &listing, &filter_now);

    let excluded = t::comments_excluded(
        listing.excluded.widgets,
        listing.excluded.popups,
        listing.excluded.trap_nets,
    );

    if listing.rows.is_empty() {
        // The empty case still discloses the filter. A drawing whose every
        // annotation is a form field is a real and common shape, and "no notes
        // or markup" alone would leave an operator who can *see* annotations on
        // the page believing the panel had failed.
        ui.label(t::comments_none());
        if let Some(line) = excluded {
            ui.label(egui::RichText::new(line).small().weak());
        }
        return;
    }

    // ★ EVERY DISCLOSURE SITS ABOVE THE LIST, without exception.
    //
    // The same rule the Bookmarks truncation note, the Signatures caveat and
    // the Fonts coverage note follow — four panels, one reason: an operator
    // who scrolls a short list and stops has already drawn their conclusion by
    // the time a footnote would reach them.
    //
    // The order is by how much it changes what the operator should do: the
    // count first (how big is this job), then what is missing from it, then why
    // the rows below look emptier than expected.
    ui.label(t::comments_count(listing.rows.len()));
    if let Some(line) = excluded {
        ui.label(egui::RichText::new(line).small().weak());
    }
    if listing.every_row_lacks_note_text() {
        ui.label(
            egui::RichText::new(t::comments_all_without_notes())
                .small()
                .weak(),
        );
    }

    // ★★★ The filter strip, and then the rows it left. `crate::panels::comments::filter`
    // carries the argument for what is offered and what is not; this is the
    // control strip and the **disclosure**, which is the half that makes
    // filtering safe on a surface whose founding rule is that nothing is
    // silently omitted.
    //
    // Drawn from the UNFILTERED listing, deliberately: a chooser built from
    // the rows that survived the current filter would drop every other author
    // from the menu the moment one was picked, leaving no route back except
    // Show all. The operator must be able to move from Ken's comments to Jo's
    // in one press.
    let total = listing.rows.len();
    filter_strip(ui, &listing.rows, &mut state.comments_mut().filter);
    // Cloned out before the rows are borrowed, so the strip's `&mut` on the
    // panel state has ended by the time the list is drawn. A `Filter` is three
    // small fields; the alternative is threading a borrow through the whole
    // draw for nothing.
    //
    // ★ Re-read rather than reusing `filter_now` from the top of the draw, and
    // that is not redundancy: `filter_strip` may have CHANGED the filter on
    // this very frame, and a chooser whose effect waited for the next repaint
    // would read as a control that does not work. The trace's copy is the
    // pre-strip one deliberately — see [`trace`].
    let narrowing = state.comments_mut().filter.clone();
    let rows = filter::apply(listing.rows.clone(), &narrowing);
    if narrowing.is_narrowing() {
        // ★ ABOVE the list, with every other disclosure and for their reason:
        // an operator who scrolls a short list and stops has already drawn
        // their conclusion by the time a footnote would reach them.
        ui.label(
            egui::RichText::new(t::comments_filtered(rows.len(), total))
                .small()
                .weak(),
        );
    }
    ui.separator();

    // Collected during the draw and applied after it — the actions-not-
    // mutations discipline at its smallest, and the same shape
    // `crate::panels::bookmarks` uses. One `Option`, not a `Vec`: two rows
    // cannot be clicked in one frame, and a `Vec` would invite a future reader
    // to push two navigations that would fight.
    let mut go: Option<(usize, Option<ObjId>)> = None;
    // The document verb one row raised, if any. One `Option` for the same
    // reason `go` is one: two rows cannot be pressed in a single frame, and a
    // `Vec` would invite a future reader to queue two edits that would each
    // bump the epoch under the other.
    let mut verb: Option<AnnotAction> = None;
    // Whether the *Add note* region has been published this frame. See
    // [`REGION_EDIT`]: one name, one row, and the first row is the only
    // deterministic choice.
    let mut published = false;
    // Tallied through `RowSink` and published to the panel's own state after
    // the draw — see `CommentsUi::writing_controls_drawn`.
    let mut writing_controls_drawn: u32 = 0;
    let epoch = doc.edit_epoch;
    // ★ Asked once per frame — see `RowSink::deletable`. A document-wide
    // question deserves one answer.
    //
    // ★★★ **TWO independent questions, and only one of them used to be asked.**
    //
    // `annotation_deletion_refusal` answers *"would `pdfcer-core` refuse this
    // document?"* — encrypted, certified. It says nothing at all about **what
    // stance the operator is in**, and until 2026-09-05 nothing else did
    // either: the Delete control and the note editor were drawn, live and
    // effective, in **Read**.
    //
    // # How it was found, and why no test could have
    //
    // A smoke launch of the release binary, off screen, on the comment
    // fixture, in Read mode — the default. Its trace read:
    //
    // ```text
    // mode-changed to=read panels=4
    // comments-panel listed=3 with_note=3 authors=3 replies=1
    // ui-rect name=comments.note_edit rect=[[1086.0 347.0] - [1146.9 365.0]]
    // ui-rect name=comments.delete    rect=[[1133.7 368.0] - [1239.0 386.0]]
    // ```
    //
    // Three Delete buttons and an editor, in the mode whose whole stated
    // posture is *the document is not yours to alter*. Every unit test passed;
    // the gates were 29 of 29; the ribbon comparison exited 0. **R1 is the
    // rule this exists to illustrate — a green suite is not a report of
    // working software** — and the surrounding tests could not have caught it
    // because none of them enters a mode: they call the panel with an
    // `OpenDoc` and no stance at all.
    //
    // # Why `author_markup` rather than `authors_anything`
    //
    // A comment is markup. Review authors markup and may delete one; Edit may;
    // Read may not. `panels::bookmarks` reaches for `authors_anything()`
    // instead, and correctly — a bookmark is document *structure*, so a mode
    // that authors markup but not content must keep the whole row. The two
    // predicates happen to agree across today's three modes, so this is a
    // statement of intent rather than a behavioural difference, and it is the
    // one that stays right if a fourth mode is ever added.
    //
    // Read off the `Context` through `canvas::tool::capabilities`, which is
    // the crate's established seam and deliberately the same call the canvas
    // makes — so the panel and the page can never disagree about what the mode
    // permits. `panels::tool::idle` set that precedent and framed it as R9.
    let authoring = crate::canvas::tool::capabilities(ui.ctx()).author_markup;
    // R9: an unavailable capability renders **nothing**. Not greyed — greying
    // is for *temporarily* unavailable, and a stance is not a temporary
    // condition. The mode selector is the visible explanation, and it is the
    // same treatment every markup tool already gets in Read.
    let deletable = authoring && doc.session.annotation_deletion_refusal().is_none();
    // ★★★ **The annotation the CANVAS has selected** — the other half of the
    // interaction `pdfcer-core` describes, and the half this panel was missing:
    //
    // > draw the shape → **it is selected** → type the comment in the panel
    // > beside the page.
    //
    // Without it the second arrow is *"now find your shape among forty rows"*,
    // and on the drawings this program is for that is a scroll and a guess: the
    // rows are headed by subtype and page, so two clouds on sheet 3 read
    // identically.
    //
    // ★★ **This is not a second selection.** It is the canvas's own, read. The
    // panel decides nothing about it, writes nothing to it, and lists the same
    // rows in the same order whether it is set or not.
    // `crate::panels::ObjectTreeUi::focus`' docs draw that line, and this is it
    // being respected rather than blurred.
    let selected = doc.selection.annot().map(|a| a.target.id);
    let ui_state = state.comments_mut();
    // Taken before the closure borrows the state, and written back after — the
    // scroll must happen once per selection CHANGE, and the closure needs to
    // know what the last one was while it is deciding.
    let already = ui_state.scrolled_to;
    let mut scrolled_to = already;
    let draft = &mut ui_state.draft;
    egui::ScrollArea::vertical()
        .id_salt("comment-rows")
        .show(ui, |ui| {
            // ★ The FILTERED rows. `last` is derived from the same vector the
            // loop walks, which is what stops a separator being drawn after
            // the final row when a filter has shortened the list — the kind of
            // off-by-one that looks like a rendering fault.
            let last = rows.len().saturating_sub(1);
            for (i, comment) in rows.iter().enumerate() {
                let is_selected = comment.id.is_some() && comment.id == selected;
                // `push_id` per row, because two rows of the same subtype on
                // the same page would otherwise give their **Go to** buttons
                // the same egui id — which shows up as the wrong button
                // responding to a hover, the same collision
                // `crate::panels::bookmarks` keys its indent against.
                let response = ui
                    .push_id(i, |ui| {
                        row(
                            ui,
                            comment,
                            &mut RowSink {
                                go: &mut go,
                                verb: &mut verb,
                                published: &mut published,
                                deletable,
                                deletable_stance: authoring,
                                writing_controls_drawn: &mut writing_controls_drawn,
                            },
                            draft,
                            epoch,
                            is_selected,
                        );
                    })
                    .response;
                // ★ Scrolled to on the frame the selection MOVES, and not while
                // it stands.
                //
                // `scroll_to_me` every frame would pin the list under the
                // operator's own scrollbar: they could not look at any other row
                // while a shape was selected on the canvas, which is a surface
                // fighting its user. `CommentsUi::scrolled_to` is what makes
                // this once-per-change rather than once-per-frame.
                if is_selected && already != selected {
                    response.scroll_to_me(Some(egui::Align::Center));
                    scrolled_to = selected;
                }
                if i != last {
                    ui.separator();
                }
            }
        });
    // ★ Written back unconditionally, INCLUDING when nothing is selected — so
    // deselecting and re-selecting the same annotation scrolls to it again,
    // which is what an operator who has scrolled away and clicked the shape a
    // second time is asking for.
    // Publish the tally on the borrow that is already open, so it describes the
    // frame that just happened — see `CommentsUi::writing_controls_drawn`.
    ui_state.writing_controls_drawn = writing_controls_drawn;
    ui_state.scrolled_to = if selected.is_some() {
        scrolled_to
    } else {
        None
    };

    if let Some((page, id)) = go {
        actions.push(Action::GoToPage(page));
        // ★★ …and open that comment where it lives. See the module header on
        // why this is written directly rather than carried as an `Action`: an
        // `Action` drains after the frame, and the pop-up has to be open on
        // the frame the page arrives.
        if let Some(id) = id {
            crate::canvas::notepopup::open::set(ui.ctx(), &doc.path, id, true);
        }
    }
    if let Some(verb) = verb {
        // ★ The draft closes here rather than in the row that raised the verb,
        // and it closes for BOTH outcomes — a save the engine accepts and one
        // it refuses.
        //
        // Leaving it open on a refusal was considered and rejected: the refusal
        // is worded on the status line, the words are still in the undo-free
        // world of the operator's own clipboard-less retyping, and an editor
        // that stays open next to a row whose text did not change reads as a
        // save that is still pending. The stamp would go stale the moment
        // anything else edited the document anyway, so "open on refusal" is a
        // state with a very short and unpredictable life.
        draft.close();
        actions.push(Action::Annot(verb));
    }
}

/// Draw one comment.
///
/// Every line below the heading is conditional, and each condition is a real
/// state of a real document rather than a formatting choice. A row is between
/// two and seven lines tall depending on what the annotation actually carries,
/// which is why this panel cannot use `ScrollArea::show_rows` — see the module
/// header.
/// **What one row can raise**, collected so the row function takes a subject
/// and a destination rather than eight loose parameters.
///
/// Three `Option`s and a flag, and each is `None`/`false` for the whole frame
/// unless exactly one row sets it — which is the invariant that makes them
/// scalars rather than `Vec`s: **two rows cannot be pressed in one frame**, and
/// a `Vec` would invite a future reader to queue two navigations or two edits
/// that would each bump the epoch under the other.
struct RowSink<'a> {
    /// ★ **What a Go to press asks for** — the page, and the annotation on it.
    ///
    /// The id travels beside the page because navigating is only half the
    /// gesture: `crate::canvas::notepopup` opens that comment's pop-up when
    /// the page arrives, so the reviewer lands on the sheet with the words in
    /// front of them rather than with six clouds to choose between. `None`
    /// where the annotation has no object id — a malformed direct dictionary,
    /// which the row already declines to offer an editor for — in which case
    /// the navigation still happens and nothing opens.
    go: &'a mut Option<(usize, Option<ObjId>)>,
    /// The document verb a Save or a Remove raised.
    verb: &'a mut Option<AnnotAction>,
    /// Whether [`REGION_EDIT`] has been published this frame — one name, one
    /// row, and the first row that offers the control is the only deterministic
    /// choice. See that constant.
    published: &'a mut bool,
    /// ★★ **Whether `delete_annotation` would be refused right now**, asked
    /// ONCE per frame in [`body`] and carried.
    ///
    /// R83: an affordance that cannot be honoured is not drawn. Asked once
    /// rather than per row because
    /// `EditSession::annotation_deletion_refusal` is a **document-wide**
    /// question — encryption and the certification gate — so a per-row call
    /// would be the same answer computed forty times, and worse, forty places
    /// it could be forgotten.
    ///
    /// ⚠ It is not a perfect oracle and `docs/core-api/03-capabilities.md`
    /// §3.4 says so: the real call can still refuse, for reasons that belong
    /// to one annotation. That is why the funnel's worded decline stays the
    /// answer of record and this is only a filter on the affordance.
    deletable: bool,
    /// **Whether the operator's MODE authors markup at all** — the stance
    /// half of the two questions [`body`] asks, kept separate from
    /// [`Self::deletable`] because they fail for different reasons and a
    /// future reader must not collapse them.
    ///
    /// `deletable` folds this in (a Delete needs both permissions); the note
    /// editor needs only this one, because a document that refuses *deletion*
    /// may still accept a note.
    deletable_stance: bool,
    /// Tally for [`crate::panels::comments::note::CommentsUi::writing_controls_drawn`].
    writing_controls_drawn: &'a mut u32,
}

fn row(
    ui: &mut egui::Ui,
    comment: &CommentRow,
    sink: &mut RowSink<'_>,
    draft: &mut NoteDraft,
    epoch: u64,
    is_selected: bool,
) {
    // The page number is 1-based **only here**, where a human reads it. The
    // index itself travels 0-based to `Action::GoToPage`; see
    // [`tests::the_page_index_travels_zero_based_and_prints_one_based`].
    let page_number = comment.page_index + 1;

    // ★ THE HEADING SAYS WHAT THE ANNOTATION ACTUALLY IS.
    //
    // A ce dimension is named as one — project rule 15, and the constructive
    // half of the exclusion argument in this module's header: the sidecar can
    // tell a ce dimension from a `/Line` markup, so the panel does not have to
    // choose between mislabelling one and hiding the other.
    let heading = if comment.is_ce_dimension {
        t::comment_row_ce_dimension_heading(&comment.subtype, page_number)
    } else {
        t::comment_row_heading(&comment.subtype, page_number)
    };
    // `.strong()` is unusable in this theme — see `DEFECTS.md` D11.
    //
    // ★★★ The selected row says so **in words**, on the heading line.
    //
    // Not a colour, not a tint, not a highlight: `DEFECTS.md` D2 is this
    // project's record of a theme change making text invisible against its own
    // background, and every list in this shell that marks a row — the Pages
    // panel's picks, the Objects tree's focus — marks it with a *shape or a
    // word* rather than with colour alone. A reviewer scanning forty rows for
    // the cloud they just drew needs the mark to survive a theme they have not
    // chosen yet.
    let heading = if is_selected {
        t::comment_row_selected_heading(&heading)
    } else {
        heading
    };
    ui.label(egui::RichText::new(heading));

    // Author and modification date, when the annotation carries either.
    //
    // `/T` is a Table 170 MARKUP key, so its absence on a `/Link` or a
    // `/PrinterMark` means "this subtype has no such concept", not "anonymous"
    // — which is why an absent one prints nothing rather than a placeholder
    // that would read as a claim about a person.
    if let Some(byline) =
        t::comment_row_byline(comment.author.as_deref(), comment.modified.as_deref())
    {
        let resp = ui.label(egui::RichText::new(byline).small().weak());
        // The tooltip explains the *date*, so it is attached only when there
        // is one. Hanging it off an author-only byline would answer a question
        // that line does not raise.
        if comment.modified.is_some() {
            resp.on_hover_text(t::comment_row_modified_tooltip());
        }
    }

    // The note itself — three states, and collapsing any two would mislead.
    match &comment.note {
        Note::Text(text) => {
            ui.label(t::comment_row_body(text));
        }
        Note::Description(text) => {
            ui.label(t::comment_row_body(text));
            // §12.5.2's other meaning. Below the text rather than above it,
            // deliberately and against this panel's own disclosure-first rule:
            // the caption is *about* the string, and a reader has to have seen
            // the string for "this is not a note somebody wrote" to attach to
            // anything. The disclosure-first rule is about caveats that change
            // what you conclude from a LIST; this one qualifies one line.
            ui.label(
                egui::RichText::new(t::comment_row_description_caption())
                    .small()
                    .weak(),
            );
        }
        Note::Absent => {
            // Worded as a fact about the document, never as missing data. On
            // markup pdfcer itself drew this is the *expected* state — the
            // engine cannot yet write `/Contents` on geometric markup — and
            // the document-wide sentence above the list has already said so
            // when it is true of every row.
            let caption = if comment.is_ce_dimension {
                t::comment_row_ce_dimension_no_note()
            } else {
                t::comment_row_no_note()
            };
            ui.label(egui::RichText::new(caption).small().weak());
        }
    }

    // The disclosures. Each is drawn only when it is true, so the marker means
    // something when it appears; a row of "not hidden / appearance fine /
    // not a reply" captions would be noise with the same information content
    // as nothing at all.
    if comment.suppressed {
        ui.label(egui::RichText::new(t::comment_row_hidden()).small().weak());
    }
    if comment.appearance_unresolved {
        ui.label(
            egui::RichText::new(t::comment_row_appearance_unresolved())
                .small()
                .weak(),
        );
    }
    match &comment.relation {
        Some(Relation::Reply) => {
            ui.label(
                egui::RichText::new(t::comment_row_is_reply())
                    .small()
                    .weak(),
            );
        }
        Some(Relation::GroupMember) => {
            ui.label(
                egui::RichText::new(t::comment_row_is_group_member())
                    .small()
                    .weak(),
            );
        }
        // An `/RT` name pdfcer has never seen, and nothing to say about it —
        // see `model::Relation::Other`. Saying "this has an unrecognised
        // relationship" would be a placeholder for a fact with no consequence.
        Some(Relation::Other) | None => {}
    }

    // The note editor, and the control that opens it. Below the disclosures
    // because it is the one thing on the row that *acts*, and an operator
    // scanning the list reads downward and stops when they reach a button.
    note_controls(ui, comment, draft, epoch, sink);

    ui.horizontal(|ui| {
        if ui
            .button(t::comment_row_goto())
            .on_hover_text(t::comment_row_goto_tooltip(page_number))
            .clicked()
        {
            *sink.go = Some((comment.page_index, comment.id));
        }
        delete_control(ui, comment, sink);
    });
}

/// ★★★ **Delete this comment** — added 2026-09-05; see the module header on
/// the paragraph that forbade it and had outlived its reason.
///
/// # Four reasons it is not drawn, and every one is R83 rather than R9
///
/// R83 — *an affordance that cannot be honoured is not drawn* — rather than
/// R9's *unavailable renders nothing*, and the distinction is real: the
/// capability **exists**, and what is missing in each case is the permission
/// to use it on *this* annotation. None of the four is a build limitation, so
/// none of them is a sentence about pdfcer.
///
/// | not drawn when | why |
/// |---|---|
/// | the row has **no object id** | a direct dictionary in `/Annots` is a malformed file (§12.5.2 Table 164 requires an indirect object) and there is nothing to name. The row already says so where its editor would be |
/// | the document **refuses deletion** | encrypted, or a certification signature forbids the change. `EditSession::annotation_deletion_refusal`, asked once per frame — see [`RowSink::deletable`] |
/// | it is a **ce dimension** | rule 15. `delete_annotation` would remove the `/Line` and leave the `/PieceInfo` sidecar describing a ce dimension that no longer exists. The Dimension groups panel owns that verb |
/// | the annotation is **hidden** | ★ deliberately NOT a reason. A hidden annotation is exactly the one a reviewer cannot reach from the canvas, so this panel is the only place it can be removed — which is the whole argument for listing it in the first place |
///
/// # ★ No confirmation, and no hover preview
///
/// The old shell computed the collateral on hover and showed it before the
/// press. This does not, and §3.4 is the reason in its own words: the preview
/// *"is not a perfect oracle"*. What is reported instead is what the engine
/// **actually did** — `delete_annotation`'s report names the pop-up it
/// removed, the replies it orphaned and the group members it promoted, and
/// `crate::app::actions::annots::delete` already surfaces it. One statement of
/// record beats a guess before and a fact after that can disagree with it.
///
/// Undo is the safety net, and it is one press: the deletion is a single
/// `CommandKind` on the same stack as every other edit.
fn delete_control(ui: &mut egui::Ui, comment: &CommentRow, sink: &mut RowSink<'_>) {
    if !sink.deletable || comment.is_ce_dimension {
        return;
    }
    let Some(id) = comment.id else {
        return;
    };
    let button = ui
        .button(t::comment_row_delete())
        .on_hover_text(t::comment_row_delete_tooltip());
    *sink.writing_controls_drawn += 1;
    // `ui_rect_visible`, not `ui_rect`: these rows live in a `ScrollArea` and a
    // control scrolled out of view still reports a rect. See [`REGION_EDIT`].
    crate::diag::ui_rect_visible(REGION_DELETE, button.rect, ui.clip_rect());
    if button.clicked() {
        *sink.verb = Some(AnnotAction::Delete {
            page: comment.page_index,
            id,
        });
    }
}

/// **The filter strip** — two choosers, a switch, an ordering and a way back.
///
/// # ★★ Built from the UNFILTERED rows, always
///
/// A chooser built from what survived the current filter would drop every
/// other author from the menu the moment one was chosen, leaving no route from
/// one reviewer's comments to another's except *Show all* and starting again.
/// The lists are therefore derived from the whole listing and are stable while
/// the operator works — which is also what makes them a map of the document
/// rather than a map of the current view.
///
/// # ★ Why `ComboBox` and not a row of toggles
///
/// Because the number of authors and the number of subtypes are properties of
/// the **document**, not of this build: a drawing set that went round six
/// reviewers has six names, and six toggles would be six lines of a 320 pt
/// dock. A chooser is one line whatever the document contains.
///
/// # `Show all` is drawn only when a filter is set
///
/// R9's shape applied to a control that would do nothing: with no filter in
/// force, *Show all* is a button whose entire effect is a repaint. It appears
/// with the first narrowing and goes when the last one is lifted, which also
/// makes its presence a second, wordless statement that something is hidden.
fn filter_strip(ui: &mut egui::Ui, all: &[CommentRow], state: &mut filter::Filter) {
    let authors = filter::authors(all);
    let subtypes = filter::subtypes(all);
    ui.horizontal_wrapped(|ui| {
        chooser(
            ui,
            "comments-filter-author", // ui-text-exempt: internal widget id, never displayed
            t::comment_filter_author(),
            &authors,
            &mut state.author,
        );
        chooser(
            ui,
            "comments-filter-type", // ui-text-exempt: internal widget id, never displayed
            t::comment_filter_type(),
            &subtypes,
            &mut state.subtype,
        );
    });
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut state.with_note_only, t::comment_filter_with_note());
        egui::ComboBox::from_id_salt("comments-sort") // ui-text-exempt: internal widget id, never displayed
            .selected_text(sort_label(state.sort))
            .show_ui(ui, |ui| {
                for sort in filter::Sort::ALL {
                    ui.selectable_value(&mut state.sort, *sort, sort_label(*sort));
                }
            })
            .response
            .on_hover_text(t::comment_sort_label());
        if state.is_narrowing() {
            let clear = ui.button(t::comment_filter_clear());
            crate::diag::ui_rect_visible(REGION_FILTER_CLEAR, clear.rect, ui.clip_rect());
            if clear.clicked() {
                // ★ The ORDERING survives, and the narrowing does not. They
                // are different acts: *Show all* is the answer to "what am I
                // missing", and an operator who asked for the list by author
                // did not ask for that to be undone as well.
                state.author = None;
                state.subtype = None;
                state.with_note_only = false;
            }
        }
    });
}

/// One "All, or exactly this one" chooser over a list of document values.
///
/// Written once for the author and the type because the two differ only in
/// their label and their values — and because a second copy is a second place
/// for the `None` entry to be forgotten, which would leave a filter nobody
/// could lift.
fn chooser(
    ui: &mut egui::Ui,
    id: &str,
    label: &str,
    values: &[String],
    chosen: &mut Option<String>,
) {
    // The chooser's resting text is the LABEL, not "All": a strip reading
    // `All  All  [ ] With text only` names nothing, and the operator has to
    // open a menu to find out what the first one was about.
    let selected = chosen.clone().unwrap_or_else(|| label.to_owned());
    egui::ComboBox::from_id_salt(id)
        .selected_text(selected)
        .show_ui(ui, |ui| {
            ui.selectable_value(chosen, None, t::comment_filter_all());
            for value in values {
                ui.selectable_value(chosen, Some(value.clone()), value);
            }
        })
        .response
        .on_hover_text(label);
}

/// The label for one ordering.
///
/// A `match` rather than a method on [`filter::Sort`], because a label is copy
/// and copy lives in `crate::text` — and because the compiler makes this
/// exhaustive, so a fourth ordering cannot ship without a word for it.
fn sort_label(sort: filter::Sort) -> &'static str {
    match sort {
        filter::Sort::Document => t::comment_sort_document(),
        filter::Sort::Author => t::comment_sort_author(),
        filter::Sort::Subtype => t::comment_sort_subtype(),
    }
}

/// **The note editor for one row, and the control that opens it.**
///
/// Three shapes, decided by what the annotation is rather than by what this
/// build can do:
///
/// | the row | what is drawn |
/// |---|---|
/// | a **ce dimension** | a caption saying where its text actually comes from |
/// | an annotation with **no object id** | a caption saying why pdfcer cannot address it |
/// | anything else | *Add note* / *Edit note*, and the editor when it is open |
///
/// # ★★★ R9: neither caption is a greyed button
///
/// *"An unavailable capability renders nothing, not a disabled stub. Greying is
/// reserved for temporarily unavailable."* Neither of these is temporary: a ce
/// dimension's `/Contents` is regenerated from its measurement by
/// `author_dimension`, so a note written over it would be silently thrown away,
/// and a direct-dictionary annotation is a **malformed file** (§12.5.2 Table 164
/// requires the dictionary to be an indirect object) with nothing to name. A
/// greyed *Edit note* would promise that some state of the program would let
/// the operator press it, and none would.
///
/// # ★★ Why a `/Link` is offered the editor
///
/// `/Contents` is dual-purpose (§12.5.2): note text on a subtype that displays
/// text, an accessibility description on one that does not — and
/// [`t::comment_row_description_caption`] already says which this row is.
/// `set_markup_note` accepts both, so withholding the editor would be
/// withholding a capability the engine has, on a guess about the operator's
/// intent. The caption is the honest half; the button is the useful half.
///
/// ★ It is worth knowing what this costs: on a `/Link` with no `/Contents` at
/// all the control still says *Add note*, because `Note::Absent` carries no
/// subtype interpretation to distinguish "nobody wrote a comment" from "nobody
/// wrote a description". Named here rather than left to be found.
fn note_controls(
    ui: &mut egui::Ui,
    comment: &CommentRow,
    draft: &mut NoteDraft,
    epoch: u64,
    sink: &mut RowSink<'_>,
) {
    // ★★★ **Nothing to type into in a reading stance.** Same finding, same
    // frame and same argument as the Delete control — see `RowSink::deletable`
    // at its assignment. `Add note` and `Edit note` both **write** to the
    // document, so a mode that does not author markup is offered neither.
    //
    // ⚠ Deliberately BEFORE the ce-dimension branch below, and the order is
    // load-bearing: that branch draws an explanatory sentence about why a ce
    // dimension's note is not editable *here*, which in Read would answer a
    // question the operator cannot have asked, about a control that is not on
    // screen. R9's rule is that an unavailable capability renders **nothing** —
    // and a sentence is something.
    if !sink.deletable_stance {
        return;
    }
    if comment.is_ce_dimension {
        ui.label(
            egui::RichText::new(t::comment_row_note_not_editable_ce_dimension())
                .small()
                .weak(),
        );
        return;
    }
    let Some(id) = comment.id else {
        ui.label(
            egui::RichText::new(t::comment_row_note_no_handle())
                .small()
                .weak(),
        );
        return;
    };

    if draft.editing(id, epoch) {
        editor(ui, comment, id, draft, sink.verb);
        return;
    }

    // The existing words, which seed the editor. `Note::Description` seeds it
    // too — the operator is editing that string whichever of §12.5.2's two
    // meanings it carries, and an editor that opened empty over a description
    // would invite them to destroy it by typing.
    let existing = match &comment.note {
        Note::Text(text) | Note::Description(text) => text.as_str(),
        Note::Absent => "",
    };
    let label = if existing.is_empty() {
        t::comment_row_add_note()
    } else {
        t::comment_row_edit_note()
    };
    let button = ui.button(label);
    // `ui_rect_visible`, not `ui_rect`: these rows live in a `ScrollArea`, and a
    // control scrolled out of view still reports a rect. A harness clicking a
    // coordinate that is behind the scroll edge clicks whatever IS there, which
    // fails as something else entirely.
    *sink.writing_controls_drawn += 1;
    if !*sink.published {
        crate::diag::ui_rect_visible(REGION_EDIT, button.rect, ui.clip_rect());
        *sink.published = true;
    }
    if button.clicked() {
        draft.begin(id, epoch, existing);
    }
}

/// ★★★ **Whether this annotation already carries a byline that is not ours to
/// move** — the one decision in this panel with a consequence in the file.
///
/// `true` means the `SetNote` action sends **no `/T` at all**, and
/// `pdfcer-core` leaves an omitted key untouched. `false` means the operator's
/// name from Settings > Comments is written, or nothing is if that name is
/// blank, which is a supported choice meaning *comment anonymously*.
///
/// # Why this is a function rather than three words at its call site
///
/// Because it is the mistake the engine warned about **by name** when it
/// shipped the verb, and it is invisible from every other angle:
///
/// > An implementation writing all three keys unconditionally would silently
/// > strip the author and date on every correction, leaving a review comment
/// > from nobody, dated never, looking exactly like a note somebody else had
/// > mangled.
///
/// A `Ui` cannot be driven in a unit test in this crate, so an expression
/// buried in [`editor`] would be reachable only by `tools/ui-verify` — and a
/// driven check can assert that *a* note was written far more easily than it
/// can assert that a `/T` was **not**. Pulled out, the rule has a name, a
/// suite, and one caller that also feeds the sentence the operator reads.
///
/// # ★ Whitespace counts as absent
///
/// A `/T` of `"  "` is a byline nobody wrote — the commonest way for one to
/// exist is a producer writing an empty string — and preserving it would leave
/// a comment credited to a space. Trimmed, so *"has an author"* means the same
/// thing here as it does in the row's own byline, which is drawn by
/// [`t::comment_row_byline`] under the same rule.
fn keeps_author(comment: &CommentRow) -> bool {
    keeps_author_name(comment.author.as_deref())
}

/// [`keeps_author`] over the name alone — **the one spelling of the rule**.
///
/// # ★★★ Why this is separate, added 2026-09-05
///
/// Because there are now **two** editors for one note: this panel's, and the
/// canvas pop-up's (`crate::canvas::notepopup`), which is the route that works
/// in Read mode and the answer to the operator's report of that date.
///
/// Two editors writing the same key is exactly the shape in which the mistake
/// `pdfcer-core` named by name gets made in one of them and not the other:
///
/// > An implementation writing all three keys unconditionally would silently
/// > strip the author and date on every correction, leaving a review comment
/// > from nobody, dated never, looking exactly like a note somebody else had
/// > mangled.
///
/// The pop-up has no [`CommentRow`] — it works from
/// `crate::canvas::notepopup::model::NoteView` — so the rule had to be
/// expressible over the name by itself or it would have been re-derived at the
/// second call site. Re-derived is how two surfaces come to disagree, and this
/// one's disagreement would be invisible until somebody read a saved file.
///
/// [`tests::a_note_with_an_author_keeps_it`] and its two siblings are the
/// suite, and they exercise this through [`keeps_author`].
#[must_use]
pub(crate) fn keeps_author_name(author: Option<&str>) -> bool {
    author.is_some_and(|author| !author.trim().is_empty())
}

/// The open editor: the box, the hint, the signature disclosure and the three
/// controls.
///
/// # ★★ The signature line is a rule-4 disclosure, not a caption
///
/// What `/T` will say is **invisible on the page** — a sticky's byline lives in
/// a pop-up window this shell does not draw, and a shape's lives nowhere at all
/// — so an operator has no way to discover what name their comments carry, or
/// that they carry none, or that editing somebody else's comment will leave
/// their name on it. Two sentences, one per case, and the case is decided by
/// the row rather than by a preference this panel cannot see.
///
/// # ★ Escape closes it, and it does so through egui rather than by reading the
/// keyboard
///
/// `TextEdit` surrenders focus on Escape, so `lost_focus()` plus the key is the
/// idiomatic test and — importantly for this codebase — it asks nothing about
/// whether "the operator is typing". A panel that read the raw key would be a
/// second claimant on a key the canvas caret and the tool arming both want, and
/// `tools/gates/check-typing-guard.sh` exists because that class of second
/// claimant has already cost this project the Delete key and the space bar.
fn editor(
    ui: &mut egui::Ui,
    comment: &CommentRow,
    id: ObjId,
    draft: &mut NoteDraft,
    verb: &mut Option<AnnotAction>,
) {
    let response = ui.add(
        egui::TextEdit::multiline(draft.text_mut())
            .desired_rows(3)
            .desired_width(f32::INFINITY),
    );
    crate::diag::ui_rect_visible(REGION_BOX, response.rect, ui.clip_rect());
    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        draft.close();
        return;
    }

    // ★★★ **A `/FreeText` row's before-the-write warning was DELETED here on
    // 2026-09-06, hours after it was added, and the deletion is the record.**
    //
    // It said, in un-`.weak()` type above the keyboard hint, that the words
    // printed in a text box could not be changed once it was placed and that
    // saving would leave the page reading what it did before. That was true and
    // measured when it was written: `EditSession::set_markup_note` committed
    // the annotation dictionary and not the `/AP` stream that paints the words.
    //
    // `pdfcer-core` `95a936e` closed it the same afternoon — `set_markup_note`
    // now re-bakes the appearance itself, in the same command and the same undo
    // entry — so the sentence became false while still reading as caution,
    // which is the kind of lie nobody notices. Deleted rather than reworded.
    //
    // ★★ And it could not be reworded to cover what survives. The one case
    // still owed a sentence is a text box whose appearance **another program**
    // drew, which pdfcer preserves rather than replaces — and that is decided
    // by baking the words and comparing bytes, *inside* `set_markup_note`.
    // Nothing drawn before the call can know it, so the surviving disclosure is
    // necessarily an after-the-fact one, on the status line, gated on
    // `MarkupNoteChange::appearance_rebaked`
    // (`crate::app::actions::annots::set_note`). The whole table is at
    // `crate::text::textannot`'s edit-time banner.
    //
    // R8b is unchanged and still met: the report is off-canvas, and the box
    // renders exactly as it will save.

    ui.label(
        egui::RichText::new(t::comment_row_note_hint())
            .small()
            .weak(),
    );

    // Whose name ends up on it. The disclosure and the action's flag come from
    // ONE function on purpose: a sentence that could disagree with the edit it
    // describes is worse than no sentence.
    let keep_author = keeps_author(comment);
    let signature = match comment.author.as_deref() {
        Some(author) if keep_author => t::comment_row_note_signature_kept(author.trim()),
        _ => t::comment_row_note_signature().to_owned(),
    };
    ui.label(egui::RichText::new(signature).small().weak());

    let had_note = !matches!(comment.note, Note::Absent);
    ui.horizontal(|ui| {
        let save = ui.button(t::comment_row_note_save());
        crate::diag::ui_rect_visible(REGION_SAVE, save.rect, ui.clip_rect());
        if save.clicked() {
            *verb = Some(AnnotAction::SetNote {
                id,
                text: draft.text().to_owned(),
                keep_author,
            });
        }
        if ui.button(t::comment_row_note_cancel()).clicked() {
            draft.close();
        }
        // Only when there is something to remove. `clear_markup_note` on an
        // annotation with no note is a call whose entire effect is an undo
        // entry, and R9's rule about a control that cannot do anything applies
        // to a control that can only do nothing.
        if had_note {
            let remove = ui
                .button(t::comment_row_note_remove())
                .on_hover_text(t::comment_row_note_remove_tooltip());
            crate::diag::ui_rect_visible(REGION_REMOVE, remove.rect, ui.clip_rect());
            if remove.clicked() {
                *verb = Some(AnnotAction::ClearNote { id });
            }
        }
    });
}

/// One `comments-panel` line per frame, carrying what the panel computed.
///
/// # Why this is more than a debug print
///
/// `HANDOFF.md` §2: *"Verify by driving the binary, not by a passing test"* —
/// and the eighth defect on its list was found **only** by printing what the
/// running application had chosen, because *"2,450 hairlines and a wash are
/// the same picture"*. This panel has the same property in a different
/// direction: a screenshot of it cannot tell you that four widgets were
/// excluded, that two rows are hidden annotations, or that the `/Line` on
/// page 3 was recognised as a ce dimension. Every one of those is arithmetic,
/// and arithmetic is what a trace is for.
///
/// Every count that drives a *decision* is here, which is the test for what
/// belongs: `with_note` decides the document-wide disclosure, the three
/// exclusion counts decide the exclusion line, and `ce_dimensions`,
/// `suppressed`, `unresolved`, `replies` and `group_members` each decide a row
/// caption. If a number here is wrong, something on screen is wrong with it.
///
/// # ★★★ `listed` is the CENSUS, `shown` is what is on screen — 2026-09-05
///
/// Until the filter landed this morning the two were the same number and this
/// comment said `listed` was *"the rows the panel drew"*. That sentence became
/// false the moment [`filter::apply`] was inserted between [`model::collect`]
/// and the draw, and it became false **silently**: a reader outside the
/// process saw a census shrink and had no way to tell a filtered list from a
/// document that had lost annotations.
///
/// That is exactly the omission this panel's founding discipline forbids —
/// *"nothing is silently omitted"* — held on the diagnostic channel as well as
/// on the screen, because two driven checks (`save_copy_round_trip` and
/// `undo_redo_round_trip`) use this line as their **only** oracle for whether
/// an annotation reached the document.
///
/// So the line now carries both, and they answer different questions:
///
/// | field | question | source |
/// |---|---|---|
/// | `listed` | how many annotations does this document have that a reviewer may work through | [`model::collect`], unfiltered — the census |
/// | `shown` | how many rows is the operator actually looking at | the same rows after [`filter::apply`] |
/// | `filtered` | is the operator's filter narrowing the list right now | [`filter::Filter::is_narrowing`] |
///
/// `listed` deliberately keeps its old meaning so no existing reader changes
/// verdict; what is new is that `filtered=1` now tells one what it never
/// could, and `shown` says by how much.
///
/// ⚠ **The filter is read one frame late**, and that is deliberate rather than
/// overlooked: this runs *before* [`filter_strip`] draws, so a filter the
/// operator changes on frame *N* appears here on frame *N+1*. The panel
/// repaints continuously, so a filter that is on is reported as on within a
/// frame; what the lag rules out is reading a single frame's line as evidence
/// about a filter set in that same frame, which nothing does.
fn trace(doc: &OpenDoc, listing: &Listing, filter: &filter::Filter) {
    crate::diag::trace(|| {
        let ce = listing.rows.iter().filter(|r| r.is_ce_dimension).count();
        let suppressed = listing.rows.iter().filter(|r| r.suppressed).count();
        let unresolved = listing
            .rows
            .iter()
            .filter(|r| r.appearance_unresolved)
            .count();
        let replies = listing
            .rows
            .iter()
            .filter(|r| matches!(r.relation, Some(Relation::Reply)))
            .count();
        let group_members = listing
            .rows
            .iter()
            .filter(|r| matches!(r.relation, Some(Relation::GroupMember)))
            .count();
        // The canvas's own selection, matched against the rows this panel
        // drew — read, never set. See `body` for why that distinction is
        // load-bearing rather than pedantic.
        let picked = doc.selection.annot().map(|a| a.target.id);
        let selected = listing
            .rows
            .iter()
            .filter(|r| r.id.is_some() && r.id == picked)
            .count();
        let descriptions = listing
            .rows
            .iter()
            .filter(|r| matches!(r.note, Note::Description(_)))
            .count();
        format!(
            // ★ `selected` is the oracle for the canvas→panel link, and it is
            // the ONLY one available from outside the process: the mark on the
            // row is a word inside a heading string, which a trace cannot see
            // and which a screenshot can only confirm if the reader already
            // knows which row to look at. This number says the panel found the
            // annotation the canvas has — 0 or 1, never more, because
            // `SelectionState` holds one.
            "comments-panel pages={} listed={} with_note={} descriptions={} authors={} \
             ce_dimensions={ce} suppressed={suppressed} unresolved={unresolved} \
             replies={replies} group_members={group_members} selected={selected} \
             excluded_widgets={} excluded_popups={} excluded_trapnet={} excluded_total={} \
             filtered={} shown={}",
            doc.pages.len(),
            listing.rows.len(),
            listing.with_note_text(),
            descriptions,
            listing.rows.iter().filter(|r| r.author.is_some()).count(),
            listing.excluded.widgets,
            listing.excluded.popups,
            listing.excluded.trap_nets,
            listing.excluded.total(),
            u8::from(filter.is_narrowing()),
            listing.rows.iter().filter(|r| filter.keeps(r)).count(),
        )
    });
}

#[cfg(test)]
mod tests;
