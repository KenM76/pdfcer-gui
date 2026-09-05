//! # `app::save` — writing a copy of the open document to a file the operator
//! names
//!
//! The body of `file.save_copy`, and **the first time anything an operator
//! authored in this shell can leave the process**. Until 2026-08-14 the command
//! was registered, drawn on the File tab, drawn on the quick-access toolbar,
//! bound to `Ctrl+S`, printed "(Ctrl+S)" in its own tooltip — and had no
//! dispatch arm, so it traced `command-unimplemented` and did nothing. Every
//! feature this project has shipped (dimensions, markup, text marks, form
//! fills, page operations, a newly created document) was unwritable to disk.
//! That is `DEFECTS.md` D1's shape with the most consequential verb in an
//! editor behind it.
//!
//! ## 1. The save mode is **incremental**, and it was decided by a shipped
//!    promise rather than by this module
//!
//! `crate::text::commands::file_save_copy`'s tooltip has said, on an
//! operator-visible surface, since the day the command was registered:
//!
//! > *"…the edits are appended as an update so the previous version stays
//! > intact inside the file."*
//!
//! That sentence is a description of `EditSession::to_incremental_bytes` and of
//! nothing else. §7.5.6 incremental update: the original bytes are kept
//! verbatim and a new revision is appended after them, so the file carries both
//! and the previous one can be recovered.
//!
//! [`EditSession::to_full_bytes`] would satisfy every test that asks whether a
//! file was written and whether the edit is in it, and it would break the
//! promise in two ways an operator cannot see until it is too late: it
//! **destroys every existing digital signature** (§12.8.1) and it discards the
//! previous revision the sentence guarantees. So the choice is not this
//! module's to re-open. If a future change finds incremental genuinely
//! impossible for some input, the honest response is to **refuse and say so**,
//! not to fall back to a full rewrite — the engine already refuses a full
//! rewrite of a hybrid-reference file by name (`WriteError::HybridFullRewrite`)
//! and points at incremental as the supported path, which is the same posture
//! from the other side.
//!
//! ### ★★★ 1.1 …EXCEPT while a redaction is staged, 2026-09-05
//!
//! There is exactly one state in which this module writes a **single-revision
//! full rewrite** instead, and it is not a fallback: a staged redaction
//! (`pdfcer-core` `Pass 250.2`). All three save verbs go through
//! [`write_copy`], and [`write_copy`] asks `EditSession::has_pending_redaction()`
//! before it asks anything else.
//!
//! Three facts about that, each of which somebody will otherwise rediscover:
//!
//! 1. **It is not optional.** While a redaction is staged the engine refuses
//!    `to_incremental_bytes` **and** `to_full_bytes` by name
//!    (`WriteError::RedactionPending`, `pdfcer-core/src/edit.rs:8348` and
//!    `:8374`), because the un-redacted content is still live in the session.
//!    A build that did not route would not leak — it would simply stop being
//!    able to save at all, loudly, on every verb.
//! 2. **It suspends §1's promise, so §1's promise gets a sentence.** The
//!    previous revision is *deliberately* not carried into the output — that is
//!    the whole of engine rule R35 — and the operator has been taught by
//!    `file.save_copy`'s own tooltip to expect the opposite.
//!    [`crate::text::redact::saved_applying_redaction`] is where he is told.
//! 3. **The staging survives the save.** `save_applying_redaction` takes
//!    `&self`; it neither mutates the session nor clears the flag. So the next
//!    save applies the redaction again, the ordinary modes stay refused, and
//!    the document on screen still shows the content — which is stated in the
//!    same sentence, because *"I saved it, so it is done"* is exactly the
//!    assumption this must not let stand.
//!
//! ## 2. Why this needs none of [`crate::app::actions`]' four-step protocol
//!
//! Every other document verb in this shell goes through `vector_edit`: cancel
//! the render worker, mutate through `Arc::get_mut`, bump `edit_epoch`, drop
//! the cached texture. **A save does none of those and must not.**
//!
//! The reason is one word in the engine's signature:
//!
//! ```text
//! pub fn to_incremental_bytes(&self, options: &SaveOptions)
//!                             ^^^^^
//! ```
//!
//! `&self`, not `&mut self`. The dirty set is computed at save time from the
//! current state, so writing is a **read** of the session. That has three
//! consequences worth stating, because each is a step somebody would otherwise
//! add out of symmetry:
//!
//! | step `vector_edit` takes | why a save must not |
//! |---|---|
//! | `RenderWorker::cancel_and_wait` | it exists only to make `Arc::get_mut` succeed. A save never calls it, so the render worker may keep its clone and a save during a raster costs the operator nothing |
//! | `Arc::get_mut` | there is nothing to mutate; `&*doc.session` is enough |
//! | bump `edit_epoch` | see §3 — it would throw away the decomposition, the page-text cache and any live disclosure to record that **nothing changed** |
//! | `page_texture = None` | the page on screen is still correct; a re-raster would be work with no cause |
//!
//! ## 3. ★ What happens to the edit epoch and the dirty state: **nothing**,
//!    in both directions
//!
//! This is the part that is easy to get wrong in the tidy-looking direction, so
//! it is written down as four separate claims.
//!
//! ### 3.1 `edit_epoch` is not bumped
//!
//! `OpenDoc::edit_epoch` means *"which revision is on the screen"*. It is the
//! staleness key for the canvas selection, the object decomposition, the font
//! inventory, the per-page text cache, the form-fill disclosure and the edit
//! disclosure. A save changes none of those — the document in memory after a
//! save is byte-for-byte the document that was there before it — so bumping the
//! epoch would dissolve the operator's selection, discard several caches, and
//! silently retire a rule-4 disclosure sentence they may not have read yet, all
//! to record an event that changed nothing they can see.
//!
//! ### 3.2 `edit_epoch` is not **reset** either, and this one has teeth
//!
//! The tempting move is `doc.edit_epoch = 0` — "the edits are saved now". It
//! would be wrong twice.
//!
//! It is wrong in principle: the edits are saved *somewhere else*. The document
//! open in front of the operator is still exactly as unsaved as it was, at its
//! own path.
//!
//! And it is wrong concretely, with a live consumer: `dialogs::ocr`'s
//! `preflight` refuses recognition when `doc.edit_epoch != 0`, because
//! `add_ocr_layer` writes an incremental revision over the document **as
//! opened** and would therefore silently omit the operator's edits. Zeroing the
//! epoch here would turn that named refusal off and hand the operator a
//! recognised copy with their work missing from it — a plausible, working,
//! wrong file, which `pdfcer-core`'s writer module calls the worst possible
//! failure shape for this subsystem.
//!
//! ### 3.3 `save_pending()` still answers `false`
//!
//! It asks *"is a save **in flight**"*, and gates New, Open and Close on the
//! answer. [`save_copy`] is synchronous: it is entered and finished inside one
//! `PdfcerApp::apply` call and no frame is drawn while it is part-way through,
//! so there is no moment at which the predicate could be true. See
//! `crate::app::files`' header, which carries the corrected rule, and
//! `PdfcerApp::save_pending`, which carries it beside the code.
//!
//! ### 3.4 `path` and `origin` do not move: Save a **copy** is not Save **as**
//!
//! A created document saved to `D:\jobs\sheet.pdf` is still called
//! `Untitled 1.pdf` afterwards, still has `Origin::Created`, still gets no
//! Recent row, and still stores no per-document page-display or guide
//! preferences. That is Inkscape's *Save a Copy* exactly — the one reference
//! application that has this verb — and it is the difference between it and
//! *Save As*, which is a different command that does not exist here.
//!
//! `OpenDoc::origin`'s own doc comment anticipated this and said *"a created
//! document that gains a file gains it through a save"*. It is worth being
//! precise about which save: **this is not that one.** A `file.save_as` would
//! write `path` and `origin`; this deliberately does not, and adding it here
//! would rename the operator's open document out from under them because they
//! asked for a copy.
//!
//! ## 4. Where the picker runs, and why the command raises an [`Action`]
//!
//! `crate::app::files::pick_save_path` documents a **frame-timing
//! requirement**: it opens a native modal, and a native modal opened from
//! inside an `egui` layout closure blocks the frame it is being drawn in,
//! leaving a half-painted window behind a dialog. `dialogs::ocr` honours it
//! with a `save_requested` flag consumed after `Window::show` returns.
//!
//! `file.save_copy` honours it by **raising
//! `crate::app::actions::Action::SaveCopy`** and doing the work in the apply
//! phase, which is step 3 of the frame — after every panel, the canvas, the
//! docks, the find bar and the dialogs have closed. That is the strongest
//! position available, and it is *also* the actions-not-mutations invariant
//! (`PROJECT_PLAN.md` §3) satisfied by the same line, which is why there was no
//! trade to make.
//!
//! It is deliberately **not** `file.open`'s shape. That arm calls the picker
//! *during dispatch* and pushes the answer, on the argument that the picked
//! path is an operand which cannot be re-derived after the frame. The argument
//! is sound for Open and does not apply here — a save has no operand to carry;
//! the suggestion is derived from the document — and the timing is the other
//! way round: **dispatch is not always outside a layout closure.**
//! `PdfcerApp::central` dispatches the canvas's context-menu tokens from inside
//! `egui::CentralPanel::show`, so an arm that opens a modal is one context-menu
//! entry away from doing it mid-layout. See the report accompanying this work.
//!
//! ## 5. What an operator sees
//!
//! **On success: the dialog they drove, and the file where they put it.** No
//! sentence is added. A save-a-copy is the one operation in this shell whose
//! whole product is visible in the operating system's own file browser, at a
//! path the operator typed a moment earlier; a status line saying so would
//! narrate what they just did.
//!
//! **On failure: a worded decline**, through `crate::app::status::decline` —
//! the surface built for *"this did not happen"*. Silence here is the exact
//! failure this project was founded on: the operator names a path, presses
//! Save, and no file appears. The engine's own reason goes to the trace, not to
//! the bar; `check-ui-strings.sh`'s exclusion 3 says in as many words that a
//! `Display` impl "is not permission to route UI text through an error type".
//!
//! **On cancel: nothing at all**, which matches `crate::app::files::raise`'s
//! ruling for a dismissed Open — the operator changed their mind, and that is a
//! complete and correct outcome that must not put a line anywhere.
//!
//! ## 6. ★★★ What a save says about a DIGITAL SIGNATURE, added 2026-08-28
//!
//! Until then: nothing, on any surface, before or after, on any of the three
//! write paths this module owns. `pdfcer-core` exposes
//! `EditSession::signature_impact_of_save` and `EditSession::changes_structure`
//! written specifically so a front end could answer the question, and this
//! shell called neither. A structural edit followed by `Ctrl+S` wrote a
//! revision over a signed document and said nothing about it.
//!
//! The whole design lives in [`crate::dialogs::signature`]; what belongs in
//! this header is the half this module performs:
//!
//! | when | who | what |
//! |---|---|---|
//! | **before** an invalidating save | `crate::app::actions::apply`, through `DialogsState::ask_signature` | a window, which the operator may cancel — so [`save_in_place`] and [`save_copy`] are simply **not reached** |
//! | **after** any successful write | [`signature_note`], here | one sentence on the disclosure row, for a signed document only |
//!
//! Three properties of that split are worth stating where the writes are:
//!
//! 1. **An unsigned document is untouched by all of it.** The engine's own
//!    instruction for `SignatureImpact::None` is *"a front end should add no
//!    friction at all"*, and that is the case for nearly every document this
//!    operator opens.
//! 2. **The note is recorded on the write paths rather than in the action
//!    arms**, so `crate::app::lifecycle::resume_after_unsaved` — which calls
//!    [`save_copy`] directly, from inside an already-answered question — gets
//!    it without a fourth call site having to remember to.
//! 3. **The compacted path is not routed through any of it.** It is a full
//!    rewrite, `crate::dialogs::compact` already discloses the loss before its
//!    picker opens, in stronger words than this module is entitled to, and
//!    those words are correct there and only there. See
//!    `crate::dialogs::signature`'s §4 and §5 — including the trap that
//!    `SignatureImpact::documentation_basis` cannot see the `SaveMode` and so
//!    must not be asked about a rewrite.

use std::path::{Path, PathBuf};

use pdfcer_core::writer::{SaveReport, WriteError};

use crate::app::files::{self, Picked};
use crate::app::state::OpenDoc;
use crate::dialogs::signature::{Disclosure, impact_of_saving};

/// **The sentence this save owes about the document's digital signatures, if
/// it owes one.**
///
/// `None` for the overwhelmingly common case — a document with no signature —
/// and that is the engine's instruction rather than an optimisation:
/// `SignatureImpact::None` means *"Nothing to say, and a front end should add
/// no friction at all."* A save of an unsigned drawing costs one census walk,
/// which is bounded (`signature::MAX_FIELD_TREE_NODES`), and produces no
/// string, no allocation past the census and no row on the bar.
///
/// # ★★ It is computed BEFORE the bytes are written, and the reason is the
/// engine's contract rather than caution
///
/// `EditSession::signature_impact_of_save`'s own documentation: *"A front end
/// asks this **immediately before Save**, not at edit time: per §11.1 the
/// dirty set is a diff computed at save time."* Asking after the write would
/// in fact return the same answer today — a save changes nothing about the
/// session, which is §3 of this module's header and is asserted by
/// `saving_a_copy_changes_nothing_about_the_open_document` — but it would be
/// asking a question the engine documented as a *pre*-save question, and the
/// day that stops being harmless is the day a save starts touching the
/// session.
///
/// # The two sentences, and why they are not one
///
/// | [`Disclosure`] | sentence | what it says |
/// |---|---|---|
/// | `Silent` | none | there is no signature |
/// | `NoteAfterSaving` | [`crate::text::signature::preserved_note`] | the bytes each signature covers are unchanged — **paired with** the fact that this is not the same as still being valid |
/// | `WarnBeforeSaving(_)` | [`crate::text::signature::invalidated_note`] | pdfcer reports this save as invalidating |
///
/// The third row is a **receipt for something the operator was asked about**
/// on the two routes that ask — and on `crate::app::lifecycle`'s
/// resume-after-unsaved route it is the whole of what they are told, because
/// that route deliberately does not stack a second window on an
/// already-answered question. `crate::dialogs::signature`'s §7 carries the
/// argument. Repeating it here for the routes that did warn is cheap and is
/// the right trade: a confirmation dismissed quickly is a confirmation not
/// read, and the bar is where every other consequence of a write is recorded.
fn signature_note(doc: &OpenDoc) -> Option<String> {
    let (disclosure, count) = impact_of_saving(doc);
    match disclosure {
        Disclosure::Silent => None,
        Disclosure::NoteAfterSaving => Some(crate::text::signature::preserved_note(count)),
        Disclosure::WarnBeforeSaving(_) => Some(crate::text::signature::invalidated_note(count)),
    }
}

/// **Ask where the copy goes, write it there, and say what happened.**
///
/// The whole of the `Action::SaveCopy` arm. Takes `&OpenDoc` rather than `&mut`
/// — see §2: the engine's write verb takes `&self`, so nothing about the open
/// document changes and the type says so.
///
/// The three answers the picker can give are treated exactly as
/// `crate::app::files::raise` treats them for Open, and for the same reasons:
/// a path is the ordinary case; a cancel is a complete outcome and is silent;
/// and *this build cannot ask* is a **build** limitation rather than an
/// operator choice and gets a trace naming the gap, because a reader of a trace
/// from a machine they cannot see most needs that told apart from "the click
/// never arrived".
///
/// Not reachable from a unit test, and that is [`crate::app::files`]' rule 3
/// applied to the save side: with `PDFCER_DIAG_SAVE_PATH` unset this opens a
/// **real modal dialog** and blocks until a human dismisses it, so a
/// `cargo test` that applied `Action::SaveCopy` would hang the suite behind an
/// invisible window. Everything below the picker is therefore reachable with
/// the answer supplied directly — [`write_copy`] and [`suggested_path`] are
/// both pure of any dialog and both are tested — and the join is proven by
/// `tools/ui-verify`'s `save_copy_round_trip`, which answers the dialog through
/// the seam and then re-opens the file that came out.
///
/// ★ **Returns whether a file was actually written**, added 2026-08-19.
///
/// It returned `()` until then, and the reason it now answers is a caller that
/// did not exist: `crate::dialogs::unsaved`'s *Save a copy…* button, which
/// **only proceeds with the close or open it is standing in front of if the
/// save succeeded.**
///
/// The three false cases are not the same thing and it is worth saying so,
/// because a future hand will be tempted to distinguish them:
///
/// * the operator **cancelled the picker** — they changed their mind
///   mid-transaction, and the least surprising reading is *"leave my document
///   alone"*;
/// * the picker is **unavailable** in this build;
/// * the write **failed** — already reported on both channels by
///   [`write_and_report`].
///
/// All three answer `false` and all three mean the same thing to that caller:
/// *do not destroy the document*. Returning a richer type so it could tell
/// them apart would invite a branch that proceeded on one of them, and there
/// is no member of that set it would be safe to proceed on.
/// **Save As** — write the document somewhere new, and *keep editing THAT file*.
///
/// # ★★★ Why this is a different command from [`save_copy`], and not a flag
///
/// Operator, 2026-09-02, `OPERATOR_REQUESTS.md` O95:
///
/// > *"we need a Save As option so that we are then making edits in the save as
/// > file instead of the original just like other programs have it."*
///
/// **The second half is the whole request.** `save_copy` already writes the
/// bytes wherever he points it — what it does not do is *move the document*.
/// The session stays bound to the original, so the next `Ctrl+S` goes straight
/// back to the file he was trying to leave, which is the opposite of what he
/// asked for and is a way to overwrite something by doing nothing wrong.
///
/// So the two are different **acts**, and every editor he uses has both:
///
/// | | writes | afterwards you are editing |
/// |---|---|---|
/// | **Save a copy** | a snapshot, somewhere else | **the original** |
/// | **Save As** | the document, somewhere else | **the new file** |
///
/// ★★ Keeping both is deliberate. *Save a copy* is the right verb for "send
/// this to somebody" and collapsing it into Save As would take that away; a
/// single command with a checkbox would make the destructive difference a
/// setting nobody reads.
///
/// # What this function does NOT do, and why the caller does it
///
/// It does not touch [`OpenDoc`]. It picks a path, writes there, and reports
/// **where it wrote**. The rebinding — `doc.path`, `doc.saved_epoch`, the tab
/// label, the window title, the recent list — happens in the caller, which
/// holds `&mut` and can see all of them.
///
/// ★ That split is not tidiness. Rebinding is the dangerous half: a document
/// whose path moved while its bytes did not is a document whose next `Ctrl+S`
/// writes the wrong file. Keeping the write pure and the rebinding in one
/// visible place means there is exactly one statement to read to know when the
/// binding moves.
///
/// # Returns
///
/// `Some(path)` when the bytes reached that path, `None` when the operator
/// cancelled, the picker is unavailable, or the write failed — the same three
/// outcomes [`save_copy`] flattens to `false`, and flattened here for the same
/// reason: **there is no member of that set on which it would be safe to
/// rebind the document.**
///
/// # ★★ The undo stack survives, and that is a decision
///
/// Nothing is closed and nothing is reopened, so the session, its history and
/// the operator's selection all continue. That is what every other editor does
/// and it is what he would expect: Save As is a save, not a round trip. The
/// alternative — write, close, reopen the new file — would silently discard
/// every undo step, which is a data loss with no warning attached to it.
pub fn save_as(doc: &OpenDoc) -> Option<std::path::PathBuf> {
    let suggested = suggested_path(doc);
    match files::pick_save_path(&suggested, crate::text::files::save_as_dialog_title()) {
        Picked::Path(target) => {
            if write_and_report(doc, &target) {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // ★★ The OLD path is on the line as well as the new one,
                    // and that is the point of tracing this at all. "The
                    // document moved" and "a copy was written" produce the
                    // same `save-copy` line today; only the pair says which
                    // file the next Ctrl+S will reach.
                    format!(
                        "save-as from={} to={}",
                        doc.path.display(),
                        target.display()
                    )
                });
                Some(target)
            } else {
                None
            }
        }
        Picked::Cancelled => None,
        Picked::Unavailable => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "save-as-unavailable reason=no-picker-in-this-build".to_owned()
            });
            None
        }
    }
}

pub fn save_copy(doc: &OpenDoc) -> bool {
    let suggested = suggested_path(doc);
    match files::pick_save_path(&suggested, crate::text::files::save_copy_dialog_title()) {
        Picked::Path(target) => write_and_report(doc, &target),
        // A cancelled save is a complete, correct, uninteresting outcome.
        Picked::Cancelled => false,
        Picked::Unavailable => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "save-copy-unavailable reason=no-picker-in-this-build".to_owned()
            });
            false
        }
    }
}

/// **Does the active document have a real file behind it?**
///
/// The question `file.save` asks before deciding between saving in place and
/// opening the picker, and it is answered by **asking the file system**, not by
/// inspecting the path.
///
/// # Why "does this file exist" and not "is this path absolute" or a flag
///
/// A blank document created in this shell is given a path like `Untitled 3.pdf`
/// — a name with no directory, and no file anywhere. A document opened from
/// disk has a path that names a file that is there. Those are the two cases,
/// and *"is there a file at this path"* separates them exactly.
///
/// The alternatives are worse in specific ways. A `created_here: bool` flag is
/// a second source of truth that has to be maintained through save-a-copy,
/// re-open and the document-tab machinery, and the failure mode when it drifts
/// is **writing over the wrong file**. "Is the path absolute" answers a
/// different question and would treat a relative path to a real file as
/// unsaved.
///
/// # The race, acknowledged rather than defended against
///
/// A file can be deleted between this check and the write. If that happens the
/// rename in [`save_in_place`] creates the file, which is the same thing every
/// other editor does and is the harmless direction: the operator gets their
/// document back where they expect it. The dangerous direction - writing over
/// something they did not choose - cannot be reached from here, because a path
/// that names no file goes to the picker.
#[must_use]
pub fn has_a_file(doc: &OpenDoc) -> bool {
    doc.path.is_file()
}

/// ★★★ **Does this document have edits that are not on disk?** — the one
/// question, in the one place.
///
/// `OPERATOR_REQUESTS.md` row **O65**, the operator:
///
/// > *"I can't edit it unless I save the document first, at which point it
/// > closes the document after saving."*
///
/// # What actually happened, because Save does NOT close the document
///
/// A successful in-place save records `doc.saved_epoch = doc.edit_epoch`, and
/// until today **nothing in production read that number**. It was written
/// once, traced once, and asserted in a single test. Every surface that asked
/// *"does this document have unsaved edits?"* asked a different question that
/// a save cannot answer:
///
/// | surface | asked | why a save could not clear it |
/// |---|---|---|
/// | `dialogs::unsaved::ask_for` | `edit_epoch == 0` | *"has anything EVER been edited"* — permanently true after the first edit |
/// | the document tab strip | `session.is_modified()` | the engine's *"differs from the BASE revision"*, and an incremental save takes `&self` so the base never moves |
/// | the close arm | `session.is_modified()` | same |
///
/// So after a perfectly good `Ctrl+S` the shell still believed the file was
/// dirty. The tab kept its unsaved marker, and the very next Close raised the
/// unsaved-edits question — whose only save button is *"Save a copy…"*, which
/// opens a picker and, on success, proceeds with the pending intent. From the
/// operator's chair: press a Save button, get asked for a filename, and watch
/// the document close. Exactly what he reported, arrived at without Save ever
/// closing anything.
///
/// ★ Driven, not inferred: with `PDFCER_DIAG_INVOKE="mode.edit,pages.rotate_right,file.save,file.close"`
/// the shipped build traces `save-in-place outcome=ok` → `save-epoch-recorded
/// epoch=1` → `diag-invoke id=file.close` → `unsaved-asked`, and no `close
/// slot=` line anywhere.
///
/// # Both halves are load-bearing
///
/// - **`session.is_modified()`** is the engine's precise *"differs from the
///   base revision"*, and it is what makes edit-then-undo come out **clean**.
///   A pure epoch comparison would call an undone edit dirty, because an undo
///   bumps the epoch like everything else.
/// - **`edit_epoch != saved_epoch`** is the only term that can see an
///   in-place save, for the reason above: `to_incremental_bytes` takes
///   `&self`, so the session's own answer cannot change when bytes are
///   written.
///
/// | state | `is_modified` | epochs differ | answer |
/// |---|---|---|---|
/// | opened, untouched | no | no | clean |
/// | edited | yes | yes | **dirty** |
/// | edited, then saved | yes | no | clean |
/// | edited, saved, edited again | yes | yes | **dirty** |
/// | edited, then undone | no | yes | clean |
///
/// # ★★ What this must NOT be confused with
///
/// [`save_pending`](crate::app::PdfcerApp::save_pending) asks *"is a save in
/// flight"*, which is a different question with a different consumer, and
/// `dialogs::unsaved`'s own header explicitly forbids gating Open / New /
/// Close on dirtiness through it.
///
/// And `saved_epoch` must never be reset to make an answer come out right:
/// `edit_epoch` is the cache key for the decomposition, the page text, the
/// texture and every live rule-4 disclosure. Two numbers, one question each.
/// # ★★★ 2026-09-04 — the third term, and the silent loss it closes
///
/// **Added on 2026-09-04 as `session.has_applied_redaction()`; CORRECTED on
/// 2026-09-05 to `session.has_pending_redaction()`.** Both halves of that are
/// recorded rather than the survivor alone, because the reason the term exists
/// outlived the verb it was first written against and would have been lost with
/// it.
///
/// **What it was for.** `EditSession::apply_redactions` (`Pass 250.1`) applied
/// a redaction into the session by **collapsing** it: the redacted bytes became
/// the session's new base and the edit and undo stacks were emptied. So
/// immediately afterwards `is_modified()` answered **`false`** — correctly, on
/// its own terms, because the session no longer differed from its base. With
/// two terms this predicate therefore answered **clean** on a document whose
/// most consequential edit had not been written: no tab marker, no question on
/// Close, no question on Quit. Apply a redaction, close the document, and it is
/// gone with nothing asked.
///
/// **Why the verb changed.** This shell no longer collapses. `Pass 250.2`'s
/// [`crate::redact::stage_into_session`] leaves base, overlay and undo
/// untouched and sets one flag, so `has_applied_redaction()` is now **`false`
/// for the life of every session this shell creates** — a term that can never
/// be true, which is worse than an absent one because it reads as a guard.
///
/// **And the hole it would have left is a real, reachable state**, not a
/// theoretical one: open a drawing that already carries `/Redact` marks from an
/// earlier session, press *Review & apply*, choose *this document*. Nothing is
/// edited — the marks were already in the file — so `is_modified()` is `false`;
/// the funnel bumps `edit_epoch`, so the epochs differ. With two terms that is
/// **clean**, and the operator closes a document with an armed removal on it
/// and is asked nothing.
///
/// `session.has_pending_redaction()` is the term that sees it. It is an OR
/// beside `is_modified()` rather than a replacement for it; the
/// `edit_epoch != saved_epoch` term is what turns it off again once the
/// redaction has actually been written.
///
/// ★ **It is strictly better behaved than the term it replaces**, in the one
/// way that matters. `has_applied_redaction()` was permanently sticky — once
/// true, true for the life of the session — so `redact → save → edit → undo`
/// answered **dirty** on a document that matched its file, costing a spurious
/// prompt that could never be cleared. `has_pending_redaction()` is turned off
/// by [`crate::redact::cancel_staged_redaction`], so a document whose staging
/// the operator called off goes genuinely clean again.
///
/// ⚠ **It is still not a save gate**, and the distinction moved rather than
/// went away. Saving while a redaction is staged is not merely permitted, it is
/// the *only* way the redaction ever happens — [`write_copy`] routes it through
/// `save_applying_redaction`. This predicate asks whether there is something to
/// save, never whether saving is permitted.
#[must_use]
pub fn has_unsaved_edits(doc: &OpenDoc) -> bool {
    (doc.session.is_modified() || doc.session.has_pending_redaction())
        && doc.edit_epoch != doc.saved_epoch
}

/// ★★★ **Save. In place. The one every other program has.**
///
/// The operator, 2026-08-20:
///
/// > *"can I please have a save button like every other program in existence
/// > has? We're on week two of this and just have a save as button."*
///
/// There is no defence. `Ctrl+S` was bound to [`save_copy`], which asks where
/// to put it every single time, and overwrite-in-place had been written down as
/// *"an operator scope decision"* and then been nobody's problem. That is the
/// same failure as `Ctrl+P` never being bound and the text caret never having
/// an index: **the basics were never audited as basics**, because every test
/// asked *"does the thing I built work?"* and nothing asked *"does the thing
/// everyone expects exist?"*.
///
/// # ★★ It writes to a TEMPORARY FILE and renames, and that is not ceremony
///
/// `std::fs::write` truncates the target and then streams into it. Everything
/// between those two acts is a window in which **the operator's only copy of
/// the document is a partial file** - and the payload here is the whole PDF,
/// which for a CAD sheet is megabytes and takes real time. A crash, a full
/// disk, or a sync client holding a lock in that window destroys the original
/// and leaves nothing to fall back to.
///
/// Save-as does not have this problem, because its target is a file that did
/// not exist. Save-in-place is the one verb in this application that can
/// destroy the operator's work, so:
///
/// 1. write the new bytes to `<name>.pdfcer-tmp` beside the target,
/// 2. `fs::rename` it over the target — which either happens or does not,
/// 3. on any failure, remove the temporary and leave the original untouched.
///
/// This is the same lesson as the portable-build packager, which destroyed the
/// fallback build **twice in one day** by clearing a directory before filling
/// it, and whose message asserted nothing had been replaced both times. Written
/// up in `D:/dev/rag/rust/`. The rule that came out of it applies here without
/// modification: *never destroy the current good state until the replacement is
/// fully materialised somewhere else on the same volume.*
///
/// The temporary sits **beside the target**, deliberately, not in the system
/// temp directory: a rename across volumes is a copy, and a copy has the unsafe
/// window back again.
///
/// # Why it returns the same `bool` as [`save_copy`]
///
/// So `crate::dialogs::unsaved`'s buttons can treat the two identically: the
/// question that dialog is asking is *may I now destroy this document*, and
/// only a successful write answers yes. See [`save_copy`]'s own note on why
/// distinguishing the failure modes there would invite a caller to proceed on
/// one of them.
///
/// # What it does NOT do
///
/// Fall back to Save-as. A document opened from a path always has one, and a
/// blank document created in this shell is given a path when it is first saved
/// — the dispatcher routes that case to [`save_copy`] before reaching here,
/// because *"where does this go?"* is a question only the operator can answer
/// and answering it silently is how a file lands in a folder nobody expected.
pub fn save_in_place(doc: &OpenDoc) -> bool {
    let target = doc.path.clone();
    let temporary = target.with_extension("pdfcer-tmp");
    // ★ Asked before a byte moves — see [`signature_note`]'s ★★ for why the
    // engine documented this as a pre-save question, and why asking after
    // would return the same answer today and be wrong on principle.
    let signature = signature_note(doc);

    // Step 1 - materialise the whole replacement somewhere else on the same
    // volume. A failure here has touched nothing the operator owns.
    //
    // ★★ On a staged redaction the "replacement" is a single-revision full
    // rewrite with the marked content gone (§1.1), and the temp-then-rename
    // below matters more here than anywhere else in this module: the target is
    // the operator's own document, and it is the last remaining copy of the
    // content being removed.
    let written = match write_copy(doc, &temporary) {
        Ok(written) => written,
        Err(error) => {
            let _ = std::fs::remove_file(&temporary);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("save-in-place outcome=failed stage=write detail={error:?}")
            });
            // The operator-visible half, and the same sentence Save-a-copy
            // uses: a write that produced no file and no sentence is
            // indistinguishable from a control that does nothing.
            crate::app::status::decline::record_save_failure();
            redaction_refusal_note(doc, &error);
            return false;
        }
    };

    // Step 2 - the atomic act. A rename either happens or does not; on Windows
    // it fails outright if the target is open, which is precisely the guarantee
    // a truncating write does not give.
    if let Err(error) = std::fs::rename(&temporary, &target) {
        let _ = std::fs::remove_file(&temporary);
        crate::app::status::decline::record_save_failure();
        let _ = error;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // The stage is in the line because the two failures mean different
            // things to whoever reads it: `write` means the bytes never
            // materialised, `rename` means they did and the swap was refused -
            // overwhelmingly "the file is open in another program".
            "save-in-place outcome=failed stage=rename".to_owned()
        });
        return false;
    }

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("save-in-place outcome=ok path={:?}", target)
    });
    // ★ A receipt, not a celebration. The operator pressed a button and the
    // only observable change is that a marker disappeared from a tab - which is
    // a change you have to already know about to notice. One line naming the
    // file it went into, on the channel every other edit reports on.
    //
    // ★★ And, for a signed document, the sentence that receipt owes beside it.
    // `record_notes` rather than two `record_note` calls: the slot holds ONE
    // disclosure, so a second call replaces the first, and the sentence it
    // would have dropped would have been chosen by statement order rather than
    // by importance. The receipt leads because it answers *"did my save
    // happen"*, which is the question the operator actually pressed the button
    // to have answered; the signature sentence follows because it answers one
    // they did not know they had.
    let mut notes = vec![crate::text::files::saved_in_place(&target)];
    notes.extend(signature);
    crate::app::actions::record_notes(doc.edit_epoch, notes);
    // ★★★ …and, when the save performed a staged redaction, the sentence that
    // says so — recorded AFTER `record_notes`, deliberately, so it is the
    // disclosure that stands.
    //
    // `record_note` replaces what is in the slot, and the ordering here is a
    // choice about which fact wins when only one can be shown. "Saved to
    // sheet.pdf" answers *"did my save happen"*, which the operator can also
    // read off the tab marker disappearing. "The content is out of the file,
    // the window still shows it, and the removal is still armed" answers three
    // questions he has no other way to answer, on the one operation that cannot
    // be undone. Rule 4 decides it and it points at the second.
    if let Written::RedactionApplied(report) = &written {
        redaction_receipt(doc, &target, report);
    }
    true
}

/// Write the copy and record the outcome on both channels.
///
/// Split from [`save_copy`] only so that the picker and the write are separable
/// in the reading as well as in the testing: everything here is what happens
/// *once a destination exists*, and it is the half that has a failure worth
/// wording.
/// Returns whether the bytes reached the disk — see [`save_copy`] for the
/// caller that turns that answer into *may this document be destroyed*.
fn write_and_report(doc: &OpenDoc, target: &Path) -> bool {
    // Before the write, for [`signature_note`]'s reason.
    let signature = signature_note(doc);
    match write_copy(doc, target) {
        // ★★★ The staged-redaction save — §1.1. A different event with
        // different fields, so a different trace line and a different sentence.
        Ok(Written::RedactionApplied(report)) => {
            redaction_receipt(doc, target, &report);
            if let Some(note) = signature {
                crate::app::actions::record_note(doc.edit_epoch, note);
            }
            true
        }
        Ok(Written::Ordinary(report)) => {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // ★ `appended=` beside `bytes=` on `HANDOFF.md` §2's own
                    // advice about the ink trail: a build that wrote a plain
                    // copy of the base file — no revision appended, the
                    // operator's edits silently absent — produces a file that
                    // opens, has the right page count and looks correct, and
                    // its trace line would be identical but for this one
                    // field. `identical=` is the same fact from the other
                    // side and is `true` exactly when nothing was edited.
                    //
                    // `epoch=` says WHICH revision was written, which is the
                    // only way a reader of a trace can tell a save that
                    // captured the operator's last edit from one that ran a
                    // frame too early.
                    //
                    // `path` is Debug-quoted, exactly as `open`'s is. A Windows
                    // path routinely contains a space, and a consumer splitting
                    // this line into `key=value` pairs would otherwise read
                    // `Files\a.pdf` as a field name and lose every field after
                    // it. `tools/ui-verify`'s parser honours double quotes;
                    // nothing else in the line needs them.
                    "save-copy path={:?} bytes={} appended={} objects={} verbatim={} \
                     reserialized={} promoted={} deleted={} identical={} delinearized={} \
                     epoch={} origin={:?}",
                    target,
                    report.bytes_written,
                    report.bytes_appended,
                    report.objects_written,
                    report.objects_verbatim,
                    report.objects_reserialized,
                    report.promoted.len(),
                    report.objects_deleted,
                    report.byte_identical,
                    report.delinearized,
                    doc.edit_epoch,
                    doc.origin,
                )
            });
            // ★★ The one sentence a successful save-a-copy is allowed to put
            // on the bar, and §5's *"no sentence is added"* ruling is not
            // being overturned by it.
            //
            // That ruling is about **narrating the act**: a status line saying
            // "a copy was written" would describe something whose whole
            // product is already visible in the operating system's own file
            // browser, at a path the operator typed a moment earlier. This is
            // not that. It is a fact about the **integrity of the file they
            // now have**, which appears nowhere — not on the canvas, not in
            // Explorer, and not in this shell's Signatures panel, which
            // reports what a document carries rather than what a save did to
            // it. Rule 4 governs, and it points the other way from §5.
            //
            // It is `None` for every unsigned document, so the common case
            // still adds nothing at all.
            if let Some(note) = signature {
                crate::app::actions::record_note(doc.edit_epoch, note);
            }
            true
        }
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("save-copy-failed path={target:?} detail={error}")
            });
            // The operator-visible half. See §5: a write that produced no file
            // and no sentence is indistinguishable from a control that does
            // nothing.
            crate::app::status::decline::record_save_failure();
            redaction_refusal_note(doc, &error);
            // Mutually exclusive with the line above — one `SaveError` value
            // reaches both, and each is silent for the other's variant. See
            // `page_tree_refusal_note` for why it is a second function rather
            // than a second arm.
            page_tree_refusal_note(doc, &error);
            false
        }
    }
}

/// **The sentence a staged-redaction save owes, and why it is not the ordinary
/// receipt.**
///
/// §1.1 item 2 and item 3, in one place so that all three save verbs get the
/// identical wording — and so that a fourth verb added later gets it by calling
/// one function rather than by remembering three facts.
///
/// The trace line is separate from `save-copy`'s on purpose. That line's fields
/// (`appended=`, `verbatim=`, `identical=`) are properties of an incremental
/// update and every one of them would be a fabrication here; this line carries
/// what a full-rewrite removal actually did. A reader of a trace must be able to
/// tell the two events apart, and `HANDOFF.md` §2's ink-trail rule is that they
/// are told apart by fields, not by hoping.
fn redaction_receipt(doc: &OpenDoc, target: &Path, report: &pdfcer_core::redact::RedactionReport) {
    let residuals = crate::redact::residual_count(report, None);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // `path` is Debug-quoted, exactly as `save-copy`'s is: a Windows
            // path routinely contains a space.
            //
            // ★ `still_staged=true` is unconditional and is the field worth
            // having. `save_applying_redaction` takes `&self` and does not
            // clear the flag, so the removal is armed again the instant this
            // returns — and a build that had started clearing it would emit an
            // otherwise identical line.
            "redact-saved path={:?} marks={} pages={} glyphs={} residuals={} epoch={} \
             still_staged=true",
            target,
            report.marks_applied,
            report.pages_redacted,
            report.glyphs_removed,
            residuals,
            doc.edit_epoch,
        )
    });
    crate::app::actions::record_note(
        doc.edit_epoch,
        crate::text::redact::saved_applying_redaction(
            &target.file_name().map_or_else(
                || target.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            ),
            report.marks_applied,
            report.pages_redacted,
            residuals,
        ),
    );
}

/// **The extra sentence a save refused *because of a staged redaction* owes.**
///
/// `record_save_failure` puts *"the file could not be written"* on the bar,
/// which is right for a full disk and wrong here: nothing is wrong with the
/// disk, the document, or the operator's folder. What is wrong is that a
/// removal is armed over marks that no longer exist, and the remedy is a
/// control in a dialog he would have no reason to open.
///
/// ★ It is added **beside** the failure rather than instead of it, on
/// `crate::text::redact::save_kept_pending_marks`'s standing reason: the save
/// genuinely did not happen, and replacing that fact with an explanation would
/// leave an operator unsure whether a file appeared.
///
/// ★ Silent for every other [`SaveError`]. A missing folder and a broken
/// provenance span have their own sentence already and do not want a second.
fn redaction_refusal_note(doc: &OpenDoc, error: &SaveError) {
    if let SaveError::RedactionRefused { refusal } = error {
        crate::app::actions::record_note(
            doc.edit_epoch,
            crate::text::redact::save_refused_message(refusal),
        );
    }
}

/// **The extra sentence a save refused by the page-tree guard owes** —
/// 2026-09-05.
///
/// [`redaction_refusal_note`]'s shape exactly, for its reasons, one variant
/// along: `record_save_failure` puts *"the copy was not written — check that
/// the folder exists and can be written to"* on the bar, which is right for a
/// full disk and wrong here. Nothing is wrong with the disk, the folder, or
/// anything the operator did. What is wrong is that the document no longer
/// agrees with itself about how many pages it has, and a file written from it
/// would open elsewhere with blank pages in it.
///
/// ★ Added **beside** the failure rather than instead of it, on the same
/// standing reason [`redaction_refusal_note`] gives: the save genuinely did not
/// happen, and replacing that fact with an explanation would leave an operator
/// unsure whether a file appeared.
///
/// ★ A **separate function** rather than a second arm inside
/// [`redaction_refusal_note`], and the reason is territorial rather than
/// aesthetic: that function belongs to the deferred-redaction work, which is in
/// flight in another track, and a shared `match` is a merge conflict in a file
/// two tracks are editing. The two are mutually exclusive by construction — one
/// [`SaveError`] value reaches both — so calling both costs nothing and neither
/// can overwrite the other's note.
///
/// ★★ **Which of the two sentences** is decided from structured data, never
/// from a message: [`crate::pagetree::Audit::root_disagreement`]. A root that
/// disagrees has an exact symptom the operator can verify (*n* blank pages at
/// the end); an interior-only disagreement does not, and promising one would be
/// the sneaky half of rule 4. See [`crate::text::pagetree`]'s header.
///
/// ★ Silent for every other [`SaveError`], exactly as its sibling is.
fn page_tree_refusal_note(doc: &OpenDoc, error: &SaveError) {
    let SaveError::PageTreeStale { audit } = error else {
        return;
    };
    let name = doc.path.file_name().map_or_else(
        || doc.path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    // ★ The choice of sentence — including the re-audit of the file on disk
    // that answers *"was it already like this when he opened it?"* — belongs to
    // `crate::pagetree`, beside the audit it reasons about. This function is
    // the delivery: one name, one audit, one note.
    let sentence = crate::pagetree::refusal_sentence(&name, audit, doc.stored_under());
    crate::app::actions::record_note(doc.edit_epoch, sentence);
}

/// **Serialize the open document as an incremental update and write it to
/// `target`.**
///
/// The one place bytes leave this shell for a document the operator authored,
/// and it is deliberately free of any dialog so that a test can drive it — see
/// [`save_copy`]'s note on why the surrounding function cannot be.
///
/// # The options, field by field, chosen rather than defaulted-into
///
/// `SaveOptions::default()`, and each of its three fields is a decision:
///
/// * **`producer: ProducerPolicy::Set`** — the default, and **ignored on this
///   path by construction**. `save_incremental` never rewrites `/Info` at all,
///   *"because doing so would mean appending a revision to a document the
///   operator did not change"*. So the value cannot affect a byte of the output
///   here, and the reason it is left at the default rather than set to
///   `Preserve` is that `Preserve` would advertise a policy that is not being
///   applied — a reader would take it as evidence that this path suppresses a
///   producer stamp, when what actually suppresses it is the writer.
/// * **`xref_entry_eol`** and **`trailing_eol`** — §7.5.4's and §7.2.3's
///   permitted end-of-line forms, both left at the values pdfcer has always
///   emitted. They apply to **newly written** cross-reference entries only, so
///   an incremental save changes nothing about the base revision's bytes
///   whichever way they are set. `SaveOptions::identity()`'s own documentation
///   draws the line this follows: *"an operator-facing save path applies the
///   persisted values explicitly; this one does not"* — and this shell has no
///   settings surface, so there is nothing persisted to apply. The day the
///   salvaged settings dialog lands and offers them, this is the call site that
///   reads it.
///
/// # ★ Why not `SaveOptions::identity()`
///
/// Because it names a **byte-comparison posture** for a harness, not an
/// operator-facing save. Its only difference from the default on this path is
/// the producer policy, which is ignored here — so it would change nothing and
/// would tell the next reader that this path is trying to be byte-identical to
/// its input, which is false the moment the operator has edited anything.
///
/// # Errors
///
/// [`SaveError`] — the engine refused to serialize, the redaction could not be
/// performed, the proof found removed text still present, or the file system
/// refused the write. They are kept apart because their remedies are: one is a
/// document pdfcer cannot express as an update, one is a staging the operator
/// has to cancel, one is a pdfcer defect, and one is a folder that does not
/// exist or cannot be written to.
fn write_copy(doc: &OpenDoc, target: &Path) -> Result<Written, SaveError> {
    // ★ Through the funnel, not `SaveOptions::default()`.
    //
    // Two settings ride on this — the cross-reference entry line ending and the
    // trailing newline — and both change the bytes of the file the operator is
    // about to receive. A bare `::default()` here would honour neither, which is
    // exactly what the old shell did: `xref_entry_eol`'s whole default was
    // changed on an operator ruling because a fixed form produced a
    // ten-thousand-byte diff on an unedited file, and the GUI could not honour
    // anything but the default anyway.
    //
    // The producer policy is the funnel's, which is `Preserve` — carried over
    // from `identity()` rather than chosen, because what pdfcer writes into
    // `/Producer` is a decision about attribution rather than about bytes and
    // no setting governs it.
    use crate::app::settings::SettingsExt;
    let options = doc.settings.save_options();

    // ★★★ THE FORK — 2026-09-05, `pdfcer-core` `Pass 250.2`. See §1.1.
    //
    // Asked of the SESSION rather than of a flag this module keeps, for
    // `has_a_file`'s reason applied to a different question: a second source of
    // truth about whether a removal is armed would drift, and the direction it
    // would drift in is a save that quietly wrote the un-redacted document.
    //
    // ★ There is no `else` that could fall back. While the flag is set, both
    // ordinary save modes return `WriteError::RedactionPending`, so a build
    // that did not fork here would not leak — it would stop being able to save
    // at all. That is the engine refusing rather than this module guarding, and
    // it is why the fork is a route rather than a gate.
    let (bytes, written, claims) = if doc.session.has_pending_redaction() {
        let (bytes, report) = crate::redact::save_applying_pending(&doc.session, &options)
            .map_err(|refusal| SaveError::RedactionRefused { refusal })?;
        // ★★ The claims that describe THESE bytes are the ones the removal that
        // produced them made — not the preview the staging recorded on the
        // document. The engine re-runs the removal over the current state, so
        // an operator who undid one mark of three between staging and saving
        // has a shorter list, and proving against the longer one would refuse a
        // legitimate save over a mark he deliberately took off.
        let claims = report.redacted_text.clone();
        (bytes, Written::RedactionApplied(report), claims)
    } else {
        let (bytes, report) = doc.session.to_incremental_bytes(&options)?;
        // The standing claim on the document. Empty on everything that has not
        // been staged, which is every ordinary save.
        (
            bytes,
            Written::Ordinary(report),
            doc.redaction_absence_claims.clone(),
        )
    };

    // ★★★ THE ABSENCE PROOF, between the bytes and the syscall — 2026-09-04,
    // and it is the shell's own, independent of the engine, on every save verb.
    //
    // `crate::redact::PreparedRedaction::write_to` makes this check one
    // statement from the write on the two destinations that produce a file
    // directly. The deferred destination — the default — arms the removal and
    // leaves the write to this function, minutes later, possibly after further
    // edits, through whichever save verb the operator reached for. So the proof
    // has to be made here or not at all, and "not at all" is the option our own
    // engine request ruled out in writing: *"the proof is not negotiable at
    // this end regardless of what the engine does."*
    //
    // ★★ What it is NOT. It is not a save gate on `has_applied_redaction()`,
    // which the engine asked us not to build. It is a sweep over the bytes,
    // whichever writer produced them, and it is expected to pass forever — a
    // check that is expected to pass is exactly the kind this project keeps
    // discovering was never wired, which is why `app::save::tests` falsifies
    // its bite rather than assuming it.
    //
    // ★ It costs nothing on an ordinary save: `redaction_absence_claims` is
    // empty on every document that has not been staged, and `prove_saved_bytes`
    // returns without decoding a single stream.
    //
    // ★ On the staged path it is deliberately the SECOND sweep of the same
    // bytes — `crate::redact::save_applying_pending` has already run one before
    // returning them. That is §2.2's rule twice rather than once, and the
    // reason is the same one it gives: the guarantee must not depend on how the
    // value was constructed, and this call site cannot see how it was.
    if let Err(survivors) = crate::redact::prove_saved_bytes(&bytes, &claims) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "save-refused-redaction-leak path={target:?} survivors={} of {}",
                survivors.len(),
                claims.len()
            )
        });
        return Err(SaveError::RedactionLeak { survivors });
    }

    // ★★★ THE STRUCTURAL GUARD, 2026-09-05 — the second proof this shell keeps
    // at this boundary, and it is here for the identical reason the first one
    // is. `crate::pagetree` carries the whole argument, the measured cost, and
    // the lesson; the three facts a reader of THIS function needs are:
    //
    // 1. **It is on the funnel, not on the delete-pages arm.** The defect is a
    //    writer's invariant, and `page-copy --cut` was measured producing
    //    byte-for-byte the same corruption as `delete-pages`. A guard on one
    //    verb would pass every other one straight through and would have to be
    //    remembered again for each verb added later.
    // 2. **It refuses; it does not repair.** `pdfcer-core` owns the page tree
    //    and is the only writer of `/Count`. A shell that patched the same key
    //    would be a second writer of one structure and the two would drift.
    // 3. **It cannot see the defect through this shell's own reader**, so it
    //    does not use it: `page_tree::pages` walks `/Kids` and reports a
    //    healthy 34-page document while the root still declares 36. The audit
    //    reads `/Count` raw and compares.
    //
    // ★ Ungated, and NOT free: 1.78 ms on his 1.8 MB drawing set, 3.51 ms on
    // the 129,758-object CAD sheet — which on the first of those is MORE than
    // `to_incremental_bytes` cost to build the bytes it is checking. Measured
    // rather than assumed, and the first draft of this comment guessed and was
    // wrong. It stays on every save because a few milliseconds is invisible
    // inside a gesture that opens a file dialog and writes megabytes to disk.
    // `crate::pagetree` §9 has the table and the argument for why a *"has the
    // page count changed this session?"* gate was rejected on correctness
    // grounds as well as on cost.
    let audit = crate::pagetree::audit_saved_bytes(&bytes);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // ★ `walked=` is the field worth having and is why this line is emitted
        // on the SUCCESS path too. A clean audit and an audit that never ran
        // produce the same `bad=0`, and this project's most-repeated failure
        // shape is a check that reported success having looked at nothing.
        format!(
            "save-pagetree walked={} pages={} declared={:?} levels={} bad={} nocount={} cycles={} deep={}",
            audit.walked,
            audit.reachable_pages,
            audit.declared_pages,
            // ★ `levels=` because a flat tree (2) CANNOT exhibit the defect
            // this guard is for, and a reader of a trace who does not know
            // that will read a clean line as evidence the writer is sound.
            audit.depth,
            audit.disagreements.len(),
            audit.nodes_without_count,
            audit.cycles,
            audit.too_deep,
        )
    });
    if !audit.is_consistent() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // The node ids, which the operator is never shown. A reader of a
            // trace needs to know HOW FAR UP the disagreement goes: the
            // immediate parent being correct and everything above it stale is
            // the signature of "there is no upward walk at all", which is a
            // different engine defect from "the walk stops one short".
            format!(
                "save-refused-pagetree path={target:?} nodes={:?}",
                audit
                    .disagreements
                    .iter()
                    .map(|d| (d.node.num, d.declared, d.reachable, d.root))
                    .collect::<Vec<_>>()
            )
        });
        return Err(SaveError::PageTreeStale { audit });
    }

    std::fs::write(target, &bytes)?;
    Ok(written)
}

/// **Which writer produced the bytes that reached the file, and what it
/// reported.**
///
/// Added 2026-09-05 with the staged-redaction route, and it is an enum rather
/// than `(SaveReport, Option<RedactionReport>)` because the two writers do not
/// both run: `EditSession::save_applying_redaction` produces no
/// [`SaveReport`] at all — it returns bytes and a
/// [`pdfcer_core::redact::RedactionReport`] — so a struct with both would have
/// to carry a fabricated one, and every field of a fabricated `SaveReport`
/// (`bytes_appended`, `byte_identical`, `promoted`) is a claim about a save
/// that did not happen in that shape.
///
/// ★ The trace lines differ for the same reason and that is the point. A reader
/// of a trace must be able to tell a save that appended a revision from one
/// that rewrote the whole document with content removed, and the two events
/// have no fields in common worth pretending they share.
#[derive(Debug)]
enum Written {
    /// The ordinary §7.5.6 incremental update — §1, and everything this module
    /// promises about the previous revision staying intact.
    Ordinary(SaveReport),
    /// A staged redaction, performed — §1.1. A single-revision full rewrite
    /// with the marked content gone, proven absent from the bytes twice before
    /// they reached the disk.
    RedactionApplied(pdfcer_core::redact::RedactionReport),
}

/// Why a save-a-copy produced no file.
///
/// Two variants rather than a `String`, on `crate::app::lifecycle`'s rule that
/// a branch is made on **structured error data, never by inspecting a message**
/// — and because the two are genuinely different facts about different
/// subsystems. Neither is worded to the operator separately today (the bar
/// carries one sentence for both; see §5), and keeping them apart is what makes
/// wording them separately a copy decision later rather than a re-plumbing.
#[derive(Debug)]
enum SaveError {
    /// `pdfcer-core` could not build the update. A refusal by name from the
    /// writer — a broken provenance span, a cross-reference form that cannot
    /// express an entry it was handed.
    Serialize(WriteError),
    /// The bytes were built and the file system refused them: the folder is
    /// gone, the path is read-only, the volume is full.
    Write(std::io::Error),
    /// ★★★ **The bytes were built and pdfcer found redacted text in them.**
    ///
    /// Added 2026-09-04 with the deferred redaction route. It means the save
    /// was refused *before any byte reached the file system*, and it is the one
    /// variant here that reports a **pdfcer defect** rather than a property of
    /// the document or of the disk: the engine's removal and pdfcer's own
    /// absence proof disagree about whether the content is gone.
    ///
    /// It is not expected to be reachable — see [`write_copy`]'s note on why
    /// the collapse makes every save mode safe by construction. It exists
    /// because a guarantee that depends on nobody ever changing the writer is
    /// not a guarantee.
    RedactionLeak {
        /// The strings that survived, for the trace. Never rendered verbatim:
        /// they are the redacted content, and putting them on screen to
        /// announce that they leaked would leak them again.
        survivors: Vec<String>,
    },
    /// ★★★ **A redaction is staged and the removal itself was refused, so no
    /// save of any kind could be built.**
    ///
    /// Added 2026-09-05. It is the one variant here the operator can reach by
    /// doing something perfectly reasonable, and the sequence is worth naming
    /// because it is the trap the deferred route brings with it:
    ///
    /// 1. mark, then *Review & apply* ▸ *this document* — the removal is armed;
    /// 2. **undo the marks**, which now works, and is the whole point of
    ///    `Pass 250.2`;
    /// 3. press `Ctrl+S`.
    ///
    /// The staging is still armed and there is nothing left to remove, so
    /// `save_applying_redaction` refuses with `NothingToApply` — and the
    /// ordinary save modes are still refused too, so the document cannot be
    /// saved at all until the staging is cancelled. That is a real corner and
    /// the sentence for it names the remedy by name rather than reporting a
    /// failure: `crate::text::redact::save_refused_message`.
    RedactionRefused {
        /// Which refusal, so the sentence can name the remedy rather than the
        /// mechanism.
        refusal: crate::redact::RedactApplyRefusal,
    },
    /// ★★★ **The bytes were built and their page tree does not agree with
    /// itself, so writing them would hand the operator a damaged file.**
    ///
    /// Added 2026-09-05, from his own report: *"I tested deleting pages from a
    /// pdf. when I open the document in Acrobat there are blank pages at the
    /// end of the document equalling the number of pages I deleted."*
    ///
    /// Like [`Self::RedactionLeak`] this reports a **pdfcer defect** rather
    /// than a property of the document or of the disk — and unlike it, this one
    /// is reachable today and he reached it. `pdfcer-core` v0.38.0's
    /// `delete_pages` decrements `/Count` on the removed page's immediate
    /// parent and on no ancestor above it (measured; the appended revision
    /// defines exactly one object), so on any document whose page tree has more
    /// than one level the root goes on declaring the pre-delete page count.
    /// `page-copy --cut` produces byte-for-byte the same corruption. Filed as
    /// `request_delete_pages_leaves_ancestor_count_stale_on_a_nested_page_tree.md`.
    ///
    /// ★ The whole [`crate::pagetree::Audit`] is carried, not a pair of
    /// numbers, because the sentence differs by *which* node disagrees (see
    /// [`crate::text::pagetree`]) and because the trace wants the node ids —
    /// and because `crate::app::lifecycle`'s rule is that a branch is made on
    /// structured error data, never by inspecting a message.
    PageTreeStale {
        /// What the walk found. Never rendered verbatim: it names object ids.
        audit: crate::pagetree::Audit,
    },
}

impl From<WriteError> for SaveError {
    fn from(error: WriteError) -> Self {
        Self::Serialize(error)
    }
}

impl From<std::io::Error> for SaveError {
    fn from(error: std::io::Error) -> Self {
        Self::Write(error)
    }
}

impl std::fmt::Display for SaveError {
    /// Diagnostic prose for the trace, and for nothing else.
    ///
    /// `check-ui-strings.sh`'s exclusion 3 permits a `Display` impl to carry
    /// text that is not in the catalog **because it is diagnostic**, and states
    /// in the same breath that this "is not permission to route UI text through
    /// an error type". Nothing here reaches an operator: the bar's sentence is
    /// `crate::text::status::save_copy_failed`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize(e) => write!(f, "the engine could not build the update: {e}"),
            Self::Write(e) => write!(f, "the file could not be written: {e}"),
            // The COUNT, never the strings. They are the redacted content, and
            // a diagnostic that announced a leak by printing the leaked text
            // into a log file would be the same failure at one remove.
            Self::RedactionLeak { survivors } => write!(
                f,
                "the save was refused: {} redacted string(s) survived in the bytes about to be \
                 written",
                survivors.len()
            ),
            // `{:?}`, deliberately: `RedactApplyRefusal` has no `Display` on
            // purpose, because it is rendered to the operator by
            // `crate::text::redact::refusal_message` and a second,
            // uncatalogued rendering is how the two drift. See
            // `crate::app::actions::redact`, which makes the same choice at the
            // other call site for the same reason.
            Self::RedactionRefused { refusal } => write!(
                f,
                "the staged redaction could not be performed, so no save was built: {refusal:?}"
            ),
            // The numbers and the node count, never the operator's sentence:
            // `crate::text::pagetree` owns that and a second, uncatalogued
            // rendering is how the two drift — the same choice
            // `RedactionRefused` makes one arm above.
            Self::PageTreeStale { audit } => write!(
                f,
                "the save was refused: the page tree declares {:?} pages over {} reachable, \
                 and {} node(s) disagree",
                audit.declared_pages,
                audit.reachable_pages,
                audit.disagreements.len()
            ),
        }
    }
}

/// **The name to suggest for the copy.**
///
/// Pure, so every rule in it is asserted headlessly. Two cases, and the
/// difference is [`OpenDoc::stored_under`] — the one predicate that separates a
/// document with a file from one that only has a name:
///
/// | the document | suggestion | why |
/// |---|---|---|
/// | opened from `D:\jobs\sheet.pdf` | `D:\jobs\sheet-copy.pdf` | beside the original, where the operator will look for it, and **never the original itself** |
/// | created by `file.new`, called `Untitled 1.pdf` | `Untitled 1.pdf`, with no directory | there is no original to avoid overwriting, and no folder it came from; the OS picker supplies its own starting directory and the operator has a name to accept |
///
/// # ★ Why the suffix, and why not for a created document
///
/// `crate::text::files::save_copy_suffix` carries the copy argument and the
/// reference-application head-count. The mechanical half is here: the promise
/// on `file.save_copy`'s tooltip is that *"the original is never overwritten
/// unless you pick it"*, and a **default that is the original** would make that
/// promise depend on the operator reading a pre-filled field before pressing
/// Enter. `crate::dialogs::ocr::suggested_path` enforces the identical rule with
/// `-recognised` and asserts it the identical way.
///
/// A created document has no original, so appending `-copy` would answer a
/// question nobody asked: it would offer `Untitled 1-copy.pdf` for a document
/// that has never been saved at all, which reads as though a first copy already
/// exists somewhere.
///
/// # The extension
///
/// Forced to `.pdf`, exactly as the OCR suggestion is, and for the same reason:
/// the bytes are a PDF whatever the source was called, and a copy of `SHEET.PDF`
/// landing as `SHEET-copy.PDF` would be correct and would be one more way for a
/// tool downstream to disagree about case. A created document's name already
/// ends in `.pdf` (`crate::text::files::untitled`), so this changes nothing for
/// it — which is why that function's own docs explain that the suffix is on the
/// name precisely so a save suggestion is not extensionless.
#[must_use]
fn suggested_path(doc: &OpenDoc) -> PathBuf {
    let Some(source) = doc.stored_under() else {
        // A name, not a location. Offer the name; the picker chooses the
        // folder, which is the only honest answer when the document has never
        // been anywhere.
        return doc.path.clone();
    };
    let stem = source.file_stem().map_or_else(
        // ui-text-exempt: a filename fallback for a path with no stem, not
        // operator copy. `crate::dialogs::ocr::suggested_path` makes the same
        // fallback for the same reason.
        || String::from("document"),
        |s| s.to_string_lossy().into_owned(),
    );
    let name = format!("{stem}{}.pdf", crate::text::files::save_copy_suffix());
    source
        .parent()
        .map_or_else(|| PathBuf::from(&name), |dir| dir.join(&name))
}

/// **Write an already-serialised compacted copy to a file the operator picks.**
///
/// `OPERATOR_REQUESTS.md` **O48**, and the counterpart to [`save_copy`] one
/// question along: that one asks *where*, this one has already asked *whether*.
///
/// # ★★★ Why this does not serialise
///
/// `crate::dialogs::compact` did, before it opened, and its headline number is a
/// measurement of the result rather than an estimate. Serialising again here
/// would put a second computation between the number the operator accepted and
/// the file they receive — see `Action::SaveCompacted`, which carries the bytes
/// for exactly that reason.
///
/// # ★★ Why it never writes in place
///
/// Because it destroys things the original still has: the earlier revision, and
/// every digital signature (§12.8.1). A command that could overwrite the
/// operator's file with a copy that has lost both is one keystroke from a loss
/// nothing can undo — so this offers only [`files::pick_save_path`], and the
/// window says *"this always writes a new one"* before the picker opens.
///
/// ★ The suggested name is [`suggested_path`]'s, shared with save-a-copy: the
/// operator's own file with a suffix, in its own folder. A second naming scheme
/// for the same act is how two commands come to disagree about what a copy is
/// called.
pub fn compacted(doc: &OpenDoc, bytes: &[u8], before: u64) -> bool {
    let suggested = suggested_path(doc);
    let Picked::Path(target) =
        files::pick_save_path(&suggested, crate::text::compact::window_title())
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "compact-cancelled".to_owned()
        });
        return false;
    };
    match std::fs::write(&target, bytes) {
        Ok(()) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // `before=` and `after=` rather than a saving, so a reader of a
                // trace can see which of the two the build got wrong. A single
                // difference is the one number that cannot be checked against
                // anything.
                format!(
                    "compact-written path={:?} before={before} after={} epoch={}",
                    target,
                    bytes.len(),
                    doc.edit_epoch
                )
            });
            crate::app::actions::record_note(
                doc.edit_epoch,
                crate::text::compact::written(
                    &target.display().to_string(),
                    before,
                    bytes.len() as u64,
                ),
            );
            true
        }
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("compact-failed path={target:?} detail={error}")
            });
            crate::app::actions::record_note(
                doc.edit_epoch,
                crate::text::compact::write_failed(&error.to_string()),
            );
            false
        }
    }
}

/// What must never be true of a file this shell wrote, in its own file since
/// 2026-09-04 — see [`tests`]'s header for the seam.
#[cfg(test)]
mod tests;
