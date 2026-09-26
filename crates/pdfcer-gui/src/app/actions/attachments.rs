//! # `app::actions::attachments` — the three verbs whose subject is a whole
//! FILE living inside the document
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/attachments.md`.

use pdfcer_core::attachments::{self, Attachment, AttachmentKind};
use pdfcer_core::edit::EditError;
use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::text::panels::attachments as t;

/// **Which attachment**, addressed the only two ways a PDF makes possible.
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
    /// **Embed a file in this document** (ISO 32000-1 §7.11.4.1, inclusion
    /// route 2).
    ///
    /// Raised by `crate::panels::attachments::attach` and by nothing else.
    ///
    /// # It carries no path, and that is the whole design of this variant
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
    /// # The description travels, because it can only be set now
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
    /// # `replacing` is carried, and it is not a convenience
    ///
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
    /// **Remove one document-level attachment** — the index entry, the file
    /// specification and the bytes, as ONE undo entry.
    ///
    /// Raised by `crate::panels::attachments` and by nothing else.
    ///
    /// # What "removed" does NOT mean, and why this variant carries a name
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
    /// A destructive verb must be *confirmed or clearly undoable*, and this is
    /// the second: one press is one `EditSession` command — `detach_file`
    /// removes the tree entry, the file specification and the stream *"all
    /// three, as ONE undo entry"* — so one `Ctrl+Z` puts the file back whole.
    ///
    /// The consequence the operator actually needs is not
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
    /// **Write one attachment's bytes out to a file the operator picks.**
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
    /// # Why the bytes are not carried
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
    /// It would also break an explicit engine contract.
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
        // `attach-file-READ`, not `attach-file`, and the suffix is not
        // decoration. `vector_edit` writes its own `attach-file page=… n=…`
        // line for the same edit two statements below, and a harness reads a
        // trace by its FIRST TOKEN — so two lines sharing a name means
        // `.last("attach-file")` returns the funnel's, which carries no `name`
        // and no `bytes`, and a check asserting on them reports *"the verb did
        // nothing"* about a verb that worked.
        //
        // ⇒ The convention that avoids it is **at the point of use**: a
        // module's own line takes a verb suffix, the funnel keeps the bare
        // name.
        format!(
            "attach-file-read name={name:?} bytes={size} described={}",
            description.is_some()
        )
    });

    super::apply::vector_edit(doc, "attach-file", 0, 1, |session| {
        match session.attach_file(&name, &bytes, description) {
            Ok(_) => Ok(vec![t::attached(&name, size)]),
            // The refusal is inspected here and the error is still returned,
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
fn detach(doc: &mut OpenDoc, key: &[u8], name: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        // `-requested`, for `attach-file-read`'s reason two functions up: the
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
            // Said on the way OUT rather than only in the panel's header,
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
fn resolve<'a>(listed: &'a [Attachment], at: &AttachmentRef) -> Option<&'a Attachment> {
    listed.iter().find(|found| match (&found.kind, at) {
        (AttachmentKind::DocumentLevel { tree_key }, AttachmentRef::DocumentLevel { key }) => {
            tree_key == key
        }
        (
            AttachmentKind::PageAnnotation { annot_id, .. },
            AttachmentRef::PageAnnotation { annot },
        ) => *annot_id == Some(*annot),
        // `AttachmentKind` is `#[non_exhaustive]`, so a kind this build has
        // never seen must resolve to **nothing** rather than to whatever is
        // nearest. A verb that acted on the wrong file because a match arm
        // guessed is the one failure this whole type exists to prevent.
        _ => false,
    })
}

/// Where the save dialog opens, and what it calls the file.
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

    /// **A document-level reference finds the document-level attachment, and
    /// a page-level reference finds the page-level one.**
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

    /// **A key that is not in the tree resolves to nothing**, rather than to
    /// the nearest row.
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

    /// **Keys are compared byte-for-byte, so case matters.**
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

    /// **A hostile name never reaches the save dialog.**
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
