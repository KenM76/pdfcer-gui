//! `checks::roster` — **which checks exist, and the order the suite runs
//! them in.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/roster.md`.

use super::*;

mod from_the_text_rung;

/// Every check, in the order the suite runs them.
#[must_use]
pub fn all() -> Vec<Box<dyn Check>> {
    let mut checks = up_to_the_text_rung();
    checks.extend(from_the_text_rung::all());
    checks
}

/// From the first check through the descent into placed drawings.
fn up_to_the_text_rung() -> Vec<Box<dyn Check>> {
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
        Box::new(page_scale::ScalingASheetScalesTheDrawing),
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
        Box::new(cloud_border::APlacedSquareCanBeMadeCloudy),
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
        // The other render approximation the status bar discloses; it opens
        // its own fixture, so it needs no --pdf.
        Box::new(spots_flattened::ExtraSpotInksAreDisclosed),
        Box::new(field_outline::DefaultFieldIsOutlined),
        Box::new(text_render_mode::ChoosingARenderModeReachesTheDocument),
        Box::new(run_width::TypingARunWidthReachesTheDocument),
        Box::new(merge_runs::MergingTextRunsReachesTheDocument),
        Box::new(split_lines::ATextObjectSplitsIntoLines),
        Box::new(repair_form_fonts::RepairingFormFontsReachesTheDocument),
        Box::new(ocr_layer_view::OcrLayerIsShownAndBlended),
        Box::new(ocr_colour_setting::OcrColourIsReadAndReset),
        Box::new(off_page_blank_overhang::BlankOverhangIsCountedNotListed),
        Box::new(remove_ocr::RemovingOcrTextReachesTheDocument),
        Box::new(export_keep_text::KeepTextDecidesTheSvg),
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
        Box::new(markup_on_page::MarkupDrawsIntoPageContent),
        Box::new(measure_perimeter::MeasurePerimeterTracesAndCloses),
        // Immediately after it, and the ordering is a dependency rather
        // than a preference: this check DRAWS a perimeter before it reshapes
        // one, so if the tracing gesture is broken the run should say so under
        // the name of the check whose subject that is. Its own early steps
        // decline with "run that one first" for the same reason.
        //
        // REGISTERED AND NEVER RUN against the binary. See its module header.
        Box::new(dimension_corner_count::ACornerCanBeAddedAndTakenAway),
        Box::new(dimension_label_drag::ADimensionDragPreviewsWhatItPlaces),
        Box::new(dimension_extension_grip::AnExtensionLineGripShortensItsLine),
        Box::new(dimension_text_slide::ADimensionTextSlidesAlone),
        Box::new(dimension_circular_label_drag::ARadiusLabelDragSwingsTheLeaderToTheDrop),
        Box::new(dimension_display_menu::ARadiusSwitchesToADiameterFromItsRightClickMenu),
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
        Box::new(group_unit_converts::ChangingAGroupsUnitKeepsItsRealLengths),
        Box::new(dimension_tool_options::AHorizontalDimensionMeasuresOnlyTheRun),
        Box::new(dimension_tool_options::ADimensionJoinsTheGroupMadeInToolOptions),
        Box::new(dimension_tool_options::ANewGroupCopiesAGroupsScaleInItsOwnUnit),
        Box::new(dimension_text_select::ClickingADimensionsTextSelectsIt),
        Box::new(draft_arrows::ArrowsInATextDraftStayWithTheCaret),
        Box::new(draft_paragraph::AParagraphReopensAsOneDraft),
        // Immediately after the check whose gesture it extends, and the order
        // is a dependency: this one drives the same two-point calibration and
        // then asks what the window did with it. If the pick itself is broken,
        // the diagnosis belongs to the line above and this should read as its
        // consequence.
        Box::new(scale_reads_the_group::SetScaleReadsTheGroupItIsAboutToOverwrite),
        Box::new(scale_ratio_units::ScaleRatioReadsAsTheDrawingStatesIt),
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
        Box::new(page_tabs_chooser::PageTabsChooser),
        Box::new(forms_spotlight::ClickingAFormRowLightsTheFieldOnThePage),
        Box::new(password_fill::ATypedPasswordIsNotSavedUnlessAsked),
        Box::new(layer_authoring::ALayerCanBeMadeRenamedAndDeleted),
        Box::new(layer_combine::LayersCanBeMergedAndFlattened),
        Box::new(layer_folders::LayerFoldersShowAndReorganise),
        Box::new(layer_assign::LayerAssignMovesTheSelection),
        Box::new(layer_assign::PasteGoesOnTheCurrentLayer),
        Box::new(layer_assign::ACaretGoesOnTheCurrentLayer),
        Box::new(layer_assign::ADrawingGoesOnTheCurrentLayer),
        Box::new(attach_file::AFileAttachesAsAMarker),
        Box::new(open_many::SeveralDocumentsOpenFromOneOpen),
        Box::new(security_tab::TheSecurityTabHoldsSecurityAndProtect),
        Box::new(security_tab::TheSnapshotToolIsOnTheLeftRail),
        Box::new(remove_ocr_pages::RemoveOcrTextTakesThePagesChosen),
        Box::new(straighten_scans::StraightenScansTurnsTiltedPages),
        Box::new(thumbnail_zoom::ThumbnailsZoomWithoutBlanking),
        Box::new(cross_window_paste::ASelectionCopiedInOneWindowPastesInAnother),
        Box::new(window_move::ADocumentMovesBetweenWindows),
        Box::new(tab_drag::ATabDraggedOutMovesItsDocument),
        Box::new(selection_drop::ASelectionDroppedOnAnotherWindowIsCopiedThere),
        Box::new(remove_metadata::RemoveMetadataTakesOnlyTheEntriesTicked),
        Box::new(model_axes::AModelOpensFromThePageAndTurnsAboutTheChosenUp),
        Box::new(attach_sound::ASoundAttachesAsAnIcon),
        Box::new(media_clip::AClipPlaysFromARegion),
        Box::new(field_scripts::AFieldIsCalculatedFromOthers),
        Box::new(field_extras::ATextFieldsExtrasReachTheFile),
        Box::new(inherited_alignment::AFieldsAlignmentCanGoBackToInherited),
        Box::new(check_mark::ACheckBoxsMarkCanBeChosen),
        Box::new(check_mark::ARadioButtonsMarkCanBeChosen),
        Box::new(new_radio_mark::ANewRadioButtonsMarkIsChosen),
        Box::new(new_radio_mark::ANewCheckBoxsMarkIsChosen),
        Box::new(widget_dash::AWidgetBordersDashCanBeChosen),
        Box::new(choice_defaults::AMultiSelectListsDefaultsCanBeChosen),
        Box::new(annot_flags::AnAnnotationCanBeHiddenAndShownAgain),
        Box::new(print_shop::PrintShopFindingsAndTinyDetails),
        Box::new(print_shop::InkPickerReadsOverprint),
        Box::new(security_notes::SecurityNotesNameTheCoverAndTheActions),
        Box::new(rc4_append::Rc4EditsWaitForTheOperatorAndSayWhatTheyCost),
        Box::new(encrypted_edit::AnEncryptedFileIsEditedUnderThePasswordItOpenedWith),
        Box::new(insert_text::ACaretMarksAnInsertion),
        Box::new(replace_text::ReplacingTextStrikesAndCarets),
        Box::new(markup_flatten::AMarkupCanBeMadePartOfThePage),
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
        Box::new(revocation_sources::SignatureNamesWhereRevocationLives),
        Box::new(revocation_verdict::SignatureSaysItsRevocationVerdict),
        Box::new(os_fonts_setting::FontFoldersLandsOnTheFontsSetting),
        Box::new(unembed_fonts::RemovingEmbeddedFontsReachesTheDocument),
        Box::new(export_form_data::ExportingFormDataWritesAFile),
        Box::new(export_dxf::ExportDxfWritesThePagesGeometry),
        Box::new(export_dxf_options::ExportDxfWritesTheVersionAndScaleChosen),
        Box::new(export_dxf_pages::ExportDxfWritesOneFilePerPage),
        Box::new(export_image_standard::ExportImageDrawsTheChosenBackgroundAndStandard),
        Box::new(tabs_look::TabsReadAsTabs),
        Box::new(system_theme::SystemThemeFollowsTheHost),
        Box::new(small_first_page::ASmallFirstPageIsTheCurrentPage),
        Box::new(export_image_emf::ExportImageWritesAMetafile),
        Box::new(copy_as_vector::CopyAsVectorPlacesTheMeasuredOrder),
        Box::new(copy_as_vector_scripted::CopyAsVectorWithoutTheMouse),
        Box::new(insert_drawing::InsertImagePlacesADrawing),
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
        Box::new(form_node_move::AnEndPointInsideAWrappedDrawingCanBeDragged),
        Box::new(form_node_move::ALineInsideAPlacedDrawingShowsItsProperties),
        Box::new(form_part_copy::CopyingAPartOfAPlacedDrawingSaysWhyNothingWasCopied),
        Box::new(form_part_delete::ASubpathInsideAPlacedDrawingCanBeDeleted),
        Box::new(form_part_delete::AnAnchorInsideAPlacedDrawingCanBeDeleted),
        Box::new(form_part_delete::ATextLineInsideAPlacedDrawingCanBeDeleted),
        // …and one rung deeper again. Third of the three, in the order an
        // operator meets them: reach it, edit it, go inside it.
        Box::new(form_leaf_descend::TheLadderGoesAsDeepInsideAContainer),
    ]
}
