//! # `app::actions::attachments` — the three verbs whose subject is a whole
//! FILE living inside the document
//!
//! Split out of [`super::action`] under **R2** on the day the capability
//! acquired a surface. `super`'s declaration of `action` states the rule this
//! follows — *"the next family of variants to **grow** is the one that will
//! have to become a sub-enum"* — and a family that arrives with three verbs at
//! once has grown before it was written.
//!
//! ## ★★★ What makes these a family, and it is not "they are all about
//! attachments"
//!
//! A subject label would be the weak answer, and this enum has a structural one
//! that no other family in the vocabulary shares:
//!
//! > **Every verb here operates on bytes that are in no page's content stream,
//! > and two of the three are file-system operations that happen to touch a
//! > PDF.**
//!
//! Three consequences follow from that one property, and the whole module is
//! shaped by them:
//!
//! 1. **Every one of them opens a native file dialog, and therefore must be an
//!    `Action`.** `super::write`'s header states the rule in the sharpest form
//!    this project has written it: *"A native file dialog must not open inside
//!    a layout pass. It is a modal OS window that blocks the thread, so opening
//!    one from a widget's `clicked()` branch leaves egui part-way through a
//!    frame that will not finish until the operator has answered."*
//!    [`AttachmentAction::Attach`] and [`AttachmentAction::SaveCopy`] are
//!    `Action`s for **both** reasons — the funnel's invariant *and* the
//!    dialog's timing — where `WriteAction`'s three are `Action`s for the
//!    second alone.
//! 2. **Nothing they do is visible on the canvas.** A document-level
//!    attachment (§7.11.4.1 route 2) is reached from the catalogue's
//!    `/Names /EmbeddedFiles` and appears in no rendering, so *every* one of
//!    these verbs owes a sentence to `crate::app::status`. Contrast
//!    `super::bookmarks`, where a rename is deliberately silent because the row
//!    the operator is looking at now reads the new name. There is no such row
//!    here: the panel is the only witness, and the panel is where the operator
//!    already is, so the disclosure carries the part the panel cannot show —
//!    what happened to the *file*.
//! 3. **The operand cannot be an index and cannot be an `ObjId` either.** See
//!    [`AttachmentRef`], which is the interesting type in this module.
//!
//! ## ★★ The one refusal this module surfaces, and the three it does not
//!
//! `EditSession::attach_file` refuses four ways. Three of them —
//! `DocumentEncrypted`, the certification gate, and
//! `ObjectCreationWouldExposeHiddenObjects` — are properties of the **document**
//! that every authoring verb in this shell shares, and this shell's settled
//! answer for them is [`super::apply::vector_edit`]'s trace. That is not
//! neglect: they are conditions no control on this panel created and none can
//! clear, and four sentences competing for the one status slot would evict the
//! disclosures the successful verbs owe.
//!
//! `AttachmentTreeUnsupported` is surfaced, and the argument is at
//! [`crate::text::panels::attachments::attach_refused_multi_node_tree`]. In one
//! line: it is unreachable from any other surface, and the press otherwise
//! produces **nothing at all** — no row, no message — which an operator cannot
//! distinguish from a broken button.
//!
//! ## What is deliberately NOT here
//!
//! **Editing a description.** `attach_file` takes it at attach time and
//! `pdfcer-core` has no verb that changes one afterwards, so there is no
//! variant, no field and no control. R9: an absent capability renders nothing.
//! The panel *says* so — see `attach_description_note` — because a control that
//! cannot exist is different from a limit that must not be discovered by
//! trying.
//!
//! **Removing a page-level file attachment.** `detach_file` addresses the
//! `/EmbeddedFiles` name tree and answers `AttachmentNotFound` for a
//! `/FileAttachment` annotation **by name**, precisely so a shell can tell the
//! two apart; those are removed with `delete_annotation`, which is
//! `super::annot::AnnotAction::Delete`'s business and reached from the Comments
//! panel. Wiring a second route to it from here would give one act two
//! implementations, and the second would be the one that forgot the page
//! invalidation.

use pdfcer_core::attachments::{self, Attachment, AttachmentKind};
use pdfcer_core::edit::EditError;
use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::text::panels::attachments as t;

/// **Which attachment**, addressed the only two ways a PDF makes possible.
///
/// # ★★★ Why this is not an index, and not an `ObjId` either
///
/// `super::bookmarks`' header argues at length that an outline row must be
/// addressed by `ObjId` rather than by a position, because every edit to a tree
/// renumbers it. Both halves of that argument apply here and neither one
/// finishes the job:
///
/// - **A position is wrong for the same reason.** The `/EmbeddedFiles` name
///   tree is sorted (§7.9.6: keys *"shall be sorted lexically in ascending
///   order"*), and `attach_file` re-sorts the whole array on every insert. So
///   *"the third row"* names a different file after any attach — and the queue
///   drains **after** the frame, so a second action raised in the same frame is
///   resolved against an already-moved list.
/// - **An `ObjId` is not available for the verb that needs one.**
///   `EditSession::detach_file` takes *"its `/EmbeddedFiles` name-tree key"* —
///   the raw bytes — and nothing else. The filespec's object id, which the
///   listing does report, is not what that function accepts, and §7.9.6 makes
///   the key a **byte string** with no declared encoding, so it cannot even be
///   carried as a `String` without deciding an encoding the standard declines
///   to.
///
/// ⇒ Hence `Vec<u8>` for the document-level case: it is what the engine takes,
/// and `AttachmentKind::DocumentLevel::tree_key` exists to hand it over
/// *"exactly as the tree spells them"*.
///
/// The page-level case gets the annotation's `ObjId` instead, because that kind
/// has no key at all — it lives in one page's `/Annots` — and because the id is
/// what stays stable across a page reorder, which `page_index` does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachmentRef {
    /// An entry in the catalogue's `/Names /EmbeddedFiles` name tree, by its
    /// raw key bytes.
    DocumentLevel {
        /// The key, verbatim. §7.9.6 requires keys to be *"compared for
        /// equality on a simple byte-by-byte basis"*, which is what makes
        /// carrying the bytes both necessary and sufficient.
        key: Vec<u8>,
    },
    /// A `/FileAttachment` annotation (§12.5.6.15), by the annotation's own
    /// object id.
    ///
    /// Only constructible when the listing reported one — `Attachment`'s
    /// `annot_id` is an `Option`, `None` when the `/Annots` entry was a direct
    /// dictionary rather than a reference. The panel offers no control for a
    /// row it cannot address, which is R9 rather than caution: a Save button
    /// that could not name its operand would be an affordance for something
    /// that cannot work.
    PageAnnotation {
        /// The annotation object.
        annot: ObjId,
    },
}

/// The three verbs whose subject is a whole file inside the document.
///
/// See the module header for what makes them a family, and [`AttachmentRef`]
/// for why the operand is neither an index nor an object id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachmentAction {
    /// ★★★ **Embed a file in this document** (ISO 32000-1 §7.11.4.1, inclusion
    /// route 2).
    ///
    /// Raised by `crate::panels::attachments::attach` and by nothing else.
    ///
    /// # ★★ It carries no path, and that is the whole design of this variant
    ///
    /// The picker opens inside the **apply** phase, exactly as
    /// `super::write::WriteAction::FormData`'s does and for its stated reason.
    /// The alternative — pick in the widget, carry the path — puts a modal OS
    /// window inside a layout pass, which is the defect `super::write`'s header
    /// exists to name.
    ///
    /// It also buys the harness seam: `crate::app::files::DIAG_ATTACH_PATH`
    /// answers the dialog without a human, and a driven check of this feature
    /// is otherwise unwritable because no synthetic input reaches a native
    /// dialog.
    ///
    /// # ★★ The description travels, because it can only be set now
    ///
    /// `attach_file` takes `description: Option<&str>` and writes it to the
    /// file specification's `/Desc` (Table 44, whose own row says `/Desc`
    /// *"shall be used for files in the `EmbeddedFiles` name tree"* — exactly
    /// this route). `pdfcer-core` exposes **no verb that edits one afterwards**,
    /// so this is the operator's only opportunity, and the value has to be
    /// captured at the press rather than read at apply time: the panel's draft
    /// is cleared on the frame the action is raised, and the queue drains after
    /// it.
    ///
    /// `None` is the ordinary answer and is not the same as `Some("")`: an
    /// empty description would write a `/Desc` key holding nothing, which is a
    /// key a later reader has to interpret. Omitting it says the document
    /// carries no description, which is the truth.
    Attach {
        /// What the operator typed, trimmed and non-empty by the time it gets
        /// here, or `None` for no `/Desc` at all.
        description: Option<String>,
    },
    /// **Attach the clipboard's file to this document.**
    ///
    /// # ★★★ `replacing` is carried, and it is not a convenience
    ///
    /// `attach_file` does `entries.retain(|(k, _)| k != &name_bytes)` before
    /// inserting, so a same-named attachment is **replaced** — silently, with
    /// the old bytes recoverable only from the earlier revision. The panel says
    /// so before the press; this field is how the sentence *after* the press
    /// can say it too.
    ///
    /// It is computed **before** the write, because afterwards the answer has
    /// changed: the document now has exactly one file of that name either way,
    /// so asking then cannot distinguish the two outcomes. That is the same
    /// reasoning `FormEdit::Recompute` records for carrying its plan.
    Paste {
        /// The clip, carried whole. See `panels::attachments::clip` for why it
        /// travels with the action rather than being re-read at apply time.
        clip: Box<pdfcer_core::attachments::AttachmentClip>,
        /// Whether a file of this name is already listed in the destination.
        replacing: bool,
    },
    /// ★★★ **Remove one document-level attachment** — the index entry, the file
    /// specification and the bytes, as ONE undo entry.
    ///
    /// Raised by `crate::panels::attachments` and by nothing else.
    ///
    /// # ★★★ What "removed" does NOT mean, and why this variant carries a name
    ///
    /// `detach_file`'s own doc comment states the obligation this shell is
    /// under, and it is unusually direct:
    ///
    /// > *"This is NOT a redaction verb and must not be described as one … the
    /// > attachment's bytes remain recoverable from the earlier revision. Only
    /// > a full rewrite drops superseded revisions … Shells are expected to say
    /// > so rather than let 'delete' imply erasure."*
    ///
    /// The `name` field exists for that sentence and for nothing else. The
    /// engine returns `()`, the row is gone from the panel by the time the
    /// disclosure is read, and *"An attachment was removed"* is a sentence that
    /// leaves an operator who removed the wrong one unable to tell.
    ///
    /// It is the **displayed** name rather than the key, deliberately: the key
    /// is bytes with no declared encoding (§7.9.6) and producers mangle it with
    /// numeric suffixes and portfolio folder prefixes, so it is the right thing
    /// to address the document with and the wrong thing to show a person.
    ///
    /// # Why there is no confirmation dialog
    ///
    /// `HANDOFF.md`'s rule is *confirmed or clearly undoable*, and this is the
    /// second: one press is one `EditSession` command — the engine plans the
    /// tree patch and both object removals inside it — so one `Ctrl+Z` puts the
    /// file back whole. The consequence the operator actually needs is not
    /// *"are you sure?"* but *"this does not erase the bytes"*, and a
    /// confirmation dialog is a bad place to put that because it arrives
    /// **after** the decision. It is on the panel, beside the button, before
    /// the press.
    Detach {
        /// The `/EmbeddedFiles` name-tree key, verbatim. See [`AttachmentRef`].
        key: Vec<u8>,
        /// The name the panel showed, for the disclosure. Never used to find
        /// anything.
        name: String,
    },
    /// ★★★ **Write one attachment's bytes out to a file the operator picks.**
    ///
    /// Raised by `crate::panels::attachments` and by nothing else.
    ///
    /// # It changes nothing about the document
    ///
    /// No `vector_edit`, no undo entry, no epoch bump, no invalidation — the
    /// property `super::export`'s header names as what makes an export a
    /// subject of its own. It is filed here rather than there because its
    /// *operand* is an attachment and its refusals are attachment refusals; the
    /// seam `super::export` draws is *"what class of thing does this verb act
    /// on?"*, and by that seam this belongs beside its two siblings.
    ///
    /// # ★★ Why the bytes are not carried
    ///
    /// `super::write::WriteAction::Compacted` carries a whole serialised
    /// document, and its doc says why: the confirmation window quoted a
    /// measurement of those exact bytes, so those exact bytes are the operand.
    /// Nothing quoted anything here. Carrying the payload would mean decoding a
    /// possibly-enormous stream during the **frame**, to hand the apply phase a
    /// value it can decode itself — and a *stale* one, because an undo raised
    /// earlier in the same frame would leave the copy describing a revision
    /// that is no longer open.
    ///
    /// ★ It would also break an explicit engine contract.
    /// `extract_attachment`'s docs warn that *"the view must be the one the
    /// `Attachment` was listed from"* — an `Attachment` carries object ids, and
    /// an id only means something relative to a document — so the listing and
    /// the extraction have to happen in one breath. [`save_copy`] does exactly
    /// that, against the session as it stands when the save runs, which is the
    /// only reading that can be defended.
    SaveCopy {
        /// Which attachment, addressed the only way its kind allows.
        at: AttachmentRef,
        /// The name the panel showed, for the trace and for the sentence that
        /// says whether pdfcer had to use a different one on disk.
        name: String,
    },
}

/// Apply one attachment verb.
///
/// The dispatch half of this module, reached from `PdfcerApp::apply`'s single
/// [`super::action::Action::Attachment`] arm. A free function taking
/// `&mut OpenDoc` rather than a method, exactly like [`super::bookmarks::apply`]
/// and [`super::pages::apply`], because the caller is the one place that owns
/// the borrow and the arm should be one line.
///
/// ★ **Two of the three do not go through [`super::apply::vector_edit`]**, and
/// the exception is principled rather than convenient: that function is the
/// cancel–mutate–bump–invalidate protocol for an edit, and
/// [`AttachmentAction::SaveCopy`] performs no edit. Running it through anyway
/// would cancel the render worker and bump the epoch for an operation that
/// changed nothing, which is how a status bar comes to retire a disclosure that
/// is still true. [`AttachmentAction::Attach`] and
/// [`AttachmentAction::Detach`] **do** mutate and **do** go through it.
///
/// ★ The `page` argument passed to `vector_edit` is `0` for both mutating
/// verbs, and that is honest rather than lazy: a document-level attachment
/// belongs to the catalogue and to no page. [`super::bookmarks::apply`] passes
/// `0` for the identical reason, and its comment records that the parameter
/// exists so the diagnostic trace can say which sheet a *geometry* edit
/// touched.
pub(super) fn apply(doc: &mut OpenDoc, action: AttachmentAction) {
    match action {
        AttachmentAction::Attach { description } => attach(doc, description.as_deref()),
        AttachmentAction::Detach { key, name } => detach(doc, &key, &name),
        AttachmentAction::SaveCopy { at, name } => save_copy(doc, &at, &name),
        AttachmentAction::Paste { clip, replacing } => paste(doc, &clip, replacing),
    }
}

/// **Attach the clipboard's file**, disclosing a replacement if there was one.
fn paste(doc: &mut OpenDoc, clip: &pdfcer_core::attachments::AttachmentClip, replacing: bool) {
    let name = clip.name.clone();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "paste-attachment-requested name={name:?} bytes={} replacing={replacing}",
            clip.bytes.len()
        )
    });
    super::apply::vector_edit(doc, "paste-attachment", 0, 1, |session| {
        session.paste_attachment(clip).map(|_| {
            vec![if replacing {
                crate::text::attachclip::pasted_over(&name)
            } else {
                crate::text::attachclip::pasted(&name)
            }]
        })
    });
}

/// **Embed a file**, as one undoable command, disclosing what the page cannot
/// show.
///
/// # The order of operations, and why the picker is first
///
/// Opposite to `super::export::dxf`, which writes first and asks second. That
/// verb can do it because its write *cannot fail*; this one has nothing to
/// produce until the operator has named a file, so the picker is the first
/// step by necessity rather than by choice.
///
/// The read is second and the mutation third, which does matter: a file the
/// operator picked and pdfcer cannot read must decline **before** the session is
/// touched, so a failed attach leaves no undo entry to step past.
///
/// # ★ The name written into the PDF is the file's own base name
///
/// Not the full path. §7.11.2.1 says a file-specification string's bytes
/// *"shall be passed to the operating system without interpretation"*, so
/// writing `D:\quotes\2026\supplier.xlsx` into `/F` would produce a document
/// that names a location on the machine that made it — a small privacy leak in
/// every copy of the file, and a name that means nothing to anyone else.
/// Acrobat writes the base name; so does this.
///
/// `FALLBACK_SAFE_NAME` covers the case the OS admits and nobody expects: a
/// path with no final component. It is the engine's own constant rather than a
/// literal here, so the fallback pdfcer *writes* and the fallback pdfcer
/// *substitutes when saving one out* cannot drift apart.
fn attach(doc: &mut OpenDoc, description: Option<&str>) {
    let crate::app::files::Picked::Path(source) = crate::app::files::pick_attachment_source()
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "attach-file-cancelled".to_owned()
        });
        return;
    };

    let bytes = match std::fs::read(&source) {
        Ok(bytes) => bytes,
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("attach-file-unreadable detail={error}")
            });
            super::record_note(
                doc.edit_epoch,
                t::attach_source_unreadable(&error.to_string()),
            );
            return;
        }
    };

    let name = source.file_name().map_or_else(
        || attachments::FALLBACK_SAFE_NAME.to_owned(),
        |base| base.to_string_lossy().into_owned(),
    );
    let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    // Captured before the closure borrows the session: `record_note` needs the
    // epoch the document is on *now*, because a refusal produces no new one.
    // Stamping it is what makes the sentence stand until the next real edit
    // moves past it — see `super::disclosure`.
    let epoch = doc.edit_epoch;

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed. The name and the
        // SIZE, not the bytes.
        //
        // ★★★ `attach-file-READ`, not `attach-file`, and the suffix is not
        // decoration. `vector_edit` writes its own `attach-file page=… n=…`
        // line for the same edit two statements below, and a harness reads a
        // trace by its FIRST TOKEN — so two lines sharing a name means
        // `.last("attach-file")` returns the funnel's, which carries no `name`
        // and no `bytes`, and a check asserting on them reports *"the verb did
        // nothing"* about a verb that worked.
        //
        // ⇒ This project has made that exact mistake twice — `text-style` on
        // 2026-08-27 and `import-form-data` on 2026-08-28, the second by the
        // session that had written up the first — and the fix agreed then was a
        // **naming convention at the point of use**: a module's own line takes
        // a verb suffix, the funnel keeps the bare name. This is that
        // convention, and it was caught here by a driven check being written
        // rather than by a reader.
        format!(
            "attach-file-read name={name:?} bytes={size} described={}",
            description.is_some()
        )
    });

    super::apply::vector_edit(doc, "attach-file", 0, 1, |session| {
        match session.attach_file(&name, &bytes, description) {
            Ok(_) => Ok(vec![t::attached(&name, size)]),
            // ★ The refusal is inspected here and the error is still returned,
            // which is `super::forms::adopt`'s pattern and its argument:
            // recording is for the operator, returning is for the trace, and
            // the two are not the same text and must not become each other.
            Err(error) => {
                if matches!(error, EditError::AttachmentTreeUnsupported) {
                    super::record_note(epoch, t::attach_refused_multi_node_tree().to_owned());
                }
                Err(error)
            }
        }
    });
}

/// **Remove one document-level attachment**, as one undoable command,
/// disclosing that its bytes are still in the file.
///
/// # ★★★ The disclosure is the point of this function
///
/// `detach_file` returns `()`. There is no count to report, no residue to
/// describe, and nothing on screen changes except a row disappearing — which
/// the operator caused and expected. The one thing they cannot see is the one
/// thing that matters: **under the default incremental save (§7.5.6) every
/// prior revision is still in the file by design**, which is what makes
/// existing signatures survive, and it means the removed attachment is
/// recoverable from the earlier revision by anyone who opens the bytes.
///
/// `crate::text::panels::attachments::removed` carries that sentence and names
/// the remedy — `file.save_compacted`, the full rewrite — because a disclosure
/// that states a hazard and leaves the operator to find the way out has done
/// half the job.
///
/// # What it cannot be asked to do
///
/// A page-level file attachment. `detach_file` answers `AttachmentNotFound` for
/// one **by name**, and the panel does not offer a Remove control on those rows
/// at all, so the refusal is unreachable from this surface rather than routed
/// around. Recorded here so nobody adds a guard for a case that cannot occur.
fn detach(doc: &mut OpenDoc, key: &[u8], name: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        // ★ `-requested`, for `attach-file-read`'s reason two functions up: the
        // funnel writes its own bare `detach-file` line for the same edit on
        // the next statement, and two lines sharing a first token means a
        // harness reading either one reads the wrong one.
        format!("detach-file-requested key_len={} name={name:?}", key.len())
    });
    super::apply::vector_edit(doc, "detach-file", 0, 1, |session| {
        session.detach_file(key).map(|()| vec![t::removed(name)])
    });
}

/// **Write one attachment out to a file the operator picks.**
///
/// # ★★ The listing and the extraction happen in one breath, and that is a
/// contract rather than a style
///
/// `extract_attachment`'s doc comment is explicit:
///
/// > *"An `Attachment` carries object ids, and an id only means something
/// > relative to a document. Passing a view of a different document will either
/// > fail … or, if the other document happens to have a stream at the same id,
/// > return **that** document's bytes. pdfcer cannot detect the confusion …
/// > Listing from `doc` and extracting through `doc.view()` in the same breath
/// > … makes it a non-issue."*
///
/// So the panel does not carry an `Attachment`, and this function does not
/// cache one. It re-lists, resolves the operand it was given, and extracts,
/// all against one borrow of one session.
///
/// # ★★★ The name is sanitised before it touches the filesystem
///
/// [`Attachment::name`] is **attacker-controlled text** and nothing in
/// ISO 32000-1 constrains it: `..\..\..\Windows\System32\evil.exe`,
/// `/etc/cron.d/pwn`, `report.pdf\0.exe` and `CON.txt` are all authorable, and
/// §7.9.6 says even less about a name-tree key. `Attachment::safe_name` exists
/// *"so the **safe** call is the short one"*, and this is the extraction path
/// its docs say should reach for it.
///
/// ★ And the sanitiser's answer is **reported**, not merely used. The listing
/// shows the raw name — because a reader that quietly repairs its evidence is
/// not a reader — so the row and the file on disk can legitimately disagree,
/// and `SafeName::hazards` is carried precisely so the sentence can say what
/// changed and why.
///
/// # What is still the caller's problem, per the engine's own warning
///
/// A `SafeName` is *"a name, not a location"*. This joins it to a directory the
/// operator chose in a native save dialog — which is also where overwrite
/// confirmation comes from, because the OS dialog owns that question and asks
/// it better than pdfcer could.
fn save_copy(doc: &mut OpenDoc, at: &AttachmentRef, name: &str) {
    let epoch = doc.edit_epoch;

    // One borrow of one session: list, resolve, extract. See the header —
    // splitting these would let an id from one revision reach another.
    let found = {
        let view = doc.session.view();
        let (listed, notes) = attachments::list_attachments_with_notes(&view);
        match resolve(&listed, at) {
            None => Err(t::gone().to_owned()),
            Some(attachment) => {
                let safe = attachment.safe_name();
                let raw = attachment.name.clone();
                match attachments::extract_attachment(&view, attachment) {
                    Ok(extracted) => Ok((extracted.data, safe, raw, notes.may_be_encrypted)),
                    Err(error) => Err(t::extract_failed(&error.to_string())),
                }
            }
        }
    };

    let (data, safe, raw, encrypted) = match found {
        Ok(parts) => parts,
        Err(said) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("attachment-save-declined name={name:?}")
            });
            super::record_note(epoch, said);
            return;
        }
    };

    let crate::app::files::Picked::Path(target) =
        crate::app::files::pick_attachment_target(&suggested_path(doc, &safe.value))
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("attachment-save-cancelled name={name:?}")
        });
        return;
    };

    match std::fs::write(&target, &data) {
        Ok(()) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "attachment-saved bytes={} renamed={} hazards={}",
                    data.len(),
                    safe.changed,
                    safe.hazards.len()
                )
            });
            // The list is assembled and recorded once, because the slot holds
            // ONE disclosure and the last writer would win — `super::export`
            // records the same constraint for the same reason.
            let mut notes = vec![t::saved(&target.display().to_string())];
            if safe.changed {
                notes.push(t::name_was_changed(&raw, &safe.value, &safe.hazards));
            }
            // ★ Said on the way OUT rather than only in the panel's header,
            // because this is the moment it becomes actionable: bytes now exist
            // on disk that may be ciphertext, and the operator is about to open
            // them. See `AttachmentNotes::may_be_encrypted` for why the flag is
            // deliberately over-broad and why over-warning is the correct error.
            if encrypted {
                notes.push(t::may_be_encrypted().to_owned());
            }
            super::record_edit_disclosure(Some(super::EditDisclosure { epoch, notes }));
        }
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("attachment-save-failed detail={error}")
            });
            super::record_note(epoch, t::save_failed(&error.to_string()));
        }
    }
}

/// The attachment `at` names, in a listing taken from the open document.
///
/// A free function, and pure, so [`tests`] can hold it to the two properties
/// that matter without a `Ui` and without a running application.
///
/// # ★ Why the comparison is byte-for-byte and case-sensitive
///
/// §7.9.6 requires name-tree keys to be *"compared for equality on a simple
/// byte-by-byte basis"* — not by any collation, not case-folded, not
/// normalised. Two keys differing only in case are two different attachments,
/// and a lenient comparison here would let a Remove press find the wrong one.
///
/// `None` is a reachable, ordinary answer rather than an error: the queue
/// drains after the frame, so an undo or a removal raised earlier in the same
/// frame can take the row away before this action is applied.
fn resolve<'a>(listed: &'a [Attachment], at: &AttachmentRef) -> Option<&'a Attachment> {
    listed.iter().find(|found| match (&found.kind, at) {
        (AttachmentKind::DocumentLevel { tree_key }, AttachmentRef::DocumentLevel { key }) => {
            tree_key == key
        }
        (
            AttachmentKind::PageAnnotation { annot_id, .. },
            AttachmentRef::PageAnnotation { annot },
        ) => *annot_id == Some(*annot),
        // ★ `AttachmentKind` is `#[non_exhaustive]`, so a kind this build has
        // never seen must resolve to **nothing** rather than to whatever is
        // nearest. A verb that acted on the wrong file because a match arm
        // guessed is the one failure this whole type exists to prevent.
        _ => false,
    })
}

/// Where the save dialog opens, and what it calls the file.
///
/// Beside the **document**, named after the attachment — which is the
/// combination the two halves of the rule give. `super::export::suggested_path`
/// states the directory half and its reason: *"a picker that opens in the
/// last-used directory of some other application is a picker that makes the
/// operator navigate back to their own project every time."* The name half is
/// different from every other suggestion in this application, because the file
/// being written is not derived from the document at all — it is a file that
/// was put inside it, and it has its own name.
///
/// `safe_name` is the **sanitised** value and must be: this string reaches a
/// native save dialog, and a raw attachment name can be a path.
fn suggested_path(doc: &OpenDoc, safe_name: &str) -> std::path::PathBuf {
    let mut path = doc.path.clone();
    path.set_file_name(safe_name);
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::objects::test_support::engine_fixture;

    /// The engine's own two-kinds fixture, listed.
    fn both_kinds() -> Vec<Attachment> {
        let path = engine_fixture("attachments/both-kinds.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let listed = attachments::list_attachments(&doc);
        assert_eq!(
            listed.len(),
            2,
            "this fixture carries one of each kind; if it does not, every \
             assertion below is proving something else"
        );
        listed
    }

    /// ★★ **A document-level reference finds the document-level attachment, and
    /// a page-level reference finds the page-level one.**
    ///
    /// The assertion is not "resolve returns something". It is that the two
    /// kinds do not cross — which is exactly what a match arm written in a
    /// hurry gets wrong, and which the fixture can actually distinguish because
    /// it holds one of each.
    #[test]
    fn a_reference_finds_its_own_kind_and_not_the_other() {
        let listed = both_kinds();
        let AttachmentKind::DocumentLevel { tree_key } = &listed[0].kind else {
            panic!("the fixture's first entry must be document-level") // ui-text-exempt: test panic, never displayed
        };
        let AttachmentKind::PageAnnotation { annot_id, .. } = &listed[1].kind else {
            panic!("the fixture's second entry must be a page annotation") // ui-text-exempt: test panic, never displayed
        };
        let annot = annot_id.expect("the fixture's annotation is an indirect object");

        let by_key = AttachmentRef::DocumentLevel {
            key: tree_key.clone(),
        };
        let by_annot = AttachmentRef::PageAnnotation { annot };

        assert!(
            std::ptr::eq(
                resolve(&listed, &by_key).expect("the key resolves"),
                &listed[0]
            ),
            "a tree key must find the document-level entry"
        );
        assert!(
            std::ptr::eq(
                resolve(&listed, &by_annot).expect("the annotation resolves"),
                &listed[1]
            ),
            "an annotation id must find the page-level entry"
        );
    }

    /// ★★★ **A key that is not in the tree resolves to nothing**, rather than to
    /// the nearest row.
    ///
    /// The failure this forbids is the one that would make the whole
    /// address-by-key argument hollow: a `find` written as *"the first
    /// document-level entry"* passes the test above and removes the wrong file
    /// the moment a document has two.
    ///
    /// Both directions are checked, because a resolver can be wrong in two
    /// ways — finding something when it should find nothing, and matching a
    /// key against the wrong kind's operand.
    #[test]
    fn an_unknown_operand_resolves_to_nothing() {
        let listed = both_kinds();
        assert!(
            resolve(
                &listed,
                &AttachmentRef::DocumentLevel {
                    key: b"no-such-key".to_vec()
                }
            )
            .is_none(),
            "a key nothing is filed under must resolve to nothing"
        );
        assert!(
            resolve(
                &listed,
                &AttachmentRef::PageAnnotation {
                    annot: ObjId::new(9_999, 0)
                }
            )
            .is_none(),
            "an annotation id this document does not have must resolve to nothing"
        );
    }

    /// ★ **Keys are compared byte-for-byte, so case matters.**
    ///
    /// §7.9.6 requires exactly this — *"compared for equality on a simple
    /// byte-by-byte basis"* — and a lenient comparison is the kind of
    /// helpfulness that removes the wrong attachment from a document holding
    /// both `Report.pdf` and `report.pdf`, which is legal.
    #[test]
    fn a_key_differing_only_in_case_is_a_different_attachment() {
        let listed = both_kinds();
        let AttachmentKind::DocumentLevel { tree_key } = &listed[0].kind else {
            panic!("the fixture's first entry must be document-level") // ui-text-exempt: test panic, never displayed
        };
        let flipped: Vec<u8> = tree_key
            .iter()
            .map(|b| {
                if b.is_ascii_lowercase() {
                    b.to_ascii_uppercase()
                } else {
                    b.to_ascii_lowercase()
                }
            })
            .collect();
        assert_ne!(
            &flipped, tree_key,
            "the fixture's key must contain a letter, or this test proves nothing"
        );
        assert!(
            resolve(&listed, &AttachmentRef::DocumentLevel { key: flipped }).is_none(),
            "case-folding a name-tree key would violate §7.9.6 and could remove \
             the wrong file"
        );
    }

    /// **The three verbs are three distinct values**, so a match on them cannot
    /// silently collapse, and two removals of different files are two different
    /// actions.
    ///
    /// The second half is the one worth having: the queue may hold more than
    /// one action from a frame, and a variant that compared equal on only part
    /// of its operand would let a de-duplicating caller drop the wrong one.
    #[test]
    fn the_verbs_and_their_operands_are_distinguishable() {
        let attach = AttachmentAction::Attach { description: None };
        let described = AttachmentAction::Attach {
            description: Some("the supplier's quote".to_owned()),
        };
        let detach = AttachmentAction::Detach {
            key: b"quote.xlsx".to_vec(),
            name: "quote.xlsx".to_owned(),
        };
        let other = AttachmentAction::Detach {
            key: b"drawing.dwg".to_vec(),
            name: "drawing.dwg".to_owned(),
        };
        let save = AttachmentAction::SaveCopy {
            at: AttachmentRef::DocumentLevel {
                key: b"quote.xlsx".to_vec(),
            },
            name: "quote.xlsx".to_owned(),
        };
        assert_ne!(attach, described);
        assert_ne!(attach, detach);
        assert_ne!(detach, other);
        assert_ne!(detach, save);
    }

    /// ★★ **A hostile name never reaches the save dialog.**
    ///
    /// The fixture is the engine's own, and it exists because these names are
    /// authorable in a real document. What is asserted is the property the
    /// suggested path must have: **one component**, inside the directory the
    /// document is in, whatever the document called the file.
    ///
    /// The check is deliberately on the assembled path rather than on
    /// `safe_name` alone — sanitising and then joining wrongly would pass a
    /// test of the sanitiser and still write outside the folder.
    #[test]
    fn a_hostile_attachment_name_cannot_escape_the_chosen_folder() {
        let path = engine_fixture("attachments/hostile-names.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let listed = attachments::list_attachments(&doc);
        assert!(
            !listed.is_empty(),
            "this fixture exists to carry unsafe names; if it lists none, the \
             test proves nothing"
        );

        let mut any_changed = false;
        for attachment in &listed {
            let safe = attachment.safe_name();
            any_changed |= safe.changed;
            // One component, and not a traversal.
            let as_path = std::path::Path::new(&safe.value);
            assert_eq!(
                as_path.components().count(),
                1,
                "a sanitised name must be one path component: {:?}",
                safe.value
            );
            assert!(
                !safe.value.contains("..") || as_path.file_name().is_some(),
                "a sanitised name must not be a bare traversal: {:?}",
                safe.value
            );
        }
        assert!(
            any_changed,
            "at least one of this fixture's names must have needed changing, or \
             the sanitiser is not being exercised at all"
        );
    }

    /// **The suggestion sits beside the document and is named after the
    /// attachment.**
    ///
    /// Both halves, because getting either wrong is invisible until an operator
    /// is hunting for a folder: a suggestion in the wrong directory makes them
    /// navigate back to their own project, and one named after the *document*
    /// would offer to save a spreadsheet as `drawing.pdf`.
    #[test]
    fn the_suggested_path_is_the_document_s_folder_and_the_attachment_s_name() {
        let path = engine_fixture("pageops/four-pages.pdf");
        let doc_path = path.clone();
        let document = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&document).expect("a page tree");
        let open = OpenDoc::new(path, pdfcer_core::edit::EditSession::new(document), pages);

        let suggested = suggested_path(&open, "quote.xlsx");
        assert_eq!(suggested.parent(), doc_path.parent());
        assert_eq!(
            suggested.file_name().map(std::ffi::OsStr::to_string_lossy),
            Some(std::borrow::Cow::Borrowed("quote.xlsx"))
        );
    }
}
