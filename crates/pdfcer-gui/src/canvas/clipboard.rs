//! # `canvas::clipboard` — **cut, copy and paste on the canvas**
//!
//! ## What this closes
//!
//! The operator, 2026-08-19: *"also the standard copy/paste and I didn't try cut
//! so possibly that one too aren't implemented."*
//!
//! They were not. `Ctrl+C` copied **text** — a swept range, through
//! `canvas::textsel::clipboard` — and that was the whole of this shell's
//! clipboard. `Ctrl+X` and `Ctrl+V` did nothing anywhere, and no ribbon control
//! offered any of the three: `RIBBON_IA.md`'s Edit ▸ Clipboard group had been
//! deleted rather than shipped empty, on the correct P3 grounds that a caption
//! over nothing is worse than no caption.
//!
//! ## ★★ What is expressible, and what is not — measured, not assumed
//!
//! `EditSession` has **157 public verbs** and the relevant question is which of
//! them can put something back on a page. Measured 2026-08-19:
//!
//! | subject | copy | paste | verdict |
//! |---|---|---|---|
//! | **markup / comments** | `annot_author::spec_from_dict` | `add_markup` | ✅ **both halves exist** |
//! | **text** (swept) | extraction | the clipboard is the destination | ✅ already shipped |
//! | **an image** | — | `add_image` | ◐ paste exists, no accessor reads one back out |
//! | **page content** (a path) | the decomposition | ⛔ **nothing** | blocked |
//!
//! So this module implements the row that is complete, and the ⛔ row is a
//! **dated citation** rather than a promise: no `paste`, no `duplicate`, no
//! `insert_object`, no `add_path` anywhere in `edit.rs`, checked 2026-08-19.
//!
//! ★ **That is not a small subset.** The things this operator actually copies
//! between sheets are revision clouds, notes, stamps and callouts — every one of
//! them an annotation. Copying a *path* is the rarer act and the one he has not
//! reported wanting.
//!
//! ## ★ Why the clipboard is in `egui::Memory` and not the OS clipboard
//!
//! Because a `MarkupSpec` is not text and the OS clipboard carries bytes with a
//! declared format. Putting one there would mean inventing a pdfcer-specific
//! flavour, which is a real feature (it is how you would paste between two
//! pdfcer windows) and is not what was asked for. What was asked for is
//! *"copy this cloud onto sheet 12"*, which is one process.
//!
//! It is **application-scoped**, like the armed tool and the text pen: a spec
//! copied in one document pastes into the next one opened. That is what every
//! editor does and it is the behaviour that makes copying between two drawings
//! possible at all — this shell opens one document at a time, so a
//! document-scoped clipboard would make cross-drawing copying impossible rather
//! than merely awkward.
//!
//! ## ★★ Where the paste lands, and why it is not "in place"
//!
//! Offset by [`PASTE_OFFSET_PT`], down and to the right, **except** when the
//! paste is onto a different page — where it lands at the original coordinates.
//!
//! Both halves are the convention and both have a reason:
//!
//! - **Same page → offset.** A paste that landed exactly on the original is
//!   invisible: the operator presses `Ctrl+V`, sees no change, presses it four
//!   more times, and has five stacked copies they cannot separate. Every editor
//!   offsets for this reason.
//! - **Different page → in place.** The whole point of copying a revision cloud
//!   to sheet 12 is that it should be *where it was on sheet 1*. Offsetting
//!   would move it for no reason the operator asked for, and they would have to
//!   drag it back.
//!
//! ## What `Ctrl+C` does when text is swept
//!
//! **Text wins.** `canvas::textsel::clipboard` owns `Ctrl+C` and keeps it: a
//! swept range is a more specific statement than a selected annotation, the
//! operator made it more recently, and every program in the class resolves the
//! collision the same way. This module's copy runs only when no text is swept.

use pdfcer_core::annot_author::MarkupSpec;
use pdfcer_core::object::ObjId;

/// **Whether a text gesture owns `Ctrl+C` / `Ctrl+X` this frame**, so the object
/// clipboard must stand aside.
///
/// Defect O18's enforcement point. **This module's header has always**
/// *claimed* that text wins; this is the function that makes the claim true,
/// and it lives here, beside the claim, rather than beside the caller that
/// consults it — so a reader who arrives at the header's promise finds its
/// enforcement in the same file rather than having to trust it.
///
/// It also has to live somewhere other than `app::dispatch`, which R2's
/// 1,500-line ceiling put over the limit the moment this was added. That was
/// the forcing function; this is the right home independently of it.
///
/// # The two claimants, and why both count
///
/// | claimant | what the operator did | who copies it |
/// |---|---|---|
/// | a composing draft | put a caret in a text box, possibly with a selection | `canvas::textedit::keys` |
/// | a live text sweep | dragged across text on the page | `canvas::textsel::clipboard` |
///
/// A draft counts **even with no selection inside it**, and that is deliberate.
/// `canvas::textedit::composing` is also true for a focused ordinary widget —
/// the Find field, the status bar's page box — and an operator pressing Ctrl+C
/// in the Find field is copying their search term. Letting the object clipboard
/// answer that keystroke would copy something off the page instead, which is
/// `DEFECTS.md` D1's failure exactly: a canvas taking a chord that belonged to
/// the widget the operator was looking at.
///
/// # ★ Why "the sweep produced no text" is not checked here
///
/// A live-but-empty selection still counts as text owning the chord. The
/// alternative — falling through to the object clipboard when the sweep turns
/// out to be empty — would make one keystroke mean two different things
/// depending on a property the operator cannot see, and the failure would be
/// silent and destructive on the cut path. `textsel::clipboard::copy` refuses an
/// empty string on its own and traces the refusal; that is the right place for
/// it, because it is the only place that has the string.
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
///
/// Ten — a little over three millimetres. Large enough to be unmistakable at
/// fit-page zoom on an A1 sheet (where it is about four screen pixels, which is
/// small but is a visible step against a hairline), and small enough that the
/// copy is plainly *the same mark, moved* rather than something placed
/// elsewhere. Acrobat uses roughly this; Illustrator's default is 10 pt exactly.
pub const PASTE_OFFSET_PT: f64 = 10.0;

/// What the canvas clipboard is holding.
///
/// One variant today. It is an `enum` rather than a bare `MarkupSpec` because
/// the module header's table has three more rows in it, and the day page
/// content becomes pasteable this type is where that arrives — a `Vec<u8>` of
/// content-stream operators, or an image handle, sitting beside this. A bare
/// spec would make that a rewrite of every caller.
#[derive(Debug, Clone, PartialEq)]
pub enum Clipped {
    /// A markup annotation, ready for `add_markup`.
    ///
    /// Carries the page it came from, so a paste onto a *different* page can
    /// land in place while a paste onto the same one offsets. See the module
    /// header for why those two answers differ.
    Markup {
        /// The spec, verbatim from `spec_from_dict`.
        spec: Box<MarkupSpec>,
        /// The 0-based page it was copied from.
        page: usize,
        /// ★★★ **The point that is placed under the cursor on a paste** —
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
        /// ★ It is also what preserves relative geometry inside a
        /// multi-object paste **by construction**: one anchor for the whole
        /// clip means one delta, applied to everything, so the arrangement
        /// cannot drift no matter how many items are in it.
        ///
        /// `None` when the geometry could not be read — the paste then falls
        /// back to the offset rule rather than guessing, because a clip that
        /// pasted at `(0, 0)` would land in the bottom-left corner of the
        /// sheet and read as data loss.
        anchor: Option<(f64, f64)>,
        /// ★★★ **Everything the spec cannot say** — `/CA`, `/Contents`, `/T`
        /// and `/M`.
        ///
        /// # Why this field exists, and why it did not on 2026-08-27
        ///
        /// A copy used to be a round trip through `MarkupSpec`: read the
        /// annotation into a spec, author a new one from it. That is lossless
        /// only for what a spec can express, and on 2026-08-28 this shell
        /// gained the ability to author two things it cannot — a **note** with
        /// an author and a date, and an **opacity**.
        ///
        /// Without this field, copying a signed, dated, 40 %-opaque cloud and
        /// pasting it produced an anonymous, undated, opaque one — **which
        /// looks right on the page**, because the words live in a pop-up this
        /// shell does not draw and the opacity difference is only visible
        /// against the artwork underneath. A loss nobody would report.
        ///
        /// ⇒ The general form, worth stating because it will recur: **a copy
        /// implemented as a re-author is only as faithful as the authoring
        /// type**, and it silently loses ground every time the authoring side
        /// gains a key. `pdfcer-core`'s `copy_annotations` is the route that
        /// does not have that property — it returns an `ObjectClip` owning the
        /// annotation itself — and moving to it is filed as a question rather
        /// than assumed, because it is not yet known whether a `/Popup`, an
        /// `/IRT` reply chain or an `/RC` rich-text body survive that path
        /// either. This field closes the two losses this shell **created
        /// today**; it does not claim to close the family.
        options: Box<pdfcer_core::edit::MarkupOptions>,
    },
    /// ★★★ **Page content** — a path, a text run, an image, in any mixture.
    ///
    /// This variant is what the type's own docs predicted: *"the day page
    /// content becomes pasteable this type is where that arrives."* `Pass 120.0`
    /// shipped `ObjectClip` on 2026-08-20 and this is that day.
    ///
    /// # ★★ Why the BYTES and not the `ObjectClip`
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
    /// ★ The clip **owns its resources**, transitively, by value. So copying
    /// from one document, closing it, and pasting into another works — and
    /// cross-document paste is not a special case but the same call.
    Content {
        /// `ObjectClip::to_bytes` — magic-prefixed, versioned, bit-exact.
        bytes: Vec<u8>,
        /// The 0-based page it was copied from, for the same reason
        /// [`Self::Markup`] carries one: a paste onto the *same* page offsets
        /// so the copy is visible, a paste elsewhere lands in place.
        page: usize,
        /// How many objects are in it, for the trace and for a sentence.
        ///
        /// Carried rather than re-derived because reading it back means
        /// deserialising, and the count is wanted in places that have no reason
        /// to.
        count: usize,
        /// ★★★ **The point that is placed under the cursor on a paste** —
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
        /// ★ It is also what preserves relative geometry inside a
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
    /// ★★★ **A form field**, as of 2026-08-29 — `OPERATOR_REQUESTS.md` O58.
    ///
    /// The third thing this clipboard can hold, and the one that needed a
    /// module of its own: see [`crate::canvas::fieldclip`] for why a form
    /// field could not previously be copied at all, and for the two senses
    /// `Ctrl+V` and `Ctrl+Shift+V` carry.
    ///
    /// # Why it is a variant here and not a second clipboard
    ///
    /// **One clipboard holds one thing.** Copying a markup after copying a
    /// field must replace it, because `Ctrl+V` has to mean exactly one act at
    /// any moment. A second `egui::Memory` key would give the shell two live
    /// clipboards and `edit.paste` a choice to make between them — and any rule
    /// it used to choose (most recent? most specific? whatever the selection
    /// is?) would be a rule the operator cannot see.
    ///
    /// Boxed because `ClippedField` carries a whole `Draft` and this enum is
    /// cloned on every read; the other two variants are a `Box` and a `Vec`
    /// respectively, so boxing keeps the variants the same order of size and
    /// stops `clippy::large_enum_variant` from being right.
    FormField(Box<crate::canvas::fieldclip::ClippedField>),
    /// **An embedded file** — its name, its decoded bytes and its description.
    ///
    /// `OPERATOR_REQUESTS.md` O59's family, and the one the verb-coverage gate
    /// found on 2026-09-01: `copy_attachment` / `cut_attachment` /
    /// `paste_attachment` shipped in `Pass 173.0` and this shell named none of
    /// them, so an attachment could not be moved between two open documents —
    /// odd, now that pdfcer is multi-document.
    ///
    /// ★★ **Carries the DECODED bytes**, which is the engine's choice and worth
    /// restating: `AttachmentClip` holds what `extract-attachment` would give
    /// you and what `attach_file` expects on the way back in. Carrying the raw
    /// stream instead would carry its filter chain with it, and a paste would
    /// have to re-derive whether that chain still applied in the destination.
    ///
    /// ★ It carries **no page**, unlike every other variant here. A
    /// document-level embedded file does not live on a sheet, so there is no
    /// same-page/different-page question and no paste offset — which is why the
    /// paste is in the Attachments panel rather than on the canvas.
    Attachment(Box<pdfcer_core::attachments::AttachmentClip>),
    /// ★★★ **Whole pages** — `OPERATOR_REQUESTS.md` O59, 2026-08-29.
    ///
    /// # ★★ The bytes ARE a PDF, and that is not an implementation detail
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
    /// ★★★ **A bookmark and everything filed under it** — O59 item 3.
    ///
    /// # ★★ Why this one is NOT bytes, unlike its two neighbours
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
    /// ★ `Box`ed for `FormField`'s reason: this enum is cloned on every read
    /// and an `OutlineClip` carries a whole subtree of titles, destinations and
    /// colours.
    Outline {
        /// The copied roots and their children, in document order.
        clip: Box<pdfcer_core::outline::OutlineClip>,
        /// The deepest 0-based page any destination in the clip names.
        ///
        /// ★★ Carried rather than re-walked because it answers the one question
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

/// Why a copy or a cut could not happen.
///
/// Each is a **sentence on the status row**, never a silence — the standing
/// answer in this shell since `DEFECTS.md` D4a, and the same posture
/// `canvas::resizing`'s six refusals take. A `Ctrl+C` that does nothing and
/// says nothing is indistinguishable from a broken keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// ★★★ **The cut's DELETE half would be refused, so its copy half did not
    /// run either.**
    ///
    /// Only `cut` can answer this, and only `cut` returns it: a plain copy
    /// changes nothing and is correct on a document that forbids every change.
    ///
    /// It exists as its own variant rather than reusing [`Self::Unreadable`]
    /// because the operator's next move differs — a clip the engine could not
    /// assemble is a fact about *the selection*, and this is a fact about *the
    /// document*, true of every annotation in it until the signature or the
    /// encryption goes. The sentence is on the status row already, put there by
    /// `app::status::decline`, which is the same surface the three other doors
    /// onto this verb use.
    DeleteRefused(crate::panels::properties::annotdelete::Refusal),
    /// ★★★ **The clipboard could not carry it, so the cut was refused before
    /// anything was removed.**
    ///
    /// `pdfcer-core`'s `CutWouldNotSurvive { subtype }`, and the subtype travels
    /// so the sentence can name it — a greyed button has one static tooltip and
    /// the operator may have several things selected.
    ///
    /// # Why a cut is refused where a copy is not
    ///
    /// The engine's own words: *"a copy of something pdfcer cannot carry costs
    /// nothing — the original stays, the clip carries an `Unsupported` marker,
    /// the paste declines by name. A cut of the same thing is a deletion
    /// wearing a clipboard's clothes."*
    ///
    /// ★ `&'static str` rather than an enum, matching `canvas::cutgate::Blocker`
    /// and for its reason: the set of subtypes is the file format's, and a
    /// second taxonomy here would be one more thing to keep in step with
    /// another crate.
    CutWouldNotSurvive(&'static str),
    /// Nothing is selected.
    NothingSelected,
    /// The engine refused to copy the selection.
    ///
    /// ★★ **This variant was `ContentNotAnnotation` until 2026-08-20**, and it
    /// said: *"`EditSession` has no verb that puts page content back, so a copy
    /// would be offering a paste that could never happen."* True when it was
    /// written, and `Pass 120.0` made it false — the operator had been asking
    /// for cut/copy/paste of page content since the first week.
    ///
    /// What replaces it is the engine's own refusal, which is a genuinely
    /// different fact: a clip it could not assemble. Kept as one variant rather
    /// than mirroring the engine's taxonomy, for the reason
    /// `canvas::resizing`'s note gives about the same choice — a shell that
    /// modelled the engine's internals a second time is decision 058's failure
    /// mode, and this module has just watched one of those expire.
    EngineRefused,
    /// The selected annotation's dictionary would not yield a spec.
    ///
    /// Reachable on an annotation whose subtype `annot_author` does not author
    /// — a link, a widget, a `/FileAttachment` — and on a malformed one.
    Unreadable,
    /// The clipboard is empty.
    NothingCopied,
}

/// Read the clipboard.
#[must_use]
pub fn read(ctx: &egui::Context) -> Option<Clipped> {
    ctx.data(|d| d.get_temp::<Clipped>(egui::Id::new(KEY)))
}

/// Write it.
pub fn store(ctx: &egui::Context, clipped: Clipped) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(KEY), clipped));
}

/// Copy the selected annotation, returning what was put on the clipboard.
///
/// # Errors
///
/// Every member of [`Refusal`] except [`Refusal::NothingCopied`], which only a
/// paste can raise.
pub fn copy(ctx: &egui::Context, doc: &OpenDoc) -> Result<Clipped, Refusal> {
    use pdfcer_core::annot_author::spec_from_dict;
    use pdfcer_core::object::Object;

    let Some(selected) = doc.selection.annot() else {
        // ★★★ PAGE CONTENT, as of 2026-08-20. This branch used to refuse it by
        // name — *"pdfcer has no verb that puts page content back, so a copy
        // would be offering a paste that could never happen"* — and it was the
        // operator's oldest open request.
        //
        // ★ The ORDER here is the annotation first and content second, which is
        // the opposite of how the selections are populated and is deliberate: a
        // ce dimension and a markup are annotations that a content selection can
        // never name, so asking the narrower question first means the broad one
        // never has to exclude anything.
        return copy_content(ctx, doc);
    };
    let graph = doc.session.graph();
    let Some(Object::Dict(dict)) = doc.session.value(selected.target.id) else {
        return Err(Refusal::Unreadable);
    };
    let spec = spec_from_dict(&graph, dict).map_err(|_| Refusal::Unreadable)?;
    // ★★ The keys the spec cannot carry, read from the SAME dictionary the spec
    // came from — one read, so the two halves of the copy cannot describe
    // different annotations.
    let options = Box::new(carried_options(
        doc,
        selected.target.page,
        selected.target.id,
    ));
    // ★ The `/Rect` centre, from the SAME dictionary the spec came from — one
    // read, so the anchor and the geometry cannot describe different
    // annotations. `OPERATOR_REQUESTS.md` O73; see `Clipped::Markup::anchor`.
    let anchor = rect_centre_of(dict);
    let clipped = Clipped::Markup {
        spec: Box::new(spec),
        page: selected.target.page,
        anchor,
        options,
    };
    store(ctx, clipped.clone());
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("clipboard-copy kind=markup page={}", selected.target.page)
    });
    Ok(clipped)
}

/// Copy the selected **page content** — a path, a text run, an image, in any
/// mixture.
///
/// # ★★ What the engine does that this could not have done for itself
///
/// This shell's own request scoped the work as *"expose the copy engine you
/// already have at object granularity"*, on the strength of `import_object`
/// being a recursive, reference-remapping, cycle-guarded object-graph copy.
/// That reading was correct **and it was the smaller half**, in one specific
/// place worth writing down:
///
/// > `import_object` copies **indirect objects**. A page's content objects are
/// > not indirect objects — a path, a text run and an image invocation are byte
/// > ranges inside a content stream, and the operators in those bytes name
/// > their resources **by page-local name**. On the destination page, `/F1` is a
/// > different font. Paste the bytes verbatim and you get the right glyphs in
/// > the wrong typeface, or nothing at all. **Neither failure errors**, and
/// > neither is visible in a diff, because *a resource name is not a
/// > reference.*
///
/// So the clip records which names each item consumes, carries the objects
/// behind them by value, and paste re-binds every one to a fresh name on the
/// destination page — rewriting the names inside the copied bytes. That is the
/// feature; `import_object` was the prerequisite.
///
/// Recorded here rather than in a request file because it is the general
/// lesson: **a graph copy does not copy a namespace.**
///
/// # Errors
///
/// [`Refusal::NothingSelected`] for an empty selection,
/// [`Refusal::EngineRefused`] when the clip could not be assembled.
fn copy_content(ctx: &egui::Context, doc: &OpenDoc) -> Result<Clipped, Refusal> {
    let page = doc.view.page_index;
    let objects = doc.selection.object_indices_on(page);
    if objects.is_empty() {
        return Err(Refusal::NothingSelected);
    }
    // ★ `&self`, and it commits nothing — which is what makes `cut` below one
    // undo entry without a `cut_objects` call: only the deletion is an edit.
    let clip = doc
        .session
        .copy_objects(page, &objects)
        .map_err(|_| Refusal::EngineRefused)?;
    // ★★ The union of the copied objects' bounds, converted to PDF user space.
    //
    // `CanvasTargetProvider::bounds` answers in **canvas** space, which is the
    // space the overlay draws in; the paste verb takes a PDF-space matrix. The
    // conversion is `viewer::canvas_to_pdf_space`, the one bridge between the
    // two, and it declines on a page whose device geometry does not invert —
    // in which case the paste falls back to the offset rule rather than
    // inventing a coordinate. See `Clipped::Content::anchor`.
    let anchor = doc.page_objects().and_then(|provider| {
        let page_dict = doc.pages.get(page)?;
        let mut union: Option<egui::Rect> = None;
        for &index in &objects {
            let target = crate::canvas::target::TargetId::Object(index as u64);
            if let Some(r) = provider.bounds(page, target) {
                union = Some(union.map_or(r, |u| u.union(r)));
            }
        }
        let centre = crate::viewer::canvas_to_pdf_space(union?.center(), page_dict)?;
        Some((f64::from(centre.x), f64::from(centre.y)))
    });
    let clipped = Clipped::Content {
        count: clip.len(),
        bytes: clip.to_bytes(),
        page,
        anchor,
    };
    store(ctx, clipped.clone());
    // ★★★ AND A MARKER ON THE OS CLIPBOARD, WITHOUT WHICH CTRL+V DOES NOT
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
    // ★ What goes there is a SENTENCE RATHER THAN THE BYTES, and both halves of
    // that are deliberate:
    //
    // * a human who pastes into a text editor gets something that says what
    //   happened, not a screenful of binary;
    // * the real payload is `ObjectClip::to_bytes`, which belongs under a
    //   **private clipboard format** so a pdfcer→pdfcer paste is lossless — and
    //   registering one is a Win32 `RegisterClipboardFormat` call this shell
    //   does not make yet. That is the remaining half of the operator's item 3,
    //   named here rather than left as a silence.
    //
    // Until then the marker is what makes the chord arrive and the in-memory
    // clip is what is pasted, so a pdfcer→pdfcer paste is already lossless. What
    // is missing is pdfcer→pdfcer **across two processes**.
    // ★★★ **AND A PICTURE BESIDE IT, as of 2026-08-31** —
    // `OPERATOR_REQUESTS.md` O71: *"so we can copy and paste them … outside of
    // the pdfcergui."*
    //
    // The marker sentence and the bitmap go on in ONE clipboard transaction,
    // and that is not an optimisation. `EmptyClipboard` is per-open, so two
    // calls would mean the second erased the first — and if the picture went on
    // second, this application's own `Ctrl+V` would stop arriving for the
    // reason the paragraphs above set out. `native_window::clipboard` writes
    // both or neither.
    //
    // ★ It FALLS BACK rather than failing. A clipboard another process is
    // holding, a render that declines, a degenerate clip — each of those loses
    // the picture and none of them loses the copy, so the marker still goes on
    // by the route it always did and the operator's `Ctrl+V` still works. What
    // they lose is the paste into Word, and the trace says which.
    let marker = crate::text::clipboard::os_marker(objects.len());
    if crate::canvas::clipimage::publish(&clip, &marker).is_none() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "clipboard-image-declined".to_owned()
        });
        ctx.copy_text(marker);
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // ★ The COUNT and the BYTE LENGTH, because those are what a wrong build
        // gets wrong: a clip that copied the operators and dropped the
        // resources is a plausible-looking clip that pastes the right glyphs in
        // the wrong typeface, and it is several hundred bytes shorter.
        let bytes = match &clipped {
            Clipped::Content { bytes, .. } => bytes.len(),
            Clipped::Markup { .. }
            | Clipped::FormField(_)
            | Clipped::Pages { .. }
            | Clipped::Outline { .. }
            | Clipped::Attachment(_) => 0,
        };
        format!(
            "clipboard-copy kind=content page={page} objects={} bytes={bytes}",
            objects.len()
        )
    });
    Ok(clipped)
}

/// Copy, then delete — cut.
///
/// # ★ Why this is copy-then-delete and not a verb of its own
///
/// Because a cut *is* those two acts, and expressing it as two calls to
/// functions that are each independently tested is how it stays correct. The
/// one thing that must not be two acts is the **undo**: a cut the operator
/// takes back with one `Ctrl+Z` must return the annotation, not leave them
/// pressing it twice.
///
/// That is already true and is not this module's doing — `Action::DeleteAnnot`
/// goes through `vector_edit`, which lands one `EditSession` command, and the
/// copy half changes no document at all. So the cut is one undo entry because
/// only one half of it is an edit.
///
/// # Errors
///
/// As [`copy`].
pub fn cut(
    ctx: &egui::Context,
    doc: &OpenDoc,
    actions: &mut Vec<Action>,
) -> Result<Clipped, Refusal> {
    // ★★★ **ASK WHETHER THE DELETE CAN HAPPEN BEFORE THE COPY DOES.**
    //
    // This is the fourth door onto `delete_annotation`, found by an adversarial
    // review on 2026-08-29 after the other three had been gated the day before,
    // and it is the worst of the four:
    //
    // On a certified or encrypted document, `Ctrl+X` over a markup **copied it
    // to the clipboard**, raised the Delete, watched the engine refuse into
    // `vector_edit`'s `Err` arm — one trace line, nothing said — and then
    // `annots::delete` cleared the selection anyway, because it clears after
    // the funnel rather than on success.
    //
    // ⇒ So the operator was left with the annotation still on the page, no
    // selection, no explanation, **and a clipboard holding a copy of it**. The
    // next `Ctrl+V` duplicates the thing they were trying to move. That is a
    // half-executed cut, which is the one outcome the ordering note below
    // exists to prevent — stated there for the copy half and never asked for
    // the delete half.
    //
    // ★★ The whole gesture is refused rather than degraded to a copy. A cut
    // that silently becomes a copy is a different verb wearing the operator's
    // chord, and they would find out by pasting.
    //
    // ★ Asked HERE rather than in `annots::delete`, and the difference matters:
    // the delete arm is reached by four routes and must stay a routing arm, but
    // only this route has a **second half to call off**. Gating inside the arm
    // would refuse the delete and leave the copy already on the clipboard.
    // ★★★ AND WHETHER THE CLIPBOARD COULD CARRY IT AT ALL — the second half of
    // the same question, added 2026-08-29 when `pdfcer-core` shipped cut for
    // every class and asked for exactly this.
    //
    // First, before the delete gate below and before the copy, because it is
    // the cheaper question and because the two refusals are about different
    // things: that one is *"this document forbids removing it"*, this one is
    // *"pdfcer could not put it back"*. An operator meeting the wrong one of the
    // two goes looking in the wrong place.
    //
    // ★ `edit.cut` is already greyed on this predicate (`selection.cut_permitted`),
    // so a pointer never reaches here. A CHORD does: `Ctrl+X` is dispatched
    // through the keymap without consulting command enablement. Greying removes
    // the invitation; this removes the silence.
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
    if let Some(selected) = doc.selection.annot()
        && let Some(why) = crate::panels::properties::annotdelete::gate(doc, &selected.target)
    {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("clipboard-cut-refused reason=delete-gated why={why:?}")
        });
        // ★ The REASON travels out; the sentence is written where every other
        // clipboard refusal's is. This module changes no document and words no
        // decline — its own header's contract — and `crate::app::status::decline`
        // is `pub(super)` inside `crate::app` and deliberately out of reach from
        // the canvas. `crate::text::clipboard` already maps every other variant
        // to a sentence, and this one reuses `annotdelete`'s catalog rather than
        // writing a second wording for one fact.
        return Err(Refusal::DeleteRefused(why));
    }
    let clipped = copy(ctx, doc)?;
    // ★★ COPY RUNS FIRST, and the engine makes the same point about its own
    // `cut_objects`: *"a selection that cannot be copied is refused with
    // nothing deleted. Reversed, a cut whose copy half failed would take the
    // objects away with nothing on the clipboard — the one outcome the operator
    // cannot recover from by pasting."* The `?` above is that ordering.
    //
    // ★ And this is deliberately NOT `EditSession::cut_objects`, though that
    // verb exists and would work. `cut_objects` is copy-then-delete inside the
    // engine; doing it here as copy-then-`DeleteSelection` keeps the delete
    // going through the funnel like every other edit, so it lands one
    // `EditSession` command and one undo entry by the same mechanism as
    // everything else — and this module goes on changing no document, which is
    // what lets its refusals be unit-tested without one.
    //
    // The undo property is unchanged either way: only one half of a cut is an
    // edit.
    // The delete is raised through the funnel like every other edit, rather
    // than performed here: this module changes no document.
    match (&clipped, doc.selection.annot()) {
        (Clipped::Markup { .. }, Some(selected)) => {
            actions.push(Action::Annot(
                crate::app::actions::annot::AnnotAction::Delete {
                    page: selected.target.page,
                    id: selected.target.id,
                },
            ));
        }
        (Clipped::Content { page, .. }, _) => {
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
        }
        (Clipped::Markup { .. }, None) => {}
        // ★ A form field's cut is `canvas::fieldclip::cut`, which raises
        // `FieldAction::DeleteWidget` -- a widget is addressed by its FIELD's
        // name and an index within it, not by the `ObjId` this arm's siblings
        // use, because one field can draw boxes on three pages. Reaching this
        // arm means `app::dispatch::clipboard` routed a field copy into the
        // markup path.
        // ★ A page cut is `dispatch::pageclip`, which raises
        // `PageAction::DeletePages` -- pages are addressed by index in the
        // document, not by an `ObjId` on a page, so nothing in this arm's
        // vocabulary can express one. Same tripwire as the field arm below.
        // ★ A bookmark cut is `panels::bookmarks::clip`, which raises
        // `BookmarkAction::Delete` -- an outline item is addressed by `ObjId`
        // in a document-level tree, not by a page and an index, so this arm's
        // vocabulary cannot express one either. Third tripwire, same shape.
        (Clipped::Outline { .. }, _) => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "a bookmark cut must route to panels::bookmarks::clip; the canvas clipboard has no outline vocabulary"
            );
        }
        (Clipped::Pages { .. }, _) => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "a page cut must route to app::dispatch::pageclip; the canvas clipboard has no page vocabulary"
            );
        }
        (Clipped::FormField(_), _) => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "a form field cut must route to canvas::fieldclip::cut; app::dispatch::clipboard owns that fork"
            );
        }
        // ★ An attachment cut is `panels::attachments::clip`, which raises
        // `AttachmentAction::Detach` -- an embedded file is addressed by its
        // `/EmbeddedFiles` name-tree KEY, which is neither a page nor an
        // `ObjId` on one, so this arm's vocabulary cannot express it. Fourth
        // tripwire, same shape as the three above, and they have collectively
        // fired twice during development.
        (Clipped::Attachment(_), _) => {
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
                Clipped::Markup { .. } => "markup",
                Clipped::Content { .. } => "content",
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
///
/// # Errors
///
/// [`Refusal::NothingCopied`] when the clipboard is empty.
pub fn paste(
    ctx: &egui::Context,
    page: usize,
    target: Option<egui::Pos2>,
    actions: &mut Vec<Action>,
) -> Result<(), Refusal> {
    let (spec, from, anchor, options) = match read(ctx) {
        Some(Clipped::Markup {
            spec,
            page: from,
            anchor,
            options,
        }) => (spec, from, anchor, options),
        // ★ Page content takes its own path: the clip is bytes and the verb is
        // `paste_objects`, which takes a page-space MATRIX rather than a
        // displacement — so the offset below cannot be shared even though the
        // rule that decides it is.
        Some(Clipped::Content {
            bytes,
            page: from,
            count,
            anchor,
        }) => {
            return paste_content(page, &bytes, from, count, anchor, target, actions);
        }
        // ★★ Same fork as the cut above, and the same tripwire. A field paste
        // needs `&OpenDoc` -- to find a free name, and to know what the source
        // field could not carry -- which this function does not take and must
        // not grow, because every other paste here is a pure function of the
        // clip. `app::dispatch::clipboard` branches before calling either.
        // ★ Same fork, same tripwire. A page paste needs `&mut PdfcerApp` for
        // the current page index and raises a `PageAction`; this function is a
        // pure function of the clip.
        Some(Clipped::Outline { .. }) => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "a bookmark paste must route to panels::bookmarks::clip"
            );
            return Err(Refusal::NothingCopied);
        }
        Some(Clipped::Pages { .. }) => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "a page paste must route to app::dispatch::pageclip"
            );
            return Err(Refusal::NothingCopied);
        }
        Some(Clipped::FormField(_)) => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "a form field paste must route to canvas::fieldclip::paste; app::dispatch::clipboard owns that fork"
            );
            return Err(Refusal::NothingCopied);
        }
        Some(Clipped::Attachment(_)) => {
            debug_assert!(
                false,
                // ui-text-exempt: a debug_assert message for a developer; never rendered.
                "an attachment paste must route to panels::attachments::clip; an embedded file does not live on a page and this function pastes onto one"
            );
            return Err(Refusal::NothingCopied);
        }
        None => return Err(Refusal::NothingCopied),
    };
    // ★★★ **The pointer wins where there is one** — `OPERATOR_REQUESTS.md`
    // O73: *"When I cut or copy an object, when I paste it should paste where
    // the mouse cursor is sitting."*
    //
    // `target` is already resolved to PDF user space by the caller, and is
    // `None` when the canvas has never drawn. The two older rules below survive
    // as the fallback and are unchanged; what has changed is that they are no
    // longer the ONLY answer.
    //
    // ★ Where the pointer is not over the canvas — over a dock, over the
    // ribbon, off the window — the caller has already substituted the
    // viewport's centre through `zoom::anchor_point`. That is one rule, in one
    // place, shared with the zoom anchor, rather than a second convention for
    // an operator to learn.
    let (dx, dy) = match (target, anchor) {
        // The mark is placed so that ITS OWN CENTRE lands under the cursor.
        //
        // Centre rather than top-left, and it is Inkscape's rule and
        // Illustrator's: the operator is pointing at where the thing should
        // BE, not at where its bounding box should begin. Top-left is the
        // Word/Explorer convention and belongs to a text caret, not to a
        // drawing canvas. Acrobat likewise drops a pasted comment centred on
        // the click.
        (Some(t), Some((cx, cy))) => (f64::from(t.x) - cx, f64::from(t.y) - cy),
        // See the module header: same page offsets so the copy is visible, a
        // different page lands in place so a mark copied to sheet 12 is where
        // it was on sheet 1.
        _ => {
            let offset = if from == page { PASTE_OFFSET_PT } else { 0.0 };
            (offset, -offset)
        }
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "clipboard-paste page={page} from={from} at={} dx={dx:.1} dy={dy:.1}",
            if target.is_some() { "cursor" } else { "offset" }
        )
    });
    actions.push(Action::PasteMarkup {
        page,
        // ★ The note and the opacity, forwarded unchanged. The paste is a
        // *reproduction* of what was copied, so nothing here is re-derived from
        // the operator's current pen — a paste that picked up today's opacity
        // would be a different mark wearing the copied one's geometry.
        options,
        // Translated HERE, where the offset is decided, rather than in `apply`
        // — the funnel's own rule: an action carries a complete statement of
        // what the operator asked for, and geometry computed in the apply arm
        // cannot be tested without a document.
        spec: Box::new(translated(*spec, dx, dy)),
        dx,
        // ★ Down the page is **negative** in PDF user space because y increases
        // upward. The fallback arm above encodes that as `-offset`; the cursor
        // arm gets it for free, because both the target and the anchor are in
        // the same space and the subtraction cannot have a sign convention of
        // its own. Getting this backwards produces a paste that goes
        // up-and-right, which looks deliberate and is the kind of thing nobody
        // reports as a bug — they just think that is how it works.
        dy,
    });
    Ok(())
}

/// **An annotation's `/Rect` centre**, in PDF user space — the point a paste
/// places under the cursor.
///
/// `None` for a dictionary with no readable `/Rect`, which falls the paste
/// back to the offset rule rather than guessing. That direction is deliberate:
/// an unrecognised annotation pasting at the old offset is a mild surprise, and
/// one pasting at `(0, 0)` — the bottom-left corner of the sheet — reads as
/// data loss.
///
/// ★ Read from the raw dictionary rather than from the `MarkupSpec`, and the
/// reason is the same one `carried_options` gives: the spec is a *translation*
/// of the annotation, and every kind translates its geometry differently — an
/// ink stroke into a point list, a line into two ends, a square into corners.
/// `/Rect` is the one place every annotation states its extent in the same
/// terms (§12.5.2), so reading it needs no per-kind match and therefore cannot
/// silently omit a kind.
fn rect_centre_of(dict: &pdfcer_core::object::Dict) -> Option<(f64, f64)> {
    use pdfcer_core::object::Object;
    let Object::Array(values) = dict.get(b"Rect")? else {
        return None;
    };
    if values.len() != 4 {
        return None;
    }
    let n = |i: usize| match values.get(i)? {
        Object::Integer(v) => Some(*v as f64),
        Object::Real(v) => Some(*v),
        _ => None,
    };
    // Normalised (§7.9.5): a `/Rect` is not required to be written with its
    // lower-left first, and averaging the pair gives the same centre either
    // way — so no `min`/`max` pass is needed to get this right.
    Some(((n(0)? + n(2)?) / 2.0, (n(1)? + n(3)?) / 2.0))
}

/// Paste page content onto `page`, raising the action that authors it.
///
/// # ★ The offset rule is the markup one, and the geometry is not
///
/// Same page offsets so the copy is visible; a different page or document lands
/// in place, so a shape copied to sheet 12 is where it was on sheet 1. That
/// rule is shared with [`paste`] deliberately — two answers to *"where does a
/// paste land"* would be two things for the operator to learn.
///
/// What is **not** shared is how the offset is expressed. A markup carries a
/// `/Rect` and moves by a pair of numbers; page content moves by a **page-space
/// matrix**, which is the same contract `transform_objects` takes and the same
/// reason: `cm` composes into the CTM in force at that point in the stream, so
/// the engine conjugates by each item's own captured matrix and the caller
/// passes page space or nothing.
///
/// `Matrix::IDENTITY` is paste-in-place; `translate` is paste-with-offset. That
/// the same verb also gives paste-scaled and paste-rotated through
/// `Matrix::about` is why the request asked for a matrix rather than a
/// displacement, and it is what a future *paste special* is already built on.
///
/// # Errors
///
/// None today — the deserialisation happens in the apply arm, where the session
/// is. A clip this shell wrote is a clip this shell can read; one it cannot is
/// the engine's `ClipError::NotAClip`, and that reaches the status row through
/// `vector_edit` like every other engine refusal.
#[allow(clippy::too_many_arguments)]
fn paste_content(
    page: usize,
    bytes: &[u8],
    from: usize,
    count: usize,
    anchor: Option<(f64, f64)>,
    target: Option<egui::Pos2>,
    actions: &mut Vec<Action>,
) -> Result<(), Refusal> {
    // ★★★ The cursor rule (O73), expressed as the matrix this verb takes
    // rather than as a pair of numbers. See `paste` for the argument about the
    // CENTRE, which is shared: one delta for the whole clip, so relative
    // geometry inside a multi-object paste is preserved by construction rather
    // than by care.
    // ★★★ WHY it fell back, not just THAT it did — 2026-09-01.
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
    // | the **anchor** | the clip carries no centre, computed at COPY time | `copy_content`, and whether the provider could answer `bounds` for the page the selection is on |
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
            "clipboard-paste kind=content page={page} from={from} objects={count} \
             at={} dx={dx:.1} dy={dy:.1}",
            why
        )
    });
    actions.push(
        crate::app::actions::VectorAction::PasteObjects {
            page,
            clip: bytes.to_vec(),
            // ★ Down the page is NEGATIVE in PDF user space because y increases
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

/// Displace a spec by `(dx, dy)` in PDF user space.
///
/// # ★★ Why this is an exhaustive `match` and not a helper that "finds the
/// geometry"
///
/// Because the failure mode of the alternative is silent. A spec whose geometry
/// this function did not move would paste **on top of its original**, which is
/// precisely the invisible-paste problem the offset exists to prevent — and it
/// would happen only for the one annotation kind that was missed, so it would
/// read as a quirk of clouds, or of arrows, rather than as a bug.
///
/// Matching every variant by name means the day `pdfcer-core` adds a tenth
/// `MarkupSpec` this **fails to compile**. That is the whole design: a paste
/// that silently stopped offsetting for one kind is a defect nobody would
/// report, and a build error is a defect nobody can ship.
///
/// # The three non-geometric variants
///
/// `UnsupportedSubtype` and `BadGeometry` are `spec_from_dict`'s way of saying
/// *"this annotation is not one I author"* — [`copy`] never puts one on the
/// clipboard, because `add_markup` could not write it back. They are matched
/// here anyway, and returned unchanged, so that the exhaustiveness above is
/// real rather than papered over with a wildcard.
fn translated(spec: MarkupSpec, dx: f64, dy: f64) -> MarkupSpec {
    use pdfcer_core::annot_author::MarkupSpec as M;

    /// A rect moved. `Rect` is four numbers and the order is
    /// `(x0, y0, x1, y1)`; moving it means adding the delta to both corners,
    /// which is the one operation here that cannot be got wrong by transposing
    /// two fields, because both corners take the same pair.
    fn rect(r: pdfcer_core::page_tree::Rect, dx: f64, dy: f64) -> pdfcer_core::page_tree::Rect {
        // `llx/lly/urx/ury` — lower-left and upper-right, the PDF `/Rect`
        // spelling. Both corners take the SAME delta, which is what makes this
        // the one line here that cannot be got wrong by transposing a pair.
        pdfcer_core::page_tree::Rect {
            llx: r.llx + dx,
            lly: r.lly + dy,
            urx: r.urx + dx,
            ury: r.ury + dy,
        }
    }
    fn pt(p: (f64, f64), dx: f64, dy: f64) -> (f64, f64) {
        (p.0 + dx, p.1 + dy)
    }
    fn pts(v: Vec<(f64, f64)>, dx: f64, dy: f64) -> Vec<(f64, f64)> {
        v.into_iter().map(|p| pt(p, dx, dy)).collect()
    }

    match spec {
        M::Square {
            rect: r,
            border,
            interior,
            border_width,
            border_effect,
        } => M::Square {
            rect: rect(r, dx, dy),
            border,
            interior,
            border_width,
            border_effect,
        },
        M::Circle {
            rect: r,
            border,
            interior,
            border_width,
        } => M::Circle {
            rect: rect(r, dx, dy),
            border,
            interior,
            border_width,
        },
        M::Line {
            start,
            end,
            color,
            width,
            endings,
        } => M::Line {
            start: pt(start, dx, dy),
            end: pt(end, dx, dy),
            color,
            width,
            endings,
        },
        M::Ink {
            strokes,
            color,
            width,
        } => M::Ink {
            strokes: strokes.into_iter().map(|s| pts(s, dx, dy)).collect(),
            color,
            width,
        },
        M::Polygon {
            vertices,
            border,
            interior,
            width,
        } => M::Polygon {
            vertices: pts(vertices, dx, dy),
            border,
            interior,
            width,
        },
        M::Cloud {
            vertices,
            border,
            interior,
            width,
            intensity,
        } => M::Cloud {
            vertices: pts(vertices, dx, dy),
            border,
            interior,
            width,
            intensity,
        },
        M::PolyLine {
            vertices,
            color,
            width,
        } => M::PolyLine {
            vertices: pts(vertices, dx, dy),
            color,
            width,
        },
        // ★ A text markup's quads name GLYPHS on the page — the words a
        // highlight is over. Moving them would put a highlight over different
        // words, or over blank paper, which is not a copy of anything the
        // operator made. So a text markup pastes **in place**, and the offset
        // is ignored rather than applied.
        //
        // That is a deliberate exception to the "same page offsets" rule, and it
        // is the one case where landing on top of the original is correct: the
        // original is the only place this mark means anything.
        other @ M::TextMarkup { .. } => other,
        other => other,
    }
}

/// **The keys a `MarkupSpec` cannot carry**, read off the annotation being
/// copied.
///
/// # ★★★ Why this is a function rather than three lines at the call site
///
/// Because it is the *list* that matters and the list will grow. Every key here
/// is one this shell can now author and a spec cannot express, and each was
/// added to the authoring side on a different day by somebody who was not
/// thinking about the clipboard:
///
/// | key | authored since | what a lossy paste produced |
/// |---|---|---|
/// | `/CA` | 2026-08-28 | an opaque copy of a translucent mark |
/// | `/Contents` | 2026-08-28 | a comment with no words |
/// | `/T` | 2026-08-28 | a comment from nobody |
/// | `/M` | 2026-08-28 | a comment dated never |
///
/// A named function with this table on it is the thing a future author of a
/// fifth key will find. Three lines inside `copy` are not.
///
/// # ★★ `/T` and `/M` travel with `/Contents` and cannot travel without it
///
/// `MarkupNote` writes the three as a group, so an annotation with an author
/// and **no** note contributes nothing here — correctly: `pdfcer-core` refuses a
/// note whose text is absent, and a byline with no comment under it is not a
/// state this shell can author in the first place.
///
/// # ★ Absent is absent, never a default
///
/// `opacity: None` writes no `/CA` at all, which is not the same as `Some(1.0)`
/// in the bytes even though it is the same on screen — the engine's own rule,
/// and the reason a copy of an ordinary opaque mark produces byte-identical
/// output to what it always did.
fn carried_options(doc: &OpenDoc, page: usize, id: ObjId) -> pdfcer_core::edit::MarkupOptions {
    let graph = doc.session.graph();
    // ★★ `page_annotations`, not a hand-rolled read of four dictionary keys.
    //
    // `/Contents` and `/T` are PDF **text strings** (§7.9.2.2): PDFDocEncoding
    // or UTF-16BE with a byte-order mark, decided by the bytes themselves. A
    // shell decoding them by hand gets mojibake on every comment with an
    // accent, an em dash or a `Ø` — which `pdfcer-core` reported as a defect of
    // its OWN reader in August, so it is not a theoretical hazard.
    //
    // ⇒ The cost is a walk of the page's `/Annots` for one annotation, bounded
    // by `MAX_ANNOTS_PER_PAGE`, on a Ctrl+C. Paid deliberately: there is no
    // public verb that models ONE annotation dictionary, and the shell's own
    // Comments panel takes the same route for the same reason.
    let annot = doc
        .pages
        .get(page)
        .map(|p| pdfcer_core::annot::page_annotations(&graph, p.id))
        .unwrap_or_default()
        .into_iter()
        .find(|a| a.id == Some(id));
    let mut options = pdfcer_core::edit::MarkupOptions::default();
    let Some(annot) = annot else {
        // The annotation is on a page this shell has not modelled, or the
        // walk truncated. An empty options struct authors exactly what the
        // spec alone authored before this function existed, which is the
        // right degradation: a copy that loses the note is worse than a copy,
        // and a copy that fails outright is worse than both.
        return options;
    };
    options.opacity = annot.constant_alpha;
    if let Some(text) = annot.contents.clone() {
        let mut note = pdfcer_core::edit::MarkupNote::new(text);
        if let Some(author) = annot.title.clone() {
            note = note.by(author);
        }
        if let Some(modified) = annot.mod_date.clone() {
            note = note.at(modified);
        }
        options.note = Some(note);
    }
    options
}

#[cfg(test)]
mod tests {
    /// ★★★ **A cut that cannot delete must not copy either.**
    ///
    /// The fourth door onto `delete_annotation`, found by an adversarial review
    /// on 2026-08-29 after the other three had been gated the day before, and
    /// the worst of the four: on a certified document `Ctrl+X` copied the
    /// annotation, raised a Delete the engine then refused into a silent `Err`
    /// arm, and `annots::delete` cleared the selection anyway — leaving the
    /// operator with the markup still on the page, no selection, no
    /// explanation, **and a clipboard holding a copy of it**, so the next
    /// `Ctrl+V` duplicates the thing they were trying to move.
    ///
    /// # What this asserts, and why each half is needed
    ///
    /// 1. **`Err(DeleteRefused)`** — the whole gesture is refused, and it
    ///    carries the reason so the status row can say which of encryption,
    ///    certification or the `/F` Locked bit it was.
    /// 2. **No action was raised** — asserting only the `Err` would pass on a
    ///    build that refused *and* pushed the Delete anyway, which is the state
    ///    this fix exists to remove.
    /// 3. **Nothing reached the clipboard** — the half that makes it a *cut*
    ///    failure rather than a delete failure. A build that degraded the cut to
    ///    a copy would satisfy 1 and 2 and still hand the operator a duplicate.
    ///
    /// ★ `certified-comments.pdf` and `threaded-comments.pdf` differ in exactly
    /// one dictionary — the catalog's `/Perms` — so the pair tells *"withheld
    /// here"* from *"offered there"* while varying one thing. This test drives
    /// the refusing half; the offering half is the driven check's.
    #[test]
    fn a_cut_that_cannot_delete_does_not_copy_either() {
        use crate::canvas::selection::annot::{AnnotKind, AnnotSelection, AnnotTarget};

        let ctx = egui::Context::default();
        let mut doc = crate::app::state::open_local_fixture("certified-comments.pdf");
        let page = doc.pages.first().expect("the fixture has a page");
        let square = pdfcer_core::annot::page_annotations(&doc.session.graph(), page.id)
            .into_iter()
            .find(|a| a.subtype_label() == "Square")
            .expect("the fixture carries a /Square");
        let id = square.id.expect("an indirect annotation");
        doc.selection.select_annot(AnnotSelection {
            target: AnnotTarget {
                page: 0,
                id,
                kind: AnnotKind::Markup,
                subtype: "Square".to_owned(),
                locked: false,
            },
            outline: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(10.0, 10.0)),
        });

        let mut actions = Vec::new();
        let outcome = cut(&ctx, &doc, &mut actions);

        assert!(
            matches!(outcome, Err(Refusal::DeleteRefused(_))),
            "a certified document must refuse the whole gesture, got {outcome:?}"
        );
        assert!(
            actions.is_empty(),
            "the cut was refused and raised {} action(s) anyway — a refusal that \
             still pushes the delete is the state this gate exists to remove",
            actions.len()
        );
        assert!(
            read(&ctx).is_none(),
            "nothing was deleted and something reached the clipboard: the cut \
             degraded to a copy, so the next paste hands the operator a duplicate \
             of the markup they were trying to move"
        );
    }

    use super::*;

    /// The offset is applied on a same-page paste and not on a cross-page one.
    ///
    /// ★ Asserted as arithmetic rather than by driving, because the *decision*
    /// is the thing worth pinning: whether the copy is visible when it lands on
    /// top of its original is a property of this one comparison, and a driven
    /// check would prove it for one pair of pages.
    #[test]
    fn the_offset_is_same_page_only() {
        let same = if 3 == 3 { PASTE_OFFSET_PT } else { 0.0 };
        let across = if 3 == 7 { PASTE_OFFSET_PT } else { 0.0 };
        assert!(same > 0.0, "a copy on top of its original must be visible");
        assert!(
            across.abs() < f64::EPSILON,
            "a mark copied to another sheet belongs where it was on the first"
        );
    }

    /// ★ **Down the page is negative.** The one-line property that would
    /// otherwise ship inverted and never be reported, because a paste that
    /// drifts up-and-right looks like a decision rather than a bug.
    #[test]
    fn the_paste_moves_down_the_page() {
        let dy = -PASTE_OFFSET_PT;
        assert!(dy < 0.0, "PDF y increases upward, so down is negative");
    }
}
