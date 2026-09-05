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
        // ★ **New — first in the band, and it now draws a glyph.**
        //
        // Order: New, Open, Recent, Close. All three reference applications
        // open their File menu with New and follow it with Open, and this is
        // also the useful order — the two ways to *get* a document, then the
        // two ways to get one *back*, then the way to put one away.
        //
        // ★★★ **The written refusal here is DISCHARGED as of 2026-09-04**, by
        // `new-document` — a glyph drawn for this role and adopted from the
        // outside review of 2026-09-03.
        //
        // # What was refused, and why the refusal is spent rather than wrong
        //
        // The refusal ran on `file.ocr`'s reason, which was the one reason on
        // the list that was never about the drawing being hard: `icons/assets/`
        // declares itself the **operator's own art**, and that declaration is
        // exactly what exempts the directory from `check-shipped-assets`'
        // notice surfaces. A machine-drawn SVG added by a build session would
        // have made that provenance note false, and a false provenance note is
        // a worse defect than a control that draws its own word. **That
        // argument was correct and it is still correct.** What changed is not
        // the argument but its premise: the asking happened, the art came back
        // from outside this session, and `icons::catalog`'s own note on the
        // batch records it the same way — *the refusal is spent rather than
        // overturned.* Nothing here was drawn by a build session; the
        // provenance note remains true and `PROVENANCE.md` is untouched.
        //
        // # ★ The part of the old refusal that OUTLIVES it — do not lose this
        //
        // The refusal also named three reuses and rejected each by name, and
        // **those three rejections are still load-bearing**, because they are
        // the shape cues that keep this glyph from converging on a neighbour in
        // a later "consistency" pass. `Icon::New`'s own doc agrees clause for
        // clause, and the four separations it must keep are:
        //
        // * **NOT `properties`** (`document.svg`) — the same square-on page,
        //   but ruled with text and unfolded. That one means *the file already
        //   open*; this one means *a file that does not exist yet*. The FOLD
        //   and the absence of text rules are the cue.
        // * **NOT `insert-pages`** — a tray with an arrow going IN, i.e. pages
        //   into a document that already exists.
        // * **NOT `export`/`upload`** — the import half of the export pair,
        //   which is about a direction of travel this command has none of.
        // * **NOT `new-from-template`**, its immediate ribbon neighbour. The
        //   two deliberately SHARE the folded-corner body and separate on the
        //   interior mark alone: a SOLID PLUS here, a DASHED placeholder frame
        //   there. Solid plus is "empty and yours to fill"; a dashed frame is
        //   "a layout is already here". Even out that one difference and the
        //   two adjacent controls become one control drawn twice.
        //
        // The shared body with `save` is likewise deliberate and likewise
        // separates on the interior mark — one label slot there, a crossed plus
        // here. All of this survives the refusal it was written under.
        //
        // **No enable predicate**, like `file.open` and for the same reason: an
        // operator with nothing open is exactly the operator most likely to
        // want this.
        command("file.new", t::file_new(), 103).with_icon("new-document"),
        // ★ The sized New, immediately after the plain one.
        //
        // Order matters here in the way §5.1's own table does: New, New from
        // template, Open, Recent, Close. The two ways to MAKE a document sit
        // together, then the two ways to get one back, then the way to put one
        // away.
        //
        // ★★ **The written refusal here is DISCHARGED as of 2026-09-04**, by
        // `new-from-template` — a glyph drawn for this role and adopted from
        // the outside review of 2026-09-03. The refusal was `file.new`'s and
        // not a new one (the icon directory is the operator's own art, so a
        // glyph was not a build session's to add), and it is spent the same way
        // that one is: the art was asked for and arrived, so nothing about the
        // provenance record changes.
        //
        // ★ **What survives is the pair rule, and it is the whole point of the
        // drawing.** The refusal's own words were that "the two New controls
        // sharing a glyph they do not have would be worse than the two of them
        // reading `New` and `New from template…`" — the hazard it named was
        // never *no glyph*, it was *one glyph on two controls*. That hazard is
        // still live, and `Icon::NewFromTemplate` answers it in the only way
        // that keeps both halves: the two SHARE the folded-corner page on
        // purpose, because a shared silhouette is how a ribbon says two
        // controls belong together, and they separate on the interior mark
        // alone — **a dashed placeholder rectangle here against `new-document`'s
        // solid plus.** The dash is the entire distinction and it is
        // load-bearing: solid plus means "empty, yours to fill", dashed frame
        // means "a layout is already here and you will fill it in". A later
        // pass that solidifies this frame for tidiness recreates exactly the
        // one-control-drawn-twice defect the refusal existed to prevent.
        //
        // Deliberately NOT a second sheet behind the page: that is `copy`'s and
        // `copy-page-text`'s vocabulary and would say this command duplicates
        // something already open, which is precisely what it does not do — a
        // template is on disk, not in the window.
        //
        // **No enable predicate**, for the strongest version of `file.new`'s:
        // an operator with nothing open is not somebody this is tolerated for,
        // they are the operator it exists for.
        command("file.new_from_template", t::file_new_from_template(), 104)
            .with_icon("new-from-template"),
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
        // ★★ **The icon refusal here is DISCHARGED as of 2026-09-04**, by
        // `recent` — a clock face drawn for this role and adopted from the
        // outside review of 2026-09-03.
        //
        // # The half that is discharged
        //
        // The refusal's operative clause was that the only glyph available to
        // borrow was the wrong one: *"`open` belongs to `file.open` and reusing
        // it would make two adjacent controls in one band look like one control
        // drawn twice."* That was an argument against a BORROW, not against a
        // picture, and a glyph of this command's own settles it.
        //
        // ★ **And the constraint it was protecting outlives it.** Open sits
        // immediately beside this, so `recent` must never drift toward
        // `open`'s folder — `Icon::Recent`'s own doc names that refusal as the
        // reason it is a clock rather than a folder-with-something-on-it. Two
        // further separations are claims rather than decoration:
        //
        // * **NOT `undo`**, the set's other time-flavoured glyph. Undo is an
        //   open ~270° arc with an arrowhead and no interior; this is a CLOSED
        //   ring carrying hands. The difference is the promise: an arrow says
        //   "go back and change what happened", and this command changes
        //   nothing — it only reopens.
        // * **NOT `info`**, the set's other closed ring, told apart by hands
        //   rather than a dot-and-stem inside.
        //
        // # ★ The half that is NOT discharged, and is deliberately left alone
        //
        // The refusal's second sentence — *"a command with no key renders as
        // text, which is a real answer … and the right one for a menu button
        // whose label is a word"* — was never falsified by art arriving, and it
        // is not being contradicted here. It is simply no longer the ONLY
        // answer available. Note what naming the key does and does not do
        // today: this command's ribbon control is not a `Button`, it is the
        // `recent_files` custom item, and `app::recent::menu` draws it — so the
        // key is declared here and is correct here (it is the one place the
        // icon vocabulary is bound to a command, and a keymap or a customized
        // quick-access toolbar reaching this command finds it), but what the
        // operator sees in File ▸ File is decided one module away.
        // Said plainly rather than implied, because a comment claiming a
        // pixel-level result this file cannot deliver is the drift this module
        // keeps recording.
        //
        // ★★★ **DISCHARGED 2026-09-05.** What stood here added *"…and
        // `app::recent::menu` draws it with `ui.menu_button(text.label, …)` …
        // what the operator sees in File ▸ File does not change until that
        // custom item is taught to paint it — which is a change in
        // `app::recent`, not here."* It is now taught: that function calls
        // `menu_image_text_button` with [`crate::icons::Icon::Recent`], and the
        // approved mockup's `['Recent','recent',{menu:1}]` — an icon AND a word
        // — is what File ▸ File draws.
        //
        // ⇒ ★★ The sentence was accurate, named the exact file, and sat
        // unactioned for a day. **A comment that names where the other half of
        // the work lives is the best available substitute for a mechanism and
        // is still not one** — the same finding `RESUME.md` records about
        // "read that Pass's note before anything else next session". What moved
        // it was `tools/compare-mockup-ribbon.py` learning to resolve BOTH
        // sides to the asset each control draws, at which point a control the
        // mock glyphs and the product does not became a printed difference
        // rather than a paragraph.
        command(FILE_RECENT, t::file_recent(), 102).with_icon("recent"),
        // ★★★ **Save**, 2026-08-20. See `text::commands::file_save` for the
        // argument that used to keep it out, and why that argument was aimed at
        // the wrong hazard.
        //
        // It takes the `save` glyph, and Save-a-copy no longer has to give
        // anything up for it to. Two adjacent controls sharing one key would
        // read as one control drawn twice - this module's own convention - and
        // of the two, the bare `save` body belongs to the one an operator
        // presses fifty times a day without reading the label.
        //
        // ★★ **The consequence of that — Save-a-copy going without — is
        // DISCHARGED as of 2026-09-04**, by `save-copy`, a glyph drawn for that
        // role and adopted from the outside review of 2026-09-03. What was
        // written here was *"Save-a-copy renders as text, which is a real answer
        // for a control whose whole meaning is in the words 'a copy'."* The
        // sentence was true and the arrangement it described was the right one
        // while `save` was the only save-family art in the set; it was a ruling
        // about which of two controls got the SHARED key, never a ruling that a
        // copy verb may not be drawn. Three bodies now exist, so nothing is
        // being taken from Save.
        //
        // ★ **The rule that survives is the one this block was really making,
        // and it is now the family's grammar rather than a tie-break.** All
        // three save controls carry the SAME body — that is what says "these
        // are the same family" — and they separate on ONE interior difference
        // each. Getting that difference evened out in a consistency pass is
        // precisely the "one control drawn twice" failure this note has always
        // been about, only three-ways. The differences, per the glyphs' own
        // docs:
        //
        // * `save` — a bare body, one label slot, no instrument over it.
        // * `save-as` — the body MARKED, by a small pencil laid across its
        //   lower-left field ("this file, under another name").
        // * `save-copy` — the body REPEATED, a second outline behind the full
        //   one ("another file").
        // * `save-compact` — the body carrying a contained downward arrow.
        //
        // ★ `save-copy` must additionally not be read as `copy`, which is also
        // two offset rects: that glyph is two BLANK rounded rects and means the
        // CLIPBOARD — take this and hold it. The retained SHUTTER and label
        // field are what keep this in the save family, and they are doing real
        // work; drop the shutter and the two collapse into each other.
        //
        // `doc.open` rather than a modified-document condition: Save is live
        // whenever there is a document, exactly as it is everywhere else. A
        // Save that greys itself when there is nothing to save is a Save the
        // operator has to think about.
        command("file.save", t::file_save(), 111)
            .with_icon("save")
            .enabled_when("doc.open"),
        command("file.save_copy", t::file_save_copy(), 110)
            .with_icon("save-copy")
            .enabled_when("doc.open"),
        // ★ **Save As**, `OPERATOR_REQUESTS.md` O95 — beside Save a copy and
        // not instead of it. The two are different acts (see `Action::SaveAs`),
        // and every editor the operator uses offers both.
        //
        // ★★ **The icon refusal here is DISCHARGED as of 2026-09-04**, by
        // `save-as` — a glyph drawn for this role and adopted from the outside
        // review of 2026-09-03. It was refused *"on the same reasoning
        // `file.new` and `file.ocr` record: the icon directory is declared the
        // operator's own art, every reuse here would mislead, and a control
        // that draws its own words is better than a false provenance note"* —
        // and it is spent the same way those are. The provenance clause was
        // never an argument that this control should be wordless forever, only
        // that a build session could not be the one to end it. It did not.
        //
        // ★ **The "every reuse would mislead" half is the durable half**, and
        // the two glyphs it was steering away from are the two this drawing has
        // to keep visibly apart from:
        //
        // * **NOT `save`.** That is the bare body with a single label slot and
        //   no instrument over it — the plain, fifty-times-a-day press, and the
        //   sibling this command must never be confused with. Save As is Save
        //   plus *you name it*, so the glyph is the save body plus the set's
        //   existing mark for authoring: a pencil across its lower-left field.
        // * **NOT `edit-text`.** That is a full-size standalone pencil meaning
        //   *edit the page's text*. Here the pencil is a small MODIFIER over a
        //   body that dominates the frame, and the size relationship is the
        //   cue — grow the pencil until it dominates and this reads as a
        //   page-editing tool on the File tab.
        //
        // And against `save-copy`, the third body in the group: that one
        // REPEATS the body, this one MARKS it — "another file" versus "this
        // file, renamed", which is exactly the distinction O95 was asking for.
        command("file.save_as", t::file_save_as(), 113)
            .with_icon("save-as")
            .enabled_when("doc.open"),
        // ★★ **The icon refusal here is DISCHARGED as of 2026-09-04**, by
        // `save-compact` — a glyph drawn for this role and adopted from the
        // outside review of 2026-09-03. The refusal for this one is not written
        // at this registration; it is recorded with the coverage count in
        // `super::super::tests`, and it read that *"its two neighbours in the
        // Save group carry icons, and a third disc beside them would be a
        // picture whose only job is to look like the other two — which is
        // exactly the confusion this command's NAME is built to prevent."*
        //
        // ★★★ **That is the strongest of this band's refusals and the one to
        // read before trusting the glyph**, because unlike `file.new`,
        // `file.new_from_template`, `file.save_as` and `file.ocr` it does not
        // rest on the provenance clause at all, and unlike `file.recent`'s and
        // Save-a-copy's it is not answered merely by the command having art of
        // its own. It did not say "no art exists"; it said a third member of a
        // lookalike family is worse than a word. The drawing answers it, but
        // it answers it by a MARK and not by a silhouette — the body is
        // deliberately the same body, because a shared body is how the ribbon
        // says these three do the same kind of thing. So the refusal is
        // discharged on the condition it named, and the condition is now a
        // standing constraint rather than a settled question: **this glyph
        // earns its place only while its interior mark is unmistakable.**
        // Normalise the interiors of the three save glyphs in a later
        // consistency pass and the exact defect the refusal predicted arrives
        // — three discs whose only job is to look like each other.
        //
        // ★ The mark, and the two separations that keep it honest:
        //
        // * A downward arrow FILLING THE FIELD BENEATH THE SHUTTER. Down is
        //   "smaller"; putting the arrow INSIDE the body is what keeps the
        //   glyph about the file rather than about a transfer.
        // * **NOT `export`** (`download.svg`) and **NOT `page-extract`**, the
        //   set's other downward arrows. Both of those point OUT — into a tray,
        //   or away from a page — and `download.svg`'s own note calls arrow
        //   direction "the family's grammar" for in/out of this document.
        //   **Nothing leaves here.** The save body ENCLOSES the arrow, and that
        //   enclosure is the whole difference between "smaller" and "outbound".
        //   This matters twice over on this tab, where both export verbs sit
        //   four registrations below wearing `export`.
        // * Distinct from `save-as` (a pencil) and `save-copy` (a second body)
        //   by carrying neither: all three are one body with one different
        //   thing said about it.
        //
        // ★ `doc.open`, like its two neighbours, and NOT gated on the document
        // having anything to reclaim. A file with nothing unused still gets a
        // copy — the window says so — because an operator who asked for one is
        // owed it, and because whether there is anything to reclaim is not
        // knowable without serialising the document, which a ribbon predicate
        // evaluated every frame must not do.
        command("file.save_compacted", t::file_save_compacted(), 112)
            .with_icon("save-compact")
            .enabled_when("doc.open"),
        // Both export verbs share `export`, and that is the header's
        // shared-key convention rather than an oversight: the glyph is the
        // download twin of `insert-pages`' upload art, reserved for exactly
        // this by the icon ui-spec §3.1, and what it says — "out of this
        // document, into a file" — is equally and completely true of both.
        // What differs is the format, which is a word only the label can say.
        command("file.export_dxf", t::file_export_dxf(), 120)
            .with_icon("export")
            .enabled_when("doc.pages"),
        // ★★★ **Export image — `OPERATOR_REQUESTS.md` O120, wired 2026-09-04.**
        //
        // Registered between the two existing export verbs and drawn in the
        // same band, because it is the same act: *content of this document,
        // written out to somewhere that is not this document.* What differs is
        // only what a reader of the result can do with it — CAD geometry there,
        // form values there, a picture here.
        //
        // ★★★ **IT WORE `export` FOR ABOUT SIX HOURS, AND THAT WAS WRONG.**
        // The argument is left standing below rather than deleted, because the
        // reversal is the useful part and a comment that quietly changed its
        // mind teaches nobody anything.
        //
        // What was argued, and it is not a bad argument: the glyph is the
        // download twin of `insert-pages`' upload art, reserved for exactly
        // this by the icon ui-spec §3.1; what it says — "out of this document,
        // into a file" — is equally true of all three export verbs; what
        // differs is the FORMAT, which is "a word only a label can say", and
        // all three labels say it. New art would then ask the operator to learn
        // a picture distinguishing three controls their labels already
        // distinguish, on a band where the three sit adjacent and are read
        // together. `file.save`, `file.save_as` and `file.save_compacted` a few
        // registrations above make the OPPOSITE choice — one body, three
        // interior marks — and can, because they are three spellings of one
        // verb where these are three different verbs.
        //
        // ★★ WHERE IT FAILS: the premise that the format is a word only a label
        // can say is **false for this one control**, and true for the other
        // two. A DXF is a coordinate list and a form-data file is a set of
        // name/value pairs — neither has a picture, so neither can be drawn,
        // so the tray is honest about them. **A raster image does have a
        // picture, and this set already draws it.** `image.svg` exists,
        // `edit.insert_image` wears it, and the operator has therefore already
        // learned in this application that a framed tile with a horizon means
        // *a raster image*. Handing the picture command a generic tray asks him
        // to unlearn that on exactly one control.
        //
        // ★ And it is the same defect the 2026-09-04 glyph batch was adopted to
        // fix, one level down. That batch's own note: four form-field tools
        // shared one asset and four measure tools shared another, "eight
        // controls rendering as two pictures", in a ribbon whose module header
        // says such controls are "distinguishable only by icon and tooltip".
        // Three export verbs rendering as one picture is that shape again, and
        // the shared-key convention is not a licence to re-commit it — the
        // convention permits a share where the art is EQUALLY true of both, and
        // `download.svg`'s emptiness is less true here than of its neighbours.
        //
        // ⇒ `export-image` — a picture tile with an arrow leaving it to the
        // right. `export_dxf` and `export_form_data` keep `export`, unchanged
        // and correctly: their share was never the problem. See
        // `icons/assets/export-image.svg` for which neighbours the new glyph
        // must stay apart from and by what cue — `image.svg` by the arrow's
        // DIRECTION (out, not in), `download.svg` by naming its cargo where
        // that draws an empty tray, `pick-link.svg` by leaving a closed frame
        // horizontally where that escapes a deliberate corner gap diagonally.
        //
        // ★ `doc.pages` rather than `doc.open`, matching `file.export_dxf` and
        // for the stronger version of its reason: every control in the window
        // is a statement about a page, and the largest-page measurement the
        // window opens with has nothing to fold over on a document with none.
        command("file.export_image", t::file_export_image(), 124)
            .with_icon("export-image")
            .enabled_when("doc.pages"),
        // ★★★ **Export text — wired 2026-09-04**, on the operator's ask:
        // *"also the engine can export PDFs as text. we should have
        // export/import for that."*
        //
        // Registered between the image export and the form-data pair, and drawn
        // in the same band, because it is the same act the three of them share:
        // *content of this document, written out to somewhere that is not this
        // document.* CAD geometry, a picture, the filled values — and now the
        // words.
        //
        // ★★ **It wears `export`, the shared tray, and that is the CORRECT side
        // of the line `file.export_image` had to cross.**
        //
        // That registration's own note draws the line and it is worth applying
        // rather than restating: the shared glyph is honest where the FORMAT
        // has no picture, and dishonest where it does. A DXF is a coordinate
        // list, a form-data file is a set of name/value pairs, and **a text
        // file is a run of characters** — none of the three has a picture, so
        // none of the three can be drawn, so the tray is telling the truth
        // about all three. `export-image` needed its own art because a raster
        // image *does* have a picture and this icon set already draws it, so a
        // generic tray there asked the operator to unlearn `image.svg`.
        //
        // ★ The near miss is `copy-page-text` / `copy-document-text`, two
        // registrations below: sheets with text RULES on them. Those are the
        // right art for *the words of this page, to the clipboard*, and sharing
        // one with this command would put three controls in one band reading as
        // two — and would lose the only cue separating a clipboard copy from a
        // file write, which is the whole difference between them.
        //
        // ★ `doc.pages` rather than `doc.open`, matching every export in this
        // band and for the copy verbs' version of the reason: text is drawn on
        // pages, and a legal `/Count 0` document has none to export from.
        command("file.export_text", t::file_export_text(), 125)
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
        // The `doc.pages` predicate and both tooltips come across unchanged:
        // nothing about what these commands DO has moved, only where an
        // operator finds them. `doc.pages` in particular is still the right
        // predicate rather than `doc.open` — text is drawn on pages, and a
        // legal `/Count 0` document has none to copy from.
        //
        // # ★★ The borrowed `copy` glyph, ENDED 2026-09-04
        //
        // The sentence above used to begin *"The `copy` icon, the `doc.pages`
        // predicate and both tooltips come across unchanged"*, and the icon
        // clause is the part that stopped being true. These two take
        // `copy-page-text` and `copy-document-text` as of 2026-09-04 — glyphs
        // drawn for these two roles and adopted from the outside review of
        // 2026-09-03.
        //
        // This was never a recorded refusal; it was a BORROW, and the borrow is
        // the failure this module's own convention names. `copy` was worn here
        // and by `edit.copy` and by `pages.copy` — **four controls drawn
        // identically**, which is precisely what "several controls reading as
        // one control drawn repeatedly" means, and two of the four sit adjacent
        // in this very group. The convention was being stated in this file and
        // then broken two registrations later.
        //
        // ★ **Three separations, and all three are load-bearing:**
        //
        // * **From `copy` itself** — `edit.copy`'s and `pages.copy`'s plain two
        //   blank offset rects — by TEXT RULES. Only these two copy WORDS; the
        //   others copy a selection and copy sheets. The rules are the claim.
        // * **From each other, by COUNT and by RULE LENGTH**, and this is the
        //   pair that matters most, because they are adjacent in one group and
        //   differ only in SCOPE: **two sheets with the front one fully ruled
        //   (three full-width rules) = one page's text; three cascading sheets
        //   with only the top one ruled (two short rules) = the whole file's.**
        //   Even out the sheet count or the rule lengths in a later consistency
        //   pass and the ribbon loses the ability to say page-versus-document
        //   at all — which is the ONLY thing separating these two commands.
        // * **`copy-document-text` from `pages` and from `layers`**, both also
        //   multi-sheet: from `pages` by the text rules, which that glyph
        //   deliberately omits because the Pages panel is about sheets as
        //   objects rather than about what is printed on them; from `layers` by
        //   being SQUARE-ON rather than isometric, per `pages.svg`'s standing
        //   rule that a layer is a plane you look through and a page is a thing
        //   you look at.
        command("file.copy_page_text", t::file_copy_page_text(), 122)
            .with_icon("copy-page-text")
            .enabled_when("doc.pages"),
        command("file.copy_document_text", t::file_copy_document_text(), 123)
            .with_icon("copy-document-text")
            .enabled_when("doc.pages"),
        // ★ Print had no icon because the salvage source drew it with the
        // *stamp* glyph, and that was a mis-assignment rather than a
        // convention to carry — `stamp` means "a mark applied with a stamp"
        // (icon ui-spec §3.4) and is shared with the reserved Bates glyph.
        //
        // Declining the wrong glyph was right; it was never a reason to have
        // none. `print` is the printer art the ui-spec §8.12 reserved, and it
        // collides with nothing.
        // ===================================================================
        // SECURITY — `OPERATOR_REQUESTS.md` O119, approved and wired
        // 2026-09-04: *"yes add encryption and permissions"*.
        // ===================================================================
        //
        // # Why a new group on **File** rather than a row in Edit ▸ Protect
        //
        // The mockup drew it there and the operator approved that mockup, and
        // the reason it drew it there survives restating: **every other command
        // on the Edit tab is an undoable edit to page content, and these two are
        // neither.** `EDITABLE_SURFACES.md` calls `set_encryption` *"a save
        // transform, not an undoable edit"* — pressing either rewrites every
        // byte and produces a different file, with nothing entered in the undo
        // log. Redaction, four groups away on Edit, removes marks from a page
        // and is undoable right up until it is applied. Putting the two on one
        // tab would say they are the same kind of verb.
        //
        // The position **within** File is the operator's own framing of the
        // question: O119 asks *"do you want to protect a drawing before you send
        // it out?"*, so the band sits immediately after the band that sends it
        // out. `crate::shell::manifest::file` places it there.
        //
        // # ★★★ Why both are gated on `doc.open` and NOT on "is it signed"
        //
        // Because whether **this** document is signed is not known when the
        // registry is built, and a predicate that had to answer it would be a
        // per-frame signature census on every ribbon draw. So the controls stay
        // present and the WINDOW refuses — it opens, names the count, explains
        // that encrypting rewrites every byte the signature covers, and offers
        // no form at all.
        //
        // That is R9 satisfied rather than dodged. The rule is *"the control is
        // absent or explained, never a button that fails on press"*, and this
        // is the explained branch: pressing it produces a sentence about the
        // operator's document, not a failure. The tooltips name the refusal too,
        // so an operator who only ever hovers still meets it.
        //
        // ★ `doc.open` rather than `doc.pages`: protection is a property of the
        // FILE, not of its pages. A legal `/Count 0` document can perfectly well
        // carry a password, and refusing to let one be set would be this shell
        // inventing a restriction the standard does not have.
        //
        // # Tokens 126 and 127
        //
        // New ids get new numbers in the `file.` block. ★ These were 125 and
        // 126 for about an hour, and 125 collided: a CONCURRENT track wired
        // `file.export_text` and took 125 in the same working tree. The
        // collision was caught by `every_handler_token_is_unique`, which is
        // precisely why that test exists and why this file's own header records
        // that a per-tab split was once refused on the grounds that a collision
        // between two blocks would be "invisible" — it is not, and this is the
        // day that got demonstrated rather than argued.
        //
        // ⇒ Moved to 126/127 rather than asking the other track to move, on the
        // rule that the later arrival yields. A token is what a trace prints;
        // reusing a retired one would make an old trace read as whatever
        // inherited its number, so 125 stays with `file.export_text`.
        command("file.encrypt", crate::text::protect::file_encrypt(), 126)
            .with_icon("encrypt")
            .enabled_when("doc.open"),
        command(
            "file.permissions",
            crate::text::protect::file_permissions(),
            127,
        )
        .with_icon("permissions")
        .enabled_when("doc.open"),
        command("file.print", t::file_print(), 130)
            .with_icon("print")
            .enabled_when("doc.open"),
        command("file.properties", t::file_properties(), 140)
            .with_icon("properties")
            .enabled_when("doc.open"),
        // ★★★ **Document properties** — the operator, 2026-09-05: *"the
        // document properties are still always visible in the properties tab.
        // it needs to get out of there and be in its own document properties
        // tab."*
        //
        // # Token 142, and it is a new number rather than a reused one
        //
        // 140 and 141 are the two commands it sits between; 142 was free. A
        // token is what a trace prints, so a retired one is never recycled —
        // `file.export_text`'s note four registrations up records the day that
        // rule was demonstrated rather than argued.
        //
        // # ★★★ The icon was SHARED with `file.properties` and is now `document`
        // — CORRECTED 2026-09-05 (later the same day), and the correction is
        // the interesting half
        //
        // What stood here said the shared key was *"the convention rather than
        // an economy"*, and closed with: *"`icons/assets/PROVENANCE.md` makes
        // that directory his own art, so the alternative is not 'draw one' but
        // 'ask him for one'."*
        //
        // **Both clauses were checkable and both were false.**
        //
        // 1. **The art already exists and is an ORPHAN.** `document.svg` ships,
        //    [`crate::icons::Icon::Document`] carries it, and that variant's own
        //    doc says in as many words that *"no command names this key today,
        //    and the variant exists anyway"* — it is kept so
        //    `every_icon_parses` and its three siblings keep walking the art.
        //    So there was never anything to draw and nobody to ask; there was a
        //    drawing with no button, one directory away.
        // 2. **The shared-key convention does not reach this pair.**
        //    `catalog/edit.rs` states the rule exactly: a shared key is the
        //    convention *"where two controls have the same SUBJECT and are
        //    separated by something else the operator can see"* — `edit.paste`
        //    beside `edit.paste_duplicate`, the three page-clipboard verbs.
        //    Those pairs share a subject and differ in a verb. **These two do
        //    not share a subject at all**: `file.properties` describes
        //    *whatever is selected on the page*, and this describes *the file*.
        //    Applying the convention here erased the one distinction the
        //    picture had to carry, which is the identical fault that same
        //    module records against the five form-field controls that all drew
        //    `form-field`.
        // 3. **And the mockup — the approved design — draws `document` here.**
        //    `mockups/pdfcer-shell-template.html`'s File ▸ Document band is
        //    `['Document','document'], ['Properties','properties'],
        //    ['Fonts','fonts']`. `tools/compare-mockup-ribbon.py` reported the
        //    pair as `properties properties fonts` against the mock's
        //    `document properties fonts` on the first run of its item phase.
        //
        // The two glyphs separate the way [`crate::icons::Icon::Document`]'s
        // own doc says they do: `properties` is three slider rules, because it
        // is about the VALUES of what is selected; `document` is the page
        // itself, because this command's subject *is* the page.
        //
        // # `doc.open`, like both its neighbours
        //
        // There is no document metadata without a document. R9's other branch
        // does not apply: this is *temporarily* unavailable in the plainest
        // sense — open a file and it works — which is exactly what greying is
        // reserved for.
        command(
            "file.document_properties",
            t::file_document_properties(),
            142,
        )
        .with_icon("document")
        .enabled_when("doc.open"),
        command("file.fonts", t::file_fonts(), 141)
            .with_icon("fonts")
            .enabled_when("doc.open"),
        // Settings, the shortcut list and About are always available: they
        // are about pdfcer, not about a document.
        // ★★★ **Open in Acrobat** — `OPERATOR_REQUESTS.md` **O122**, the
        // operator, 2026-09-04: *"beside our read-review-edit buttons at the
        // top there should be an open in acrobat button."*
        //
        // ## Where it appears, and why that is not a tab
        //
        // It is the sole member of the manifest's TRAILING region — the far
        // right of the tab-strip row, past the mode selector — which is where
        // he asked for it and which had no way to hold anything until this
        // work. `egui_shell::manifest::Trailing` carries the argument for the
        // region existing; the argument for this command being in it is his
        // sentence and nothing more, and that is sufficient.
        //
        // ## ★★★ Registered UNCONDITIONALLY, absent CONDITIONALLY
        //
        // The obvious spelling is to register this only on a machine that has
        // an Acrobat, so R8's rule holds by construction: *a capability's
        // presence is expressed by registering its command, and by nothing
        // else.* It is wrong here, for a reason specific to this capability
        // rather than a general objection.
        //
        // The registry is built ONCE, at start-up. The path to Acrobat is a
        // SETTING the operator can change at any moment, and O122's whole
        // escape hatch is that the setting is reachable *while the button is
        // absent* — somebody with a non-standard install types a path and the
        // control has to appear. A conditional registration cannot do that
        // without a restart, and a restart to make a button appear is the
        // failure the escape hatch exists to prevent.
        //
        // So the ITEM carries `visible_when: "acrobat.available"`, which is
        // `egui_shell`'s own R9 mechanism — its `Item::Command::visible_when`
        // doc states it in those words: *"an unavailable capability renders
        // nothing; greying is reserved for TEMPORARILY unavailable"* — and is
        // re-evaluated every frame. See `crate::shell::manifest`'s trailing
        // block, and `PdfcerApp::conditions`, which sets the name from the one
        // resolved viewer.
        //
        // ## ★★★ The icon refusal was ARGUED and then DISCHARGED, within hours
        //
        // This registration was written with no icon, and the argument was the
        // one `file.save_as`, `edit.select_all` and `edit.attachments` all
        // reached: every reuse available would MISLEAD — `export` says "out of
        // this document into a file", which this does not do, and `open` says
        // "bring a file in here", which is its opposite — and drawing a new
        // glyph is not a build session's to do, because
        // `icons/assets/PROVENANCE.md` declares that directory the operator's
        // own art.
        //
        // ★ The second clause is what got spent, and it got spent the same
        // afternoon: `open-in-acrobat.svg` was drawn on the icon track for
        // this command, before this command existed to name it. So the glyph
        // is purpose-drawn rather than borrowed, `PROVENANCE.md` is untouched,
        // and the refusal is DISCHARGED rather than reversed — the same word
        // `file.ocr`'s was, and for the same reason: the argument was never
        // wrong, its supply premise simply stopped holding.
        //
        // ⚠ The asset's own note carries the constraint that made it drawable
        // at all: the LABEL names a vendor, and the art carries nothing of
        // that vendor's mark.
        //
        // ## `doc.open`, not `doc.pages`
        //
        // Acrobat is handed a FILE, and a file with no pages pdfcer could
        // rasterize is still a file Acrobat may well open — it is a more
        // capable reader of damaged documents than this shell is. Greying it
        // on `doc.pages` would refuse the case where handing the document over
        // is the most useful thing pdfcer could do.
        command(
            "file.open_in_acrobat",
            crate::text::acrobat::file_open_in_acrobat(),
            161,
        )
        .with_icon("open-in-acrobat")
        .enabled_when("doc.open"),
        command("file.settings", t::file_settings(), 150).with_icon("settings"),
        command("file.shortcuts", t::file_shortcuts(), 151).with_icon("keyboard"),
        // ★ `file.about` carries an OBLIGATION, not a courtesy: it is the
        // in-application half of the attribution surface that shipping
        // CC-BY-SA-4.0 OCR model weights requires, BY needing the notice to
        // reach the RECIPIENT rather than a reader of the repository. The
        // argument is in `crate::text::about`; the gate that keeps both
        // halves true is `tools/gates/check-shipped-assets.py`.
        command("file.about", t::file_about(), 152).with_icon("info"),
        // ★★ **`file.ocr`'s icon refusal is DISCHARGED as of 2026-09-04**, by
        // `recognise-text` — a glyph drawn for this role and adopted from the
        // outside review of 2026-09-03.
        //
        // The refusal's full argument is the `file.ocr` row of this module's
        // header table, and it was the one on that table with a reason of its
        // own: in one line, the icon directory is declared the operator's OWN
        // ART, so a new glyph was not a build session's to add, and every
        // available reuse would tell an operator the button does something it
        // does not. The first clause is spent — the art was asked for and came
        // back from outside — and `icons/assets/PROVENANCE.md` is untouched,
        // which is the property that clause existed to protect.
        //
        // ★ **The second clause is not spent; it is the drawing's brief.** The
        // reuses it rejected by name are the neighbours this glyph has to stay
        // visibly apart from, and two of the three are on surfaces an operator
        // reaches in the same minute:
        //
        // * **NOT `fonts`.** This is the close one, because both are a capital
        //   A on a rule. Two cues separate them and BOTH are required: this A
        //   sits INSIDE a page outline, and its rule is DASHED where the Fonts
        //   rule is solid. `fonts.svg`'s own note explains why its baseline is
        //   solid — a solid rule turns "a letter" into "a typeface" — and the
        //   Fonts panel only REPORTS. This command writes a text layer into the
        //   file, so it may not wear the glyph of a surface that changes
        //   nothing. Solidify this rule for tidiness and the ribbon starts
        //   promising a read-only panel where it means an irreversible write.
        // * **NOT `search`'s magnifier over a page** — that says Find, which is
        //   a different command that also exists.
        // * **NOT `text-select`** (the text TOOL) and **NOT `set-scale`**
        //   (`convert.svg`, a format change): each names a capability this
        //   command does not have.
        //
        // The dashed sweep across the page's foot is the positive claim, not
        // just the separator: recognition is a SCAN, and a dashed line is the
        // one cue that reads as "in progress, not yet certain" at 16 px.
        //
        // `doc.pages` rather than `doc.open`: recognition needs a page to
        // rasterize, and a document with none would open a dialog whose only
        // possible outcome is a refusal.
        //
        // ★ On the FILE tab, where `RIBBON_IA.md` §5.7 says Tools. Read's tab
        // list is `["file", "view"]`, so Tools would put OCR out of reach in
        // the one mode the operator asked for it in. Argued in full in
        // `super::manifest::tools`'s header.
        command("file.ocr", t::file_ocr(), 160)
            .with_icon("recognise-text")
            .enabled_when("doc.pages"),
    ]
}
