//! # `app::status::decline` — the worded decline: saying that a command did
//! *not* run
//!
//! `FEATURES.md`'s Phase 3 row read *"traced and greyed but never worded"*.
//! `crate::canvas::zoom` has always returned
//! [`ZoomOutcome::NoBounds`]/[`ZoomOutcome::NoCanvas`] and always traced them,
//! and `crate::app::dispatch` has always dropped the value on the floor. This
//! module is the half that was missing: a store, a retirement rule, and one
//! line in the status bar's left half.
//!
//! ## ★ The distinction this module exists to hold: a decline is not a
//! disclosure
//!
//! The bar's left half already carries two rule-4 sentences
//! ([`super::fill_disclosure`], [`super::edit_disclosure`]) and they are a
//! different **speech act** from this one:
//!
//! | | says | is true because |
//! |---|---|---|
//! | disclosure | *this happened, and here is the part you cannot see* | a document changed |
//! | decline | *this did not happen* | a document did **not** change |
//!
//! They share the *place* and the *discipline* — the same
//! [`super::disclosure_line`], the same named-region publication, the same
//! R128 fixed row — and they share nothing else. In particular they must not
//! share a store, and the wording must diverge too: *"Nothing to zoom to"* is
//! not *"About your last edit: …"*. One slot and one wording for both would
//! make a completed gesture and a refused one wear the same sentence in the
//! same place, which is **worse than the trace-only state this replaces**.
//!
//! ## ★★ Why this is NOT keyed on [`crate::app::state::OpenDoc::edit_epoch`]
//!
//! The epoch key is what makes the two disclosures safe, and it is exactly
//! what would make this one wrong. Three independent reasons, any one of them
//! sufficient:
//!
//! 1. **A decline changes no document, so the epoch never moves.** The
//!    disclosures retire because the *next edit* bumps the epoch past them,
//!    with no code remembering to clear anything. A decline produces no edit,
//!    so an epoch-keyed decline would never retire — it would still read
//!    "Nothing to zoom to" forty gestures later, which is the precise inverse
//!    of the property that makes the edit disclosure safe.
//! 2. **A decline must be repeatable.** Pressing the chord twice with nothing
//!    selected is **two events**, and the operator needs the second to
//!    register. An epoch key cannot express a repeat, because by construction
//!    nothing changed between the two — the key is identical, so the second
//!    press is indistinguishable from the first never having been retired.
//!    `crate::canvas::zoom::trace_outcome` makes the same ruling on the trace
//!    channel and states it in the same words: *"two identical zoom commands
//!    are two events, and a gate that silenced the second would make a harness
//!    unable to tell a command that ran twice from one that ran once."*
//! 3. **They are different speech acts** — see the table above.
//!
//! ## ★ The precedent that IS right: `page_box`'s clamp note
//!
//! [`super::page_box`] already has a note that is retired **by the operator's
//! next act** rather than by an epoch. Its rule is *"the note is true while
//! you are still where it put you"*, and its test is
//! `page_box::tests::a_clamp_note_is_forgotten_once_the_operator_moves_away`.
//! A decline is that shape, and this module is modelled on it. Two halves,
//! and both are needed:
//!
//! - **[`retire`] at the dispatcher.** `crate::app::dispatch` is the one
//!   choke point every command arrives at, which makes it the one place that
//!   knows *"the operator has just invoked something"*. A decline is retired
//!   there, before the arm for the new command runs — so pressing Ctrl+F, or
//!   clicking Fit page, ends the sentence, and re-pressing the zoom chord ends
//!   it and then raises it again, which is reason 2 above made mechanical.
//! - **[`live`]'s still-true filter at the bar.** Not every act is a command:
//!   *selecting something* is a canvas gesture and reaches no dispatcher. So
//!   the bar draws the sentence only while the reason that produced it is
//!   still true, asked through **the same predicate that produced it**
//!   ([`zoom::can_zoom_to_selection`], [`zoom::last_frame`]) rather than
//!   through a second spelling that could drift. A decline can therefore never
//!   become a lie, only stale — and the dispatcher handles stale.
//!
//! The filter is a *filter* rather than a clear, exactly as
//! [`crate::app::actions::last_edit_disclosure`]'s epoch comparison is: state
//! that must be cleared is state that will one day be shown against the wrong
//! document.
//!
//! ## ★ What this module deliberately does NOT word
//!
//! **The raster-ceiling-clamped region zoom is not a decline — it is a partial
//! grant.** [`ZoomOutcome::Zoomed`] carries both the scale that was asked for
//! and the scale that was pinned, and
//! [`ZoomOutcome::ceiling_changed_the_answer`] reports when they differ. It is
//! tempting to word that here. It would be wrong:
//!
//! - the region **is** framed, centred, at the closest scale the page can go
//!   to — the operator got the honest partial answer, not a refusal;
//! - the clamp **already reports itself**, and does so in the one place an
//!   operator is already looking for a scale: the framing verb raises
//!   `Action::ZoomTo` carrying the *clamped* number, so the zoom readout three
//!   controls to the right states the truth on the same frame.
//!
//! Wording it would word a non-event, and would train the operator to read a
//! decline line that fires when nothing was declined — which is how a surface
//! stops being read. The decision is recorded beside
//! [`ZoomOutcome::ceiling_changed_the_answer`] as well, because that is where
//! the next reader will look.
//!
//! ## Why the store is a thread-local
//!
//! The same answer, for the same reason, as
//! [`crate::app::actions::last_edit_disclosure`]'s `LAST_EDIT` and
//! `crate::panels::forms::edit`'s `LAST_FILL`: it *should* be a field on
//! `OpenDoc`, and `crate::app::state` is not this work's to extend — a
//! **territory boundary rather than a design judgement**, stated here so
//! whoever lifts it knows what the preferred shape is.
//!
//! It is nonetheless sound, and rather more obviously so than its two
//! neighbours: this is not document state at all. It records that a command
//! declined; it cannot change a pixel of the page; nothing reads it except a
//! bar deciding whether to draw a sentence; and `eframe`'s update loop is one
//! thread, so the writer and the reader are the same thread while a test on
//! another thread gets its own empty slot rather than another test's
//! leftovers.
//!
//! One thing it does **not** need that its neighbours do: a document
//! identity. A decline that outlived a document close would be filtered out on
//! the next frame anyway, because a freshly-opened document has drawn no page
//! and has nothing selected — which makes the sentence *true* rather than
//! stale — and the first command the operator invokes retires it.

use std::cell::RefCell;

use crate::app::state::OpenDoc;
use crate::canvas::zoom::{self, ZoomOutcome};
use crate::text::status as t;

/// Named region: the worded decline, when one is live.
///
/// Named for the same reason its two disclosure siblings are: the whole
/// requirement of a decline is that it is **on screen and legible**, and
/// `ui-verify` can only assert that about a rect the application published.
/// Matched literally by `tools/ui-verify`, so renaming it silently un-aims
/// whatever check was measuring it.
const REGION_DECLINE: &str = "status-group:decline"; // ui-text-exempt: trace region name, never displayed

// ---------------------------------------------------------------------------
// What was declined
// ---------------------------------------------------------------------------

/// A framing zoom that did not happen, and why.
///
/// A *narrower* type than [`ZoomOutcome`] on purpose: that enum's third
/// variant is a zoom that **did** happen (possibly clamped, which is a partial
/// grant and not a decline — see the module docs), and a store that could hold
/// it would be a store a future edit could word. This one cannot represent a
/// grant at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Declined {
    /// Nothing on the page resolved to a box to frame.
    ///
    /// [`ZoomOutcome::NoBounds`], which `crate::canvas::zoom` raises for three
    /// situations it rules are one from the operator's side — nothing
    /// selected, the selection on another page, or a selection that no longer
    /// resolves after an edit.
    NothingToFrame,
    /// The canvas had not drawn a page, so there was no viewport to frame
    /// anything into. [`ZoomOutcome::NoCanvas`].
    CanvasNotDrawn,
    /// **`file.save_copy` was given a destination and produced no file.**
    ///
    /// The first decline here that is not about zoom, which is why this enum's
    /// two constructors are now [`Declined::of`] (zoom's) and
    /// [`record_save_failure`] (the save's) rather than one. It joins rather
    /// than getting a store of its own for the reason the module header gives
    /// about the two disclosures: a second mechanism beside this one would give
    /// the bar two ways to learn the same *kind* of thing, and the second would
    /// be the one that forgot to retire itself.
    ///
    /// # ★ It is retired by the operator's next act, and by nothing else
    ///
    /// [`Self::still_true`] answers `true` for it unconditionally, and that is a
    /// decision rather than a gap. Its two neighbours have a live predicate to
    /// re-ask — *is something framable now?*, *has the page drawn?* — because
    /// their reasons can stop being true on their own, with the operator doing
    /// nothing. A failed write has no such state: the folder is not going to
    /// reappear on the next frame, and if it did, the sentence would still be a
    /// true report of what happened when the operator pressed Save.
    ///
    /// So it lives exactly as long as [`retire`] lets it — until the next
    /// command — which is `super::page_box`'s clamp-note rule and the one the
    /// module header names as the right precedent. Pressing `Ctrl+S` again
    /// retires it and then records it again, so two failed saves are two events.
    ///
    /// The engine's own reason is **not** carried. It goes to the trace, from
    /// `crate::app::save`; see [`crate::text::status::save_copy_failed`] for why
    /// a `Display` impl's prose is not operator copy.
    SaveFailed,
    /// **A verb was asked to act on something drawn inside a form XObject.**
    ///
    /// # ★ The two states this keeps apart, and why merging them is expensive
    ///
    /// *"Nothing is selected"* and *"the thing you selected cannot be reached
    /// by this verb"* are the operator's mistake and the program's limit. An
    /// interface that reports the second as the first sends them looking for
    /// something they did not do wrong — and here it would contradict the
    /// outline they can see on screen, which is the most confusing shape a
    /// message can have.
    ///
    /// This became reachable on 2026-08-27, when a click started reaching
    /// inside form XObjects. Before that a form-interior object could not be
    /// selected at all, so no verb could be asked about one; now it can be
    /// selected, measured, described and copied *nowhere*, because
    /// `EditSession`'s paint-order verbs write to the **page's** content stream
    /// and a leaf's tokens index the **form's**.
    ///
    /// # Retired by the operator's next act, and by the selection changing
    ///
    /// Unlike [`Self::SaveFailed`], this one has a live predicate worth
    /// re-asking: the moment the operator selects something else, the sentence
    /// stops being about what they are looking at. [`Self::still_true`]
    /// therefore consults `selection_in_form`, gathered from the same
    /// accessor `crate::app::conditions` publishes `selection.in_form` from —
    /// so the greyed control and the sentence in the bar cannot come from
    /// different questions.
    InsideForm,
    /// **A restyle of existing text did not happen** — O37, 2026-08-27.
    ///
    /// The payload is [`crate::text::status::TextStyleRefusal`], which is the
    /// shell's own reading of which refusals an operator can act on. It is a
    /// `Copy` enum rather than a `String` so this whole type stays `Copy` and
    /// [`Declined::line`] stays `&'static str`; the reasoning for keeping the
    /// engine's own prose OFF the status bar is on that enum.
    ///
    /// # ★ It is retired by the operator's next act, and by nothing else
    ///
    /// [`Self::still_true`] answers `true` for it unconditionally, in the same
    /// ruling as `SaveFailed` and the two adopt refusals: a page is not going
    /// to grow a Bold face, and a run is not going to become pinnable, between
    /// one frame and the next. The remedy is always something the operator
    /// *does* — pick a different face, select different text, press the button
    /// again — and every one of those is a command, which [`retire`] catches.
    ///
    /// ★★ It is deliberately NOT keyed on the selection the way `InsideForm`
    /// is. `InsideForm`'s sentence sends the operator to select something else,
    /// so selecting something else is the remedy completing and the sentence
    /// must go. These sentences send the operator to press a *different
    /// control* on the *same* selection — so retiring on a selection change
    /// would be right, and retiring on it alone would let the sentence vanish
    /// while the operator was reading it.
    TextStyle(crate::text::status::TextStyleRefusal),
    /// **A rotation did not happen** — the ninth handle, 2026-08-28.
    ///
    /// The payload is [`crate::text::rotating::RotateRefusal`], modelled on
    /// [`Self::TextStyle`]'s and for the identical reasons: a `Copy` enum keeps
    /// this type `Copy` and [`Declined::line`] `&'static str`, and it keeps the
    /// engine's own prose off the status bar.
    ///
    /// # ★★★ Why this variant exists for a gesture that should never refuse
    ///
    /// Because **this project's founding defect shape is a grip that is
    /// dragged, released, and does nothing with no explanation** — and a rotate
    /// handle is the newest grip on the canvas. Two of the four refusals it can
    /// carry describe routing failures that cannot happen while the routing
    /// holds (`canvas::rotating` matches on `AnnotKind`, and a widget is never
    /// an annotation selection at all). That is the argument for wording them,
    /// not against: a routing bug with a sentence is a bug report, and one
    /// without is a handle that does nothing.
    ///
    /// ★ The genuinely reachable case is a **certified** document, and it is
    /// the one an operator cannot possibly guess at — a signed drawing looks
    /// exactly like an unsigned one on the canvas.
    ///
    /// # ★★ Recorded from three places, which is unusual and is correct
    ///
    /// `canvas::rotating` records [`RotateRefusal::NoDimensionRecord`] before
    /// any verb is called, because that condition is a **query** the shell can
    /// answer itself — the same placement `record_flatten_certified` uses.
    /// `app::actions::annots::{rotate, rotate_dimension}` record the other
    /// three from **inside** the `vector_edit` closure, because whether the
    /// engine will refuse is not knowable before the call — the same placement
    /// `record_resize_not_rebuildable` uses, and for the reason its own docs
    /// give.
    ///
    /// # Retired by the operator's next act
    ///
    /// [`Self::still_true`] answers `true` unconditionally, with `TextStyle`
    /// and the others whose state cannot change between two frames: a document
    /// does not stop being signed, and a sidecar does not grow a record, while
    /// the operator reads the status bar. What retires it is their next
    /// command, which [`retire`] catches.
    Rotate(crate::text::rotating::RotateRefusal),
    /// **"Give this page its own copy" did not happen** — the form-XObject
    /// verb, 2026-08-28.
    ///
    /// The payload is [`crate::text::unshare::UnshareRefusal`], modelled on
    /// [`Self::TextStyle`]'s and [`Self::Rotate`]'s and for the identical
    /// reasons: a `Copy` enum keeps this type `Copy` and [`Declined::line`]
    /// `&'static str`, and it keeps the engine's own diagnostic prose off the
    /// status bar.
    ///
    /// # ★★★ Why this refusal matters more than any other in this enum
    ///
    /// Because **a successful unshare and a refused one look identical on the
    /// canvas**, and no other decline here has that property: the copy is
    /// byte-identical until it is edited, so the page renders pixel-for-pixel
    /// the same either way. Silence therefore does not read as *"nothing
    /// happened"* — it reads as *"it worked"*, and the operator's very next act
    /// is to edit the drawing they believe they have just privatised. On this
    /// operator's documents that is a title block shared by thirty-six sheets.
    /// `crate::text::unshare`'s header carries the full account and is why every
    /// one of the verb's refusals is worded where `resize` words one of six.
    ///
    /// ★★ Recorded from three positions, unlike [`Self::Rotate`]'s two — see
    /// [`record_unshare`] for the split — and retired by the operator's next
    /// act: [`Self::still_true`] answers `true` unconditionally, deliberately
    /// **not** on `selection_in_form` the way [`Self::InsideForm`] does.
    Unshare(crate::text::unshare::UnshareRefusal),
    /// **The Settings window's Save wrote nothing.**
    ///
    /// # ★ Why this is not [`Self::SaveFailed`], although both are failed writes
    ///
    /// Because the two sentences have to say opposite things about what
    /// happened to the operator's work.
    ///
    /// A failed `file.save_copy` produced **no file**: the operator asked for
    /// something and got nothing, and there is no partial state to explain.
    ///
    /// A failed settings save is the opposite shape. The application **adopted
    /// the configuration anyway** — deliberately, because the operator asked
    /// for it and a disk that refuses should not cost them a choice they made —
    /// so what is true is *"this is in force now and will be gone when pdfcer
    /// restarts"*. Reusing the save-a-copy sentence would tell them their
    /// choice did not take, which is false, and they would make it again.
    ///
    /// Sharing a variant would also make the two indistinguishable in the one
    /// place it matters: an operator who pressed Save in two different windows
    /// in one minute.
    ///
    /// # Retired by the operator's next command, like its neighbour
    ///
    /// [`Self::still_true`] answers `true` unconditionally, for the same reason
    /// `SaveFailed` does: the folder is not going to become writable on the
    /// next frame, and if it did, the sentence would still be a true report of
    /// what happened when Save was pressed.
    ///
    /// The engine's own reason is **not** carried — it goes to the trace from
    /// `crate::app::settings_window`, which is where the store location is also
    /// recorded. A `Display` impl's prose is not operator copy.
    SettingsNotSaved,
    /// **`edit.undo` was invoked with an empty command log.**
    ///
    /// # ★ Why this is worded when the control that raises it is greyed
    ///
    /// Because the control is not the only route, and the other route is the
    /// one where greying explains nothing. `edit.undo` is gated on
    /// `undo.available`, so the quick-access button is un-pressable with an
    /// empty log — and it is *also* bound to `Ctrl+Z`, and
    /// [`crate::app::modes::capability::offers_command`] lets it through in
    /// **every** mode because it sits on no tab. So the reachable case is a
    /// chord, fired by an operator whose eyes are on the page rather than on an
    /// 18 pt icon in the title bar.
    ///
    /// That is [`Self::NothingToFrame`]'s *"reached by a chord"* argument with
    /// the reflexes of a whole industry behind it: `Ctrl+Z` is the keystroke an
    /// operator presses without deciding to, and answering the commonest
    /// keystroke in editing with nothing at all is the exact "the button does
    /// nothing" state this project was founded on.
    ///
    /// # It has a live predicate, and it is the one that produced it
    ///
    /// [`Self::still_true`] re-asks `EditSession::can_undo` — the same question
    /// `PdfcerApp::conditions` publishes `undo.available` from and the same one
    /// the apply arm declined on. So authoring anything at all retires the
    /// sentence on the next frame, without the operator invoking a command,
    /// which is [`Self::NothingToFrame`]'s shape exactly: *the remedy happened,
    /// so the sentence is history.*
    /// **A widget could not be registered: another field already has the name.**
    ///
    /// `EditError::FieldNameTaken`, raised by `EditSession::adopt_widget`.
    ///
    /// # ★ Why this is refused rather than auto-renamed, which is the engine's
    /// ruling and this shell agrees with it
    ///
    /// ISO 32000-2 SS12.7.3.1 makes the **fully qualified name the field's
    /// identity**. Two top-level fields called `Address` are not two fields —
    /// they are *one field with two widgets*, so typing in either fills both.
    /// No viewer reports this. The operator discovers it by typing into one box
    /// and watching another change, which is the worst possible way to learn it.
    ///
    /// `pageops::assemble` auto-renames on merge because it has nobody to ask.
    /// This surface **does** have somebody to ask, and the engine put it
    /// plainly: *"`Address_2` is a name nobody chose."* So the edit declines and
    /// the operator retypes, with what they typed still in the box in front of
    /// them.
    ///
    /// The clashing name is **not** carried into the sentence. It is a `Copy`
    /// enum, and more to the point the name is already on screen — the operator
    /// typed it seconds ago and it is still in the field they typed it into.
    FieldNameTaken,
    /// **A widget could not be registered because it carries no name of its
    /// own, and none was supplied.**
    ///
    /// `EditError::WidgetHasNoFieldIdentity`.
    ///
    /// # ★ What this actually means, and why the sentence must not say
    /// "recovered"
    ///
    /// It is a **bare kid**: a widget whose `/Parent` pointed at its field, in a
    /// document where that `/Parent` is gone. The engine measured a real form
    /// and found 2 of 13 in this shape after an insert, and named its own cause
    /// — `insert_pages` drops `/Parent` from every dictionary it copies, which
    /// is correct for a page and destroys a widget's only link to its identity.
    ///
    /// What was lost is not just the name. It was the name **and** the field
    /// type, the radio flags and the value. Nothing in this document holds any
    /// of it, so a name typed here **creates a new field**; it does not recover
    /// the old one. The sentence says so, because an operator told they had
    /// "restored" a radio button would go looking for its group.
    WidgetHasNoName,
    /// ★★★ **A resize was refused because the appearance cannot be rebuilt**
    /// (`OPERATOR_REQUESTS.md` O51, engine `Pass 151.0`).
    ///
    /// The one decline in this enum that names a **remedy the operator can
    /// reach in one click**, and it exists because of a fact about the format
    /// rather than about pdfcer.
    ///
    /// An annotation's artwork is placed through §12.5.5's matrix, which a
    /// resize makes a scale, and that matrix is applied *after* stroking. So
    /// the drawn stroke scales with it whatever `/BS /W` says — and **no
    /// per-axis stroke width exists** in PDF or in SVG. Two states are
    /// therefore unsatisfiable when pdfcer did not author the appearance:
    ///
    /// | scale | *Scale line weight* | why |
    /// |---|---|---|
    /// | uniform | **off** | the matrix scales it anyway, against the request |
    /// | non-uniform | either | the stroke is anisotropic; no scalar describes it |
    ///
    /// ⇒ `uniform` is carried so the sentence can name the remedy that
    /// actually applies. Under a uniform scale, turning *Scale line weight* on
    /// makes the resize **exact**; under a non-uniform one it does not help and
    /// only *Allow the artwork to distort* will proceed.
    ///
    /// ★★ Inkscape hit the identical limit in SVG (Launchpad #1335376) and
    /// closed it **Invalid** — correct spec behaviour — and its response is to
    /// silently produce a distorted stroke. This is the sentence that makes
    /// pdfcer better than the parity reference rather than equal to it, which is
    /// what the engine's own note recommended.
    ResizeNotRebuildable {
        /// Whether the drag was proportional, which decides which remedy the
        /// sentence names.
        uniform: bool,
    },
    /// **`edit.form_flatten` was invoked and the document's certification
    /// forbids it.**
    ///
    /// The ribbon control is `enabled_when("doc.pages")` and is therefore live
    /// on any open document, where the Forms panel's own Flatten button greys
    /// itself from `EditSession::flatten_refusal`. The two disagree on
    /// *appearance* and agree on *behaviour*, which is the intended shape:
    /// publishing a certification condition would cost a query per frame for a
    /// control that is almost never pressed, so the ribbon asks at the moment
    /// of the press and answers in a sentence.
    ///
    /// ★ It asks `flatten_refusal` and **not** `fill_refusal`, which is the
    /// distinction the panel's own comment spent twenty lines earning: flatten
    /// removes the form, so it takes the strict structural gate, and on a
    /// certified fillable form at `/P 2` filling is permitted while flattening
    /// is not. An operator who has just typed into the form and then finds
    /// Flatten refusing is meeting a real rule rather than a broken control,
    /// and the sentence says which.
    FlattenCertified,
    /// **A form field or one of its boxes was asked to be deleted and the
    /// document's structure is frozen** — `EditSession::deletion_refusal`
    /// answered `Some`.
    ///
    /// [`Self::FlattenCertified`]'s sibling with the same gate underneath
    /// (`structural_form_refusal`: `/Encrypt`, then the strict certification
    /// check), asked about a different verb.
    ///
    /// They share a gate and are still different questions — flatten
    /// additionally creates page content and carries a `/Size`-suppression
    /// guard deletion does not — which is why core exposes them as two
    /// functions and why this is a second variant rather than a second caller
    /// of the first.
    ///
    /// # ★★★ Why it exists when all four doors are already gated
    ///
    /// It should be unreachable, and that is exactly why it is worded. Every
    /// route an operator has to `delete_field` / `delete_widget` now consults
    /// `crate::panels::properties::formfield::refuses_delete` before offering
    /// anything: the Properties panel's two buttons, the `canvas.field` menu
    /// item (through `selection.delete_permitted`), the Delete key's rung 0,
    /// and the dispatcher's arm.
    ///
    /// What is left is the residue those four cannot cover, because **a gate
    /// is a forecast of the engine's guard and the guard is the authority**:
    ///
    /// * a **chord** bound to `format.delete` — a chord consults no
    ///   `visible_when`, so no menu or ribbon condition reaches it;
    /// * a condition that went stale inside a frame;
    /// * a refusal `deletion_refusal` does not predict;
    /// * and the case the *panel* cannot cover either — a delete reaching the
    ///   verb with **no field selected**, where there is no properties section
    ///   drawing a sentence at all.
    ///
    /// ⇒ Before 2026-08-29 that residue was **silence**:
    /// `crate::app::actions::apply::vector_edit`'s `Err` arm wrote one line to
    /// the trace and, by its own recorded decision, said nothing to the
    /// operator. (Since O116 it words [`Self::EditRefused`], which names no
    /// field and no gate — the floor, not this sentence.) R83's rule is not
    /// *gate the controls*, it is **a refusal must
    /// be a sentence, never a silence** — so the verb still owes the sentence
    /// for the case the forecast missed.
    ///
    /// ★★ It is deliberately **not** the wording the Properties panel draws.
    /// That one is a standing *description* of the document, drawn from the
    /// moment a field is selected; this is a *decline*, reporting that a
    /// gesture just happened and took no effect. This module's header insists
    /// the two speech acts must not wear the same words in the same place, and
    /// [`crate::text::status::field_delete_declined_structural`] carries the
    /// full argument for every word the two do not share.
    FieldDeleteRefused,
    /// ★★★ **The Points tool was pressed in a mode that cannot author.**
    ///
    /// `OPERATOR_REQUESTS.md` row **O69**. The arm has always declined — an
    /// anchor is selected in order to be *dragged*, and a mode that refuses
    /// the drag must refuse the tool rather than arm it and say no to every
    /// gesture afterwards — but it declined into the trace alone, so the
    /// operator pressed a control and the program did nothing and said
    /// nothing. That silence is half of why he reported the route as
    /// unreliable.
    ///
    /// The ribbon item is now withheld outside Edit, so the only surviving
    /// route to this decline is the bare `A` chord: chords are filtered by
    /// **tab** visibility and View is in every mode, so the key still reaches
    /// the arm. A key that does nothing has no control to hover, which makes
    /// it the case that most needs a sentence rather than the least.
    NodeToolNeedsEditMode,
    /// ★★★ **A corner of a ce dimension could not be added or taken away** —
    /// the operator's report of 2026-09-05, in his own words:
    ///
    /// > *"I also can't edit or delete nodes of a markup shape once it is
    /// > drawn."*
    ///
    /// The payload is [`crate::text::measure::VertexEditRefusal`], modelled on
    /// [`Self::TextStyle`]'s and [`Self::Rotate`]'s and for the identical two
    /// reasons: a `Copy` enum keeps this type `Copy` and [`Declined::line`]
    /// `&'static str`, and it keeps the engine's own diagnostic prose off the
    /// status bar.
    ///
    /// # ★ Why a gesture with a preflight still needs a decline
    ///
    /// `canvas::dimdrag::count_edit` asks `EditSession::vertex_edit_preview`
    /// before it draws anything, so a refused edit is never previewed and never
    /// raised as an action — the engine is never asked to refuse. That is the
    /// right design and it makes this sentence **the only report of the
    /// refusal that exists**: no action, no funnel, no `EditRefused`. Without
    /// it the operator drags a corner of a triangle out of the shape, releases,
    /// and the triangle is still a triangle with nothing anywhere saying why.
    /// That silence is precisely the shape of the report this whole surface
    /// answers.
    ///
    /// # Recorded from the gesture, before any verb
    ///
    /// From `canvas::dimdrag` on the release frame — the placement
    /// [`record_flatten_certified`] uses and for its stated reason: the
    /// condition is a **query** the shell can answer itself, so it is answered
    /// where the gesture is rather than inside a funnel the gesture never
    /// enters.
    ///
    /// # Retired by the operator's next act
    ///
    /// [`Self::still_true`] answers `true`, joining the group whose reason is
    /// *nothing happened*: the edit was refused before it began, so the epoch
    /// did not move, the sidecar is as it was, and there is no state for a
    /// later frame to find the sentence stale against. Deliberately **not**
    /// re-asked through `vertex_edit_preview` — that is a sidecar read, and
    /// putting it in the per-frame path that decides whether a status line is
    /// still true would pay for it sixty times a second to learn an answer that
    /// cannot change without a command.
    VertexEditRefused(crate::text::measure::VertexEditRefusal),
    /// ★★★ **A node of a MARKUP shape could not be moved, added or taken
    /// away** — the other half of the operator's report of 2026-09-05:
    ///
    /// > *"I also can't edit or delete nodes of a markup shape once it is
    /// > drawn."*
    ///
    /// [`Self::VertexEditRefused`] answers for a **ce dimension** and this for
    /// a comment shape, and they are two variants rather than one for R8b rule
    /// 15's reason: the ce-dimension sentences say *"measurement"*, which is
    /// the wrong word for a polygon somebody drew as a comment, and one enum
    /// serving both would have to say something vague enough to be true of
    /// either. See [`crate::text::markup::NodeEditRefusal`].
    ///
    /// # ★★ Why a gesture with a preflight still needs a decline
    ///
    /// `canvas::annotnodes` asks `EditSession::reshape_annotation_preview`
    /// before it draws anything, so a refused edit is never previewed and never
    /// raised — the engine is never asked to refuse. That is the right design
    /// and it makes this sentence **the only report of the refusal that
    /// exists**: no action, no funnel, no `EditRefused`. Without it the
    /// operator drags a corner of a triangle out of the shape, releases, and
    /// the triangle is still a triangle with nothing anywhere saying why.
    ///
    /// # ★ It is also raised where there was never a gesture
    ///
    /// `annotnodes::explain_unreshapable` raises it when the operator arms the
    /// **Points tool** over a shape that shows no anchors at all — a rectangle,
    /// an ellipse, a freehand mark. R9 says an unavailable capability renders
    /// nothing, and *nothing* is also what a build that forgot to draw the
    /// anchors renders. The operator cannot tell those apart by looking, so the
    /// absence is stated.
    ///
    /// # Retired by the operator's next act
    ///
    /// [`Self::still_true`] answers `true`, joining the group whose reason is
    /// *nothing happened*: the edit was refused before it began, so the epoch
    /// did not move and there is no state for a later frame to find the
    /// sentence stale against.
    MarkupNodeRefused(crate::text::markup::NodeEditRefusal),
    /// **The field-group deletion PREVIEW refused**, so the operator was never
    /// offered the confirmation.
    ///
    /// Its own variant rather than folded into [`Self::FieldGroupDeleteRefused`]
    /// because they are different moments with different remedies: this one
    /// means pdfcer could not work out what the deletion would remove, and the
    /// other means it worked that out, showed the operator, and was then
    /// refused. An operator who reads the second after pressing the first
    /// learns nothing about which half failed.
    FieldGroupPreviewRefused,
    /// **The field-group deletion refused**, after the operator confirmed it.
    FieldGroupDeleteRefused,
    /// **A bookmark was dropped on itself, or somewhere inside itself** — the
    /// shell's own forecast of `EditError::OutlineMoveIntoOwnSubtree`,
    /// 2026-08-29.
    ///
    /// # ★★★ Why a drag needs this more than a button does
    ///
    /// A drag that is released and does nothing is **this project's founding
    /// defect shape** — the sentence [`Self::Rotate`] carries about the ninth
    /// handle, and the reason the eight resize grips were the shell's longest
    /// standing complaint. A bookmark drag is worse than a grip, because the
    /// row genuinely leaves the operator's pointer during the gesture: what a
    /// silence looks like from their side is *"it went somewhere"*, and this
    /// very feature can put a bookmark somewhere they cannot see (see
    /// [`crate::text::panels::bookmarks::bookmark_move_into_collapsed`]).
    ///
    /// ⇒ So the two readings a silence invites — *"the drag did not register"*
    /// and *"it moved and I have lost it"* — are both wrong, and one of them is
    /// a state the panel can genuinely produce. R83's rule is not *gate the
    /// control*; it is **a refusal must be a sentence, never a silence.**
    ///
    /// ★ The caret is already dimmed over such a landing before the press,
    /// which is this panel's preferred channel. This is what is owed to the
    /// operator who released anyway — and they will, because the mark is faint
    /// by design and a hand that has committed to a drag finishes it.
    ///
    /// # ★★ Recorded from the VERB, although the shell saw it coming
    ///
    /// The panel forecasts this landing — it is a question about the tree it
    /// has already drawn — and uses the forecast to draw the faintest of its
    /// three carets. It does **not** use it to skip the call.
    ///
    /// The reason is this module's own boundary. `decline` is `pub(super)`
    /// inside `crate::app` on a stated argument — *"a decline is written by the
    /// one dispatcher and read by the one bar"* — and a panel is outside it.
    /// The two ways round are worse than going through: a `record_note` from
    /// the panel would render the sentence under `⚑ About your last edit:`,
    /// which [`crate::text::status`]' own rule forbids for a decline, and
    /// widening this module would trade a real invariant for one call site.
    ///
    /// ⇒ So the move is raised, `EditSession::move_outline_item` refuses it by
    /// name (`EditError::OutlineMoveIntoOwnSubtree`, *"refused unconditionally
    /// … a cycle is a defect whatever Acrobat does"*), and
    /// `crate::app::actions::bookmarks::move_to` records this from inside the
    /// `vector_edit` closure. The guard runs before the verb plans anything, so
    /// nothing is written, no epoch moves and no undo entry appears.
    ///
    /// ★ It also puts the authority in one place. The forecast decides what the
    /// **caret** looks like; the engine decides what **happens**. They cannot
    /// drift into disagreeing about the outcome, because only one of them
    /// produces it.
    BookmarkMoveIntoOwnSubtree,
    /// **The engine refused a bookmark move**, 2026-08-29 — the residue the
    /// shell's forecast cannot cover.
    ///
    /// [`Self::BookmarkMoveIntoOwnSubtree`]'s sibling at the other end of the
    /// call, and its own variant for exactly the reason the two field-group
    /// declines are two: *"they are different moments with different remedies"*.
    /// This one means the shell asked and pdfcer said no; the other means the
    /// shell never asked. An operator who read one after the other would learn
    /// nothing about which half refused.
    ///
    /// What can reach it: `/Encrypt`, the certification gate, and an id that
    /// stopped resolving between the frame that drew the row and the apply that
    /// moved it — which is the ordinary state one frame after an undo. None is
    /// guessable from the screen, which is
    /// [`crate::text::status::field_delete_declined_structural`]'s argument for
    /// its own verb.
    ///
    /// ★ Recorded from **inside** the `vector_edit` closure —
    /// [`record_resize_not_rebuildable`]'s placement, and its stated reason:
    /// whether the engine will refuse is not knowable before the call.
    BookmarkMoveRefused,
    NothingToUndo,
    /// **`edit.redo` was invoked with an empty redo stack.**
    ///
    /// Distinct from [`Self::NothingToUndo`] for the reason the module header
    /// gives about the disclosures and the declines generally — the operator
    /// gets **one** line, and these two describe different states with
    /// different remedies. An empty undo log means nothing has been changed at
    /// all; an empty redo stack is the ordinary state of a document that has
    /// been edited and never undone, and it is *also* what a fresh edit after an
    /// undo produces, because `EditSession::commit` clears the redo stack when a
    /// new command is recorded (*"the redone future no longer exists once
    /// history diverges"*).
    ///
    /// Its live predicate is `EditSession::can_redo`, for
    /// [`Self::NothingToUndo`]'s reason and asked the same way.
    NothingToRedo,
    /// ★★★ **The engine refused an edit and this shell cannot say why** —
    /// `OPERATOR_REQUESTS.md` **O116**, 2026-09-04.
    ///
    /// The **last** variant in this enum in every sense: it is what the
    /// operator is told when no other variant applies, recorded from the one
    /// funnel every document change passes through
    /// ([`super::super::actions::funnel`]) rather than from any verb.
    ///
    /// # ★★★ It is the deferral this file's neighbours kept naming, taken
    ///
    /// Six variants above cite `vector_edit`'s error arm by name and say some
    /// version of *"before this, that residue was a **silence**"* —
    /// [`Self::FieldDeleteRefused`], [`Self::BookmarkMoveRefused`],
    /// [`Self::ResizeNotRebuildable`] among them. Each of them worded **one**
    /// verb's residue. The arm itself stayed silent for every other verb, and
    /// its own comment said so deliberately: *"That is `FEATURES.md`'s 'Worded
    /// decline' row, which wants its own decision about wording and placement;
    /// this arm is where it lands when it is taken."* This is it, taken.
    ///
    /// What made it urgent rather than tidy is that the silence became
    /// reachable on **an ordinary CAD drawing with an ordinary embedded font**:
    /// Edit ▸ Edit text arms, a caret lands, characters are typed, Enter
    /// commits, `EditSession::edit_text` refuses a symbolic font it cannot
    /// re-encode, and nothing whatever appears. That is this project's founding
    /// defect class — *"I did the thing and nothing happened and nothing said
    /// why"* — reproduced by the driven check `text_edit_on_a_real_drawing`.
    ///
    /// # ★★ It carries NO payload, unlike every other refusal variant here
    ///
    /// [`Self::TextStyle`], [`Self::Rotate`] and [`Self::Unshare`] each carry a
    /// `Copy` enum saying *which* refusal, because in those three cases the
    /// shell can tell: the verb has a small, closed set of engine errors and a
    /// hand-written `refusal_for` maps them. This one deliberately has none,
    /// and adding one would be the exact mistake those three narrowly avoid at
    /// scale — a second copy of `pdfcer-core`'s whole taxonomy, in this crate,
    /// drifting from theirs. [`crate::text::status::edit_declined_by_engine`]
    /// carries the full argument, including why the sentence points nowhere.
    ///
    /// ⇒ A payload arrives the day `EditError` exposes a coarse `kind()`. Until
    /// then the honest arity is zero.
    ///
    /// # ★★★ Retirement: the `retire`-only class, and NOT for its usual reason
    ///
    /// [`Self::still_true`] answers `true` unconditionally, joining
    /// [`Self::SaveFailed`], [`Self::FlattenCertified`] and the rest — but the
    /// argument those variants use **does not hold here**, and copying it would
    /// be recording a fact this shell does not have.
    ///
    /// Their argument is *stability*: a document does not stop being certified,
    /// a folder does not become writable, an appearance does not become
    /// rebuildable, between one frame and the next. **This decline cannot claim
    /// that.** Its causes are unknown by construction, and the set certainly
    /// contains conditions that change under the operator — the residue
    /// [`Self::BookmarkMoveRefused`] names is *"an id that stopped resolving
    /// between the frame that drew the row and the apply that moved it, which
    /// is the ordinary state one frame after an undo"*, and that is squarely
    /// inside what an unexplained refusal can be.
    ///
    /// So it is not in the stable class. Nor can it be in the live-predicate
    /// class ([`Self::NothingToFrame`], [`Self::NothingToUndo`],
    /// [`Self::InsideForm`]), and the reason is structural rather than
    /// awkward: **that class's entry requirement is that the sentence be
    /// re-asked through the same predicate that produced it**, and there is no
    /// predicate here to re-ask. Inventing a plausible one would be the
    /// "second spelling that drifts" this module's header forbids, with a
    /// failure mode worse than drift — a guessed predicate that answered
    /// `false` would retire a **true** sentence while the operator was reading
    /// it, which is the silence all over again with an extra step.
    ///
    /// ⇒ What actually earns the `true` is the **tense**. This sentence is a
    /// report of a past moment — *that change was refused, and the document is
    /// unchanged* — and it was true when it was written whatever the frame does
    /// afterwards. That is [`Self::SaveFailed`]'s second clause, the one its
    /// docs add after the stability claim: *"and if it did, the sentence would
    /// still be a true report of what happened when the operator pressed
    /// Save."* Here that clause is not the supporting argument; it is the whole
    /// of it. A sentence in the past tense can go stale, and [`retire`] — the
    /// operator's next command — is what handles stale.
    EditRefused,
    /// ★★★ **A reflow that did not happen, and which of its eight causes it
    /// was** — `OPERATOR_REQUESTS.md` **O127**, defect 3.
    ///
    /// All eight were **already being reported** before O127, and none of them
    /// reached the operator: four went through
    /// `crate::app::actions::record_note`, which draws under `⚑ About your last
    /// edit:` for a press where nothing happened, and four collapsed into
    /// [`Self::EditRefused`]'s nine cause-free words. His verdict was *"I
    /// haven't seen the reflow option actually work with anything when I press
    /// it."* `decline/textedit.rs` carries the whole argument.
    ///
    /// ★ It carries the cause rather than being eight variants, on
    /// [`Self::Rotate`]'s and [`Self::TextStyle`]'s precedent: the catalog owns
    /// the wording and this enum owns only which sentence.
    Reflow(crate::text::textedit::ReflowRefusal),
    /// ★★★ **Enter was pressed in text that is already on the page, where a
    /// line break cannot go** — `OPERATOR_REQUESTS.md` **O127**, defect 2.
    ///
    /// Enter means *a new line* in every draft this shell has; in an existing
    /// show operator the FILE forbids one, so it declines by name instead. It
    /// used to **commit**, silently — the operator asked *"can the enter key
    /// create new lines?"* and was answered by an edit finishing under him.
    /// See `decline/textedit.rs`.
    EnterCannotSplit,
    /// ★★★ **A cut or a paste the active MODE does not do** — 2026-09-05, and
    /// it is the second half of the defect the driven sweep found as A1.
    ///
    /// The first half was that `edit.paste` could not be *reached* in Review at
    /// all: `app::modes::capability::offers_command` refused the chord because
    /// Paste lives on the Edit tab, so an operator in the mode whose entire
    /// purpose is marking up somebody else's drawing could copy a comment and
    /// had nowhere to put it. Two independent driven checks traced
    /// `chord-not-offered id=edit.paste mode=review`.
    ///
    /// ⇒ The chord now reaches the dispatcher, which was **already** gating the
    /// effect correctly on what is on the clipboard. This variant is what makes
    /// that safe: the moment a chord is allowed through blind, every refusal it
    /// can meet has to be worded, or the fix trades *"the key does nothing and
    /// the trace says why"* for *"the key does nothing and nothing says why"* —
    /// which is worse, because the second has no trace line either.
    ///
    /// ★ It carries [`crate::text::clipboard::ModeRefusal`] rather than being
    /// six variants, on [`Self::Rotate`]'s and [`Self::Reflow`]'s precedent: the
    /// catalog owns the wording and this enum owns only which sentence.
    ///
    /// ## Retirement: the `retire`-only class, on the TENSE argument
    ///
    /// [`Self::still_true`] answers `true`, and the reason is
    /// [`Self::EditRefused`]'s rather than [`Self::SaveFailed`]'s. It **cannot**
    /// claim stability — the operator can change the mode, and changing the mode
    /// is precisely the remedy the sentence names, so the condition it reports
    /// is one they are being invited to falsify. Nor can it be re-asked through
    /// a live predicate: the fact recorded is *what the clipboard held at the
    /// moment of the press*, and the clipboard can change under it.
    ///
    /// ⇒ What earns the `true` is that the sentence is a **report of a past
    /// moment** — *that press did nothing, and the document is unchanged* — and
    /// it was true when it was written whatever the next frame does. An
    /// operator who reads it, moves the selector and presses again retires it
    /// with that press, through [`retire`], which is the honest lifetime.
    ClipboardMode(crate::text::clipboard::ModeRefusal),
    /// ★★★ **A text edit the engine refused, and WHICH KIND of refusal it was**
    /// — `OPERATOR_REQUESTS.md` **O140**, 2026-09-05.
    ///
    /// The operator: *"on page 2 there is a spelling mistake — clien instead of
    /// client. if I try to edit the edit is not accepted."*
    ///
    /// # What this replaces, and it is not a silence
    ///
    /// [`Self::EditRefused`] was already reaching him — O116 shipped it on
    /// 2026-09-04 and a driven run on his own file confirms the `⊗` slot draws
    /// one frame after the refusal. What he read was *"That change was refused,
    /// and the document is unchanged."* True, complete about the document, and
    /// **silent about the one thing he wanted**: why, and whether he can do
    /// anything.
    ///
    /// ⇒ So this is not the founding defect class a second time. It is the
    /// *next* rung of it: a sentence that says nothing actionable is not the
    /// same as no sentence, and it is not good enough either.
    ///
    /// # ★★★ It exists because [`Self::EditRefused`]'s stated blocker LIFTED
    ///
    /// That variant's documentation is explicit — *"It carries NO payload,
    /// unlike every other refusal variant here… adding one would be the exact
    /// mistake those three narrowly avoid at scale — a second copy of
    /// `pdfcer-core`'s whole taxonomy… ⇒ A payload arrives the day `EditError`
    /// exposes a coarse `kind()`. Until then the honest arity is zero."*
    ///
    /// **`pdfcer-core` shipped `text_edit::RefusalKind` at `b1033ab`**, in
    /// answer to this project's own request, deliberately not
    /// `#[non_exhaustive]` so a front end may match it exhaustively. The
    /// condition that variant named is met, so this one exists — and it carries
    /// a payload for [`Self::Reflow`]'s and [`Self::Rotate`]'s reason: the
    /// catalog owns the wording and this enum owns only which sentence.
    ///
    /// ★ [`Self::EditRefused`] is **not** deleted, and that is deliberate
    /// rather than an oversight. It is the funnel's floor for **every other
    /// verb** — ~78 call sites — and only `edit_text` has been given a
    /// classifier. Deleting it would silence the other seventy-seven.
    ///
    /// # Retirement: the `retire`-only class, on the TENSE argument
    ///
    /// [`Self::still_true`] answers `true`, and the argument is
    /// [`Self::EditRefused`]'s exactly: this is a **report of a past moment** —
    /// *that commit was refused, and the document is unchanged* — true when it
    /// was written whatever the next frame does. It cannot claim stability (an
    /// operator may unlock a protected document, and `DocumentProtected` names
    /// that as the remedy), and it cannot be re-asked through a live predicate,
    /// because the fact recorded is *what the engine answered about a request
    /// that no longer exists*. The operator's next command retires it.
    EditText(crate::text::textedit::EditRefusal),
}

impl Declined {
    /// The decline in an outcome, if it is one.
    ///
    /// `None` for [`ZoomOutcome::Zoomed`] **including the clamped case**. See
    /// the module docs: a clamped framing zoom is a partial grant that already
    /// reports itself through the zoom readout, and wording it here would word
    /// a non-event.
    #[must_use]
    pub(crate) fn of(outcome: ZoomOutcome) -> Option<Self> {
        match outcome {
            ZoomOutcome::NoBounds => Some(Self::NothingToFrame),
            ZoomOutcome::NoCanvas => Some(Self::CanvasNotDrawn),
            ZoomOutcome::Zoomed { .. } => None,
        }
    }

    /// Whether this decline still describes the application in front of the
    /// operator.
    ///
    /// **Pure, and that is the point** — the project's standing split
    /// (`crate::viewer`'s header: *"this module is unit-testable and the widget
    /// code is not"*). Every property of the retirement rule that can be wrong
    /// is decided here and asserted headlessly; [`live`] adds only "go and ask
    /// the two questions".
    ///
    /// The facts are named as booleans rather than taken as a `&OpenDoc`
    /// so that the caller is forced to state *which* question it asked. All
    /// are asked through the same predicates that produced the decline in the
    /// first place, which is what stops a second spelling of "is there
    /// anything to frame?" drifting away from the first.
    ///
    /// # ★ Why a fourth parameter rather than a `&OpenDoc`
    ///
    /// [`History`] arrived with the undo wiring and needed a third fact — *is
    /// there anything on the stack now?* — which is where the temptation to
    /// collapse the list into the document it is all read from is strongest.
    /// The list stays, for the reason it was a list to begin with: a
    /// `&OpenDoc` here would make this function able to ask **any** question,
    /// and the one property that makes it worth testing is that every question
    /// it asks was asked by the code that produced the decline. The parameters
    /// are the contract; [`live`] is the only place allowed to go and get them.
    ///
    /// The two history variants take their fact as *one* [`History`] pair
    /// rather than as two more booleans, so a caller cannot transpose them —
    /// and each arm below names the field it reads, so neither can read the
    /// other's stack.
    #[must_use]
    fn still_true(
        self,
        has_bounds: bool,
        canvas_has_drawn: bool,
        history: History,
        selection_in_form: bool,
    ) -> bool {
        match self {
            // The operator has selected something framable: the sentence is
            // now history, and a stale explanation beside a live control is
            // worse than none — it attaches a refusal to a state that would
            // not produce one.
            Self::NothingToFrame => !has_bounds,
            // The page has drawn. The remedy happened on its own, without the
            // operator doing anything, which is exactly what the sentence
            // promised ("…has not finished drawing").
            Self::CanvasNotDrawn => !canvas_has_drawn,
            // ★ Neither fact is about this one, and there is no third fact to
            // add. A write that failed stays failed until the operator does
            // something about it, and what they do about it is a *command* —
            // which `retire` catches. See the variant's own docs; the two
            // parameters are deliberately ignored rather than being joined by a
            // third that would always be `true`.
            // ★ `ResizeNotRebuildable` joins them: whether an appearance can
            // be rebuilt is a property of the FILE, and it does not change
            // while the operator reads the status bar. What retires it is their
            // next act — including, in the good case, ticking the switch the
            // sentence just named.
            Self::SaveFailed | Self::SettingsNotSaved | Self::ResizeNotRebuildable { .. } => true,
            // ★ `true`, with the others whose state cannot change between two
            // frames. A document's certification is a property of the file: it
            // does not lapse while the operator looks at the status bar, and
            // the only thing that would retire this sentence is opening a
            // different document — which retires every sentence.
            //
            // Deliberately NOT re-asked through `flatten_refusal`. It is a
            // certification census over the whole document, and putting it in
            // the per-frame path that decides whether a status line is still
            // true would pay for it sixty times a second to learn an answer
            // that never moves.
            // ★ `FieldDeleteRefused` joins it on the identical argument with
            // `deletion_refusal` substituted for `flatten_refusal`: `/Encrypt`
            // and a certification signature are properties of the FILE, neither
            // lapses while the operator reads the status bar, and the only
            // thing that would retire the sentence is opening a different
            // document — which retires every sentence. Deliberately NOT
            // re-asked through `deletion_refusal`: that is a signature census
            // over the whole document, and putting it in the per-frame path
            // that decides whether a status line is still true would pay for it
            // sixty times a second to learn an answer that never moves.
            // ★★ The two field-group declines are `true` for a DIFFERENT reason
            // from their neighbours above, and the difference is worth the
            // separate arm rather than an extra `|`.
            //
            // Those are true because the FILE cannot change under them. These
            // are true because **nothing happened**: the preview refused or the
            // deletion refused, so the epoch did not move, the form is as it
            // was, and there is no state for a later frame to find the sentence
            // stale against. What retires them is the operator's next act,
            // which is what retires every decline.
            // ★★ The two bookmark declines join the field-group pair on the
            // *second* argument rather than the first, and it is worth saying
            // which: **nothing happened.** The move was never made or was
            // refused, so the epoch did not move, the outline is as it was, and
            // there is no state for a later frame to find the sentence stale
            // against. Deliberately NOT re-asked against the tree: a walk of
            // the outline to decide whether a status line is still true would
            // pay for a `read_outline` sixty times a second to learn an answer
            // that cannot change without a command — and a command is what
            // `retire` catches.
            // ★ The mode is not going to change between one frame and the
            // next without a **command**, and a command is exactly what
            // `retire` catches — so there is no live predicate to re-ask, on
            // the identical argument the five below it make. Note the
            // asymmetry with `NothingToFrame`: a selection can appear while
            // the operator reads, a mode cannot.
            Self::NodeToolNeedsEditMode
            | Self::FlattenCertified
            | Self::FieldDeleteRefused
            | Self::FieldGroupPreviewRefused
            | Self::FieldGroupDeleteRefused
            | Self::BookmarkMoveIntoOwnSubtree
            | Self::BookmarkMoveRefused
            | Self::VertexEditRefused(_)
            | Self::MarkupNodeRefused(_) => true,
            // ★ Same ruling, third and fourth cases. A name is not going to
            // stop being taken, and a widget is not going to grow a `/T`,
            // between one frame and the next. Both are corrected by the
            // operator doing something — typing a different name and pressing
            // Register again — and pressing Register is a command, which
            // `retire` catches.
            Self::FieldNameTaken | Self::WidgetHasNoName => true,
            // ★★ Same ruling, and here the temptation to key on
            // `selection_in_form` is strongest: all but one of the sentences are
            // about the DOCUMENT (encrypted, signed, damaged index, nested
            // drawing, drawn nowhere else), none of which changes while the bar
            // is read, and the last sends the operator to click INSIDE a form —
            // so re-asking would delete the instruction as they began to follow it.
            Self::Unshare(_) => true,
            // ★ The stack filled up. Something was authored — or, for redo,
            // something was undone — and the sentence is now history, exactly
            // as `NothingToFrame` is once something is selected. The operator
            // reaches this without invoking any command, which is why the
            // filter is needed at all: `retire` would not have run.
            Self::NothingToUndo => !history.can_undo,
            Self::NothingToRedo => !history.can_redo,
            // ★ True while the operator is still looking at the selection the
            // sentence is about. Selecting something else — including the
            // containing form, which is the remedy the sentence exists to send
            // them to — ends it, without any command being invoked and so
            // without `retire` running. That is precisely the case the filter
            // exists for, and it is the same shape as `NothingToFrame`.
            Self::InsideForm => selection_in_form,
            // ★ See the variant's docs: nothing on the frame can make a
            // restyle refusal stop being a true report of what happened when
            // the operator pressed the control. `retire` ends it.
            Self::TextStyle(_) => true,
            // ★ Same ruling again, and each of the four refusals earns it
            // separately: a document does not stop being signed, a sidecar does
            // not grow a record, and a routing bug does not fix itself, between
            // one frame and the next. The remedy is always something the
            // operator *does* — and doing it is a command, which `retire`
            // catches.
            Self::Rotate(_) => true,
            // ★★★ `true`, and NOT on the stability argument its neighbours in
            // this arm use — see the variant's own docs, which spend a section
            // refusing to claim it. An unexplained refusal's causes are unknown
            // by construction and some of them do change under the operator.
            //
            // What earns it is that the sentence is in the **past tense**: it
            // reports what happened when the operator pressed, so nothing on a
            // later frame can falsify it — only make it stale, which is
            // `retire`'s job. And there is nothing to re-ask: the live-predicate
            // class exists for declines that can be put back to *the predicate
            // that produced them*, and this one was produced by an error value
            // this shell is not permitted to interpret.
            Self::EditRefused => true,
            // ★ Both on the TENSE argument, not the stability one: several of
            // these causes are live predicates the operator changes in a click,
            // so `EditRefused`'s reasoning applies rather than its neighbours'.
            // The sentence reports the press; `retire` owns stale.
            Self::Reflow(_) | Self::EnterCannotSplit => true,
            // ★ On the TENSE argument too, and here it is the ONLY argument
            // available: the condition this reports is the one the sentence
            // asks the operator to change. A live predicate would retire the
            // explanation at the instant they acted on it.
            Self::ClipboardMode(_) => true,
            // ★★ On the TENSE argument, and inheriting `EditRefused`'s section
            // wholesale — this variant is that one with a cause attached, so
            // the retirement reasoning is unchanged by the payload. It reports
            // what the engine answered about a request that no longer exists;
            // there is no predicate to re-ask, and `retire` owns stale.
            Self::EditText(_) => true,
        }
    }

    /// The sentence, from the catalog.
    ///
    /// The mapping is the whole of this module's contribution to the copy;
    /// every word an operator reads is [`crate::text::status`]'s, under rule
    /// R1.
    #[must_use]
    fn line(self) -> &'static str {
        match self {
            Self::NothingToFrame => t::zoom_declined_no_selection(),
            Self::CanvasNotDrawn => t::zoom_declined_not_drawn(),
            Self::InsideForm => t::selection_inside_form_declined(),
            Self::SaveFailed => t::save_copy_failed(),
            Self::SettingsNotSaved => t::settings_not_saved(),
            Self::NothingToUndo => t::undo_declined_empty(),
            Self::NothingToRedo => t::redo_declined_empty(),
            Self::FieldNameTaken => t::adopt_declined_name_taken(),
            Self::WidgetHasNoName => t::adopt_declined_no_name(),
            Self::ResizeNotRebuildable { uniform } => t::resize_not_rebuildable(uniform),
            Self::FlattenCertified => t::flatten_declined_certified(),
            Self::FieldDeleteRefused => t::field_delete_declined_structural(),
            // ★ Stays in `crate::text::status` rather than reaching across the
            // way the five arms below do, and the reach-across rule is what
            // decides it rather than what is bent for it: *a string lives with
            // the surface that owns its subject*, and this sentence's subject
            // is not a tool, a panel or a verb — it is any edit at all,
            // arriving from ~78 call sites through one funnel. The only surface
            // that owns it is this bar's `⊗` slot, whose catalog area is
            // `text::status`. It is in a FILE of its own there for the reason
            // `field_delete_declined_structural` above is: `text::status`'
            // `mod.rs` stands two dozen lines from R2's ceiling.
            Self::EditRefused => t::edit_declined_by_engine(),
            // ★ Reaches across to `crate::text::textedit` on the same rule the
            // arms below use: a string lives with the surface that owns its
            // subject, and every one of these eight sentences is about the text
            // caret and the paragraph under it. `ReflowRefusal::line` is the
            // one mapping, so the shell-side causes and the engine-side ones
            // cannot drift into two voices.
            Self::Reflow(why) => why.line(),
            // ★ Same catalog and same subject as the reflow family above: the
            // text caret and what the page under it will accept.
            Self::EnterCannotSplit => crate::text::textedit::enter_cannot_split_existing_text(),
            // ★ Reaches across to `crate::text::tool` rather than adding an
            // entry to `crate::text::status`, on the precedent the two field
            // -group sentences below already set: a string lives with the
            // surface that owns its subject, and `text::status` is at 1,482
            // lines against R2's 1,500.
            Self::NodeToolNeedsEditMode => crate::text::tool::node_tool_needs_edit_mode(),
            // ★ These two reach across to `text::forms::groups` rather than
            // adding entries here, and that is the catalog rule honoured rather
            // than bent: a string lives in `crate::text::…`, and the module
            // that owns this surface's other twenty sentences is the one that
            // owns these. `crate::text::status` is also two dozen lines from
            // R2's ceiling, which is a reason to notice the seam and not a
            // reason to choose it.
            Self::FieldGroupPreviewRefused => {
                crate::text::forms::groups::field_group_preview_declined()
            }
            Self::FieldGroupDeleteRefused => {
                crate::text::forms::groups::field_group_delete_declined()
            }
            // ★ These two reach across to `text::panels::bookmarks` on the
            // identical argument the field-group pair above records: a string
            // lives in `crate::text::…`, and the module that owns this
            // surface's other sentences owns these. `crate::text::status` is
            // two dozen lines from R2's ceiling, which is a reason to notice
            // the seam and not a reason to choose it.
            Self::BookmarkMoveIntoOwnSubtree => {
                crate::text::panels::bookmarks::bookmark_move_declined_own_subtree()
            }
            Self::BookmarkMoveRefused => {
                crate::text::panels::bookmarks::bookmark_move_declined_engine()
            }
            Self::TextStyle(why) => why.line(),
            Self::Rotate(why) => why.line(),
            Self::Unshare(why) => why.line(),
            // ★ Reaches across to `crate::text::clipboard` on the same rule the
            // arms above use: a string lives with the surface that owns its
            // subject, and this one's subject is the clipboard — where the
            // other three clipboard refusals already live, so a fourth wording
            // of "that did not happen" cannot grow up beside them.
            Self::ClipboardMode(why) => why.line(),
            // ★★ Same catalog as `Reflow` and `EnterCannotSplit` above, and the
            // same rule: this enum owns which sentence, `crate::text::textedit`
            // owns the words. `EditRefusal::Unstated` forwards to
            // `t::edit_declined_by_engine` — the line `Self::EditRefused` shows
            // — so the un-categorised case is the *same string*, in one place,
            // and cannot drift into two voices for one condition.
            Self::EditText(why) => why.line(),
            // ★ Reaches across to `crate::text::measure` on the same rule: a
            // string lives with the surface that owns its subject, and this
            // one's subject is what a ce dimension measures — where the
            // vertex-move disclosure it is the refusal twin of already lives.
            Self::VertexEditRefused(why) => why.line(),
            // ★ Reaches across to `crate::text::markup` on the same rule
            // every arm above uses: a string lives with the surface that owns
            // its subject, and this one's subject is a markup shape — where the
            // other twenty sentences about markup already live, so a second
            // wording of "that shape did not change" cannot grow up beside
            // them.
            Self::MarkupNodeRefused(why) => why.line(),
        }
    }
}

/// **What the command log says right now** — the fact
/// [`Declined::NothingToUndo`] and [`Declined::NothingToRedo`] are retired by.
///
/// A pair rather than two parameters because they are read together, from one
/// borrow of one session, and because a caller that had to pass two loose
/// booleans in the right order would eventually pass them in the wrong one —
/// and the symptom would be a sentence that retires when the *other* stack
/// fills, which reads exactly like a sentence that retires correctly.
///
/// Both are asked through `EditSession`'s own predicates, which is the same
/// pair `crate::app::conditions` publishes `undo.available`/`redo.available`
/// from and the same pair `crate::app::actions`' history arm declines on. Three
/// readers, one derivation: the control cannot be greyed while the sentence
/// says the opposite.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct History {
    /// `EditSession::can_undo` — something has been changed and not taken back.
    pub(crate) can_undo: bool,
    /// `EditSession::can_redo` — something has been taken back and not
    /// re-applied, and no command has been recorded since.
    pub(crate) can_redo: bool,
}

impl History {
    /// What the open document's session currently says.
    ///
    /// The one derivation, so the bar cannot learn this from a different
    /// question than the one that produced the sentence.
    #[must_use]
    fn of(doc: &OpenDoc) -> Self {
        Self {
            can_undo: doc.session.can_undo(),
            can_redo: doc.session.can_redo(),
        }
    }
}

thread_local! {
    /// The most recent declined command, waiting to be read by the status
    /// bar. See the module docs for why a thread-local, and why that is sound
    /// rather than smuggled.
    static LAST: RefCell<Option<Declined>> = const { RefCell::new(None) };
}

// ---------------------------------------------------------------------------
// The store — written by the dispatcher, read by the bar
// ---------------------------------------------------------------------------

/// Forget any live decline — **the operator's next act**.
///
/// Called at the top of `crate::app::dispatch::PdfcerApp::dispatch_command`,
/// before the arm for the new command runs. That placement is the whole
/// retirement rule and it is deliberate on both counts:
///
/// - **the dispatcher**, because it is the one choke point that knows an
///   operator has invoked *something*, and "the next thing you did" is the
///   only honest lifetime for a sentence about a gesture. See the module docs
///   for why an epoch cannot serve here;
/// - **before the arm**, so that re-pressing the declining chord retires the
///   old sentence and then [`record`]s a new one. Two presses are two events
///   (module docs, reason 2), and this is where that becomes mechanical rather
///   than aspirational.
///
/// Idempotent and free: one `Option` write per *invoked command*, which is an
/// operator click, not a frame.
pub(crate) fn retire() {
    LAST.with_borrow_mut(|slot| *slot = None);
}

/// The live decline, if there is one and it still describes what the operator
/// is looking at.
///
/// The bar's read. Both facts are gathered from the modules that own them —
/// [`zoom::can_zoom_to_selection`] is the same predicate `view.zoom_selection`
/// is gated on and the same one [`zoom::zoom_to_selection`] declines from, and
/// [`zoom::last_frame`] is the same record the framing verbs check for
/// [`ZoomOutcome::NoCanvas`]. Asking the producing predicate rather than an
/// equivalent-looking one (`doc.page_texture.is_some()`, say, which is a
/// *different* question by one frame) is what keeps the retirement rule from
/// drifting away from the decline it retires.
///
/// Filters rather than clears; see the module docs.
#[must_use]
pub(super) fn live(ctx: &egui::Context, doc: &OpenDoc) -> Option<Declined> {
    let has_bounds = zoom::can_zoom_to_selection(doc);
    let canvas_has_drawn = zoom::last_frame(ctx).is_some();
    let history = History::of(doc);
    // ★ The same accessor `crate::app::conditions` publishes `selection.in_form`
    // from, asked here in the same words, so that the greyed control and the
    // sentence explaining why it is greyed cannot come from two questions that
    // drift apart.
    let selection_in_form = !doc
        .selection
        .leaf_indices_on(doc.view.page_index)
        .is_empty();
    LAST.with_borrow(|slot| {
        slot.filter(|d| d.still_true(has_bounds, canvas_has_drawn, history, selection_in_form))
            .to_owned()
    })
}

/// Record that a verb refused because what is selected lives inside a form
/// XObject.
///
/// # ★ Recorded by the DISPATCHER, not by an apply arm
///
/// [`record_history_empty`]'s docs argue the opposite placement for undo, and
/// the argument holds there: *"is there anything to undo?"* is a question about
/// the document that the apply phase has to ask anyway, so asking it twice is
/// how the greyed control and the sentence come to disagree.
///
/// This one is different in the way that matters. *"Is this selection inside a
/// form?"* is answered from the **selection**, which the dispatcher holds, and
/// there is no apply phase to reach: the refusal is that no `Action` is raised
/// at all. An arm that raised a doomed action so that the apply phase could
/// decline it would be manufacturing an edit in order to have somewhere to
/// refuse it.
pub(crate) fn record_inside_form() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::InsideForm));
}

/// **The raw store, for tests only.**
///
/// [`live`] is the bar's read and applies the retirement filter, which needs a
/// document and a context. A test asserting that a *dispatcher* recorded a
/// decline is asking a narrower question — did the sentence get written down? —
/// and routing it through the filter would make the assertion depend on zoom
/// bounds and canvas state that have nothing to do with what it is testing.
///
/// ★ `cfg(test)` rather than `pub(crate)` unconditionally, so nothing in the
/// shipped build can read the store without the retirement rule.
#[cfg(test)]
#[must_use]
pub(crate) fn recorded_for_test() -> Option<Declined> {
    LAST.with_borrow(|slot| *slot)
}

// ---------------------------------------------------------------------------
// The line
// ---------------------------------------------------------------------------

/// Draw the worded decline into the bar's single row, if one is live.
///
/// Drawn through [`super::disclosure_line`] rather than by hand, which is the
/// point of that function existing: the R128 defence is four small rules that
/// only work together — a bounded sub-region, a fixed row height,
/// `truncate()` rather than wrapping, and the full text on hover — and a third
/// hand-written copy would be a third chance to omit one of them.
///
/// **It does not make the bar taller**, and that matters more here than for
/// its neighbours rather than less. A decline arrives from a *keyboard chord*,
/// which is the gesture during which the operator's hands are furthest from
/// the thing they are looking at; if this line grew the bar, an active
/// `FitMode` would recompute its zoom from a smaller viewport on the very next
/// frame and the page would shrink under a gesture that, by construction,
/// changed nothing. "The page moved when the command did nothing" is the
/// worst-reading symptom on this surface.
/// [`tests::a_worded_decline_does_not_change_the_bar_height`] pins it.
pub(super) fn show(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(declined) = live(ui.ctx(), doc) else {
        return;
    };
    super::disclosure::disclosure_line(ui, REGION_DECLINE, declined.line());
}

/// ★ **The funnel's floor**, split out under R2 when this file reached 1,530
/// lines. See `decline/floor.rs`'s header for why that particular seam: it is
/// the one part of this module that answers a question about somebody else's
/// protocol rather than about what a decline is.
mod floor;
/// Re-exported so that the one caller — `crate::app::actions::funnel` — still
/// says `decline::before_the_verb()`. The split is about where the code lives; a
/// call site should not have to learn that a submodule exists.
pub(crate) use floor::before_the_verb;

/// ★★★ **The two declines the text caret raises**, split out under R2 on
/// 2026-09-04 when `OPERATOR_REQUESTS.md` O127 took this file past 1,500 lines
/// for the second time. See `decline/textedit.rs`'s header for the seam and for
/// the argument both of them share — that a sentence in the wrong slot is
/// indistinguishable, from the operator's chair, from no sentence at all.
mod textedit;
/// Re-exported so the four call sites still say `decline::record_reflow(..)`
/// and `decline::record_enter_cannot_split()`. `floor`'s rule: the split is
/// about where the code lives, and a call site should not have to learn that a
/// submodule exists.
pub(crate) use textedit::{record_edit_text_refusal, record_enter_cannot_split, record_reflow};

/// ★★★ **Every writer of the decline slot**, split out under R2 on 2026-09-05
/// when this file reached 1,497 lines against the ceiling for the third time —
/// see `decline/record.rs`'s header for the seam. It is the same seam `floor`
/// and `textedit` already stand on: this file answers *what a decline is and
/// how long it owes its sentence*, and a recorder answers *who says one*.
mod record;
/// Re-exported with a glob, uniquely among the three submodules, and that is a
/// deliberate exception rather than a shortcut. `floor` and `textedit` name
/// their two or three items because each is a small, closed set with an
/// argument attached; this is the whole recording surface — twenty-odd
/// constructors that grow by one every time a verb learns to decline — and a
/// hand-written list of them here would be a second register of the same
/// family, free to fall out of step with the file it mirrors. Every name it
/// exports is `pub(crate) fn record_*` and nothing else, so the glob cannot
/// leak anything a reader would not expect to find under `decline::`.
pub(crate) use record::*;

/// ★★★ **The mode's refusal of a cut or a paste**, 2026-09-05. Its own file
/// rather than a function in `record` for `textedit`'s reason: it carries an
/// argument of its own — why a chord pushed blind at the gate obliges the
/// dispatcher to word every refusal it can now meet — and that argument would
/// be buried among twenty siblings.
mod clipboard;
/// Re-exported so the two call sites in `app::dispatch::clipboard` say
/// `decline::record_mode_refusal(..)`. `floor`'s rule.
pub(crate) use clipboard::record_mode_refusal;

/// See `decline/tests.rs`.
#[cfg(test)]
mod tests;
