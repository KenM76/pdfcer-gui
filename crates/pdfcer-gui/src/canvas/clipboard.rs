//! # `canvas::clipboard` — **cut, copy and paste on the canvas**
//!
//! ## What this closes
//!
//! The operator, 2026-08-19: *"also the standard copy/paste and I didn't try cut
//! so possibly that one too aren't implemented."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/clipboard.md`.

use pdfcer_core::object::ObjId;

/// **Whether a text gesture owns `Ctrl+C` / `Ctrl+X` this frame**, so the object
/// clipboard must stand aside.
pub fn text_owns_the_chord(ctx: &egui::Context, doc: &crate::app::state::OpenDoc) -> bool {
    if crate::canvas::textedit::composing(ctx) {
        return true;
    }
    doc.text_selection
        .as_ref()
        .is_some_and(|selection| selection.live(doc.edit_epoch))
}

use crate::app::actions::Action;
use crate::app::state::OpenDoc;

/// The `egui::Memory` key the clipboard is parked under.
const KEY: &str = "pdfcer.canvas.clipboard"; // ui-text-exempt: memory key, never displayed

/// How far a same-page paste is displaced, in PDF points.
pub const PASTE_OFFSET_PT: f64 = 10.0;

/// What the canvas clipboard is holding.
#[derive(Debug, Clone, PartialEq)]
pub enum Clipped {
    /// **A copied selection** — page content, annotations, or both, as one
    /// `ObjectClip`.
    ///
    /// This variant is what the type's own docs predicted: *"the day page
    /// content becomes pasteable this type is where that arrives."* `Pass 120.0`
    /// shipped `ObjectClip` on 2026-08-20 and this is that day.
    ///
    ///
    /// The name was accurate while the only thing a clip could hold was a page
    /// object. It stopped being accurate the moment this shell started routing
    /// annotations through `copy_selection`, and a variant called `Content`
    /// holding nothing but a sticky note is the sentence-describing-a-different-
    /// world shape `DEFECTS.md` D4a is about, one layer down in the source.
    /// `Selection` is what the engine calls it too, which removes one
    /// translation between the two vocabularies.
    ///
    /// # Why the BYTES and not the `ObjectClip`
    ///
    /// Three reasons, and the third is the one that decides it:
    ///
    /// 1. `egui::Memory` wants `Clone + Send + Sync + 'static`, and bytes are
    ///    all four without asking anything of the engine's type.
    /// 2. `Clipped` derives `PartialEq`, which bytes give for free.
    /// 3. **It is the same representation the OS clipboard will take.** The
    ///    engine's `to_bytes` is magic-prefixed, versioned and bit-exact
    ///    precisely so a pdfcer→pdfcer paste is lossless, and registering it as a
    ///    private format is the remaining half of the operator's item 3. Holding
    ///    the live struct here would mean serialising at the moment of
    ///    registration instead — a second code path, for the same bytes.
    ///
    /// The clip **owns its resources**, transitively, by value. So copying
    /// from one document, closing it, and pasting into another works — and
    /// cross-document paste is not a special case but the same call.
    Selection {
        /// `ObjectClip::to_bytes` — magic-prefixed, versioned, bit-exact.
        bytes: Vec<u8>,
        /// The 0-based page it was copied from, for the same reason
        /// [`Self::Markup`] carries one: a paste onto the *same* page offsets
        /// so the copy is visible, a paste elsewhere lands in place.
        page: usize,
        /// How many **content objects** are in it, for the trace and for a
        /// sentence.
        ///
        /// Carried rather than re-derived because reading it back means
        /// deserialising, and the count is wanted in places that have no reason
        /// to.
        count: usize,
        /// How many **annotations** are in it — separately, because they are a
        /// separate payload on the clip and a separate address space on the
        /// way in.
        ///
        /// Kept apart from `count` rather than summed, and the reason is
        /// the same one the engine gives for taking two index lists: a total
        /// makes *"three objects"* and *"two objects and a comment"*
        /// indistinguishable on the trace, and those are the two builds a wrong
        /// paste has to be told apart. It is also what the paste's mode gate
        /// reads — page content needs `edit_content`, a comment needs only
        /// `author_markup`, and a clip that could not say which it held would
        /// have to demand the stricter of the two.
        annotations: usize,
        /// The annotations' object ids, in the order they were copied — for
        /// the **cut**'s delete half only.
        ///
        /// A delete is raised by `ObjId` through the funnel, and an
        /// `/Annots` position is not one. Re-deriving the ids from the indices
        /// after the clip was taken would be a second walk of the page that
        /// could disagree with the first — and the window between them is
        /// exactly where a cut deletes the wrong annotation.
        ///
        /// Empty for a pure-content clip and for a paste from another
        /// document, both of which are the same fact: nothing here is to be
        /// removed.
        annot_ids: Vec<ObjId>,
        /// **What the copy could not carry**, as the `/Subtype`s the
        /// engine refused, verbatim.
        ///
        /// Carried on the clip rather than said at copy time and forgotten,
        /// because the caller that words it is `app::dispatch::clipboard` and
        /// this module words no decline — its own standing contract. A
        /// non-empty list means the operator selected something whose copy is
        /// *partial*, and rule 4's "fuzzy never sneaky" makes disclosing that
        /// mandatory: the paste will look complete.
        ///
        /// Almost always empty today, because the only annotations the
        /// engine refuses — `/Widget`, `/Popup`, `/Redact` — are either routed
        /// elsewhere or refuse the whole copy. It becomes reachable the day a
        /// selection can hold more than one annotation, which is the same day
        /// the mixed marquee arrives.
        left_behind: Vec<String>,
        /// How many annotations on the clip travel as a `MarkupSpec` and will
        /// therefore arrive **without** `/CA`, `/T`, `/M` or `/Contents`.
        ///
        thin: usize,
        /// How many selected targets painted inside a form XObject the copy
        /// left out: the engine copies page objects only (G145).
        in_form_left: usize,
        /// **The point that is placed under the cursor on a paste** —
        /// the clip's centre, in **PDF user space**.
        ///
        /// `OPERATOR_REQUESTS.md` O73: *"when I paste it should paste where
        /// the mouse cursor is sitting."*
        ///
        /// # Why it is captured at COPY time and not derived at paste time
        ///
        /// Because at paste time the source may be gone. The clip outlives the
        /// selection it came from, outlives an undo of the cut that produced
        /// it, and — for a cut — outlives the objects themselves. Deriving the
        /// centre from the document on paste would work in the common case and
        /// fail in exactly the case `Ctrl+X` `Ctrl+V` exists for.
        ///
        /// # Why a CENTRE
        ///
        /// The operator pointing at a spot means *"put it here"*, not *"begin
        /// its bounding box here"*. That is Inkscape's rule and Illustrator's;
        /// top-left is the Word/Explorer convention and belongs to a text
        /// caret rather than to a drawing canvas. Acrobat drops a pasted
        /// comment centred on the click too.
        ///
        /// It is also what preserves relative geometry inside a
        /// multi-object paste **by construction**: one anchor for the whole
        /// clip means one delta, applied to everything, so the arrangement
        /// cannot drift no matter how many items are in it.
        ///
        /// `None` when the geometry could not be read — the paste then falls
        /// back to the offset rule rather than guessing, because a clip that
        /// pasted at `(0, 0)` would land in the bottom-left corner of the
        /// sheet and read as data loss.
        anchor: Option<(f64, f64)>,
    },
    /// **A form field**, as of 2026-08-29 — `OPERATOR_REQUESTS.md` O58.
    ///
    ///
    /// # Why it is a variant here and not a second clipboard
    ///
    ///
    /// One entry per copied field, never empty, the primary last. A `Vec`
    /// keeps the variant the size of its neighbours' boxes.
    FormField(Vec<crate::canvas::fieldclip::ClippedField>),
    /// **An embedded file** — its name, its decoded bytes and its description.
    ///
    ///
    /// **Carries the DECODED bytes**, which is the engine's choice and worth
    /// restating: `AttachmentClip` holds what `extract-attachment` would give
    /// you and what `attach_file` expects on the way back in. Carrying the raw
    /// stream instead would carry its filter chain with it, and a paste would
    /// have to re-derive whether that chain still applied in the destination.
    ///
    /// It carries **no page**, unlike every other variant here. A
    /// document-level embedded file does not live on a sheet, so there is no
    /// same-page/different-page question and no paste offset — which is why the
    /// paste is in the Attachments panel rather than on the canvas.
    Attachment(Box<pdfcer_core::attachments::AttachmentClip>),
    /// **Whole pages** — `OPERATOR_REQUESTS.md` O59, 2026-08-29.
    ///
    /// # The bytes ARE a PDF, and that is not an implementation detail
    ///
    /// `PageClip::bytes` is a complete document, openable by anything — the
    /// engine's own choice, because `pageops::assemble` already does object
    /// copying, reference remapping and page-tree construction on every split
    /// and merge, and a private page format would have been a second
    /// implementation of the most-exercised code in that crate.
    ///
    /// The consequence for this shell is that the day a private OS clipboard
    /// format is registered, **this variant needs no new serialisation at
    /// all** — and `application/pdf` is a flavour other programs already read.
    ///
    /// # Why it is in this enum rather than a key of its own
    ///
    /// One clipboard holds one thing, which is [`Self::FormField`]'s reason
    /// verbatim. But the collision it prevents is different here and worth
    /// naming: page copy and object copy are reached by **different controls**
    /// — `pages.copy` on the Pages tab, `Ctrl+C` on the canvas — so an
    /// operator can plausibly believe both are live at once. Sharing one slot
    /// makes the last one they pressed the one that pastes, which is the only
    /// rule they can hold in their head.
    /// **A bookmark and everything filed under it** — O59 item 3.
    ///
    /// # Why this one is NOT bytes, unlike its two neighbours
    ///
    /// `Clipped::Content` and `Clipped::Pages` carry serialised clips because
    /// the engine gives them one — `ObjectClip::to_bytes` and a `PageClip` that
    /// **is** a PDF. `OutlineClip` has no serialisation at all, so there are no
    /// bytes to carry and the live structure is the only representation there
    /// is.
    ///
    /// That is a real difference in what the two can do, and it is worth
    /// knowing rather than discovering: the day this shell registers a private
    /// OS clipboard format, pages and page content will cross a process
    /// boundary and **bookmarks will not**, until the engine gives this type a
    /// codec. Not filed as a request, because nothing has asked for it — a
    /// bookmark subtree pasted into another program has no meaning, and the
    /// pdfcer-to-pdfcer case this variant serves works entirely in memory.
    ///
    /// `Box`ed for `FormField`'s reason: this enum is cloned on every read
    /// and an `OutlineClip` carries a whole subtree of titles, destinations and
    /// colours.
    Outline {
        /// The copied roots and their children, in document order.
        clip: Box<pdfcer_core::outline::OutlineClip>,
        /// The deepest 0-based page any destination in the clip names.
        ///
        /// Carried rather than re-walked because it answers the one question
        /// that must be asked **before** the paste: a destination naming a page
        /// the destination document does not have is **dropped, not clamped**,
        /// and a dropped-destination bookmark still shows, still has its title,
        /// and does nothing when clicked. Nothing on screen distinguishes it.
        ///
        /// `None` when no bookmark in the clip navigates anywhere — which
        /// §12.3.3 permits, and which is a legal, honest shape rather than a
        /// broken one.
        deepest_page: Option<usize>,
    },
    Pages {
        /// The clip, as a complete PDF document.
        bytes: Vec<u8>,
        /// How many pages it holds, for the sentence and the trace.
        ///
        /// Carried rather than re-derived because reading it back means
        /// parsing a document, and the count is wanted in places that have no
        /// reason to.
        count: usize,
    },
}

pub use pdfcer_gui_base::refusals::clipboard::Refusal;

/// The clip's centre in page space, from the engine's own bbox over content
/// items and annotation `/Rect`s; `None` for an empty box.
pub(crate) fn anchor_of(clip: &pdfcer_core::vector::ObjectClip) -> Option<(f64, f64)> {
    (!clip.bbox().is_empty()).then(|| {
        let b = clip.bbox();
        ((b.min.x + b.max.x) / 2.0, (b.min.y + b.max.y) / 2.0)
    })
}

/// Put the copy on the OS clipboard: a sentence, the clip's bytes and a
/// picture in one transaction, else the sentence alone.
pub(crate) fn publish(
    ctx: &egui::Context,
    clip: &pdfcer_core::vector::ObjectClip,
    objects: usize,
    annots: usize,
) {
    // AND A MARKER ON THE OS CLIPBOARD, WITHOUT WHICH CTRL+V DOES NOT
    // ARRIVE AT ALL.
    //
    // Not a nicety and not a placeholder. `egui-winit` turns `Ctrl+V` into
    // `Event::Paste(contents)` **only if the OS clipboard has non-empty text**,
    // and returns before pushing a key event either way — so with an empty
    // clipboard the keystroke vanishes completely, no event of any kind.
    // `app::keyboard::clipboard_chord` carries the whole account.
    //
    // So a copy that put nothing on the OS clipboard would leave `Ctrl+V`
    // working or not depending on **whether the operator had recently copied
    // text in another application**, which is the worst kind of intermittent:
    // it is not random, it is not reproducible, and the thing that fixes it has
    // nothing to do with pdfcer.
    //
    // The text is a SENTENCE, so a human who pastes into a text editor gets
    // something that says what happened. The clip's own bytes go beside it
    // under `clipimage::OBJECT_CLIP_FORMAT`, which is what another pdfcer-gui
    // window pastes (`canvas::clipshared`); this window pastes its in-memory
    // copy.
    // **AND A PICTURE BESIDE IT, as of 2026-08-31** —
    // `OPERATOR_REQUESTS.md` O71: *"so we can copy and paste them … outside of
    // the pdfcergui."*
    //
    // The sentence, the clip and the bitmap go on in ONE clipboard
    // transaction, and that is not an optimisation. `EmptyClipboard` is
    // per-open, so two calls would mean the second erased the first — and if
    // the picture went on second, this application's own `Ctrl+V` would stop
    // arriving for the reason the paragraphs above set out.
    // `native_clipboard::place` writes all or none.
    //
    // It FALLS BACK rather than failing. A clipboard another process is
    // holding, a render that declines, a degenerate clip — each of those loses
    // the picture and none of them loses the copy, so the marker still goes on
    // by the route it always did and the operator's `Ctrl+V` still works. What
    // they lose is the paste into Word, and the trace says which.
    //
    // An ANNOTATION-only clip reaches `publish` with `clip.items` empty, so
    // the raster is degenerate and the picture declines — by the same path a
    // zero-area content clip already took, with no new branch. That is the
    // right outcome rather than a gap: `clipimage` renders the page content a
    // clip carries, and a comment's appearance is not page content. The
    // operator still gets the marker, so `Ctrl+V` still arrives.
    let marker = crate::text::clipboard::os_marker(objects, annots);
    let published = crate::canvas::clipimage::publish(clip, &marker);
    if published.is_none_or(|p| p.picture.is_none()) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "clipboard-image-declined".to_owned()
        });
    }
    if published.is_none() {
        ctx.copy_text(marker);
    }
}

/// Read the clipboard.
#[must_use]
pub fn read(ctx: &egui::Context) -> Option<Clipped> {
    ctx.data(|d| d.get_temp::<Clipped>(egui::Id::new(KEY)))
}

/// Write it.
pub fn store(ctx: &egui::Context, clipped: Clipped) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(KEY), clipped));
    crate::canvas::clipseq::mark(ctx);
}

/// **Copy what is selected** — page content, annotations, or both — and say
/// what reached the clipboard.
pub fn copy(ctx: &egui::Context, doc: &OpenDoc) -> Result<Clipped, Refusal> {
    let taken = super::take::selection(doc)?;
    let clipped = taken.clipped();
    store(ctx, clipped.clone());
    publish(ctx, &taken.clip, taken.objects, taken.plan.carried());
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // The count and the byte length are what a wrong build gets wrong: a
        // clip that dropped the resources is several hundred bytes shorter.
        // `annots=` and `thin=` beside them, because `annots=0` on a copy of a
        // sticky note is the whole failure and neither shows in `bytes=`.
        format!(
            "clipboard-copy kind=selection page={} objects={} annots={} thin={} \
             left_behind={} in_form_left={} bytes={}",
            taken.page,
            taken.objects,
            taken.plan.carried(),
            taken.plan.thin,
            taken.plan.refused.len(),
            taken.in_form_left,
            taken.clip.to_bytes().len(),
        )
    });
    Ok(clipped)
}

/// Copy, then delete — cut. One undo entry: the copy is not an edit.
///
/// Every question that could stop the delete is asked before the copy, so a
/// cut is never left half-done with a copy on the clipboard and the original
/// still on the page — the next paste would double it. A selection one of
/// whose annotations cannot be deleted refuses the whole cut, rather than
/// cutting the rest and leaving that one copied as well as kept.
pub fn cut(
    ctx: &egui::Context,
    doc: &OpenDoc,
    actions: &mut Vec<Action>,
) -> Result<Clipped, Refusal> {
    // `edit.cut` is greyed on these predicates, but `Ctrl+X` is dispatched
    // without consulting enablement; this is what the chord meets.
    if let Some(blocker) = crate::canvas::cutgate::blocker(doc) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "clipboard-cut-refused reason=would-not-survive subtype={}",
                blocker.subtype
            )
        });
        return Err(Refusal::CutWouldNotSurvive(blocker.subtype));
    }
    for selected in doc.selection.annots() {
        if let Some(why) = crate::panels::properties::annotdelete::gate(doc, &selected.target) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("clipboard-cut-refused reason=delete-gated why={why:?}")
            });
            return Err(Refusal::DeleteRefused(why));
        }
    }
    let clipped = copy(ctx, doc)?;
    // Copy then `DeleteSelection`, not the engine's `cut_objects`: the delete
    // goes through the edit funnel like every other edit, and this module
    // changes no document.
    match &clipped {
        Clipped::Selection {
            page, annot_ids, ..
        } => {
            let objects = doc.selection.object_indices_on(*page);
            if !objects.is_empty() {
                actions.push(
                    crate::app::actions::VectorAction::DeleteSelection {
                        page: *page,
                        objects,
                    }
                    .into(),
                );
            }
            // One `Delete`, or one `DeleteMany` for several: one undo entry.
            // Every one passed the gate above, so none is kept as locked.
            let annots = doc.selection.annots();
            debug_assert_eq!(annots.len(), annot_ids.len());
            if let Some(delete) = pdfcer_gui_base::annotselection::delete_selected(&annots) {
                actions.push(Action::Annot(delete));
            }
        }
        // A form field's cut is `canvas::fieldclip::cut`, which raises
        // `FieldAction::DeleteWidget` -- a widget is addressed by its FIELD's
        // name and an index within it, not by the `ObjId` this arm's siblings
        // use, because one field can draw boxes on three pages. Reaching this
        // arm means `app::dispatch::clipboard` routed a field copy into the
        // markup path.
        // A page cut is `dispatch::pageclip`, which raises
        // `PageAction::DeletePages` -- pages are addressed by index in the
        // document, not by an `ObjId` on a page, so nothing in this arm's
        // vocabulary can express one. Same tripwire as the field arm below.
        // A bookmark cut is `panels::bookmarks::clip`, which raises
        // `BookmarkAction::Delete` -- an outline item is addressed by `ObjId`
        // in a document-level tree, not by a page and an index, so this arm's
        // vocabulary cannot express one either. Third tripwire, same shape.
        Clipped::Outline { .. } => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "a bookmark cut must route to panels::bookmarks::clip; the canvas clipboard has no outline vocabulary"
            );
        }
        Clipped::Pages { .. } => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "a page cut must route to app::dispatch::pageclip; the canvas clipboard has no page vocabulary"
            );
        }
        Clipped::FormField(_) => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "a form field cut must route to canvas::fieldclip::cut; app::dispatch::clipboard owns that fork"
            );
        }
        // An attachment cut is `panels::attachments::clip`, which raises
        // `AttachmentAction::Detach` -- an embedded file is addressed by its
        // `/EmbeddedFiles` name-tree KEY, which is neither a page nor an
        // `ObjId` on one, so this arm's vocabulary cannot express it. Fourth
        // tripwire, same shape as the three above, and they have collectively
        // fired twice during development.
        Clipped::Attachment(_) => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "an attachment cut must route to panels::attachments::clip; the canvas clipboard has no name-tree vocabulary"
            );
        }
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "clipboard-cut kind={}",
            match &clipped {
                Clipped::Selection { .. } => "selection",
                Clipped::FormField(_) => "form-field",
                Clipped::Pages { .. } => "pages",
                Clipped::Outline { .. } => "outline",
                Clipped::Attachment(_) => "attachment",
            }
        )
    });
    Ok(clipped)
}

/// Paste onto `page`, raising the action that authors it.
pub fn paste(
    ctx: &egui::Context,
    page: usize,
    target: Option<egui::Pos2>,
    actions: &mut Vec<Action>,
) -> Result<(), Refusal> {
    match read(ctx) {
        // A clip takes its own path: it is bytes and the verb is
        // `paste_objects`, which takes a page-space MATRIX rather than a
        // displacement — so the offset below cannot be shared even though the
        // rule that decides it is.
        //
        // ONE verb plants BOTH halves. `paste_objects` commits the content
        // command and then calls the private `paste_clip_annotations`, so this
        // shell raises **one** action for a mixed clip and does not have to
        // sequence two. The engine's own note is that the annotation half
        // lands as **its own undo entry**, which is disclosed here rather than
        // discovered: `Ctrl+Z` after a mixed paste takes back the comments
        // first and the geometry second. Not reachable today; filed on the
        // clipboard row of `ENGINE_BACKLOG.md`.
        Some(Clipped::Selection {
            bytes,
            page: from,
            count,
            annotations,
            anchor,
            ..
        }) => paste_clip(
            page,
            &bytes,
            from,
            count,
            annotations,
            anchor,
            target,
            actions,
        ),
        // Same fork as the cut above, and the same tripwire. A field paste
        // needs `&OpenDoc` -- to find a free name, and to know what the source
        // field could not carry -- which this function does not take and must
        // not grow, because every other paste here is a pure function of the
        // clip. `app::dispatch::clipboard` branches before calling either.
        // Same fork, same tripwire. A page paste needs `&mut PdfcerApp` for
        // the current page index and raises a `PageAction`; this function is a
        // pure function of the clip.
        Some(Clipped::Outline { .. }) => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "a bookmark paste must route to panels::bookmarks::clip"
            );
            Err(Refusal::NothingCopied)
        }
        Some(Clipped::Pages { .. }) => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "a page paste must route to app::dispatch::pageclip"
            );
            Err(Refusal::NothingCopied)
        }
        Some(Clipped::FormField(_)) => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "a form field paste must route to canvas::fieldclip::paste; app::dispatch::clipboard owns that fork"
            );
            Err(Refusal::NothingCopied)
        }
        Some(Clipped::Attachment(_)) => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "an attachment paste must route to panels::attachments::clip; an embedded file does not live on a page and this function pastes onto one"
            );
            Err(Refusal::NothingCopied)
        }
        None => Err(Refusal::NothingCopied),
    }
}

/// Paste a clip onto `page`, raising the action that authors it — **page
/// content, annotations, or both**.
#[allow(clippy::too_many_arguments)]
fn paste_clip(
    page: usize,
    bytes: &[u8],
    from: usize,
    count: usize,
    annotations: usize,
    anchor: Option<(f64, f64)>,
    target: Option<egui::Pos2>,
    actions: &mut Vec<Action>,
) -> Result<(), Refusal> {
    //
    // The operator reported *"copy and paste still doesn't paste where the
    // cursor is, it just pastes near the copied object"*, and this rule needs
    // **both** halves: where the pointer is, and where the clip's own centre
    // was. Either one missing silently degrades to the offset, and the trace
    // said only `at=offset` — which is the outcome, not the cause.
    //
    // The two causes want opposite investigations:
    //
    // | missing | means | look at |
    // |---|---|---|
    // | the **cursor** | no canvas frame, or the pointer is not over the canvas | the paste's ROUTE — a ribbon or menu press has the pointer somewhere else |
    // | the **anchor** | the clip carries no centre, computed at COPY time | `copy`, and whether `ObjectClip::bbox` came back empty for the selection |
    //
    // ⇒ A trace that cannot tell them apart makes the operator's report
    // unfalsifiable from a log, which is exactly the position this project
    // spent a morning in over a save that closed. One word ends it.
    let why = match (target, anchor) {
        (Some(_), Some(_)) => "cursor",
        (None, Some(_)) => "offset-no-cursor",
        (Some(_), None) => "offset-no-anchor",
        (None, None) => "offset-neither",
    };
    let (dx, dy) = match (target, anchor) {
        (Some(t), Some((cx, cy))) => (f64::from(t.x) - cx, f64::from(t.y) - cy),
        _ => {
            let offset = if from == page { PASTE_OFFSET_PT } else { 0.0 };
            (offset, -offset)
        }
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            // `annots=` is on this line because a wrong build gets it wrong
            // invisibly: a clip whose annotation payload the serialiser dropped
            // pastes its content perfectly and traces `objects=3` either way.
            // It is the number the annotation half of this feature lives or
            // dies on, and the one a driven check reads.
            "clipboard-paste kind=selection page={page} from={from} objects={count} \
             annots={annotations} at={} dx={dx:.1} dy={dy:.1}",
            why
        )
    });
    actions.push(
        crate::app::actions::VectorAction::PasteObjects {
            page,
            clip: bytes.to_vec(),
            // Down the page is NEGATIVE in PDF user space because y increases
            // upward — the identical trap `paste` names one function up, and
            // worth repeating rather than cross-referencing because getting it
            // backwards produces a paste that goes up-and-right, which looks
            // deliberate and is the kind of thing nobody reports.
            at: pdfcer_core::vector::Matrix::translate(dx, dy),
        }
        .into(),
    );
    Ok(())
}

#[cfg(test)]
mod tests;
