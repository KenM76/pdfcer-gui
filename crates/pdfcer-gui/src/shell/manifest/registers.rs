//! # `shell::manifest::registers` — the two lists of commands this manifest
//! does NOT emit, and the one list of commands it emits anyway
//!
//! Split out of [`super`] under **R2** on 2026-08-27, when the Format tab's
//! Font group took that file to within eight lines of the 1,500 ceiling.
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
//! They change for different reasons, which is the test R2 actually cares
//! about. A tab gains a group, an item moves, a collapse priority is
//! re-ranked: [`super`]. A command is built and its deferral note has to come
//! out, or a new gap is discovered and written down: here. In the six months
//! before the split those two edits never once landed together.
//!
//! ## ★ Nothing here is enforced from here
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
/// and alone, that turns a specification of 180-odd commands into a
/// manifest of 76 with no record of the other 100, and the next person to
/// read this module cannot tell a command that was *considered and
/// deferred* from one that was *never noticed*. Those are very different
/// facts and only one of them is a plan.
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
    // -- The two splits, moved here from SCAFFOLDED on 2026-08-31 -----------
    //
    // ★★★ `OPERATOR_REQUESTS.md` O68. Ken: *"the Merge files and Split files
    // buttons don't do anything."*
    //
    // They were REGISTERED with no dispatch arm — drawn, enabled, and inert —
    // and their entries sat on the `SCAFFOLDED` allow-list, which forces an
    // explanation and never forces a fix. Merge was wired (the engine verb was
    // complete and its blocker named a missing panel); these two are
    // **unregistered**, because their blocker names a missing capability and
    // R9 says a capability that is not built renders nothing.
    //
    // ⇒ The move between registers is the whole point. `SCAFFOLDED` means
    // *"registered, drawn, and does nothing"*; `PLANNED` means *"named in the
    // IA and not built"*. These were the first for weeks and should always
    // have been the second. `no_scaffolded_command_is_also_planned` keeps the
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
    // ★ `file.new` was here — "N — a blank or from-template document. pdfcer has
    // no document-creation path at all." It shipped on 2026-08-14, and the note
    // is kept as a comment for the same reason `file.recent`'s is: "this used to
    // be planned and is now built" is the one transition this list exists to
    // make legible. Its second sentence remains true of the ENGINE and always
    // will — `pdfcer-core`'s `document.rs:10-19` states "no separate
    // builder/generation model may ever be introduced" as a named invariant —
    // which is exactly why the shipped command opens a bundled blank template
    // instead of asking pdfcer to grow a creation path. See `crate::app::blank`.
    //
    // What did NOT ship is the other half of §5.1's row, and it has its own
    // entry below rather than being folded into a comment, because it is a
    // capability an operator will ask for by name.
    // `file.new_from_template` was here — "N — §5.1's `New (blank / from
    // template)` row shipped only its BLANK half […] It needs one template
    // asset per offered size […] and a chooser; no engine work at all."
    //
    // ★ Both halves of that note turned out to be wrong in the same direction,
    // and the correction is worth keeping. It said "no engine work at all",
    // and the engine work was the whole blocker: nothing in `pdfcer-core` wrote
    // a `/MediaBox`, so the asset-per-size plan was the ONLY implementation
    // available — and it could not answer a custom size at any number of
    // assets. `crate::app::blank`'s §3a is the full record. `set_media_box`
    // shipped 2026-08-18, and the command now needs **one** asset, which is
    // the one that was already there.
    //
    // Kept as a comment rather than deleted because "this was planned and is
    // now built" is the transition this list exists to make legible.
    // `file.recent` was here — "N — needs a persisted recent-files list;
    // nothing writes one today." Something writes one now
    // (`crate::app::recent`), so the command is registered, the `recent_files`
    // custom item in File ▸ File draws it, and the entry moved to
    // `CUSTOM_BACKED`. Recorded as a comment rather than silently deleted
    // because "this used to be planned and is now built" is the one transition
    // this list exists to make legible.
    // ★★★ **`file.save` BUILT 2026-08-20** — kept as a comment rather than
    // silently deleted, for the reason this list states above: *"this used to be
    // planned and is now built"* is the transition it exists to make legible.
    // Its note read:
    //
    //   "N — in-place save is blocked on autosave and crash recovery."
    //
    // That was aimed at the wrong hazard, and it stood for a fortnight while the
    // operator saved every document through a file picker. pdfcer writes an
    // INCREMENTAL UPDATE: the previous revision stays in the file, so the format
    // already WAS the crash recovery the note was waiting for. What was
    // genuinely unsafe was the WRITE — `fs::write` truncates and then streams —
    // and that has a three-line answer (temporary beside the target, then
    // rename) which nobody had written because nobody was asking.
    //
    // The lesson, third instance in two days: **a blocker is a measurement, and
    // the question you measured is part of the measurement.**
    (
        "file.revert",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — meaningless until there is a save point to revert to, so it follows `file.save`.",
    ),
    (
        "file.export_image",
        "C — pdfcer-core rasterises to PNG/JPEG/TIFF already. Needs a DPI picker and a save \
         dialog; no engine work.",
    ),
    (
        "file.export_text",
        "C — pdfcer-core extracts text already. Needs a save dialog and nothing else. Not to be \
         confused with `file.copy_page_text` / `file.copy_document_text`, which sit in the same \
         band and are shipped: those write the extracted text to the CLIPBOARD, this one writes \
         it to a file the operator names.",
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
    // `file.about` was here — "N — there is no about box." There is one now
    // (`crate::dialogs::about`), so the command is registered and draws in
    // File ▸ pdfcer. Recorded as a comment rather than silently deleted, on the
    // `file.recent` precedent above: "this used to be planned and is now
    // built" is the one transition this list exists to make legible.
    //
    // Worth keeping the reason it stopped being optional. The box was N for as
    // long as this shell redistributed only permissively-licensed code, whose
    // notices the shipped `LICENSE` covers. The operator's 2026-08-14 decision
    // to ship CC-BY-SA-4.0 OCR model weights ends that: BY requires the notice
    // to reach the RECIPIENT of the work, and nothing in this program reached
    // them. See `crate::text::about`.
    // -- View -- `RIBBON_IA.md` §5.2 ----------------------------------------
    // ★ `view.page_continuous`, `view.page_facing` and
    // `view.page_facing_continuous` were here until Phase 4, marked N with the
    // note that the build was *"larger than it looks: the viewer holds a single
    // page index and the object provider returns nothing for any page but the
    // current one"*. All three are now emitted by `view.rs` and registered by
    // `super::commands`, so they are removed from this list rather than left
    // with a stale reason — `planned_commands_are_genuinely_absent` asserts in
    // both directions and fails on an entry that has shipped.
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
    // ★ `view.rulers`, `view.grid` and `view.guides` were here until the
    // rulers landed, marked N. Their notes were *"rulers along the canvas
    // edges, in the document's units"*, *"a drawing grid drawn over the
    // page"*, and — the one with a condition attached — *"draggable guides,
    // which need a per-document store to survive a reopen."*
    //
    // All three are now emitted by `view.rs` and registered by
    // `super::commands`, so they are removed from this list rather than left
    // with a stale reason: `planned_commands_are_genuinely_absent` asserts in
    // both directions and fails on an entry that has shipped. The condition on
    // the third is discharged by `crate::canvas::guides`, whose header records
    // why `guides.txt` is a fourth store beside `layout.ron`, `recent.txt` and
    // `page-display.txt` rather than a field in any of them.
    //
    // "The document's units" turned out to be the interesting half of that
    // first note; `crate::canvas::rulers`' header §1 is the answer.
    // ★ `view.panel_pages` was here, with the reason *"page thumbnails are the
    // sidebar rail's first pane and have no independent toggle;
    // `view.sidebar` shows the rail"*. That reason described the OLD shell's
    // rail, which this build does not have — the Pages panel is an ordinary
    // dock panel like Bookmarks and Layers, so it needs an ordinary panel
    // toggle. The entry was stale rather than early, and it is removed rather
    // than reworded because the command is now registered and drawn.
    //
    // `every_panel_is_reachable_from_the_ribbon` is the test that made the
    // staleness visible: the panel existed, was filtered out of every mode by
    // the §5b capability rule, and no operator could open it.
    // ★ `view.panel_forms` was here too, with the reason *"there is no
    // standalone Forms panel; the forms surface is reached from Edit ▸
    // Forms"*. Both halves were true when written and the first stopped
    // being true when the Forms panel shipped — the entry survived because
    // `edit.form_fill` was still the way in, so nothing forced the question.
    //
    // What forced it was the operator's answer on 2026-08-14 that Read
    // fills forms. Read is shown `file` and `view` alone, so this id — the
    // one this list had reserved to say the panel had no toggle of its own
    // — is now that toggle, and `edit.form_fill` is the entry that no
    // longer exists. Recorded as a comment rather than silently deleted for
    // the same reason as `file.recent` above: *"this used to be planned and
    // is now built"* is the one transition this list exists to make legible.
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
    (
        "pages.resize",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — rescale or re-media-box a set of pages.",
    ),
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
    // ★★ `edit.cut`, `edit.copy` and `edit.paste` were HERE until 2026-08-19,
    // marked N with the note *"there is no object clipboard"*. They are now
    // registered and drawn — the operator reported their absence, and the
    // measurement behind the note turned out to be broader than the truth:
    // page CONTENT cannot be pasted (157 verbs in `edit.rs` and none inserts
    // any), but a markup can, through `annot_author::spec_from_dict` out and
    // `add_markup` back. `canvas::clipboard`'s header carries the table.
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
    // ★ `markup.polyline`, `markup.polygon` and `markup.ink` were here and are
    // now REGISTERED — Phase 6, 2026-08-14, on the two gestures that were their
    // only blocker. Their shared reason read:
    //
    //   "N — not drag-shaped: deferred in the canvas alongside Ink and Polygon,
    //    all three needing a multi-click or freehand gesture the two-point band
    //    cannot express."
    //
    // Every word of which was true, and all of it was about a **gesture** rather
    // than about the engine: `MarkupSpec::PolyLine`, `Polygon` and `Ink` have
    // been in `pdfcer-core` since Pass 6.1. `canvas::markup::vertex` built the
    // multi-click gesture — with two endings, on the operator's own 2026-08-14
    // ruling for `measure.finish` — and `canvas::markup::ink` built the freehand
    // one. `markup.finish` is registered with them and was never in this list,
    // because the problem it solves did not exist until the tools did.
    //
    // Removed rather than annotated, because this list's contract is that
    // everything in it is absent and a "planned" row for a shipped command is the
    // drift the list exists to prevent — the same treatment the three text-markup
    // rows below got earlier the same day. The removal is recorded in
    // `manifest::markup`'s header instead, where a reader is looking at the band
    // that gained them.
    // ★★ `markup.cloud` was here and is now REGISTERED — 2026-08-19,
    // `MarkupKind::Cloud`. Removed rather than annotated, because this list's
    // contract is that everything in it is absent and a "planned" row for a
    // shipped command is the drift the list exists to prevent.
    //
    // **Its reason is worth keeping where the removal is, because the reason
    // was WRONG for weeks and nothing noticed.** It read: *"the ONLY markup
    // kind still absent for an ENGINE reason rather than a gesture one."*
    // `MarkupSpec::Cloud` shipped in `pdfcer-core` — `annot_author.rs`,
    // vertices/border/interior/width/intensity, with `EditError::TooFewVertices`
    // beside it — and this entry went on asserting a blocker that no longer
    // existed. The operator asked for the tool three times in that window.
    //
    // A PLANNED reason naming an EXTERNAL blocker is a claim about a repository
    // this project does not control, and it decays silently. There is no gate
    // for that and inventing one here would be inventing a dependency on
    // `pdfcer-core`'s internals; what there is instead is this paragraph, in the
    // place the next person will look.
    // ★ `markup.underline`, `markup.strikeout` and `markup.squiggly` were here
    // and are now REGISTERED — Phase 6, 2026-08-14, on the text-selection
    // gesture that was their only blocker. Their entries are removed rather
    // than annotated, because this list's contract is that everything in it is
    // absent, and a "planned" row for a shipped command is the drift the list
    // exists to prevent. The removal is recorded in `manifest::markup`'s header
    // instead, where a reader is looking at the band that gained them.
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
    // ★ `measure.two_line` was here and is now REGISTERED — Phase 7,
    // 2026-08-14. Its entry is removed rather than annotated, because this
    // list's contract is that everything in it is absent, and a "planned"
    // row for a shipped command is the drift the list exists to prevent.
    //
    // Worth recording where it went, because this entry had already been
    // wrong once: it read *"the pick gesture has no caller"*, which was false
    // in five documents at once — the old shell calls `pick_line_in_page` at
    // `main.rs:23564` and pdfcer's own ledger marks the row `gui [x]`. The
    // caller that was missing was ours, and it now exists:
    // `crate::canvas::measure` hosts the pick and `TwoLinePick` came across
    // with it. See `SALVAGE.md`'s correction note for the full account.
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
    // `tools.ocr` was here — "N — blocked on an OCR engine decision; see the
    // roadmap." It is registered now as **`file.ocr`** (`super::commands`, token
    // 160) and draws in File ▸ Recognise — not Tools ▸ Recognise, for the
    // reason `super::tools`'s header gives. The entry is removed rather than
    // left with a
    // stale reason: `planned_commands_are_genuinely_absent` asserts in both
    // directions and fails on an entry that has shipped. Recorded as a comment
    // on the `file.recent` and `file.about` precedents above — "this used to be
    // planned and is now built" is the one transition this list exists to make
    // legible.
    //
    // ★ The reason is worth keeping, because the note was wrong about what the
    // blocker was. It said "an OCR engine decision"; the engine had been chosen
    // (`ocrs`, for being the only surveyed candidate that passes pdfcer's wasm32
    // gate) and the whole recognition path had shipped in `pdfcer-core`. What was
    // actually blocked was **redistributing CC-BY-SA-4.0 model weights from an
    // MIT repository**, which is a licensing question and not a GUI one at all.
    // The operator answered it on 2026-08-14 — "yes ship that model in the mit
    // repo with proper credit" — and the credit mechanism was built first, so
    // `about.hbs`, `crates/pdfcer-gui/src/text/about.rs` and
    // `tools/package-portable.py` now fail together if any one of them forgets.
    //
    // The lesson is the general one this list is for: a one-line reason is a
    // claim, and a claim that names the wrong blocker sends the next reader to
    // solve the wrong problem.
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
    // Twenty-four entries from one section, and they are all one decision:
    // "build order: panel first, tab second. The panel is the harder half
    // and the tab's contents are a subset of it, so building the tab first
    // would mean writing the property editors twice."
    (
        "format.colour",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — markup property editors are built panel-first; see format.rs.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("format.fill", "N — markup property; panel first."),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("format.line_width", "N — markup property; panel first."),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("format.line_style", "N — markup property; panel first."),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("format.opacity", "N — markup property; panel first."),
    (
        "format.arrowheads",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — markup property, arrow and line only; panel first.",
    ),
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
    // ★★ `format.font` and `format.font_size` were HERE, with the note
    // "N — a text run's font; panel first." Both shipped on 2026-08-27, with
    // `format.bold`, `format.italic` and `format.font_colour`, as the Format
    // tab's Font group. The build order they cite was followed exactly: the
    // Properties panel's *This text* section landed first, the tab's controls
    // are a subset of it, and the editors were written once.
    //
    // ★ The removal is not optional bookkeeping.
    // `planned_commands_are_genuinely_absent` asserts in both directions —
    // nothing listed here may be referenced by the manifest, and nothing
    // listed here may be registered — so leaving these two rows in would have
    // failed the suite rather than becoming a stale comment. That is the
    // property the register was built for, and this is the first time it has
    // been exercised by a *deletion*.
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
/// `(id, why)`. Eight entries, and they exist as a list rather than as
/// prose because otherwise this manifest would look like it applied P3
/// everywhere except in two groups, for no stated reason.
///
/// Two of them — the render quality and settle knobs — are `partial G`:
/// the *value* exists as a compiled-in constant today and what is new is
/// the control that exposes it. The rest were named individually, with
/// their value sets and their defaults, when this shell was commissioned,
/// which is a stronger statement of intent than a status mark in a table:
/// a specification detailed enough to say *"App initiative: Never · Ask ·
/// Allowed, default **Never**"* is describing something decided, not
/// something wished for.
///
/// The honest reading of the tension: P3 exists so an operator is never
/// shown a control that does nothing. These eight are settings rather than
/// actions, every one of them has a specified default, and a setting
/// showing its default is not a stub. That is the argument. It is written
/// down here so that if it turns out to be wrong, the fix is deleting
/// eight rows from one list rather than re-deriving which entries were
/// deliberate.
pub const DIRECTED: &[(&str, &str)] = &[(
    "format.delete",
    "Not status-marked: RIBBON_IA.md §5.8 lists Delete in every selection type's row \
         without a mark. Modeless select-and-delete works today — it is what the removal of \
         the `Editing on` toggle relies on — so the command is real.",
)];
