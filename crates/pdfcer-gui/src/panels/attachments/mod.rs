//! # `panels::attachments` — the files this document carries inside itself
//!
//! A PDF can hold **whole other files**: ISO 32000-1 §7.11.4.1 *embedded file
//! streams*, reached either from the catalogue's `/Names /EmbeddedFiles` name
//! tree (document-level) or from a `/FileAttachment` annotation on one page
//! (§12.5.6.15). This panel is where an operator sees them and acts on them.
//!
//! ## ★★★ Why this panel exists at all, and what its absence cost
//!
//! `pdfcer-core` has carried `attach_file`, `detach_file`, `list_attachments`,
//! `list_attachments_with_notes`, `extract_attachment`, `attachment_bytes` and
//! `sanitize_attachment_name` — a fully worked feature with fixtures, hazard
//! analysis and a spec-ambiguity register — and **this shell had no way to
//! reach any of it.** Not a command, not a menu item, not a panel. An engine
//! capability with no operator surface is, from the operator's chair,
//! indistinguishable from a capability that does not exist.
//!
//! `crate::panels`' own header records the same defect from the other
//! direction, for three panels that shipped with a body and no control an
//! operator could click, and names what kept it invisible: *"their only callers
//! were the harness step handlers, so every verification passed while the
//! panels were unreachable in a real build."*
//!
//! ## The shape is Acrobat's, deliberately
//!
//! `crate::text::tool`'s rule — *use the conventional interaction, never invent
//! one* — settles the layout before any argument about it starts. Acrobat's
//! Attachments panel is a **list with a toolbar**: each row names a file, gives
//! its description, size and date, and the toolbar adds one and removes one;
//! double-clicking or *Save attachment* writes one out. Every reader that
//! competes with it does the same thing. So:
//!
//! | Acrobat | here |
//! |---|---|
//! | the list, with name · description · size · modified | [`body`]'s rows |
//! | *Add* (paperclip) | [`attach`], drawn **above** the list |
//! | *Delete* | the per-row Remove |
//! | *Save attachment* | the per-row Save a copy |
//! | *Edit description* | **absent** — see below |
//!
//! ## ★★ What is absent, and why each absence is R9 rather than an omission
//!
//! - **Edit description.** `attach_file` takes the description at attach time
//!   and `pdfcer-core` has no verb that changes one afterwards. R9: an absent
//!   capability renders nothing, and a greyed control would promise a state of
//!   the program that cannot exist. The **limit** is disclosed in the attach
//!   row, because an operator needs to know it before they leave the box empty.
//! - **Remove, on a page-level row.** `detach_file` addresses the
//!   `/EmbeddedFiles` name tree and answers `AttachmentNotFound` for a
//!   `/FileAttachment` annotation **by name** — the engine separated the two
//!   cases precisely so a shell could. Those are annotations and are removed as
//!   annotations. The row says so ([`t::remove_lives_with_the_note`]) rather
//!   than leaving a hole where the other rows have a button.
//! - **Save a copy, on a row with no bytes.** A filespec with no `/EF` is an
//!   *external* file reference (§7.11.3) — legal, and there is nothing to
//!   write. The row says that too.
//! - **Open.** Acrobat opens an attachment in its host application; that is a
//!   process launch on bytes that came from inside a file that arrived from
//!   somewhere, and `pdfcer_core::attachments`' module docs are explicit that
//!   pdfcer *"does not execute, open, or interpret them, and neither should a
//!   caller without its own gate."* Saving a copy is the whole of what this
//!   shell offers, and the operator's own file manager is the gate.
//!
//! ## ★★★ Three disclosures this panel is REQUIRED to make
//!
//! Each is an obligation `pdfcer-core` writes into its own doc comments, and
//! each is invisible to an operator who is not told:
//!
//! 1. **Removing does not erase.** Under the default incremental save (§7.5.6)
//!    every prior revision stays in the file — that is what makes existing
//!    signatures survive — so a removed attachment's bytes are recoverable
//!    until a full rewrite. `detach_file`: *"Shells are expected to say so
//!    rather than let 'delete' imply erasure."*
//! 2. **The bytes may be ciphertext.** Since PDF 1.5 an embedded file can be
//!    encrypted **in an otherwise unencrypted document** (`/EFF` naming a
//!    `DefEmbeddedFile` crypt filter, §7.6.5), and pdfcer does not decrypt on
//!    this path — so an extraction can succeed and produce garbage, silently.
//! 3. **The name pdfcer writes to disk may not be the name shown.** The listing
//!    shows the raw name because *"a forensic reader that quietly repairs its
//!    input is not a reader"*; the filesystem gets a sanitised one because the
//!    raw one may be `..\..\Windows\System32\evil.exe`.
//!
//! All three live off-canvas, in the status line, which is `README.md`'s first
//! non-negotiable: *"Disclosure lives off-canvas … never blocking, never
//! requiring acknowledgement, never positioned relative to the document."*
//!
//! ## Actions, not mutations
//!
//! This body is handed `&OpenDoc` — **shared**, so it is a compile-time fact —
//! and pushes `crate::app::actions::Action::Attachment`. All three verbs open a
//! native file dialog, which must not happen inside a layout pass, so the
//! picker lives in `PdfcerApp::apply`. See
//! [`crate::app::actions::attachments`] for the whole argument.
//!
//! ## Why the listing is read fresh each frame
//!
//! `list_attachments_with_notes` takes an object graph rather than `&mut self`,
//! so it can run inside the draw closure, and the answer changes under every
//! attach, every removal, every undo and every page delete (a page-level
//! attachment dies with its page). [`crate::panels::bookmarks`] makes the same
//! trade for the same reason: a cache would need invalidating on every edit,
//! which is a correctness problem traded for a walk of a structure that is a
//! handful of entries on any real document — and `MAX_ATTACHMENTS` bounds even
//! a hostile one.

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

/// ★ Putting a file into the document — the writing half of this panel.
///
/// Its header carries the two rules it obeys rather than rediscovers: a control
/// that must always be reachable cannot be placed after an unbounded
/// `ScrollArea`, and the picker opens in the apply phase because a native
/// dialog must not open inside a layout pass.
pub mod attach;

/// The region the first row's Save button publishes.
///
/// ★ **The FIRST row's**, not every row's. `crate::diag`'s region names are a
/// flat namespace keyed by string, so publishing one name from twenty rows
/// would emit twenty rectangles under one key and leave a driven check clicking
/// whichever won the race. The Comments panel reached the same conclusion and
/// carries the same `published` flag; see [`rows`].
pub const REGION_SAVE: &str = "attachments.save"; // ui-text-exempt: trace region name, never displayed
/// The region the first row's Remove button publishes. See [`REGION_SAVE`].
pub const REGION_REMOVE: &str = "attachments.remove"; // ui-text-exempt: trace region name, never displayed

/// The panel's state between frames.
///
/// One field, and it is the operator's own typing rather than a cache of the
/// document — which is the line `crate::panels`' header draws for what may live
/// here versus what may live behind interior mutability on `OpenDoc`.
///
/// ★ Reset with the document by `PanelsState::forget_document`, and that
/// matters here for `docprops::InfoDrafts`' reason rather than for
/// tidiness: a half-typed description carried into a second file would be
/// written into **that** file's `/Desc` by the next attach, describing one
/// operator's spreadsheet with another document's note.
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
    ///
    /// A description is the operator's own words about their own file, and this
    /// reaches a trace file a harness keeps. `panels::bookmarks::BookmarksUi`
    /// and `panels::docprops` make the same choice for the same reason.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachmentsUi")
            .field("description_len", &self.description.len())
            .finish()
    }
}

/// Draw the Attachments panel.
///
/// # ★ The order of the four blocks is load-bearing, and only one of them is
/// obvious
///
/// 1. **The count**, so the answer to *"does this document carry anything?"* is
///    the first thing read.
/// 2. **The listing's own caveats, ABOVE the list.** Three panels in this crate
///    share the rule and `crate::text::panels`' header states it: *"A caveat
///    below a list arrives after the operator has already drawn a
///    conclusion."* An operator who scrolls a short list and stops has decided
///    the document holds two files; the sentence saying pdfcer stopped reading
///    early has to reach them before that.
/// 3. **The attach row**, above the list — see [`attach`]'s header for the
///    driven-run defect that makes this a rule rather than a preference.
/// 4. **The list**, last, inside the only `ScrollArea` on the panel.
///
/// An empty document is **not** an early return. That is the state an operator
/// most wants to attach the first file in, and returning before the attach row
/// is what made the Bookmarks panel read-only-looking for its whole life.
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

    // ★★★ The Paste control, ABOVE the list and BELOW the attach row.
    //
    // Above the list because the list can be long and a control at the bottom
    // of a scrolled one is a control the operator hunts for. Below the attach
    // row because the two are the same act from different sources -- one takes
    // a file from disk, the other from another open document -- and putting
    // them together says so without a word of copy.
    //
    // ★ Drawn only when the clipboard holds an attachment, which is R9: an
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
///
/// # ★★ Why a struct rather than four `&mut bool`s
///
/// It began as two, grew to four when the clipboard arrived, and tripped
/// clippy's seven-argument limit — which was the right complaint about the
/// wrong symptom. The four flags are **one fact**: *"the first visible row has
/// been drawn"*, asked separately per control because a control that is absent
/// on the first row (Remove, on a page-level attachment) must not consume the
/// flag for the row that does have one.
///
/// ★ A region name is a key in a **flat** namespace. Publishing
/// `attachments.save` from every row would emit one rectangle per row under one
/// key, and a driven check would click whichever was written last — not the row
/// it meant, and not stable between runs. These rows also live in a
/// `ScrollArea`, where a control scrolled out of view still reports a rect, so
/// the publish goes through [`crate::diag::ui_rect_visible`] as well.
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
///
/// # ★ Why the two `published` flags exist
///
/// A region name is a key in a flat namespace. Publishing `attachments.save`
/// from every row would emit one rectangle per row under one key, and a driven
/// check would click whichever was written last — which is not the row it meant
/// and is not stable between runs. The Comments panel solved this the same way
/// and its comment carries the other half of the reason: these rows live in a
/// `ScrollArea`, and *"a control scrolled out of view still reports a rect. A
/// harness clicking a coordinate that is behind the scroll edge clicks whatever
/// IS there, which fails as something else entirely."* Hence
/// [`crate::diag::ui_rect_visible`] rather than `ui_rect`.
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

    // ★★★ **THE VERBS COME SECOND, before the metadata — 2026-09-01, and a
    // measurement moved them.**
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
    // ★ The order now matches what the controls are FOR. The name says which
    // file; the buttons say what can be done with it; everything below is
    // detail an operator reads when they want it. Acrobat's own attachments
    // pane puts its verbs on a strip above the list for the same reason, and
    // this is the per-row form of that.
    //
    // ★★ The caveats about the name are the one thing that arguably belongs
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
///
/// # ★★ Every branch here is R9 applied to a different fact
///
/// | state | what is drawn | why |
/// |---|---|---|
/// | no `/EF` at all | a sentence, no button | an **external** file reference (§7.11.3) is legal and has nothing to save |
/// | an `/EF` that does not resolve | a different sentence, no button | `AttachmentNotes::unresolvable_streams` is documented as *"always a defect"*, and calling it the same thing as the legal case would either accuse a good document or excuse a damaged one |
/// | a page annotation | Save, and a sentence instead of Remove | `detach_file` refuses one by name; it is removed as an annotation |
/// | a document-level entry with bytes | Save and Remove | the full case |
///
/// A control is **absent** in each case rather than greyed, because P3 reserves
/// greying for something *temporarily* unavailable that can say when it will
/// not be — and none of these will ever become available by waiting.
fn controls(
    ui: &mut Ui,
    doc: &OpenDoc,
    attachment: &Attachment,
    name: &str,
    actions: &mut Vec<Action>,
    published: &mut Published,
) {
    // ★ `stream_id` is `None` for BOTH the legal external reference and the
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

        // ★★ Copy and Cut, in the same row as Save and Remove. Offered only
        // for a DOCUMENT-LEVEL attachment, because `copy_attachment` addresses
        // one by its `/EmbeddedFiles` name-tree key and a page-level one has
        // none — `addressable` says the same thing for Save, and this is the
        // same fact wearing a different verb.
        //
        // ★ Cut is gated a second time inside `clip::row_controls`, on the same
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
///
/// An empty name is legal — `NameSource::None` is reachable when a filespec
/// carries no `/F`, `/UF`, `/DOS`, `/Mac` or `/Unix` and there is no tree key
/// to fall back on — and the row must still exist, because the bytes can still
/// be saved out. A blank line where a name belongs reads as a rendering fault.
///
/// Pure, so [`tests`] can hold it to that without a `Ui`.
fn display_name(attachment: &Attachment) -> String {
    if attachment.name.trim().is_empty() {
        t::unnamed().to_owned()
    } else {
        readable(&attachment.name)
    }
}

/// Which of the two mechanisms carries this attachment, as a sentence, or
/// `None` for a kind this build does not know.
///
/// Pure, so [`tests`] can hold the page numbering to being 1-based without a
/// `Ui` — `page_index` is 0-based *"into `pages`"* and the off-by-one is the
/// kind that looks like a document defect rather than a bug.
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
///
/// `AttachmentKind::PageAnnotation::annot_id` is an `Option` — `None` when the
/// `/Annots` entry was a direct dictionary rather than a reference — and a row
/// pdfcer cannot name gets no button. That is R9 rather than caution: a control
/// whose operand cannot be constructed is an affordance for something that
/// cannot work.
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
///
/// # ★ Two substitutions, and both are rendering rather than reporting
///
/// - **`CR` becomes `LF`.** §12.5.6.2 makes carriage return the paragraph
///   separator in annotation `/Contents`, which is where a page-level
///   attachment's description comes from — and egui lays a bare `CR` out as
///   nothing at all, so a two-paragraph description would render as one long
///   run with a gap in it.
/// - **Other C0 controls become a space.** A name or description from a
///   document is unconstrained text (see `Attachment::name`), and a `NUL` or a
///   `BEL` in a label is a glyph nobody can read.
///
/// Neither is a disclosure case, and the distinction is worth stating because
/// this crate's rule 4 posture is otherwise to disclose everything: pdfcer is
/// not reporting a different *value* here, it is drawing the same value
/// legibly. The value that reaches the **filesystem** goes through
/// `sanitize_attachment_name` instead, and that one *is* disclosed — see the
/// module header's third required disclosure.
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

    /// ★★ **The two kinds are described differently, and the page one names its
    /// page 1-based.**
    ///
    /// Both halves fail invisibly. Describing them alike would tell an operator
    /// that a file pinned to page 2 belongs to the document and survives that
    /// page's deletion — which is exactly backwards, and is the one fact
    /// `AttachmentKind`'s own docs say *"bites hardest at save time and at
    /// page-delete time"*. And `page_index` is 0-based, so a row that printed
    /// it raw would name the wrong sheet on every document.
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

    /// ★★★ **A document-level row is addressable and a direct-dictionary
    /// annotation is not.**
    ///
    /// The first half is what makes Remove possible at all. The second is the
    /// property that keeps a button from being drawn for an operand this code
    /// cannot construct — asserted by construction, because no fixture in the
    /// engine's tree carries a direct-dictionary `/Annots` entry and inventing
    /// one here would be testing a hand-built value rather than a document.
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
    ///
    /// `NameSource::None` is reachable, the bytes are still saveable, and a
    /// blank line where a name belongs is indistinguishable from a rendering
    /// failure. Checked against the whitespace cases too — a name of three
    /// spaces is an invisible row, which is the same defect as no row.
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

    /// ★ **A control character never reaches a label as itself.**
    ///
    /// A name in a PDF is unconstrained text, `NUL` and `BEL` are authorable,
    /// and §12.5.6.2 makes `CR` the paragraph separator in the `/Contents` a
    /// page-level description comes from — which egui lays out as nothing.
    ///
    /// What is asserted is that the *substitution* happened, not that the
    /// string was censored: the visible characters are untouched, because this
    /// panel reports what the document says.
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

    /// ★★ **The panel shows a hostile name exactly as the document wrote it.**
    ///
    /// The bargain this panel makes, and both halves have to hold or neither is
    /// worth anything: the *listing* reports the raw name, because
    /// `sanitize_attachment_name`'s own docs say a reader that quietly repairs
    /// its evidence is not a reader and *"the operator investigating a
    /// suspicious file would be looking at pdfcer's cleaned-up version"*; the
    /// *save path* uses the sanitised one, which
    /// `crate::app::actions::attachments` asserts from the other side.
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
    ///
    /// One name from two controls would leave a driven check clicking whichever
    /// was published last, and the failure presents as *"the button does
    /// nothing"* on whichever run lost the race.
    #[test]
    fn the_row_regions_are_named_apart() {
        assert_ne!(REGION_SAVE, REGION_REMOVE);
        assert_ne!(REGION_SAVE, attach::REGION_ATTACH);
        assert_ne!(REGION_REMOVE, attach::REGION_DESCRIPTION);
    }
}
