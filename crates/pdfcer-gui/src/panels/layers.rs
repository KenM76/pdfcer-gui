//! # `panels::layers` — the document's optional-content groups
//!
//! Salvaged from the old shell's `panels_structure.rs`. The **report** came
//! across whole at S3; the **checkbox did not**, and at S4 it is back, with
//! the `/RBGroups` radio behaviour and the Reset control that travelled with
//! it. This module's header is therefore in two halves: what the panel says,
//! and the history of the control it says it about.
//!
//! # What it shows that a name cannot
//!
//! Whether a reader opening this document with no interaction would DRAW
//! each layer. A "Confidential" watermark that is off by default is a
//! different document from one where it is on, and the two are
//! indistinguishable by name.
//!
//! That value comes from `pdfcer-core`'s own `Layer::visible_by_default` —
//! the initial `/D` state, which is the same thing the renderer resolves —
//! so the panel cannot say "on" about content the page hides.
//!
//! Six per-layer facts are surfaced as tooltips, and each exists because
//! without it the operator's only available reading is *"pdfcer got it
//! wrong"*:
//!
//! | Fact | Why it must be said |
//! |---|---|
//! | the operator has changed this row | Names the document's own state, so the divergence can be seen without resetting to find out. |
//! | `/Intent` excludes `View` (§8.11.2.3) | The group does not participate in visibility **under the document's own configuration**, so its state in the document's `/OFF` array has no effect on what a reader draws — but a switch here does affect it, for the reason in [`crate::text::panels::layer_design_intent_tooltip`]'s docs. A layer marked visible that the file's own `/OFF` names looks like a defect otherwise. |
//! | `/Locked` (Table 101) | *"the UI shall not allow the visibility state to be changed"* — an interface lock, not a guarantee, since the specification's own table blesses JavaScript and `/AS` bypass. |
//! | not in the default configuration | Page content uses the layer and the document never registered it. Some readers will not show it at all. |
//! | in an `/RBGroups` radio group | At most one member visible at a time, so this layer's state is not independent of its siblings'. |
//! | that radio group contains a **locked** member | pdfcer will not switch that sibling off, so this group can end up with two members showing. See "`DA-A8`" below. |
//!
//! §8.11.4.4's **auto-managed** groups get a line of their own, above the
//! list: some states are not the document's to state — a viewer recomputes
//! them from the magnification — so a zoom-banded layer can read "shown"
//! here while its content is off the page. Said out loud rather than left to
//! be discovered as a defect.
//!
//! # ★ The visibility control, and the three preconditions it waited on
//!
//! **All three now hold.** The section is kept rather than deleted because
//! it is a worked example of `crate::render::worker`'s rule — *"the key
//! ships in the same commit as its control"* — actually preventing a defect
//! rather than merely being stated, and because the next person to blame a
//! control for redrawing nothing should be able to read how this one was
//! diagnosed.
//!
//! The old panel let an operator toggle a layer for the session, honouring
//! `/RBGroups` radio semantics, with a Reset that returned to the document's
//! own configuration. All of that machinery is *good*. Three things had to be
//! true for the tick to change a pixel:
//!
//! 1. **The renderer must accept an override.** True, and always was —
//!    [`pdfcer_render::LayerVisibility`] exists and `pdfcer-render` honours
//!    content-stream `/OC`.
//! 2. **The render worker's cache key must vary with it.** **True as of
//!    S4.** `crate::render::worker`'s `RenderKey` carries
//!    `layers_generation`; `crate::app::state::OpenDoc` carries the override
//!    the renderer takes, the counter that keys it, and the mutators a
//!    control calls — `hidden_layers`, `set_hidden_layers`,
//!    `set_layer_visible`, `reset_layers`. `OpenDoc::render_key` is compared
//!    against the cached texture's own key every frame, so a change to the
//!    override re-rasterizes at once rather than waiting out the zoom
//!    debounce. [`tests::the_render_key_no_longer_blocks_a_layer_toggle`]
//!    pins exactly that, against this panel's own data path.
//! 3. **An action must carry the toggle.** **True as of S4**, and the last
//!    to land. [`crate::app::actions::Action`] gained `SetLayerVisible`,
//!    `ResetLayers` and `ToggleAnnotations`, each implemented in
//!    `PdfcerApp::apply`. Before that a panel body — handed `&OpenDoc`, a
//!    *shared* reference, precisely so that it cannot mutate — had nowhere
//!    to send a click.
//!
//! ## Why the last precondition was not simply worked around
//!
//! Kept because the shortcut is still available and still wrong.
//!
//! `OpenDoc` holds two caches behind a `RefCell`, so the shape of a shortcut
//! is visible from here: put the override behind one too, and the panel could
//! toggle it through the shared reference it already has.
//!
//! That would be wrong, and the difference is not stylistic. The `RefCell`s
//! hold **derived caches** — filling one changes nothing an observer could
//! see, which is why a shared reference may do it. Layer visibility decides
//! **what appears on the page**. Routing it around the action funnel would
//! forfeit the fourth property `crate::app::actions`' header claims for that
//! funnel — *"every state change is greppable: what can change the zoom? has
//! a complete answer"* — and it would do so for the one class of state where
//! an operator can see the result and not find the cause. It would also put
//! the change outside the one place an undo log will be written.
//!
//! ## The pair that must move together, and the record of it not doing so
//!
//! The old panel's own header records this exact doc comment being **wrong
//! for three commits** in the opposite direction: it said the checkbox could
//! not exist, after the commit that added it. It was found by a person
//! reading the file, not by any check — nothing compiles a doc comment
//! against the behaviour it describes.
//!
//! So the control and the sentence that describes it are stated together, and
//! `the_layers_note_says_a_toggle_changes_the_view_and_not_the_document` in
//! `crate::text::panels` is the check that at least the sentence cannot
//! silently revert to describing a program that does not exist. That test
//! replaced `the_layers_note_states_that_switching_is_unavailable` **in this
//! commit**, which is the discipline working in the other direction: the
//! assertion that pinned the absence came out with the absence.
//!
//! # ★ `/RBGroups`: how the radio behaviour survives an action funnel
//!
//! Table 101's `/RBGroups` are "radio button" groups — at most one member ON
//! — and `pdfcer_core::layers` hands the panel everything needed to honour
//! them *before the first click*: [`pdfcer_core::layers::Layer::radio_group`]
//! is an index into [`pdfcer_core::layers::Layers::radio_groups`], which
//! carries the full membership.
//!
//! The interesting part is that [`crate::app::state::OpenDoc::set_layer_visible`]
//! deliberately does **not** implement radio semantics, and says so: the
//! sibling list is *"the control's reading, not this type's"*, and a
//! half-implementation there would be a second visibility algebra beside the
//! engine's. So the panel composes the whole gesture itself — and it does so
//! **as a list of actions**, one per layer that has to move, rather than by
//! reaching for a `SetHiddenLayers` variant that carries a set.
//!
//! That composes correctly for a reason worth stating, because it is the
//! only thing making the simple variant sufficient: `apply_actions` applies
//! in the order raised, and each `set_layer_visible` recomputes from
//! `hidden_layers()` — the *current* answer, including the effect of the
//! actions applied a moment ago in the same frame. So N actions in one frame
//! settle to exactly the set one composed call would have produced, they bump
//! the generation N times (which costs one re-render, since the raster
//! settles once per frame), and the funnel keeps its grep property: every
//! layer that moved has its own `Action` naming it.
//!
//! [`toggle_actions`] is that composition, extracted as a pure function so
//! the radio rule is tested against real fixtures without an egui context.
//!
//! ## Turning one OFF does not turn a sibling ON
//!
//! "At most one" permits none. Picking a replacement would be pdfcer choosing
//! which alternate the operator meant, which is exactly the class of
//! invention rule 4 forbids.
//!
//! ## ★ `DA-A8`: a locked layer inside a radio group
//!
//! `pdfcer_core::layers` names this as a genuine gap in the standard and hands
//! the decision here verbatim: a locked group's state *"cannot be changed
//! through the user interface"*, while a sibling being turned ON means all
//! others *"shall be turned OFF"*. Both clauses address the **user
//! interface**, so nothing in the specification breaks the tie.
//!
//! **pdfcer lets the lock win**: [`toggle_actions`] skips a locked sibling, so
//! a group can end up showing two members. The full argument is on
//! [`crate::text::panels::layer_radio_locked_sibling_tooltip`], which is the
//! disclosure — in one line, the alternative would switch off a locked layer
//! as a side effect of clicking a *different* row, which is a lock bypass
//! nobody watching the screen would see, and this way the violation is on the
//! page where the operator already is.
//!
//! # ★ Reset means "the document's own default", not "show everything"
//!
//! [`crate::app::actions::Action::ResetLayers`] drops the override, which
//! restores `/D` (§8.11.4.3). It is emphatically **not**
//! `set_hidden_layers(BTreeSet::new())`, which would reveal every layer the
//! document turns off — on a drawing with a "Confidential" watermark that is
//! a disclosure event rather than a cosmetic one. The distinction is core API
//! trap T-12.9 and it is the whole reason
//! [`crate::text::panels::layers_reset_tooltip`] names what it returns *to*.
//!
//! The control is drawn **only when something differs**, which is the old
//! shell's decision kept: a Reset sitting there permanently implies a change
//! that has not happened (`RIBBON_IA.md` P3 — an unavailable capability
//! renders nothing).
//!
//! "Differs" is computed by comparing [`crate::app::state::OpenDoc::hidden_layers`]
//! against `pdfcer_core::annot::optional_content_default_off`, **not** by
//! counting clicks the way the old shell did. That is a deliberate
//! improvement: a layer switched off and back on again agrees with the
//! document, and an operator asked "how many did you change?" would say
//! none. `OpenDoc` exposes no "is an override in force?" predicate, so this
//! is also the only answer reachable from a panel — and it happens to be the
//! better one.
//!
//! # A note on `/Locked` rows
//!
//! Drawn as a **disabled** control, not as an absent one, which is the old
//! shell's call and survives review: the widget is a *state display* that
//! happens not to be interactive, not a stub for a capability the build
//! lacks. Every other row has a tick, and a locked row with nothing where the
//! tick goes reads as a rendering fault.
//!
//! One defect from the salvage source is fixed here: it attached the
//! explanation with `on_hover_text` alone, and **egui does not show the
//! ordinary hover text of a disabled widget**. So the one row whose whole
//! problem is that it looks broken was silent about why. Both are attached
//! now, the same fix [`crate::panels::bookmarks`] made for its disabled rows.

/// Only [`settle`] and the tests below need a set type of their own; the
/// panel body works entirely with the one `OpenDoc::hidden_layers` hands it.
#[cfg(test)]
use std::collections::BTreeSet;

use pdfcer_core::layers::Layers;
use pdfcer_core::object::ObjId;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::panels as t;

/// Draw the Layers panel.
pub fn body(ui: &mut egui::Ui, doc: &OpenDoc, _state: &mut PanelsState, actions: &mut Vec<Action>) {
    let view = doc.session.view();
    let read = pdfcer_core::layers::read_layers(&view);

    if read.diagnostics.no_optional_content {
        ui.label(t::layers_none());
        return;
    }
    ui.label(t::layers_count(read.layers.len()));
    ui.label(
        egui::RichText::new(t::layers_session_only_note())
            .small()
            .weak(),
    );

    // The set the page is ACTUALLY drawn from — the operator's override if
    // there is one, and otherwise the document's own answer. Every row's tick
    // reads from this rather than from `visible_by_default`, or a checkbox
    // would tick itself back the moment the panel repainted.
    let effective_hidden = doc.hidden_layers();
    // What the document asks for, resolved by the same function the renderer
    // would have used. Held separately so the panel can say how far the two
    // have diverged; see the module docs on why this beats counting clicks.
    let document_hidden = pdfcer_core::annot::optional_content_default_off(&view);
    let differing = effective_hidden
        .symmetric_difference(&document_hidden)
        .count();

    // Offered only once there is something to undo, so the control never sits
    // there implying a change that has not happened.
    if differing > 0 {
        ui.horizontal(|ui| {
            ui.label(t::layers_overridden(differing));
            if ui
                .button(t::layers_reset_label())
                .on_hover_text(t::layers_reset_tooltip())
                .clicked()
            {
                actions.push(Action::ResetLayers);
            }
        });
    }

    // §8.11.4.4: some of these states are not the document's to state — a
    // viewer recomputes them from the magnification. The rows below show
    // what the document OPENS in, so a zoom-banded layer can read "shown"
    // while its content is off the page. Said here rather than left to be
    // discovered as a defect.
    if read.diagnostics.auto_managed_groups > 0 {
        ui.label(
            egui::RichText::new(t::layers_auto_managed(read.diagnostics.auto_managed_groups))
                .small(),
        );
    }
    ui.separator();

    // Collected while the read is borrowed, turned into actions after — the
    // actions-not-mutations discipline, and the same shape
    // `crate::panels::bookmarks` uses for its click.
    let mut toggled: Option<(ObjId, bool)> = None;

    egui::ScrollArea::vertical()
        .id_salt("layers-rows")
        .show(ui, |ui| {
            for l in &read.layers {
                // An undeclared name shows as a placeholder, never as an
                // invented one. `/Name` is Required (Table 98), so its
                // absence is a real malformation and a synthesised "Layer 3"
                // would disguise it as data from the file.
                let name = if l.name_declared {
                    l.name.clone()
                } else {
                    t::layer_unnamed().to_owned()
                };
                let effective = !effective_hidden.contains(&l.id);
                let notes = row_caveats(&read, l, effective);
                ui.horizontal(|ui| {
                    // Table 101 `/Locked`: "the UI shall not allow the
                    // visibility state to be changed". Disabled and
                    // explained, never hidden and never silently ignored —
                    // R83. `on_disabled_hover_text` as well as
                    // `on_hover_text` because egui shows neither the
                    // ordinary hover text nor any tooltip at all on a
                    // disabled widget without it, and this row's whole
                    // problem is that it looks broken.
                    let mut want = effective;
                    let cb = ui
                        .add_enabled(!l.locked, egui::Checkbox::new(&mut want, ""))
                        .on_hover_text(if l.locked {
                            t::layer_locked_tooltip()
                        } else {
                            t::layer_toggle_tooltip()
                        })
                        .on_disabled_hover_text(t::layer_locked_tooltip());
                    if cb.changed() {
                        toggled = Some((l.id, want));
                    }
                    // The state as TEXT as well as a tick (R84): never
                    // colour or a glyph alone, and it is what still says
                    // which way the document itself asked when the two
                    // disagree.
                    ui.label(if effective {
                        t::layer_visible_marker()
                    } else {
                        t::layer_hidden_marker()
                    });
                    // Every caveat that applies, hung off the name. Folded
                    // rather than written as a chain of `if`s so that adding
                    // one cannot leave the previous last line assigning to a
                    // variable nothing reads — and so that the LIST is a pure
                    // value this module can test.
                    let _row = notes
                        .into_iter()
                        .fold(ui.label(name), |r, note| r.on_hover_text(note));
                });
                crate::diag::trace(|| {
                    format!(
                        "layer-row name={:?} visible={effective} default={} locked={} registered={} intent_view={}",
                        l.name, l.visible_by_default, l.locked, l.in_default_config, l.intent_view
                    )
                });
            }
        });

    if let Some((id, visible)) = toggled {
        actions.extend(toggle_actions(&read, id, visible));
    }
}

/// Is `id` a group the document marks `/Locked`?
///
/// A linear scan rather than a map, deliberately. It runs only when a radio
/// group is being examined, over at most [`pdfcer_core::layers::MAX_LAYERS`]
/// rows, and a map built per frame to serve a handful of lookups would cost
/// more than it saved. Measure before trading back.
///
/// A member id that is not in `read.layers` answers `false` — it is a
/// dangling reference inside `/RBGroups`, which cannot be locked because it
/// is not a group. Failing closed here would silently refuse a legal toggle.
fn is_locked(read: &Layers, id: ObjId) -> bool {
    read.layers.iter().any(|l| l.id == id && l.locked)
}

/// The complete list of actions one click on `id`'s control should raise.
///
/// # What it does
///
/// - Always ends with `SetLayerVisible { group: id, visible }`.
/// - When `visible` is `true` **and** `id` is in an `/RBGroups` array, it is
///   preceded by one `SetLayerVisible { …, visible: false }` per **unlocked**
///   sibling. Table 101: at most one member of a radio group is ON.
///
/// # What it deliberately does NOT do
///
/// - **Turning a layer off does not turn a sibling on.** "At most one"
///   permits none, and choosing a replacement would be pdfcer deciding which
///   alternate the operator meant.
/// - **It does not switch off a locked sibling** — `DA-A8`, argued in the
///   module docs and disclosed by
///   [`crate::text::panels::layer_radio_locked_sibling_tooltip`].
/// - **It does not chase a group's *other* radio arrays.**
///   `Layer::radio_group` reports the **first** array a group belongs to and
///   `LayerDiagnostics::overlapping_radio_groups` counts the rest, because
///   the constraints are not jointly satisfiable and the standard is
///   permanently silent on the case (`DA-N1`). Honouring the first array is
///   the engine's own reported answer; inventing a resolution for the others
///   would be pdfcer deciding something ISO declined to.
///
/// # Why a `Vec<Action>` rather than one action carrying a set
///
/// See the module docs: they compose correctly because `apply` runs them in
/// order and each one recomputes from the *current* hidden set, and keeping
/// one `Action` per layer that moved is what keeps "what changed this layer?"
/// answerable by grep.
///
/// The clicked layer goes **last** so that a group whose array (legally)
/// lists the clicked group among its own members cannot end with the
/// sibling sweep switching off the very layer being switched on.
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
///
/// A pure function over one [`pdfcer_core::layers::Layer`] and the effective
/// state, so the *set* of caveats a row carries is testable without an egui
/// context — which matters because each one exists to stop an operator
/// concluding "pdfcer got it wrong", and a row that silently loses its
/// explanation looks exactly like a row that never needed one.
///
/// Order is meaningful: the operator's own change comes first, because if
/// they changed this row that is the explanation they are looking for.
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
///
/// [`pdfcer_core::layers::Layer::radio_group`] is an *index* into
/// [`pdfcer_core::layers::Layers::radio_groups`] rather than a member list, so
/// every use of it is this two-step. Written once, in a trait, so the two
/// places that need it (the tooltip decision and the toggle composition)
/// cannot come to disagree about which array a layer belongs to.
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
///
/// Test-only, and it exists so the radio tests can assert on the *outcome* of
/// a click rather than on the shape of the action list. Mirrors
/// `OpenDoc::set_layer_visible`'s arithmetic exactly — insert to hide, remove
/// to show — which is the thing being modelled.
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

    /// **★ Precondition 2 is satisfied: a layer toggle invalidates the
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

    /// **★ Precondition 3: the click reaches `apply`, and the whole round
    /// trip lands on the page.**
    ///
    /// The end-to-end statement the other tests only approach: compose the
    /// click the way [`super::body`] does, hand the result to the real
    /// [`crate::app::actions::Action`] machinery via the mutator `apply`
    /// calls, and assert the effective hidden set moved AND the render key
    /// with it.
    ///
    /// Without this, every piece could be individually correct and the panel
    /// still inert — which is precisely the S3 state, and it took a person
    /// reading the file to notice.
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
    ///
    /// Core API trap T-12.9 in one assertion, on a fixture that actually
    /// declares a layer off. `Action::ResetLayers` maps to
    /// `OpenDoc::reset_layers`, and the failure it guards against is a
    /// plausible one — `set_hidden_layers(BTreeSet::new())` reads like
    /// "clear the override" and means "reveal every layer the document
    /// turns off", which on a drawing with a "Confidential" watermark is a
    /// disclosure event.
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
    ///
    /// Pins the predicate [`super::body`] uses, since the control's presence
    /// is the only signal that an override is in force at all — `OpenDoc`
    /// exposes no "is it overridden?" accessor.
    ///
    /// The second half is the part the old shell got subtly wrong: it
    /// counted *clicks*, so a layer switched off and back on left the panel
    /// claiming a difference that no longer existed. Comparing sets says
    /// what an operator would say.
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
    ///
    /// Table 101's whole content, against the fixture built for it. The
    /// failure this prevents is not cosmetic: on a CAD drawing the members of
    /// an `/RBGroups` array are mutually exclusive alternates, so two of them
    /// on means two title blocks painted over each other, and the operator
    /// has no way to know pdfcer did that rather than the document.
    ///
    /// The sibling list is taken from [`RadioGroupLookup::radio_group_of`],
    /// not from whichever array the search happened to walk. That is not
    /// fussiness — the first draft of this test iterated `radio_groups`
    /// directly and failed, because this fixture's shared member belongs to
    /// **two** arrays and `radio_group` reports only the first (`DA-N1`). A
    /// test that asserts against a different array from the one the panel
    /// consults is testing nothing the panel does.
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
    ///
    /// pdfcer's answer to a conflict the standard leaves open, asserted rather
    /// than merely written down — see the module docs for why the lock wins.
    ///
    /// Note what it asserts: not that the group ends up legal, but that no
    /// action *names* the locked layer. A click on one row must never change
    /// a layer the document told the interface not to touch, and the failure
    /// mode is a silent one: it would look like the radio rule working.
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
    ///
    /// "At most one" permits none. Choosing a replacement would be pdfcer
    /// deciding which alternate the operator meant — the class of invention
    /// rule 4 forbids — and it would do so at the exact moment the operator
    /// asked to see *less*.
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
    ///
    /// `DA-N1`: a group may legally appear in more than one `/RBGroups`
    /// array, the standard never says what a reader does with it, and the
    /// constraints are not jointly satisfiable. `pdfcer_core` answers with
    /// "the first array, plus a count of the overlaps", and this panel
    /// carries that answer through rather than inventing a resolution.
    ///
    /// The fixture is built for exactly this — two inner arrays sharing a
    /// member. The invariant asserted is the strong, general one: **no click
    /// ever names a layer outside the array core reported for it**, checked
    /// for every clickable row rather than for a hand-picked one. A version
    /// that hunted for the shared member and asserted about that row alone
    /// would pass on a fixture whose shared member happened to be the locked
    /// one, and prove nothing.
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
    ///
    /// R84 — never colour alone. With the checkbox back these are no longer
    /// the *only* state cue, which weakens the argument not at all: a tick is
    /// a glyph, so a panel whose state was carried by the tick alone would be
    /// exactly the colour-only failure R84 names, one substitution along.
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
    ///
    /// Eight of them now: the six per-layer caveats, both arms of the
    /// override tooltip, and the ordinary toggle tooltip they share a row
    /// with. Each exists because without it the operator's only available
    /// reading of a surprising row is "pdfcer got it wrong". Two that read
    /// alike would send them looking for the wrong cause — and the
    /// design-intent one in particular explains a row that *contradicts the
    /// file's own `/OFF` array*, which is the most alarming thing this panel
    /// can show.
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
    ///
    /// The caveat set is what the operator reads to find out why a row looks
    /// wrong, so a row that explains something that did not happen is as bad
    /// as one that explains nothing.
    ///
    /// Driven off `Layer::visible_by_default` rather than off the hidden set,
    /// because that is the field [`super::row_caveats`] actually branches on
    /// — a test that reconstructs the same answer by another route is one
    /// more thing that can disagree.
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
    ///
    /// A blanket warning on every radio row would train the operator to
    /// ignore it, and the row it matters on is the one where two members of a
    /// mutually exclusive group can end up showing at once.
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
    ///
    /// `/Name` is Required, so its absence is a real malformation. A
    /// synthesised "Layer 3" would disguise a defect in the file as data
    /// from it.
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
