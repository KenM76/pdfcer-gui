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
//! ### The one place this build departs, and why
//!
//! The old shell also excluded **`/TrapNet`**, and its reason was
//! *delete-shaped*: core refuses a `/TrapNet` deletion by name, so listing one
//! *"would put a row here whose only possible action is a refusal, which is the
//! affordance R83 forbids."* **This build has no Delete** (see below), so that
//! reason does not reach.
//!
//! It is still excluded, on the half of the old argument that survives without
//! a Delete button: a `/TrapNet` is **prepress output state** — it records the
//! trapping a RIP applied to the page — so it is not a comment, nobody wrote
//! it, and there is nothing in it for a reviewer to work through. That is the
//! same shape as the `/Widget` exclusion: not "we cannot act on it" but "this
//! surface is not about it."
//!
//! What the departure buys instead is that **nothing is silently omitted**.
//! Every exclusion is counted and disclosed by
//! [`crate::text::panels::comments::comments_excluded`], so a reviewer looking
//! at six rows on a drawing they know carries forty annotations is told the
//! arithmetic and where each missing kind went. The old shell stated the rule
//! only on the empty case; this states the numbers on every case.
//!
//! **When a Delete lands here**, the old shell's `/TrapNet` reasoning becomes
//! live again as well and nothing needs to change: the row is already absent,
//! for a reason that does not depend on the button.
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
//! ## Actions, not mutations — and this panel raises exactly one
//!
//! [`Action::GoToPage`], from a row's **Go to** control, exactly as
//! `crate::panels::bookmarks` does. The body is handed `&OpenDoc` — a
//! **shared** reference, so this is a compile-time fact and not a convention —
//! it reads, and it pushes. It never touches the document.
//!
//! ### ★ There is no Delete, and its absence is a decision rather than a gap
//!
//! The old shell's panel could delete an annotation, with a hover-computed
//! collateral preview, a per-row `Locked` refusal and a document-wide
//! certification gate. **None of it is carried here**, because
//! `crate::app::actions::Action` has no variant that could carry the intent
//! and `app/actions.rs` is not this work's to extend.
//!
//! So the control renders **nothing at all**, which is the no-placeholders
//! rule (`HANDOFF.md` §6): *"A capability that is absent renders nothing,
//! never a greyed control that explains itself badly."* A disabled Delete
//! whose tooltip said "not built yet" would be the half-built surface
//! `crate::panels`' own header is about, and the strings for it are
//! deliberately absent from the catalog too — see
//! `crate::text::panels::comments`' header.
//!
//! What the day it lands needs, so nobody re-derives it, is written up in this
//! module's report to the shell owner: one `Action` variant, one dispatch arm
//! calling `EditSession::delete_annotation`, and the three disclosures
//! `docs/core-api/03-capabilities.md` §3.4 requires — *"delete is not
//! redaction"*, the deletion preview's collateral **before** the click, and
//! the fact that the preview *"is not a perfect oracle"* and the real call can
//! still refuse.
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
    // Asked ONCE per frame, never per row. `dimension_model` walks the catalog
    // to the `/PieceInfo` sidecar and deserializes it — cheap, and bounded by
    // the number of ce dimensions rather than by the document — but calling it
    // per row would make the panel O(rows x sidecar), which is the shape of
    // defect the old shell's hover-gated deletion preview was fixing.
    let ce_dimensions = model::ce_dimension_annots(&doc.session);
    // Read the SESSION, not the file on disk — see the module header.
    let view = doc.session.view();
    let listing = model::collect(&view, &doc.pages, &ce_dimensions);

    trace(doc, &listing);

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
    ui.separator();

    // Collected during the draw and applied after it — the actions-not-
    // mutations discipline at its smallest, and the same shape
    // `crate::panels::bookmarks` uses. One `Option`, not a `Vec`: two rows
    // cannot be clicked in one frame, and a `Vec` would invite a future reader
    // to push two navigations that would fight.
    let mut go: Option<usize> = None;
    // The document verb one row raised, if any. One `Option` for the same
    // reason `go` is one: two rows cannot be pressed in a single frame, and a
    // `Vec` would invite a future reader to queue two edits that would each
    // bump the epoch under the other.
    let mut verb: Option<AnnotAction> = None;
    // Whether the *Add note* region has been published this frame. See
    // [`REGION_EDIT`]: one name, one row, and the first row is the only
    // deterministic choice.
    let mut published = false;
    let epoch = doc.edit_epoch;
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
            let last = listing.rows.len() - 1;
            for (i, comment) in listing.rows.iter().enumerate() {
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
    ui_state.scrolled_to = if selected.is_some() {
        scrolled_to
    } else {
        None
    };

    if let Some(page) = go {
        actions.push(Action::GoToPage(page));
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
    /// The page a **Go to** press asks for.
    go: &'a mut Option<usize>,
    /// The document verb a Save or a Remove raised.
    verb: &'a mut Option<AnnotAction>,
    /// Whether [`REGION_EDIT`] has been published this frame — one name, one
    /// row, and the first row that offers the control is the only deterministic
    /// choice. See that constant.
    published: &'a mut bool,
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

    if ui
        .button(t::comment_row_goto())
        .on_hover_text(t::comment_row_goto_tooltip(page_number))
        .clicked()
    {
        *sink.go = Some(comment.page_index);
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
    comment
        .author
        .as_deref()
        .is_some_and(|author| !author.trim().is_empty())
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
fn trace(doc: &OpenDoc, listing: &Listing) {
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
             excluded_widgets={} excluded_popups={} excluded_trapnet={} excluded_total={}",
            doc.pages.len(),
            listing.rows.len(),
            listing.with_note_text(),
            descriptions,
            listing.rows.iter().filter(|r| r.author.is_some()).count(),
            listing.excluded.widgets,
            listing.excluded.popups,
            listing.excluded.trap_nets,
            listing.excluded.total(),
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::Panel;

    /// A row with the author this test is about and nothing else that matters.
    ///
    /// Built by hand rather than by `model::collect`, because the subject is a
    /// decision about **one field** and collecting a row would make the test
    /// depend on a document, an annotation and a walk — three things that can
    /// fail for reasons this assertion is not about.
    fn row_by(author: Option<&str>) -> CommentRow {
        CommentRow {
            page_index: 0,
            id: Some(pdfcer_core::object::ObjId {
                num: 7,
                generation: 0,
            }),
            subtype: "Square".to_owned(),
            is_ce_dimension: false,
            note: Note::Absent,
            author: author.map(str::to_owned),
            modified: None,
            suppressed: false,
            appearance_unresolved: false,
            relation: None,
        }
    }

    /// ★★★ **Correcting somebody else's typo must not re-attribute their
    /// comment.**
    ///
    /// The mistake `pdfcer-core` warned about by name when it shipped
    /// `set_markup_note`: writing all three keys unconditionally *"would
    /// silently strip the author and date on every correction, leaving a review
    /// comment from nobody, dated never, looking exactly like a note somebody
    /// else had mangled."*
    ///
    /// `true` here means the action sends **no `/T`**, which is what leaves the
    /// existing one alone.
    #[test]
    fn a_note_with_an_author_keeps_it() {
        assert!(keeps_author(&row_by(Some("Ken Mantle"))));
    }

    /// The other half, and it is the half that makes the first one mean
    /// something: a shape this shell drew has no byline, so a note written onto
    /// it is **ours to sign**.
    ///
    /// Asserting only the preservation case would pass on an implementation
    /// that never writes `/T` at all — every comment anonymous, which is the
    /// same defect wearing the other value.
    #[test]
    fn a_note_with_no_author_is_ours_to_sign() {
        assert!(!keeps_author(&row_by(None)));
    }

    /// ★ Whitespace is absent. A producer writing `/T ()` or `/T ( )` leaves a
    /// byline nobody wrote, and preserving it would credit the comment to a
    /// space — while the row's own byline, which trims the same way, would show
    /// nothing at all. Two surfaces, one rule.
    #[test]
    fn a_blank_author_is_no_author() {
        assert!(!keeps_author(&row_by(Some(""))));
        assert!(!keeps_author(&row_by(Some("   "))));
    }

    use crate::shell::{commands, manifest};
    use egui_shell::CommandRegistry;
    use std::collections::BTreeSet;

    /// **★ The command that opens this panel exists and is on the ribbon.**
    ///
    /// The check three panels in the old shell shipped without: they had a
    /// body, a rail entry and a diagnostic step, and *"no control an operator
    /// could click"*, so every verification passed while they were unreachable
    /// in a real build.
    ///
    /// Two assertions, and both are needed. A command **the manifest
    /// references** is one the ribbon draws a control for; a command **the
    /// registry holds** is one that has a label, a tooltip and an enable
    /// predicate. Either alone is half a control.
    ///
    /// `crate::panels::tests::every_panel_is_reachable_from_the_ribbon` sweeps
    /// the same property across every panel; this one names *this* panel in
    /// its failure message, which is what a reader who has just added it
    /// wants to see.
    #[test]
    fn the_comments_command_is_reachable_from_the_ribbon() {
        let shell = manifest::built_in();
        let mut registry = CommandRegistry::new();
        commands::register(&mut registry);
        let referenced: BTreeSet<String> = shell
            .command_references()
            .into_iter()
            .map(|(_, id)| id)
            .collect();

        assert!(
            referenced.contains(COMMAND_ID),
            "no tab, QAT slot or key binding references `{COMMAND_ID}`, so an \
             operator cannot open the Comments panel. `RIBBON_IA.md` §7 puts it \
             on Markup ▸ Comments."
        );
        assert!(
            registry.get(COMMAND_ID).is_some(),
            "`{COMMAND_ID}` is not registered, so the ribbon has an id with no \
             label, no tooltip and no enable predicate, and draws nothing for it."
        );
    }

    /// **The panel and this module name the same command.**
    ///
    /// Two spellings of one id is two things to keep in step, and the failure
    /// when they drift is a panel that opens from the ribbon and draws nothing
    /// in the dock — which looks like a rendering bug and is not.
    #[test]
    fn the_panel_enum_and_this_module_agree() {
        assert_eq!(Panel::Comments.command_id(), COMMAND_ID);
    }

    /// **★ The page index travels 0-based and prints 1-based.**
    ///
    /// The off-by-one that would otherwise be invisible.
    /// [`crate::app::actions::Action::GoToPage`] takes a 0-based index — the
    /// same convention `crate::panels::bookmarks` pins from its own side — and
    /// every string a human reads takes the number one higher. Getting it
    /// backwards produces a panel that navigates one page past every comment,
    /// which looks like a document defect.
    ///
    /// Asserted against a real fixture rather than a constructed row, so the
    /// indices are ones the collector actually produced.
    #[test]
    fn the_page_index_travels_zero_based_and_prints_one_based() {
        use crate::panels::objects::test_support::engine_fixture;

        let path = engine_fixture("annot/thread.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        let session = pdfcer_core::edit::EditSession::new(doc);
        let listing = model::collect(
            &session.view(),
            &pages,
            &model::ce_dimension_annots(&session),
        );
        assert!(
            !listing.rows.is_empty(),
            "the fixture must carry annotations, or this test proves nothing"
        );

        for comment in &listing.rows {
            // What the row would push …
            let action = Action::GoToPage(comment.page_index);
            assert_eq!(action, Action::GoToPage(comment.page_index));
            // … and what it prints, which is one higher, in both the heading
            // and the button's tooltip.
            let human = comment.page_index + 1;
            let heading = t::comment_row_heading(&comment.subtype, human);
            assert!(heading.contains(&human.to_string()), "{heading}");
            let tip = t::comment_row_goto_tooltip(human);
            assert!(tip.contains(&human.to_string()), "{tip}");
        }
    }

    /// **A ce dimension's heading names it as one and keeps the subtype.**
    ///
    /// Rule 15 at the point of use. The bracketed `/Line` is not decoration:
    /// the exclusion argument in this module's header turns on ce dimensions
    /// *being* `/Line` annotations, and a heading that hid that would quietly
    /// contradict the argument that put the row in the list.
    #[test]
    fn a_ce_dimension_row_says_ce_dimension_and_still_says_line() {
        let heading = t::comment_row_ce_dimension_heading("Line", 3);
        assert!(heading.contains("ce dimension"), "{heading}");
        assert!(heading.contains("Line"), "{heading}");
        // …and an ordinary `/Line` markup is not relabelled.
        let plain = t::comment_row_heading("Line", 3);
        assert!(!plain.contains("dimension"), "{plain}");
    }
}
