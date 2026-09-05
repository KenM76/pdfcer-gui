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
//! **This table was written 2026-08-19 and every "blocked" row in it has since
//! expired. Corrected in place on 2026-09-05 rather than left standing beside
//! its correction, per R5.** The 2026-08-19 reading — *"markup can be copied
//! through `spec_from_dict`/`add_markup`, page content cannot be put back at
//! all"* — was true when taken and is kept only in this sentence, because its
//! shape is the lesson: **a capability claim about another crate is a dated
//! citation with a shelf life measured in hours.**
//!
//! Re-measured 2026-09-05 against engine **v0.38.0 (`b01964f`)**, from source:
//!
//! | subject | copy | paste | verdict |
//! |---|---|---|---|
//! | **page content** (paths, text runs, images) | `copy_objects` `edit.rs:10410` | `paste_objects` `edit.rs:11079` | ✅ shipped 2026-08-20 |
//! | **annotations** (any subtype but three) | `copy_annotations` `edit.rs:10432` | `paste_objects` — it plants both halves | ✅ **shipped here 2026-09-05** |
//! | **both at once** | `copy_selection` `edit.rs:10456` | the same `paste_objects` | ✅ wired; see the note below on why it cannot yet be *reached* |
//! | **text** (swept) | extraction | the clipboard is the destination | ✅ already shipped |
//! | **`/Widget`, `/Popup`, `/Redact`** | — | — | ⛔ the engine refuses all three **by name**, deliberately |
//!
//! ★ **What the annotation row changed, in the operator's terms.** Until
//! 2026-09-05 this module copied an annotation by reading a `MarkupSpec` out of
//! its dictionary and authoring a *new* annotation from it — so the eight
//! subtypes `pdfcer-core` models could be copied and **everything else could
//! not**. A sticky note, a stamp, a text box, a link and a file attachment all
//! answered `Ctrl+C` with *"that annotation is not one pdfcer authors."* A
//! sticky note is the most-copied comment in a review workflow.
//!
//! ★★★ **And the lossless route turned out to be lossy in the other
//! direction.** `copy_selection` carries a markup pdfcer *models* as a spec and
//! plants it with `add_markup` — not `add_markup_with` — so it drops `/CA`,
//! `/T`, `/M` and `/Contents` on exactly the kinds this module could already
//! copy faithfully. The fork that keeps both halves is
//! [`crate::canvas::annotclip`], whose header carries the whole account and the
//! `file:line` for every claim in it. **Read that before changing anything
//! here.**
//!
//! ## ★★ A mixed marquee — content AND annotations in one gesture
//!
//! The copy below is **one call to `copy_selection` with both index lists**,
//! which is the engine's own prescription: *"the verb a shell should call when
//! a marquee caught both — which on a marked-up drawing is the ordinary case,
//! not the exotic one."*
//!
//! ⇒ **It cannot be reached from the canvas today, and the reason is the
//! selection model rather than the clipboard.**
//! `canvas::selection::SelectionState` holds `annot: Option<AnnotSelection>`
//! and makes the two mutually exclusive by construction — *"One canvas, one
//! selection"* — so a marquee that sweeps a line and a revision cloud selects
//! the line and drops the cloud before `Ctrl+C` is ever pressed. Nothing in
//! this file can change that, and nothing in this file pretends otherwise: the
//! clipboard is the half that is ready. Recorded on the clipboard row of
//! `ENGINE_BACKLOG.md` and in [`crate::canvas::annotclip::selected`].
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
        /// gains a key.
        ///
        /// # ★★★ THE QUESTION IN THIS PARAGRAPH IS ANSWERED — 2026-09-05
        ///
        /// It said: *"`pdfcer-core`'s `copy_annotations` is the route that does
        /// not have that property … and moving to it is filed as a question
        /// rather than assumed, because it is not yet known whether a `/Popup`,
        /// an `/IRT` reply chain or an `/RC` rich-text body survive that path
        /// either."*
        ///
        /// It was read, not asked. The answers, from engine source at
        /// `b01964f`:
        ///
        /// * **`/RC` survives** — the raw carrier copies every key of the
        ///   dictionary (`edit.rs:10694`).
        /// * **`/Popup` and `/IRT` do NOT, deliberately** — both are on
        ///   `CLIP_STRIPPED_ANNOT_KEYS` (`edit.rs:10672`) because each names a
        ///   *relationship* in the source document: a popup reference without
        ///   its object is a dangling pointer, and a reply whose parent did not
        ///   travel is a thread with no root.
        /// * **and the route does not have the property the sentence credited
        ///   it with.** For a markup pdfcer *models* it carries a
        ///   `MarkupSpec` and plants it with `add_markup`, so it drops these
        ///   four keys exactly as a re-author does.
        ///
        /// ⇒ **So this variant did not become a legacy path.** It is now the
        /// route taken for precisely the annotations the engine models, chosen
        /// by reading the engine's own carrier off the clip — see
        /// [`crate::canvas::annotclip::Plan::spec_is_more_faithful`]. Deleting
        /// it would make every copied revision cloud anonymous, undated and
        /// opaque.
        options: Box<pdfcer_core::edit::MarkupOptions>,
    },
    /// ★★★ **A copied selection** — page content, annotations, or both, as one
    /// `ObjectClip`.
    ///
    /// This variant is what the type's own docs predicted: *"the day page
    /// content becomes pasteable this type is where that arrives."* `Pass 120.0`
    /// shipped `ObjectClip` on 2026-08-20 and this is that day.
    ///
    /// # ★★ It was called `Content` until 2026-09-05, and the rename is a
    /// correction rather than a tidy-up
    ///
    /// The name was accurate while the only thing a clip could hold was a page
    /// object. It stopped being accurate the moment this shell started routing
    /// annotations through `copy_selection`, and a variant called `Content`
    /// holding nothing but a sticky note is the sentence-describing-a-different-
    /// world shape `DEFECTS.md` D4a is about, one layer down in the source.
    /// `Selection` is what the engine calls it too, which removes one
    /// translation between the two vocabularies.
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
        /// ★★ Kept apart from `count` rather than summed, and the reason is
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
        /// ★ A delete is raised by `ObjId` through the funnel, and an
        /// `/Annots` position is not one. Re-deriving the ids from the indices
        /// after the clip was taken would be a second walk of the page that
        /// could disagree with the first — and the window between them is
        /// exactly where a cut deletes the wrong annotation.
        ///
        /// Empty for a pure-content clip and for a paste from another
        /// document, both of which are the same fact: nothing here is to be
        /// removed.
        annot_ids: Vec<ObjId>,
        /// ★★★ **What the copy could not carry**, as the `/Subtype`s the
        /// engine refused, verbatim.
        ///
        /// Carried on the clip rather than said at copy time and forgotten,
        /// because the caller that words it is `app::dispatch::clipboard` and
        /// this module words no decline — its own standing contract. A
        /// non-empty list means the operator selected something whose copy is
        /// *partial*, and rule 4's "fuzzy never sneaky" makes disclosing that
        /// mandatory: the paste will look complete.
        ///
        /// ★ Almost always empty today, because the only annotations the
        /// engine refuses — `/Widget`, `/Popup`, `/Redact` — are either routed
        /// elsewhere or refuse the whole copy. It becomes reachable the day a
        /// selection can hold more than one annotation, which is the same day
        /// the mixed marquee arrives.
        left_behind: Vec<String>,
        /// How many annotations on the clip travel as a `MarkupSpec` and will
        /// therefore arrive **without** `/CA`, `/T`, `/M` or `/Contents`.
        ///
        /// Zero for every clip this shell parks today: a lone modelled markup
        /// takes the spec-plus-options route instead, which carries all four.
        /// It becomes non-zero only when a modelled markup rides along beside
        /// content or another annotation, and it exists so that case discloses
        /// rather than silently thins the copy. See
        /// [`crate::canvas::annotclip::Plan::thin`].
        thin: usize,
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
///
/// ★★ **It stopped being `Copy` on 2026-09-05**, when [`Self::CannotCarry`]
/// arrived carrying the `/Subtype`s the engine refused. That is data taken off
/// the clip rather than a compile-time constant, and it has to be: the whole
/// point of the variant is to name *which* thing could not be copied, and a
/// `&'static str` would mean this shell keeping a third copy of a subtype
/// table `pdfcer-core` already owns. Every call site moves the value rather
/// than copying it, so nothing needed changing but the derive.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// ★★★ **The engine refuses to put that annotation on a clipboard at
    /// all**, and the `/Subtype`s it named travel with the refusal.
    ///
    /// `/Widget`, `/Popup` and `/Redact` — `EditSession::raw_copy_refusal`
    /// (`edit.rs:10689`), and each for a stated reason rather than because it
    /// is hard: a widget would need a field name in the destination's
    /// `/AcroForm` that pdfcer cannot guess, a popup is not an independent
    /// annotation (§12.5.6.14) and belongs to the comment that opens it, and a
    /// redaction is a **pending destructive operation** — pasting one arms a
    /// redaction in a document nobody reviewed.
    ///
    /// ★ The list is read off the clip, never mirrored here. `canvas::cutgate`
    /// does keep a mirror of the same three, and its own header explains why
    /// that one has to exist — it greys a control *before* the gesture, where
    /// nothing but a compile-time string will do. This is after the gesture,
    /// and the engine has already answered.
    CannotCarry(Vec<String>),
    /// **The selected annotation is no longer on the page it names.**
    ///
    /// # ★★ This variant's meaning CHANGED on 2026-09-05, and its old sentence
    /// was about to become false
    ///
    /// It used to mean *"the dictionary would not yield a `MarkupSpec`"*, and
    /// its sentence said so in the operator's terms: *"That annotation is not
    /// one pdfcer authors — a link, a form field or an attachment — so there is
    /// nothing for it to copy."* Every word was true of the re-authoring
    /// clipboard.
    ///
    /// It is false now. A link copies. An attachment copies. A sticky note, a
    /// stamp and a text box copy, with their baked appearances — that is the
    /// whole point of routing through `copy_selection`. Leaving the sentence up
    /// would have been the fourth *"a limit reported as an absence"* this
    /// project has paid for, and `RESUME.md` records what those cost.
    ///
    /// So the variant keeps its name and takes the one job left over: the
    /// selected `ObjId` is not among the page's annotations. Reachable after an
    /// undo, after an external reload, or on a page whose `/Annots` walk
    /// truncated. It is a **stale selection**, not an unsupported kind, and the
    /// operator's next move — click it again — is completely different.
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

/// **Copy what is selected** — page content, an annotation, or both — and say
/// what reached the clipboard.
///
/// # ★★★ ONE ENGINE CALL, TWO ADDRESS SPACES
///
/// `EditSession::copy_selection` (`edit.rs:10456`) takes an object-index list
/// **and** an annotation-index list, and its own doc comment says why they
/// cannot be one list: *"an annotation is not content, so it has no paint-order
/// index."* This function is the only place in the shell that fills both, and
/// it fills them in **one call** rather than two, so that:
///
/// * a mixed selection is one clip, one paste and one gesture — the day the
///   selection model can hold one (see the header);
/// * neither list can be built from the other's numbering, which is the mistake
///   the two-argument signature exists to make impossible.
///
/// # ★★★ AND THEN IT ASKS THE ENGINE WHAT IT DID
///
/// The clip comes back carrying, per annotation, the **carrier** the engine
/// chose. For a markup pdfcer models that carrier is a `MarkupSpec`, which
/// cannot express `/CA`, `/T`, `/M` or `/Contents` — so for the one shape where
/// this shell's older spec-plus-options route is strictly more faithful (a
/// single modelled markup, alone), the clip is thrown away and that route is
/// taken instead. [`crate::canvas::annotclip::Plan`] holds the whole argument
/// and the falsifying tests.
///
/// ⇒ **The fork is read off the payload, never off a subtype list here.** A
/// list would be a fourth copy of a taxonomy `pdfcer-core` owns, and would be
/// wrong and silent the first time the engine modelled a ninth subtype — which
/// is the exact defect shape this whole route was built to remove.
///
/// # Errors
///
/// Every member of [`Refusal`] except [`Refusal::NothingCopied`], which only a
/// paste can raise.
pub fn copy(ctx: &egui::Context, doc: &OpenDoc) -> Result<Clipped, Refusal> {
    use crate::canvas::annotclip;

    // ★ The annotation's OWN page wins where there is one. A selected
    // annotation and `view.page_index` can differ — the view can be scrolled
    // onto the next sheet with a comment still selected on this one — and the
    // clip must describe the page the thing is actually on, because that is the
    // page `copy_selection` will index into.
    let annots = annotclip::selected(doc)?;
    let page = annots.first().map_or(doc.view.page_index, |a| a.page);
    let objects = doc.selection.object_indices_on(page);
    if objects.is_empty() && annots.is_empty() {
        return Err(Refusal::NothingSelected);
    }
    let indices: Vec<usize> = annots.iter().map(|a| a.index).collect();
    // ★ `&self`, and it commits nothing — which is what makes `cut` below one
    // undo entry without a `cut_selection` call: only the deletion is an edit.
    //
    // ★★ `copy_selection`, not `copy_objects` and not `copy_annotations`. The
    // two narrow verbs are wrappers over this one body (`edit.rs:10415`,
    // `edit.rs:10437`), so calling it directly costs nothing and removes the
    // branch that would otherwise have to decide which wrapper to use — a
    // branch whose wrong answer is a silently half-copied selection.
    let clip = doc
        .session
        .copy_selection(page, &objects, &indices)
        .map_err(|_| Refusal::EngineRefused)?;
    let plan = annotclip::Plan::of(&clip);

    if plan.nothing_to_carry(objects.len()) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "clipboard-copy-refused reason=cannot-carry what={:?}",
                plan.refused
            )
        });
        return Err(Refusal::CannotCarry(plan.refused));
    }

    if plan.spec_is_more_faithful(objects.len()) {
        // ★★★ THE ENGINE MODELLED IT, SO THE OLDER ROUTE IS THE FAITHFUL ONE.
        //
        // Not a fallback and not a legacy path: it is the branch taken for
        // every revision cloud, arrow, ink stroke and callout this operator
        // draws, because those are exactly the kinds `spec_from_dict` reads.
        // Taking the clip here instead would compile, pass a "the paste
        // happened" test, and hand him an anonymous, undated, opaque copy of a
        // signed comment.
        let selected = annots
            .first()
            // ui-text-exempt: a panic message for a developer. The condition is
            // `Plan::spec_is_more_faithful`, which is `thin == 1` with nothing
            // else on the clip, and `thin` counts annotations the shell asked
            // for by index — so an empty `annots` here would mean the engine
            // classified a markup that was never requested. Unreachable, and
            // named rather than silenced with a `?` that would degrade the copy
            // to a lossy one on a state that cannot happen.
            .expect("spec_is_more_faithful requires exactly one annotation");
        return copy_as_spec(ctx, doc, selected);
    }

    // ★★ The clip's OWN bbox, unioned by the engine over both content items and
    // annotation `/Rect`s (`edit.rs:10569`), converted to a centre.
    //
    // This replaced a walk of `CanvasTargetProvider::bounds` on 2026-09-05, and
    // the change is a correction rather than a simplification: the provider
    // answers for **content objects only**, so a clip whose only member was an
    // annotation got `anchor: None` and silently fell back to the offset rule —
    // the operator's O73 complaint, reproduced for the new payload on the day
    // it shipped. The clip's bbox is also the representation that survives the
    // source document being closed, which is what makes "copy, close, paste
    // where I point" work at all.
    let anchor = (!clip.bbox().is_empty()).then(|| {
        let b = clip.bbox();
        ((b.min.x + b.max.x) / 2.0, (b.min.y + b.max.y) / 2.0)
    });
    let clipped = Clipped::Selection {
        count: clip.len(),
        annotations: plan.carried(),
        annot_ids: annots.iter().map(|a| a.id).collect(),
        left_behind: plan.refused.clone(),
        thin: plan.thin,
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
    //
    // ★★ An ANNOTATION-only clip reaches `publish` with `clip.items` empty, so
    // the raster is degenerate and the picture declines — by the same path a
    // zero-area content clip already took, with no new branch. That is the
    // right outcome rather than a gap: `clipimage` renders the page content a
    // clip carries, and a comment's appearance is not page content. The
    // operator still gets the marker, so `Ctrl+V` still arrives.
    let marker = crate::text::clipboard::os_marker(objects.len(), plan.carried());
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
        //
        // ★★ And `annots=` and `thin=` beside them, because a wrong build gets
        // THOSE wrong: `annots=0` on a copy of a sticky note is the whole
        // failure, and `thin=1` is the build that took the engine's model
        // carrier where the faithful route was available. Neither is visible in
        // `objects=` or in `bytes=`.
        let bytes = match &clipped {
            Clipped::Selection { bytes, .. } => bytes.len(),
            Clipped::Markup { .. }
            | Clipped::FormField(_)
            | Clipped::Pages { .. }
            | Clipped::Outline { .. }
            | Clipped::Attachment(_) => 0,
        };
        format!(
            "clipboard-copy kind=selection page={page} objects={} annots={} thin={} \
             left_behind={} bytes={bytes}",
            objects.len(),
            plan.carried(),
            plan.thin,
            plan.refused.len(),
        )
    });
    Ok(clipped)
}

/// **Copy one modelled markup through its `MarkupSpec` and the four keys a
/// spec cannot carry.**
///
/// The older of the two annotation routes and, for the annotations
/// `pdfcer-core` models, still the faithful one — see
/// [`crate::canvas::annotclip::carried_options`] for the four keys and
/// [`crate::canvas::annotclip::Plan::spec_is_more_faithful`] for when this is
/// reached.
///
/// # Errors
///
/// [`Refusal::Unreadable`] when the dictionary is gone from under the
/// selection between [`crate::canvas::annotclip::selected`] resolving it and
/// this reading it — a window of one function call, and closed by name rather
/// than by an `expect`.
fn copy_as_spec(
    ctx: &egui::Context,
    doc: &OpenDoc,
    selected: &crate::canvas::annotclip::Selected,
) -> Result<Clipped, Refusal> {
    use pdfcer_core::annot_author::spec_from_dict;
    use pdfcer_core::object::Object;

    let graph = doc.session.graph();
    let Some(Object::Dict(dict)) = doc.session.value(selected.id) else {
        return Err(Refusal::Unreadable);
    };
    let spec = spec_from_dict(&graph, dict).map_err(|_| Refusal::Unreadable)?;
    // ★★ The keys the spec cannot carry, read from the SAME dictionary the spec
    // came from — one read, so the two halves of the copy cannot describe
    // different annotations.
    let options = Box::new(crate::canvas::annotclip::carried_options(
        doc,
        selected.page,
        selected.id,
    ));
    // ★ The `/Rect` centre, from the SAME dictionary the spec came from — one
    // read, so the anchor and the geometry cannot describe different
    // annotations. `OPERATOR_REQUESTS.md` O73; see `Clipped::Markup::anchor`.
    let anchor = crate::canvas::annotclip::rect_centre_of(dict);
    let clipped = Clipped::Markup {
        spec: Box::new(spec),
        page: selected.page,
        anchor,
        options,
    };
    store(ctx, clipped.clone());
    // ★★ The OS marker goes on here too, and its absence was a real defect
    // once: `egui-winit` raises `Event::Paste` only when the OS clipboard holds
    // non-empty text, so a copy route that skipped this leaves `Ctrl+V` working
    // or not depending on what the operator last copied in another program.
    // `RESUME.md` records the form-field copy shipping with exactly that gap,
    // one function away from the comment explaining it.
    ctx.copy_text(crate::text::clipboard::os_marker(0, 1));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // ★ `carrier=spec` is the word that makes the fork observable. Without
        // it a build that took the engine's model carrier — losing the author,
        // the date, the note and the opacity — traces identically to one that
        // took this route, and the difference is invisible on the page.
        format!(
            "clipboard-copy kind=markup carrier=spec page={} annots=1",
            selected.page
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
        // ★★★ **BOTH HALVES OF A SELECTION CLIP**, as of 2026-09-05, and they
        // are two actions rather than one because they address two different
        // things: page content by paint-order index into a content stream, an
        // annotation by `ObjId` in the page's `/Annots`. Nothing in this shell
        // — or in `EditSession` — takes both in one call.
        //
        // ★★ SO A MIXED CUT IS TWO UNDO ENTRIES, and that is stated rather
        // than hidden. It is not reachable today (the selection model holds
        // content or an annotation, never both), and when it becomes reachable
        // the honest fix is a `cut_selection` on the engine side rather than a
        // second delete funnel here — filed on the clipboard row of
        // `ENGINE_BACKLOG.md`. A pure-content cut and a pure-annotation cut are
        // each ONE entry, which is every cut an operator can make today.
        //
        // ★ The ids come off the CLIP, not from a fresh read of the selection.
        // Re-deriving them here would be a second walk of the page between the
        // copy and the delete, and the window between two walks is exactly
        // where a cut removes the annotation next to the one it copied.
        (
            Clipped::Selection {
                page, annot_ids, ..
            },
            _,
        ) => {
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
            for id in annot_ids {
                actions.push(Action::Annot(
                    crate::app::actions::annot::AnnotAction::Delete {
                        page: *page,
                        id: *id,
                    },
                ));
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
        // ★ A clip takes its own path: it is bytes and the verb is
        // `paste_objects`, which takes a page-space MATRIX rather than a
        // displacement — so the offset below cannot be shared even though the
        // rule that decides it is.
        //
        // ★★ ONE verb plants BOTH halves. `paste_objects` commits the content
        // command and then calls the private `paste_clip_annotations`
        // (`edit.rs:10901`), so this shell raises **one** action for a mixed
        // clip and does not have to sequence two. The engine's own note is that
        // the annotation half lands as **its own undo entry**, which is
        // disclosed here rather than discovered: `Ctrl+Z` after a mixed paste
        // takes back the comments first and the geometry second. Not reachable
        // today; filed on the clipboard row of `ENGINE_BACKLOG.md`.
        Some(Clipped::Selection {
            bytes,
            page: from,
            count,
            annotations,
            anchor,
            ..
        }) => {
            return paste_clip(
                page,
                &bytes,
                from,
                count,
                annotations,
                anchor,
                target,
                actions,
            );
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
        spec: Box::new(crate::canvas::annotclip::translated(*spec, dx, dy)),
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

/// Paste a clip onto `page`, raising the action that authors it — **page
/// content, annotations, or both**.
///
/// ★ Renamed from `paste_content` on 2026-09-05, when the clip stopped being
/// content-only. The old name would have been the same kind of falsehood the
/// `Clipped::Content` variant's rename removes: a function called
/// `paste_content` that plants a sticky note.
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
            // ★★ `annots=` is on this line because a wrong build gets it wrong
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

#[cfg(test)]
mod tests;
