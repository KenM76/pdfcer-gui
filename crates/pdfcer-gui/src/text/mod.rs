//! # text — the operator-visible string catalog
//!
//! **Every string a human can read in this application is defined here and
//! nowhere else.** That is a standing convention carried across from the
//! old crate (`ui_text.rs`, 7,912 lines and 1,193 entries), and it is
//! enforced mechanically rather than by review: a CI gate scans the module
//! tree for string literals outside the catalog and fails the build.
//!
//! ## Why a catalog rather than literals at the call site
//!
//! Three reasons, in order of weight:
//!
//! 1. **Copy is a design surface with its own quality bar.** pdfcer's error
//!    prose distinguishes "your file is damaged" from "pdfcer is not
//!    finished yet" from "this page would not draw", and it does that
//!    consistently because all three sentences are visible in one file
//!    next to each other. Scattered literals drift into three different
//!    voices within a month.
//! 2. **Translation, when it comes, is a mechanical job or an impossible
//!    one.** Which it is was decided the day the first literal was written.
//! 3. **It makes "no placeholders" checkable.** A label that says `TODO`
//!    or `Panel` is visible in the catalog in a way it never is inline.
//!
//! ## Why this is a directory, not a file
//!
//! The old catalog broke the project's 1,500-line ceiling by a factor of
//! five. It is split by AREA here from the first commit — `mod.rs` holds
//! shell-wide strings, and each future surface (ribbon, panels, dialogs,
//! tools) gets a sibling module — so the split never has to be done as a
//! migration. At S0 there is exactly one area, which is why `mod.rs` is
//! currently the whole catalog.
//!
//! ## Conventions
//!
//! - **Sentence case, no trailing period on labels; full sentences with
//!   punctuation for prose.** A label is a name; a message is a statement.
//! - **Name the thing that went wrong and what the operator can do.**
//!   "Failed to open" is not a message; it is a shrug with a capital F.
//! - **Never apologise, never blame the file without evidence.** The three
//!   open-failure functions below exist precisely so the shell does not
//!   have to guess which of "your document is broken" and "pdfcer cannot do
//!   this yet" is true — `pdfcer-core` returns structured errors that say.
//!
//! ## Areas
//!
//! | Module | Surface |
//! |---|---|
//! | `mod.rs` (this file) | shell-wide strings: window title, canvas states, the three open failures |
//! | [`about`] | the About dialog — the product line, pdfcer's own licence, and the **attribution catalog** for every third-party work the binary redistributes |
//! | [`ribbon`] | the ribbon's *structural* strings — tab labels, the one-line question each tab answers, group captions, mode labels |
//! | [`commands`] | the label and tooltip of every ribbon command |
//! | [`files`] | the open/close/recent surface: the file dialog's title and filters, and everything the Recent control draws |
//! | [`find`] | the Find bar — the field, the step buttons, the position readout, the four search options, and the status bar's Find toggle |
//! | [`markup`] | the Markup ▸ Style group's three tooltips and its unit — the only place a swatch and a number can say what they are |
//! | [`menus`] | the copy a **context menu** owns rather than borrows. Empty by construction — a menu row's words are its command's — and its header is the argument for why |
//! | [`ocr`] | the Recognise-text dialog and the Find bar's offer. The catalog with the hardest job in the crate: it has to disclose that **every word OCR produces is a guess and this engine scores none of them**, without ever implying a mark on the page |
//! | [`pages`] | the Pages panel — the page counts, the tile tooltip, the **four sentences an undrawn thumbnail can say**, and the preview control that stops the grid |
//! | [`panels`] | every string the dock's panel bodies show — Bookmarks, Layers, Signatures, Fonts, Objects, Properties |
//! | [`print`] | the print dialog — three tabs, the preview, the device refusals, and the commit button whose label carries the clip count |
//! | [`redact`] | marking, review, and the apply report. **The strictest wording rules in the catalog** — the one surface where a comfortable sentence is a security defect, and the only one entitled to the word *verified* |
//! | [`rotating`] | the ninth handle — four refusals and two disclosures, including the one the engine commissioned by name about a dimension's axis lock |
//! | [`status`] | the status bar — the render-notes disclosure, the fit/zoom mirrors, and the editable page box |
//!
//! The split between `ribbon` and `commands` follows the seam in the data
//! itself: `crate::shell::manifest` consumes [`ribbon`] and
//! `crate::shell::commands` consumes [`commands`], so a change to one file
//! has one reviewer and one consumer. [`panels`] follows the same rule one
//! surface over — `crate::panels` is its sole consumer — and is itself a
//! directory, because six panel bodies' worth of copy is more than one file
//! should hold and the 1,500-line ceiling is not raised for catalogs.

pub mod about;
/// ★★★ **Every word `OPERATOR_REQUESTS.md` O122 puts on screen** — the
/// *Open in Acrobat* control beside the mode selector, the three things it can
/// say before it acts, and the Settings field that says where Acrobat is. One
/// module for four surfaces because they are one conversation; see its header.
/// Consumed by `crate::shell::commands`, `crate::dialogs::open_in_acrobat` and
/// `crate::dialogs::settings::acrobat`.
pub mod acrobat;
/// Every word the About dialog shows, plus the structured attribution catalog
/// naming the third-party material this binary redistributes. Consumed by
/// `crate::dialogs::about`.
/// ★★★ **Reading a comment where the comment is** — every word the canvas
/// note pop-up and its hover tooltip show. Consumed by
/// `crate::canvas::notepopup`, which is the only route to a note's `/Contents`
/// that works in Read mode. Its header carries the two capabilities that are
/// deliberately WORDLESS here, under R9, because the engine cannot reach them.
pub mod annotpopup;
/// The attachment clipboard's words, including the one question a paste must
/// ask before the press: the engine REPLACES a same-named attachment.
pub mod attachclip;
/// ★★★ **What a push button DOES** — every word the placement dialog's action
/// chooser says, including the submit disclosure. Its own module because two of
/// the seven choices write an address into the document that some other program
/// may act on, and the operator cannot see that by looking at the page.
pub mod buttonaction;
/// The label and tooltip of every ribbon command. Consumed by
/// `crate::shell::commands`.
/// The four sentences the object clipboard can say when it cannot act.
pub mod clipboard;
pub mod commands;
/// The words form-data export says — `file.export_form_data`, wired
/// 2026-08-27. Its one load-bearing sentence is the CSV neutralisation
/// disclosure; see the module header.
/// What the Embed-fonts window says before it changes anything — a report
/// rather than a form, because the operator's only decision is yes or no. See
/// its header for the three things it must say and in what order.
/// What the Save-a-compacted-copy window says before it throws anything away —
/// a revision history, possibly every signature, and the original file's role
/// as the canonical one.
pub mod compact;
/// The three sentences a held Shift puts on the status row while it is
/// constraining a drag. Consumed by [`crate::canvas::constrain::caption`].
pub mod constrain;
/// Every word the Render-diagnostics dialog adds around the findings — the
/// title, the three measurements of the render itself, and the two states in
/// which there is nothing to report. The findings themselves stay in
/// [`status`]. Consumed by `crate::dialogs::diagnostics`.
pub mod diagnostics;
/// Every word the Manage-dimension-groups window shows.
///
/// A sibling of [`scale`] and it inherits that module's rule 15 discipline: the
/// bare word "dimension" never appears, because a **ce dimension** (one pdfcer
/// authors) and a **pdf dimension** (CAD-exported page content pdfcer must not
/// alter) are opposites and the ambiguity has already cost one investigation.
///
/// Its own hardest job is different from `scale`'s: explaining that a group
/// edit reaches **backwards**, over dimensions already placed on pages that are
/// not on screen — and doing it with a count that is computed before the edit
/// rather than reported after it.
pub mod dimension_groups;
/// The three sentences a dragged-and-dropped file can answer with.
/// **The document tab strip, and the page drag between documents.** What a tab
/// says, and what a drag says it is about to do.
pub mod doctabs;
pub mod dropped;
pub mod embed;
/// What the measure tools say about what they INFERRED — the two-line
/// gesture's refusals, the angle an override overrode, and an apex that is
/// only real if the lines are extended.
///
/// Its own header carries the reason it exists: `pdfcer-core` gives that
/// gesture three facts a shell is expected to surface, and this build
/// surfaced none of them until 2026-08-19.
/// Every word the Insert-image window shows, and the disclosures a placement
/// owes afterwards.
///
/// Its hardest job is stated in its own header: a resolution is a property of
/// the **placement**, not of the file, and every mistake it can report — a
/// 2000 dpi photo in a 2-inch box, a 12 dpi logo across a page — looks perfect
/// on screen at editing zoom and only shows up on the plot.
/// Every word the Export-DXF window shows.
///
/// Its header carries the sentence the whole feature turns on, quoted from
/// `pdfcer-core`: every generic PDF-to-DXF converter exports at paper scale and
/// says nothing, so a 1:2 detail arrives at half size **looking plausible**.
pub mod export_dxf;
pub mod export_form;
/// ★★★ Every word the Export-image window shows, and every sentence an image
/// export owes afterwards. `OPERATOR_REQUESTS.md` O120.
///
/// Its header carries the operator's own parenthesis — *"(including
/// transparency where supported!)"* — and why that parenthesis is an
/// instruction rather than an aside: one of the three formats cannot do it, and
/// what is being asked for is that pdfcer be the thing that says which.
pub mod export_image;
/// ★★★ Every word the Export-text window shows, and every sentence a text
/// export owes afterwards.
///
/// Its header carries the sentence the whole feature is arranged around — **a
/// scanned drawing has no text layer, so exporting it writes an empty file, and
/// an empty file looks exactly like a successful export** — and the reason the
/// losses are said twice, in the window and in the receipt, in two different
/// registers.
pub mod export_text;
/// ★ The FORM-FIELD clipboard's sentences — five refusals and the paste's
/// off-canvas loss note. Separate from [`clipboard`] because the loss note is
/// not a refusal: the paste worked, and the sentence exists because part of the
/// field could not travel and the operator cannot see which part.
pub mod fieldclip;
/// The copy the open/close/recent surface owns — the file dialog's title and
/// filter names, and every string the Recent control draws. Consumed by
/// `crate::app::files` and `crate::app::recent`.
pub mod files;
/// Every string the Find bar shows, plus the status bar's Find toggle.
/// Consumed by `crate::find::bar` and `crate::app::status`.
pub mod find;
/// What the font-donor scan says when it skips a file — five sentences, all
/// about something that did not happen. See its header for why a skip is worth
/// a sentence.
pub mod fonts;
/// Every string the Forms panel shows. Consumed by `crate::panels::forms`.
pub mod formfield;
pub mod forms;
pub mod images;
/// Every word the Recognise-text surface says — the dialog that runs OCR and
/// discloses what it inferred, and the offer the Find bar makes on a page with
/// no text on it. Consumed by `crate::dialogs::ocr` and `crate::find::bar`.
/// ★ What the program says about a **link it cannot follow** — four
/// sentences for four different causes, plus one for a `/Link` with no
/// destination at all. A link that WORKS says nothing: it navigates, and
/// that is the feedback. See its header.
pub mod links;
pub mod markup;
/// ★ Every word the **maximum-zoom** control says — the popup behind the
/// status bar's zoom readout (O24).
///
/// Its header carries why the copy is unusually plain: the operator settled
/// the performance question himself, so the control states where the crossover
/// is and offers no advice about it.
pub mod maxzoom;
pub mod measure;
/// The copy the **context-menu** surface owns, as distinct from the copy
/// its rows borrow from [`commands`]. Currently empty by construction; its
/// header carries the argument and the list of what would land there.
pub mod menus;
pub mod merge;
/// The sized-New dialog's copy — the size list, the orientation pair, the
/// custom fields and the one refusal. Consumed by
/// `crate::dialogs::new_document`.
pub mod new_document;
pub mod ocr;
/// ★ The PAGE clipboard's four sentences — three of which are facts the
/// operator cannot see. Its header carries why a page paste is rule 4's
/// sharpest case.
pub mod pageclip;
pub mod pages;
/// Every string the Pages panel shows — the counts, the tile tooltip, the
/// four sentences an *undrawn* thumbnail can say, and the preview control.
/// Consumed by `crate::panels::pages`.
/// The object-colour control's words, including the sentence that stands where
/// a swatch cannot honestly go.
pub mod paint;
/// Every string the dock's panel bodies show. Consumed by `crate::panels`.
pub mod panels;
/// ★ Every word the **selection filter** says — the status-bar control, the
/// eleven class rows, and the standing line that appears when nothing at all
/// is selectable. Consumed by `crate::app::status` and driven by
/// `crate::canvas::pick`.
///
/// Its header carries the vocabulary argument, which is the interesting part:
/// every row has a correct specification name that would be the wrong label,
/// and the file explains each substitution — including why a form XObject is
/// called a **Block**, borrowing the CAD word for the thing rather than
/// inventing one or exposing "form XObject" to somebody who has not read the
/// specification.
pub mod pick;
/// Every word the Markup ▸ Style group shows — three tooltips and a unit.
///
/// Small, and load-bearing out of proportion to its size: the controls are a
/// colour swatch and a number, so the tooltip is the only place they can say
/// what they are, and the only place an operator learns the setting applies to
/// the **next** mark rather than to one already drawn.
/// ★ What pdfcer says after combining files — `OPERATOR_REQUESTS.md` O68.
///
/// Separate from [`files`], which owns the dialog HEADINGS the operating
/// system draws. These are what pdfcer says afterwards, on its own status row.
/// What a window says when it steps aside so the operator can point at the page
/// — `OPERATOR_REQUESTS.md` O66.
pub mod placing;
/// Every word the print dialog shows. Consumed by `crate::dialogs::print`.
pub mod print;
/// Every word the two Security controls that **write** protection into a file
/// say — O119. The WRITE side; [`security`] is the READ side, and the two are
/// separate modules because they make opposite kinds of claim. Nothing is
/// duplicated across the seam: this module calls `security`'s wording verbatim
/// wherever one fact serves both.
pub mod protect;
/// The left rail's own words — O123 part 7.
pub mod rail;
/// ★★★ The sentence a document that reaches outside itself earns — a submit
/// button, a launch action, a script that runs on open. Its header carries the
/// two opposite ways to word it wrongly.
pub mod reachout;
pub mod redact;
/// Every word the redaction surface says — the marking panel, the apply
/// report, the two acknowledgements, and the residual lines. Consumed by
/// `crate::panels::redact` and `crate::dialogs::redact`.
///
/// ★ The catalog with the strictest wording rules in the crate, and its header
/// carries all three: never say "removed" when anything was left, never say
/// "verified" unless a verification step ran, and never put the word "Undo"
/// near a post-apply state. This is the one feature where a comfortable
/// sentence is a security defect.
/// ★ Every sentence the eight resize grips show — six refusals and one
/// disclosure.
///
/// Its header carries why the refusals matter more than the feature: the grips
/// were drawn, cursored and drag-consuming for the whole life of this shell and
/// committed nothing, which is `DEFECTS.md` D4a's shape exactly.
pub mod resizing;
/// The ribbon's structural strings: tab labels and questions, group
/// captions, mode labels. Consumed by `crate::shell::manifest`.
pub mod ribbon;
/// ★ Every sentence the **ninth handle** shows — four refusals and two
/// disclosures, for `crate::canvas::rotating` and the two rotation verbs.
///
/// The sibling of [`resizing`], and its header carries the one thing worth
/// knowing before reading either: a rotation is an **isometry**, so it has no
/// stroke-scaling question, no distortion warning and no options type — which
/// is why this catalog is half the size of its neighbour despite covering three
/// kinds of target rather than one.
///
/// ★★ It carries the disclosure `pdfcer-core` asked for by name: a `Linear` ce
/// dimension's axis lock cannot survive a rotation, and *"an operator whose
/// dimension silently stopped being axis-locked will find out later and blame
/// something else."*
pub mod rotating;
/// Every word the Set-scale dialog shows. The hardest job in this catalog:
/// explaining what a ratio is measured *against*, when the honest answer for a
/// PDF is 1/72 inch and nobody's intuition is in those.
pub mod scale;
/// What the Remove-fonts window says before it takes something out - the
/// destructive twin of `embed`, and the four consequences an operator cannot
/// see on the canvas.
pub mod unembed;

/// Every word the Settings window shows — the thirteen spec-ambiguity choices,
/// what each leaves open, and what each costs.
///
/// The one area of this catalog with a rule of its own: a string here must be
/// readable by someone who has never opened the PDF standard, because the
/// operator is being asked to make a judgement and a judgement cannot be made
/// from a clause number.
/// The six strings the keyboard reference shows — and **none of them is a
/// shortcut**.
///
/// Its header carries the rule: a string here may *describe* the reference; it
/// may not be *part* of it. Every chord and every command name comes from the
/// live keymap and the registry, because `DEFECTS.md` D5 was a hand-maintained
/// list that omitted six live bindings and that nothing exercised.
/// Encryption, passwords and signatures — `OPERATOR_REQUESTS.md` O108. Two of
/// its sentences are `pdfcer-core`'s own wording and must not be re-worded; its
/// header says which and why.
pub mod security;
pub mod shortcuts;

/// **What this shell says about a digital signature before and after it
/// writes.** The claim-bearing area of the catalog: every sentence is a
/// translation of a distinction `pdfcer-core`'s `signature` module draws, and
/// its header carries the three things no string in it is allowed to say.
/// Consumed by [`crate::dialogs::signature`] and [`crate::app::save`].
pub mod signature;

pub mod settings;

/// Every string the status bar shows. Consumed by `crate::app::status`.
pub mod status;

pub mod textannot;
pub mod textedit;
/// Every sentence the text-EDITING tool shows: the three refusals a caret can
/// meet, and the rule-4 disclosure the engine does not write for a pinned tail.
/// Consumed by `crate::canvas::textedit` and by the `CommitTextEdit` apply arm.
/// Copy for the three markup kinds that carry words. Its header carries the
/// one distinction every string in it has to preserve: a text box prints and a
/// sticky note does not.
/// ★ Every word the TOOLS say, wherever they are said — the one-line status
/// strip, the Properties panel's armed-tool section, and the canvas refusals.
///
/// It was *"every word the Tool panel says"* until `OPERATOR_REQUESTS.md` O123
/// dissolved that panel; the copy outlived it and its header tabulates which
/// surface now says which sentence, and which fifteen strings were deleted with
/// the tool list rather than re-homed.
///
/// The three rules are unchanged: no label that the command registry already
/// owns, no sentence that is a tip rather than a fact, and no instruction that
/// fails to say how its gesture ends.
pub mod tool;
/// ★ The one-line tool status's own two strings — `OPERATOR_REQUESTS.md` O123.
///
/// Deliberately tiny. Everything else the strip says is already written down
/// somewhere authoritative — the tool's NAME in the command registry, its
/// SENTENCE in [`tool`], its verb in [`tool::put_down_button`] — and its
/// header tabulates which is which and why none of them was copied.
pub mod toolstatus;
/// ★★★ Whether a signature's signer can be trusted — and the four different
/// sentences for the four ways trust can go unchecked.
///
/// Kept apart from [`signature`], which is about a save that would INVALIDATE a
/// signature, because the two answer opposite questions and share only a noun.
/// Its header carries the four rules that govern it, and the one worth reading
/// first is that `NotChecked` renders as itself: never as a soft "no", never as
/// a grey tick, never omitted.
pub mod trust;
/// The words of the question `file.close` had been promising to ask since it
/// shipped, and did not.
///
/// Its own header carries the two rules the copy follows and both are unusual:
/// nothing in it says *"changes"* — it says how many **edits**, because the
/// decision an operator is being asked to make depends entirely on whether they
/// moved one dimension or spent an hour — and nothing in it says *"Save"*,
/// because this build has no Save and a button that claimed one would be the
/// same lie as the tooltip that exposed the defect.
pub mod unsaved;
/// ★★ Every sentence *"give this page its own copy"* can say — seven refusals,
/// the disclosure a **successful** unshare owes, and the remedy sentence this
/// shell appends to the engine's `SHARED CONTENT` report.
///
/// Its header carries the two things a reader must not have to rediscover.
/// First, why a feature this small needs the biggest refusal catalog on the
/// canvas: `unshare_form` is a *structural* verb, so it runs the whole
/// engine-wide guard ladder before it acts, and **not one** of its seven
/// declines is visible on the page. Second, why the SUCCESS owes a sentence
/// too — the copy is byte-identical to the original, so a page that has just
/// been unshared renders pixel-for-pixel as it did before, and without a
/// sentence the operator's evidence that it worked is indistinguishable from
/// their evidence that nothing happened.
///
/// Consumed by `crate::app::actions::xobject`,
/// `crate::app::dispatch::format` and the `CommitTextEdit` apply arm.
pub mod unshare;

use std::path::Path;

/// The window title.
///
/// Just the product name at S0. Once a document can be open, the
/// convention every document application follows is `<file> — pdfcer`, and
/// that belongs here rather than at the `ViewportBuilder` call site.
#[must_use]
pub fn window_title() -> &'static str {
    "pdfcer"
}

/// Shown on the canvas when nothing is open.
///
/// ★ **This sentence changed when `file.open` was wired**, and the change is
/// the rule rather than an edit. It used to read *"No document open. Start
/// pdfcer with a PDF path, for example: pdfcer-gui drawing.pdf"*, because at S0
/// there was no Open command and *"a message that names a control the
/// operator cannot find is worse than no message."* The command exists now —
/// on the File tab, on the quick-access toolbar, and on Ctrl+O — so the
/// message names it. The old wording would have been the same defect in
/// reverse: telling an operator to restart the application to do something
/// there is a button for.
///
/// The command line stays in the sentence because it is still true and is
/// still how a file association or a shell "Open with" reaches pdfcer.
#[must_use]
pub fn canvas_no_document() -> &'static str {
    "No document open. Choose File > Open, press Ctrl+O, or start pdfcer with a PDF path."
}

/// Shown when a document opened successfully but contains no pages.
///
/// This is a real, legal PDF: `/Count 0`. Presenting it as a failure would
/// be a lie about the operator's file, which is why the page-index clamp in
/// [`crate::viewer::clamp_page_index`] maps the empty document to page 0
/// rather than panicking — the "no pages" condition is a *presentation*
/// decision and this is the presentation.
#[must_use]
pub fn canvas_no_pages() -> &'static str {
    "This document has no pages."
}

/// Shown when the current page could not be rasterized.
///
/// The document stays open. One page that will not draw is not a reason to
/// close a file the operator can still navigate, and it is not the same
/// event as a file that would not load — hence a distinct message rather
/// than reusing [`open_failed`].
///
/// `detail` is `pdfcer-render`'s own error `Display`, passed through rather
/// than rewritten: the renderer's errors are structured, specific
/// diagnostics ("requested raster size 115200x86400 exceeds
/// MAX_PIXMAP_EDGE"), and replacing one with "an error occurred" throws
/// away the only part of the sentence that helps.
#[must_use]
pub fn canvas_render_failed(detail: &str) -> String {
    format!("This page could not be drawn. {detail}")
}

// ---------------------------------------------------------------------------
// The three things a page with no picture says about itself
// ---------------------------------------------------------------------------
//
// ★ These exist because `PROJECT_PLAN.md` §3 forbids placeholders, and a white
// rectangle where a page will be is exactly one. Under a continuous
// page-display mode several pages are on screen and the renderer fills them in
// one at a time (see `crate::render::strip`), so at any moment some of them
// have no raster. Drawing those as blank paper would be pdfcer making a claim
// about the operator's document — "sheet 12 is empty" — that it has no basis
// for and that on a drawing set is simply false.
//
// So an undrawn page states which page it is and what is happening to it.
// Three sentences rather than one, because the operator's response to each
// differs: wait a moment, wait longer, and *this page has something wrong with
// it*. The page number is 1-based, like every page number the operator sees.

/// Shown on a page whose raster is being made right now.
///
/// Present tense and an ellipsis, because something is happening and it will
/// finish. Distinguished from [`canvas_page_waiting`] so that a strip filling
/// in slowly looks like progress rather than like a stall — on the benchmark
/// CAD sheet one page takes over a second, and a row of identical "not drawn"
/// labels would give the operator no way to tell a working renderer from a
/// stuck one.
#[must_use]
pub fn canvas_page_drawing(page_number: usize) -> String {
    format!("Page {page_number} — drawing…")
}

/// Shown on a visible page the renderer has not started yet.
///
/// "Not drawn yet" rather than "loading" or a bare page number: it says
/// plainly that the absence is pdfcer's doing and is temporary, which is the
/// whole difference between this and a blank rectangle. No ellipsis, because
/// nothing is happening to *this* page at this moment — the ellipsis belongs
/// to [`canvas_page_drawing`], and spending it here would make the two
/// indistinguishable.
#[must_use]
pub fn canvas_page_waiting(page_number: usize) -> String {
    format!("Page {page_number} — not drawn yet")
}

/// Shown on a page that will not draw at all.
///
/// The per-page sibling of [`canvas_render_failed`], and the difference
/// between them is which page is being talked about: that one is shown
/// *instead of* the canvas when the current page fails, this one is shown *in
/// the failing page's own rectangle* while the pages around it draw normally.
/// One bad sheet in a forty-page set must not replace the other thirty-nine
/// with a message.
///
/// `detail` is `pdfcer-render`'s own error text, passed through rather than
/// rewritten, for the reason [`canvas_render_failed`] gives: the renderer's
/// errors are specific and replacing one with "an error occurred" throws away
/// the only part of the sentence that helps.
#[must_use]
pub fn canvas_page_refused(page_number: usize, detail: &str) -> String {
    format!("Page {page_number} could not be drawn. {detail}")
}

/// Shown when the background render thread died without reporting.
///
/// Distinguished from [`canvas_render_failed`] because the causes are
/// different in kind: a render *failure* is something about this page, and
/// a stopped worker is something about the process. Conflating them would
/// send an operator looking at their document for a fault that is ours.
#[must_use]
pub fn canvas_render_worker_stopped() -> &'static str {
    "The page renderer stopped unexpectedly. Reopen the document to try again."
}

/// The document could not be read: it is damaged, truncated, or not a PDF.
///
/// One of **three distinct ways to fail, said three distinct ways** — a
/// distinction carried across from the old shell because it is one of the
/// things pdfcer does that most viewers do not:
///
/// - this function — *the file is wrong*;
/// - [`open_unsupported`] — *the file is fine and pdfcer is not finished*;
/// - [`open_needs_password`] — *the file is encrypted and pdfcer has not
///   been told the password*.
///
/// The branch between them is made on **structured error data** from
/// `pdfcer-core`, never by matching on a message string. That is what makes
/// the distinction reliable rather than a heuristic that decays.
#[must_use]
pub fn open_failed(path: &Path, detail: &str) -> String {
    format!(
        "{} could not be opened. {detail}",
        path.file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
    )
}

/// The document is well-formed and uses something pdfcer does not implement.
///
/// Saying "failed to open" here would tell the operator a lie about their
/// own file. `pdfcer-core` detects such a document and refuses it *cleanly*
/// rather than misparsing it into plausible-looking garbage, and this
/// sentence is the other half of that honesty.
#[must_use]
pub fn open_unsupported(path: &Path, detail: &str) -> String {
    format!(
        "{} uses a PDF feature pdfcer does not support yet. {detail}",
        path.file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
    )
}

/// The document is encrypted with a password pdfcer has not been given.
///
/// A third thing: neither damaged nor unsupported. pdfcer *can* decrypt this
/// file and has not been told how.
///
/// # ★★★ IT SAID "THIS BUILD CANNOT YET PROMPT FOR A PASSWORD" WHILE PROMPTING
///
/// Corrected 2026-09-03, on an outside reviewer's report: the canvas carried
/// that sentence **in the same frame** as `dialogs::password` asked for the
/// password. Two surfaces, one `Status::NeedsPassword`, saying opposite things.
///
/// The old doc comment is worth keeping because it is the whole diagnosis:
///
/// > *S0 has no password prompt, and this message says so plainly instead of
/// > showing an input the shell would then ignore. That is the "no
/// > placeholders" invariant (`PROJECT_PLAN.md` §3) [...] The prompt lands with
/// > the rest of the open/save surface at **stage S2**.*
///
/// Every word of that was true when it was written, and it cited R9 correctly.
/// **Then S2 arrived**, `dialogs::password` shipped, `Action::OpenWithPassword`
/// shipped — and nothing re-read the sentence whose only justification was that
/// they did not exist. R9 stopped applying the moment the capability stopped
/// being unavailable.
///
/// ★ This is the sixth time this project has recorded the same shape: **a claim
/// that was true when written, cited later as evidence, with nothing re-reading
/// its premise.** It is also the reason the canvas arm in `app::surfaces` no
/// longer draws anything for this status: the dialog IS the surface now, and a
/// second surface repeating a superseded fact is how the two came to disagree.
#[must_use]
pub fn open_needs_password(path: &Path) -> String {
    format!(
        "{} is password-protected. Enter its password to open it.",
        path.file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// The three open-failure sentences must be genuinely different.
    ///
    /// Not a tautology test: the whole value of the three-way distinction
    /// is that an operator can tell from the words alone which of "my file
    /// is broken", "pdfcer is not finished" and "I need to type a password"
    /// is true. Three functions that produced near-identical prose would
    /// satisfy the type system and defeat the design.
    #[test]
    fn the_three_open_failures_read_differently() {
        let p = PathBuf::from("drawing.pdf");
        let a = open_failed(&p, "unexpected end of file");
        let b = open_unsupported(&p, "hybrid-reference file");
        let c = open_needs_password(&p);
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
        // And each must name the file, or the operator with two documents
        // open cannot tell which one is complaining.
        for message in [&a, &b, &c] {
            assert!(message.contains("drawing.pdf"));
        }
    }

    /// A path with no file name must still produce a usable sentence.
    ///
    /// `Path::file_name` returns `None` for a bare root or a path ending in
    /// `..`, and an unwrap there would turn a nonsense command line into a
    /// panic instead of a message.
    #[test]
    fn a_path_without_a_file_name_still_names_something() {
        let message = open_failed(Path::new("D:\\"), "not a PDF");
        assert!(message.contains("D:\\"));
    }
}
