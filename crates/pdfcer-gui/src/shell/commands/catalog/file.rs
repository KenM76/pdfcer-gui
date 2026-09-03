//! # `shell::commands::catalog::file` — the File tab — opening, saving, exporting, printing, and pdfcer itself
//!
//! One band of [`super::all`]'s catalogue. Split out of [`super`] under **R2**
//! on 2026-08-28, when the Attachments command took that file to 1,495 of its
//! 1,500 lines and the next command registered would have broken the rule.
//!
//! ## ★★★ The split is per TAB, and the reason it was refused before is gone
//!
//! [`super`]'s header argued against exactly this cut:
//!
//! > a per-tab split would put the handler-token blocks in eight files where a
//! > collision between two of them is invisible.
//!
//! **That objection was already false when it was written.**
//! `super::super::tests::every_handler_token_is_unique` sweeps the whole
//! registry, and `every_handler_token_is_in_its_tabs_block` asserts each token
//! sits in its own tab's hundred. A collision is not invisible — it is a red
//! test, in either arrangement — so the argument that kept 120 commands in one
//! file rested on a property two tests had already taken over.
//!
//! ⇒ Recorded rather than quietly reversed, because it is the same shape this
//! project keeps finding: **a reason that was true when written, is checked by
//! nobody, and outlives what made it true.**
//!
//! ## What is here, and what is not
//!
//! The `Command` entries and the argument for each one's label, tooltip,
//! handler token, icon and enable predicate. **The prose is the point** — most
//! of this file is the record of decisions that would otherwise be re-litigated,
//! which is also why the byte count grew past a limit in the first place.
//!
//! Not here: the registration itself ([`super::super::register`]), the
//! command-id-to-behaviour mapping ([`super::super::mapping`]), and the
//! reachability register ([`super::super::reach`]).

use egui_shell::Command;

use super::super::FILE_RECENT;
use super::command;
use crate::text::commands as t;

/// This band's commands, in ribbon order.
pub(super) fn band() -> Vec<Command> {
    vec![
        // ★ **New — first in the band, and with no glyph.**
        //
        // Order: New, Open, Recent, Close. All three reference applications
        // open their File menu with New and follow it with Open, and this is
        // also the useful order — the two ways to *get* a document, then the
        // two ways to get one *back*, then the way to put one away.
        //
        // **No icon, and it is a recorded refusal rather than an oversight.**
        // The refusal has `file.ocr`'s reason, which is the one reason on the
        // list that is not about the drawing being hard: `icons/assets/`
        // declares itself the **operator's own art**, and that declaration is
        // exactly what exempts the directory from `check-shipped-assets`'
        // notice surfaces. A machine-drawn SVG added by this session would make
        // that provenance note false, and a false provenance note is a worse
        // defect than a control that draws its own word. Reusing an existing
        // key was considered and refused too: `document` is the Properties
        // glyph, `insert-pages` means *pages into this document*, and `upload`
        // is the import half of the export pair — each would say something New
        // does not do. A blank-page glyph is the operator's to draw, and until
        // it exists this control reads `New`, which nobody has ever had to look
        // up.
        //
        // **No enable predicate**, like `file.open` and for the same reason: an
        // operator with nothing open is exactly the operator most likely to
        // want this.
        command("file.new", t::file_new(), 103),
        // ★ The sized New, immediately after the plain one.
        //
        // Order matters here in the way §5.1's own table does: New, New from
        // template, Open, Recent, Close. The two ways to MAKE a document sit
        // together, then the two ways to get one back, then the way to put one
        // away.
        //
        // **No icon**, for `file.new`'s reason and not a new one: the icon
        // directory is declared the operator's own art, and the two New
        // controls sharing a glyph they do not have would be worse than the
        // two of them reading `New` and `New from template…`.
        //
        // **No enable predicate**, for the strongest version of `file.new`'s:
        // an operator with nothing open is not somebody this is tolerated for,
        // they are the operator it exists for.
        command("file.new_from_template", t::file_new_from_template(), 104),
        command("file.open", t::file_open(), 100).with_icon("open"),
        command("file.close", t::file_close(), 101)
            .with_icon("close")
            .enabled_when("doc.open"),
        // ★ Recent — the one command whose ribbon control is NOT a button.
        //
        // It is drawn by the `recent_files` custom item in File ▸ File, which
        // is what asks *which* of the ten documents; this command is the verb
        // that opens the answer. `super::manifest::CUSTOM_BACKED` records the
        // arrangement and `super::tests::no_registered_command_is_orphaned`
        // consults that register, so a command with no route at all still
        // fails while this one passes for a stated reason.
        //
        // **No enable predicate**, and that is deliberate rather than an
        // omission. The vocabulary of conditions is five names published once
        // per frame, and "the operator has opened something before" would be a
        // sixth that only one control reads — while that control is a menu
        // that has to decide its own greying anyway, from a list it already
        // holds. So the availability rule lives with the control
        // (`app::recent::menu`, which greys the button on an empty list and
        // explains it on hover, exactly as P3 requires) and this command stays
        // available to a keymap or a customized quick-access toolbar, where it
        // opens the newest document it can still see.
        //
        // No icon: `open` belongs to `file.open` and reusing it would make two
        // adjacent controls in one band look like one control drawn twice. A
        // command with no key renders as text, which is a real answer — see
        // the header — and the right one for a menu button whose label is a
        // word.
        command(FILE_RECENT, t::file_recent(), 102),
        // ★★★ **Save**, 2026-08-20. See `text::commands::file_save` for the
        // argument that used to keep it out, and why that argument was aimed at
        // the wrong hazard.
        //
        // It takes the `save` glyph, and Save-a-copy gives it up. Two adjacent
        // controls sharing one key would read as one control drawn twice - this
        // module's own convention - and of the two, the glyph belongs to the one
        // an operator presses fifty times a day without reading the label.
        // Save-a-copy renders as text, which is a real answer for a control
        // whose whole meaning is in the words "a copy".
        //
        // `doc.open` rather than a modified-document condition: Save is live
        // whenever there is a document, exactly as it is everywhere else. A
        // Save that greys itself when there is nothing to save is a Save the
        // operator has to think about.
        command("file.save", t::file_save(), 111)
            .with_icon("save")
            .enabled_when("doc.open"),
        command("file.save_copy", t::file_save_copy(), 110).enabled_when("doc.open"),
        // ★ **Save As**, `OPERATOR_REQUESTS.md` O95 — beside Save a copy and
        // not instead of it. The two are different acts (see `Action::SaveAs`),
        // and every editor the operator uses offers both.
        //
        // ★ No icon, on the same reasoning `file.new` and `file.ocr` record:
        // the icon directory is declared the operator's own art, every reuse
        // here would mislead, and a control that draws its own words is better
        // than a false provenance note.
        command("file.save_as", t::file_save_as(), 113).enabled_when("doc.open"),
        // ★ `doc.open`, like its two neighbours, and NOT gated on the document
        // having anything to reclaim. A file with nothing unused still gets a
        // copy — the window says so — because an operator who asked for one is
        // owed it, and because whether there is anything to reclaim is not
        // knowable without serialising the document, which a ribbon predicate
        // evaluated every frame must not do.
        command("file.save_compacted", t::file_save_compacted(), 112).enabled_when("doc.open"),
        // Both export verbs share `export`, and that is the header's
        // shared-key convention rather than an oversight: the glyph is the
        // download twin of `insert-pages`' upload art, reserved for exactly
        // this by the icon ui-spec §3.1, and what it says — "out of this
        // document, into a file" — is equally and completely true of both.
        // What differs is the format, which is a word only the label can say.
        command("file.export_dxf", t::file_export_dxf(), 120)
            .with_icon("export")
            .enabled_when("doc.pages"),
        command("file.export_form_data", t::file_export_form_data(), 121)
            .with_icon("export")
            .enabled_when("doc.open"),
        // ★★ Registered beside its twin and reached the same way. The two are
        // one round trip, and `file.export_form_data`'s note above applies to
        // both: the FORMAT is the file's extension, not a third dialog.
        //
        // ★ `doc.pages` rather than a condition about whether the document has
        // a form. A certification refusal, an encrypted file and a document
        // with no `/AcroForm` are all engine answers this shell would have to
        // ask for per frame, and all three are worded declines at the moment of
        // the press — `dispatch::forms`' standing ruling: *greying is a hint;
        // the worded decline is the answer.*
        command("file.import_form_data", t::file_import_form_data(), 118)
            .with_icon("import-form-data")
            .enabled_when("doc.pages"),
        // ★ **Copy page text / Copy document text — were `edit.copy_page_text`
        // and `edit.copy_document_text`, tokens 420 and 421, until the operator
        // decided on 2026-08-14 that they belong here.**
        //
        // This is the same taxonomy move `view.panel_forms` records one block
        // down, applied to the same line from the other side. Filling a form is
        // not authoring; **copying text out is not authoring either**. Both
        // verbs read the document and write somewhere that is not the document,
        // and neither can change a byte of the file.
        //
        // # What forced it, and why the Edit tab was the wrong home
        //
        // The chord/mode gate landed the same day
        // (`crate::app::modes::capability::offers_command`): a chord may reach a
        // command the active mode **shows**, or one that lives on **no ordinary
        // tab**. `Ctrl+Shift+C` is bound to the page-text copy, which sat on the
        // Edit tab — a tab Read does not show — so Read refused it. Acrobat
        // Reader copies text, and *replacing Acrobat Reader* is this project's
        // stated goal for Read, so a Read that cannot copy is wrong about the
        // one thing that mode exists to be. The gate SURFACED that; it did not
        // cause it. The command had been on the wrong tab since the day it
        // arrived there.
        //
        // # Why File ▸ Export rather than a new group or View
        //
        // The File tab is in every mode's tab list, so a command here is
        // reachable from Read, Review and Edit without any exception list — the
        // gate's own second clause never has to be invoked. And this group is
        // the right group rather than merely an available one: `file.export_dxf`
        // writes the page's geometry out to a file another program reads, and
        // `file.export_form_data` writes the filled values out the same way.
        // **Copying the page's text out is an export of content**, differing
        // only in the destination — a clipboard rather than a path — which is a
        // difference the labels carry and the caption does not need to.
        //
        // # Tokens 122 and 123, and the two gaps left behind
        //
        // New ids get new numbers in the `file.` block; 420 and 421 stay unused
        // for the reason the header states and `edit.form_fill`'s vacated 430
        // already demonstrates — a token is what a trace prints, and reusing one
        // would make an old trace of a text copy read as whatever inherited its
        // number. Gaps in the numbering are fine and expected.
        //
        // The `copy` icon, the `doc.pages` predicate and both tooltips come
        // across unchanged: nothing about what these commands DO has moved, only
        // where an operator finds them. `doc.pages` in particular is still the
        // right predicate rather than `doc.open` — text is drawn on pages, and a
        // legal `/Count 0` document has none to copy from.
        command("file.copy_page_text", t::file_copy_page_text(), 122)
            .with_icon("copy")
            .enabled_when("doc.pages"),
        command("file.copy_document_text", t::file_copy_document_text(), 123)
            .with_icon("copy")
            .enabled_when("doc.pages"),
        // ★ Print had no icon because the salvage source drew it with the
        // *stamp* glyph, and that was a mis-assignment rather than a
        // convention to carry — `stamp` means "a mark applied with a stamp"
        // (icon ui-spec §3.4) and is shared with the reserved Bates glyph.
        //
        // Declining the wrong glyph was right; it was never a reason to have
        // none. `print` is the printer art the ui-spec §8.12 reserved, and it
        // collides with nothing.
        command("file.print", t::file_print(), 130)
            .with_icon("print")
            .enabled_when("doc.open"),
        command("file.properties", t::file_properties(), 140)
            .with_icon("properties")
            .enabled_when("doc.open"),
        command("file.fonts", t::file_fonts(), 141)
            .with_icon("fonts")
            .enabled_when("doc.open"),
        // Settings, the shortcut list and About are always available: they
        // are about pdfcer, not about a document.
        command("file.settings", t::file_settings(), 150).with_icon("settings"),
        command("file.shortcuts", t::file_shortcuts(), 151).with_icon("keyboard"),
        // ★ `file.about` carries an OBLIGATION, not a courtesy: it is the
        // in-application half of the attribution surface that shipping
        // CC-BY-SA-4.0 OCR model weights requires, BY needing the notice to
        // reach the RECIPIENT rather than a reader of the repository. The
        // argument is in `crate::text::about`; the gate that keeps both
        // halves true is `tools/gates/check-shipped-assets.py`.
        command("file.about", t::file_about(), 152).with_icon("info"),
        // ★ `file.ocr` — REGISTERED WITH NO ICON. The refusal's full argument
        // is the `file.ocr` row of this module's header table; in one line, the
        // icon directory is declared the operator's OWN ART, so a new glyph is
        // not a build session's to add, and every available reuse would tell an
        // operator the button does something it does not.
        //
        // `doc.pages` rather than `doc.open`: recognition needs a page to
        // rasterize, and a document with none would open a dialog whose only
        // possible outcome is a refusal.
        //
        // ★ On the FILE tab, where `RIBBON_IA.md` §5.7 says Tools. Read's tab
        // list is `["file", "view"]`, so Tools would put OCR out of reach in
        // the one mode the operator asked for it in. Argued in full in
        // `super::manifest::tools`'s header.
        command("file.ocr", t::file_ocr(), 160).enabled_when("doc.pages"),
    ]
}
