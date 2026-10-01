//! # text — the operator-visible string catalog
//!
//! **Every string a human can read in this application is defined here and
//! nowhere else.** That is a standing convention carried across from the
//! old crate (`ui_text.rs`, 7,912 lines and 1,193 entries), and it is
//! enforced mechanically rather than by review: a CI gate scans the module
//! tree for string literals outside the catalog and fails the build.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/mod.md`.

pub mod about;
/// What the clipboard verbs say on the status row.
pub mod clipboard;

/// **Every word `OPERATOR_REQUESTS.md` O122 puts on screen** — the
/// *Open in Acrobat* control beside the mode selector, the three things it can
/// say before it acts, and the Settings field that says where Acrobat is. One
/// module for four surfaces because they are one conversation; see its header.
/// Consumed by `pdfcer_gui::shell::commands`, `pdfcer_gui::dialogs::open_in_acrobat` and
/// `pdfcer_gui::dialogs::settings::acrobat`.
pub mod acrobat;
/// Every word the About dialog shows, plus the structured attribution catalog
/// naming the third-party material this binary redistributes. Consumed by
/// `pdfcer_gui::dialogs::about`.
/// **Reading a comment where the comment is** — every word the canvas
/// note pop-up and its hover tooltip show. Consumed by
/// `pdfcer_gui::canvas::notepopup`, which is the only route to a note's `/Contents`
/// that works in Read mode. Its header carries the two capabilities that are
/// deliberately WORDLESS here, under R9, because the engine cannot reach them.
pub mod annotpopup;
/// **What the file said twice, and which reading pdfcer used** — every
/// word of the load-anomaly disclosure that engine `Pass 283.0` made owed.
/// Consumed by `pdfcer_gui::app::status::disclosure` for the status bar's census
/// line and by `pdfcer_gui::panels::docprops` for the per-object detail. Its header
/// argues why there is deliberately no "this file opened cleanly" string.
pub mod anomalies;
/// **Moving a mark that is already on the page** — the four refusals an
/// arrow-key nudge can owe and the five disclosures a *Bring to front* can.
/// Consumed by `pdfcer_gui::canvas::moving::nudge` and
/// `pdfcer_gui::app::actions::reorder`. One catalog for two gestures because they
/// refuse for the same three reasons in the same words; its header argues why
/// the lock sentence is deliberately NOT the Properties panel's.
pub mod arrange;
/// **Every word `OPERATOR_REQUESTS.md` O173 puts on screen** — the
/// ask-once offer, the Settings group it lives in afterwards, the line that says
/// what Windows actually opens PDFs with, and the two strings Windows itself
/// displays in its *Open with* menu. Consumed by `pdfcer_gui::app::assoc`,
/// `pdfcer_gui::dialogs::defaultapp` and `pdfcer_gui::dialogs::settings::defaultapp`. Its
/// header carries the platform fact every word is shaped by: no program can
/// make itself the default PDF viewer, so the offer must never claim it did.
pub mod assoc;
/// The attachment clipboard's words, including the one question a paste must
/// ask before the press: the engine REPLACES a same-named attachment.
pub mod attachclip;
/// Pages ▸ Stamp ▸ Bates numbering…: its window and its receipt.
/// Consumed by `pdfcer_gui::dialogs::bates`.
pub mod bates;
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
/// Every word the Render-diagnostics dialog adds around the findings — the
/// title, the three measurements of the render itself, and the two states in
/// which there is nothing to report. The findings themselves stay in
/// [`status`]. Consumed by `pdfcer_gui::dialogs::diagnostics`.
pub mod diagnostics;
/// Every word the Manage-dimension-groups window shows.
pub mod dimension_groups;
/// **The document tab strip, and the page drag between documents.** What a tab
/// says, and what a drag says it is about to do.
pub mod doctabs;
pub mod dropped;
/// What a value box says when it cannot read what was typed.
pub mod entry;
/// What the measure tools say about what they INFERRED — the two-line
/// gesture's refusals, the angle an override overrode, and an apex that is
/// only real if the lines are extended.
pub mod export_dxf;
pub mod export_form;
/// Every word File ▸ Export ▸ Tables… shows, and its receipt.
pub mod export_tables;
/// What the Word and table exports add when they follow a tagged PDF's own structure.
pub mod export_tagged;
/// Every word the Export-text window shows, and every sentence a text
/// export owes afterwards.
pub mod export_text;
/// Every word File ▸ Export ▸ Word document… shows, and its receipt.
pub mod export_word;
/// The copy the open/close/recent surface owns — the file dialog's title and
/// filter names, and every string the Recent control draws. Consumed by
/// `pdfcer_gui::app::files` and `pdfcer_gui::app::recent`.
pub mod files;
/// Every string the Find bar shows, plus the status bar's Find toggle.
/// Consumed by `find::bar` and `pdfcer_gui::app::status`.
pub mod find;
/// What making a markup part of the page says.
pub mod flattenannot;
pub mod formfonts;
pub mod forms;
pub mod images;
/// **The words of the Import-text window** — the return journey's chooser.
/// Its header carries the one way it departs from `export_text`'s shape:
/// exporting names LOSSES and importing names INVENTIONS, so this window is a
/// chooser rather than a warning.
pub mod import_text;
/// **What the import says AFTERWARDS** — the receipt and the refusals. Its
/// header records why the engine's own `PlaceTextReport::disclosures` are not
/// printed: they are correct, useful to a developer, and written in the
/// implementer's voice.
pub mod importtext;
/// Pages ▸ Stamp ▸ Number pages…: its window and its receipts.
/// Consumed by `pdfcer_gui::dialogs::labels`.
pub mod labels;
/// Every word the Recognise-text surface says — the dialog that runs OCR and
/// discloses what it inferred, and the offer the Find bar makes on a page with
/// no text on it. Consumed by `pdfcer_gui::dialogs::ocr` and `find::bar`.
/// What the program says about a **link it cannot follow** — four
/// sentences for four different causes, plus one for a `/Link` with no
/// destination at all. A link that WORKS says nothing: it navigates, and
/// that is the feedback. See its header.
pub mod links;
pub mod markup;
/// Every word the **maximum-zoom** control says — the popup behind the
/// status bar's zoom readout (O24).
pub mod maxzoom;
pub mod measure;
pub mod merge;
/// The sized-New dialog's copy — the size list, the orientation pair, the
/// custom fields and the one refusal. Consumed by
/// `pdfcer_gui::dialogs::new_document`.
pub mod new_document;
pub mod ocr;
/// The words for **content that is in the file but not on the
/// sheet** — the census window's copy. Its header carries the sentence the
/// whole module exists for: off-page marks do not render and are still
/// extractable, so the disclosure leads with WHAT THEY SAY rather than with a
/// count. Consumed by `pdfcer_gui::dialogs::offpage` and
/// `pdfcer_gui::app::actions::offpage`.
pub mod offpage;
/// The words for **the visible area of a sheet** — the Crop… window and the
/// disclosures its commit raises. Consumed by `pdfcer_gui::dialogs::page_crop`
/// and `pdfcer_gui::app::actions::pagesize::crop`.
pub mod page_crop;
/// The words for **changing the paper an open drawing sits on** — the
/// sheet-size window and the disclosures its commit raises. Consumed by
/// `pdfcer_gui::dialogs::page_size` and `pdfcer_gui::app::actions::pagesize`. Its header
/// carries the one sentence the whole module exists for: a `/MediaBox` change
/// changes the **paper**, and does not move or scale anything drawn on the
/// page — which is the opposite of what every other "page size" control an
/// operator has ever used does.
pub mod page_size;
/// The PAGE clipboard's four sentences — three of which are facts the
/// operator cannot see. Its header carries why a page paste is rule 4's
/// sharpest case.
pub mod pageclip;
pub mod pages;
/// The two sentences a save refused by [`crate::pagetree`]'s structural
/// guard says — *"this document says it has 36 pages and only 34 are really
/// there"*. Consumed by `pdfcer_gui::app::save`. Its header carries the wording
/// rule that makes them recognisable to the operator who reported the defect:
/// name the symptom the **other** reader will show, because pdfcer's own
/// reader cannot see anything wrong.
pub mod pagetree;
/// Every string the Pages panel shows — the counts, the tile tooltip, the
/// four sentences an *undrawn* thumbnail can say, and the preview control.
/// Consumed by `pdfcer_gui::panels::pages`.
/// The object-colour control's words, including the sentence that stands where
/// a swatch cannot honestly go.
pub mod paint;
/// Every string the dock's panel bodies show. Consumed by `pdfcer_gui::panels`.
pub mod panels;
/// Every word the Markup ▸ Style group shows — three tooltips and a unit.
pub mod placing;
/// Every word the print dialog shows. Consumed by `pdfcer_gui::dialogs::print`.
pub mod print;
/// Every word File ▸ Security ▸ Remove old passwords… shows, and its receipt.
pub mod purge_passwords;
/// The left rail's own words — O123 part 7.
pub mod rail;
/// Every word the **review-status** control says — `/State` and
/// `/StateModel` (§12.5.6.3, Table 171), consumed by
/// `crate::commentreviewstate` and by
/// `pdfcer_gui::app::actions::reviewstate`.
pub mod reviewstate;
/// The ribbon's structural strings: tab labels and questions, group
/// captions, mode labels. Consumed by `pdfcer_gui::shell::manifest`.
pub mod ribbon;
/// Every sentence the **ninth handle** shows — four refusals and two
/// disclosures, for `pdfcer_gui::canvas::rotating` and the two rotation verbs.
pub mod rotating;
/// Why merging text runs was declined, and the width it wrote.
pub mod runmerge;
/// Every word the Set-scale dialog shows. The hardest job in this catalog:
/// explaining what a ratio is measured *against*, when the honest answer for a
/// PDF is 1/72 inch and nobody's intuition is in those.
pub mod scale;
/// Every word the tools say: the one-line status strip, the Properties
/// panel's armed-tool section, and the canvas refusals.
pub mod tool;
/// What the Remove-fonts window says before it takes something out - the
/// destructive twin of `embed`, and the four consequences an operator cannot
/// see on the canvas.
pub mod unembed;

/// Every word the Settings window shows — the thirteen spec-ambiguity choices,
/// what each leaves open, and what each costs.
pub mod security;
pub mod shortcuts;

/// File ▸ Security ▸ Add archive time-stamp….
#[cfg(feature = "signing")]
pub mod archive;
/// File ▸ Security ▸ Add validation evidence….
#[cfg(feature = "signing")]
pub mod evidence;
/// Every operator-facing string on the control that SIGNS a document — the
/// write side of a subject whose read side is `text::security` (what a document
/// says about its protection) and `text::trust` (what pdfcer could and could
/// not check about a signature that already exists).
#[cfg(feature = "signing")]
pub mod sign;
/// **What this shell says about a digital signature before and after it
/// writes.** The claim-bearing area of the catalog: every sentence is a
/// translation of a distinction `pdfcer-core`'s `signature` module draws, and
/// its header carries the three things no string in it is allowed to say.
/// Consumed by `pdfcer_gui::dialogs::signature` and `pdfcer_gui::app::save`.
pub mod signature;

pub mod settings;

/// **Custom stamp collections** — the Save-as-stamp-collection window and
/// the Document Properties section that discloses a collection someone else
/// wrote. One catalog for two surfaces because they share a vocabulary, and
/// its header fixes that vocabulary: *stamp*, *collection*, *category*,
/// *display name*. ⚠ Carries the sentence the engine's `page_index` defect
/// makes dangerous to word — see `properties_stamp_no_page`.
/// Consumed by `crate::stamps`, `pdfcer_gui::dialogs::stamp_collection` and
/// `pdfcer_gui::panels::docprops`.
pub mod stamps;

/// Every string the status bar shows. Consumed by `pdfcer_gui::app::status`.
pub mod status;

/// **Why a committed text edit was refused** — split out of [`textedit`] on
/// 2026-09-06 under R2, along the seam that file's own section banner had
/// already drawn. Everything in it is re-exported from [`textedit`], so no call
/// site moved.
pub mod editrefusal;
/// Every sentence the text-editing tool shows: the refusals a caret can meet,
/// and the disclosure the engine does not write for a pinned tail.
pub mod textedit;
/// The one-line tool status's own two strings — `OPERATOR_REQUESTS.md` O123.
pub mod toolstatus;
/// Whether a signature's signer can be trusted — and the four different
/// sentences for the four ways trust can go unchecked.
pub mod trust;
/// The words of the question `file.close` had been promising to ask since it
/// shipped, and did not.
pub mod unsaved;
/// Every sentence *"give this page its own copy"* can say — seven refusals,
/// the disclosure a **successful** unshare owes, and the remedy sentence this
/// shell appends to the engine's `SHARED CONTENT` report.
pub mod unshare;

/// **The way back out of a mode that hides its own control** — the read-mode
/// exit, said on the window title and on the status bar.
pub mod window;

/// The places a copy of the removed text can hide.
pub mod redactcarriers;

/// The copy naming where the redacted document goes.
pub mod redactdestcopy;

/// The removed words themselves, not the count of them.
pub mod redactremoved;

/// **What a push button DOES** — every word the placement dialog's action
/// chooser says, including the submit disclosure. Its own module because two of
/// the seven choices write an address into the document that some other program
/// may act on, and the operator cannot see that by looking at the page.
pub mod buttonaction;
/// The three sentences a held Shift puts on the status row while it is
/// constraining a drag. Consumed by `crate::canvas::constrain::caption`.
pub mod constrain;
/// **The three sentences a Delete that removed nothing shows** — for
/// `crate::canvas::deleting`, the module that routes a Delete to the verb for
/// the rung the operator is on.
pub mod deleting;
pub mod embed;
/// Every word the Export-image window shows, and every sentence an image
/// export owes afterwards. `OPERATOR_REQUESTS.md` O120.
pub mod export_image;
/// The SVG/EMF keep-text choice and what an export that kept text owes afterwards.
pub mod export_keeptext;
/// The FORM-FIELD clipboard's sentences — five refusals and the paste's
/// off-canvas loss note. Separate from [`clipboard`] because the loss note is
/// not a refusal: the paste worked, and the sentence exists because part of the
/// field could not travel and the operator cannot see which part.
pub mod fieldclip;
/// What the font-donor scan says when it skips a file — five sentences, all
/// about something that did not happen. See its header for why a skip is worth
/// a sentence.
pub mod fonts;
/// Every word the **selection filter** says — the status-bar control, the
/// eleven class rows, and the standing line that appears when nothing at all
/// is selectable. Consumed by `crate::app::status` and driven by
/// `crate::canvas::pick`.
pub mod pick;
/// The sentence a document that reaches outside itself earns — a submit
/// button, a launch action, a script that runs on open. Its header carries the
/// two opposite ways to word it wrongly.
pub mod reachout;
/// Every word the redaction surface says — the marking panel, the apply
/// report, the two acknowledgements, and the residual lines. Consumed by
/// `pdfcer_gui::panels::redact` and `pdfcer_gui::dialogs::redact`.
pub mod redact;
pub mod resizing;
pub mod textannot;

use std::path::Path;

/// The window title.
#[must_use]
pub fn window_title() -> &'static str {
    "pdfcer"
}

/// Shown on the canvas when nothing is open.
#[must_use]
pub fn canvas_no_document() -> &'static str {
    "No document open. Choose File > Open, press Ctrl+O, or start pdfcer with a PDF path."
}

/// Shown when a document opened successfully but contains no pages.
#[must_use]
pub fn canvas_no_pages() -> &'static str {
    "This document has no pages."
}

/// Shown when the current page could not be rasterized.
#[must_use]
pub fn canvas_render_failed(detail: &str) -> String {
    format!("This page could not be drawn. {detail}")
}

/// The `detail` clause for the one render refusal that has a remedy the
/// operator can carry out, and the reason this shell does **not** pass the
/// engine's own sentence through here.
#[must_use]
pub fn canvas_zoom_past_rasterizer() -> &'static str {
    "This zoom is further in than pdfcer can rasterize. Zoom out and it will draw again. Nothing about the page has changed."
}

// ---------------------------------------------------------------------------
// The three things a page with no picture says about itself
// ---------------------------------------------------------------------------
//
// These exist because `PROJECT_PLAN.md` §3 forbids placeholders, and a white
// rectangle where a page will be is exactly one. Under a continuous
// page-display mode several pages are on screen and the renderer fills them in
// one at a time (see `pdfcer_gui::render::strip`), so at any moment some of them
// have no raster. Drawing those as blank paper would be pdfcer making a claim
// about the operator's document — "sheet 12 is empty" — that it has no basis
// for and that on a drawing set is simply false.
//
// So an undrawn page states which page it is and what is happening to it.
// Three sentences rather than one, because the operator's response to each
// differs: wait a moment, wait longer, and *this page has something wrong with
// it*. The page number is 1-based, like every page number the operator sees.

/// Shown on a page whose raster is being made right now.
#[must_use]
pub fn canvas_page_drawing(page_number: usize) -> String {
    format!("Page {page_number} — drawing…")
}

/// Shown on a visible page the renderer has not started yet.
#[must_use]
pub fn canvas_page_waiting(page_number: usize) -> String {
    format!("Page {page_number} — not drawn yet")
}

/// Shown on a **neighbour** page in a continuous strip whose whole-sheet raster
/// is larger than the renderer can allocate at the zoom the operator is at.
#[must_use]
pub fn canvas_page_beyond_raster(page_number: usize) -> String {
    format!("Page {page_number} — not drawn this far in. Zoom out to see it.")
}

/// Shown on a page that will not draw at all.
#[must_use]
pub fn canvas_page_refused(page_number: usize, detail: &str) -> String {
    format!("Page {page_number} could not be drawn. {detail}")
}

/// Shown when a page's box is empty.
#[must_use]
pub fn canvas_page_has_no_area() -> &'static str {
    "This page has no area to draw — its page box is empty."
}

/// Shown when the background render thread died without reporting.
#[must_use]
pub fn canvas_render_worker_stopped() -> &'static str {
    "The page renderer stopped unexpectedly. Reopen the document to try again."
}

/// The sentence for a render that came back with no pixels.
#[must_use]
pub fn render_refusal(reason: &crate::renderworker::RefusalReason) -> String {
    use crate::renderworker::RefusalReason as R;
    match reason {
        R::PastRasterizer => canvas_zoom_past_rasterizer().to_owned(),
        R::NoArea => canvas_page_has_no_area().to_owned(),
        R::WorkerStopped => canvas_render_worker_stopped().to_owned(),
        R::Engine(sentence) => sentence.clone(),
    }
}

/// The document could not be read: it is damaged, truncated, or not a PDF.
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

    /// **The engine's own sentence must still be free of panic text.**
    #[test]
    fn the_engine_keeps_its_panic_text_out_of_the_message() {
        let panic_message =
            "range start index 442613758592 out of range for slice of length 1088737";
        let e = pdfcer_render::RenderError::RasterizerLimit {
            scale: 8_053_069.0,
            panic_message: panic_message.to_owned(),
        };
        let shown = e.to_string();
        assert!(
            !shown.contains(panic_message) && !shown.contains("range start index"),
            "pdfcer-render put third-party panic text back into RasterizerLimit's Display, \
             which makes the wildcard arm in renderworker unsafe again: {shown}"
        );
        assert!(
            shown.contains("8053069"),
            "the scale is the half of this refusal a caller can act on and it must survive: {shown}"
        );
    }

    /// **This shell's sentence is an instruction, not a paraphrase of the
    /// engine's fact.**
    #[test]
    fn the_zoom_refusal_tells_him_what_to_do_and_that_nothing_broke() {
        let ours = canvas_zoom_past_rasterizer();
        assert!(
            !ours.contains("rasterizer cannot work at scale"),
            "this must not decay into a restatement of the engine's Display: {ours}"
        );
        assert!(
            ours.contains("Zoom out"),
            "the remedy is the only actionable half of this event: {ours}"
        );
        assert!(
            ours.contains("Nothing about the page has changed"),
            "a refusal where a picture was reads as damage unless it says otherwise: {ours}"
        );
    }

    /// A path with no file name must still produce a usable sentence.
    #[test]
    fn a_path_without_a_file_name_still_names_something() {
        let message = open_failed(Path::new("D:\\"), "not a PDF");
        assert!(message.contains("D:\\"));
    }
}

/// Every word the two Security controls that **write** protection into a file
/// say — O119. The WRITE side; [`security`] is the READ side, and the two are
/// separate modules because they make opposite kinds of claim. Nothing is
/// duplicated across the seam: this module calls `security`'s wording verbatim
/// wherever one fact serves both.
pub mod protect;

/// Every string the form-field pop-up shows.
pub mod formfield;
