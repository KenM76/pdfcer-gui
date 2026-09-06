//! # text::commands — the label and tooltip of every ribbon command
//!
//! One function per command, each returning a [`CommandText`]. The ribbon's
//! *structural* strings — tab labels, tab questions, group captions, mode
//! labels — live next door in [`crate::text::ribbon`].
//!
//! ## Why a pair rather than two functions
//!
//! Every command needs both a label and a tooltip, and they are written
//! together: the tooltip's job is to say what the label cannot fit, so
//! reviewing one without the other is reviewing half a sentence. Two
//! functions per command would also double a file that is already the
//! longest in the catalog, for no gain — nothing ever wants one without
//! being able to reach the other.
//!
//! ## Every command has a tooltip. That is a rule, not an accident.
//!
//! `RIBBON_IA.md` P3 reserves greying for *temporarily* unavailable — no
//! document open, undo stack empty — and requires that it *"is always
//! explained on hover."* A command with no tooltip cannot honour that, so
//! [`CommandText`] has no way to express "no tooltip" and a test below
//! asserts none is empty.
//!
//! The salvage source got this wrong in exactly one place and it is
//! instructive: the four Measure buttons (`Linear Dimension`,
//! `Radius / Diameter Dimension`, `Set Group Scale…`, `Manage Dimension
//! Groups…`) were rendered as text-only selectables **with no tooltip at
//! all** — the four controls on the tab most likely to be used by someone
//! who has never used a PDF measuring tool.
//!
//! ## Voice, carried across from the salvage source deliberately
//!
//! pdfcer's tooltips are unusually long and unusually specific, and that is
//! a deliberate quality of the product rather than an accident of who
//! wrote them. They say what a command *changes* ("This changes the
//! document, not just the view"), what it *cannot* do, and what is
//! *irreversible*. Where the salvage source's wording said something worth
//! keeping, it is kept close to verbatim.
//!
//! ★★★ **The two examples this paragraph used to quote were both FALSE by
//! 2026-09-05, and they were quoted here as models of good voice.** They were
//! *"pdfcer does not check whether they are valid"* (untrue once
//! `signature::verify_all_with_trust` was wired) and *"Marking is reversible;
//! applying is not"* (untrue on the default destination once `Pass 250.2`
//! made applying stage the next save). Both have been corrected in place —
//! see [`edit_redact`] and [`crate::text::commands::view::view_panel_signatures`].
//!
//! ⇒ **The lesson is about the QUOTING, not the sentences.** A header that
//! holds up a live string as an exemplar makes a second copy of that string's
//! claim, in a file nobody edits when the claim expires. Name the *shape* of
//! the good sentence, not its text. The examples that survive above are the
//! ones that describe a permanent property of a tooltip rather than a
//! measurement of the build.
//!
//! Two things are trimmed:
//!
//! 1. **Tooltips that enumerate the alternatives.** The old `Add Text`
//!    tooltip explained itself by contrast with three other commands over
//!    four sentences. One contrast is a clarification; three is a menu.
//! 2. **Tooltips that describe a defect.** `"click-to-place editing on the
//!    canvas is coming"` is a roadmap entry, not a tooltip.
//!
//! ## Labels: three renames that `RIBBON_IA.md` §5.4 requires
//!
//! `Aa`, `I⁺ Aa` and `Obj` become **Edit text**, **Add text** and **Edit
//! objects**. They are the primary content-editing tools and were the
//! least legible controls in the application — and the first two returned
//! the *same literal*, `"Aa"`, distinguished only by icon and tooltip.

/// The two operator-visible strings a ribbon command carries.
///
/// A plain pair of `&'static str` rather than owned `String`s because
/// every one of them is a literal in this file: the catalog is the
/// definition site, so there is nothing to allocate and a command's text
/// can be read in a `const` context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandText {
    /// What the control says. Sentence case, no trailing period; an
    /// ellipsis when activating it opens a dialog rather than acting.
    pub label: &'static str,
    /// What the control says on hover. A full sentence, with punctuation.
    pub tooltip: &'static str,
}

impl CommandText {
    /// Pair a label with its tooltip.
    #[must_use]
    pub const fn new(label: &'static str, tooltip: &'static str) -> Self {
        Self { label, tooltip }
    }
}

/// **The View tab's entries**, split out on 2026-08-20 when this file crossed
/// rule R2's ceiling.
///
/// Re-exported below, so nothing changed for a caller: every call site still
/// writes `crate::text::commands::view_zoom_in()`. See that module's header
/// for why the seam was drawn there and nowhere else.
/// ★ The File tab's Save As copy, split out on 2026-09-02 under R2 — the same
/// seam [`annotate`] and [`view`] already are. Re-exported so callers keep
/// spelling it `text::commands::file_save_as`.
mod file;
pub use file::{file_export_image, file_export_text, file_save_as};
mod view;

pub use view::*;

// ===========================================================================
// FILE TAB
// ===========================================================================

/// `file.open`
#[must_use]
pub const fn file_open() -> CommandText {
    CommandText::new("Open…", "Open a PDF document (Ctrl+O).")
}

/// `file.new`
///
/// **`New`, with no ellipsis**, and that is the label carrying a promise: an
/// ellipsis says *this will ask you something*, and this does not. Two of the
/// three reference applications create a document immediately from a default
/// (Acrobat from a locale default, Inkscape from its default template) and
/// only SolidWorks asks — and what it asks is which *kind* of document, a
/// question pdfcer has no analogue for. See `crate::app::blank` §3.
///
/// The tooltip states the page size **because the command does not ask for
/// it**. A default that is never mentioned is a default an operator discovers
/// by measuring the page they just made, and A4 is a real choice — argued from
/// what the three reference applications do, and from this operator's own
/// A-series drawings — rather than an accident to be hidden.
///
/// ★ **The last sentence was a claim, and it stopped being true on
/// 2026-08-14.**
///
/// It read: *"This build cannot yet write a document to disk, so a new document
/// lasts as long as the window does."* That was accurate when it was written —
/// `file.save_copy` was registered with no dispatch arm — and `file.save_copy`
/// is now wired, so leaving it would have told the operator that the document
/// in front of them cannot be kept when a control two groups away keeps it.
///
/// It is replaced rather than deleted, because the thing an operator will
/// otherwise find out the hard way has changed rather than gone: New is still
/// the command where the *shape* of saving bites first. `Save a copy…` asks for
/// a destination every time and never adopts it, so a created document keeps
/// its `Untitled` name however often it is saved — which is what the new
/// sentence says, and which is Inkscape's behaviour for the same verb. See
/// `crate::app::save` §3.4.
///
/// The record of the correction is kept here for the reason `HANDOFF.md` §10
/// gives about prose that quotes a fact: this is the fifth such drift the
/// project has recorded, and the only defence that works is noticing them at
/// the site of the change that invalidated them.
#[must_use]
pub const fn file_new() -> CommandText {
    CommandText::new(
        "New",
        "Make a new document: one blank A4 page (Ctrl+N). It replaces what is open. Use Save a \
         copy to keep it; it is asked where to write every time, so the document itself stays \
         untitled.",
    )
}

/// `file.new_from_template`
///
/// # ★ The label follows `RIBBON_IA.md` and the tooltip corrects for it
///
/// §5.1 specifies the row as `New from template… (page size)`, following
/// Inkscape's `Ctrl+Alt+N`. What this shell offers is page sizes and not a
/// template gallery, so the word "template" over-promises — and the IA is
/// settled and reviewed, so a session may propose an amendment and may not
/// make one.
///
/// The tooltip is therefore doing real work rather than restating the label:
/// it says **page size** in its first four words, so an operator hovering
/// before they click learns what the window offers without opening it. See
/// `crate::dialogs::new_document`'s header for the full argument.
#[must_use]
pub const fn file_new_from_template() -> CommandText {
    CommandText::new(
        "New from template…",
        "Choose a page size and make a new document: A0 to A6, Letter, Legal, Tabloid, the ANSI \
         engineering sizes, or a size you type. It replaces what is open.",
    )
}

/// `file.close`
///
/// ★★ **The tooltip that was a promise nothing kept, from the day it shipped
/// until 2026-08-19.** *"You are asked what to do about unsaved edits first"* —
/// and nothing asked. `Action::Close` consulted `save_pending`, which is
/// permanently `false` by design, and then dropped the `EditSession`. Every
/// edit made since the file was opened went with it, silently, with no prompt
/// and no undo.
///
/// The sentence is **unchanged**, because it was never wrong about what pdfcer
/// should do — it was a specification sitting on the ribbon, and the build had
/// not met it. `crate::dialogs::unsaved` is the surface that now does.
///
/// The generalisable half is worth keeping here, where the next tooltip gets
/// written: **an operator-visible string that describes behaviour is a claim,
/// and nothing in this project checks a claim of that shape.** The ui-strings
/// gate asserts the string lives in `text/`; the catalog tests assert it is a
/// sentence and that no two labels collide; no gate can ask whether it is
/// *true*. This one was found by an outside audit, three weeks after the fact,
/// by someone reading the tooltip and then reading the code.
#[must_use]
pub const fn file_close() -> CommandText {
    CommandText::new(
        "Close",
        "Close this document (Ctrl+W). You are asked what to do about unsaved edits \
         first. Your other open documents stay open.",
    )
}

/// `file.recent`
///
/// **The control this text belongs to is not a button.** `file.recent` is
/// drawn by the `recent_files` custom item in File ▸ File — a menu of the
/// documents the operator had open — so this label and tooltip are what the
/// *menu button* says, and the rows inside it are file names from
/// [`crate::text::files`]. See `crate::shell::manifest::CUSTOM_BACKED`.
///
/// The tooltip names the two behaviours an operator would otherwise have to
/// discover: the cap, and the fact that a document on a drive which is not
/// connected right now is hidden rather than forgotten.
#[must_use]
pub const fn file_recent() -> CommandText {
    CommandText::new(
        "Recent",
        "Open one of the last ten documents you had open. A document stored on a drive that \
         is not connected right now is hidden from the list until it comes back; it is not \
         forgotten.",
    )
}

/// `file.save`
///
/// ★★★ **Save. In place. Added 2026-08-20, on the operator:** *"can I please
/// have a save button like every other program in existence has? We're on week
/// two of this and just have a save as button."*
///
/// # The argument that used to stand here, and why it does not
///
/// [`file_save_copy`]'s doc comment said, and still says of itself:
///
/// > *"A button labelled `Save` would promise in-place saving, which cannot
/// > ship before autosave and crash recovery exist."*
///
/// That was a real position rather than an oversight, and it is **weaker than
/// it looks, for a reason specific to this application**: pdfcer writes an
/// INCREMENTAL UPDATE. The new revision is appended; the previous one stays in
/// the file, byte for byte, reachable through its own cross-reference table.
/// An in-place save here does not overwrite the operator's document in the
/// sense the objection assumed — **the format is the crash recovery**, and it
/// was already shipping.
///
/// What remained genuinely unsafe was the WRITE, not the save: `fs::write`
/// truncates and then streams, so a crash mid-write leaves a partial file where
/// a whole one was. That is a solved problem, and `save::save_in_place` solves
/// it — materialise the replacement in a temporary beside the target, then
/// rename, which either happens or does not.
///
/// So the honest account is not *"the operator overruled a safety rule"*. It is
/// that the rule was aimed at the wrong hazard, and the right hazard has a
/// three-line answer that had not been written because nobody was asking the
/// question. That is the same shape as `Ctrl+P` never being bound.
///
/// # The description names the incremental behaviour on purpose
///
/// Because an operator who has been told for a fortnight that pdfcer *never*
/// overwrites deserves to know exactly what changed, and because "the previous
/// version stays inside the file" is the fact that makes pressing this button
/// comfortable.
#[must_use]
pub const fn file_save() -> CommandText {
    CommandText::new(
        "Save",
        "Save this document over the file you opened (Ctrl+S). The edits are appended as an \
         update, so the previous version stays inside the file and nothing is thrown away. Use \
         Save a copy to write somewhere else instead.",
    )
}

/// `file.save_copy`
///
/// The label is `Save a copy…`, not `Save`, and that is load-bearing:
/// pdfcer writes the edits as an incremental update to a file you name, and
/// never overwrites the original unless you pick it. A button labelled
/// `Save` would promise in-place saving, which cannot ship before autosave
/// and crash recovery exist.
#[must_use]
pub const fn file_save_copy() -> CommandText {
    CommandText::new(
        "Save a copy…",
        "Write the document, including unsaved edits, to a file you choose (Ctrl+Shift+S). The \
         original is never overwritten unless you pick it, and the edits are appended as an \
         update so the previous version stays intact inside the file.",
    )
}

/// `file.save_compacted`
///
/// ★★★ **The name is the disclosure**, and it is the first line of defence
/// against a press nobody meant. `OPERATOR_REQUESTS.md` **O48** asked for it
/// *"named so it cannot be pressed by accident"*, and *Save a compacted copy…*
/// is a phrase nobody reaches for while looking for Save.
///
/// ★★ The tooltip names **both losses before the gain**. That inverts the usual
/// order and it is deliberate: an operator scanning tooltips reads the first
/// clause, and the first clause of this one has to be the part they cannot see.
/// A smaller file needs no advocacy — it is why they are hovering.
#[must_use]
pub const fn file_save_compacted() -> CommandText {
    CommandText::new(
        "Save a compacted copy…",
        "Write the whole document fresh to a file you choose, dropping anything no longer \
         used. Unlike Save a copy, this does NOT keep the previous version inside the \
         file and CANNOT keep a digital signature — so the copy may be much smaller, \
         and your original is left untouched. Use it after removing pages, images or \
         embedded fonts.",
    )
}

/// `edit.reflow_block`
///
/// ★★★ **The label says what it does to a paragraph, and the tooltip says the
/// one thing that will otherwise surprise.** `OPERATOR_REQUESTS.md` **O54**.
///
/// A reflow is planned against the document *as opened* — it needs position
/// information the in-session staging buffer does not carry — so it refuses a
/// page this session has already changed. One typed character is enough. That
/// is a correctness property rather than a limitation (the alternative is
/// splicing offsets into a stream that has moved), and the remedy is specific,
/// so both are in the tooltip where an operator meets them before the refusal
/// rather than after it.
///
/// ## ★★★ Rewritten 2026-09-04 — `OPERATOR_REQUESTS.md` **O127**, defect 3
///
/// > *"I also haven't seen the reflow option actually work with anything when I
/// > press it."*
///
/// The sentence above **presupposed a caret**: *"the paragraph the caret is
/// in"* tells somebody who already has one what will happen, and tells somebody
/// who has not that they are missing something without naming it. And it named
/// only one of the two preconditions.
///
/// ⇒ The tooltip now **leads with what must be true before the press**, in the
/// order the operator has to satisfy them, using the words on the buttons they
/// have to press (*Edit text*, not "place the caret"). R9's obligation is that a
/// control which requires something says so before it is pressed; this is that
/// sentence, and the `⊗` decline is the one after.
///
/// ★ The third fact — that it only works on prose, not on a title-block cell or
/// an isolated label — is deliberately here too. It is the most likely refusal
/// on the drawings this program is for, and an operator who reads it here stops
/// pressing the control on a dimension label and wondering.
#[must_use]
pub const fn edit_reflow_block() -> CommandText {
    CommandText::new(
        "Reflow paragraph",
        "Re-wrap a paragraph so its lines fill their box again, after retyping a sentence that \
         made a line too long or too short. First choose Edit text and click inside the \
         paragraph, then press this. It needs real prose — a title-block cell or a single \
         label is not a paragraph and cannot be re-wrapped — and it works on the document as \
         you opened it, so if you have already changed this file, save it and open it again \
         first.",
    )
}

/// `file.export_dxf`
#[must_use]
pub const fn file_export_dxf() -> CommandText {
    CommandText::new(
        "Export DXF…",
        "Write this page's lines, curves and text out as a DXF file that CAD and CNC software \
         can open.",
    )
}

/// `file.import_form_data`
///
/// ★★ The tooltip says **what it overwrites**, because that is the fact an
/// operator needs before pressing rather than after. An import sets values in
/// the document they have open; a data file that names a field they have
/// already filled replaces what is in it. One `Ctrl+Z` takes the whole import
/// back — the engine makes it a single command however many fields it sets —
/// and saying so is what makes the press safe to try.
#[must_use]
pub const fn file_import_form_data() -> CommandText {
    CommandText::new(
        "Import form data…",
        "Fill this document's form from an FDF, XFDF or CSV file, replacing any values it \
         names. One Ctrl+Z takes the whole import back.",
    )
}

/// `file.export_form_data`/// `file.export_form_data`
#[must_use]
pub const fn file_export_form_data() -> CommandText {
    CommandText::new(
        "Export form data…",
        "Write this document's filled form values out as FDF, XFDF or CSV.",
    )
}

/// `file.copy_page_text`
///
/// ★ **Was `edit_copy_page_text`, in the EDIT TAB section, until 2026-08-14.**
/// The command moved to File ▸ Export by operator decision — copying is not
/// authoring, and the File tab is the one tab every mode shows — so this
/// catalog entry moved with it, because this file is ordered by tab and a
/// command's text sitting under the wrong heading is how the next reader
/// concludes the command is somewhere it is not.
///
/// **The wording is unchanged, deliberately.** Nothing about what the command
/// does has moved, and a tooltip rewritten during a re-parenting is a change
/// nobody asked for arriving inside one they did. The chord it names is still
/// `Ctrl+Shift+C`, still bound in `crate::shell::manifest`'s keymap — now to
/// this id — and the sentence about inferred word and line breaks is still the
/// thing an operator cannot guess: a PDF is under no obligation to record where
/// a word ends, so pdfcer infers it from letter positions and says how much of
/// the copy was inferred.
#[must_use]
pub const fn file_copy_page_text() -> CommandText {
    CommandText::new(
        "Copy page text",
        "Copy this page's text to the clipboard (Ctrl+Shift+C). Where a PDF does not say where \
         words and lines end, pdfcer works it out from the position of the letters, and says \
         how much of the copy that was.",
    )
}

/// `file.copy_document_text`
///
/// Was `edit_copy_document_text`; see [`file_copy_page_text`] for the move.
/// Wording unchanged, including the warning about the window not responding,
/// which is the honest description of a synchronous extraction over every page
/// and is the kind of sentence a re-parenting must not quietly lose.
#[must_use]
pub const fn file_copy_document_text() -> CommandText {
    CommandText::new(
        "Copy document text",
        "Copy every page's text to the clipboard. On a long document this can take a few \
         seconds, during which the window will not respond.",
    )
}

/// `file.print`
#[must_use]
pub const fn file_print() -> CommandText {
    CommandText::new(
        "Print…",
        "Set up and print this document. Nothing prints until you press Print in the dialog.",
    )
}

/// `file.properties`
///
/// ## ★★★ The tooltip lost its first half on 2026-09-05, and that is the point
///
/// It read: *"The document's own title, author, subject and keywords, and the
/// properties of whatever is selected on the page."* One command, two subjects
/// — and the panel drew both, the document half permanently, at the foot of a
/// surface whose subject is the selection. The operator: *"the document
/// properties are still always visible in the properties tab. it needs to get
/// out of there and be in its own document properties tab."*
///
/// So the second subject is [`file_document_properties`], and this sentence now
/// describes exactly one panel. ★ Corrected rather than merely shortened: the
/// old wording is the sentence three modules quoted as their commission, and
/// leaving it here would have kept a control promising something a different
/// control does.
///
/// ★ It names the three kinds of thing that can be selected, because the panel
/// is empty until one of them is and an operator hovering an empty panel's
/// control deserves to know what would fill it.
#[must_use]
pub const fn file_properties() -> CommandText {
    CommandText::new(
        "Properties",
        "The properties of whatever is selected on the page — an object, a mark you have \
         placed, or a form field.",
    )
}

/// `file.document_properties`
///
/// ★★★ **The operator's own words for the surface**, 2026-09-05: *"it needs to
/// get out of there and be in its own **document properties** tab."* The label
/// is what names the dock tab — `PdfcerApp::new` builds every `PanelInfo` from
/// its command's label — so this string is the tab he asked for, spelled the
/// way he asked for it.
///
/// ★ *"Document properties"* rather than *"This document"* (the panel's own
/// heading) or *"Metadata"* (the format's word). A tab has to be recognisable
/// in a strip of five and legible out of context; a heading sits under a tab
/// that has already said which document. The two are deliberately different
/// strings, and `text::panels::docprops`' own test asserts they stay different.
///
/// The tooltip names the four fields, because they are the reason an operator
/// opens this, and then the facts, because they are what they get for free.
#[must_use]
pub const fn file_document_properties() -> CommandText {
    CommandText::new(
        "Document properties",
        "This document's own title, author, subject and keywords — stored in the file and \
         travelling with it — and the facts pdfcer read about it.",
    )
}

/// `file.fonts`
///
/// Moved here from View ▸ Panels. The Fonts panel answers "what is inside
/// this file", not "what is on my screen", so it belongs beside Properties
/// as document-level inspection.
#[must_use]
pub const fn file_fonts() -> CommandText {
    CommandText::new(
        "Fonts",
        "Show every font this document declares — type, encoding, embedded size, and whether \
         its embedded program could be removed.",
    )
}

/// `file.settings`
#[must_use]
pub const fn file_settings() -> CommandText {
    CommandText::new(
        "Settings…",
        "Choose how pdfcer reads and writes documents where the PDF standard leaves the answer \
         open — colour, printing separations, text extraction. Your choices are kept in a file \
         beside the program and survive restarts.",
    )
}

/// `file.shortcuts`
#[must_use]
pub const fn file_shortcuts() -> CommandText {
    CommandText::new("Keyboard shortcuts", "Show every keyboard shortcut.")
}

/// `file.about`
///
/// ★ **No ellipsis, deliberately.** This catalog's `…` means *you will be
/// asked something before anything happens* — the reading `view_reset_layout`
/// had its ellipsis taken away for getting wrong. About asks nothing; it
/// shows. Its neighbour `file_shortcuts` is the same kind of window and is
/// spelled the same way, and all three reference applications agree: Acrobat,
/// Inkscape and SolidWorks all write "About <product>" plain.
///
/// The tooltip names **all three** things the window carries rather than just
/// the version, because the version is the least of them. The reason this
/// command exists is the attribution surface — see [`crate::text::about`] —
/// and an operator looking for licence terms has to be able to tell from the
/// hover that this is where they live.
#[must_use]
pub const fn file_about() -> CommandText {
    CommandText::new(
        "About pdfcer",
        "Show this build's version, pdfcer's own licence, and the third-party material included \
         in the program.",
    )
}

/// `file.ocr`
///
/// ★ **The tooltip states the uncertainty, and that is not optional here.**
/// OCR is the single largest inference pdfcer makes — `pdfcer-core`'s own
/// `ocr::layer` header says *"every word here is a guess"* — and rule 4 asks
/// that an inherently uncertain inference say so rather than imply otherwise.
/// A hover is the first place an operator meets this command, and a tooltip
/// that described only the benefit would be the sentence they remember.
///
/// It also states what does **not** change, because that is the question a
/// scanned document raises: nothing visible is added and the image is never
/// re-encoded, so a scan that is the record of something stays exactly the
/// bytes it was. That is `ocr::layer`'s own guarantee rather than a claim this
/// catalog is making on its behalf.
///
/// The dialog's fuller disclosure lives in [`crate::text::ocr`]; this is the
/// one-line version, and the two must not drift apart in what they promise.
#[must_use]
pub const fn file_ocr() -> CommandText {
    CommandText::new(
        "Recognise text…",
        "Read the words in a scanned page and add them as invisible text behind the image, so \
         Find and copy work. Every word is a guess and this recogniser scores none of them, so \
         you are shown what it read before anything is saved. The page still looks the same and \
         the scan is never re-encoded.",
    )
}

// ===========================================================================
// PAGES TAB
//
// Every command here operates on THIS document's page set and respects the
// thumbnail rail's selection when there is one. That is the tab's
// organising rule and it is what distinguishes it from Tools, which
// produces new files. The tooltips say so where the distinction is easy to
// get wrong — `pages.merge_into` against `tools.merge_files` especially.
// ===========================================================================

/// `pages.insert_from_file`
#[must_use]
pub const fn pages_insert_from_file() -> CommandText {
    CommandText::new(
        "Insert from file…",
        "Insert the pages of another PDF into this document, before or after the page you have \
         selected.",
    )
}

/// `pages.delete`
///
/// **`Delete pages`, not `Delete`.** `RIBBON_IA.md` §5.3 writes the row as
/// `Delete`, which is unambiguous *in its band* — it sits under a tab
/// called Pages, in a group called Organise, beside Extract and Move.
/// It is not unambiguous against the contextual Format tab's `Delete`,
/// which removes the selected object and can appear over any tab at any
/// time. Two controls reading `Delete`, one of which removes a sheet from
/// a drawing set, is a collision worth two extra characters.
#[must_use]
pub const fn pages_delete() -> CommandText {
    CommandText::new(
        "Delete pages",
        "Remove the selected pages from this document. Undo reverses it.",
    )
}

/// `pages.extract`
#[must_use]
pub const fn pages_extract() -> CommandText {
    CommandText::new(
        "Extract…",
        "Write the selected pages out as a new PDF. This document is left unchanged.",
    )
}

/// `pages.move_up`
#[must_use]
pub const fn pages_move_up() -> CommandText {
    CommandText::new(
        "Move up",
        "Move the selected pages one place earlier in the document (Alt+Up).",
    )
}

/// `pages.move_down`
#[must_use]
pub const fn pages_move_down() -> CommandText {
    CommandText::new(
        "Move down",
        "Move the selected pages one place later in the document (Alt+Down).",
    )
}

/// `pages.split`
#[must_use]
pub const fn pages_split() -> CommandText {
    CommandText::new(
        "Split…",
        "Split this document into several files at page boundaries you choose.",
    )
}

/// `pages.merge_into`
#[must_use]
pub const fn pages_merge_into() -> CommandText {
    CommandText::new(
        "Merge into this document…",
        "Add the pages of one or more other PDFs to this document. To combine files into a new \
         one instead, leaving this document alone, use Tools > Merge files.",
    )
}

/// `pages.rotate_left`
#[must_use]
pub const fn pages_rotate_left() -> CommandText {
    CommandText::new(
        "Rotate left",
        "Turn the selected pages 90° counter-clockwise ([). This changes the document, not \
         just the view, and is saved with it — use Undo to reverse it.",
    )
}

/// `pages.rotate_right`
#[must_use]
pub const fn pages_rotate_right() -> CommandText {
    CommandText::new(
        "Rotate right",
        "Turn the selected pages 90° clockwise (]). This changes the document, not just the \
         view, and is saved with it — use Undo to reverse it.",
    )
}

/// `pages.resize`
///
/// ★★★ **The tooltip says what the command does NOT do**, and that is the
/// whole reason it is worded this way. Every other "page size" control an
/// operator has met — Word, LibreOffice, a print dialog's Fit-to-page —
/// reflows or scales, and this one changes the paper and leaves the drawing
/// exactly where it is. A tooltip reading *"change the page size"* would be
/// true, useless, and would confirm the wrong belief.
///
/// The window itself says it again, at greater length, with the overhang
/// measured — see `crate::text::page_size`. Saying it twice is deliberate: the
/// tooltip is what an operator reads *before deciding whether to open the
/// window at all*.
#[must_use]
pub const fn pages_resize() -> CommandText {
    CommandText::new(
        "Sheet size…",
        "Put the selected pages on a different size of paper. This changes the paper only — \
         nothing on the page moves and nothing is scaled to fit, so a smaller sheet crops the \
         drawing rather than shrinking it. The window shows what would fall off before you \
         commit.",
    )
}

// ===========================================================================
// EDIT TAB
// ===========================================================================

/// `edit.text`
#[must_use]
pub const fn edit_text() -> CommandText {
    CommandText::new(
        "Edit text",
        "Edit words already on this page — fix a typo, resize, or recolour existing text \
         (Ctrl+E). To add brand-new page text instead, use Add text.",
    )
}

/// `edit.add_text`
#[must_use]
pub const fn edit_add_text() -> CommandText {
    CommandText::new(
        "Add text",
        "Add new text to the page itself — a label, caption or note that becomes real, \
         permanent page content, exactly like the text already here (Ctrl+Shift+E). For a \
         removable comment instead, use Markup > Text box.",
    )
}

/// `edit.insert_image`
#[must_use]
pub const fn edit_insert_image() -> CommandText {
    CommandText::new("Image…", "Place an image file on this page.")
}

/// `edit.attachments`
///
/// ★★ **The tooltip says what the panel SHOWS as well as what it does**, which
/// is [`view_panel_bookmarks`]' shape and is the right one here for a reason of
/// its own: an operator has no way to discover that a PDF can carry whole files
/// inside it, because nothing on the page ever shows one. A tooltip reading
/// only *"Manage attachments"* would name a capability to somebody who does not
/// know the capability exists.
///
/// ★ It does **not** promise a description edit. `attach_file` takes a
/// description at attach time and `pdfcer-core` has no verb that changes one
/// afterwards, and `view_reset_layout`'s recorded defect is exactly this — a
/// tooltip that promised a choice the build did not offer. The panel discloses
/// the limit where an operator meets it.
#[must_use]
pub const fn edit_attachments() -> CommandText {
    CommandText::new(
        "Attachments",
        "The files this document carries inside itself — attach one, save one out, or remove one. \
         They appear on no page.",
    )
}

/// **Text field** — the box an operator types into.
#[must_use]
pub const fn edit_form_text_field() -> CommandText {
    CommandText::new(
        "Text field",
        "A box to type into. Click where you want it, or drag out the exact size.",
    )
}

/// **Check box** — one independent on/off box.
#[must_use]
pub const fn edit_form_check_box() -> CommandText {
    CommandText::new(
        "Check box",
        "A single box that is either ticked or not. Click where you want it, or drag out the exact size.",
    )
}

/// **Radio button** — one of a group.
///
/// ★ The tooltip names the grouping rule, because it is the only one of the
/// five whose behaviour depends on another field: two radios sharing a name are
/// one control. An operator who does not know that places two buttons that both
/// stay on and reasonably calls it a bug.
#[must_use]
pub const fn edit_form_radio_button() -> CommandText {
    CommandText::new(
        "Radio button",
        "One of a set, where choosing one clears the others. Give them the same group name to make them alternatives.",
    )
}

/// **Choice** — a drop-down or list.
#[must_use]
pub const fn edit_form_choice() -> CommandText {
    CommandText::new(
        "Drop-down",
        "A list of options to choose from. Click where you want it, or drag out the exact size.",
    )
}

/// ★★★ **Select everything on this page, including what has slid off it.**
///
/// The tooltip names the RECOVERY rather than the mechanism, because that is
/// what sends an operator looking for it. 2026-09-01: *"I sometimes drop
/// objects there, and when I do I can't get them back."*
///
/// ★ "Select all" is the label because it is the phrase every hand already
/// knows and a ribbon group competes for width. The off-the-sheet half — the
/// thing that makes this a rescue rather than a convenience — is in the
/// tooltip, where there is room to say it properly.
#[must_use]
pub const fn edit_select_all() -> CommandText {
    CommandText::new(
        "Select all",
        "Selects everything drawn on this page, including anything moved off the edge of the sheet and out of reach of the mouse.",
    )
}

/// **Push button** — authorable, inert, and greyed until pdfcer can run actions.
#[must_use]
pub const fn edit_form_push_button() -> CommandText {
    CommandText::new("Button", "A button that runs an action when pressed.")
}

/// Why the push button is greyed.
///
/// ★★★ R9 permits greying only for a **temporarily** unavailable capability
/// that is **always explained on hover**, and this is the explanation. It draws
/// the distinction that matters: pdfcer can *place* a button perfectly well —
/// what it cannot do is *run* what the button would do, because it executes no
/// PDF actions. Placing one would give the operator a control that looks
/// finished and does nothing, which is worse than not offering it.
///
/// ★ It says what is missing rather than apologising, so an operator can judge
/// whether it matters to them and can ask for it if it does.
#[must_use]
pub const fn edit_form_push_button_unavailable() -> &'static str {
    "pdfcer can place a button but cannot yet run what a button does, so one placed now would do nothing when pressed."
}

pub const fn edit_form_create_field() -> CommandText {
    CommandText::new(
        "Create field",
        "Add a new form field to the page. Click where you want it, or drag out the exact size.",
    )
}

/// `edit.form_manage_fields`
#[must_use]
/// `edit.form_manage_fields`
///
/// ★★ **"retype" was struck on 2026-08-28**, because it was a promise nothing
/// can keep. Acrobat has offered no field-type conversion since Acrobat 6, and
/// `pdfcer-core` models the same limit by making the request **unrepresentable**
/// rather than by accepting it and returning an error — so there is not even a
/// control to grey. A tooltip is a contract, and this clause had been offering
/// an operator something no route in either crate provides.
///
/// ★ The label keeps *"Manage fields"* and the command now opens the **Forms
/// panel**, which is where listing, renaming and removing already live.
pub const fn edit_form_manage_fields() -> CommandText {
    CommandText::new(
        "Manage fields",
        "Open the Forms panel to list every field in this document, fill it, rename it or \
         remove it.",
    )
}

/// `edit.form_flatten`
#[must_use]
pub const fn edit_form_flatten() -> CommandText {
    CommandText::new(
        "Flatten",
        "Turn the filled values into ordinary page content, so they draw everywhere but can no \
         longer be edited as fields.",
    )
}

/// `edit.find`
///
/// ★ **The one command in this catalog whose only control is on the status
/// bar.** `RIBBON_IA.md` §6 puts the Find toggle there rather than on the
/// ribbon, so this label and tooltip are what that toggle's *command* says —
/// reachable from a keymap, from a customized quick-access toolbar, and from
/// the shortcut list — while `crate::text::find` holds the copy the bar's own
/// controls own. The two are not duplicates: this one is keyed by command id
/// and consumed by `crate::shell::commands`, that one is keyed by control and
/// consumed by a widget.
///
/// The tooltip names `Ctrl+F` because the chord genuinely works: the manifest
/// keymap binds it AND `crate::app::keyboard::parse_chord` can spell it. Both
/// halves are required — `Ctrl+O` was in the keymap and printed in a tooltip
/// for the whole of the ribbon's first life while pressing it did nothing,
/// because the spelling table held only digits.
///
/// It also names the two limits an operator has no way to guess and that
/// account for almost every surprising empty result: the search is over the
/// text **drawn on the pages**, and it matches within one text run at a time.
#[must_use]
pub const fn edit_find() -> CommandText {
    CommandText::new(
        "Find",
        "Search the text drawn on this document's pages, and highlight every hit (Ctrl+F). Form fields, comments, bookmarks and attachments are not searched, and a word the producer split across two text runs is not found.",
    )
}

/// `edit.redact`
///
/// ★★★ **CORRECTED 2026-09-05.** The closing clause read *"Marking is
/// reversible; applying is not."* That was true of every route this shell had
/// until `Pass 250.2` (engine `pdfcer-core` v0.38.0 at `b01964f`), which made
/// the **default** destination arm the next save instead of rewriting at the
/// click: the undo log survives, the page does not change, and a Cancel
/// disarms it. `crate::text::redact::panel_intro` was rewritten for that the
/// same day; these two tooltips were not, because the work opened the panel's
/// catalog and not this one.
///
/// What is unchanged, and stays exactly as emphatic, is that **once a file
/// with the marks applied has been written, nothing brings the content
/// back.** What moved is *when*, never *whether* — which is the same
/// distinction `panel_intro`'s own note draws, and it is deliberately worded
/// to match so an operator comparing the two is not comparing two accounts.
#[must_use]
pub const fn edit_redact() -> CommandText {
    CommandText::new(
        "Redact",
        "Mark what is to be permanently removed — a whole page, every occurrence of some text, \
         or everything matching a pattern. Marking is reversible, and so is applying until the \
         file is written; once it is written, that cannot be undone.",
    )
}

/// `edit.redact_apply`
///
/// ★★★ **CORRECTED 2026-09-05** — see [`edit_redact`] above for the whole
/// account. The sentence read *"This cannot be undone."* On the default
/// destination the click now stages the removal into the next save
/// (`crate::redact::stage_into_session`), which is undoable and cancellable;
/// the irreversible moment is the write, and both ordinary save routes refuse
/// by name while a redaction is armed rather than writing a half-redacted
/// file.
#[must_use]
pub const fn edit_redact_apply() -> CommandText {
    CommandText::new(
        "Apply redactions",
        "Permanently remove everything the redaction marks cover. By default this happens when \
         you next save, and can be cancelled until then; once the file is written it cannot be \
         undone.",
    )
}

/// `edit.undo`
///
/// # ★ Why this does NOT name the operation, and what it would take to
///
/// `SALVAGE.md` records the old shell as having *"undo tooltips naming the
/// specific operation"*, and the engine still supplies everything needed for
/// one: `EditSession::undo_kind` answers *what would be undone* without
/// undoing it, over 44 `CommandKind` variants. Writing *"Undo add annotation
/// (Ctrl+Z)"* is therefore catalog work and nothing more — a
/// `CommandKind → &'static str` mapping in this file, with a fallback for the
/// kinds this shell cannot author.
///
/// **The blocker is the registry, not the catalog.** `egui_shell`'s
/// `Command::tooltip` is a `String` fixed at registration;
/// `CommandRegistry` exposes `get`, `iter` and `register` and **no mutable
/// accessor and no removal**, and `PdfcerApp::commands` is built once in
/// `PdfcerApp::new` and handed to the ribbon by shared reference every frame.
/// So a tooltip that changes with the log needs one of two things:
///
/// 1. a `get_mut` (or a `tooltip` closure) on `CommandRegistry` — which is a
///    change to `crates/egui-shell`, the crate `check-shell-purity.sh` keeps
///    application-agnostic and which this work is not permitted to touch; or
/// 2. rebuilding the whole 101-command registry every frame so one string can
///    differ — which pays a hundred allocations a frame for one tooltip, and
///    changes the **accessible name** of an icon-only control under the
///    operator's pointer, since `egui_shell::ribbon::a11y` promotes the
///    tooltip to the name when there is no visible label.
///
/// Half-doing it — naming the operation in the status bar instead, say — would
/// put the answer somewhere the operator is not looking when they hover the
/// control that asks the question. So the plain label ships, deliberately, and
/// the operation *is* named on the diagnostic channel (`undo kind=…`), which is
/// where it is currently readable. The right fix is (1), by whoever next has
/// cause to open `egui-shell`'s command registry.
///
/// The chord is still printed, and that is the part P3 actually requires: this
/// command is greyed whenever the log is empty, and a greyed control must
/// explain itself on hover. `egui_shell::ribbon::qat` uses
/// `on_disabled_hover_text`, so this sentence is read in exactly the state it
/// most needs to be.
#[must_use]
pub const fn edit_undo() -> CommandText {
    CommandText::new("Undo", "Undo the last change (Ctrl+Z).")
}

/// `edit.redo`
#[must_use]
pub const fn edit_redo() -> CommandText {
    CommandText::new(
        "Redo",
        "Redo the change you just undid (Ctrl+Y or Ctrl+Shift+Z).",
    )
}

// ===========================================================================
// MARKUP AND MEASURE — in `annotate`
//
// ★ **Moved out on 2026-08-14 under R2**, at the seam that module's header
// argues for: these two tabs are what an operator *adds on top of* the page,
// which is the line `app::modes::Capabilities` already draws between
// `edit_content` and the two authoring flags, and the line `shell::manifest`
// already draws by keeping `markup.rs` and `measure.rs` as files of their own.
//
// Re-exported by name — not by glob — so every call site still writes
// `t::markup_rectangle()` and nothing outside `text/` learns the catalog was
// split, while a function added over there still has to be named here to reach
// the crate. The catalog's discipline is that every operator-visible string is
// named somewhere a reviewer looks.
// ===========================================================================
pub mod annotate;

pub use annotate::{
    markup_arrow, markup_cloud, markup_comments, markup_ellipse, markup_finish, markup_highlight,
    markup_ink, markup_polygon, markup_polyline, markup_rectangle, markup_squiggly, markup_stamp,
    markup_sticky_note, markup_strikeout, markup_text_box, markup_underline, measure_finish,
    measure_length, measure_linear, measure_manage_groups, measure_perimeter,
    measure_radius_diameter, measure_set_scale, measure_two_line,
};

// ===========================================================================
// TOOLS TAB
// ===========================================================================

/// `tools.merge_files`
#[must_use]
pub const fn tools_merge_files() -> CommandText {
    CommandText::new(
        "Merge files…",
        "Combine several PDFs into one new file. This document is not changed — to add pages \
         to it instead, use Pages > Merge into this document.",
    )
}

/// `tools.split_files`
#[must_use]
pub const fn tools_split_files() -> CommandText {
    CommandText::new(
        "Split files…",
        "Split one or more PDFs into separate files. The originals are not changed.",
    )
}

/// `tools.font_folders`
#[must_use]
pub const fn tools_font_folders() -> CommandText {
    CommandText::new(
        "Font folders…",
        "Point pdfcer at folders of your own font files (.ttf/.otf) so it can draw a document's \
         missing text with the real typeface instead of a bundled substitute. This changes how \
         missing fonts look, not where text sits on the page.",
    )
}

/// `tools.embed_fonts`
#[must_use]
pub const fn tools_embed_fonts() -> CommandText {
    CommandText::new(
        "Embed fonts",
        "Copy the font programs this document relies on into the file itself, so it draws the \
         same on a machine that does not have them.",
    )
}

/// `tools.unembed_fonts`
#[must_use]
pub const fn tools_unembed_fonts() -> CommandText {
    CommandText::new(
        "Unembed fonts",
        "Remove embedded font programs from the file. The document gets smaller and starts \
         depending on the reader having those fonts.",
    )
}

/// `tools.render_diagnostics`
#[must_use]
pub const fn tools_render_diagnostics() -> CommandText {
    CommandText::new(
        "Render diagnostics",
        "Show what the renderer did with the last page — how long it took, at what raster \
         size, and anything it could not draw.",
    )
}

// ===========================================================================
// FORMAT TAB (contextual)
// ===========================================================================

/// `format.delete`
#[must_use]
pub const fn format_delete() -> CommandText {
    CommandText::new(
        "Delete",
        "Remove what is selected from the page. Undo reverses it.",
    )
}

/// `format.select_form`
///
/// ★★★ **The deliberate second act that pays for the deep hit test.**
///
/// Since 2026-08-27 a click reaches *inside* a form XObject and selects what is
/// drawn there, and the form itself is excluded from the hit test outright — a
/// `/BBox` is a clipping extent (§8.10.1), not a claim about ink, so a
/// page-sized form was winning every click at every point. That was the
/// operator's report: *"when I click on one of the objects all I get is the
/// page selected."*
///
/// But a form is a perfectly good thing to want. It is one page object with an
/// ordinary paint-order index, and moving a title block or deleting a stamp is
/// *the form*, not the two hundred objects inside it. So the reach gained
/// inside forms must not cost the reach to the form, and this command is how
/// that is paid: reachable on purpose, never by default.
///
/// # Every clause of the tooltip, and what it answers
///
/// **"the form that contains it"** names the structure, which is the fact the
/// operator can act on — it explains the page-sized outline they used to get,
/// and it is the word they need if they go looking in another tool.
///
/// **"one object you can move, delete or copy"** is the reason to press it.
/// The thing selected before pressing is none of those, and this sentence is
/// the only place that trade is stated in the operator's own vocabulary.
///
/// **"Everything drawn inside it moves with it"** is the consequence they must
/// know *before* pressing, not after. It is also the honest warning that a form
/// may be shared: `pdfcer-core`'s decision 076 rules editing inside a shared
/// form as edit-in-place, and a page invoking a form twice draws it twice.
///
/// # Why "the form" and not "the container" or "the group"
///
/// Because *form XObject* is what the file calls it, what `pdfcer
/// object-list` prints, and what the Objects panel row says. A friendlier word
/// invented here would be a fourth vocabulary for one thing, and the operator
/// reads all four.
#[must_use]
pub const fn format_select_form() -> CommandText {
    CommandText::new(
        "Select the form",
        "Select the form that contains what you have selected, so you have one object you \
         can move, delete or copy. Everything drawn inside it moves with it.",
    )
}

/// `format.unshare_form`
///
/// ★★★ **The "option" half of `pdfcer-core`'s decision 076, and the remedy the
/// SHARED CONTENT disclosure needs somewhere to point at.**
///
/// Since `Pass 119.0` this shell can edit text **inside a form XObject**. ISO
/// 32000-1 §8.10.1 names a CAD system's standard component as the *purpose* of
/// that construct, and no clause in either edition binds a form to a page — a
/// confirmed permanent negative in pdfcer's spec corpus (`FX-N1`). So one title
/// block is one stream object invoked from thirty-six sheets, and an operator
/// fixing a typo on sheet 12 changes all thirty-six.
///
/// pdfcer cannot prevent that structurally: there is exactly one stream to
/// write. Decision 076 ruled edit-in-place-and-disclose the **default**, and
/// `R206` requires that two defensible behaviours ship as two options with a
/// chosen default. **This is the second option**, and for a week it did not
/// exist — the engine's own note calls that *"the state R206 exists to
/// prevent"*.
///
/// # Every clause of the label and the tooltip, and what it answers
///
/// **"Give this page its own copy"** is the label, and it is a *sentence in the
/// imperative*, which no other command on this tab is. That is deliberate. The
/// alternatives were all worse in the same way: **"Unshare"** is the engine's
/// word and means nothing to a drafter; **"Detach"** implies the drawing is
/// removed from the page; **"Make unique"** is Inkscape's phrasing for a
/// different act and invites the reading *"make it look different"*. What the
/// operator wants is stated exactly by what happens: this page gets a copy.
///
/// **★★★ "if it is also drawn on other pages"** is a CONDITION, and until
/// 2026-08-29 it was an assertion — *"This drawing is drawn on other pages
/// too."* — stated unconditionally by a tooltip that had measured nothing.
///
/// That is the defect this clause was rewritten for, and it is worth being
/// precise about why it was one. A tooltip is not decoration; it is a claim
/// about the operator's own file, and this one was made before anything in the
/// command's chain had asked how many times the form was invoked. On an
/// ordinary one-page CAD sheet wrapped in a single form — the shape of this
/// operator's own SolidWorks exports — it was simply **false**. And because it
/// was identical either way, the operator who genuinely *did* have a
/// thirty-six-sheet title block learned nothing from it: an unconditional
/// sentence carries no information about the case it is unconditional over.
///
/// ⇒ **A control cannot assert a fact about a document it has not measured.**
/// The tooltip's job is to say what the command *does* and under what condition
/// it helps; the measurement is a whole-document page walk, it belongs on the
/// press, and what it found is disclosed afterwards. See
/// `crate::app::actions::xobject::fanout` for the walk and R9 for why it is not
/// in a per-frame condition.
///
/// **"changes here will not affect them"** is the reason to press it — the
/// promise, in the operator's terms, and the only clause that distinguishes
/// this command from `format.select_form` one row above.
///
/// **★★ "pdfcer checks when you press, and says what it found"** is the clause
/// that replaces the deleted assertion, and it does two jobs. It tells the
/// operator where the answer to *"is it actually shared?"* comes from, which
/// nothing else on screen states — the Objects panel does not say a form is
/// shared, the canvas cannot, and the SHARED CONTENT disclosure says it only
/// *after* an edit has already fanned out. And it sets the expectation that
/// pressing this may **decline**: on a drawing nothing else draws,
/// `crate::text::unshare::UnshareRefusal::NotShared` says so and changes
/// nothing, which is a service rather than a failure and should not arrive as a
/// surprise.
///
/// **"Everything looks exactly the same afterwards"** is the clause an operator
/// would otherwise report as a bug. The copy is byte-identical to the original
/// until it is edited, so a successful unshare renders pixel-for-pixel as
/// before. A tooltip that promised a visible result would make every success
/// look like a failure.
///
/// # ★★ Why the tooltip does not say "before you edit it"
///
/// It is the true instruction — unsharing *after* an edit copies the
/// already-edited stream and leaves every other page changed as well — but a
/// tooltip is read while deciding whether to press, and a sequencing
/// instruction there competes with the four clauses above for the one line an
/// operator actually reads. The sequence belongs where it is acted on, which is
/// the disclosure that fires the moment an edit *has* fanned out:
/// `crate::text::unshare::shared_content_remedy` states it in order, and its
/// doc comment carries the argument.
#[must_use]
pub const fn format_unshare_form() -> CommandText {
    CommandText::new(
        "Give this page its own copy",
        "If this drawing is also drawn on other pages, this gives the page its own copy so \
         changes here will not affect them. pdfcer checks when you press, and says what it found. \
         Everything looks exactly the same afterwards.",
    )
}

/// `format.properties`
///
/// ★ **A second route to `file.properties`, not a second implementation of
/// it.** Its dispatch arm raises `Action::Command("file.properties")`, which is
/// the mechanism that exists so exactly this cannot become two ways of opening
/// one panel with two sets of guards — the Find bar's OCR offer is the
/// precedent.
///
/// It is registered as its own id rather than listing `file.properties` twice
/// because the shell enforces **one command, one tab**, and the two placements
/// answer different questions: File ▸ Document is *"tell me about this file"*
/// and Format is *"tell me about the thing I just clicked"*.
///
/// The tooltip names the ce dimension case explicitly. That is the capability
/// the panel gained on 2026-08-18 — the style cascade, the tolerance and the
/// radius/diameter switch — and it is the one an operator has no other way to
/// discover, because a selected ce dimension looks exactly like an unselected
/// one apart from its outline.
#[must_use]
pub const fn format_properties() -> CommandText {
    CommandText::new(
        "Properties",
        "Show the Properties panel for what is selected — for a dimension, its \
         group, what it measured, and every setting it inherits from its group \
         or overrides for itself.",
    )
}

// ---------------------------------------------------------------------------
// The Font group — `RIBBON_IA.md` §5.8's "Text run" row, built 2026-08-27
//
// ★★★ **Every tooltip below has to read correctly in TWO states**, and that
// is the constraint that shaped all five of them.
//
// `egui_shell::ribbon::control::render_command` shows a command's tooltip with
// `on_hover_text` when the control is enabled and `on_disabled_hover_text`
// when it is not — the **same string**. These five are enabled only while a
// text range is swept (`selection.text`), which is *not* the state an operator
// is in when they go looking for them: they have clicked a piece of text with
// the Select tool, the Format tab has appeared, and the Font controls are
// greyed.
//
// So each tooltip says what the control does **and how to give it something to
// act on**. That second clause is not padding — it is the answer to O37's own
// admission that *"you must press T first and nothing on screen says so"*, and
// a greyed control an operator can hover is the one surface in this
// application that can say it at the moment the question is asked.
//
// ★ It is a **statement**, not a tip. `crate::text::tool`'s rule 2 —
// *"every sentence states a fact about the program, never a tip"* — is why
// these read "Sweeping text with the Text tool chooses what this applies to"
// rather than "Try sweeping some text!".
// ---------------------------------------------------------------------------

/// `format.font`
///
/// ★ Drawn by an `Item::Custom`, not by this command's button, because a face
/// chooser has to ask *which* of the page's fonts and a button cannot. The
/// label and tooltip are still registered here and still used: the shell reads
/// them for the a11y name and for `shell::commands::reach`'s reachability
/// check, and the custom renderer draws the label beside its combo.
#[must_use]
pub const fn format_font() -> CommandText {
    CommandText::new(
        "Font",
        "Set the selected text in another of the fonts this page already carries. Sweeping \
         text with the Text tool (T) chooses what it applies to.",
    )
}

/// `format.font_size`
#[must_use]
pub const fn format_font_size() -> CommandText {
    CommandText::new(
        "Size",
        "Set the size of the selected text, in points. Sweeping text with the Text tool (T) \
         chooses what it applies to.",
    )
}

/// `format.bold`
///
/// ★★ The tooltip names the **fallback**, exactly as the Properties panel's
/// does, because the fallback is what an operator would otherwise meet as a
/// surprise: a page carrying a real bold cut gets the real face, and one that
/// does not gets thickened letters and a sentence in the status bar saying so.
/// Rule 4 — the thickened text renders exactly as the saved file will render
/// it, and the disclosure is off-canvas.
#[must_use]
pub const fn format_bold() -> CommandText {
    CommandText::new(
        "Bold",
        "Set the selected text in bold — the page's real bold face where it has one, and \
         thickened letters with a note in the status bar where it does not. Sweeping \
         text with the Text tool (T) chooses what it applies to.",
    )
}

/// `format.italic`
#[must_use]
pub const fn format_italic() -> CommandText {
    CommandText::new(
        "Italic",
        "Slant the selected text — the page's real italic face where it has one, and \
         slanted letters with a note in the status bar where it does not. Sweeping text \
         with the Text tool (T) chooses what it applies to.",
    )
}

/// `format.font_colour`
///
/// ★ `format.colour` was already taken, by the markup property editor in the
/// same tab's future, and the two are genuinely different subjects — one is an
/// annotation's ink, the other is a page-content fill. Word calls this one
/// *Font Color*, which settles the name in the operator's own vocabulary
/// rather than by disambiguation.
///
/// ★★ The tooltip states the **refusal** as well as the act, because the
/// refusal is common on exactly the documents this program is for: a run
/// painted in DeviceCMYK or a spot colour has no faithful sRGB, so the swatch
/// is replaced by a sentence rather than showing a nearest-match that the next
/// press would write back — converting a drawing's ink on its way to a printer
/// that cares.
#[must_use]
pub const fn format_font_colour() -> CommandText {
    CommandText::new(
        "Colour",
        "Set the colour of the selected text. Text painted in CMYK or a spot colour is left \
         alone, so a drawing's ink is not converted to screen colour behind your back. \
         Sweeping text with the Text tool (T) chooses what it applies to.",
    )
}

// ===========================================================================
// MODES
//
// Not ribbon commands: these are the three positions of the selector at the
// far right of the tab row, reachable from the keymap. They are registered
// commands because a key binding resolves against the registry, and because
// the mode selector is a control like any other.
//
// Each tooltip states the rule that makes the feature safe — a mode changes
// what is VISIBLE and never makes a visible control silently inert. That
// distinction is the whole difference between this and the `editing_enabled`
// master toggle it replaces.
// ===========================================================================

/// `mode.read`
#[must_use]
pub const fn mode_read() -> CommandText {
    CommandText::new(
        "Read",
        "Show only what a reader needs: File and View (Ctrl+1). Nothing is hidden from the \
         document — only from the interface — and your edits are untouched.",
    )
}

/// `mode.review`
#[must_use]
pub const fn mode_review() -> CommandText {
    CommandText::new(
        "Review",
        "Add the Pages, Markup and Measure tabs (Ctrl+2) — comment on a drawing, measure it, \
         and reorganise the sheets, without the content-editing tools.",
    )
}

/// `mode.edit`
#[must_use]
pub const fn mode_edit() -> CommandText {
    CommandText::new("Edit", "Show every tab (Ctrl+3).")
}

/// The properties every command's copy must hold — one label per command,
/// no two labels alike, every tooltip a sentence. Split out under R2; see
/// that module's header.
#[cfg(test)]
mod tests;
