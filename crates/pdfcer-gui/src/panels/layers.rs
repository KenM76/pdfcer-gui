//! # `panels::layers` — the document's optional-content groups
//!
//! Contract: one row per layer with its visibility switch, arranged as the
//! document's `/D /Order` arranges it (folders, sublayers, groupings) while
//! the search field is empty and as a flat filtered list while it is not.
//! Every disclosure about the document sits above the list; everything that
//! appears in answer to a click sits in the footer. Nothing here writes the
//! document: every control raises an [`Action`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/layers.md`.

/// Only [`settle`] and the tests below need a set type of their own.
#[cfg(test)]
use std::collections::BTreeSet;

use pdfcer_core::layers::{Layer, Layers};
use pdfcer_core::object::ObjId;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::panels::footer;
use crate::text::panels as t;

/// The footer's collapsible header, published so a driven check can open it.
pub const REGION_TOOLS: &str = "layers.tools"; // ui-text-exempt: trace region name, never displayed
use crate::text::panels::layers as tl;
use crate::text::panels::layersearch as ts;

/// Narrowing the list as you type: the predicate and the counts.
use pdfcer_gui_base::layersearch as search;

/// Which layer the current selection is on.
pub(crate) mod highlight;

/// New layer, and each row's Properties and Delete.
mod authoring;

/// Merge a layer into another; Flatten all.
mod combine;

/// Dragging a layer or folder to a new place.
mod drag;

/// New folder, and the Rename folder window.
mod folders;

/// One layer's row.
mod row;

/// The list as the document's `/Order` arranges it.
mod tree;

/// The trace name of the search field's rectangle.
// ui-text-exempt: trace region name, never displayed.
const REGION_SEARCH: &str = "panel.layers.search";
/// The trace name of the control that empties the search field.
// ui-text-exempt: trace region name, never displayed.
const REGION_SEARCH_CLEAR: &str = "panel.layers.search.clear";

/// Draw the Layers panel.
pub fn body(ui: &mut egui::Ui, doc: &OpenDoc, state: &mut PanelsState, actions: &mut Vec<Action>) {
    let view = doc.session.view();
    let read = pdfcer_core::layers::read_layers(&view);

    let authoring = authoring::enabled(ui.ctx());
    let panel_height = ui.available_height();
    if authoring {
        authoring::windows(ui.ctx(), actions);
        combine::windows(ui.ctx(), &read, actions);
        folders::rename_window(ui.ctx(), actions);
    }
    if read.diagnostics.no_optional_content {
        ui.label(t::layers_none());
        if authoring {
            footer::show(
                ui,
                REGION_TOOLS,
                panel_height,
                t::layers_tools(),
                |_| {},
                Some(|ui: &mut egui::Ui| authoring::new_layer_row(ui, &read, actions)),
            );
        }
        return;
    }
    // The set the page is ACTUALLY drawn from: the operator's override if
    // there is one, else the document's own. A tick read from
    // `visible_by_default` would tick itself back on the next repaint.
    let effective_hidden = doc.hidden_layers();
    let differing = effective_hidden
        .symmetric_difference(&pdfcer_core::annot::optional_content_default_off(&view))
        .count();
    document_lines(ui, &read);
    let Some(query) = search_field(ui, &read, state) else {
        return;
    };
    let membership = highlight::resolve(doc);
    let highlighted = membership.highlighted();
    let report = selection_report(&read, &query, membership);
    // One path object can hold a thousand subpaths, so "this is on layer
    // Grid" is said with the count when it is more than one.
    let granularity = highlight::parts_in_selected_object(doc).map(tl::layer_selection_granularity);
    ui.separator();

    let cx = row::RowCtx {
        read: &read,
        hidden: &effective_hidden,
        highlighted,
        authoring,
    };
    let mut toggled = None;
    egui::ScrollArea::vertical()
        .id_salt("layers-rows")
        .max_height(footer::list_height(ui, REGION_TOOLS))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            toggled = if query.is_empty() {
                let tree = pdfcer_gui_base::layerorder::tree(&read.order);
                tree::show(ui, &cx, &tree, actions)
            } else {
                flat(ui, &cx, &query)
            };
        });

    let mut reset = false;
    footer::show(
        ui,
        REGION_TOOLS,
        panel_height,
        t::layers_tools(),
        |ui: &mut egui::Ui| {
            // Offered only once there is something to undo.
            if differing > 0 {
                ui.horizontal(|ui| {
                    ui.label(t::layers_overridden(differing));
                    reset = ui
                        .button(t::layers_reset_label())
                        .on_hover_text(t::layers_reset_tooltip())
                        .clicked();
                });
            }
            for line in report.iter().chain(granularity.iter()) {
                ui.label(egui::RichText::new(line).small());
            }
        },
        authoring.then_some(|ui: &mut egui::Ui| {
            authoring::new_layer_row(ui, &read, actions);
            folders::new_folder_row(ui, &read, actions);
            combine::flatten_button(ui, &read);
        }),
    );
    if reset {
        actions.push(Action::ResetLayers);
    }
    if let Some((id, visible)) = toggled {
        actions.extend(toggle_actions(&read, id, visible));
    }
}

/// The lines about the document as a whole, above the search field.
fn document_lines(ui: &mut egui::Ui, read: &Layers) {
    ui.label(t::layers_count(read.layers.len()));
    ui.label(
        egui::RichText::new(t::layers_session_only_note())
            .small()
            .weak(),
    );
    // §8.11.4.4: a viewer recomputes some states from the magnification, so
    // a zoom-banded layer can read "shown" while its content is off the page.
    if read.diagnostics.auto_managed_groups > 0 {
        ui.label(
            egui::RichText::new(t::layers_auto_managed(read.diagnostics.auto_managed_groups))
                .small(),
        );
    }
}

/// The search field and the lines about what it narrowed. `None` when the
/// query matched nothing: the sentence and its Clear button are drawn, and no
/// list is (an empty scroll area paints a blank slab).
fn search_field(ui: &mut egui::Ui, read: &Layers, state: &mut PanelsState) -> Option<String> {
    let total = read.layers.len();
    if total < search::MIN_LAYERS_FOR_SEARCH {
        // The stored query is emptied too, or a query typed on another
        // document would filter this one through a box that is not drawn.
        state.layers_search_mut().clear();
        return Some(String::new());
    }
    ui.horizontal(|ui| {
        let field = ui.add(
            // escape-disposition: not-content — a filter over the layer list.
            egui::TextEdit::singleline(state.layers_search_mut())
                .hint_text(ts::field_hint())
                .desired_width(f32::INFINITY),
        );
        crate::diag::ui_rect(REGION_SEARCH, field.rect);
        field.on_hover_text(ts::field_tooltip());
    });
    // Read back after the field is drawn, so this frame filters on what was
    // just typed.
    let query = state.layers_search_mut().trim().to_owned();
    let shown = read
        .layers
        .iter()
        .filter(|l| search::matches(&row_name(l), &query))
        .count();
    let filtered = if query.is_empty() {
        search::Filtered::all(total)
    } else {
        search::Filtered {
            shown,
            hidden: total - shown,
        }
    };
    if let Some(line) = ts::narrowed(filtered.shown, total).filter(|_| filtered.is_narrowed()) {
        ui.label(egui::RichText::new(line).small());
    }
    if filtered.is_empty_because_of_the_query() {
        ui.label(ts::none_matched(&query, total));
        let clear = ui
            .button(ts::clear_label())
            .on_hover_text(ts::clear_tooltip());
        crate::diag::ui_rect(REGION_SEARCH_CLEAR, clear.rect);
        if clear.clicked() {
            state.layers_search_mut().clear();
        }
        return None;
    }
    Some(query)
}

/// The sentence about the canvas selection's layer, if one is owed. A row
/// the query has filtered away has a different remedy (clear the search)
/// from a group the document does not list (none), so they are two answers.
fn selection_report(read: &Layers, query: &str, m: highlight::Membership) -> Option<String> {
    let highlighted = m.highlighted();
    let filtered_away = highlighted
        .and_then(|id| read.layers.iter().find(|l| l.id == id))
        .map(row_name)
        .filter(|name| !search::matches(name, query));
    let row = match (highlighted, filtered_away.as_deref()) {
        (None, _) => tl::RowOfAnswer::NotAGroup,
        (Some(_), Some(name)) => tl::RowOfAnswer::HiddenBySearch(name),
        (Some(id), None) if read.layers.iter().any(|l| l.id == id) => tl::RowOfAnswer::OnScreen,
        (Some(_), None) => tl::RowOfAnswer::NotListed,
    };
    tl::layer_selection_report(m, row)
}

/// The filtered list: every matching layer, flat, with no moves.
fn flat(ui: &mut egui::Ui, cx: &row::RowCtx<'_>, query: &str) -> Option<(ObjId, bool)> {
    let mut toggled = None;
    for l in cx
        .read
        .layers
        .iter()
        .filter(|l| search::matches(&row_name(l), query))
    {
        toggled = toggled.or(row::draw(ui, cx, l, |_| {}).toggled);
    }
    toggled
}

/// What this layer's row is called: the one spelling the row and the search
/// both use. An undeclared `/Name` (Required, Table 98) shows as a
/// placeholder, never an invented one.
fn row_name(l: &Layer) -> String {
    if l.name_declared {
        l.name.clone()
    } else {
        t::layer_unnamed().to_owned()
    }
}

/// **What a layer is called, for a surface that is not this panel.**
pub(crate) fn layer_name_for(read: &pdfcer_core::layers::Layers, id: ObjId) -> Option<String> {
    read.layers.iter().find(|l| l.id == id).map(row_name)
}

/// Is `id` a group the document marks `/Locked`?
fn is_locked(read: &Layers, id: ObjId) -> bool {
    read.layers.iter().any(|l| l.id == id && l.locked)
}

/// The complete list of actions one click on `id`'s control should raise.
fn toggle_actions(read: &Layers, id: ObjId, visible: bool) -> Vec<Action> {
    let mut out = Vec::new();
    if visible && let Some(members) = read.radio_group_of(id) {
        for sibling in members {
            if *sibling != id && !is_locked(read, *sibling) {
                out.push(Action::SetLayerVisible {
                    group: *sibling,
                    visible: false,
                });
            }
        }
    }
    out.push(Action::SetLayerVisible { group: id, visible });
    out
}

/// Everything this row has to explain about itself, in the order shown.
fn row_caveats(
    read: &Layers,
    layer: &pdfcer_core::layers::Layer,
    effective: bool,
) -> Vec<&'static str> {
    let mut notes = Vec::new();
    if effective != layer.visible_by_default {
        notes.push(t::layer_overridden_tooltip(layer.visible_by_default));
    }
    // §8.11.2.3: a group whose `/Intent` excludes `View` does not participate
    // in visibility under the document's own configuration, so its state in
    // `/OFF` has no effect on what a reader draws.
    //
    // Said out loud because the alternative is an operator seeing a layer
    // marked visible that the file's own `/OFF` array names, with no way to
    // tell whether that is intent filtering or a pdfcer bug. pdfcer inferred
    // something (this group does not count) and the inference changed the
    // page — rule 4 says the inference is disclosed, not merely correct.
    if !layer.intent_view {
        notes.push(t::layer_design_intent_tooltip());
    }
    if layer.locked {
        notes.push(t::layer_locked_tooltip());
    }
    if !layer.in_default_config {
        notes.push(t::layer_unregistered_tooltip());
    }
    if layer.radio_group.is_some() {
        notes.push(t::layer_radio_tooltip());
        // Only when a sibling is actually locked. A blanket warning on every
        // radio row would train the operator to ignore it, and the row it
        // matters on is the one where two members can end up showing.
        if read
            .radio_group_of(layer.id)
            .is_some_and(|m| m.iter().any(|s| *s != layer.id && is_locked(read, *s)))
        {
            notes.push(t::layer_radio_locked_sibling_tooltip());
        }
    }
    notes
}

/// `Layers` does not offer this join, so the panel makes it once.
trait RadioGroupLookup {
    /// The members of `id`'s first `/RBGroups` array, if it is in one.
    fn radio_group_of(&self, id: ObjId) -> Option<&[ObjId]>;
}

impl RadioGroupLookup for Layers {
    fn radio_group_of(&self, id: ObjId) -> Option<&[ObjId]> {
        let layer = self.layers.iter().find(|l| l.id == id)?;
        // `get`, not indexing: `radio_group` indexes a vector this type also
        // owns, so they cannot disagree — but the crate denies indexing and a
        // panic here would take out a panel over a malformed file.
        self.radio_groups.get(layer.radio_group?).map(Vec::as_slice)
    }
}

/// A `BTreeSet` of the ids `actions` would leave hidden, applied in order.
#[cfg(test)]
fn settle(start: &BTreeSet<ObjId>, actions: &[Action]) -> BTreeSet<ObjId> {
    let mut hidden = start.clone();
    for a in actions {
        if let Action::SetLayerVisible { group, visible } = a {
            if *visible {
                hidden.remove(group);
            } else {
                hidden.insert(*group);
            }
        }
    }
    hidden
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::objects::test_support::engine_fixture;
    use crate::text::panels as t;

    /// Load a fixture's layer report and its document-default hidden set.
    fn read_fixture(rel: &str) -> (Layers, BTreeSet<ObjId>) {
        let path = engine_fixture(rel);
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let read = pdfcer_core::layers::read_layers(&doc);
        let hidden = pdfcer_core::annot::optional_content_default_off(&doc);
        (read, hidden)
    }

    /// **Precondition 2 is satisfied: a layer toggle invalidates the
    /// cached page.**
    ///
    /// The reason this panel shipped without its checkbox at S3 was that
    /// `RenderKey` compared only page index and raster scale, so a tick
    /// would have changed a field and redrawn nothing — a control that looks
    /// broken, which is worse than one that is absent.
    ///
    /// This drives the panel's *own* data path: read the layers the way
    /// [`super::body`] does, take a group's `ObjId` off a [`Layer`] the way
    /// the checkbox does, toggle it through the only mutator a control is
    /// allowed to use, and assert the render key moved. It is deliberately
    /// not a test of `RenderKey`'s field list (that is the worker's own) —
    /// it is a test that *this panel's* route to that key is connected.
    ///
    /// It also pins the toggle as **discrete**: a checkbox has no gesture in
    /// flight, so inheriting the 150 ms zoom debounce would make it feel
    /// broken in a subtler way.
    ///
    /// Kept unchanged now that the control has landed, because it is what
    /// says the plumbing behind it was proven before it shipped — and
    /// because it is the test that fails if a later change to `RenderKey`
    /// drops the field again.
    ///
    /// [`Layer`]: pdfcer_core::layers::Layer
    #[test]
    fn the_render_key_no_longer_blocks_a_layer_toggle() {
        let path = engine_fixture("layers/painted-layers.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        let mut open =
            crate::app::state::OpenDoc::new(path, pdfcer_core::edit::EditSession::new(doc), pages);

        // Exactly what the panel body reads.
        let read = pdfcer_core::layers::read_layers(&open.session.view());
        assert!(
            !read.diagnostics.no_optional_content,
            "this fixture must declare optional content, or the test is vacuous"
        );
        let hidden = read
            .layers
            .iter()
            .find(|l| !l.visible_by_default)
            .expect("the fixture must carry a layer the document turns OFF");

        let before = open.render_key(1.0);
        // The gesture the checkbox makes: show a layer the document hides.
        open.set_layer_visible(hidden.id, true);
        let after = open.render_key(1.0);

        assert_ne!(
            before, after,
            "ticking a layer must make the cached texture stale, or the \
             checkbox redraws nothing"
        );
        assert_eq!(
            before.scale_bits(),
            after.scale_bits(),
            "a layer toggle must not look like a zoom, or it inherits the \
             zoom debounce and a click takes 150 ms to do anything"
        );
        assert_ne!(before.discrete_inputs(), after.discrete_inputs());
    }

    /// **Precondition 3: the click reaches `apply`, and the whole round
    /// trip lands on the page.**
    #[test]
    fn a_composed_click_changes_both_the_hidden_set_and_the_render_key() {
        let path = engine_fixture("layers/painted-layers.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        let mut open =
            crate::app::state::OpenDoc::new(path, pdfcer_core::edit::EditSession::new(doc), pages);

        let read = pdfcer_core::layers::read_layers(&open.session.view());
        let target = read
            .layers
            .iter()
            .find(|l| !l.visible_by_default)
            .expect("the fixture must carry a layer the document turns OFF");

        let before_key = open.render_key(1.0);
        assert!(
            open.hidden_layers().contains(&target.id),
            "the document must be hiding this layer before the click"
        );

        // What the panel raises …
        let raised = toggle_actions(&read, target.id, true);
        assert!(!raised.is_empty(), "a click must raise at least one action");
        // … and what `PdfcerApp::apply` does with each of them.
        for a in &raised {
            if let Action::SetLayerVisible { group, visible } = a {
                open.set_layer_visible(*group, *visible);
            }
        }

        assert!(
            !open.hidden_layers().contains(&target.id),
            "the layer the operator showed is still hidden — the click went nowhere"
        );
        assert_ne!(
            before_key,
            open.render_key(1.0),
            "the page would not re-rasterize, so the control would look inert"
        );
    }

    /// **Reset returns to the document's default, which is NOT "show
    /// everything".**
    #[test]
    fn a_reset_restores_the_document_rather_than_revealing_everything() {
        let path = engine_fixture("layers/painted-layers.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        let mut open =
            crate::app::state::OpenDoc::new(path, pdfcer_core::edit::EditSession::new(doc), pages);

        let document_hidden = open.hidden_layers();
        assert!(
            !document_hidden.is_empty(),
            "this fixture must hide at least one layer by default, or the \
             difference between 'reset' and 'show everything' is invisible"
        );

        // Diverge, then come back.
        let target = *document_hidden.iter().next().expect("checked non-empty");
        open.set_layer_visible(target, true);
        assert_ne!(open.hidden_layers(), document_hidden);
        open.reset_layers();

        assert_eq!(
            open.hidden_layers(),
            document_hidden,
            "reset must restore the document's own configuration"
        );
        assert!(
            !open.hidden_layers().is_empty(),
            "reset revealed every layer — that is 'show everything', a \
             different act with a disclosure consequence"
        );
    }

    /// **The Reset control is offered only when something differs.**
    #[test]
    fn the_reset_control_appears_exactly_when_the_view_differs_from_the_document() {
        let path = engine_fixture("layers/painted-layers.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        let mut open =
            crate::app::state::OpenDoc::new(path, pdfcer_core::edit::EditSession::new(doc), pages);

        let document_hidden =
            pdfcer_core::annot::optional_content_default_off(&open.session.view());
        let differing = |open: &crate::app::state::OpenDoc| {
            open.hidden_layers()
                .symmetric_difference(&document_hidden)
                .count()
        };

        assert_eq!(differing(&open), 0, "nothing has been changed yet");

        let target = *document_hidden
            .iter()
            .next()
            .expect("the fixture must hide a layer");
        open.set_layer_visible(target, true);
        assert_eq!(differing(&open), 1, "exactly one layer now differs");

        // Back to where it started: the count must fall to zero even though
        // the override slot is still occupied.
        open.set_layer_visible(target, false);
        assert_eq!(
            differing(&open),
            0,
            "a layer switched off and on again agrees with the document, and \
             a panel that still claims a difference is counting clicks"
        );
    }

    /// **Turning on a radio member turns its unlocked siblings off.**
    #[test]
    fn turning_on_a_radio_member_turns_its_unlocked_siblings_off() {
        let (read, document_hidden) = read_fixture("layers/radio-locked.pdf");
        assert!(
            !read.radio_groups.is_empty(),
            "this fixture exists to carry /RBGroups; without one the test is vacuous"
        );

        // A clickable member whose REPORTED group has an unlocked sibling.
        let target = read
            .layers
            .iter()
            .find(|l| {
                !l.locked
                    && read
                        .radio_group_of(l.id)
                        .is_some_and(|m| m.iter().any(|s| *s != l.id && !is_locked(&read, *s)))
            })
            .expect("the fixture must carry a radio group with two unlocked members")
            .id;
        let siblings = read
            .radio_group_of(target)
            .expect("it was chosen for being in one")
            .to_vec();

        let settled = settle(&document_hidden, &toggle_actions(&read, target, true));
        assert!(
            !settled.contains(&target),
            "the layer the operator clicked must end up shown"
        );
        let mut swept = 0_usize;
        for s in &siblings {
            if *s != target && !is_locked(&read, *s) {
                assert!(
                    settled.contains(s),
                    "an unlocked sibling stayed on — /RBGroups says at most one \
                     member of the group is visible at a time"
                );
                swept += 1;
            }
        }
        assert!(
            swept > 0,
            "no sibling was actually swept, so the assertion above passed \
             vacuously"
        );
    }

    /// **`DA-A8`: a locked sibling is left exactly as it was.**
    #[test]
    fn a_locked_radio_sibling_is_never_switched_off_by_a_click_elsewhere() {
        let (read, _) = read_fixture("layers/radio-locked.pdf");
        let locked: Vec<ObjId> = read
            .layers
            .iter()
            .filter(|l| l.locked && l.radio_group.is_some())
            .map(|l| l.id)
            .collect();
        assert!(
            !locked.is_empty(),
            "this fixture exists to carry a /Locked group inside a radio group"
        );

        // Every layer the panel could offer a click on.
        for l in read.layers.iter().filter(|l| !l.locked) {
            for actions in [
                toggle_actions(&read, l.id, true),
                toggle_actions(&read, l.id, false),
            ] {
                for a in &actions {
                    if let Action::SetLayerVisible { group, .. } = a {
                        assert!(
                            !locked.contains(group),
                            "clicking {:?} raised an action against locked group \
                             {group:?} — a lock bypassed through a side door",
                            l.id
                        );
                    }
                }
            }
        }
    }

    /// **Turning a layer OFF never turns a sibling ON.**
    #[test]
    fn turning_a_radio_member_off_leaves_its_siblings_alone() {
        let (read, _) = read_fixture("layers/radio-locked.pdf");
        for l in read.layers.iter().filter(|l| !l.locked) {
            let actions = toggle_actions(&read, l.id, false);
            assert_eq!(
                actions,
                vec![Action::SetLayerVisible {
                    group: l.id,
                    visible: false,
                }],
                "switching a layer off must move exactly that layer"
            );
        }
    }

    /// **The panel honours the FIRST radio array and no other.**
    #[test]
    fn a_click_never_reaches_outside_the_radio_array_core_reported() {
        let (read, _) = read_fixture("layers/radio-locked.pdf");
        assert!(
            read.diagnostics.overlapping_radio_groups > 0,
            "this fixture exists to carry a group in two /RBGroups arrays"
        );

        for l in read.layers.iter().filter(|l| !l.locked) {
            let reported: Vec<ObjId> = read
                .radio_group_of(l.id)
                .map(<[ObjId]>::to_vec)
                .unwrap_or_default();
            for action in toggle_actions(&read, l.id, true) {
                let Action::SetLayerVisible { group, .. } = action else {
                    panic!("a layer click must raise only SetLayerVisible");
                };
                assert!(
                    group == l.id || reported.contains(&group),
                    "clicking {:?} reached {group:?}, which core did not report \
                     as a member of its radio group — pdfcer has invented a \
                     resolution for DA-N1 rather than carrying core's",
                    l.id
                );
            }
        }
    }

    /// **The two state markers are words, and they are different words.**
    #[test]
    fn a_layers_state_is_carried_by_words_not_by_a_cue() {
        let shown = t::layer_visible_marker();
        let hidden = t::layer_hidden_marker();
        assert_ne!(shown, hidden);
        for m in [shown, hidden] {
            assert!(!m.trim().is_empty(), "an empty state marker");
            assert!(
                m.chars().all(|c| c.is_ascii_alphabetic()),
                "a state marker must be a word, not a symbol: {m}"
            );
        }
    }

    /// **Every sentence a row can show is a different explanation.**
    #[test]
    fn each_per_layer_caveat_explains_a_different_surprise() {
        let all = [
            t::layer_overridden_tooltip(true),
            t::layer_overridden_tooltip(false),
            t::layer_design_intent_tooltip(),
            t::layer_locked_tooltip(),
            t::layer_unregistered_tooltip(),
            t::layer_radio_tooltip(),
            t::layer_radio_locked_sibling_tooltip(),
            t::layer_toggle_tooltip(),
        ];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
        // The lock is an interface lock, not a guarantee, and the sentence
        // has to say so — the specification's own table blesses JavaScript
        // and `/AS` bypass.
        assert!(
            t::layer_locked_tooltip().contains("not a guarantee"),
            "{}",
            t::layer_locked_tooltip()
        );
        // The design-intent tooltip must not stop at "the document's setting
        // does not affect what is drawn": with an override in force the
        // renderer's OFF set IS the override, unfiltered, so a switch here
        // does affect it. Half the sentence reads as "no point clicking".
        assert!(
            t::layer_design_intent_tooltip().contains("Switching it here does"),
            "{}",
            t::layer_design_intent_tooltip()
        );
    }

    /// **A row that agrees with the document carries no override caveat, and
    /// one that disagrees carries exactly one.**
    #[test]
    fn only_a_diverging_row_says_the_operator_changed_it() {
        let (read, _) = read_fixture("layers/radio-locked.pdf");
        for l in &read.layers {
            let agreeing = row_caveats(&read, l, l.visible_by_default);
            assert!(
                !agreeing.contains(&t::layer_overridden_tooltip(true))
                    && !agreeing.contains(&t::layer_overridden_tooltip(false)),
                "a row matching the document must not claim the operator \
                 changed it: {:?}",
                l.id
            );
            let diverging = row_caveats(&read, l, !l.visible_by_default);
            assert!(
                diverging.contains(&t::layer_overridden_tooltip(l.visible_by_default)),
                "a diverging row must name the state the document asked for: {:?}",
                l.id
            );
        }
    }

    /// **The locked-sibling warning appears only where a sibling is locked.**
    #[test]
    fn the_locked_sibling_warning_is_not_shown_to_every_radio_row() {
        let (read, _) = read_fixture("layers/radio-locked.pdf");
        let mut warned = 0_usize;
        for l in &read.layers {
            let notes = row_caveats(&read, l, l.visible_by_default);
            let has_warning = notes.contains(&t::layer_radio_locked_sibling_tooltip());
            let deserves = read
                .radio_group_of(l.id)
                .is_some_and(|m| m.iter().any(|s| *s != l.id && is_locked(&read, *s)));
            assert_eq!(
                has_warning, deserves,
                "the locked-sibling warning disagrees with the data for {:?}",
                l.id
            );
            warned += usize::from(has_warning);
        }
        assert!(
            warned > 0,
            "this fixture carries a locked group inside a radio group, so at \
             least one row must carry the DA-A8 disclosure — otherwise the \
             assertion above is satisfied by warning nobody"
        );
    }

    /// An unnamed layer is disclosed as unnamed, not given a number.
    #[test]
    fn an_unnamed_layer_is_not_given_an_invented_name() {
        let placeholder = t::layer_unnamed();
        assert!(placeholder.starts_with('('), "{placeholder}");
        assert!(
            placeholder.contains("no name"),
            "the placeholder must say the file is missing something: {placeholder}"
        );
        assert!(
            !placeholder.chars().any(|c| c.is_ascii_digit()),
            "a numbered placeholder reads as data from the file: {placeholder}"
        );
    }
}
