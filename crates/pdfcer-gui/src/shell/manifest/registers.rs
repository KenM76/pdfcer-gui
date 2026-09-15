//! # `shell::manifest::registers` — the two lists of commands this manifest
//! does NOT emit, and the one list of commands it emits anyway
//!
//! ## The seam, and why it is the right one
//!
//! [`super`] answers *"what is on the ribbon?"* — it **builds** a
//! `egui_shell::manifest::Shell` out of eight tab modules, applies the
//! collapse ladder, and exports the handful of constants a surface outside the
//! manifest needs in order to agree with it. This file answers a different
//! question — *"what is NOT on the ribbon, and why?"* — and it answers it in
//! **data** rather than in code: two `&[(&str, &str)]` tables that no function
//! in this crate branches on.
//!
//! The two answer to different edits, which is the test a seam has to pass. A
//! tab gains a group, an item moves, a collapse priority is re-ranked:
//! [`super`]. A command is built and its deferral note has to come out, or a
//! new gap is discovered and written down: here.
//!
//! ## Nothing here is enforced from here
//!
//! Both tables are asserted by tests that live in [`super`], and they stay
//! there deliberately. `planned_commands_are_genuinely_absent` asks a question
//! about the **manifest** — *does this id appear on any tab?* — and it needs
//! `built_in()` to ask it. A test that moved with its data would either drag
//! the builder over here or re-derive it, and re-deriving the emitted set is
//! how the two copies come to disagree.
//!
//! So: the data moved, the checks did not, and the checks read `registers::`
//! paths. That is the shape every one of this crate's data-and-rule splits
//! has taken.

// ===========================================================================
// PLANNED
// ===========================================================================

/// **Every command `RIBBON_IA.md` specifies that this manifest does not
/// emit, and why.**
///
/// `(id, reason)`. The reason is the entry's whole value: it is what lets
/// a later stage tell a **C** row — engine written and tested, shell
/// missing, a day's work — from an **N** row that is a month, without
/// re-deriving the analysis from the specification each time.
///
/// # Why this exists rather than a comment
///
/// P3 says an unavailable capability renders nothing. Applied literally
/// and alone, that turns `RIBBON_IA.md`'s specification into a much smaller
/// manifest with no record of the difference, and the next person to read
/// this module cannot tell a command that was *considered and deferred*
/// from one that was *never noticed*. Those are very different facts and
/// only one of them is a plan. The sizes of the two sets are not restated
/// here: the tables are the only copy, and a count in prose beside a table
/// is a second copy that drifts.
///
/// So the omissions are data:
///
/// - **tested**, in both directions — `planned_commands_are_genuinely_absent`
///   asserts nothing here is referenced by the manifest *and* nothing here
///   is registered, so an entry that gets built and not removed fails the
///   suite rather than becoming a stale comment;
/// - **enumerable**, so a diagnostic surface or a roadmap tool can list
///   the gap;
/// - **greppable by id**, so the search that finds `measure.two_line` in
///   the manifest also finds the note saying where it went.
///
/// # Ordering
///
/// By tab, in the tab order of [`built_in`], then in the order
/// `RIBBON_IA.md` §5 lists them within their group. Not sorted
/// alphabetically: this list is read against the specification, and a
/// reader checking §5.3 against it wants the Pages entries together and in
/// the document's order.
pub const PLANNED: &[(&str, &str)] = &[
    // -- The two splits, and why they are PLANNED and not SCAFFOLDED -------
    //
    // `OPERATOR_REQUESTS.md` O68. Ken: *"the Merge files and Split files
    // buttons don't do anything."*
    //
    // ⚠ **A command whose blocker is a missing CAPABILITY belongs here; one
    // whose blocker is missing wiring belongs on the `SCAFFOLDED` allow-list.**
    // `SCAFFOLDED` means *"registered, drawn, and does nothing"* — it forces an
    // explanation and never forces a fix. `PLANNED` means *"named in the IA and
    // not built"*, and R9 says a capability that is not built renders nothing,
    // so a `PLANNED` command is unregistered and draws no control at all.
    // Putting a capability gap on the wrong list is how an operator comes to
    // press an inert button. `no_scaffolded_command_is_also_planned` keeps the
    // two lists disjoint, which is what makes the distinction mean something.
    (
        "pages.split",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — the boundary chooser does not exist. `pageops::plan_split` takes a plan \
         (every N pages, at bookmarks, at an explicit list) plus a destination directory \
         and a name template, and there is no honest default: splitting a 36-sheet \
         drawing set into 36 files because nobody was asked is not a lesser version of \
         the feature. The engine half is COMPLETE and was built for this dialog — \
         `plan_split` is separate from `split` precisely so a UI can preview the parts \
         before anything is written.",
    ),
    (
        "tools.split_files",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — the same dialog as `pages.split` with a different operand set: one or more \
         files chosen on disk rather than the open document. It comes back when that \
         dialog does, and not before, because half a chooser is a control that splits \
         somebody's drawing set the way pdfcer guessed.",
    ),
    // -- File -- `RIBBON_IA.md` §5.1 ----------------------------------------
    //
    // ⚠ **A reason in this table is a measurement, and the question it measured
    // is part of it.** Three standing rules, each of which a reason on this tab
    // has broken while reading as accurate:
    //
    // 1. **A blocker aimed at the wrong hazard blocks until somebody re-asks
    //    the question.** In-place save is safe because pdfcer writes an
    //    INCREMENTAL UPDATE — the previous revision stays in the file, so the
    //    format is its own crash recovery. What is genuinely unsafe is that
    //    `fs::write` truncates before it streams, and the answer to that is
    //    three lines: a temporary beside the target, then a rename.
    // 2. **A capability note measures what the ENGINE lacks and is silent on
    //    what the operator's documents will do.** A scanned drawing extracts
    //    text successfully and returns nothing, so a save-dialog-only text
    //    export would write a zero-byte `.txt` indistinguishable from a
    //    successful export of a blank page. `app::actions::export::text`
    //    refuses before the picker opens and names `Recognise text`.
    // 3. **An entry here is REMOVED when the command ships, never annotated.**
    //    `planned_commands_are_genuinely_absent` asserts in both directions —
    //    nothing listed here may be referenced by the manifest, and nothing
    //    listed here may be registered — so a row for a shipped command fails
    //    the suite rather than decaying into a stale sentence. This is the one
    //    property the register was built for and it holds for every tab below.
    //
    // ⚠ **There is no document-creation path in the engine and there must not
    // be one.** `pdfcer-core`'s `document` module states *"no separate
    // builder/generation model may ever be introduced"* as a named invariant.
    // That is why File ▸ New opens a bundled blank template and sizes it with
    // `set_media_box` rather than asking the engine to grow a creation path —
    // and why one asset suffices where an asset-per-offered-size plan could
    // never have answered a custom size at any number of assets. See
    // `crate::app::blank`, whose §3a carries the full record.
    //
    // ⚠ **Text export and `file.copy_document_text` are one answer reaching two
    // destinations.** At its defaults the export writes byte-for-byte the
    // string the clipboard verb puts on the clipboard, which is a property no
    // amount of careful labelling could have bought. The IMPORT half of that
    // pair is deliberately not a row here: this list is for surfaces this shell
    // has not built, and an import is a verb the engine does not have. See
    // `app::actions::exporttext`'s header and `ENGINE_BACKLOG.md`.
    //
    (
        "file.revert",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — meaningless until there is a save point to revert to, so it follows `file.save`.",
    ),
    (
        "file.imposition",
        "C — n-up, booklet and poster imposition exist in core and the CLI. Needs a \
         print-time dialog.",
    ),
    (
        "file.security",
        "N — no encryption or permissions surface. Encryption is disclosed in the status bar \
         today, and opening a signed or encrypted document into Read mode is the nearer fix.",
    ),
    // ⚠ **An about box is a licence obligation on this build, not a
    // courtesy.** For as long as a distribution carries only permissively
    // licensed code, the shipped `LICENSE` covers its notices. This one ships
    // CC-BY-SA-4.0 OCR model weights, and BY requires the notice to reach the
    // RECIPIENT of the work — which nothing but a surface inside the running
    // program does. See `crate::text::about`.
    //
    // -- View -- `RIBBON_IA.md` §5.2 ----------------------------------------
    //
    // ⚠ **A reason that describes a SHELL this build does not have is stale
    // rather than early, and it hides a real gap.** Every panel here is an
    // ordinary dock panel — Pages no less than Bookmarks and Layers — so every
    // panel needs an ordinary panel toggle; a note explaining that one of them
    // is reached from a rail instead describes a shell that was replaced.
    // `every_panel_is_reachable_from_the_ribbon` is the test that makes such a
    // note visible, and the failure it catches is severe: a panel can exist, be
    // filtered out of every mode by `SHELL_FRAMEWORK.md` §5b's capability rule,
    // and be openable by nobody.
    //
    // ⚠ **Read mode is shown `file` and `view` alone**, and the operator fills
    // forms in Read — so `view.panel_forms` is the Forms panel's own toggle and
    // there is no `edit.form_fill`.
    //
    // Two headers carry what the absent view rows pointed at:
    // `crate::canvas::rulers` §1 answers *"in the document's units"*, and
    // `crate::canvas::guides` records why `guides.txt` is a fourth store beside
    // `layout.ron`, `recent.txt` and `page-display.txt` rather than a field in
    // any of them.
    (
        "view.rotate_view_left",
        "N — rotates the VIEW without changing the document, which is a different command \
         from `pages.rotate_left` and is the one a reader wants.",
    ),
    (
        "view.rotate_view_right",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `view.rotate_view_left`, clockwise.",
    ),
    (
        "view.save_workspace",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — named workspaces are a superset of layout persistence, which lands at stage S3.",
    ),
    (
        "view.load_workspace",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `view.save_workspace`.",
    ),
    // -- Pages -- `RIBBON_IA.md` §5.3 ---------------------------------------
    (
        "pages.insert_blank",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "C — pdfcer-core inserts blank pages already. Needs a size-and-count dialog only.",
    ),
    (
        "pages.insert_scan",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — there is no scanner acquisition path of any kind.",
    ),
    (
        "pages.replace",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — replace the selected pages with pages from another file.",
    ),
    (
        "pages.crop",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — needs an interactive crop-box gesture and a /CropBox writer.",
    ),
    // ⚠ **Rescale and re-media-box are two commands, not one command with two
    // spellings.** Re-media-boxing changes the paper and leaves the drawing
    // exactly where it is; rescaling moves every mark on the page, and would
    // have to rescale every ce dimension group's calibration with them or start
    // printing wrong measurements. Pages ▸ Transform ships the second of those.
    // `crate::app::actions::pagesize`'s header carries the measurement and the
    // argument for not building the first quietly alongside it.
    //
    // `pages.crop` above stays **N** and is a genuinely different command
    // again: `/CropBox` is the visible region *within* the paper, it needs an
    // interactive gesture, and `set_media_boxes` does not write it. The
    // sheet-size window discloses a crop box the new paper no longer contains
    // and deliberately does not repair one.
    (
        "pages.watermark",
        "N — the whole Pages ▸ Stamp group is unbuilt, so the GROUP is absent too rather \
         than present and empty.",
    ),
    (
        "pages.header_footer",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `pages.watermark`, in the same absent group.",
    ),
    (
        "pages.bates",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — Bates numbering, in the same absent group. See DEFECTS.md §2.",
    ),
    // -- Edit -- `RIBBON_IA.md` §5.4 ----------------------------------------
    (
        "edit.insert_shape",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — real page shapes, as distinct from the markup shapes on the Markup tab.",
    ),
    (
        "edit.align",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — the whole Edit ▸ Arrange group is unbuilt, so the GROUP is absent too.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("edit.distribute", "N — as `edit.align`, same absent group."),
    (
        "edit.bring_forward",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — needs a content-stream reordering primitive that does not exist.",
    ),
    (
        "edit.send_backward",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `edit.bring_forward`, in the other direction.",
    ),
    (
        "edit.group",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — object grouping has no representation in the object model yet.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("edit.ungroup", "N — as `edit.group`."),
    (
        "edit.flip_horizontal",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `edit.align`, same absent group.",
    ),
    (
        "edit.flip_vertical",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `edit.align`, same absent group.",
    ),
    // ⚠ **Page CONTENT cannot be pasted; a MARKUP can.** The object clipboard
    // is narrower than "there is no object clipboard" suggests, and the line
    // falls between the two: 157 verbs in `pdfcer-core`'s `edit` module and
    // none inserts page content, while a markup round-trips through
    // `annot_author::spec_from_dict` out and `add_markup` back.
    // `canvas::clipboard`'s header carries the table.
    (
        "edit.paste_in_place",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — and deliberately so, rather than pending. `edit.paste` offsets a same-page paste \
         so the copy is visible and lands a CROSS-page paste in place already, so this command \
         would differ from Ctrl+V only on the one page where an operator would never want it. \
         The fourth clipboard command is a control with no distinct behaviour to offer.",
    ),
    (
        "edit.sanitise",
        "N — strip metadata, scripts and hidden content. Distinct from redaction, which \
         removes what a mark covers.",
    ),
    // -- Markup -- `RIBBON_IA.md` §5.5 --------------------------------------
    (
        "markup.line",
        "N — the shipped build has four markup kinds and none is a plain line; the existing \
         `Arrow line` is the arrow. See markup.rs.",
    ),
    // ⚠ **A reason that names an EXTERNAL blocker is a claim about a
    // repository this project does not control, and it decays silently.**
    // There is no gate for it, and inventing one would be inventing a
    // dependency on `pdfcer-core`'s internals. What there is instead is this
    // paragraph, in the place the next reader looks: a row of that shape is
    // re-measured against the engine when it is read, never trusted.
    //
    // ⇒ The distinction row after row here has turned on: **a missing GESTURE
    // is ours and a missing SPEC is the engine's**, and only the second is an
    // external blocker at all. `MarkupSpec`'s `PolyLine`, `Polygon`, `Ink` and
    // `Cloud` are all in `pdfcer-core` — `Cloud` with vertices, border,
    // interior, width and intensity, and `EditError::TooFewVertices` beside it
    // — so what kept those tools absent was the multi-click and freehand
    // gestures, now `canvas::markup::vertex` and `canvas::markup::ink`.
    // `markup.finish` was never a row here, because the problem it solves did
    // not exist until those gestures did. `manifest::markup`'s header describes
    // the bands that carry them.
    (
        "markup.callout",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a note with a leader line to what it refers to.",
    ),
    (
        "markup.line_width",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — Style sets the NEXT markup's properties and only colour has a control today.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("markup.fill", "N — as `markup.line_width`."),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("markup.opacity", "N — as `markup.line_width`."),
    (
        "markup.clear_page",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — remove every markup on this page in one action.",
    ),
    (
        "markup.clear_all",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — remove every markup in the document in one action.",
    ),
    // -- Measure -- `RIBBON_IA.md` §5.6 -------------------------------------
    (
        "measure.aligned",
        "partial G — the constraint exists inside the linear tool, but there is no separate \
         tool to arm, and a button that arms nothing is the placeholder P3 forbids.",
    ),
    (
        "measure.angular",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — angular dimensions. One of the two conspicuous absences for takeoff work.",
    ),
    // ⚠ **"Nothing calls it" is a claim about THIS shell, not about the
    // engine.** `pick_line_in_page` and `TwoLinePick` are `pdfcer-core`'s, and
    // `crate::canvas::measure` is the caller. A reason that reads as an engine
    // gap when the gap is our own caller sends the next reader to the wrong
    // repository — and this one was contradicted by five documents at once
    // before anybody noticed.
    (
        "measure.calibrate",
        "partial G — calibrate from a known length. The least certain judgement in this \
         list: it may already be reachable through the scale entry, in which case this \
         moves into Measure ▸ Scale. See measure.rs.",
    ),
    (
        "measure.distance",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — the whole Measure ▸ Quantity group is unbuilt, so the GROUP is absent too.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    (
        "measure.area",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `measure.distance`, and the other conspicuous absence for takeoff work.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("measure.count", "N — as `measure.distance`."),
    (
        "measure.takeoff_schedule",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — the whole Measure ▸ Takeoff group is unbuilt, so the GROUP is absent too.",
    ),
    (
        "measure.takeoff_export",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `measure.takeoff_schedule`; a schedule to export must exist first.",
    ),
    // -- Tools -- `RIBBON_IA.md` §5.7 ---------------------------------------
    (
        "tools.batch_print",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — printing a set of files unattended, with one print setup.",
    ),
    (
        "tools.compare",
        "N — document comparison. A large build, and an OPEN QUESTION in RIBBON_IA.md §8 \
         rather than a scheduled item: it is the one absence an AEC reviewer names first.",
    ),
    // ⚠ **OCR is `file.ocr`, in File ▸ Recognise rather than Tools ▸
    // Recognise** — `super::tools`' header gives the reason.
    //
    // ⚠ **Its blocker was never the engine decision.** `ocrs` is the OCR
    // engine, chosen for being the only surveyed candidate that passes pdfcer's
    // wasm32 gate, and the whole recognition path is `pdfcer-core`'s. What had
    // to be answered first was **redistributing CC-BY-SA-4.0 model weights from
    // an MIT repository**, which is a licensing question and not a GUI one at
    // all — which is why the credit mechanism is built to fail loudly:
    // `about.hbs`, `crates/pdfcer-gui/src/text/about.rs` and
    // `tools/package-portable.py` fail together if any one of them forgets.
    //
    // ⇒ **A one-line reason is a claim, and a claim that names the wrong
    // blocker sends the next reader to solve the wrong problem.**
    (
        "tools.pdfa_validate",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — PDF/A validation and conversion. See DEFECTS.md §2.",
    ),
    (
        "tools.optimise",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — resample, subset and recompress to shrink a file.",
    ),
    // -- Format (contextual) -- `RIBBON_IA.md` §5.8 -------------------------
    //
    // Every row from this section is one decision: **build order, panel first
    // and tab second.** The panel is the harder half and the tab's contents are
    // a subset of it, so building the tab first would mean writing the property
    // editors twice. Where a Format group has shipped, that order was followed
    // and the editors were written once, which is the whole argument for it.
    //
    // ⚠ **A reason of the form "the engine has no verb for this" has an
    // hours-long shelf life on this project, and a register whose rows are
    // prose cannot know when one closes.** What keeps such a row honest is that
    // `planned_commands_are_genuinely_absent` asserts in BOTH directions, so
    // registering the command fails the suite by name rather than leaving a
    // false sentence sitting in a table nobody re-reads. **A blocker that is a
    // test is a blocker that expires.** There is no row here for a markup's
    // dash pattern for exactly that reason: `MarkupStyle::dash` exists, with
    // `MarkupOptions::dash` beside it for authoring.
    //
    // ⚠ **`format.note_text` below is a different kind of absence and does not
    // expire.** It is not deferred by build order either, but its reason is
    // **surface** rather than capability: a note's `/Contents` is
    // `MarkupNote`'s and the control is prose, which a ribbon band is the wrong
    // shape for. An engine shipping something does not discharge it.
    (
        "format.note_text",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — the text of a placed note; panel first.",
    ),
    (
        "format.dimension_group",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — which group a placed dimension belongs to; panel first.",
    ),
    (
        "format.dimension_scale",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a placed dimension's scale, as distinct from the current group's; panel first.",
    ),
    (
        "format.precision",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a placed dimension's number format; panel first.",
    ),
    (
        "format.units",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a placed dimension's units; panel first.",
    ),
    (
        "format.standard",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a placed dimension's drafting standard; panel first.",
    ),
    (
        "format.witness_lines",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a placed dimension's witness lines; panel first.",
    ),
    (
        "format.size",
        "N — a selected image's size. The panel carries the typed W/H, which is the surface \
         that makes /Rect resize reachable without a drag.",
    ),
    (
        "format.position",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a selected object's position, typed rather than dragged; panel first.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("format.crop", "N — cropping a placed image; panel first."),
    (
        "format.replace_image",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — swap the image behind a placed image object; panel first.",
    ),
    (
        "format.stroke",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a vector object's stroke; panel first.",
    ),
    (
        "format.winding_rule",
        "N — a vector object's winding rule. A read-only fact more often than an edit, which \
         is precisely why it belongs in the panel rather than the tab.",
    ),
    (
        "format.node_tools",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — add, remove and convert a vector object's nodes; panel first.",
    ),
    (
        "format.spacing",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a text run's character and line spacing. Not a scheduling gap and no longer \
         \"panel first\": `EditSession` has no verb for it. `format_text` sets face, size, \
         weight and fill; there is nothing that writes `Tc`, `Tw` or `TL` for an existing run, \
         so neither the panel nor the tab can carry it.",
    ),
    (
        "format.alignment",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a text run's alignment. Same blocker as `format.spacing`, and a harder one: \
         alignment is not a property a PDF text run HAS. It is a consequence of where each \
         show operator was positioned, so re-aligning existing text means re-laying it out, \
         which is `add_text`'s job on new content and nothing's job on old.",
    ),
    // -- Not from `RIBBON_IA.md` §5: commanded by a context menu ------------
    //
    // One entry, and it is here rather than in a register of its own because
    // the question it answers is the same one every row above answers —
    // *"why is this not on the surface that obviously wants it?"* — and a
    // second list would be a second place to look.
    //
    // `super::menus`' `dock.tab` menu is where a Close belongs, and §6 does
    // not specify one because §6 is about the ribbon. The dock closes a tab
    // today through its own hard-coded button and its own internal intent,
    // which is a dock mechanism rather than a pdfcer command.
    (
        "dock.close_panel",
        "N — closing one panel is an `egui_shell::dock` INTENT, not a command: the dock \
         draws its own tabs, owns their secondary click, and exposes no seam for an \
         application menu. Registering an id with no way to reach the dock's intent from \
         `dispatch_token` would be a command that cannot work. See `shell::menus`' header \
         for what would close the gap.",
    ),
];

// ===========================================================================
// DIRECTED
// ===========================================================================

/// **Commands emitted despite not carrying a `G` mark, and the instruction
/// that put them there.**
///
/// `(id, why)`. It exists as a list rather than as prose because otherwise
/// this manifest would look like it applied P3 everywhere except in one
/// place, for no stated reason.
///
/// ⚠ **The bar is a command the specification describes in enough detail to
/// be a decision rather than a wish**, even though no status mark says so.
/// A row that names a verb in every selection type it applies to has
/// specified something; a row that names a capability has not.
///
/// The tension this register holds, stated rather than smoothed over: P3
/// exists so an operator is never shown a control that does nothing, and an
/// entry here is a control P3 would otherwise have suppressed. It earns its
/// place by being **real** — `format.delete` is wired through
/// `PdfcerApp::dispatch_token` to `SelectionState::deletable_objects_on`,
/// the same rule the Delete key reads — not by being specified.
///
/// ⇒ **A settings knob does not belong here.** A knob whose value is a
/// compiled-in constant reads as `partial G` and is tempting, but a
/// preference surface is where a setting is discovered and changed; this
/// register is for commands. `app::prefs` and `dialogs::settings::display`
/// are where such knobs live, and `shell::commands::reach` records what
/// moving them cost. Keeping the list short is what makes deleting a row
/// from it cheaper than re-deriving which entries were deliberate.
pub const DIRECTED: &[(&str, &str)] = &[(
    "format.delete",
    "Not status-marked: RIBBON_IA.md §5.8 lists Delete in every selection type's row \
         without a mark. Modeless select-and-delete works today — it is what the removal of \
         the `Editing on` toggle relies on — so the command is real.",
)];

/// **The commands whose only surface is a per-panel menu**, with the reason
/// a ribbon control could not be one.
///
/// # Why this register exists, and what it must NOT become
///
/// Two tests state one rule from two sides:
/// `shell::tests::no_registered_command_is_orphaned` and
/// `shell::menus::tests::every_menu_command_is_also_reachable_from_the_ribbon`.
/// The rule is right and it is worth restating in its own words:
///
/// > *A command reachable only by right-clicking one particular surface is
/// > a command nobody can find: a context menu is discovered by an operator
/// > who already suspects something is there, which is exactly the state a
/// > command with no other home cannot put them in.*
///
/// ⇒ **The bar for an entry here is [`CUSTOM_BACKED`]'s bar, unchanged:
/// the command needs an OPERAND a ribbon control cannot ask for.** Not
/// *"a button would be redundant"*, not *"the menu is the natural place"* —
/// the ribbon control must be impossible to make *correct*.
///
/// `CUSTOM_BACKED` answers that by drawing a non-button control on the
/// ribbon that asks for the operand (a recent-files menu, a font-face
/// chooser). This register answers it for the case where **even that is
/// impossible**, because the operand is *the surface the operator
/// gestured at*. There is no ribbon control that can ask "which of the
/// twelve panels?" and get the answer "the one you just right-clicked",
/// because at the moment a ribbon control is pressed the operator is not
/// pointing at a panel.
///
/// # How discoverability is answered instead
///
/// The rule's reason is discoverability, and a register creates none.
///
/// The **capability** is on the ribbon even though the per-panel verbs are
/// not. View ▸ Window carries `view.dock_all_panels` — *"Bring every
/// floating panel back into the dock"* — and `view.reset_layout`. An
/// operator reading that group learns that panels can float and that there
/// is a way back, which is the fact worth discovering; where the verb that
/// floats *this* panel lives is then the universal idiom, on the tab.
///
/// That is a weaker answer than a ribbon button and it is stated as such
/// rather than dressed up. The day a panel tab grows a visible affordance
/// — a close cross, a chevron — these entries come out, because then there
/// is a control on the surface itself and the menu is a second route rather
/// than the only one.
///
/// # What an entry buys and what it does not
///
/// It buys the two tests above. It does **not** buy the rename check:
/// `every_command_every_menu_names_is_registered` still runs, so an entry
/// naming a command that no longer exists, or a command here that is not
/// in any menu, fails [`tests::every_tab_scoped_entry_is_real`] in both
/// directions.
pub const TAB_SCOPED: &[(&str, &str)] = &[
    (
        "view.panel_float",
        // ui-text-exempt: a register reason for a reviewer and a test; never rendered.
        "Its operand is THE PANEL THE OPERATOR RIGHT-CLICKED. A ribbon button has no such operand: at the moment it is pressed the pointer is on the ribbon, not on a panel, so the button would have to invent a subject (the active tab of which dock? the last one clicked?) and would then act on something other than what the operator was pointing at. The capability is discoverable from View ▸ Window's `view.dock_all_panels`, whose tooltip names floating panels.",
    ),
    (
        "view.panel_dock",
        // ui-text-exempt: a register reason for a reviewer and a test; never rendered.
        "The mirror of `view.panel_float`, with the same operand and the same argument. It is additionally offered on the floating window's own header strip, which is a visible surface rather than a menu — so this one is nearer to having a control than its sibling, and it is listed here because the header strip is not a ribbon and the test asks about the ribbon.",
    ),
    (
        "view.panel_close",
        // ui-text-exempt: a register reason for a reviewer and a test; never rendered.
        "Same operand, same argument. Deliberately NOT given a ribbon home by aliasing it onto the View ▸ Panels toggles: those toggle a NAMED panel and this closes the one under the pointer, and collapsing the two would make a toggle behave differently depending on where it was invoked from.",
    ),
    // The two markup-node verbs, whose surface is a CANVAS menu rather than a
    // panel tab. They meet the bar for the same reason the three above do.
    (
        "markup.add_node",
        // ui-text-exempt: a register reason for a reviewer and a test; never rendered.
        "Its operand is A POINT ON ONE EDGE OF ONE SHAPE — which edge, and where along it. A ribbon button has no such operand: at the moment it is pressed the pointer is on the ribbon, so it would have to invent one (the first edge? the longest? the last one clicked?) and would then split an edge the operator was not pointing at. The capability is discoverable from the tool row's `view.tool_node`, labelled Points, which draws an anchor on every corner of the selected shape and so teaches that a drawn shape HAS corners you can aim at.",
    ),
    (
        "markup.remove_node",
        // ui-text-exempt: a register reason for a reviewer and a test; never rendered.
        "The mirror of `markup.add_node`, with the same operand and the same argument — this corner, the one under the pointer. It is additionally the more dangerous of the two to give a ribbon home: a button that removed some invented default corner would silently reshape a drawing on a press the operator read as harmless.",
    ),
    // The route to one line of a text block — `OPERATOR_REQUESTS.md` O188(A).
    //
    // ⚠ This is the one entry here that is **the only** route rather than the
    // discoverable second one, and that is deliberately visible in its reason
    // rather than smoothed over. The three panel-tab entries and the two markup
    // verbs all end by naming where else the capability can be found; this one
    // ends by saying there is nowhere else, because the report it closes is
    // that there was nowhere at all.
    (
        "format.select_text_line",
        // ui-text-exempt: a register reason for a reviewer and a test; never rendered.
        "Its operand is ONE LINE OF ONE BLOCK OF TEXT - which run of which text object, decided by where the pointer was at the instant of the secondary click. A ribbon button has no such operand: by the time it is pressed the pointer is on the ribbon, so it would have to invent one (the first line? the last one clicked?) and would then descend onto a line the operator was not pointing at - which is this feature's whole defect class, since the next thing pressed at that rung is Delete. It is NOT the case that the capability is discoverable elsewhere: it was reachable only by arming `view.tool_node` (Points) with a chord before clicking, and that nothing named it is the report this row exists to close, so this menu is currently the ONLY route rather than the second one. A ribbon home is owed the day the Part rung gains a subject a button can name - `RIBBON_IA.md` has no such control today.",
    ),
];
