//! # `app::actions::attachments` — the three verbs whose subject is a whole
//! FILE living inside the document
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/attachments.md`.

use pdfcer_core::attachments::{self, Attachment, AttachmentKind};
use pdfcer_core::edit::EditError;
#[cfg(test)]
use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::text::panels::attachments as t;

pub use pdfcer_gui_base::subactions::AttachmentRef;

pub use pdfcer_gui_base::editactions::AttachmentAction;

/// Apply one attachment verb.
pub(super) fn apply(doc: &mut OpenDoc, action: AttachmentAction) {
    match action {
        AttachmentAction::Attach { description } => attach(doc, description.as_deref()),
        AttachmentAction::Detach { key, name } => detach(doc, &key, &name),
        AttachmentAction::SaveCopy { at, name } => save_copy(doc, &at, &name),
        AttachmentAction::Paste { clip, replacing } => paste(doc, &clip, replacing),
        AttachmentAction::SaveModel { artwork } => super::models::save(doc, &artwork),
        AttachmentAction::InsertModel { page } => super::models::insert(doc, page),
        AttachmentAction::SaveMesh { artwork } => super::models::save_mesh(doc, &artwork),
        // Opened in `apply`, which holds the dialogs; without `3d` there is
        // no button.
        AttachmentAction::ViewModel { .. } => {}
        AttachmentAction::SetModelPoster {
            artwork,
            width,
            height,
            rgba,
        } => {
            let image = pdfcer_core::image_import::ImportedImage::from_rgba8(width, height, &rgba);
            super::models::set_poster(doc, &artwork, image.map_err(|e| e.to_string()));
        }
        AttachmentAction::SetModelViews {
            artwork,
            views,
            default,
        } => super::models::set_views(doc, &artwork, &views, default),
        AttachmentAction::PickModelPoster { artwork } => super::models::pick_poster(doc, &artwork),
        AttachmentAction::SaveModelPicture { artwork, png } => {
            super::models::save_picture(doc, &artwork, &png);
        }
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
