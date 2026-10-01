//! Design and rationale: `docs/modules/pdfcer-gui-base/opendoc/tests.md`.
#![cfg(test)]
//! # `opendoc::tests` — the document record's own assertions
//!
//! The inner `#![cfg(test)]` at the top is **load-bearing beyond the
//! compiler**. `check-ui-strings.sh` recognises that exact attribute as "this
//! whole file is out of the shipped binary" and stops reporting its assertion
//! messages as operator copy — matched on the attribute rather than on the
//! filename, because the property that earns the exemption is *not in the
//! binary* and a filename is a restatement of that which goes stale. Remove it
//! and every assertion message here is reported as untranslated operator copy,
//! which is how a report gets trained out of being read.

use super::*;

// Named here rather than left to the `use super::*;` glob, so this file's
// dependencies are its own and survive the parent's import list changing.
use std::collections::BTreeSet;

use pdfcer_core::object::ObjId;

// =======================================================================
// The staleness keys
// =======================================================================

/// **Every input that changes the picture changes the render key.**
#[test]
fn every_view_input_that_changes_the_picture_changes_the_render_key() {
    let mut doc = open_fixture(PAINTED_LAYERS);
    let base = doc.render_key(2.0);

    assert_ne!(base, doc.render_key(2.5), "the raster scale");

    doc.view.page_index = 1;
    assert_ne!(base, doc.render_key(2.0), "the page");
    doc.view.page_index = 0;

    doc.set_annotations_visible(false);
    assert_ne!(base, doc.render_key(2.0), "annotation visibility");
    doc.set_annotations_visible(true);
    assert_eq!(base, doc.render_key(2.0), "…and back again");

    doc.set_layer_visible(ObjId::new(5, 0), true);
    assert_ne!(base, doc.render_key(2.0), "the layer override");
}

/// **A layer or annotation change is DISCRETE, not debounced.**
#[test]
fn a_layer_or_annotation_change_commits_at_once_rather_than_settling() {
    let mut doc = open_fixture(PAINTED_LAYERS);
    let before = doc.render_key(2.0);
    doc.set_layer_visible(ObjId::new(5, 0), true);
    let after = doc.render_key(2.0);
    assert_ne!(after.discrete_inputs(), before.discrete_inputs());
    assert_eq!(
        after.scale_bits(),
        before.scale_bits(),
        "a layer toggle must not look like a zoom, or it inherits the debounce"
    );

    doc.set_annotations_visible(false);
    let hidden = doc.render_key(2.0);
    assert_ne!(hidden.discrete_inputs(), after.discrete_inputs());
    assert_eq!(hidden.scale_bits(), after.scale_bits());
}

/// **"Obey the document" and "hide nothing" are different renders.**
#[test]
fn obeying_the_document_is_not_the_same_as_hiding_nothing() {
    let mut doc = open_fixture(PAINTED_LAYERS);
    assert!(
        doc.layer_visibility().is_none(),
        "a freshly opened document obeys its own configuration"
    );

    doc.set_hidden_layers(BTreeSet::new());
    let showing_all = doc.layer_visibility().expect("an override is in force");
    assert_eq!(showing_all.hidden_count(), 0);

    doc.reset_layers();
    assert!(
        doc.layer_visibility().is_none(),
        "reset must restore `None`, not an empty override"
    );
}

/// **The first toggle starts from the DOCUMENT's answer, not from
/// nothing.**
#[test]
fn the_first_layer_toggle_seeds_from_the_documents_own_defaults() {
    let mut doc = open_fixture(PAINTED_LAYERS);
    let defaults = doc.hidden_layers();
    assert_eq!(
        defaults.len(),
        2,
        "this fixture must declare layers that are OFF by default, or the \
         seeding path is untested: {defaults:?}"
    );

    doc.set_layer_visible(ObjId::new(4, 0), false);
    let hidden = doc.hidden_layers();
    assert!(
        hidden.contains(&ObjId::new(4, 0)),
        "the operator's own change"
    );
    for id in &defaults {
        assert!(
            hidden.contains(id),
            "the document's own OFF set must survive the first toggle, or \
             hiding one layer reveals every hidden one: {hidden:?}"
        );
    }

    doc.set_layer_visible(ObjId::new(5, 0), true);
    let hidden = doc.hidden_layers();
    assert!(!hidden.contains(&ObjId::new(5, 0)));
    assert!(hidden.contains(&ObjId::new(6, 0)), "and only that one");
}

/// **Every change to the override moves the generation.**
#[test]
fn every_layer_mutation_moves_the_generation() {
    let mut doc = open_fixture(PAINTED_LAYERS);
    assert_eq!(doc.layers.generation, 0);
    doc.set_layer_visible(ObjId::new(5, 0), true);
    assert_eq!(doc.layers.generation, 1);
    doc.set_hidden_layers(BTreeSet::new());
    assert_eq!(doc.layers.generation, 2);
    doc.reset_layers();
    assert_eq!(doc.layers.generation, 3);
}

/// **A view toggle is not an edit.**
#[test]
fn hiding_annotations_or_a_layer_is_not_an_edit() {
    let mut doc = open_fixture(PAINTED_LAYERS);
    let _ = doc.page_objects();
    let _ = doc.font_inventory();

    doc.set_annotations_visible(false);
    doc.set_layer_visible(ObjId::new(4, 0), false);

    assert_eq!(doc.edit_epoch, 0, "no content changed");
    let key = doc.page_objects.built_for.get();
    let _ = doc.page_objects();
    assert_eq!(
        doc.page_objects.built_for.get(),
        key,
        "hiding annotations or a layer changes no content, so the decomposition must be reused"
    );
    assert_eq!(doc.fonts.built_for.get(), Some(0));
}

// =======================================================================
// Selection lifetime
// =======================================================================

/// **A selection cannot outlive the document it was made on.**
#[test]
fn a_selection_cannot_outlive_the_document_it_was_made_on() {
    use crate::canvastarget::TargetId;
    use crate::selectionidentity::{ClickHit, SelectionLevel};

    let mut doc = open_fixture(FOUR_PAGES);
    assert!(
        doc.selection.is_empty(),
        "a freshly opened document has nothing selected"
    );

    doc.selection.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(1)),
            ..ClickHit::default()
        },
        false,
        false,
    );
    assert_eq!(doc.selection.len(), 1);

    doc = open_fixture(PAINTED_LAYERS);
    assert!(
        doc.selection.is_empty(),
        "a new document starts with an empty selection, whatever address \
         its session landed on"
    );
    assert_eq!(
        doc.selection.level(),
        SelectionLevel::Object,
        "…and at the top rung, not inside an object of the previous file"
    );
}

// ===========================================================================
// The held preview — `OPERATOR_REQUESTS.md` O63's third piece
// ===========================================================================

/// Build a document with a hold already in place, `captured_at_epoch` frames
/// old, captured `age` ago.
fn with_hold(
    edit_epoch: u64,
    page_texture_epoch: u64,
    captured_at_epoch: u64,
    age: std::time::Duration,
) -> OpenDoc {
    let mut doc = open_local_fixture("polyline-nodes.pdf");
    doc.edit_epoch = edit_epoch;
    // THE TEXTURE'S EPOCH IS SET THROUGH ITS REAL RELATIONSHIP, never by
    // assignment. `page_texture_epoch` carries a **`PageEpochs`** value and
    // `app::settle` is its only writer — `self.page_texture_epoch =
    // self.page_epochs.get(page)`. It is not an `edit_epoch`.
    //
    // Callers pass the two arguments EQUAL to mean *"the raster has caught up"*
    // and unequal to mean *"it is behind"*, and that intention is expressed here
    // against the counter the readers actually consult. Assigning both fields
    // directly instead lets a test pass while describing a model the program
    // does not use.
    doc.page_texture_epoch = if page_texture_epoch == edit_epoch {
        // Caught up: the texture carries this page's current revision.
        doc.page_epochs.get(doc.view.page_index)
    } else {
        // Behind: any value this page's counter has not reached. `wrapping_sub`
        // rather than `+ 1`, because "behind" is the honest direction and the
        // counter only ever increases.
        doc.page_epochs.get(doc.view.page_index).wrapping_sub(1)
    };
    doc.held_preview = Some(super::HeldPreview {
        shape: crate::shapepreview::ShapePreview::default(),
        captured_at_epoch,
        since: std::time::Instant::now() - age,
    });
    doc
}

/// The ordinary case: the edit committed, the raster has not caught up, so the
/// preview stays on screen.
#[test]
fn a_committed_edit_whose_raster_has_not_landed_keeps_its_preview() {
    let doc = with_hold(8, 7, 7, std::time::Duration::from_millis(50));
    assert!(
        doc.held_preview_to_draw().is_some(),
        "the edit bumped the epoch to 8 and the texture is still at 7, so the page on screen \
         does NOT show this edit — which is exactly when the preview has to stay up"
    );
}

/// The raster landed. The document's own picture is correct now and is better
/// than the preview in every way, so the preview goes.
#[test]
fn the_preview_goes_the_moment_the_page_catches_up() {
    let doc = with_hold(8, 8, 7, std::time::Duration::from_millis(50));
    assert!(
        doc.held_preview_to_draw().is_none(),
        "the texture epoch caught the edit epoch, so the page already shows the edit"
    );
}

/// THE ONE THAT MATTERS: a refused edit holds nothing for long.
#[test]
fn an_edit_the_epoch_never_moved_for_stops_drawing_almost_at_once() {
    // 20 ms — about one frame. Legitimate: the Action was raised this frame and
    // has not been drained yet.
    let fresh = with_hold(7, 7, 7, std::time::Duration::from_millis(20));
    assert!(
        fresh.held_preview_to_draw().is_some(),
        "one frame after release the commit has not been applied yet, and blinking the preview \
         off for that frame is the flicker this feature exists to remove"
    );

    // 400 ms with the epoch still unmoved is not "not yet". It is a refusal.
    let refused = with_hold(7, 7, 7, std::time::Duration::from_millis(400));
    assert!(
        refused.held_preview_to_draw().is_none(),
        "400 ms with the epoch unmoved means the engine REFUSED — and a preview of a move that \
         did not happen, drawn over a document that disagrees with it, is worse than no preview"
    );
}

/// The backstop fires even when everything else says "keep drawing".
#[test]
fn the_backstop_drops_a_preview_no_raster_ever_arrived_for() {
    let doc = with_hold(8, 7, 7, std::time::Duration::from_secs(10));
    assert!(
        doc.held_preview_to_draw().is_none(),
        "ten seconds is not a slow render, it is a raster that is never coming"
    );
}

/// `retire_held_preview` clears what `held_preview_to_draw` has stopped
/// returning — and leaves a live one alone.
///
/// Two assertions rather than one, because a retire that cleared everything
/// would pass the first half and silently delete the feature.
#[test]
fn retiring_clears_a_dead_hold_and_keeps_a_live_one() {
    let mut dead = with_hold(8, 8, 7, std::time::Duration::from_millis(50));
    dead.retire_held_preview();
    assert!(
        dead.held_preview.is_none(),
        "a dead hold must not sit in memory"
    );

    let mut live = with_hold(8, 7, 7, std::time::Duration::from_millis(50));
    live.retire_held_preview();
    assert!(
        live.held_preview.is_some(),
        "retiring must not delete a hold that is still doing its job"
    );
}

/// The page-is-catching-up line is silent under the threshold, and speaks
/// past it — and speaks for EVERY edit, not only the ones with a shape.
#[test]
fn the_catching_up_line_waits_before_it_speaks() {
    let mut doc = open_local_fixture("polyline-nodes.pdf");
    doc.edit_epoch = 5;
    // One behind THIS PAGE's epoch — the quantity `page_is_catching_up`
    // reads. See `with_hold` for why this is not `edit_epoch - 1`.
    doc.page_texture_epoch = doc.page_epochs.get(doc.view.page_index).wrapping_sub(1);

    doc.last_edit_at = Some(std::time::Instant::now());
    assert!(
        !doc.page_is_catching_up(),
        "the picture is behind by a few milliseconds after every edit; saying so each time is \
         noise that costs every other sentence in the bar"
    );

    doc.last_edit_at = Some(std::time::Instant::now() - std::time::Duration::from_millis(600));
    assert!(
        doc.page_is_catching_up(),
        "600 ms is past the point where a person starts wondering whether the program heard \
         them — and an operator who cannot tell 'drawing' from 'ignored me' presses the button \
         again, which is a second edit neither of them wanted"
    );
}

/// It stops the moment the picture is correct.
#[test]
fn the_catching_up_line_stops_when_the_raster_lands() {
    let mut doc = open_local_fixture("polyline-nodes.pdf");
    doc.edit_epoch = 5;
    // Exactly this page's epoch — the texture carries the current revision.
    doc.page_texture_epoch = doc.page_epochs.get(doc.view.page_index);
    doc.last_edit_at = Some(std::time::Instant::now() - std::time::Duration::from_secs(30));
    assert!(
        !doc.page_is_catching_up(),
        "the texture carries the edit, so there is nothing to catch up to — and a sentence \
         claiming otherwise over a correct picture is simply false"
    );
}

/// **An edit on another sheet must not strand the line on this one.**
#[test]
fn a_page_edit_elsewhere_does_not_strand_the_catching_up_line() {
    let mut doc = open_local_fixture("polyline-nodes.pdf");
    doc.view.page_index = 0;

    // The raster for page 0 has landed and is current: `app::settle` writes
    // this field from `page_epochs.get(page)`, so that is how the test writes
    // it too.
    doc.page_texture_epoch = doc.page_epochs.get(0);
    doc.last_edit_at = Some(std::time::Instant::now() - std::time::Duration::from_secs(30));
    assert!(
        !doc.page_is_catching_up(),
        "the texture carries this page's current revision, so there is nothing to catch up to"
    );

    // Now an edit lands somewhere that is NOT the page on screen. Both counters
    // move, independently — which is exactly what happens on a multi-sheet
    // drawing, and what `bump_all` plus a resize guarantees on a page delete.
    doc.edit_epoch = doc.edit_epoch.wrapping_add(1);
    doc.page_epochs.bump(1);

    assert!(
        !doc.page_is_catching_up(),
        "★ an edit on ANOTHER sheet must not put a 'catching up' line over this one. Before \
         2026-09-03 this compared `page_texture_epoch` against `edit_epoch` — two independent \
         counters — so this assertion failed, and it failed FOREVER: nothing brings the two \
         numbers back into step, so the sentence stayed on the bar for the rest of the session."
    );
}

/// A document nobody has edited says nothing, however far apart the epochs are.
#[test]
fn an_unedited_document_never_says_it_is_catching_up() {
    let mut doc = open_local_fixture("polyline-nodes.pdf");
    doc.edit_epoch = 0;
    doc.page_texture_epoch = 7;
    assert!(doc.last_edit_at.is_none());
    assert!(
        !doc.page_is_catching_up(),
        "nothing has been edited, so there is no edit for the picture to be behind"
    );
}
