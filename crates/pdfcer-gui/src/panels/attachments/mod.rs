//! # `panels::attachments` — the files this document carries inside itself
//!
//! A PDF can hold **whole other files**: ISO 32000-1 §7.11.4.1 *embedded file
//! streams*, reached either from the catalogue's `/Names /EmbeddedFiles` name
//! tree (document-level) or from a `/FileAttachment` annotation on one page
//! (§12.5.6.15). This panel is where an operator sees them and acts on them.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/attachments/mod.md`.

/// Copy, cut and paste an embedded file. Its own module because the paste
/// carries a disclosure obligation the rest of this panel does not.
pub(crate) mod clip;

use egui::Ui;
use pdfcer_core::attachments::{Attachment, AttachmentKind, NameSource};

use crate::app::actions::Action;
use crate::app::actions::attachments::{AttachmentAction, AttachmentRef};
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::panels::attachments as t;

/// Putting a file into the document — the writing half of this panel.
pub mod attach;

/// The 3D models section.
pub mod models;

mod listing_tests;

/// The region the first row's Save button publishes.
pub const REGION_SAVE: &str = "attachments.save"; // ui-text-exempt: trace region name, never displayed
/// The region the first row's Remove button publishes. See [`REGION_SAVE`].
pub const REGION_REMOVE: &str = "attachments.remove"; // ui-text-exempt: trace region name, never displayed

/// The panel's state between frames.
#[derive(Default)]
pub struct AttachmentsUi {
    /// What has been typed into the optional description field.
    ///
    /// It can only be spent once: `attach_file` takes the description at attach
    /// time and there is no verb that edits one afterwards, so [`attach::show`]
    /// clears this on the press rather than letting it follow the operator to
    /// the next file.
    pub(super) description: String,
}

impl std::fmt::Debug for AttachmentsUi {
    /// The draft's **length**, not its text.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachmentsUi")
            .field("description_len", &self.description.len())
            .finish()
    }
}

/// Draw the Attachments panel.
pub fn body(ui: &mut Ui, doc: &OpenDoc, state: &mut PanelsState, actions: &mut Vec<Action>) {
    let (listed, notes) = {
        let view = doc.session.view();
        pdfcer_core::attachments::list_attachments_with_notes(&view)
    };

    // The trace is the only oracle a driven check has while the operator is at
    // the machine: a screenshot harness would seize their screen, and this
    // panel's whole subject is invisible on the page.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "attachments-panel count={} document_level={} page_level={} notes={}",
            listed.len(),
            listed
                .iter()
                .filter(|a| matches!(a.kind, AttachmentKind::DocumentLevel { .. }))
                .count(),
            listed
                .iter()
                .filter(|a| matches!(a.kind, AttachmentKind::PageAnnotation { .. }))
                .count(),
            t::listing_notes(&notes).len()
        )
    });

    ui.label(t::count(listed.len()));
    for said in t::listing_notes(&notes) {
        ui.label(egui::RichText::new(said).small().weak());
    }

    ui.separator();
    attach::show(ui, state.attachments_mut(), actions);

    // The Paste control, ABOVE the list and BELOW the attach row.
    //
    // Above the list because the list can be long and a control at the bottom
    // of a scrolled one is a control the operator hunts for. Below the attach
    // row because the two are the same act from different sources -- one takes
    // a file from disk, the other from another open document -- and putting
    // them together says so without a word of copy.
    //
    // Drawn only when the clipboard holds an attachment, which is R9: an
    // unavailable capability renders NOTHING. It is not greyed, because greying
    // is reserved for the temporarily unavailable and an operator with an empty
    // clipboard is not waiting for anything.
    //
    // It takes the names ALREADY LISTED, because the paste has to say whether
    // it will replace one -- `attach_file` retains-then-pushes, so a same-named
    // attachment is displaced silently. See `clip`'s header.
    let existing: Vec<String> = listed.iter().map(|a| a.name.clone()).collect();
    clip::paste_control(ui, &existing, actions);
    ui.separator();
    models::section(ui, doc, actions);

    if listed.is_empty() {
        ui.label(t::empty());
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("attachment-rows")
        .show(ui, |ui| {
            rows(ui, doc, &listed, actions);
        });
}

/// **Which of this panel's row controls have already published a rectangle.**
#[derive(Debug, Default)]
struct Published {
    /// `attachments.save`.
    save: bool,
    /// `attachments.remove`.
    remove: bool,
    /// `attachments.copy`.
    copy: bool,
    /// `attachments.cut`.
    cut: bool,
}

/// Draw one row per attachment.
fn rows(ui: &mut Ui, doc: &OpenDoc, listed: &[Attachment], actions: &mut Vec<Action>) {
    let mut published = Published::default();
    for attachment in listed {
        row(ui, doc, attachment, actions, &mut published);
        ui.separator();
    }
}

/// One attachment: what it is, what pdfcer could not vouch for, and what can be
/// done with it.
fn row(
    ui: &mut Ui,
    doc: &OpenDoc,
    attachment: &Attachment,
    actions: &mut Vec<Action>,
    published: &mut Published,
) {
    // The name, RAW. `Attachment::name` is what the document says, and this
    // panel's job is to report that — see the module header's third required
    // disclosure for the other half of the bargain, which is that pdfcer never
    // hands this string to the filesystem.
    let name = display_name(attachment);
    ui.label(name.clone());

    //
    // They were last, after the name, the location, the description, the size,
    // the dates, the claimed type and two possible caveats about the name. On a
    // default Edit layout the Attachments panel body is about **182 pt tall**,
    // and the attach row above the list ends at roughly two thirds of it — so
    // with **one** attachment listed, not one of its buttons was on screen.
    //
    // Measured, not guessed: a driven check reported that `attachments.save`,
    // `attachments.remove` and both new clipboard controls declared no
    // rectangle at all, while `attachments.attach` and `attachments.description`
    // (which are ABOVE the list) declared theirs. `ui_rect_visible` suppresses a
    // clipped rect, so "no rectangle" is precisely "off the bottom of the
    // panel".
    //
    // ⇒ An operator with a single attached file had to scroll a panel that
    // looked complete in order to find any verb at all. That is the shape of
    // defect this project keeps finding — a control that exists, is correct, and
    // is unreachable — and it is invisible to every test that does not render.
    //
    // The order now matches what the controls are FOR. The name says which
    // file; the buttons say what can be done with it; everything below is
    // detail an operator reads when they want it. Acrobat's own attachments
    // pane puts its verbs on a strip above the list for the same reason, and
    // this is the per-row form of that.
    //
    // The caveats about the name are the one thing that arguably belongs
    // above the buttons, and they stay below deliberately: they qualify the
    // NAME, they are drawn in small weak text, and hoisting a conditional block
    // above the verbs would make the buttons move up and down as the operator
    // scrolls a list of mixed rows -- which is worse than reading them second.
    controls(ui, doc, attachment, &name, actions, published);

    if let Some(said) = where_it_lives(&attachment.kind) {
        ui.label(egui::RichText::new(said).small().weak());
    }
    if let Some(description) = &attachment.description {
        ui.label(egui::RichText::new(readable(description)).small().weak());
    }
    ui.label(
        egui::RichText::new(t::size(attachment.declared_size, attachment.size_check))
            .small()
            .weak(),
    );
    if let Some(dates) = t::dates(
        attachment.created.as_deref(),
        attachment.modified.as_deref(),
    ) {
        ui.label(egui::RichText::new(dates).small().weak())
            .on_hover_text(t::date_tooltip());
    }
    if let Some(mime) = &attachment.mime {
        ui.label(egui::RichText::new(t::kind_claimed(mime)).small().weak());
    }

    // What pdfcer could not vouch for about this row, in the order an operator
    // reads: the name first (it is the thing they are looking at), then whether
    // there are bytes at all.
    if !attachment.name_exact {
        ui.label(egui::RichText::new(t::name_is_approximate()).small().weak());
    }
    if attachment.name_source == NameSource::TreeKey {
        ui.label(
            egui::RichText::new(t::name_is_the_index_key())
                .small()
                .weak(),
        );
    }
}

/// The row's verbs, and the sentences that stand where a verb cannot.
fn controls(
    ui: &mut Ui,
    doc: &OpenDoc,
    attachment: &Attachment,
    name: &str,
    actions: &mut Vec<Action>,
    published: &mut Published,
) {
    // `stream_id` is `None` for BOTH the legal external reference and the
    // damaged dangling one, so the two are told apart by the size check, which
    // is the only place the engine records the difference:
    // `DeclaredSizeCheck::NoStream` covers both, and
    // `AttachmentNotes::unresolvable_streams` counts only the second — at the
    // listing level, not per row. What a row can honestly say is therefore the
    // weaker of the two sentences, and the stronger one is in the listing's
    // notes above. Stating the strong one here would accuse a document that
    // legitimately points at a file on disk.
    if attachment.stream_id.is_none() {
        ui.label(egui::RichText::new(t::no_bytes()).small().weak());
    }

    ui.horizontal(|ui| {
        if attachment.stream_id.is_some()
            && let Some(at) = addressable(&attachment.kind)
        {
            let save = ui.button(t::save_button()).on_hover_text(t::save_tooltip());
            if !published.save {
                crate::diag::ui_rect_visible(REGION_SAVE, save.rect, ui.clip_rect());
                published.save = true;
            }
            if save.clicked() {
                actions.push(Action::Attachment(AttachmentAction::SaveCopy {
                    at,
                    name: name.to_owned(),
                }));
            }
        }

        // Copy and Cut, in the same row as Save and Remove. Offered only
        // for a DOCUMENT-LEVEL attachment, because `copy_attachment` addresses
        // one by its `/EmbeddedFiles` name-tree key and a page-level one has
        // none — `addressable` says the same thing for Save, and this is the
        // same fact wearing a different verb.
        //
        // Cut is gated a second time inside `clip::row_controls`, on the same
        // predicate Remove uses. Two gates for one rule reads like belt and
        // braces and is not: the outer one decides whether a KEY exists, the
        // inner whether a DELETE is possible, and a page-level attachment fails
        // both for different reasons.
        if let AttachmentKind::DocumentLevel { tree_key } = &attachment.kind {
            super::attachments::clip::row_controls(
                ui, doc, tree_key, name, true, published, actions,
            );
        }

        match &attachment.kind {
            AttachmentKind::DocumentLevel { tree_key } => {
                let remove = ui
                    .button(t::remove_button())
                    .on_hover_text(t::remove_tooltip());
                if !published.remove {
                    crate::diag::ui_rect_visible(REGION_REMOVE, remove.rect, ui.clip_rect());
                    published.remove = true;
                }
                if remove.clicked() {
                    actions.push(Action::Attachment(AttachmentAction::Detach {
                        key: tree_key.clone(),
                        name: name.to_owned(),
                    }));
                }
            }
            AttachmentKind::PageAnnotation { .. } => {
                ui.label(
                    egui::RichText::new(t::remove_lives_with_the_note())
                        .small()
                        .weak(),
                );
            }
            // `AttachmentKind` is `#[non_exhaustive]`. A kind this build has
            // never seen gets **no verb at all**, which is the only safe
            // default: a Remove button whose operand this code could not
            // construct would be an affordance for an act it cannot perform.
            _ => {}
        }
    });
}

/// What the row calls this attachment.
fn display_name(attachment: &Attachment) -> String {
    if attachment.name.trim().is_empty() {
        t::unnamed().to_owned()
    } else {
        readable(&attachment.name)
    }
}

/// Which of the two mechanisms carries this attachment, as a sentence, or
/// `None` for a kind this build does not know.
fn where_it_lives(kind: &AttachmentKind) -> Option<String> {
    match kind {
        AttachmentKind::DocumentLevel { .. } => Some(t::where_document().to_owned()),
        AttachmentKind::PageAnnotation { page_index, .. } => {
            Some(t::where_page(page_index.saturating_add(1)))
        }
        // A kind added to the engine after this build says **nothing** rather
        // than guessing at one of the two it knows. The two have different
        // lifetimes, and claiming the wrong one would tell an operator their
        // file survives a page delete when it does not.
        _ => None,
    }
}

/// How this attachment can be addressed after the frame, or `None` when it
/// cannot be.
fn addressable(kind: &AttachmentKind) -> Option<AttachmentRef> {
    match kind {
        AttachmentKind::DocumentLevel { tree_key } => Some(AttachmentRef::DocumentLevel {
            key: tree_key.clone(),
        }),
        AttachmentKind::PageAnnotation { annot_id, .. } => {
            annot_id.map(|annot| AttachmentRef::PageAnnotation { annot })
        }
        _ => None,
    }
}

/// One string, safe to lay out in a single-line-ish label.
fn readable(raw: &str) -> String {
    raw.chars()
        .map(|c| match c {
            '\r' => '\n',
            '\n' | '\t' => c,
            c if c.is_control() => ' ',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::objects::test_support::engine_fixture;

    /// Everything the engine's two-kinds fixture lists.
    fn both_kinds() -> Vec<Attachment> {
        let path = engine_fixture("attachments/both-kinds.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        pdfcer_core::attachments::list_attachments(&doc)
    }

    /// **The two kinds are described differently, and the page one names its
    /// page 1-based.**
    #[test]
    fn the_two_kinds_are_described_differently_and_the_page_is_one_based() {
        let listed = both_kinds();
        let document_level = listed
            .iter()
            .find(|a| matches!(a.kind, AttachmentKind::DocumentLevel { .. }))
            .expect("the fixture carries a document-level attachment");
        let page_level = listed
            .iter()
            .find(|a| matches!(a.kind, AttachmentKind::PageAnnotation { .. }))
            .expect("the fixture carries a page-level attachment");

        let doc_said = where_it_lives(&document_level.kind).expect("a sentence");
        let page_said = where_it_lives(&page_level.kind).expect("a sentence");
        assert_ne!(doc_said, page_said);

        let AttachmentKind::PageAnnotation { page_index, .. } = &page_level.kind else {
            unreachable!() // ui-text-exempt: test control flow, never displayed
        };
        assert!(
            page_said.contains(&(page_index + 1).to_string()),
            "the row must name the human page number: {page_said}"
        );
    }

    /// **A document-level row is addressable and a direct-dictionary
    /// annotation is not.**
    #[test]
    fn only_a_nameable_attachment_gets_a_verb() {
        let listed = both_kinds();
        for attachment in &listed {
            assert!(
                addressable(&attachment.kind).is_some(),
                "both of this fixture's entries are indirect and must be addressable"
            );
        }
        // A page annotation whose `/Annots` entry was a direct dictionary
        // reports no id, and must therefore offer nothing.
        let unnameable = AttachmentKind::PageAnnotation {
            page_index: 0,
            page_id: pdfcer_core::object::ObjId::new(1, 0),
            annot_id: None,
            icon: None,
        };
        assert!(addressable(&unnameable).is_none());
    }

    /// **An unnamed attachment still gets a row label.**
    #[test]
    fn an_unnamed_attachment_is_labelled_rather_than_blank() {
        assert!(!t::unnamed().trim().is_empty());
        for blank in ["", " ", "\t\n"] {
            assert!(
                blank.trim().is_empty(),
                "this pins the inputs the row's emptiness test must catch"
            );
        }
    }

    /// **A control character never reaches a label as itself.**
    #[test]
    fn a_control_character_is_made_legible_without_changing_the_words() {
        let said = readable("first\rsecond\u{0}third");
        assert!(said.contains("first"), "{said}");
        assert!(said.contains("second"), "{said}");
        assert!(said.contains("third"), "{said}");
        assert!(
            !said.contains('\r'),
            "a bare CR lays out as nothing: {said:?}"
        );
        assert!(!said.contains('\u{0}'), "{said:?}");
        assert!(
            said.contains('\n'),
            "the paragraph break survives: {said:?}"
        );
        // Ordinary text passes through untouched.
        assert_eq!(readable("quote.xlsx"), "quote.xlsx");
    }

    /// **The panel shows a hostile name exactly as the document wrote it.**
    #[test]
    fn a_hostile_name_is_shown_and_not_quietly_repaired() {
        let path = engine_fixture("attachments/hostile-names.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let listed = pdfcer_core::attachments::list_attachments(&doc);
        let traversal = listed
            .iter()
            .find(|a| a.name.contains("..") || a.name.contains('/') || a.name.contains('\\'));
        let Some(traversal) = traversal else {
            panic!("this fixture exists to carry a path-shaped name") // ui-text-exempt: test panic, never displayed
        };
        let shown = display_name(traversal);
        assert!(
            shown.contains("..") || shown.contains('/') || shown.contains('\\'),
            "the row must show the traversal that makes the file suspicious: {shown:?}"
        );
        // …and the sanitiser disagrees with it, which is the point.
        assert_ne!(traversal.safe_name().value, traversal.name);
    }

    /// **The two published regions are named apart.**
    #[test]
    fn the_row_regions_are_named_apart() {
        assert_ne!(REGION_SAVE, REGION_REMOVE);
        assert_ne!(REGION_SAVE, attach::REGION_ATTACH);
        assert_ne!(REGION_REMOVE, attach::REGION_DESCRIPTION);
    }
}
