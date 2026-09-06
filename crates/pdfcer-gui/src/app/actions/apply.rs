//! # `app::actions::apply` — the other end of the funnel: what an [`Action`]
//! actually does
//!
//! [`super`] declares the **vocabulary** — one variant per thing an operator
//! can ask for, each carrying a complete statement of that intent. This file is
//! the **interpreter**: [`PdfcerApp::apply_actions`] drains the frame's queue,
//! [`PdfcerApp::apply`] routes one intent to one state transition, and
//! [`vector_edit`] is the four-step protocol every arm that changes a document
//! goes through.
//!
//! ## ★ Why this is its own file
//!
//! `app/actions.rs` crossed the 1,500-line gate (standing rule **R2**) when
//! `file.save_copy` was wired, and the rule's own justification decides where
//! the cut goes rather than the line count: *"the value of the limit is that
//! the file has to have a single subject."*
//!
//! Two subjects were sharing one file, and they change for entirely different
//! reasons:
//!
//! | | subject | what changes it |
//! |---|---|---|
//! | [`super`] | **what an operator can ask for**, and what each request must carry to remain resolvable after the frame that raised it | a new command with a new operand |
//! | here | **what happens when one is granted**, and the ordering that makes a mutation safe | a new engine verb, or a change to the cancel-mutate-bump-invalidate protocol |
//!
//! It is the same seam `app/mod.rs` has been split along four times —
//! `dispatch.rs` (*what does this verb do*), `conditions.rs` (*what is true
//! right now*), `gating.rs` (*what may this mode do*), `panels.rs` — and the
//! same one `app/state.rs` was split along to produce `lifecycle.rs`. The test
//! for whether a split was along a seam is whether the tests came with it, and
//! they did: both tests below drive [`vector_edit`], and neither reads the
//! [`Action`] enum at all.
//!
//! ## What did NOT move, and why
//!
//! The **edit-disclosure store** stayed in [`super`], beside the type it holds,
//! even though [`vector_edit`] is its only writer. Two reasons: it is read by
//! `crate::app::status` through [`super::last_edit_disclosure`], which makes it
//! part of this module tree's published surface rather than an implementation
//! detail of applying; and Rust's own visibility rule means a child module can
//! reach an ancestor's private items, so `record_edit_disclosure` is callable
//! here without being made `pub` to anybody else. Splitting a store from its
//! type to follow its writer would have been the tidier-looking edit and the
//! one that widened a private thing's visibility for no gain.

use pdfcer_core::vector::MarqueeMode;

use super::forms::FieldAction;

use super::Action;
use crate::app::PdfcerApp;
use crate::app::state::Status;
use crate::viewer;

impl PdfcerApp {
    /// Apply every action raised during the frame just drawn.
    ///
    /// Applied in the order raised. `pixels_per_point` is passed in rather
    /// than read from a context because the per-page zoom ceiling depends
    /// on it — see [`viewer::max_zoom_for_page`] — and threading it makes
    /// this function pure with respect to egui, which is what keeps it
    /// reviewable.
    pub fn apply_actions(&mut self, actions: Vec<Action>, pixels_per_point: f32) {
        for action in actions {
            self.apply(action, pixels_per_point);
        }
    }

    /// Apply a single action.
    ///
    /// Every arm is a state transition on [`crate::viewer::ViewState`],
    /// which is where the clamping and the ladder arithmetic live and are
    /// tested. This function decides *which* transition, never *what it
    /// means* — a zoom that saturates, a page step that stops at the last
    /// page and a NaN that falls back to actual size are all decided in
    /// `viewer`, under unit test.
    fn apply(&mut self, action: Action, pixels_per_point: f32) {
        // ★ The three actions that are about WHICH document is open, matched
        // BEFORE the guard below.
        //
        // Every other arm acts on the open document, so the guard's "no
        // document: silently drop" is the right answer for all of them. It is
        // the wrong answer for these three, and in opposite directions: an Open
        // or a New with nothing open is the *ordinary* case — it is how the
        // operator gets their first document after launching with no argument —
        // and a Close with nothing open is a no-op that must still not be
        // reached through a path that assumes a document.
        //
        // All three consult `save_pending` first. See its own docs for the
        // rule; the short version is that an Open must not run out from under a
        // save, and that the one save this build has — `file.save_copy`, applied
        // two arms down — is **synchronous**, so it is never in flight across a
        // frame and there is nothing for these three to run out from under. It
        // is emphatically not "are there unsaved edits?"; see
        // `crate::app::save` section 3.
        //
        // Two more arms follow them for different reasons — `SaveCopy` because
        // the guard's silence is wrong for a chord, `Find` because of a borrow —
        // and each says so at its own site.
        //
        // The `_ => {}` arm moves nothing, and every arm that moves `action`
        // returns, so the value is still whole below. That is a property of the
        // control flow rather than a coincidence: adding an arm here that fell
        // through would be a use-after-move and the compiler would say so.
        match action {
            // ★★ The four arms that replace the open document live in
            // `super::document`, one function each, with the two guards they
            // share and the table that orders them.
            //
            // Moved there on 2026-08-19 when this file crossed R2's 1,500-line
            // gate. The seam was already drawn in prose above — *"the actions
            // that are about WHICH document is open"* — and it is a real one:
            // everything below acts ON the open document, and these four decide
            // WHICH document is open, or whether there is one. Different
            // subject, different failure mode. An arm below can be wrong about
            // a page; one of these can be wrong about an afternoon's work.
            //
            // They stay listed here, by name, rather than behind a single
            // catch-all, because this `match` is the one place a reader can see
            // the whole action vocabulary in order.
            Action::Open(path) => {
                self.apply_open(path);
                return;
            }
            // Beside `Open`, and returning like it: both replace what is on
            // screen with a freshly loaded document, so neither may fall through
            // to the edit-epoch bookkeeping below — there is no edit here.
            Action::OpenWithPassword { path, password } => {
                self.apply_open_with_password(path, &password);
                return;
            }
            Action::New => {
                self.apply_new();
                return;
            }
            Action::NewSized {
                width_pt,
                height_pt,
            } => {
                self.apply_new_sized(width_pt, height_pt);
                return;
            }
            Action::Close => {
                self.apply_close();
                return;
            }
            // ★ Beside `Close` and returning like it, for `Close`'s own
            // reason: nothing here is an edit, so it must not fall through to
            // the edit-epoch bookkeeping below. It raises a question and
            // returns; the acts it leads to happen in the drain.
            Action::OpenInAcrobat => {
                self.apply_open_in_acrobat();
                return;
            }
            // ★ Beside the four above rather than below the document guard,
            // and for a third reason again: this arm needs `&mut self`, not
            // `&mut OpenDoc`. It reads a **parked** document's session while
            // it writes the active one's, which is a borrow the guard below
            // cannot express — everything past it has already narrowed `self`
            // to one document.
            Action::CloseDocument(slot) => {
                self.apply_close_document(slot);
                return;
            }
            Action::CloseOtherDocuments(keep) => {
                self.apply_close_other_documents(keep);
                return;
            }
            Action::InsertPagesFromOpenDocument {
                source_slot,
                pages,
                position,
                take,
            } => {
                self.apply_insert_from_open_document(source_slot, &pages, position, take);
                return;
            }
            // ★ Save a copy — matched here for the guard's own reason rather
            // than for a borrow one, and it is the third kind of case this
            // pre-guard block holds.
            //
            // It needs `&self.status` and nothing else, so it could have sat
            // below with the rest. It is here because the guard's answer for
            // "no document" is a **silent** drop, and that is the wrong answer
            // for a command bound to `Ctrl+S`: a keymap reaches any command
            // from any state, and an operator who presses the save chord over
            // an empty shell must not be indistinguishable, in the trace, from
            // one whose keystroke never arrived. `Action::Find` is here for the
            // same half of the same argument.
            //
            // The arm routes and does not compute. Everything about what a save
            // IS — the suggested name, the picker, the incremental mode, the
            // `SaveOptions`, the write, what happens to the epoch (nothing) —
            // lives in `crate::app::save`, which is also where the reason this
            // runs in the apply phase rather than in the dispatcher is written
            // down. Note what it does NOT do: `vector_edit`. A save reads the
            // session (`to_incremental_bytes(&self)`), so there is no worker to
            // cancel, no `Arc::get_mut` to fail, no epoch to bump and no texture
            // to drop — see `save`'s section 2.
            // ★ Save-in-place. A blank document with no file behind it never
            // reaches here: `dispatch` routes it to `file.save_copy` first,
            // because *"where does this go?"* is only the operator's to answer.
            // ★★ **The three saves**, all matched here for the same two
            // reasons: each asks a signature question that must run before the
            // document guard, and each returns rather than falling through.
            //
            // Their bodies are in `crate::app::actions::saving`, split out on
            // 2026-09-02 under R2 — the same seam `destination` and `view`
            // already are. Nothing about a save is decided in this file, which
            // is this arm's whole shape: it is routing.
            Action::Save | Action::SaveCopy | Action::SaveAs => {
                super::saving::apply(self, &action);
                return;
            }
            // ★ The third arm matched before the document guard, and it is
            // here for a **borrow** reason rather than for the guard's.
            //
            // Applying a find request needs two of this struct's fields at
            // once — `self.find` and the open document inside `self.status` —
            // and the guard below takes `&mut self.status` for the rest of the
            // function, after which `self.find` is unreachable. Splitting the
            // borrow has to happen while `self` is still whole, which is here.
            //
            // It is *also* correct on the guard's own terms: with nothing
            // open there is nothing to search, and saying so on the trace is
            // more useful than the guard's silent drop, because a keymap can
            // reach `edit.find` from any state and "the chord did nothing" and
            // "the chord did nothing because no document is open" need
            // different responses from whoever is reading the trace.
            Action::Find(request) => {
                match &mut self.status {
                    Status::Open(doc) => crate::find::apply(&mut self.find, doc, request),
                    _ => crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        format!("find-declined request={request:?} reason=no-document")
                    }),
                }
                return;
            }
            _ => {}
        }

        let Status::Open(doc) = &mut self.status else {
            // No document: nothing to zoom, nothing to navigate. Silently
            // dropping the action is correct rather than lax — the controls
            // that raise these do not exist without a document, so reaching
            // here at all would mean a keyboard binding was installed
            // without its guard.
            return;
        };

        // The per-page raster ceiling, recomputed here rather than cached
        // because it depends on BOTH the current page's extent and the
        // display's density, either of which can change between frames (a
        // page step, a window dragged to a different monitor). Caching it
        // is how a guard passes its tests and still lets the operator zoom
        // into an allocation failure on the one machine that matters.
        // ★★ O24: the ceiling now honours the operator's configured maximum.
        // `zoom_ceiling` is the one place the whole-page limit and the region
        // tier are reconciled, so this site and `canvas::zoom` cannot answer
        // the question differently.
        let max_zoom = viewer::zoom_ceiling(
            doc.current_extent(),
            pixels_per_point,
            self.prefs.max_zoom_percent,
        );
        let page_count = doc.pages.len();

        // ★ Which zoom changes are DISCRETE, and why that matters.
        //
        // `settle_and_rasterize` debounces a zoom by 150 ms so a Ctrl+wheel
        // gesture — which emits dozens of intermediate values — rasterizes
        // once rather than dozens of times. A discrete *command* has no
        // gesture in flight, so waiting out that debounce would make a
        // keypress feel unresponsive for no benefit.
        //
        // So every zoom-changing arm except `ZoomBy` (the wheel path) sets
        // this flag. Getting it backwards is not a crash: it is a keyboard
        // zoom that lags by 150 ms, or a wheel gesture that re-rasterizes a
        // CAD sheet on every notch. Both were real behaviours in the old
        // shell's history, which is why the distinction is a named flag
        // rather than an inline condition.
        // `|=`, not `=`: several actions can be raised in one frame, and a
        // later non-zoom action must not clear a flag an earlier zoom
        // command set. `settle_and_rasterize` clears it once per frame,
        // which is the only place it may be cleared.
        //
        // Matched on a REFERENCE since `Action` stopped being `Copy` (see the
        // module docs): `matches!` moves its scrutinee, and the `match` below
        // needs the value.
        // `ZoomTo` belongs here and was missing when the variant landed —
        // the comment above already said "every zoom-changing arm except
        // `ZoomBy`", so the list and its own description disagreed. The
        // symptom was quiet: Actual size, from the button *and* from Ctrl+0,
        // waited out the 150 ms wheel-settle debounce before re-rastering,
        // as though it were a continuous gesture. A discrete command should
        // commit at once.
        doc.zoom_commanded |= matches!(
            &action,
            Action::ZoomIn | Action::ZoomOut | Action::Fit(_) | Action::ZoomTo(_)
        );

        match action {
            // Handled above, before the guard that needs an open document —
            // which is the point, since three of them are how a document becomes
            // open. Spelled out rather than folded into a catch-all so that a
            // new variant added to the enum still fails to compile here.
            // ui-text-exempt: a panic message, read from a stack trace by
            // whoever moved one of these two arms. Never rendered.
            Action::Open(_)
            | Action::OpenWithPassword { .. }
            | Action::New
            | Action::NewSized { .. }
            | Action::Close
            | Action::OpenInAcrobat
            | Action::CloseDocument(_)
            | Action::CloseOtherDocuments(_)
            | Action::InsertPagesFromOpenDocument { .. }
            | Action::Save
            | Action::SaveCopy
            | Action::SaveAs
            | Action::Find(_) => {
                // ui-text-exempt: a panic message, read from a stack trace by
                // whoever moved one of these six arms. Never rendered.
                unreachable!("handled before the document guard")
            }
            // ★★ A row click in the Objects panel, arriving as an action for
            // the reason `Action::SelectObject`'s own docs give: a panel body
            // holds `&OpenDoc`, not `&mut`, so a panel that changes something
            // asks rather than writes.
            //
            // Here rather than before the document guard, because it needs the
            // document and has no reason to run without one — the pre-guard
            // match is for the actions that *make* a document open.
            //
            // ★ No `vector_edit`, no epoch bump, no cache invalidation: **a
            // selection is not an edit.** It names parts of a document and
            // changes nothing a save would write. `canvas`'s header makes that
            // argument for the canvas selection; this is the same argument
            // arriving from the other end of the same selection.
            Action::SelectObject { page, object } => match object {
                Some(object) => doc.selection.select_only(page, object, "objects-panel"),
                None => {
                    doc.selection.clear();
                }
            },
            // ★ A canvas gesture that refused, asking for its sentence. It
            // changes nothing about the document — see the variant's docs for
            // why it is an action at all, and why it carries no payload.
            Action::DeclineInsideForm => crate::app::status::decline::record_inside_form(),
            Action::ZoomBy(factor) => doc.view.zoom_by(factor, max_zoom),
            Action::ZoomIn => doc.view.zoom_in(max_zoom),
            Action::ZoomOut => doc.view.zoom_out(max_zoom),
            // ★★ **The seven view verbs**, in one arm and one function.
            //
            // Fit, zoom, the three page steps and a bookmark destination
            // are one subject — *where the operator is looking* — and none
            // of them touches the document. Split under R2 on 2026-09-01,
            // and the seam is a real one rather than a line count: every
            // other arm in this match changes a PDF.
            Action::Fit(_)
            | Action::ZoomTo(_)
            | Action::NextPage
            | Action::PrevPage
            | Action::GoToPage(_)
            | Action::GoToDestination(_) => {
                super::view::apply(doc, action, page_count, max_zoom);
            }
            // ★ Every geometry verb, routed. The body is in
            // [`super::vector`], beside the enum, which is the pattern
            // [`super::dimensions`] already sets one arm below — and it is R2's
            // answer for this file as much as for `action.rs`: the seven arms
            // were 120 lines of a match that is otherwise a routing table.
            Action::Vector(action) => super::vector::apply(doc, action),
            // ★ One markup annotation, through the same four-step protocol
            // every other document change uses.
            //
            // The arm routes; it does not compute. `markup::spec` is a pure,
            // unit-tested function of the kind and the two endpoints — which is
            // where the per-kind normalisation rule lives, and where it must
            // live, because "an arrow keeps its raw endpoints" is a rule with a
            // test rather than a line of wiring.
            //
            // `.map(|_| Vec::new())` adapts `add_markup`'s `ObjId` to the
            // disclosure list `vector_edit` traces, and the empty vec is a
            // statement rather than a placeholder: authoring an annotation
            // rewrites no existing operator, so there is nothing whose *form*
            // changed and therefore nothing rule 4 obliges us to disclose. The
            // new object's id is discarded because nothing here addresses it —
            // the Comments panel that will is a separate surface with its own
            // way of finding annotations on a page.
            // ★ Every ce-dimension verb, routed. The body is in
            // `super::dimensions` — a sibling of `annots` and `pages`, split
            // out under R2 along the same seam — because the family shares a
            // rule this file cannot express in one arm: four of the eight
            // rewrite every member of a group across every page, and four
            // touch one annotation. See that module's header.
            //
            // ★ The `CommitMarkup` arm below is deliberately NOT routed with
            // it, even though authoring a ce dimension and authoring a markup
            // look like the same act. They share no invalidation rule — a
            // markup has no group whose other members could move — and folding
            // them together would put a document-wide raster clear one careless
            // edit away from every markup placed on the canvas.
            Action::Dimension(action) => super::dimensions::apply(doc, action),
            // ★ One image, one undo entry, and every disclosure it owes.
            //
            // The closure's return value IS the disclosure list — the funnel's
            // own mechanism — and here it carries the three facts an operator
            // cannot see at editing zoom: the effective resolution, whether the
            // picture kept its shape, and whether pdfcer re-encoded the source
            // rather than storing the file's own bytes. A picture placed at
            // 12 dpi and one placed at 300 look identical on screen and
            // different on paper, which is why the number is stated every time
            // rather than only when it is bad.
            //
            // ★ `recompressed` reaches the catalog through its own `Display`.
            // Same call `text::images::import_failed` makes, for the same
            // reason: the value is a SPECIFIC explanation — a colour model PDF
            // has no space for, a filter pdfcer cannot re-emit — and a catalog
            // sentence would have to discard the half that is the whole answer.
            Action::InsertImage {
                page,
                rect,
                fit,
                image,
            } => {
                vector_edit(doc, "add-image", page, 1, |session| {
                    // ★ The builder, not a struct literal: `NewImage` is
                    // `#[non_exhaustive]`, so a downstream crate cannot
                    // construct it field-by-field — and the constructor is
                    // what keeps a field added upstream from silently
                    // defaulting here.
                    let spec = pdfcer_core::edit::NewImage::new(page, rect, &image);
                    let spec = match fit {
                        pdfcer_core::edit::ImageFit::Stretch => spec.stretching(),
                        // `Contain` is the constructor's own default, and the
                        // wildcard is forced by `#[non_exhaustive]` rather than
                        // chosen. A third fit mode pdfcer gains would land here
                        // as Contain, which is the safe direction: it never
                        // distorts a picture the operator did not ask to
                        // distort.
                        _ => spec,
                    };
                    session.add_image(&spec).map(|outcome| {
                        let d = &outcome.disclosures;
                        crate::text::images::placement_disclosures(
                            d.effective_dpi,
                            d.below_screen_resolution,
                            d.letterboxed,
                            d.aspect_distorted,
                            d.recompressed,
                            d.source_bytes,
                            d.stored_bytes,
                        )
                    })
                });
                // ★★★ **And it arrives SELECTED** — 2026-08-26, closing the
                // operator's *"if I add an image I Expect to click on it to
                // resize but dragging doesn't resize."*
                //
                // He was right about the symptom and it was never the resize: a
                // driven check had already proved a selected image resizes from
                // a corner grip and moves from a body drag. It arrived
                // unselected, so his first press was a press on unselected
                // paper, and `gesture::meaning` reads that as a marquee.
                //
                // ★ The new image is the LAST object in paint order, because
                // `add_image` appends to the content stream — so its target is
                // the decomposition's final index. Taken from the rebuilt model
                // rather than from a count kept before the edit: the edit
                // invalidated the cache, `page_objects()` rebuilds it against
                // the new epoch, and a remembered count would be a count of the
                // page as it was.
                //
                // A placement that produced no model — a page whose content
                // stream will not decompose — simply leaves the selection
                // alone. The image is still on the page; what is missing is the
                // shell's ability to name it, and inventing an index for it
                // would select whatever happens to be at that position.
                //
                // ★ The count is taken and the borrow released in one
                // statement, before the selection is touched. `page_objects()`
                // hands back a `Ref` into the document's own cache, and
                // `select_placed` wants `&mut doc.selection` — holding the
                // first across the second does not compile, which is the
                // borrow checker enforcing the short-borrow discipline the
                // panels already keep by hand.
                let count = doc
                    .page_objects()
                    .map(|provider| provider.page_objects().objects.len());
                if let Some(last) = count.and_then(|n| n.checked_sub(1)) {
                    doc.selection
                        .select_placed(page, crate::canvas::target::TargetId::Object(last as u64));
                }
            }
            // ★ One dictionary entry, through the same four-step protocol as a
            // page rewrite — because the protocol is what makes an edit
            // undoable, epoch-bumping and cache-invalidating, and a shortcut
            // for a "small" edit is how one edit ends up outside the log.
            //
            // Page `0` with a note, as every document-scoped verb here passes:
            // `/Info` is in the trailer and belongs to no page. The page
            // reaches `vector_edit` only for the trace line.
            //
            // ★★★ **A completed recognition, applied as one edit.**
            //
            // See `Action::ApplyOcr` for why the dialog cannot do this itself
            // and why the whole run is one command. What is worth reading here
            // is the shape: this is an ordinary `vector_edit`, identical to the
            // seventeen above it, which is the entire point of the engine Pass
            // that made it possible. Recognition used to be the one capability
            // in this program that was not an edit.
            Action::ApplyOcr { pages } => {
                // The borrowed view the engine's slice wants, built here so the
                // owned `OcrPage`s outlive it. `page` for the trace is the
                // first one touched; the count is what makes the line useful.
                let first = pages.first().map_or(0, |(index, _)| *index);
                let count = pages.len();
                vector_edit(doc, "ocr-layer", first, count, |session| {
                    let layers: Vec<pdfcer_core::edit::OcrPageLayer<'_>> = pages
                        .iter()
                        .map(|(index, recognised)| pdfcer_core::edit::OcrPageLayer {
                            page_index: *index,
                            recognised,
                        })
                        .collect();
                    session
                        .add_ocr_layer(&layers, &pdfcer_core::ocr::layer::OcrLayerOptions::new())
                        // ★ Every page's disclosures, flattened onto the one
                        // channel every other edit reports on. The dialog does
                        // NOT re-render them: two accounts of one run, worded
                        // differently, is a pair that drifts.
                        .map(|reports| {
                            reports
                                .iter()
                                .flat_map(pdfcer_core::ocr::layer::OcrLayerReport::disclosures)
                                .collect()
                        })
                });
            }
            // ★ Nothing is invalidated beyond the epoch, deliberately. Document
            // metadata is not drawn on any page, so clearing rasters would
            // throw away every cached page to no purpose — the one arm in this
            // file where the *absence* of an invalidation is the decision.
            Action::SetInfoField { field, value } => {
                vector_edit(doc, "set-info-field", 0, 1, |session| {
                    session
                        .set_info_field(field, value.as_deref())
                        .map(|()| Vec::new())
                });
            }
            // ★ The `Option` is `markup::Refusal::Mismatched` arriving at the
            // only place it can: a kind holding another family's geometry.
            //
            // Unconstructible from a gesture — `markup::action` refuses the pair
            // before an `Action` exists — and handled rather than unwrapped,
            // because an `Action` is plain data that a test can build and a
            // future undo/redo surface could replay. Declining by name beats a
            // panic in the frame that is trying to draw, and it emphatically
            // beats authoring a shape nobody asked for.
            // ★★★ **EVERY ANNOTATION VERB, IN ONE ARM** — moved out on
            // 2026-09-05 under **R2**, when the three node verbs took this file
            // past 1,500 lines.
            //
            // The seam is this file's own, stated in its header and drawn
            // twice already (`super::pages`, `super::annots`): **this file
            // ROUTES and the family module DECIDES.** Eleven arms that each
            // destructured one variant and called one function two lines long
            // were routing spelled out eleven times; `annots::apply_action` is
            // the same routing in the file that owns the subject, where the
            // disclosure rules that govern all eleven are stated once.
            //
            // ★ The author name travels rather than being read there.
            // `keep_author` is a fact about the DOCUMENT the raising surface
            // had in front of it, and the name is a fact about the OPERATOR
            // that only this scope can see — `AnnotAction::SetNote`'s own field
            // doc carries that split, and `annots::set_note` still applies it.
            Action::Annot(action) => {
                super::annots::apply_action(doc, action, self.prefs.author_name.trim());
            }
            // ★ A paste is an `add_markup` and nothing more, which is the
            // whole reason this feature was buildable at all: the spec that
            // came off the clipboard is the same shape the authoring path
            // already hands the engine, so there is one call site's worth of
            // new code and no second notion of what a markup is.
            //
            // The displacement is already IN the spec — `clipboard::paste`
            // translates it before raising this — so `dx`/`dy` here are carried
            // for the trace and the disclosure only. They are not applied
            // twice, and the field docs say so.
            Action::PasteMarkup {
                page,
                spec,
                dx,
                dy,
                options,
            } => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // ★ `note` and `ca` are traced because they are the two
                    // things a lossy paste gets wrong while LOOKING correct: a
                    // comment with no words lives in a pop-up this shell does
                    // not draw, and an opacity difference is only visible
                    // against the artwork underneath. A screenshot cannot tell
                    // a faithful paste from a lossy one; this line can.
                    format!(
                        "paste-markup-requested page={page} dx={dx:.1} dy={dy:.1} \
                         note={} ca={:?}",
                        options.note.is_some(),
                        options.opacity
                    )
                });
                vector_edit(doc, "paste-markup", page, 1, |session| {
                    session
                        .add_markup_with(page, &spec, &options)
                        .map(|_| Vec::new())
                });
            }
            Action::CommitMarkup {
                page,
                kind,
                geometry,
                pen,
            } => {
                if let Some(spec) = crate::canvas::markup::spec(kind, &geometry, pen) {
                    // ★★★ `add_markup_with`, not `add_markup` — ONE verb and ONE
                    // undo entry for a translucent mark.
                    //
                    // This is the engine's own argument, and it is a defect
                    // argument rather than a convenience one. Authoring at an
                    // opacity used to take two calls (author, then restyle), and
                    // that is two undo entries: *"an operator who draws a
                    // translucent highlight and presses Ctrl+Z once gets an
                    // OPAQUE highlight, not no highlight. That is a state they
                    // never asked for and cannot have created any other way."*
                    //
                    // ★ `opacity_option()` answers `None` at fully opaque, which
                    // writes no `/CA` at all — so a build whose operator never
                    // touches the control authors the same bytes it did before
                    // the control existed.
                    let options = pdfcer_core::edit::MarkupOptions {
                        opacity: pen.opacity_option(),
                        ..Default::default()
                    };
                    vector_edit(doc, "add-markup", page, 1, |session| {
                        session
                            .add_markup_with(page, &spec, &options)
                            .map(|_| Vec::new())
                    });
                } else {
                    crate::canvas::markup::decline(
                        kind,
                        page,
                        crate::canvas::markup::Refusal::Mismatched,
                    );
                }
            }
            // ★ Placing a text-bearing annotation CHANGES NOTHING. It opens the
            // dialog, and the words decide whether anything is authored.
            //
            // It is in this file rather than handled at the canvas, because the
            // canvas may not reach `PdfcerApp` — everything a gesture wants the
            // application to do arrives here as an `Action`, and "open a
            // window" is no exception to that just because it touches no
            // document.
            // ★ Selecting a form field changes no document — it is view
            // state, and it deliberately does NOT reach `vector_edit`, bump the
            // epoch or invalidate a page. It is here rather than mutated at the
            // canvas because that is this shell's one rule: everything a
            // gesture wants the application to do arrives as an `Action`.
            // ★ The form-field family, and it is routed in THREE arms rather
            // than one. The enum moved to `super::forms` on 2026-08-27 under
            // R2; the bodies had lived there since the family shipped. What
            // could not follow is the two verbs that need application state
            // this file holds and `&mut OpenDoc` does not — `self.dialogs`,
            // `self.form_defaults` and `self.status`.
            //
            // ★★ And they cannot be handed to a router either, for a borrow
            // reason worth stating because it is invisible until you try it:
            // `doc` above IS `&mut self.status`. Passing `doc` and
            // `&self.status` to one function is two borrows of one field and
            // will not compile. It works *inside* an arm only because NLL ends
            // `doc`'s borrow at its last use, which is `field_names(doc)` on
            // the line before. That is a real constraint on how far this
            // family's arms can be moved, not an accident of style.
            //
            // Placing a form control CHANGES NOTHING, exactly as placing a
            // text-bearing annotation does. It opens the dialog, and the
            // details decide whether anything is authored. The document is read
            // HERE rather than at the canvas, because generating the field's
            // name needs the existing ones and a gesture has no business
            // parsing an `/AcroForm`.
            Action::Field(FieldAction::Begin { page, kind, rect }) => {
                let existing = super::forms::field_names(doc);
                let draft = self.form_defaults.next(kind, &existing);
                self.dialogs
                    .open_form_field(&self.status, page, rect, draft);
            }
            // …and this is the one that reaches the document, through the same
            // `vector_edit` funnel every other authoring verb uses.
            //
            // ★★ The settings are remembered on the way past — the operator's
            // *"remember last settings"* — and remembered HERE rather than in
            // the dialog, because this is the point at which they were
            // ACCEPTED. Remembering at the dialog would remember a draft the
            // operator then cancelled.
            Action::Field(FieldAction::Commit { page, rect, draft }) => {
                self.form_defaults.remember(&draft);
                super::forms::author::author(doc, page, rect, &draft);
            }
            // ★ The restyle family, routed like every other sub-verb. Its arm
            // is one line and its module is four hundred, which is the right
            // proportion: what happens here is a decision about WHICH runs and
            // in WHAT ORDER, and neither is a fact this file knows.
            Action::TextStyle { page, runs, change } => {
                super::textstyle::apply(doc, page, &runs, &change);
            }
            // Everything else in the family needs the document and nothing
            // else, so it routes the way `Vector`, `Dimension` and `Page` do.
            Action::Field(action) => super::forms::apply(doc, action),
            Action::BeginTextAnnot { page, kind, rect } => {
                self.dialogs.open_text_annot(&self.status, page, kind, rect);
            }
            // …and this is the one that reaches the document, through the same
            // `vector_edit` funnel every other authoring verb uses. The engine
            // verb differs (`add_text_annotation` rather than `add_markup`)
            // because the spec type does; nothing else about the protocol does.
            Action::CommitTextAnnot {
                page,
                kind,
                rect,
                text,
                stamp,
            } => super::textannot::commit(
                doc,
                &self.prefs,
                &super::textannot::Placement {
                    page,
                    kind,
                    rect,
                    stamp,
                },
                &text,
                self.pen.ink,
                self.pen.opacity_option(),
            ),
            // ★ One text markup, through the SAME funnel and the same engine
            // verb as the drag-shaped kinds above.
            //
            // The label differs (`add-text-markup`) and nothing else does, which
            // is the point: a `/Underline` and a `/Square` are one
            // `add_markup(page, &spec)` apart, so the four-step protocol, the
            // undo entry, the epoch bump and the texture drop are not written a
            // second time for a second family of annotation.
            //
            // The arm routes; it does not compute. `markup::text::spec` is a
            // pure, unit-tested function of the kind, the quads and the pen, and
            // the quads themselves were derived once, by `textsel::resolve`,
            // from the same pass that painted the wash the operator was looking
            // at when they pressed the button.
            //
            // ★ The `pen` comes off the ACTION, never off `self`. It was sampled
            // in `dispatch` at the moment the operator invoked the command; this
            // arm runs at the end of the frame, by which time the live pen may
            // have moved. Reading it here would author the mark in a colour the
            // operator had not chosen when they asked — the exact hazard
            // `Action::CommitMarkup`'s `pen` field documents, and the exact
            // reason this variant grew one.
            //
            // ★ Note the second-order consequence, which is deliberate and is
            // documented at `canvas::textsel` §7: `vector_edit` bumps
            // `edit_epoch`, so the selection that authored this annotation is
            // **stale on the next frame** and its wash disappears. Acrobat keeps
            // its selection across a markup; this does not, because the epoch is
            // the only staleness signal there is and refining it into kinds of
            // edit would be a second rule living outside the module that owns
            // the first.
            Action::CommitTextMarkup {
                page,
                kind,
                quads,
                pen,
            } => {
                let spec = crate::canvas::markup::text::spec(kind, quads, pen);
                // The same options as the drag-shaped kinds above, for the same
                // reason and off the same pen. ★ A translucent HIGHLIGHT is the
                // case that matters most here: a highlight is the one markup
                // whose whole job is to sit over text and let it show through,
                // and `/CA` is the only key that does that — a highlighter's
                // yellow is otherwise a solid band over the words.
                let options = pdfcer_core::edit::MarkupOptions {
                    opacity: pen.opacity_option(),
                    ..Default::default()
                };
                vector_edit(doc, "add-text-markup", page, 1, |session| {
                    session
                        .add_markup_with(page, &spec, &options)
                        .map(|_| Vec::new())
                });
            }
            // ★★ **The commit `DEFECTS.md` D4b is about**, and the two lines
            // that make it different from the old shell's are `plan`'s.
            //
            // The old shell wrote, at its ONLY call site:
            //
            // ```text
            // doc.session_mut().edit_text(&req, &EditOptions::default())
            // ```
            //
            // `EditOptions::default()` is `FollowerDisposition::Reflow`, for
            // every run on every page of every document — so a right-aligned
            // tail was pushed off its margin and a rotated line's tail was slid
            // sideways along an axis its baseline does not run down. Neither is
            // a missing feature; both are the wrong arithmetic, chosen by
            // default because the type was constructible at the point of use.
            //
            // The arm still routes. `canvas::textedit::plan` is where the rule
            // lives, and it is one call: it re-derives the provenance pin, reads
            // the run's matrices, asks the engine for the block's alignment, and
            // returns the request and the options together so a caller cannot
            // take one without the other.
            //
            // ★ The disclosure list is the engine's own, PLUS one this shell
            // adds. `EditReport::disclosures` already says a reflowed line may
            // overrun its margin; it says nothing at all about a pinned one,
            // because from the engine's side pinning is what was asked for. But
            // a pinned tail does not make room, so a longer replacement grows
            // into it — and rule 4 forbids letting the operator discover that
            // from a diff. `plan`'s reason is what decides whether the sentence
            // is appended, so the disclosure and the disposition cannot disagree.
            Action::CommitTextEdit {
                page,
                run,
                original,
                replacement,
            } => {
                let plan = crate::canvas::textedit::plan(doc, page, run, &original, &replacement);
                let reason = plan.reason;
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // ★ It names the DISPOSITION and the REASON, and it has to
                    // name both. `HANDOFF.md` §2's grid lesson is that a check
                    // asserting a relation is satisfied by any absurdity in the
                    // right direction — and "an edit happened" is exactly such a
                    // relation here. A build that had reverted to
                    // `EditOptions::default()` would produce an identical
                    // `edit-text` line one line below this one; only this line
                    // carries the number that build would get wrong.
                    format!(
                        "text-edit-plan page={page} run={run} disposition={:?} reason={reason:?} \
                         pinned={}",
                        plan.options.disposition,
                        plan.request.pinned_span.is_some()
                    )
                });
                // ★ Gathered BEFORE the edit, because it reads a `Ref` into the
                // decomposition cache and `vector_edit` wants `&mut OpenDoc`.
                // It decides whether the engine's SHARED CONTENT sentence may
                // be followed by a remedy; `report::PageLevelForms` carries the
                // whole argument, including the nested-form case where the
                // remedy would succeed and change nothing.
                let page_forms = crate::canvas::textedit::report::PageLevelForms::of(doc);
                // ★★★ **The one fact that turns the engine's answer into a
                // sentence he can act on** — `OPERATOR_REQUESTS.md` O140. Copied
                // out of the plan before the closure takes `plan` by reference,
                // because the classification below runs inside it.
                let one_operator = plan.one_operator;
                vector_edit(doc, "edit-text", page, 1, |session| {
                    session
                        .edit_text(&plan.request, &plan.options)
                        // ★★★ **O140 — the refusal is CLASSIFIED, and the arm
                        // routes rather than deciding.**
                        //
                        // `app::status::decline::textedit` owns *"what does the
                        // text caret decline, and who says so"*; this is its
                        // third decline, and its header carries the whole
                        // argument — the engine's coarse `RefusalKind`, the one
                        // fact the engine cannot see (whether the pinned run is
                        // a single show operator), and why the tidier-looking
                        // home in `canvas::textedit::report` was rejected.
                        //
                        // ★ Inside the closure, through `inspect_err`, on
                        // `textstyle::reflow`'s precedent: the funnel takes its
                        // decline floor *before* running this and fills the slot
                        // only `if slot.is_none()`, so the classified sentence
                        // survives and the generic one stands aside.
                        .inspect_err(|error| {
                            crate::app::status::decline::record_edit_text_refusal(
                                page,
                                run,
                                one_operator,
                                error,
                            );
                        })
                        .map(|report| {
                            // ★ The shared-content fan-out, on the trace.
                            // `canvas::textedit::trace_target` owns the whole
                            // argument for why those three numbers exist and
                            // what a wrong build gets wrong about them; this arm
                            // routes, as every other arm here does.
                            crate::canvas::textedit::report::trace_target(page, run, &report);
                            let mut notes = report.disclosures.clone();
                            if reason.pins_the_tail() {
                                notes.push(crate::text::textedit::pinned_tail_disclosure(reason));
                            }
                            // ★ The engine's SHARED CONTENT sentence says WHAT
                            // happened; it cannot say what to do, because it
                            // has never heard of this shell's commands. Nothing
                            // here re-words it — this is appended after it.
                            notes.extend(page_forms.remedy_for(&report));
                            notes
                        })
                });
            }
            // ★ New page text, through the same funnel and the same four steps.
            //
            // `AddTextRequest::new` supplies the engine's own documented default
            // — a bundled 12-pt black Helvetica run — and this arm does not
            // override it. That is a decision and not an omission: a font, size
            // and colour picker is a *surface*, and `RIBBON_IA.md` P3's rule is
            // that a capability which is absent renders nothing rather than a
            // control that explains itself badly. Picking a face here from
            // nothing would be this arm computing rather than routing, and the
            // engine's default is the one answer that is already argued
            // somewhere. What the operator gets today is legible, real page
            // content they can undo; what they do not get yet is a choice about
            // its appearance, and `Format` is where that lands.
            // Insert another document's pages — a PAGE verb, so its body
            // lives in `pages` beside rotate, delete, reorder and extract, and
            // this arm routes. See `pages::insert_from_file` for why it must
            // mutate the session rather than replace it.
            // ★★ Restyle a placed markup, through the same four-step protocol
            // every other document change uses.
            //
            // The arm routes; it does not compute. Which field is `Some` was
            // decided by the control the operator moved, in
            // `panels::properties::markup`, and assembling a `MarkupStyle` here
            // from anything would be this arm making a decision the surface
            // already made — the failure `MarkupStyle`'s own doc names.
            //
            // ★ `report.dropped` is carried into the disclosure list rather than
            // discarded, and it is the whole reason this verb returns one:
            // regenerating an appearance loses anything the original expressed
            // OUTSIDE the model pdfcer draws — a border effect it does not
            // author, a producer's own decoration — and the dictionary key
            // survives while the picture does not. Rule 4: an inference the
            // operator cannot see still owes an off-canvas report.
            Action::SetMarkupStyle { page, id, style } => {
                vector_edit(doc, "set-markup-style", page, 1, |session| {
                    session.set_markup_style(id, &style).map(|report| {
                        report
                            .dropped
                            .iter()
                            .map(|d| crate::text::panels::properties::markup_dropped(*d).to_owned())
                            .collect::<Vec<String>>()
                    })
                });
            }
            // ★★★ Moved to [`super::addtext`] on 2026-09-04 under **R2**, and
            // the seam is [`super::funnel`]'s: this file ROUTES, and placing
            // text now DECIDES — a clicked caret that holds a line break has no
            // width, and something has to say where the second line ends. That
            // module's header carries the whole argument, including why the
            // width is read off the operator's own sheet rather than invented,
            // and why a one-line click still takes the point path untouched.
            Action::CommitAddText {
                page,
                origin,
                text,
                pen,
                wrap,
            } => super::addtext::commit(
                doc,
                super::addtext::Placed {
                    page,
                    origin,
                    text,
                    pen,
                    wrap,
                },
            ),
            // ★ The three things a mode change does. See the variant's docs
            // for why each one is here and not somewhere more convenient.
            //
            // The no-op guard is not an optimisation: the ribbon raises this
            // on every click, including a click on the position that is
            // already active, and without the guard each of those would drop
            // every strip raster and re-render the visible pages.
            Action::SetPageDisplay(display) => {
                if doc.view.display != display {
                    doc.view.display = display;
                    doc.strip_rasters.clear();
                    // `tracked_page` follows, so the new arrangement does not
                    // read the current page as "navigated to" and scroll to it
                    // on its first frame.
                    doc.tracked_page = doc.view.page_index;
                }
                // Recorded unconditionally, because the operator has stated a
                // choice and a document that was showing the mode by *default*
                // has nothing on disk saying so. `remember` is itself a no-op
                // when the file already says this, so a repeated click still
                // costs no write.
                //
                // ★ …and only for a document that HAS a file. `stored_under`
                // is the one predicate that says so. A created document's path
                // is a name, so writing here would store an arrangement
                // against `Untitled 1.pdf` — which the next session's
                // `Untitled 1.pdf` would then inherit, having chosen nothing.
                if let Some(path) = doc.stored_under() {
                    crate::viewer::remembered::remember(path, display);
                }
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!(
                        "page-display-set mode={} page={}",
                        display.id(),
                        doc.view.page_index
                    )
                });
            }
            Action::SetLayerVisible { group, visible } => doc.set_layer_visible(group, visible),
            Action::ResetLayers => doc.reset_layers(),
            Action::ToggleAnnotations => {
                let showing = doc.annotations_visible();
                doc.set_annotations_visible(!showing);
            }
            Action::ToggleViewChrome(chrome) => {
                let on = !chrome.read(&doc.view);
                chrome.write(&mut doc.view, on);
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("view-chrome {chrome:?} on={on}")
                });
            }
            // The guides are stored and written to disk in one step, because
            // the file IS the store's authority: `remember` is a
            // read-modify-write of the whole line, so there is no half-applied
            // state to guard against and nothing to reconcile on the next
            // open. Unconditional, exactly as `SetPageDisplay`'s `remember` is
            // — a `Guides` that equals what is already there is a gesture that
            // raised no action at all (`canvas::guides::release` compares
            // before it pushes), so a redundant write is unreachable rather
            // than merely cheap.
            // ★ …with the same `stored_under` guard `SetPageDisplay` carries,
            // for the same reason and against the same failure: a guide
            // position stored under a name rather than a location is a guide
            // the next `Untitled 1.pdf` inherits. The guides still work in the
            // session — they live on `doc` — they simply are not persisted for
            // a document that has nowhere to persist them to.
            Action::SetGuides(guides) => {
                doc.guides = guides;
                if let Some(path) = doc.stored_under() {
                    crate::canvas::guides::remember(path, &doc.guides);
                }
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!(
                        "guides-set n={} page={}",
                        doc.guides.len(),
                        doc.view.page_index
                    )
                });
            }
            // ★ The three REDACTION arms live in `super::redact`.
            //
            // Moved there on 2026-08-18 under rule R2, and the seam is a real
            // one rather than a line count: they are the only arms whose
            // subject is *marking content for removal*, they share a vocabulary
            // (`RedactAppearance`, the census, the mark ids) that nothing else
            // in this file uses, and their comments carry the argument for the
            // one operation pdfcer cannot undo. Moving the arms without their
            // reasoning would have been the split this project warns about.
            Action::MarkRedactionsBySearch { .. }
            | Action::MarkPageForRedaction { .. }
            | Action::RemoveRedactionMark { .. }
            // ★★★ …and, since 2026-09-04, the one that ARMS a removal (and,
            // since 2026-09-05, disarms it). It is routed here beside the three
            // marking arms rather than into a module of its own because it is
            // the other half of their subject — the module's header used to say
            // "nothing in this file removes anything", and the correction is in
            // that file rather than in a fourth location a reader would have to
            // find.
            | Action::PendingRedaction(_) => {
                super::redact::apply(doc, action);
            }
            // ★ Its own module, not a fourth arm in `redact`: it is the only
            // marking route whose geometry comes from the CANVAS.
            Action::MarkSelectionForRedaction { appearance } => {
                super::redactsel::mark_selection(doc, &appearance);
            }
            // ★ Every page verb, routed. The bodies have lived in
            // `super::pages` since page operations shipped; the ENUM and these
            // arms joined them on 2026-08-19 under R2, when image placement
            // pushed this file past 1,500 lines.
            //
            // The cut is along the seam that module's header already draws —
            // *"a page index is a position, not an identity"* — and it is what
            // took the panel-selection consequences with it: a delete clears
            // the picks, a reorder remaps them, a rotation leaves them alone.
            // Those three answers to one edit are the whole subject over there,
            // and they were the only part of it living here.
            // ★ The separation policy travels from the settings store, which
            // this scope can see and `pages::apply` cannot. See `pages::delete`
            // for the promise it was until 2026-08-28.
            Action::Page(action) => {
                super::pages::apply(doc, &mut self.panels, action, self.settings.separations);
            }
            // One line, like its neighbours: the whole of this verb — its two
            // correctable refusals, its three unreachable ones and the three
            // conditional clauses of its disclosure — is `super::forms`, whose
            // header carries the measurement that makes the wording necessary.
            // ★ An export changes NOTHING — no `vector_edit`, no undo entry,
            // no epoch bump, no invalidation. It reads the open page and writes
            // a different file, which is why its body is in `super::export`
            // rather than beside the mutations. See that module's header.
            // ★ One arm for the three file-picker verbs. See
            // `super::write::WriteAction` for why they are a family: they are
            // `Action`s only because a native dialog must not open inside a
            // layout pass, and none of them changes the open document.
            Action::Text(super::text::TextAction::Reflow { page, block }) => {
                super::textstyle::reflow(doc, page, block);
            }
            // ★ The one arm here that changes no document. It carries a
            // sentence from a keystroke handler across the `crate::app`
            // boundary to the bar — see the variant's own docs for why a
            // keypress needs an `Action` to speak at all.
            Action::Text(super::text::TextAction::EnterCannotSplit) => {
                crate::app::status::decline::record_enter_cannot_split();
            }
            Action::Write(write) => match write {
                // (the enum is `super::write::WriteAction`)
                super::write::WriteAction::Dxf { page, options } => {
                    super::export::dxf(doc, page, &options)
                }
                // O120. The refusal of an impossible combination, the picker
                // and the whole disclosure live in `super::export::image`.
                super::write::WriteAction::Image { plan } => super::export::image(doc, &plan),
                // ★ The extraction, the scan refusal, the picker and the whole
                // disclosure live in `super::export::text`. The refusal is the
                // interesting half: a scanned page extracts successfully and
                // returns nothing, and a zero-byte `.txt` on disk is
                // indistinguishable from a successful export.
                super::write::WriteAction::Text { plan } => super::export::text(doc, &plan),
                super::write::WriteAction::FormData => super::export::form_data(doc),
                super::write::WriteAction::Compacted { bytes, before } => {
                    crate::app::save::compacted(doc, &bytes, before);
                }
            },

            // ★ Unlike its two neighbours above, this one DOES change the
            // document - it is here rather than in `super::export` for that
            // reason alone. One undo entry, one epoch bump, and every page
            // re-rasterized, because a font gaining a program changes how it
            // draws everywhere it is used.
            Action::EmbedFonts { request } => super::fonts::embed(doc, &request),
            Action::UnembedFonts { request } => super::fonts::unembed(doc, &request),
            // ★ The bookmark family — add, rename, delete-with-its-subtree.
            //
            // One line, like `Action::Dimension` and `Action::Page` above it.
            // The three arms moved into `super::bookmarks` under R2 on
            // 2026-08-28, and its header carries what a reader needs first:
            // every verb names its operand by `ObjId` because an outline is
            // renumbered by every edit to it, and `/Count` is two different
            // quantities whose SIGN carries open-or-closed (§12.3.3), which is
            // why none of them describes itself by diffing a count.
            Action::Bookmark(action) => super::bookmarks::apply(doc, action),
            // ★ The attachment family — attach, remove, save one out.
            //
            // One line, like its neighbours. What its module carries and this
            // arm must not restate: all three open a native file dialog, so the
            // picker lives in the APPLY phase rather than in the widget that
            // asked for it; two of the three mutate and go through
            // `vector_edit`, and `SaveCopy` deliberately does not, because it
            // changes nothing and bumping the epoch for it would retire a
            // disclosure that is still true.
            Action::Attachment(action) => super::attachments::apply(doc, action),
            // ★ The form-XObject family — give this page its own copy, today.
            //
            // One line, like its three neighbours. The one thing its module
            // carries that a reader of THIS file needs: every refusal is worded
            // from inside the `vector_edit` closure rather than falling to the
            // generic `Err` arm below, because a silent decline on this verb
            // reads as SUCCESS — the copy is byte-identical, so refused and
            // applied look exactly alike on the canvas.
            Action::XObject(action) => super::xobject::apply(doc, action),
            // ★ **Undo and redo**, through the same [`vector_edit`] funnel every
            // other document change goes through — which is the whole of why
            // these two arms are one line each. See [`history_step`].
            // ★★★ **Everything on the page, wherever it now sits** — and the
            // whole argument for the command, which `Action::SelectAllOnPage`
            // points here for.
            //
            // The operator, 2026-09-01: *"we should be able to select things off
            // the side of the page, especially since I sometimes drop objects
            // there, and when I do I can't get them back."*
            //
            // He is describing a ONE-WAY DOOR. `canvas::present` allocates the
            // page's own rectangle as the interaction area, and its comment
            // gives the right reason: a hit area reaching off the sheet would
            // overlap its neighbours in a continuous strip. So an object dragged
            // past the edge sits outside every rectangle that can be clicked,
            // outside any rubber band that can be drawn, and outside the crop
            // box that decides what is painted. Invisible, unreachable, and
            // still in the file.
            //
            // ★★ This is the way out of the room, and it is deliberately NOT a
            // fix for off-page input sensing — a band still cannot be started
            // off the sheet, and `OPERATOR_REQUESTS.md` O88's direction-
            // sensitive marquee is still owed. What recommends it is that it is
            // buildable and verifiable in one sitting, where the gesture work is
            // not, and it turns "I cannot get them back" into one keystroke
            // today.
            //
            // ★ CONTENT only. Annotations, form widgets and ce dimensions have
            // their own surfaces and are not in `PageObjects`; sweeping all four
            // kinds into one selection would produce a set most of this shell's
            // verbs would then refuse.
            //
            // Asked for by a rect that cannot fail to contain: `Rect::EVERYTHING`
            // through the same `hit_test_rect` a rubber band uses, rather than
            // by walking `PageObjects::objects` and minting ids here. Two
            // reasons, and the second is the load-bearing one:
            //
            // 1. The provider owns the `TargetId` encoding — page objects and
            //    form leaves are different index spaces and only it mints
            //    either. A hand-rolled `0..objects.len()` would produce
            //    `Object(i)` for everything and silently drop every leaf.
            // 2. **`MarqueeMode::Enclosed` against an infinite rect selects
            //    everything by construction**, including objects with negative
            //    or out-of-page coordinates, which is exactly the case this
            //    command exists for. Nothing has to be special-cased for
            //    off-page geometry because nothing is bounded.
            // ★★ The refusal list is DATA, not an error — the call succeeds
            // with a partial result, which is what the request asked for:
            // "nine changed", not "done".
            Action::SetObjectPaint {
                page,
                objects,
                fill,
                stroke,
            } => {
                // 8-bit sRGB from the swatch back into PDF's own 0..1 units.
                // The inverse of `panels::properties::paint::to_bytes`, and the
                // pair is what keeps the operand in the engine's units rather
                // than leaving a byte triple to be reinterpreted downstream.
                let to_rgb = |c: [u8; 3]| pdfcer_core::vector::Rgb {
                    r: f32::from(c[0]) / 255.0,
                    g: f32::from(c[1]) / 255.0,
                    b: f32::from(c[2]) / 255.0,
                };
                super::apply::vector_edit(doc, "set-object-paint", page, objects.len(), |s| {
                    s.set_object_paint(page, &objects, fill.map(to_rgb), stroke.map(to_rgb))
                        .map(|outcome| {
                            let changed = outcome.changed.len();
                            let refused = outcome.refused.len();
                            vec![if refused == 0 {
                                crate::text::paint::recoloured(changed)
                            } else {
                                crate::text::paint::recoloured_partly(changed, refused)
                            }]
                        })
                });
            }
            Action::SelectAllOnPage => {
                let page = doc.view.page_index;
                // ★★★ A LARGE FINITE RECT, not `Rect::EVERYTHING`.
                //
                // The first version used `EVERYTHING` and selected **nothing**,
                // measured on the operator's own drawing: `select-all page=0
                // n=0`. The provider maps the query rectfrom canvas space into
                // PDF space before asking the engine, and an infinite rect put
                // through an affine transform yields NaN — after which every
                // containment test is false and the answer is silently empty.
                //
                // ⇒ Infinity is not a safe "everything" when a coordinate
                // system change stands between the caller and the comparison.
                // A million points is about 350 metres of paper; no page
                // approaches it, and every arithmetic step stays finite.
                const EVERYWHERE: f32 = 1.0e6;
                let all = egui::Rect::from_min_max(
                    egui::pos2(-EVERYWHERE, -EVERYWHERE),
                    egui::pos2(EVERYWHERE, EVERYWHERE),
                );
                let hits = doc
                    .page_objects()
                    // ★ `Enclosed` stated rather than implicit, as of 2026-09-02
                    // when the mode became a parameter (O88): under
                    // `Rect::EVERYTHING` the two modes agree, and a reader must
                    // not have to work that out before believing Select All is
                    // unaffected by a change to what a rubber band means.
                    .map(|p| p.hit_test_rect(page, all, MarqueeMode::Enclosed))
                    .unwrap_or_default();
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("select-all page={page} n={}", hits.len())
                });
                // `false` — a Select All REPLACES. Extending would make a
                // second press a no-op and a first press after a click keep the
                // click, neither of which is what the command says.
                doc.selection.marquee(page, &hits, false);
            }
            Action::Undo => super::history::history_step(doc, super::history::Direction::Undo),
            Action::Redo => super::history::history_step(doc, super::history::Direction::Redo),
            // ★ Reaching here means the frame's drain was removed or moved.
            // Traced rather than ignored: the symptom otherwise is a control
            // that does nothing on every press, with no evidence why.
            Action::Command(id) => crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "command-action-undrained id={id}"
                )
            }),
        }
    }
}

/// Apply **one** vector-geometry edit to `doc`, as one undoable command.
///
/// The shared body of every arm above that changes the document — Delete, the
/// three move verbs, and [`Action::CommitMarkup`].
///
/// **Markup joins it rather than getting its own copy**, and the name is now a
/// little narrow for what it does. That is the better trade: `add_markup` needs
/// the identical four steps in the identical order — the worker cancelled
/// first, the mutation through `Arc::get_mut`, the epoch bumped so the canvas
/// re-resolves and the raster is rebuilt, the texture dropped — and the only
/// thing it does differently is return an `ObjId` where the vector verbs return
/// a disclosure list. That is one `.map` at the call site, against a whole
/// second hand-written copy of a protocol whose entire reason for existing is
/// that four hand-written copies would be four chances to omit a step.
///
/// It exists as one function rather than five copies for
/// the reason the ordering below is load-bearing: each of the four steps is a
/// separate way to end up with an edit that is silently declined or a page that
/// silently keeps drawing what was just changed, and four hand-written copies
/// of a four-step protocol is four chances to omit a step. The `label` and the
/// operand count are carried only so the trace can say which verb ran.
///
/// # The four things that have to happen in this order
///
/// 1. **Stop the render worker.** `OpenDoc::session` is an `Arc` precisely so
///    a worker can hold a clone while it rasterizes, and
///    `RenderWorker::cancel_and_wait`'s own docs call itself *"the choke
///    point that makes `Arc<EditSession>` sound"*: `Arc::get_mut` fails while
///    any other strong reference exists, so a mutation attempted mid-render
///    would simply be refused. Cancelling first is what turns "sometimes
///    refused, depending on how fast the page rasterized" into "always
///    applied".
/// 2. **Mutate through `Arc::get_mut`.** A `None` here is not a panic: it
///    means something else still holds the session, which is a bug in the
///    caller's ordering rather than in the operator's document. It is traced
///    and the edit is declined, because declining an edit is recoverable and
///    corrupting one is not.
/// 3. **Bump `edit_epoch`.** `OpenDoc::edit_epoch`'s own doc comment names
///    this exact seam: *"the first mutating arm added to
///    `PdfcerApp::apply` must bump it"*, so the object-count trace re-reads
///    and — the part that matters here — the canvas's selection re-resolves
///    against the new decomposition rather than keeping an entry that now
///    names a hole.
///
///    **A move needs the epoch bump for the geometry, not for the identity.**
///    `move_*` rewrites operator operands in place and adds or removes no
///    operator, so paint-order indices are stable across it — measured, by
///    `crates/pdfcer-core/tests/object_identity_across_edits.rs`. The selection's
///    *entries* therefore survive untouched and nothing has to be remapped;
///    what has changed is where each entry's outline goes, and the epoch is
///    what makes `SelectionState::resolve` recompute it. `delete_*` excises
///    byte spans and does renumber, which is the case
///    `pdfcer_core::vector::remap_index_after_delete` exists for.
/// 4. **Drop the cached texture** — see below.
///
/// # The disclosures, and the one thing this function cannot yet discharge
///
/// Every vector verb returns `Result<Vec<String>, EditError>`, and the
/// `Vec<String>` is the **disclosure list**: operator-facing strings the
/// surgery owes under rule 4, non-empty when the edit had to change an
/// operator's *form* to express the request — an `re` rectangle expanded into
/// explicit segments, an implicitly-started subpath's `m` materialised. The
/// drawing is unchanged but the bytes are no longer recoverable by reversing
/// the gesture, and rule 4 forbids letting the operator find that out from a
/// diff.
///
/// They are traced here, in full, so nothing is lost. **Tracing is not
/// surfacing**: a disclosure belongs on an operator-visible surface, and the
/// status line is `app::status`'s to own, not this module's to invent.
///
/// ## ★ The outstanding half, discharged 2026-08-14
///
/// The paragraph above used to end *"That is the outstanding half"*, and it is
/// no longer outstanding. The list is now **recorded as well as traced** — as
/// an [`EditDisclosure`] stamped with the epoch this edit produced — and
/// [`crate::app::status`] draws it in the bar beside the fill disclosure it
/// copies, on the row that may not grow (R128).
///
/// Three things about that, each a decision:
///
/// - **The trace is unchanged, not replaced.** It is the record of what
///   happened; the bar is what an operator reads *now* and loses at the next
///   edit. A disclosure that survives only on screen is one the next reader of
///   `PDFCER_DIAG` cannot audit, and one that survives only in the trace is the
///   defect this section used to name.
/// - **This module still does not draw anything**, which is why the split is
///   at a recorded value rather than at a formatted line. Everything an
///   operator sees — the framing, the mark, the eliding, the hover — is
///   decided in `app::status` and `text::status`, exactly as the original
///   sentence said it must be.
/// - **The stamp is the epoch bumped one line above, not the one the edit ran
///   against.** The revision on screen from now until the next edit is the new
///   one, so an undo silences the sentence by moving the epoch past it, with
///   nothing anywhere remembering to clear it. Stamping the old epoch would
///   produce a disclosure that was invisible from the moment it was written —
///   which is the failure mode `crate::panels::forms::edit::apply`'s ★ comment
///   records for the fill precedent.
///
/// # Why the cached texture is dropped
///
/// Nothing else notices an edit. `settle_and_rasterize` compares the cached
/// texture against the page index and the raster scale, and an edit changes
/// neither — so without this the page would keep showing the object where it
/// used to be until the operator zoomed or paged away. Dropping it forces a
/// re-raster on the same frame (step 4 runs after step 3), and
/// `RenderWorker::spawn` waits a bounded number of milliseconds inline, so a
/// page that rasterizes quickly never shows a gap at all.
///
/// The *right* fix is for the texture's key to carry a content generation, so
/// staleness is a property of the key rather than something each mutating arm
/// has to remember. That key lives in `render/`, which is not this module's to
/// extend; this is the honest interim, and it is one line in one shared
/// function rather than a convention spread across four verbs and counting.
///
/// # ★ Why the error type is generic, and it is one word of generality
///
/// It took `pdfcer_core::edit::EditError` for the whole of its life, because
/// every verb that came through it — delete, the three moves, `add_markup`,
/// `add_dimension` — reports that type. The two text verbs do not:
/// `EditSession::edit_text` reports `text_edit::EditError` and
/// `EditSession::add_text` reports `text_edit::AddTextError`, and neither
/// converts into the first.
///
/// The three ways out were: a second copy of the four-step protocol for each new
/// error type, which is the exact thing this function exists to prevent; a
/// `map_err` to a string at every call site, which would put the *formatting* of
/// a refusal in five places; or one bound. The bound is `Display`, which is the
/// only capability the error branch below actually uses — it puts the message on
/// the trace and declines. Nothing here inspects a variant, so nothing here
/// needed to know the type.
/// ★ The edit funnel moved to [`super::funnel`] on 2026-08-30 under R2, and is
/// re-exported here so that every call site written as `apply::vector_edit`
/// still resolves.
///
/// Kept as a re-export rather than updating forty call sites: the move is about
/// where the code *lives*, and rewriting every caller would have made a
/// mechanical relocation look like a change to what they do.
pub(super) use super::funnel::{vector_edit, vector_edit_on_page};

#[cfg(test)]
mod tests;
