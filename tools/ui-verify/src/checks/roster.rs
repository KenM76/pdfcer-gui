//! `checks::roster` — **which checks exist, and the order the suite runs
//! them in.**
//!
//! # Why this is its own file
//!
//! Split out of [`super`] on 2026-09-05 under **R2** (no `.rs` file over 1,500
//! lines), when registering
//! [`layers_membership::SelectingAnObjectNamesItsLayer`] took `checks/mod.rs`
//! to 1,540. It could have been kept under the limit by writing less about the
//! new check, and that is the wrong trade — the limit exists to force a seam,
//! not to ration explanation.
//!
//! ## The seam, and the argument for it
//!
//! `checks/mod.rs` now holds **the harness's vocabulary**: the [`Check`] trait
//! every check implements, the [`CheckContext`] every check is handed, and the
//! module declarations that make the tree. This file holds **the roster**: one
//! `Box::new(..)` per check, in run order, each with the note explaining why it
//! sits where it does.
//!
//! Those are genuinely two subjects, and the tell is who edits them:
//!
//! | | `mod.rs` | `roster.rs` |
//! |---|---|---|
//! | changes when | the harness gains a capability every check can use | **any** check is added, removed or re-ordered |
//! | how often | rarely | ★ every single landing that ships a driven check |
//! | reviewed for | is the contract still right? | is this check in the right place, and does its note say why? |
//!
//! ★★ The second row is the whole argument. This list grows with every feature
//! this project ships, and it was the *only* thing in `mod.rs` that did. A file
//! whose growth is unbounded by construction, sharing a file with a trait that
//! has changed three times in six months, guarantees the limit is hit again —
//! and hit by whoever is unlucky rather than by whoever is responsible.
//!
//! ## ★ The `pub mod` declarations stayed behind, deliberately
//!
//! They look like roster material and they are not: a `mod` declaration
//! *defines the module path*, so moving `pub mod layers_search;` here would
//! rename the check to `checks::roster::layers_search` and break every
//! reference in the crate. They stay in `mod.rs`, which is also where a
//! reader looking for "does a check for X exist?" will look first.
//!
//! ## The ordering notes are content, not decoration
//!
//! Several entries carry a paragraph about **why they are adjacent to the one
//! above** — a dependency (`unshare_form` SKIPs on what `form_selection`
//! asserts), a pairing (two checks that differ only in the document they open),
//! or a diagnosis that only reads correctly when two verdicts sit together in
//! the summary. Those notes travelled with the entries. Re-ordering this list
//! without reading them has cost this project a misdiagnosis before.

use super::*;

/// Every check, in the order the suite runs them.
#[must_use]
pub fn all() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(delete_key::DeleteKeyAfterCanvasClick),
        // The Delete key's OTHER subject. `delete_key` drives it over page
        // content on an ordinary document; this drives it over an annotation on
        // a certified one, where the honest outcome is that the control is not
        // offered at all. Adjacent because a reader comparing two Delete
        // verdicts wants them together, and because a run in which the key
        // channel is broken should say so in the cheaper check first.
        Box::new(annot_delete_gate::ACertifiedDocumentWithholdsAnnotationDelete),
        // The FORM-FIELD half of the same question, on the same fixture pair,
        // immediately after the annotation half — a reader comparing two Delete
        // verdicts on one certified document wants them together, and the two
        // checks aim at the two objects that file carries for exactly that
        // reason.
        Box::new(field_delete_gate::ACertifiedDocumentWithholdsFieldDelete),
        Box::new(ribbon_captions::RibbonGroupCaptionsLegible),
        // Immediately after the captions check, and for that check's own stated
        // reason: both launch, both measure the band, and a reader comparing two
        // ribbon verdicts wants them adjacent. This one raises the window, so it
        // is second of the two.
        Box::new(ribbon_mockup::RibbonMatchesTheMockupGeometry),
        // Reads the trace only — no window is raised and no capture is taken,
        // so it costs nothing and cannot take the operator's focus. Placed
        // after the captions check because both launch, and a reader
        // comparing two ribbon verdicts wants them adjacent.
        Box::new(qat_icons::QatControlsAreIconOnly),
        // ★ Immediately after the QAT's icon check, because it is the SAME
        // defect on a second surface — an icon painter that exists and is
        // never handed to a call site — and a run where both fail says
        // "the icon set is broken" while a run where only this one fails
        // says "one wiring line is missing". Ordering them adjacently is
        // what makes that difference legible in the summary.
        Box::new(menu_icons::MenuRowsDrawTheirIcons),
        // ★★★ O126's THREE drivable features. This comment used to say there
        // were two, and that the third — selecting an object highlighting its
        // layer — could not be driven because `pdfcer-core` could not report a
        // content object's optional-content group. **That was true when it was
        // written and false within a day.** `Pass 250.0` answered this
        // project's own filed request (`oc: Option<ObjId>` on `PathObject`,
        // `TextObject` and `ImageObject`), and the check is
        // `layers_membership::SelectingAnObjectNamesItsLayer`, registered
        // below among the driving checks.
        //
        // ⇒ The seventh recurrence of this project's most expensive pattern: a
        // sentence about what the engine cannot do is a **dated citation with a
        // shelf life measured in hours**, and a citation living in a comment
        // cannot go red when it expires. Where the claim can be an assertion,
        // make it one — which is what the check now is.
        Box::new(panel_float::PanelsFloatCloseAndDock),
        Box::new(layers_search::LayersSearchNarrowsTheList),
        // ★ The first *driving* check, and it goes first among them on
        // purpose: it is the cheapest — two clicks on one always-enabled
        // control, no canvas gesture, no keystroke, no capture — and it is the
        // one whose failure most changes what a later failure means. Every
        // check below assumes a document is on screen; this one is the only
        // one that makes a document rather than being handed one, so if the
        // ribbon-click channel is broken it says so here, in seconds, instead
        // of at the end of a canvas drag.
        Box::new(new_document::NewDocumentMakesAPage),
        // ★ Immediately after its sibling, and it drives the OTHER New.
        //
        // `new_document` asserts that `file.new` makes a page at all;
        // this one asserts that `file.new_from_template` makes a page of the
        // size that was asked for. Adjacent because a failure in the first
        // explains a failure in the second, and reading them the other way
        // round wastes the reader's first hypothesis.
        //
        // It launches with NO fixture, like its sibling and for the command's
        // own reason: `file.new_from_template` is registered with no
        // `enabled_when` because an operator with nothing open is the one it
        // exists for.
        Box::new(new_document_size::NewDocumentSizesThePage),
        // Clicks and captures, so it takes the desktop — but only with the
        // mouse, and only for a few seconds. Placed after the three ribbon
        // chrome checks because it depends on the same rects they read and a
        // reader comparing ribbon verdicts wants them together; placed before
        // the two typing checks because a run that fails here should fail
        // before paying for a keystroke that may never arrive.
        Box::new(markup_move::DraggingAMarkupMovesIt),
        // ★★★ Immediately after the move, and deliberately: the two share
        // steps 1-3 verbatim in shape — draw a rectangle, put the pen down,
        // click it — so a failure in EITHER of those here should be read
        // against `dragging_a_markup_moves_it`'s result first. If both fail at
        // the same step, the defect is in authoring or selection rather than in
        // either gesture.
        Box::new(annot_rotate::RotatingAMarkupTurnsIt),
        Box::new(widget_move::DraggingAFormFieldMovesIt),
        // `OPERATOR_REQUESTS.md` O76, 2026-08-31. Beside the move check
        // because they are the same gesture family on the same operand and
        // differ in one field of one trace line — which is exactly the
        // distinction a reader comparing them needs to see.
        Box::new(checkbox_resize::AResizedCheckBoxIsRedrawn),
        // ★ Beside the other form checks. It reuses `widget_move`'s first two
        // steps verbatim in shape, so a failure in EITHER of them here should
        // be read against that check's result first.
        Box::new(field_menu::RightClickingAFormFieldOpensItsMenu),
        Box::new(markup_rectangle::MarkupRectangleArmsFromTheRibbon),
        // ★ Form-field placement and selection. After the markup checks and
        // not before, because it borrows their gesture machinery — a band drag
        // and a canvas click — so a failure in either would be reported there
        // first, where the cause is, rather than here where the symptom is.
        Box::new(form_field::FormFieldPlaceAndSelect),
        // ★★ Directly after it, and the order is a **dependency** rather than a
        // preference. The refusals half ends by clicking a widget the canvas
        // census names and reading the Properties pane — which is
        // `form_field`'s phases C and D exactly. A run in which canvas
        // selection is broken should report that as `form_field`'s failure,
        // where the cause is, and a reader who has already seen it fail knows
        // to ignore this one's phase G.
        //
        // The group half comes first of the two because it needs no canvas
        // gesture at all: it is three panel clicks, so a failure in it is
        // unambiguously about the Forms panel's own wiring.
        Box::new(form_groups::FieldGroupDeleteRemovesTheSubtree),
        Box::new(form_groups::StructuralRefusalsAreSentencesNotControls),
        // ★ The blend-space disclosure. After the zoom checks, because it
        // climbs with Ctrl+wheel and a wheel that does not reach the canvas is
        // `zoom_gallery`'s failure to report, not this one's.
        Box::new(blend_space::BlendSpaceFallbackIsDisclosed),
        // ★ Immediately after `pan_refresh`, because they are the two halves of
        // one gesture: that one asserts the new area RENDERS, this one asserts
        // the page is not blank WHILE it renders. A failure of the first should
        // be read first — if nothing renders at all, what is on screen during
        // the wait is a secondary question.
        Box::new(progressive::ProgressiveRenderNeverGoesBlank),
        // Beside it, because it is the same shape of check on the same
        // surface — a ribbon control that arms a canvas tool — and a reader
        // comparing the two verdicts wants them adjacent. It goes second
        // because it is the longer of the two: it clicks the page as well as
        // the ribbon, and a snap candidate that needs confirming costs it an
        // extra click.
        // Directly after `markup_rectangle`, because it is the same surface and
        // the longer of the two: it arms three tools, clicks out two runs and
        // drives one drag. A run where the four-link chain itself is broken
        // should report that as `markup_rectangle`'s failure first — this one
        // would fail for the same reason with three more candidate causes in
        // front of it.
        Box::new(markup_shapes::MarkupFreehandAndVertexKinds),
        // ★ Directly after the markup checks, and for the same dependency
        // reason: this one is about the pen those gestures author WITH, so a
        // run in which the ribbon-click channel is broken should report that as
        // `markup_rectangle`'s failure first.
        Box::new(markup_style::MarkupStyleGroupIsDrawn),
        Box::new(measure_perimeter::MeasurePerimeterTracesAndCloses),
        // ★★★ Immediately after it, and the ordering is a dependency rather
        // than a preference: this check DRAWS a perimeter before it reshapes
        // one, so if the tracing gesture is broken the run should say so under
        // the name of the check whose subject that is. Its own early steps
        // decline with "run that one first" for the same reason.
        //
        // ⬜ REGISTERED AND NEVER RUN — written 2026-09-05 while another track
        // held the pointer. See its module header.
        Box::new(dimension_corner_count::ACornerCanBeAddedAndTakenAway),
        // ★ Early, and deliberately: it is the cheapest possible statement of
        // "can this program open the file at all", and a failure here changes
        // what every later failure on an encrypted document would mean.
        Box::new(password_prompt::AnEncryptedDocumentCanBeOpenedWithItsPassword),
        Box::new(measure_linear::MeasureLinearPlacesADimension),
        // Beside the linear check: same tab, same arming path, and a reader
        // comparing two measure verdicts wants them together. It pins its own
        // fixture rather than taking --pdf — see its header for why a sweep's
        // document cannot exhibit the defect it exists for.
        Box::new(measure_circular_points::ThreeClicksRoundAHoleMeasureTheHole),
        // Straight after the linear tool, because it is the same gesture with a
        // different ending and a reader scanning the suite should meet them
        // together — and because a calibration is worthless if the linear pick
        // it reuses is broken, so failing in that order reads as a diagnosis.
        Box::new(measure_calibrate::MeasureCalibratesByPickingTwoPoints),
        Box::new(measure_hover::MeasureHoverShowsWhatItWillTake),
        // ★ The Manage-groups window, wired 2026-08-18 after being registered,
        // drawn and inert for the whole life of this build. Beside the two
        // measure checks because it is the third link in the same chain: a
        // tool places a dimension, a window calibrates its group, and this one
        // is where the group comes from.
        Box::new(dimension_groups::DimensionGroupsPanelMakesAGroup),
        // ★ Directly after the markup checks, and the order is a **dependency**
        // rather than a preference: this one begins by arming Rectangle and
        // dragging, which is `markup_rectangle`'s and `markup_shapes`' whole
        // subject. A run in which the four-link arm chain is broken should
        // report that as their failure and this one's second — here the same
        // symptom has the entire save path stacked behind it, and a reader who
        // has already seen Rectangle fail knows to ignore the rest.
        //
        // It is the most expensive check in the suite by some way: it launches
        // the binary **twice**, because the round trip it exists to prove is
        // that a second process can read what the first one wrote. Placed
        // before the two typing-dependent checks all the same, because it is
        // the only one whose subject is a file on disk and a run that cannot
        // save should say so before it spends anything on a keystroke that may
        // never arrive.
        Box::new(save_in_place::SaveWritesOverTheFileYouOpened),
        Box::new(save_copy::SaveCopyRoundTrip),
        // ★ Directly after it, and the order is a **dependency** rather than a
        // preference: this check ends by saving a copy and re-opening it, so a
        // run in which `file.save_copy` itself is broken should report that as
        // `save_copy_round_trip`'s failure and this one's second. A reader who
        // has already seen the save fail knows to ignore everything below
        // phase E here.
        //
        // It is the second most expensive check in the suite, and for the same
        // reason: it launches the binary **twice**, because a page count read
        // in the process that deleted the page is a count the code under test
        // wrote about itself.
        Box::new(page_cache::PagesStayDrawnWhenYouScrollBack),
        Box::new(page_ops::PageOpsRoundTrip),
        // ★ The FIRST check anywhere that drives the Pages PANEL rather than
        // the Pages tab. `page_ops` above drives the ribbon and records why it
        // does not touch the panel — the tile's context menu is an egui popup
        // that declares no regions. The tiles themselves declare regions as of
        // 2026-08-18, which is what made this possible.
        Box::new(pages_drag::PagesDragShowsWhereItLands),
        Box::new(tab_order_drag::TabOrderDragMovesAFieldAndShowsWhere),
        Box::new(forms_spotlight::ClickingAFormRowLightsTheFieldOnThePage),
        Box::new(display_two_rows::TheDisplayButtonsStackInTwoRows),
        Box::new(title_build_stamp::TheTitleBarCarriesTheBuildTime),
        Box::new(field_shading::FillableFieldsAreShadedOnThePage),
        Box::new(preset_group_reachable::TheStandardsPresetsGroupIsReachable),
        Box::new(redact_image_warning::MarkingOverAnImageSaysSoBeforeApply),
        // The DXF export, wired 2026-08-19 after being the FIRST entry in
        // `reach`'s scaffold list. Beside the page checks because it is the
        // other verb that writes a file the operator hands to somebody else.
        Box::new(embed_fonts::EmbeddingFontsPutsAProgramInTheDocument),
        Box::new(embed_bundled::EmbeddingWorksWithNoFontFolderAtAll),
        Box::new(compact_save::ACompactedCopyIsActuallySmaller),
        // ★★★ The page-tree guard, wired 2026-09-05 from the operator's own
        // report. Placed immediately after `compact_save` because the two are
        // the same subject from opposite ends: that one asserts a save
        // PRODUCES the file it promised, this one asserts a save REFUSES to
        // produce a file it knows is damaged. A reader comparing the two
        // verdicts is reading both halves of "what may leave this program".
        //
        // ⚠ It pins its own fixture and ignores `--pdf`, so it needs nothing
        // from the sweep's aim table — and it is UNDRIVEN. Its header says so.
        Box::new(pagetree_guard::ASaveThatWouldProduceBlankPagesIsRefused),
        // ★★★ The signature warning, wired 2026-08-28. Beside the save checks
        // because it is the fourth thing this shell can do to a file somebody
        // else will open — and the only one whose subject is what the file
        // CLAIMS about itself rather than what it contains.
        //
        // After `compact_save` deliberately: that check drives the one save
        // path that already disclosed its effect on signatures (a full rewrite
        // destroys them all, §12.8.1, and its window says so before the picker
        // opens), so a reader comparing the two verdicts is reading the two
        // halves of one subject in the order they were built.
        Box::new(signature_save::AnInvalidatingSaveIsWarnedAbout),
        Box::new(trust_store::SignatureTrustIsReportedAsItsOwnFact),
        Box::new(os_fonts_setting::FontFoldersLandsOnTheFontsSetting),
        Box::new(unembed_fonts::RemovingEmbeddedFontsReachesTheDocument),
        Box::new(export_form_data::ExportingFormDataWritesAFile),
        Box::new(export_dxf::ExportDxfWritesThePagesGeometry),
        Box::new(export_image_emf::ExportImageWritesAMetafile),
        Box::new(copy_as_vector::CopyAsVectorPlacesTheMeasuredOrder),
        Box::new(export_text::ExportTextWritesTheDocumentsWords),
        // Insert an image, wired 2026-08-19. Its last assertion is the one
        // that matters: the promised resolution and the reported one are the
        // same number, which is the shell's half of a single-derivation
        // guarantee `pdfcer-core` holds up on its side with a test.
        Box::new(attachment_clip::AnAttachmentMovesBetweenTwoOpenDocuments),
        Box::new(marquee_table::AMarqueeOverATableTakesItsTextAsWellAsItsLines),
        Box::new(bookmark_dest::ABookmarkLandsOnTheDetailItNames),
        Box::new(ocr_text_select::TextOnAScanCanStillBeSweptOverTheImage),
        Box::new(save_after_edit::CtrlSAfterAnEditSavesAndTheProgramIsStillRunning),
        Box::new(button_action::APlacedButtonCanBeGivenSomethingToDo),
        Box::new(insert_image::InsertImagePlacesAPicture),
        // `OPERATOR_REQUESTS.md` O66, 2026-08-31. Immediately after its
        // sibling, because it depends on everything that one establishes — the
        // picker seam, the window opening, the fixture PNG — and adds exactly
        // one thing: that the window gets out of the way when asked and comes
        // back afterwards. Its real subject is the JOIN; every part of the
        // placement arm is unit-tested and each part passes alone.
        Box::new(insert_image_place::TheInsertWindowStepsAside),
        // `OPERATOR_REQUESTS.md` O67, 2026-08-31. Beside the insert checks
        // because it is the third route to the same verb — the picker, the
        // dialog, and now a file dropped on the grid — and the one that has to
        // prove a POSITION was used rather than a default.
        Box::new(drop_onto_thumbnails::ADrawingDroppedOnTheThumbnails),
        // `OPERATOR_REQUESTS.md` O70, 2026-08-31. It reads the same
        // `canvas-selection` line the font-group check does and asks the one
        // question that line was extended for: WHICH index space did the click
        // land in? Both answers are `sel=1 level=Object`.
        Box::new(smart_select::AClickSelectsTheWholeDrawing),
        // Immediately after the descent, because it depends on everything that
        // check establishes and adds exactly one thing: that what you reached
        // can be edited. Reaching something and being unable to move it is a
        // worse state than not reaching it — the outline is a promise the
        // gesture then breaks.
        Box::new(form_leaf_move::AThingInsideAWrappedDrawingCanBeDragged),
        // …and one rung deeper again. Third of the three, in the order an
        // operator meets them: reach it, edit it, go inside it.
        Box::new(form_leaf_descend::TheLadderGoesAsDeepInsideAContainer),
        // The chain's last rung, and the one that leaves the geometry: text.
        Box::new(double_click_text::DoubleClickingATextBoxEditsTheText),
        // `OPERATOR_REQUESTS.md` O71, 2026-08-31. Beside the Smart-Selector
        // check because both are about what a plain click MEANS — that one in
        // Edit, this one in Read, which is the stance where the answer had
        // always been "text, or nothing".
        Box::new(read_image_copy::ReadModeCopiesAPicture),
        // The Properties panel's document half, wired 2026-08-19 after a
        // recorded blocker — "`pdfcer-core` exposes no /Info accessor" — turned
        // out to have cleared without the prose moving. Beside the page checks
        // because it is the other surface that edits the DOCUMENT rather than
        // a page's content.
        Box::new(properties_metadata::PropertiesMetadataRoundTrips),
        // ★ Two ribbon clicks and one trace line — cheap, no capture, no
        // canvas gesture, no keystroke — so its position is chosen for what a
        // reader wants adjacent rather than for cost.
        //
        // It sits here, among the checks that drive a real document, because
        // its precondition is one: `file.print` is gated on `doc.open`. It
        // must NOT move up among the chrome checks, which run without a
        // fixture.
        //
        // ★ It never presses the commit button, and no future edit may make it
        // do so. That button is the one control in the application that
        // consumes paper and cannot be undone; a harness that can start a
        // print job will eventually start one by accident. The module header
        // states what that costs and why the cost is worth paying.
        Box::new(print_dialog::PrintDialogReachesTheSpooler),
        // ★ Immediately after the spooler check, because the two are the two
        // halves of "does Print work": one asks whether the job reaches the
        // device, the other whether the window the operator drives it from is
        // usable. The gap between those halves is where four defects shipped —
        // see `print_layout`'s header.
        Box::new(print_layout::PrintDialogBodyDoesNotDeadlockItsScrollbars),
        // ★ Immediately after it, and the ORDER is load-bearing rather than
        // tidy. This check's every skip message defers to `print_dialog` for
        // the diagnosis — "the dialog never opened", "the spooler refused",
        // "the ribbon control is missing" are all its subject, not this one's.
        // Running it first means the reader of a failing run meets the
        // specific cause before the vaguer one, instead of reading a paper
        // check skip and having to go looking for why.
        //
        // ★ It never presses Properties…, and no future edit may make it do
        // so. That button opens a VENDOR DRIVER's own modal dialog: a nested
        // Win32 message loop whose layout pdfcer does not know, cannot publish
        // rects for, and cannot reliably dismiss — and one left standing
        // blocks the application's event loop, so a failed dismissal does not
        // fail this check, it hangs every check after it.
        //
        // ★ And it never presses commit, for the reason stated above.
        Box::new(print_paper::PrintPaperChangesThePlan),
        // ★ Last of the four print checks, and after them all for the reason
        // the two above give: every skip message it can produce defers to
        // `print_dialog` for the diagnosis — "the dialog never opened", "the
        // spooler refused", "the ribbon control is missing" are its subject.
        // A reader of a failing run should meet the specific cause first.
        //
        // ★ It never presses commit either. Four print checks now state that
        // rule; it is restated rather than referenced because the day somebody
        // adds a fifth by copying one of them, the copied file is what they
        // will read.
        Box::new(print_clip_claim::PrintClipClaimFollowsThePreview),
        // ★ Fifth of the print family and last, for the reason the four above
        // give. ⚠ **NEVER RUN** — registered 2026-09-05 with the operator at
        // his machine; its own header says so first and says what a first run
        // will probably teach it. It never presses commit, like the four above.
        Box::new(preview_popout::ThePrintPreviewPopsIntoItsOwnWindow),
        // ★ Beside it because it is the same shape — two ribbon clicks into a
        // dialog — and because both are checks whose subject is a control that
        // was drawn and did nothing.
        //
        // It launches with NO fixture, deliberately: `file.settings` is
        // application-scoped and must work with nothing open. That also makes
        // it the cheapest driving check in the suite, so a run whose ribbon
        // channel is broken says so here without paying for a render.
        Box::new(settings_theme::SettingsThemeTakesEffect),
        // ★★★ Its sibling, from the same file, and it goes IMMEDIATELY after
        // for the reason `print_layout` goes after `print_dialog`: every skip
        // message it can produce defers to the check above it for the
        // diagnosis. "The three presets painted the same surround" is
        // `settings_theme_takes_effect`'s subject, not this one's, and a reader
        // of a failing run should meet the specific cause before the vaguer one.
        //
        // It is the more expensive of the two — it opens a document, renders a
        // page and takes three captures — which is why it is second despite
        // asserting the more important property. A run whose ribbon or dialog
        // channel is broken says so above, in seconds, without paying for a
        // render.
        //
        // ★ It closes both halves of `REVIEW_TRIAGE.md` PartC at once: the Airy
        // preset, which nothing in this repository had ever clicked, and the
        // PAGE, which nothing had ever sampled under a theme.
        Box::new(theme_page::EveryThemePresetKeepsThePageWhite),
        // ★ Directly after it, and for the same dependency reason it sits
        // after the markup checks: this one also begins by arming Rectangle and
        // dragging, so a run in which the four-link arm chain is broken should
        // report that as `markup_rectangle`'s failure and this one's last.
        //
        // Placed AFTER `save_copy` rather than before it, although it is the
        // cheaper of the two (one process, no file on disk): a shell that
        // cannot write what an operator authored is a worse finding than one
        // that cannot take it back, and a run is likelier to be read from the
        // top than from the bottom.
        Box::new(undo_redo::UndoRedoRoundTrip),
        // ★ Third of the two-process checks, and placed here for the same
        // dependency reason the two above it are: it ends by writing a file and
        // re-opening it, so a run in which writing itself is broken should
        // report that as `save_copy_round_trip`'s failure and this one's
        // second.
        //
        // It is the most expensive check in the suite by a small margin —
        // two launches, eight clicks, and a full rewrite of a document
        // performed synchronously inside one of them — and the most valuable
        // per second spent, because its subject is the only irreversible
        // operation the program has. Its fixture is **generated**, so unlike
        // every other driving check it does not consult `--pdf` and cannot be
        // aimed at a document that lacks the strings it scans for.
        Box::new(object_clipboard::CopyAndPastePageContent),
        Box::new(clipboard_annotation::CopyingAStickyNoteCarriesTheWholeComment),
        // ★★★ Its complement, added 2026-09-05 with the fix for the driven
        // sweep's finding A1. The check above owns the GRANT — a comment
        // copied in Review pastes in Review. This one owns the REFUSAL: a
        // clip of page CONTENT must still be refused there, and the refusal
        // must be a sentence rather than a silent return. Opening the chord
        // gate is only safe with both, and only this one can fail on the
        // half that opening it puts at risk.
        Box::new(clipboard_mode::APasteReviewMayNotDoSaysSo),
        Box::new(clipboard_text::CtrlCCopiesTextToTheOsClipboard),
        Box::new(select_filter::SelectFilterChangesWhatAClickHits),
        Box::new(scroll_input::ScrollingFarKeepsTheCanvasItsPointerInput),
        Box::new(max_zoom::TheZoomReadoutOpensTheMaximumZoomPopup),
        Box::new(deep_zoom::ZoomingPastThePixmapCeilingStillRenders),
        Box::new(deep_pan::PanningAtDeepZoomStaysWhereItWasPut),
        Box::new(scale_sweep::MouseWorkSurvivesEveryRenderTier),
        Box::new(zoom_keeps_place::ZoomingDoesNotThrowAwayWhereTheOperatorPanned),
        // ★ Its inverse. The climb above never rolls the wheel the other way, so
        // the DOWNWARD hand-over between the f32 scroll offset and the f64
        // anchor had never been driven. O26e.
        Box::new(zoom_out_keeps_place::ZoomingBackOutKeepsTheView),
        // ★ O28 and O29: a fit sets the scale AND places the view, and there
        // is a third mode. Pans into the pasteboard first, because the state
        // the request is about did not exist before O23.
        Box::new(fit_places_the_view::AFitCommandPutsThePageOnScreen),
        // ★ Immediately after it: same subject, opposite outcome, and a failure
        // in the sibling should be read first because this one builds on it.
        Box::new(fit_left_by_a_pan::APanLeavesTheFit),
        // ★ O30: the wheel as a page turn, from the status-bar toggle. Asserts
        // the DEFAULT is silent first, so a build that flipped unconditionally
        // could not pass — and that the control is absent where the choice
        // does not exist.
        Box::new(wheel_flips_pages::TheWheelTurnsPagesWhenTheOperatorAsksItTo),
        Box::new(zoom_gallery::ThePageStillRendersAtEveryDecadeOfZoom),
        Box::new(pan_refresh::PanningPastTheOverscanRendersTheNewArea),
        Box::new(resize::ResizeScalesAShape),
        // ★ Directly after `resize`, because it is that check plus one switch:
        // a failure in `resize_scales_a_shape` should be read first, since
        // every link it covers is in front of this one.
        Box::new(scale_switch::TheLineWeightSwitchReachesTheResize),
        Box::new(rotate::RotateHandleTurnsASelection),
        Box::new(shift_constrains::ShiftConstrainsAResize),
        Box::new(geometry_fields::GeometryFieldsResizeAShape),
        Box::new(restyle_text::RestylingSelectedTextReachesTheDocument),
        // ★ Directly after `restyle_text`, because it is that check plus a
        // popup: every link it covers — the sweep, the section, the read-back
        // stamp — is in front of this one, so a failure there should be read
        // first.
        Box::new(std14_face::TheFaceChooserOffersAFaceTheDocumentDoesNotContain),
        Box::new(font_group::TheFormatTabOffersFontControlsForSweptText),
        // ★ After `font_group`: same state, and that one asserts the sentence
        // while this asserts the control. Read a sentence failure first.
        Box::new(colour_clicked_text::ClickingTextOffersItsColour),
        Box::new(multi_node::MultiNodeMoveMovesEveryPickedAnchor),
        Box::new(shape_preview::DraggingANodeBendsTheLine),
        Box::new(bezier_handle::BezierHandleDragChangesACurve),
        Box::new(tool_row::TheTextToolTypesOnOneClick),
        Box::new(tool_row::ThePointsToolShowsPointsOnOneClick),
        Box::new(tool_row::ShowPointsDrawsAnObjectsPointsWithoutDescending),
        Box::new(dropped_file::ADroppedImageReachesThePlacementWindow),
        Box::new(first_frame::TheFirstFrameNamesTheArmedTool),
        Box::new(master_detail::TheInspectorIsOneMasterDetailColumn),
        // The left rail — `OPERATOR_REQUESTS.md` O123 part 7 and O126.
        // ⚠ Written on 2026-09-04 and NOT executed: the operator was at his
        // keyboard and a watchdog kills GUI processes on sight. Registered
        // anyway, because a check that is not in the list is a check nobody
        // will ever run.
        Box::new(left_rail::TheLeftRailIsReachableAndConstantWidth),
        Box::new(properties_tool::TheArmedToolsSettingsAreInProperties),
        // ⚠ O119 — registered without having been run: an unregistered check
        // is one nobody will ever run.
        Box::new(protect::ProtectShowsTheDocumentAndRefusesASignedOne),
        Box::new(redaction::RedactionRemovesAndProvesIt),
        // ★ Beside the text-editing checks and owning its own fixture, like
        // `text_edit` and `redaction` above: its verdict is a LINE COUNT that
        // only `fixtures/paragraph.pdf` produces, so it takes no `--pdf`.
        Box::new(reflow::ReflowingAParagraphRewrapsIt),
        // ★★★ O127 defect 2 — and ⚠ registered WITHOUT having been run, on the
        // precedent `left_rail`, `properties_tool` and `protect` above set: a
        // check that exists and is not registered is a check nobody will ever
        // run. Whoever runs the suite next is the first thing that executes it.
        Box::new(enter_newline::EnterMakesASecondLineAndControlEnterCommits),
        // ★ Directly after `redaction`, and before the two selection checks,
        // because it is the second most expensive check in the suite — it
        // launches the binary twice, for the same reason `save_copy` does — and
        // because its subject is the one this project exists for. A run in which
        // `save_copy` failed should be read first: every link from Ctrl+S
        // onwards is that check's, and this one has the whole text-edit path
        // stacked in front of them.
        // ★ The multi-document pair, 2026-08-20. Both SKIP without
        // `--second-pdf`, and the SKIP reason says why the same file cannot be
        // passed twice.
        Box::new(document_tabs::TwoDocumentsGetTwoTabs),
        Box::new(tab_reorder::DocumentTabsCanBeRearranged),
        Box::new(page_drag_between_documents::PageDraggedBetweenDocuments::COPY),
        Box::new(page_drag_between_documents::PageDraggedBetweenDocuments::MOVE),
        Box::new(about::AboutReportsTheBuild),
        // Beside About because it is its neighbour in every way that matters to
        // a run: it launches with NO document, it opens one window, and it
        // asserts on a trace rather than on pixels. It is also cheap.
        Box::new(shortcuts::ShortcutsReferenceIsLive),
        // ★ Immediately after the two checks whose SKIPs found the defect it
        // exists for. Both of those now maximise the window and are green;
        // this one deliberately does NOT, so the narrow band they used to
        // trip over is still driven by something every run.
        Box::new(band_scroll::ACommandTwoScrollStopsAwayIsStillReachable),
        Box::new(block_nav::ArrowKeysWalkBetweenBlocks),
        Box::new(dialog_windows::DialogsOpenInTheirOwnWindow),
        Box::new(draft_selection::ShiftArrowsSelectText),
        Box::new(bookmark_add::BookmarkCanBeWritten),
        // ★ Immediately after its sibling, because they are one assertion in
        // two halves: that authoring is REACHABLE in Review, and that it is
        // ABSENT in Read. Either alone is satisfied by a build that is simply
        // wrong the other way — a panel that never draws the row passes the
        // absence test, and one that draws it everywhere passes the presence
        // test.
        Box::new(bookmark_add::ReadModeOffersNoBookmarkAuthoring),
        Box::new(bookmark_edit::ABookmarkCanBeRenamedAndRemoved),
        Box::new(bookmark_move::ABookmarkCanBeDraggedAndABranchCollapsed),
        Box::new(attachments::AFileCanBeAttachedAndTakenBackOut),
        Box::new(comment_note::ANoteCanBeWrittenOntoAShape),
        // ★★★ Its opposite number, added 2026-09-05: `comment_note` proves a
        // comment can be WRITTEN, and this one proves one can be READ — in
        // **Read mode**, where until that date there was no route to a note's
        // words at all. The operator's report is the check's own defect string.
        // ⬜ NOT RUN; the module says so in its header.
        Box::new(comment_popup::ACommentCanBeReadOnThePageInReadMode),
        // Last of the three new ones and the most expensive: it drives Insert
        // pages, the Forms panel and the Tab-order section in one session,
        // because the shape it registers does not exist in any fixture — pdfcer
        // makes it. See that module's header.
        Box::new(adopt_widget::AdoptWidgetPutsAFormControlBack),
        Box::new(add_text::AddTextTakesRealKeystrokes),
        Box::new(chords::EveryDeclaredChordDispatches),
        Box::new(text_annot::TextAnnotPlacesAndAuthors),
        Box::new(text_annot_focus::TextAnnotTakesTheKeyboardUnclicked),
        Box::new(text_box::TextBoxTakesAParagraph),
        Box::new(text_edit::TextEditPinsAnAlignedTail),
        Box::new(text_edit_real::TextEditOnARealDrawing),
        // After both, because it is the only driving check that does not touch
        // the ribbon band at all — it clicks mode segments and the page — and
        // because it is the slowest: it searches for a point with content
        // under it, and every candidate costs four clicks.
        Box::new(read_mode::ReadModeRefusesCanvasEdits),
        Box::new(text_selection::TextSelectionSweepsAndCopies),
        // ★ Directly after the sweep it depends on. This one asserts EXACT
        // character and box counts on its own committed fixture, where the
        // check above asserts liveness on whatever `--pdf` names — so a run in
        // which the sweep gesture is broken at all should report THAT here
        // first, and this one's more specific failure second.
        Box::new(rotated_text::RotatedTextSelectsAndCopiesAsOneLine),
        // Directly after it, and the order is a dependency rather than a
        // preference: this one *begins* by making a text selection, so a run
        // where the sweep itself is broken should report that as the sweep's
        // failure first and this one's SKIP second. It is also the longer of
        // the two — three ribbon clicks and a drag, one of which authors an
        // annotation into the open document (never onto disk: nothing here
        // saves).
        Box::new(text_markup::TextMarkupMarksASelection),
        // Directly after it, and again the order is a dependency rather than a
        // preference: this one does everything `text_markup` does and then some,
        // in a different mode and behind a tool that has to arm first. A run
        // where the marking path itself is broken should report that as
        // `text_markup`'s failure, and this one's second — because here the same
        // symptom has one more candidate cause (the tool), and a reader who has
        // already seen Review fail knows to ignore it.
        //
        // It is the longest driving check in the suite: five ribbon clicks
        // across three tabs, two drags, and one annotation authored into the
        // open document (never onto disk; nothing here saves).
        // ★ Last of the driving checks, and the slowest: it runs a real
        // recognition, which is a second in a release build. Placed after the
        // cheap ones so a run that is going to fail on something structural
        // fails before spending it.
        // ★★ O58's discharge, 2026-08-29. It runs AFTER the selection checks
        // deliberately: its first three phases are `field_menu`'s (place,
        // clear, select), so if those are broken this check should not be the
        // first thing to say so — it would name the clipboard for a selection
        // defect. Its own SKIP messages distinguish the two.
        // ★ O59's first item, before the clipboard pair: it is the one whose
        // failure is DESTRUCTIVE. The other two prove a capability works; this
        // proves one cannot happen.
        Box::new(cut_gate::CuttingARedactionMarkIsRefusedBeforeAnythingIsRemoved),
        // ★ O60 — redacting what is selected, the third marking route.
        Box::new(redact_selection::ASelectedObjectCanBeMarkedForRedaction),
        // ★ O61 — the document-safety disclosure.
        Box::new(reach_out::ADocumentThatPhonesHomeSaysSo),
        // ★ O62 — the rotation direction, which is one sign and invisible.
        Box::new(widget_rotate::TurningAFieldRightTurnsItRight),
        // ★ O59 item 2 — the page clipboard.
        Box::new(page_clipboard::PagesCanBeCopiedAndPasted),
        // ★ O59 item 3 — the bookmark clipboard, and the one operation in
        // this program Acrobat cannot do between two files at all.
        Box::new(bookmark_clipboard::ABookmarkSubtreeCanBeCopiedAndPasted),
        Box::new(field_clipboard::AFormFieldCanBeCopiedAndPastedBothWays),
        // ★★ Immediately after its sibling, and it is the MIRROR of it: same
        // gestures, opposite expectations, one environment variable apart. A
        // build that ignored the paste-order setting passes the first and fails
        // this one on its first assertion.
        Box::new(field_clipboard::TheAcrobatPasteOrderSwapsWhichChordDoesWhich),
        Box::new(form_selection::AClickInsideAFormSelectsWhatIsDrawnThere),
        // ★★ Immediately after it, and the adjacency is the point: both are
        // "what does a click on the canvas mean?", read through two different
        // off-canvas oracles. A run where both fail says the click is not
        // arriving; a run where only this one fails says the click arrives and
        // the layer relation is broken. Ordering them apart would make that
        // difference unreadable in the summary.
        //
        // ⚠ **NOT RUN.** Written 2026-09-05 with the operator possibly at his
        // machine; it has never seen a running binary. See its module header.
        Box::new(layers_membership::SelectingAnObjectNamesItsLayer),
        // ★ Immediately after it, and the order is a dependency rather than a
        // preference: this check's second step is `form_selection`'s first
        // assertion — a click inside a form must select the leaf — and it
        // SKIPs rather than fails when that does not hold, so that a broken
        // deep hit test is reported once, by the check that owns it, instead of
        // twice with the second reading blaming the wrong file.
        Box::new(unshare_form::TheContextMenuGivesThisPageItsOwnCopyOfASharedForm),
        // ★★★ Its pair, and it must stay adjacent to it. The two press the same
        // row through the same five steps and differ only in the document they
        // open — `shared-across-two-pages.pdf` against `page-sized-form.pdf` —
        // so they are one behaviour's two halves, not two features. Reading the
        // suite output, an author who sees only one of them run has been told
        // that the other half of a branch is unmeasured, which is exactly the
        // state that let a check named `…_of_a_shared_form` sit on an unshared
        // fixture for a day.
        //
        // ★ It costs a second launch and a second window, deliberately: the
        // condition under test is a property of the open file, so it cannot be
        // reached by any further gesture within the first check's session.
        Box::new(unshare_form::TheUnshareDeclinesWhenNothingElseDrawsTheForm),
        // The two link checks, before OCR: they are two launches and two clicks
        // apiece against kilobyte fixtures, where the OCR checks below are a
        // minute each. A run that fails on something cheap should fail before
        // paying for something expensive.
        Box::new(off_page_marquee::ABandDraggedIntoTheMarginReachesAnObjectOffThePage),
        // Two launches and an Alt+F4, so it is placed with the other
        // multi-process checks rather than among the single-window ones.
        Box::new(page_display_pref::APageDisplayChoiceSurvivesACloseAndReachesANewDocument),
        Box::new(save_as::SaveAsRebindsTheDocument),
        // Last of the new group: it is TWO launches and it presses Alt+F4,
        // so a run that fails on something cheaper should fail first.
        Box::new(quit_unsaved::ClosingTheProgramAsksBeforeLosingUnsavedWork),
        Box::new(link_follow::ALinkGoesToThePageItNames),
        Box::new(link_follow::ALinkItCannotFollowSaysSo),
        Box::new(ocr::OcrRecognisesAPageAndTheDocumentKeepsIt),
        // The three about a run in progress. After the one-page check, because
        // a build in which recognition does not work at all should say so
        // before three checks spend a minute apiece observing it not working.
        Box::new(ocr_progress::OcrSaysHowFarItHasGot),
        Box::new(ocr_progress::StoppingOcrKeepsWhatItHasDone),
        Box::new(ocr_progress::CancellingOcrThrowsAwayWhatItHadDone),
        Box::new(text_tool::TextToolSelectsAndMarksInEdit),
        // Last, because it is the only check that TYPES. Everything above
        // either reads a trace or captures a window; this one presses a
        // chord, types a needle and presses Enter into a real foreground
        // window, so it costs the operator their focus for a few seconds.
        // A run that fails earlier should fail before paying that.
        // Cheap and non-destructive: two ribbon clicks, no canvas gesture, no
        // keystroke, and a window that changes nothing. Placed here rather than
        // among the first driving checks only because it depends on a raster
        // having landed, and everything above has already waited for one.
        Box::new(render_diagnostics::RenderDiagnosticsOpensItsReport),
        Box::new(find_bar::FindOpensAndFinds),
        // ★ Second to last among the driving checks, and the placement is a
        // property of what it does rather than of what it costs: it is the only
        // check that **cannot put the application back**. Read mode's exit is
        // `Ctrl+H` and this machine cannot inject keystrokes, so the session
        // ends with the chrome hidden. That harms nothing — every check launches
        // its own process and read mode is per-session by design — but a reader
        // scanning a run for the first failure should not meet a check whose
        // window looks broken in its artefacts before the ones whose windows
        // look ordinary.
        //
        // Cheap otherwise: two ribbon clicks, two captures, no canvas gesture
        // and no keystroke.
        // ★★★ Immediately BEFORE its neighbour, and the two are the two halves
        // of one subject: that one asserts read mode **hides** the chrome, this
        // one asserts it **says how to get it back**. A reader scanning a run
        // wants those two verdicts together.
        //
        // First of the two for the reason the cheaper check always goes first:
        // it sends no pointer and no keystroke — `PDFCER_DIAG_INVOKE` rings the
        // command through the dispatcher — so it costs nothing, takes no focus,
        // and can run on a machine somebody is using. If both go red, the one
        // that needed no input is the one whose diagnosis to believe.
        //
        // ⬜ NOT RUN by the session that wrote it (2026-09-05); its own header
        // says so in its first section.
        Box::new(read_mode_exit::ReadModeSaysHowToGetBackOut),
        Box::new(read_mode_chrome::ReadModeHidesTheChrome),
        Box::new(settings_headings::SettingsHeadingsLegible),
        // ★ LAST, and for a reason that is the mirror of the one above.
        //
        // This check WRITES `userdata/preferences.txt` beside the binary and
        // deliberately does not restore it — see `write_preference` on why
        // tidying up would hide the state the next check inherits. Every check
        // that runs after it would therefore start at whatever scale it left
        // behind, and a window at 1.8x is a window whose every coordinate
        // differs from what the others were written against.
        //
        // Putting it last makes that a property of the *file on disk* between
        // runs rather than a property of the *suite*, which is the same
        // distinction `delete_key`'s persisted-mode defect turned on. The file
        // is left holding the large scale on purpose: the next run's base
        // launch then has something to move away from, which is what makes its
        // `ui-scale` trace line meaningful rather than vacuous.
        Box::new(ui_scale::UiScaleResizesTheChrome),
    ]
}
