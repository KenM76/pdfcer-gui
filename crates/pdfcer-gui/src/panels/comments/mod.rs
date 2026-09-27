//! # `panels::comments` — every annotation on this document, listed
//!
//! The comment list a reviewer works through. The classification lives in
//! [`model`]; this file is the drawing, the disclosures and the actions the
//! panel can raise.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/comments/mod.md`.

/// **Narrowing and ordering the work list** — the filter, the sort, and the
/// disclosure a filtered list owes. Its header carries the four things
/// Acrobat's Comment pane offers that this cannot, and which engine gap each is
/// filed under.
pub use pdfcer_gui_base::commentfilter as filter;

/// Turning a document into a comment list — the classification, testable
/// without a `Ui`.
pub use pdfcer_gui_base::commentmodel as model;
#[cfg(test)]
mod model_tests;

/// **A comment's review status** — `/State` and `/StateModel` (§12.5.6.3),
/// read into a per-reviewer history, shown on the row, filtered beside the
/// sort, and recorded through
/// `pdfcer_core::edit::EditSession::add_review_state`.
pub use pdfcer_gui_base::commentreviewstate as reviewstate;

/// The note being typed, and the `(annotation, edit epoch, destination)` stamp
/// that keeps it honest.
pub use pdfcer_gui_base::commentnote as note;

/// **Everything on a row that WRITES** — *Add note*, *Reply*, and the one text
/// box both of them open. Separate from this file under **R2**, and its header
/// carries the seam: this module is **the list**, that one is the only part of
/// the panel that holds the operator's unfinished words and the only part whose
/// output is a verb.
mod editor;

use pdfcer_core::object::ObjId;

use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::panels::comments as t;

pub(crate) use self::editor::keeps_author_name;
use self::editor::note_controls;
#[cfg(test)]
use self::editor::{keeps_author, reply_is_postable};
use self::model::{CommentRow, Listing, Note, Relation};
use self::note::NoteDraft;

/// The ribbon command that opens this panel.
pub const COMMAND_ID: &str = "markup.comments";

/// **The region the *Add note* / *Edit note* control publishes** — on the
/// FIRST row that offers one, and only that row.
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
/// The region the FIRST row's *Reply* publishes — one name, one row, for
/// [`REGION_EDIT`]'s stated reason.
pub const REGION_REPLY: &str = "comments.reply"; // ui-text-exempt: trace region name, never displayed
/// The region the open reply editor's *Post reply* publishes. Unique by
/// construction — one draft, one editor, one commit.
pub const REGION_POST: &str = "comments.reply_post"; // ui-text-exempt: trace region name, never displayed

/// Draw the Comments panel.
pub fn body(ui: &mut egui::Ui, doc: &OpenDoc, state: &mut PanelsState, actions: &mut Vec<Action>) {
    // FIRST, before anything is drawn: drop the operator's half-typed note if
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
    // per row would make the panel O(rows x sidecar).
    let ce_dimensions = model::ce_dimension_annots(&doc.session);
    // Read the SESSION, not the file on disk — see the module header.
    let view = doc.session.view();
    let listing = model::collect(&view, &doc.pages, &ce_dimensions);
    // ONCE per frame, for `ce_dimensions`' reason exactly: a review status
    // lives on OTHER annotations (§12.5.6.3), so answering "what is this
    // comment's status" needs the whole document. Asked per row it would make
    // the panel quadratic in the annotation count. Read from the same session
    // view for the same reason the listing is — a status recorded thirty
    // seconds ago and not yet saved must be on the row.
    let statuses = reviewstate::read(&view, &doc.pages);

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

    // EVERY DISCLOSURE SITS ABOVE THE LIST, without exception.
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

    // The filter strip, and then the rows it left. `crate::panels::comments::filter`
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
    // The status chooser, drawn as its own strip rather than inside
    // `filter_strip`, because its values come from the DOCUMENT's statuses
    // rather than from the rows — see `reviewstate::status_strip`. It writes
    // into the same `Filter`, so *Show all* lifts it and `is_narrowing` counts
    // it; that is the whole reason the state lives there and not beside it.
    reviewstate::status_strip(ui, &statuses, &mut state.comments_mut().filter.status);
    // Cloned out before the rows are borrowed, so the strip's `&mut` on the
    // panel state has ended by the time the list is drawn. A `Filter` is three
    // small fields; the alternative is threading a borrow through the whole
    // draw for nothing.
    //
    // Re-read rather than reusing `filter_now` from the top of the draw, and
    // that is not redundancy: `filter_strip` may have CHANGED the filter on
    // this very frame, and a chooser whose effect waited for the next repaint
    // would read as a control that does not work. The trace's copy is the
    // pre-strip one deliberately — see [`trace`].
    let narrowing = state.comments_mut().filter.clone();
    // TWO passes, because there are two questions. `filter::apply` answers
    // everything knowable from a row; `reviewstate::narrow` answers the one
    // thing that is not on the row at all — a status is on OTHER annotations
    // (§12.5.6.3). `Filter::status`' own doc carries why the state is in one
    // place and the predicate in two.
    let rows = reviewstate::narrow(
        filter::apply(listing.rows.clone(), &narrowing),
        &statuses,
        narrowing.status.as_ref(),
    );
    if narrowing.is_narrowing() {
        // ABOVE the list, with every other disclosure and for their reason:
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
    // The status a row asked to RECORD, and whether the Record-status region
    // has been published this frame. Two more scalars for `RowSink`'s reason:
    // two rows cannot be pressed in one frame, and a `Vec` would invite a
    // future reader to queue two edits that would each bump the epoch under the
    // other. Kept beside `verb` rather than inside `RowSink` so this feature
    // adds nothing to that struct — see `reviewstate::RowStatusCtx`.
    let mut status_verb: Option<(ObjId, pdfcer_core::edit::ReviewState)> = None;
    let mut status_published = false;
    // Whether the operator asked *which comments has nobody reviewed* — the one
    // condition under which an unreviewed row says so on its face. Read from
    // the filter that is already in hand rather than re-cloned.
    let unreviewed = matches!(
        narrowing.status,
        Some(reviewstate::StatusChoice::Unrecorded)
    );
    // The same, for the *Reply* control. See `RowSink::reply_published` for
    // why it is a second flag rather than a second use of the first.
    let mut reply_published = false;
    // Tallied through `RowSink` and published to the panel's own state after
    // the draw — see `CommentsUi::writing_controls_drawn`.
    let mut writing_controls_drawn: u32 = 0;
    let epoch = doc.edit_epoch;
    // Asked once per frame — see `RowSink::deletable`. A document-wide
    // question deserves one answer.
    //
    // **TWO independent questions, and neither answers the other.**
    //
    // `annotation_deletion_refusal` answers *"would `pdfcer-core` refuse this
    // document?"* — encrypted, certified. It says nothing at all about **what
    // stance the operator is in**, so a panel that asked only it would draw a
    // live Delete and a live note editor in **Read**, whose whole stated
    // posture is *the document is not yours to alter*.
    //
    // No test in this module can catch that, and that is a property of the
    // tests rather than an accident: they call the panel with an `OpenDoc` and
    // no stance at all, so a mode-dependent affordance is invisible to them.
    // Only a driven run in a named mode reports it — R1.
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
    // **The annotation the CANVAS has selected** — the other half of the
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
    // **This is not a second selection.** It is the canvas's own, read. The
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
            // The FILTERED rows. `last` is derived from the same vector the
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
                                reply_published: &mut reply_published,
                                deletable,
                                deletable_stance: authoring,
                                writing_controls_drawn: &mut writing_controls_drawn,
                            },
                            draft,
                            epoch,
                            is_selected,
                        );
                        // The review status, drawn INSIDE the same `push_id`
                        // so its chooser gets the row's own egui id — two rows
                        // of the same subtype on the same page would otherwise
                        // share one, which shows up as the wrong menu opening.
                        //
                        // Below the comment rather than above it: a status is a
                        // disclosure ABOUT the comment, and the panel's rule
                        // that disclosures come first is about caveats that
                        // change what you conclude from a LIST, not about ones
                        // that qualify a single row.
                        reviewstate::row_status(
                            ui,
                            comment,
                            &statuses,
                            reviewstate::RowStatusCtx {
                                authoring,
                                unreviewed_asked: unreviewed,
                                published: &mut status_published,
                                controls_drawn: &mut writing_controls_drawn,
                                verb: &mut status_verb,
                            },
                        );
                    })
                    .response;
                // Scrolled to on the frame the selection MOVES, and not while
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
    // Written back unconditionally, INCLUDING when nothing is selected — so
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
        // …and open that comment where it lives. See the module header on
        // why this is written directly rather than carried as an `Action`: an
        // `Action` drains after the frame, and the pop-up has to be open on
        // the frame the page arrives.
        //
        // **Resolved to the thread ROOT first.**
        // `crate::canvas::notepopup` draws a window for a
        // comment and lists that comment's replies inside it; it draws none for
        // a reply, because `add_reply` places a reply on its **parent's own
        // `/Rect`** and a bubble for it would sit on top of — and make
        // unclickable — the comment it answers. So a Go to on a reply row that
        // asked for the reply's own window would open nothing at all, and the
        // operator would press a button that visibly did half its job.
        //
        // `model::thread_root` walks `/IRT` upward, bounded against the cyclic
        // file §7.3.10 permits. On an ordinary comment it is the identity and
        // costs one lookup.
        if let Some(id) = id {
            let root = model::thread_root(&listing.rows, id);
            crate::canvas::notepopup::open::set(ui.ctx(), &doc.path, root, true);
        }
    }
    // Raised as its own `Action` rather than through `AnnotAction`, and the
    // seam is the one this feature keeps drawing: `AnnotAction`'s verbs all
    // change **the annotation named**, and this one changes nothing about it —
    // `add_review_state` *"returns a new `ObjId` rather than mutating the
    // target, and … nothing about the target changes"*. The draft is
    // deliberately left alone for the same reason: recording a status is not an
    // edit to the note the operator may be halfway through typing.
    if let Some((id, state)) = status_verb {
        actions.push(Action::RecordReviewState(
            crate::app::actions::reviewstate::RecordStatus { id, state },
        ));
    }
    if let Some(verb) = verb {
        // The draft closes here rather than in the row that raised the verb,
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
struct RowSink<'a> {
    /// **What a Go to press asks for** — the page, and the annotation on it.
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
    /// Whether [`REGION_REPLY`] has been published this frame — its own flag
    /// rather than a second use of [`Self::published`], because the two
    /// controls are drawn under different conditions.
    ///
    /// A row whose note editor is **open** draws no *Add note* and therefore
    /// publishes no [`REGION_EDIT`]; sharing one flag would let that row's
    /// state decide which row a harness finds *Reply* on. Two names, two
    /// flags, each naming the first row that actually drew the control it
    /// names.
    reply_published: &'a mut bool,
    /// **Whether `delete_annotation` would be refused right now**, asked
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

    // THE HEADING SAYS WHAT THE ANNOTATION ACTUALLY IS.
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
    // The selected row says so **in words**, on the heading line.
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

/// **Delete this comment.**
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
                // The ORDERING survives, and the narrowing does not. They
                // are different acts: *Show all* is the answer to "what am I
                // missing", and an operator who asked for the list by author
                // did not ask for that to be undone as well.
                state.author = None;
                state.subtype = None;
                state.with_note_only = false;
                // …and the status, which is why `Filter` holds it. A
                // narrowing the operator can set and cannot lift from the one
                // control labelled *Show all* is the trap version of a filter.
                state.status = None;
            }
        }
    });
}

/// One "All, or exactly this one" chooser over a list of document values.
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
fn sort_label(sort: filter::Sort) -> &'static str {
    match sort {
        filter::Sort::Document => t::comment_sort_document(),
        filter::Sort::Author => t::comment_sort_author(),
        filter::Sort::Subtype => t::comment_sort_subtype(),
    }
}

/// One `comments-panel` line per frame, carrying what the panel computed.
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
            // `selected` is the oracle for the canvas→panel link, and it is
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
