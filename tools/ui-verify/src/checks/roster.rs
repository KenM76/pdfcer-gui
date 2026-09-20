//! `checks::roster` — **which checks exist, and the order the suite runs
//! them in.**
//!
//! # Why this is its own file
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
//! | how often | rarely | every single landing that ships a driven check |
//! | reviewed for | is the contract still right? | is this check in the right place, and does its note say why? |
//!
//! The second row is the whole argument. This list grows with every feature
//! the project ships; the trait beside it barely moves. Keeping an unboundedly
//! growing list in the same file as a stable contract makes every landing touch
//! the contract's file, and makes the file's size somebody's problem at random
//! rather than the problem of whoever owns the growth.
//!
//! ## The `pub mod` declarations stayed behind, deliberately
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
        // Immediately after the QAT's icon check, because it is the SAME
        // defect on a second surface — an icon painter that exists and is
        // never handed to a call site — and a run where both fail says
        // "the icon set is broken" while a run where only this one fails
        // says "one wiring line is missing". Ordering them adjacently is
        // what makes that difference legible in the summary.
        Box::new(menu_icons::MenuRowsDrawTheirIcons),
        // O126's three drivable features. The third of them — selecting an
        // object names its layer — is
        // `layers_membership::SelectingAnObjectNamesItsLayer`, registered below
        // among the driving checks; it rests on the engine reporting a content
        // object's optional-content group (`oc: Option<ObjId>` on `PathObject`,
        // `TextObject` and `ImageObject`).
        //
        // The rule that entry exists to enforce: **a sentence about what the
        // engine cannot do has a shelf life measured in hours**, and a claim
        // living in a comment cannot go red when it expires. Where such a claim
        // can be made an assertion instead, make it one.
        Box::new(panel_float::PanelsFloatCloseAndDock),
        // The pointer-driven half of the same capability, immediately after
        // the command-driven one: `panel_float` says the window opens, draws
        // and can be sent home by a menu row; this says it can be picked up
        // and dropped somewhere the menu row cannot reach. A failure in the
        // first explains a failure in the second, and reading them the other
        // way round does not work.
        Box::new(panel_carry::PanelCarriedHomeLandsWhereItWasAimed),
        // The two affordances the gesture shows BEFORE it commits, read after
        // the two that say the commit works. In that order because a failure
        // in the commit explains a failure in the preview and not the other
        // way round: a compass that offers a compartment the dock cannot move
        // a panel into is one defect, not two.
        Box::new(dock_drop::ADragOverTheDockOffersTheCompartmentUnderThePointer),
        Box::new(dock_drop::ADragCarriedOffTheDockOffersAWindow),
        Box::new(layers_search::LayersSearchNarrowsTheList),
        // The first *driving* check, and it goes first among them on
        // purpose: it is the cheapest — two clicks on one always-enabled
        // control, no canvas gesture, no keystroke, no capture — and it is the
        // one whose failure most changes what a later failure means. Every
        // check below assumes a document is on screen; this one is the only
        // one that makes a document rather than being handed one, so if the
        // ribbon-click channel is broken it says so here, in seconds, instead
        // of at the end of a canvas drag.
        Box::new(new_document::NewDocumentMakesAPage),
        // Immediately after its sibling, and it drives the OTHER New.
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
        // Immediately after its sibling, and the pairing is the point: that
        // one asserts a NEW document gets the size asked for, this one asserts
        // an OPEN one can be changed. The chooser they drive is the same
        // widget, reached from two different commands.
        Box::new(page_size::ResizingASheetChangesThePaperInTheSavedFile),
        // Clicks and captures, so it takes the desktop — but only with the
        // mouse, and only for a few seconds. Placed after the three ribbon
        // chrome checks because it depends on the same rects they read and a
        // reader comparing ribbon verdicts wants them together; placed before
        // the two typing checks because a run that fails here should fail
        // before paying for a keystroke that may never arrive.
        Box::new(markup_move::DraggingAMarkupMovesIt),
        // Its sibling, and the pairing is the point: `markup_move` proves
        // the MOVE ghost reaches the frame and this proves the RESIZE one does.
        // They were one arm apart in `canvas::overlay` and only the first was
        // ever driven, which is how O154 shipped.
        Box::new(markup_resize_preview::DraggingACommentsCornerShowsWhereItIsGoing),
        // Immediately after the move, because the two share their first two
        // steps — arm the rectangle tool through `PDFCER_DIAG_INVOKE`, draw a
        // shape with one drag — and a reader who sees both fail at that step
        // should read it as one defect in authoring rather than two.
        //
        // This one goes SECOND of the pair for the reason the cheaper check
        // usually goes first, inverted: it is the cheaper one (one drag, two
        // captures, no selection and no second gesture) but its subject is
        // downstream of the other's. A rectangle whose colour is wrong is worth
        // knowing about only once there is a rectangle at all.
        Box::new(markup_palette::ANewMarkupIsDrawnInAcrobatsRed),
        // Third of the three that begin the same way, and the most expensive:
        // it draws the shape, selects it, raises a contextual tab, drags a
        // spinner and photographs the page twice. Last of the group so that a
        // reader who sees all three fail at the drawing step reads it as one
        // defect in authoring rather than three.
        Box::new(markup_band::TheFormatTabRestylesASelectedMark),
        // Immediately after the move, and deliberately: the two share
        // steps 1-3 verbatim in shape — draw a rectangle, put the pen down,
        // click it — so a failure in EITHER of those here should be read
        // against `dragging_a_markup_moves_it`'s result first. If both fail at
        // the same step, the defect is in authoring or selection rather than in
        // either gesture.
        Box::new(annot_rotate::RotatingAMarkupTurnsIt),
        Box::new(import_text::ATextFileBecomesPages),
        Box::new(annot_angle_typed::TheTypedAngleTurnsAMark),
        Box::new(foreign_icon_name::AForeignIconNameReachesThePanel),
        Box::new(widget_move::DraggingAFormFieldMovesIt),
        // `OPERATOR_REQUESTS.md` O76. Beside the move check
        // because they are the same gesture family on the same operand and
        // differ in one field of one trace line — which is exactly the
        // distinction a reader comparing them needs to see.
        Box::new(checkbox_resize::AResizedCheckBoxIsRedrawn),
        // Beside the other form checks. It reuses `widget_move`'s first two
        // steps verbatim in shape, so a failure in EITHER of them here should
        // be read against that check's result first.
        Box::new(field_menu::RightClickingAFormFieldOpensItsMenu),
        Box::new(markup_rectangle::MarkupRectangleArmsFromTheRibbon),
        // Form-field placement and selection. After the markup checks and
        // not before, because it borrows their gesture machinery — a band drag
        // and a canvas click — so a failure in either would be reported there
        // first, where the cause is, rather than here where the symptom is.
        Box::new(form_field::FormFieldPlaceAndSelect),
        // Directly after it, and the order is a **dependency** rather than a
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
        // The blend-space disclosure. After the zoom checks, because it
        // climbs with Ctrl+wheel and a wheel that does not reach the canvas is
        // `zoom_gallery`'s failure to report, not this one's.
        Box::new(blend_space::BlendSpaceFallbackIsDisclosed),
        // Beside `blend_space`, because they are the same shape of check on
        // the same two surfaces: both climb the zoom with Ctrl+wheel, both read
        // pixels out of the canvas, and both end at a status-bar disclosure. A
        // run in which the wheel does not reach the canvas should report that
        // as `zoom_gallery`'s failure, and a reader who has just seen
        // `blend_space` SKIP for want of a climb knows to expect this one to as
        // well.
        //
        // It goes SECOND of the two because it also presses a ribbon item,
        // so it has one more candidate cause than its neighbour.
        Box::new(line_weights::LineWeightsOffThinsTheDrawingAndSaysSo),
        // Immediately after `pan_refresh`, because they are the two halves of
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
        // Directly after the markup checks, and for the same dependency
        // reason: this one is about the pen those gestures author WITH, so a
        // run in which the ribbon-click channel is broken should report that as
        // `markup_rectangle`'s failure first.
        Box::new(markup_style::MarkupStyleGroupIsDrawn),
        Box::new(measure_perimeter::MeasurePerimeterTracesAndCloses),
        // Immediately after it, and the ordering is a dependency rather
        // than a preference: this check DRAWS a perimeter before it reshapes
        // one, so if the tracing gesture is broken the run should say so under
        // the name of the check whose subject that is. Its own early steps
        // decline with "run that one first" for the same reason.
        //
        // REGISTERED AND NEVER RUN against the binary. See its module header.
        Box::new(dimension_corner_count::ACornerCanBeAddedAndTakenAway),
        // Beside it, and after it, because they are the two halves of one
        // report and this is the half that needed an engine Pass. It is placed
        // second on purpose: if both fail, the ce-dimension one failing too
        // says the fault is in the shared gesture grammar
        // (`dimdrag::intent`, the Points tool, the modifier path) rather than
        // in the markup side, and that is a different first place to look.
        Box::new(markup_node_edit::AMarkupShapesNodesCanBeEdited),
        // Early, and deliberately: it is the cheapest possible statement of
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
        // Immediately after the check whose gesture it extends, and the order
        // is a dependency: this one drives the same two-point calibration and
        // then asks what the window did with it. If the pick itself is broken,
        // the diagnosis belongs to the line above and this should read as its
        // consequence.
        Box::new(scale_reads_the_group::SetScaleReadsTheGroupItIsAboutToOverwrite),
        Box::new(measure_hover::MeasureHoverShowsWhatItWillTake),
        // The Manage-groups window. Beside the two
        // measure checks because it is the third link in the same chain: a
        // tool places a dimension, a window calibrates its group, and this one
        // is where the group comes from.
        Box::new(dimension_groups::DimensionGroupsPanelMakesAGroup),
        // Directly after the markup checks, and the order is a **dependency**
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
        // Directly after it, and the order is a **dependency** rather than a
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
        // Immediately after `page_cache`, and deliberately: the two are the
        // halves of one claim. That one says a page he has SEEN is not drawn
        // twice; this one says a page he has not reached yet is drawn once,
        // early. A run where both fail is a cache that forgets; a run where
        // only this one fails is a band that fills and is evicted.
        Box::new(page_prefetch::PagesAreDrawnBeforeHeScrollsToThem),
        Box::new(page_ops::PageOpsRoundTrip),
        // The FIRST check anywhere that drives the Pages PANEL rather than
        // the Pages tab. `page_ops` above drives the ribbon and records why it
        // does not touch the panel — the tile's context menu is an egui popup
        // that declares no regions. The tiles themselves do declare regions,
        // which is what makes this check possible.
        Box::new(pages_drag::PagesDragShowsWhereItLands),
        Box::new(tab_order_drag::TabOrderDragMovesAFieldAndShowsWhere),
        Box::new(forms_spotlight::ClickingAFormRowLightsTheFieldOnThePage),
        Box::new(display_two_rows::TheDisplayButtonsStackInTwoRows),
        Box::new(title_build_stamp::TheTitleBarCarriesTheBuildTime),
        Box::new(field_shading::FillableFieldsAreShadedOnThePage),
        // The first check to reach a form field's Properties pane without a
        // pointer, through `PDFCER_DIAG_SELECT_FIELD`. Beside `field_shading`
        // because both drive the same fixture family and neither sends input.
        Box::new(option_arrows::TheOptionArrowsAreGreyedOnlyAtTheEndsOfTheList),
        // The canvas half of the same subject, and the one that sends input:
        // it pins the same fixture and ignores `--pdf`, because no other
        // document in the corpus carries a choice field to click.
        Box::new(canvas_choice_fill::ADropDownCanBeAnsweredOnThePage),
        Box::new(preset_group_reachable::TheStandardsPresetsGroupIsReachable),
        Box::new(redact_image_warning::MarkingOverAnImageSaysSoBeforeApply),
        // The DXF export. Beside the page checks because it is the
        // other verb that writes a file the operator hands to somebody else.
        Box::new(embed_fonts::EmbeddingFontsPutsAProgramInTheDocument),
        Box::new(embed_bundled::EmbeddingWorksWithNoFontFolderAtAll),
        Box::new(compact_save::ACompactedCopyIsActuallySmaller),
        // The page-tree guard, from the operator's own report. Placed immediately
        // after `compact_save` because the two are
        // the same subject from opposite ends: that one asserts a save
        // PRODUCES the file it promised, this one asserts a save REFUSES to
        // produce a file it knows is damaged. A reader comparing the two
        // verdicts is reading both halves of "what may leave this program".
        //
        // It pins its own fixture and ignores `--pdf`, so it needs nothing
        // from the sweep's aim table — and it is UNDRIVEN. Its header says so.
        Box::new(pagetree_guard::ASaveThatWouldProduceBlankPagesIsRefused),
        // The signature warning. Beside the save checks
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
        // Insert an image. Its last assertion is the one
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
        // `OPERATOR_REQUESTS.md` O66. Immediately after its
        // sibling, because it depends on everything that one establishes — the
        // picker seam, the window opening, the fixture PNG — and adds exactly
        // one thing: that the window gets out of the way when asked and comes
        // back afterwards. Its real subject is the JOIN; every part of the
        // placement arm is unit-tested and each part passes alone.
        Box::new(insert_image_place::TheInsertWindowStepsAside),
        // `OPERATOR_REQUESTS.md` O67. Beside the insert checks
        // because it is the third route to the same verb — the picker, the
        // dialog, and now a file dropped on the grid — and the one that has to
        // prove a POSITION was used rather than a default.
        Box::new(drop_onto_thumbnails::ADrawingDroppedOnTheThumbnails),
        // `OPERATOR_REQUESTS.md` O70. It reads the same
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
        // Adjacent to that rung deliberately: the chunk boxes are its VISIBLE
        // half. The descent above proves the selection can reach one chunk of
        // text; this proves an operator can see which chunk before committing a
        // gesture to it, which is what `OPERATOR_REQUESTS.md` O215 reports is
        // missing. It also drives a ribbon toggle, so it runs after the checks
        // that establish the ribbon is reachable at all.
        Box::new(text_chunks::TheChunkBoxesShowWhatATextBlockIsMadeOf),
        // Immediately after, because it is the same row's other half and it
        // cannot run without what that one measures: the chunk rung is offered
        // only where a box is drawn. Running it second means a build that lost
        // the boxes reports the boxes, once, rather than reporting a selection
        // defect twice.
        Box::new(chunk_click::ClickingAChunkSelectsThatChunk),
        // Third of the same row, and last of the three for the same reason:
        // it builds a set of chunks, so it cannot mean anything on a build
        // that cannot select one. A failure here on top of a failure above is
        // one defect reported twice; in this order the first row names it.
        Box::new(chunk_multi_move::ShiftClickBuildsAChunkSetTheWholeProgramHonours),
        // Fourth of the same row, and after the Shift-click one for the
        // same reason it came after the single-click one: a band builds a set
        // of chunks, so it cannot mean anything on a build that cannot select
        // one, and its modifier arms are the same arms that check measures on
        // clicks. In this order the first row to fail names the defect.
        Box::new(chunk_band::ARubberBandInsideANoteTakesItsLines),
        // Fifth of the same row, and last of it: it drags a set of chunks, so
        // it cannot mean anything on a build that cannot build one. Placed
        // after the band because it uses a band to build its plural arm.
        Box::new(chunk_ghost::DraggingAChunkShowsWhereItIsGoing),
        // Sixth and last of the same row, and it must run after the one above:
        // that check asserts the painter's account of the preview, this one
        // photographs the preview. Ordered so that a build which draws nothing
        // at all fails the cheaper trace check first and names the defect
        // without a pixel comparison having to.
        Box::new(chunk_ghost_pixels::TheTravellingCopyIsOnTheGlass),
        // O216 ask 1. Beside the chunk row because it is the same unit of
        // selection, and after it because a build that cannot place a caret
        // in a chunk should fail on the caret checks first: this one's
        // failure sentence names a guard, and it is only worth reading once
        // the cheaper checks have ruled out the aim.
        Box::new(chunk_empty::EmptyingAChunkCommitsTheEmptying),
        // `OPERATOR_REQUESTS.md` O71. Beside the Smart-Selector
        // check because both are about what a plain click MEANS — that one in
        // Edit, this one in Read, which is the stance where the answer had
        // always been "text, or nothing".
        Box::new(read_image_copy::ReadModeCopiesAPicture),
        // The Properties panel's document half, wired after a
        // recorded blocker — "`pdfcer-core` exposes no /Info accessor" — turned
        // out to have cleared without the prose moving. Beside the page checks
        // because it is the other surface that edits the DOCUMENT rather than
        // a page's content.
        Box::new(properties_metadata::PropertiesMetadataRoundTrips),
        // The load-anomaly disclosure, both halves.
        //
        // The status-bar one is placed HERE, among the checks that drive a
        // real document, rather than up among the chrome checks that run
        // without a fixture — it pins its own two documents and ignores
        // `--pdf`, but it is still entirely about what a document does to
        // the window.
        //
        // It sends no input at all and puts its window off the desktop, so
        // it is one of the few checks that can run while the operator is at
        // the machine. Do not fuse it with the panel check below on the
        // grounds that they share a subject: the panel one SKIPs under
        // `--no-input`, and a SKIP is not red.
        Box::new(load_anomalies::LoadAnomaliesReachTheStatusBar),
        // The long form of the same disclosure, beside the other check that
        // opens the Document properties panel — they share its opener.
        Box::new(load_anomalies::LoadAnomaliesAreListedInDocumentProperties),
        // The OTHER loader disclosure that lives in the same panel, placed
        // directly under its neighbour because the two are constantly mistaken
        // for each other.
        //
        // `Document::recovery()` and `Document::load_anomalies()` are
        // DISJOINT questions. One is about the cross-reference machinery — the
        // index was unusable and pdfcer rebuilt it by scanning — and the other
        // is about the objects, where a key was defined twice and pdfcer had to
        // pick. The fixtures are authored so that each lights exactly one of
        // them, deliberately: `contradicts-itself.pdf` computes its own xref
        // offsets so it does NOT recover, and the two recovery fixtures have no
        // xref at all so they raise no anomaly. A check reading the wrong one
        // would be green about a surface it never touched.
        //
        // Its control launch is a RECOVERED file rather than a sound one,
        // which is the whole reason a second fixture exists. On a sound file the
        // entire neighbourhood draws nothing, so the absence would be satisfied
        // by a closed panel, by a document that was never recovered, and by the
        // thing under test — three states, one green tick.
        Box::new(recovery_losses::RecoveryLossesAreListedInDocumentProperties),
        // O169's read half, placed here because it shares the Document
        // properties opener with the two above it. Read in order the three say:
        // the panel opens, it discloses what the loader had to decide, and it
        // discloses what KIND of document this is.
        //
        // Like its neighbours it pins its own two fixtures and ignores
        // `--pdf`. The control launch is not optional: without it the check is
        // satisfied by a build that pins a *Stamp collection* heading to every
        // document in the shop.
        Box::new(stamp_collection::AStampCollectionDisclosesItself),
        // O169's AUTHOR half, and it must stay directly under the read half
        // rather than migrating to the export checks.
        //
        // Its final assertion is the read half's surface: it writes a
        // collection, reopens it, and requires Document properties to disclose
        // it. That is only evidence because the reader above was calibrated
        // against `fixtures/stamp-collection.pdf`, which a Python script built
        // from the spec with no pdfcer involved. Delete or disable the check
        // above and this one keeps passing while proving nothing.
        //
        // Read in order the two say: pdfcer can read somebody else's stamp
        // collection, and the ones pdfcer writes are the same kind of thing.
        Box::new(stamp_collection::StampCollectionReachesTheEngine),
        // The way OUT of the same disclosure.
        //
        // Placed immediately after the check that proves the block is drawn,
        // because it depends on that block being drawn and its failure message
        // would otherwise be ambiguous between "no control" and "no block".
        // Read in order, the two say: the rows are there, and the button under
        // them reaches a loader.
        //
        // It is the longest chain any check in this file drives — panel to
        // action to two guards to a pending intent to the loader — and every
        // step of it has a unit test that cannot see the next one.
        Box::new(load_anomalies::RereadingUnderTheOtherValueIsOffered),
        // Two ribbon clicks and one trace line — cheap, no capture, no
        // canvas gesture, no keystroke — so its position is chosen for what a
        // reader wants adjacent rather than for cost.
        //
        // It sits here, among the checks that drive a real document, because
        // its precondition is one: `file.print` is gated on `doc.open`. It
        // must NOT move up among the chrome checks, which run without a
        // fixture.
        //
        // It never presses the commit button, and no future edit may make it
        // do so. That button is the one control in the application that
        // consumes paper and cannot be undone; a harness that can start a
        // print job will eventually start one by accident. The module header
        // states what that costs and why the cost is worth paying.
        Box::new(print_dialog::PrintDialogReachesTheSpooler),
        // Immediately after the spooler check, because the two are the two
        // halves of "does Print work": one asks whether the job reaches the
        // device, the other whether the window the operator drives it from is
        // usable. The gap between those halves is where four defects shipped —
        // see `print_layout`'s header.
        Box::new(print_layout::PrintDialogBodyDoesNotDeadlockItsScrollbars),
        // Immediately after it, and the ORDER is load-bearing rather than
        // tidy. This check's every skip message defers to `print_dialog` for
        // the diagnosis — "the dialog never opened", "the spooler refused",
        // "the ribbon control is missing" are all its subject, not this one's.
        // Running it first means the reader of a failing run meets the
        // specific cause before the vaguer one, instead of reading a paper
        // check skip and having to go looking for why.
        //
        // It never presses Properties…, and no future edit may make it do
        // so. That button opens a VENDOR DRIVER's own modal dialog: a nested
        // Win32 message loop whose layout pdfcer does not know, cannot publish
        // rects for, and cannot reliably dismiss — and one left standing
        // blocks the application's event loop, so a failed dismissal does not
        // fail this check, it hangs every check after it.
        //
        // And it never presses commit, for the reason stated above.
        Box::new(print_paper::PrintPaperChangesThePlan),
        // IMMEDIATELY after `print_paper`, because it drives the same
        // control and every skip it can produce about that control — "the
        // dialog published no `print.paper` region", "the device enumerated
        // no forms" — is answered more specifically by the check above it.
        // A reader of a failing run should meet the general cause first and
        // the special one second.
        //
        // The two are NOT redundant and the difference is worth stating,
        // because they look alike from the roster. `print_paper` clicks a
        // numbered driver form and asserts the plan followed it — that the
        // combo is wired to the job at all. This one clicks the **auto**
        // entry, which is not a sheet but a policy, and asserts that the
        // sheet it produces has something to do with the document's own page
        // sizes. The first would stay green on a build where auto resolved
        // to the first form in the list; that build is exactly the defect
        // this one exists for.
        //
        // It is deliberately fixture-sensitive in one direction only: it
        // asserts an INVARIANT (`matched` implies the page fits) rather than
        // a sheet name, so it holds against any driver on any machine — but
        // a document with no measurable pages makes it SKIP, saying so.
        //
        // It never presses commit, and it never presses Properties… — the
        // rule every print check states in its own words, because the day
        // somebody adds another by copying one of these files, the copied
        // file is the only one they will read.
        Box::new(print_auto_paper::MatchThePagesPicksTheSheetFromTheDocument),
        // After the print checks above it, for the reason the two above
        // give — stated as a POSITION rather than an ordinal, because this
        // comment once read "last of the four", a fifth was added saying "and
        // last", and a sixth falsified both. A relative position stays true
        // however many there are: every skip message it can produce defers to
        // `print_dialog` for the diagnosis — "the dialog never opened", "the
        // spooler refused", "the ribbon control is missing" are its subject.
        // A reader of a failing run should meet the specific cause first.
        //
        // It never presses commit either. Four print checks now state that
        // rule; it is restated rather than referenced because the day somebody
        // adds a fifth by copying one of them, the copied file is what they
        // will read.
        Box::new(print_clip_claim::PrintClipClaimFollowsThePreview),
        // After the other print checks, for the reason they are after
        // `print_dialog`: every skip this one can produce -- the dialog never
        // opened, the spooler refused, the ribbon control is missing -- is
        // somebody else's subject, and a reader of a failing run should meet
        // the specific cause first. It never presses commit either.
        Box::new(print_position::ThePrintedPageCanBeMovedOnThePaper),
        // After the print checks above it, for the reason they give.
        // **NEVER RUN** — registered with the operator at
        // his machine; its own header says so first and says what a first run
        // will probably teach it. It never presses commit, like the four above.
        Box::new(preview_popout::ThePrintPreviewPopsIntoItsOwnWindow),
        // After the print checks above it, for the reason they give: every
        // skip it can produce — "the dialog never opened", "the ribbon control
        // is missing" — is `print_dialog`'s subject, not this one's, and a
        // reader of a failing run should meet the specific cause first.
        //
        // **It WRITES `userdata/preferences.txt`, and it deletes it again
        // on every path out.** It is the only check in the print family that
        // writes an input to the program rather than only reading what the
        // program wrote, so its position carries a second obligation the
        // others do not have. The two other checks that touch that file —
        // `page_display_pref` and `ui_scale` — are both registered far below
        // this point AND both write or delete the file themselves before
        // launching, so neither can be harmed by what this one leaves. That
        // was verified, not assumed — and so was the thing that makes it
        // mostly moot: **the suite sandboxes every check by default** (a
        // private directory, `userdata/` NOT copied in, the whole directory
        // removed afterwards), so under a default run no check can see another
        // check's preferences file at all. The ordering above therefore only
        // bites under `--shared-profile`, or if isolation fails. It is written
        // down anyway, because that flag is what a person reaches for when
        // they are already confused, and "it only matters when something else
        // has gone wrong" describes every ordering rule in this file.
        //
        // It never presses commit. Every print check now states that rule
        // in its own words rather than referencing a neighbour, because the
        // day somebody adds a seventh by copying one of them, the copied file
        // is the only one they will read. The button it must never press is
        // the one control in the application that consumes paper and cannot be
        // undone.
        //
        // It never presses Properties… either — that opens a vendor
        // driver's own Win32 modal, and one left standing does not fail this
        // check, it hangs every check after it.
        //
        // It is TWO launches, not one: a control process to measure what the
        // shipped defaults actually are, then a second against a seeded file.
        // It is therefore among the most expensive checks in the suite, which
        // is the other reason it is not higher.
        // Beside the Print window's twin because it is the same subject one
        // menu along -- does a window open on what the operator last chose --
        // and because a reader comparing the two verdicts is reading one
        // question about six windows.
        //
        // It is also two launches, and for the same reason: the shipped
        // defaults are MEASURED off a control process rather than asserted
        // here. Unlike its twin it needs no input at all, so it is the one
        // of the pair that still runs under --no-input.
        Box::new(export_remembered::TheExportWindowsOpenOnTheSettingsYouLastUsed),
        Box::new(print_remembered::ThePrintWindowOpensOnTheSettingsYouLastUsed),
        // Beside its twin because they are two halves of one subject: that
        // one proves the settings SURVIVE a close, this one proves that WHICH
        // close you took decides whether they should have. Run together they
        // are the whole of O166 and O185; run apart, either reads as complete
        // and is not.
        //
        // Two launches, like its twin, and for a related reason — but not the
        // same one. That check needs a second process to measure the shipped
        // defaults without asserting them; this one needs a second process
        // because the two routes out of the window must be compared against
        // each other, and an assertion both outcomes satisfy measures neither.
        Box::new(print_dismissal::ThePrintWindowForgetsWhatCancelUndid),
        // Beside it because it is the same shape — two ribbon clicks into a
        // dialog — and because both are checks whose subject is a control that
        // was drawn and did nothing.
        //
        // It launches with NO fixture, deliberately: `file.settings` is
        // application-scoped and must work with nothing open. That also makes
        // it the cheapest driving check in the suite, so a run whose ribbon
        // channel is broken says so here without paying for a render.
        Box::new(settings_theme::SettingsThemeTakesEffect),
        // Its sibling, from the same file, and it goes IMMEDIATELY after
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
        // It is the only check that clicks the Airy preset, and the only one
        // that samples the PAGE raster rather than the window body. Both
        // halves matter: Airy is the preset likeliest to fail a contrast
        // assertion, and a theme that tints the paper is a defect no
        // window-body sample can see.
        Box::new(theme_page::EveryThemePresetKeepsThePageWhite),
        // Directly after it, and for the same dependency reason it sits
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
        // Third of the two-process checks, and placed here for the same
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
        // Its complement, added with the fix for the driven
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
        // O186, immediately after it and deliberately so. `deep_zoom` proves
        // the ACTING page survives the pixmap ceiling; this proves nothing is
        // ordered for its NEIGHBOURS there, and then that the rasterizer's own
        // wall stops the zoom and is explained on the bottom bar. A failure in
        // `deep_zoom` should be read first: if the region tier is broken, the
        // strip has no healthy page to be measured against.
        Box::new(raster_wall::TheStripNeverOrdersARasterItCannotFill),
        // O186 clause four, on the dense drawing. LAST of the three, and
        // SKIPPED on any machine without it: the region tier never refuses
        // on sparse line work, so the wall this asserts about cannot be
        // reached with a repository fixture, and measured.
        Box::new(raster_wall::TheRasterWallStopsTheZoomInsteadOfPaintingAnError),
        Box::new(deep_pan::PanningAtDeepZoomStaysWhereItWasPut),
        // O186 stage one, LAST of the deep-tier cluster and deliberately
        // so. Everything above it proves the deep tier draws, rasterizes and
        // pans; this one proves the view cannot be carried off the sheet
        // altogether. A failure in any of the four above should be read first:
        // if the tier does not draw, "the page left the screen" is not this
        // check's subject.
        //
        // It is also the most expensive of the five — a hundred and sixty
        // Ctrl+wheel notches with a settle between every ten — which is the
        // second reason it is last.
        Box::new(off_sheet::AViewCarriedOffTheSheetComesBack),
        Box::new(scale_sweep::MouseWorkSurvivesEveryRenderTier),
        Box::new(zoom_keeps_place::ZoomingDoesNotThrowAwayWhereTheOperatorPanned),
        // Its inverse. The climb above never rolls the wheel the other way, so
        // the DOWNWARD hand-over between the f32 scroll offset and the f64
        // anchor had never been driven. O26e.
        Box::new(zoom_out_keeps_place::ZoomingBackOutKeepsTheView),
        // O28 and O29: a fit sets the scale AND places the view, and there
        // is a third mode. Pans into the pasteboard first, because the state
        // the request is about did not exist before O23.
        Box::new(fit_places_the_view::AFitCommandPutsThePageOnScreen),
        // Immediately after it: same subject, opposite outcome, and a failure
        // in the sibling should be read first because this one builds on it.
        Box::new(fit_left_by_a_pan::APanLeavesTheFit),
        // O177, both halves. Third in the fit group and after both of the
        // single-page ones deliberately: it asserts about a two-page ROW, and
        // a build where a single-page fit is broken would fail here too and
        // say something much less specific about why.
        Box::new(
            page_display_recentres::SwitchingThePageDisplayRecentresAndAFacingFitFitsTheSpread,
        ),
        // O30: the wheel as a page turn, from the status-bar toggle. Asserts
        // the DEFAULT is silent first, so a build that flipped unconditionally
        // could not pass — and that the control is absent where the choice
        // does not exist.
        Box::new(wheel_flips_pages::TheWheelTurnsPagesWhenTheOperatorAsksItTo),
        Box::new(zoom_gallery::ThePageStillRendersAtEveryDecadeOfZoom),
        Box::new(pan_refresh::PanningPastTheOverscanRendersTheNewArea),
        Box::new(resize::ResizeScalesAShape),
        // Directly after `resize`, because it is that check plus one switch:
        // a failure in `resize_scales_a_shape` should be read first, since
        // every link it covers is in front of this one.
        Box::new(scale_switch::TheLineWeightSwitchReachesTheResize),
        Box::new(rotate::RotateHandleTurnsASelection),
        Box::new(shift_constrains::ShiftConstrainsAResize),
        Box::new(geometry_fields::GeometryFieldsResizeAShape),
        Box::new(restyle_text::RestylingSelectedTextReachesTheDocument),
        // Directly after `restyle_text`, because it is that check plus a
        // popup: every link it covers — the sweep, the section, the read-back
        // stamp — is in front of this one, so a failure there should be read
        // first.
        Box::new(std14_face::TheFaceChooserOffersAFaceTheDocumentDoesNotContain),
        Box::new(refused_character_face::ARefusedCharacterOffersAFaceThatCanTypeIt),
        Box::new(font_group::TheFormatTabOffersFontControlsForSweptText),
        // Immediately after its twin, and the adjacency is the point: the two
        // assert the same two surfaces, one on a pinned fixture and one on
        // whatever `--pdf` names. A reader comparing their two lines in a sweep
        // report gets the diagnosis for free. SKIPs without `--pdf` and
        // `--doc-point`, deliberately — it has no fixture to fall back on.
        Box::new(font_group_real::TheFontControlsAreLiveOnTheDrawingYouOpen),
        // After `font_group`: same state, and that one asserts the sentence
        // while this asserts the control. Read a sentence failure first.
        Box::new(colour_clicked_text::ClickingTextOffersItsColour),
        Box::new(multi_node::MultiNodeMoveMovesEveryPickedAnchor),
        Box::new(shape_preview::DraggingANodeBendsTheLine),
        // Immediately after `shape_preview`, and the order is the diagnosis
        // order: that one asks whether a preview is built and painted AT ALL,
        // this one asks how WIDE it was painted. A build where nothing is
        // drawn fails both, and reading the width failure first would send a
        // reader after a stroke rule when the preview never reached the
        // painter.
        Box::new(preview_width::ADragPreviewDoesNotThickenWithZoom),
        Box::new(bezier_handle::BezierHandleDragChangesACurve),
        // The three deeper-rung deletes. The first two share `bezier_handle`'s
        // fixture; the third needs a text object holding SEVERAL runs and SKIPs
        // without one, because the property it asserts is that deleting one
        // label leaves the other runs of the same object intact — which a
        // single-run object cannot witness.
        Box::new(deeper_rung_delete::DeletingALineLeavesTheRestOfTheShapeAlone),
        Box::new(deeper_rung_delete::DeletingAPointLeavesTheRestOfTheLineAlone),
        Box::new(deeper_rung_delete::DeletingALabelLeavesTheOtherLabelsAlone),
        Box::new(deeper_rung_delete::DeletingAClickedChunkLeavesTheRestOfTheBlockAlone),
        // Immediately after the label delete, and deliberately: it stands on
        // the SAME rung of the SAME fixture, reached by the same chord and the same
        // click, and differs only in the gesture that follows. So a failure here
        // with the three above green is a statement about the MOVE path alone,
        // while a failure of all four is a statement about the Part rung on text —
        // which is the first fork a reader needs and is free if the order holds it.
        Box::new(move_line_of_text::DraggingOneLineOfTextMovesItOrSaysWhy),
        // And immediately after the refusal, because the two are the two
        // halves of O188 and they are ordered the way the operator met them.
        // `move_line_of_text` asserts that the gesture he TRIED is refused out
        // loud; this one asserts that a route he could have found BEFORE trying
        // exists in the menu. A failure of this one alone says the capability
        // works and remains undiscoverable — which is the whole of O188(A), and
        // is invisible to every other check in this file.
        //
        // It stands on the same rung of the same fixture as its neighbour and
        // reaches it a different way, so a failure of BOTH is a statement about
        // the Part rung on text rather than about either route.
        Box::new(run_menu_route::TheRightClickOffersTheLineYouClicked),
        Box::new(autosize_overflow::AFieldTooSmallForItsTextSaysSo),
        Box::new(tool_row::TheTextToolTypesOnOneClick),
        Box::new(tool_row::AClickOnBlankPaperStartsNewText),
        Box::new(tool_row::ThePointsToolShowsPointsOnOneClick),
        Box::new(tool_row::ShowPointsDrawsAnObjectsPointsWithoutDescending),
        Box::new(dropped_file::ADroppedImageReachesThePlacementWindow),
        Box::new(first_frame::TheFirstFrameNamesTheArmedTool),
        Box::new(master_detail::TheInspectorIsOneMasterDetailColumn),
        // The left rail — `OPERATOR_REQUESTS.md` O123 part 7 and O126.
        // Written and NOT executed: the operator was at his
        // keyboard and a watchdog kills GUI processes on sight. Registered
        // anyway, because a check that is not in the list is a check nobody
        // will ever run.
        Box::new(left_rail::TheLeftRailIsReachableAndConstantWidth),
        Box::new(properties_tool::TheArmedToolsSettingsAreInProperties),
        // O119 — registered without having been run: an unregistered check
        // is one nobody will ever run.
        Box::new(protect::ProtectShowsTheDocumentAndRefusesASignedOne),
        // Signing — from the operator's report.
        // Beside `protect` and `redaction` because the three share a subject
        // (what pdfcer writes into a file that is about the file rather than
        // about a page) and because this check reads the other two's fixtures.
        //
        // It is the longest check in the suite — four processes, roughly
        // twenty clicks — and it is that long because its verdict cannot be
        // taken in the process that produced it. See its header.
        Box::new(signing::ADocumentCanBeSignedAndTheSignatureIsInTheFile),
        Box::new(redaction::RedactionRemovesAndProvesIt),
        // Immediately after `redaction`, which it shares a fixture generator
        // with. Read in this order the two answer the same question from
        // opposite ends: `redaction` asks whether the words left the file,
        // this asks whether the operator was told which words they were
        // while cancelling was still possible.
        Box::new(redact_preview::TheApplyReportListsTheTextItWillDestroy),
        // Immediately after `redaction`, and the order is load-bearing in one
        // direction: this check writes a preference into the profile and
        // restores it on the way out, so a failure to restore is cheapest to
        // diagnose while the check that shares its surface is still fresh in
        // the report above it. It also types, which `redaction` does not, so
        // the two together cover both routes into the marking panel.
        Box::new(redaction_reach::TheRedactionReachSettingDecidesWhatSurvives),
        // Beside the text-editing checks and owning its own fixture, like
        // `text_edit` and `redaction` above: its verdict is a LINE COUNT that
        // only `fixtures/paragraph.pdf` produces, so it takes no `--pdf`.
        Box::new(reflow::ReflowingAParagraphRewrapsIt),
        // O127 defect 2 — and registered WITHOUT having been run, on the
        // precedent `left_rail`, `properties_tool` and `protect` above set: a
        // check that exists and is not registered is a check nobody will ever
        // run. Whoever runs the suite next is the first thing that executes it.
        Box::new(enter_newline::EnterMakesASecondLineAndControlEnterCommits),
        // O140, and it sits here rather than beside `text_edit_real`
        // deliberately: it is the same family and the OPPOSITE question. That
        // check asks whether the shell can place a caret and reach the engine
        // on a real drawing; this one starts from a commit the engine is
        // certain to refuse and asks only what the operator is told about it.
        // Running it after `enter_newline` puts the two keystroke-level text
        // checks together and both after the reflow, which is the order a
        // reader debugging "text editing does not work" wants: can it commit,
        // can it break a line, can it explain itself.
        Box::new(typo_refusal::HisTypoCanBeCorrectedOnHisOwnFile),
        // Directly after `redaction`, and before the two selection checks,
        // because it is the second most expensive check in the suite — it
        // launches the binary twice, for the same reason `save_copy` does — and
        // because its subject is the one this project exists for. A run in which
        // `save_copy` failed should be read first: every link from Ctrl+S
        // onwards is that check's, and this one has the whole text-edit path
        // stacked in front of them.
        // The multi-document pair. Both SKIP without
        // `--second-pdf`, and the SKIP reason says why the same file cannot be
        // passed twice.
        Box::new(document_tabs::TwoDocumentsGetTwoTabs),
        Box::new(tab_reorder::DocumentTabsCanBeRearranged),
        Box::new(panel_tab_reorder::PanelTabsCanBeRearranged),
        Box::new(page_drag_between_documents::PageDraggedBetweenDocuments::COPY),
        Box::new(page_drag_between_documents::PageDraggedBetweenDocuments::MOVE),
        Box::new(about::AboutReportsTheBuild),
        // Beside About because it is its neighbour in every way that matters to
        // a run: it launches with NO document, it opens one window, and it
        // asserts on a trace rather than on pixels. It is also cheap.
        Box::new(shortcuts::ShortcutsReferenceIsLive),
        Box::new(band_scroll::ACommandTwoScrollStopsAwayIsStillReachable),
        Box::new(block_nav::ArrowKeysWalkBetweenBlocks),
        Box::new(dialog_windows::DialogsOpenInTheirOwnWindow),
        Box::new(draft_selection::ShiftArrowsSelectText),
        Box::new(bookmark_add::BookmarkCanBeWritten),
        // Immediately after its sibling, because they are one assertion in
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
        // Its opposite number: `comment_note` proves a
        // comment can be WRITTEN, and this one proves one can be READ — in
        // **Read mode**, where until that date there was no route to a note's
        // words at all. The operator's report is the check's own defect string.
        // NOT RUN; the module says so in its header.
        Box::new(comment_popup::ACommentCanBeReadOnThePageInReadMode),
        // O173. It launches TWICE and deletes a file the
        // sandbox wrote, which no other check does — both are explained at
        // length in its header, and neither is optional: the seed exists so
        // this offer does not open in front of the other two hundred checks,
        // and the second launch is the only place "ask once" can be observed.
        // DRIVEN: PASS. The offer opened, its action, decline and
        // checkbox were all declared, the answer reached the profile's
        // preferences file, and the second launch did not ask again.
        Box::new(default_app_offer::TheDefaultAppOfferIsAskedOnce),
        // Last of the three new ones and the most expensive: it drives Insert
        // pages, the Forms panel and the Tab-order section in one session,
        // because the shape it registers does not exist in any fixture — pdfcer
        // makes it. See that module's header.
        Box::new(adopt_widget::AdoptWidgetPutsAFormControlBack),
        Box::new(add_text::AddTextTakesRealKeystrokes),
        Box::new(chords::EveryDeclaredChordDispatches),
        Box::new(stamp_size::StampSizeReachesTheEngine),
        // Adjacent to `stamp_size` deliberately: the two are the AUTHORING
        // and RESTYLE halves of one operator report, and reading either without
        // the other leaves the impression that a sentence he wrote twice is
        // covered when half of it is. This one places its own stamp first, so it
        // depends on no fixture carrying one.
        // DRIVEN: PASS, and falsified afterwards. The label-size
        // row is drawn for a placed stamp, seeded from the file at 28 pt, and
        // reads back 30 pt after the edit — typed, written and read back.
        Box::new(stamp_size_properties::StampSizeInThePropertiesBox),
        // O171. Immediately after `stamp_size` because it
        // repeats that check's route and then does the thing no other check
        // does: opens the same dialog a SECOND time. The operator's report was
        // that the first opening was fine and the second had its Add and
        // Cancel below its own bottom edge, so a suite of checks that each
        // open it once is structurally blind to it.
        // DRIVEN: PASS — both stamp dialogs have Add and Cancel on
        // the screen, so O171's report is closed by measurement rather than by
        // reading the layout code.
        //
        // Its FIRST driven run failed, and the defect was in the check: it
        // clicked Markup > Stamp before each placement, and that ribbon item is
        // a toggle, so the second click put the armed tool DOWN. `arm_stamp`
        // now reads the tool's state before acting. A driven failure is a claim
        // about the check too.
        Box::new(stamp_dialog_reopen::TheSecondStampDialogStillHasItsButtons),
        // O172 — *"add our own custom stamps and use them"*.
        // Beside the two checks above because it repeats their route as far as
        // the dialog and then does the thing neither does: reads the gallery's
        // CUSTOM half, presses one of the operator's own stamps, and follows
        // the artwork all the way onto the page.
        //
        // It plants `fixtures/stamp-collection.pdf` into a scratch
        // `%APPDATA%` tree and redirects the child process's APPDATA at it.
        // The obvious alternative — read his real Acrobat folder, SKIP when
        // empty — is worthless on any other machine and goes vacuous on his
        // own the day he deletes a stamp. A SKIP is not red.
        Box::new(custom_stamp::CustomStampReachesThePage),
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
        // Directly after the sweep it depends on. This one asserts EXACT
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
        // Last of the driving checks, and the slowest: it runs a real
        // recognition, which is a second in a release build. Placed after the
        // cheap ones so a run that is going to fail on something structural
        // fails before spending it.
        // O58's discharge. It runs AFTER the selection checks
        // deliberately: its first three phases are `field_menu`'s (place,
        // clear, select), so if those are broken this check should not be the
        // first thing to say so — it would name the clipboard for a selection
        // defect. Its own SKIP messages distinguish the two.
        // O59's first item, before the clipboard pair: it is the one whose
        // failure is DESTRUCTIVE. The other two prove a capability works; this
        // proves one cannot happen.
        Box::new(cut_gate::CuttingARedactionMarkIsRefusedBeforeAnythingIsRemoved),
        // O60 — redacting what is selected, the third marking route.
        Box::new(redact_selection::ASelectedObjectCanBeMarkedForRedaction),
        // O217, and it must run AFTER the row above: that one establishes
        // that a selected object can be marked at all, and this one asks a
        // narrower question about the SAME verb — which unit it addresses
        // and whether the canvas can reach it. In this order a build that
        // lost redaction entirely reports redaction once, rather than
        // reporting a chunk defect on top of it.
        Box::new(redact_chunk::RedactingAClickedChunkMarksOnlyThatChunk),
        // O61 — the document-safety disclosure.
        Box::new(reach_out::ADocumentThatPhonesHomeSaysSo),
        // O62 — the rotation direction, which is one sign and invisible.
        Box::new(widget_rotate::TurningAFieldRightTurnsItRight),
        // O59 item 2 — the page clipboard.
        Box::new(page_clipboard::PagesCanBeCopiedAndPasted),
        // O59 item 3 — the bookmark clipboard, and the one operation in
        // this program Acrobat cannot do between two files at all.
        Box::new(bookmark_clipboard::ABookmarkSubtreeCanBeCopiedAndPasted),
        Box::new(field_clipboard::AFormFieldCanBeCopiedAndPastedBothWays),
        // Immediately after its sibling, and it is the MIRROR of it: same
        // gestures, opposite expectations, one environment variable apart. A
        // build that ignored the paste-order setting passes the first and fails
        // this one on its first assertion.
        Box::new(field_clipboard::TheAcrobatPasteOrderSwapsWhichChordDoesWhich),
        // Before the click and tab checks because it is the gesture that comes
        // first: he arms a tool and looks at the page before he commits to a
        // placement. It runs on the shared fixture and aim point.
        Box::new(form_ghost::TheOutlineFollowsThePointerAndIsWhatGetsPlaced),
        Box::new(form_selection::AClickInsideAFormSelectsWhatIsDrawnThere),
        // Straight after the click check, and the order is the argument this
        // feature makes: a click puts the keyboard IN a field, and Tab is what
        // he does next. A run where both fail is a form surface no gesture
        // reaches; a run where only this one fails is the press reaching egui
        // before the canvas sees it, which is O204's whole subject.
        //
        // It opens its own document. Not one form fixture in the engine corpus
        // carries a text field with a drawn appearance, and an undrawn field is
        // not on the canvas to click.
        Box::new(tab_navigation::TabMovesBetweenFormFields),
        // Immediately after it, and the adjacency is the point: both are
        // "what does a click on the canvas mean?", read through two different
        // off-canvas oracles. A run where both fail says the click is not
        // arriving; a run where only this one fails says the click arrives and
        // the layer relation is broken. Ordering them apart would make that
        // difference unreadable in the summary.
        //
        // **NOT RUN.** Written with the operator possibly at his
        // machine; it has never seen a running binary. See its module header.
        Box::new(layers_membership::SelectingAnObjectNamesItsLayer),
        // Immediately after it, and the order is a dependency rather than a
        // preference: this check's second step is `form_selection`'s first
        // assertion — a click inside a form must select the leaf — and it
        // SKIPs rather than fails when that does not hold, so that a broken
        // deep hit test is reported once, by the check that owns it, instead of
        // twice with the second reading blaming the wrong file.
        Box::new(unshare_form::TheContextMenuGivesThisPageItsOwnCopyOfASharedForm),
        // Its pair, and it must stay adjacent to it. The two press the same
        // row through the same five steps and differ only in the document they
        // open — `shared-across-two-pages.pdf` against `page-sized-form.pdf` —
        // so they are one behaviour's two halves, not two features. Reading the
        // suite output, an author who sees only one of them run has been told
        // that the other half of a branch is unmeasured, which is exactly the
        // state that let a check named `…_of_a_shared_form` sit on an unshared
        // fixture for a day.
        //
        // It costs a second launch and a second window, deliberately: the
        // condition under test is a property of the open file, so it cannot be
        // reached by any further gesture within the first check's session.
        Box::new(unshare_form::TheUnshareDeclinesWhenNothingElseDrawsTheForm),
        // The two link checks, before OCR: they are two launches and two clicks
        // apiece against kilobyte fixtures, where the OCR checks below are a
        // minute each. A run that fails on something cheap should fail before
        // paying for something expensive.
        Box::new(off_page_marquee::ABandDraggedIntoTheMarginReachesAnObjectOffThePage),
        // Immediately after its sibling, and deliberately: they share one
        // fixture and one ribbon route, so a failure in both at once names the
        // harness and a failure in one names the feature.
        Box::new(off_page_press::ABandThatStartsInTheMarginReachesAnObjectOffThePage),
        // Third of the off-page group, and last of the three on purpose: it is
        // the only one that takes a screenshot, and a pixel oracle is worth
        // nothing until the two trace-level siblings have said the object is
        // there to be painted.
        Box::new(off_page_visible::AnObjectOffThePageIsActuallyDrawn),
        // Fourth and last of the off-page group, and the most expensive: it
        // takes a screenshot AND rolls the wheel a dozen-odd notches with a
        // settle after each. Placed after its three siblings so that a build
        // where the object cannot be reached, or is not painted at all, says
        // so before this one spends a minute proving it is also not zoomable.
        Box::new(off_page_zoom::AnObjectOffThePageSurvivesBeingZoomedInOn),
        // Fifth and last of the off-page group. Deliberately AFTER the four
        // that prove the object can be reached, painted and zoomed: this one
        // asserts the census FINDS it, and a census that reports nothing on a
        // fixture whose object is not actually there would be a true answer
        // reported as a defect. Its siblings running first is what makes its
        // `objects >= 1` assertion mean something.
        Box::new(off_page_census::TheOffPageCensusFindsTheObjectAndMarksIt),
        // Sixth and by far the most expensive of the off-page group:
        // FIVE launches, five maximizes and five screenshots, against one
        // shared profile. Last of the group deliberately — every rung of it
        // rests on the object being reachable and painted, which is exactly
        // what the five above establish. If they are red, this one's report
        // about a preference would be a true statement about the wrong thing.
        Box::new(off_page_toggle::TheOffPageToggleIsPerModeAndRemembered),
        // Two launches and an Alt+F4, so it is placed with the other
        // multi-process checks rather than among the single-window ones.
        Box::new(page_display_pref::APageDisplayChoiceSurvivesACloseAndReachesANewDocument),
        // Immediately after its opposite number, and the pairing is the
        // point: both are `a preference survives a close`, and they close the
        // window in deliberately different ways. `page_display_pref` presses
        // Alt+F4 because its write is debounced and needs the exit hook; this
        // one KILLS all three of its processes because O187's write must not.
        // Reading them side by side is how a future session learns that the
        // close style is an assertion rather than a convenience.
        Box::new(page_previews_pref::ThePagePreviewLimitIsRememberedAndZeroMeansNever),
        Box::new(save_as::SaveAsRebindsTheDocument),
        // Last of the new group: it is TWO launches and it presses Alt+F4,
        // so a run that fails on something cheaper should fail first.
        Box::new(quit_unsaved::ClosingTheProgramAsksBeforeLosingUnsavedWork),
        Box::new(link_follow::ALinkGoesToThePageItNames),
        Box::new(link_follow::ALinkItCannotFollowSaysSo),
        // The pair that enforces D47, and they are a pair: the first is the
        // control and the second is the witness, so a run that reports one
        // without the other has measured half a conditional.
        Box::new(point_destination::APointDestinationLeavesTheMagnificationAlone),
        Box::new(point_destination::APointDestinationOffScreenMovesTheHorizontal),
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
        // Immediately after `find_bar`, and the order is load-bearing
        // rather than tidy. Both of these press Ctrl+F and type into the
        // same field, so if that gesture is broken the neighbour above
        // names it in one sentence and these two report a SKIP off their
        // own control probe. A reader scanning a run should meet the
        // general failure before the two specific ones it explains.
        //
        // They are also the two most expensive checks in the suite:
        // FOUR launches between them, because each asserts an ABSENCE and
        // an absence needs a control launch that produced the presence.
        // See the module header.
        Box::new(find_options::ATrailingBlankDoesNotChangeWhatASearchFinds),
        Box::new(find_options::ZoomOffHoldsTheViewOnAFindJump),
        // Second to last among the driving checks, and the placement is a
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
        // Immediately BEFORE its neighbour, and the two are the two halves
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
        // NOT RUN against the binary by whoever wrote it; its own header
        // says so in its first section.
        Box::new(read_mode_exit::ReadModeSaysHowToGetBackOut),
        Box::new(read_mode_chrome::ReadModeHidesTheChrome),
        Box::new(settings_headings::SettingsHeadingsLegible),
        // LAST, and for a reason that is the mirror of the one above.
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
