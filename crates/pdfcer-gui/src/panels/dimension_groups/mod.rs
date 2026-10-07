//! # `panels::dimension_groups` — where dimension groups are made, chosen and
//! configured
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/dimension_groups/mod.md`.

/// Renaming a group and removing one.
mod identity;
mod style;

use egui::Ui;
use pdfcer_core::dimension::{DEFAULT_GROUP_ID, DimStandard, GroupId, Unit};

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::OpenDoc;
use crate::text::dimension_groups as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "panel:dimension-groups"; // ui-text-exempt: trace region name, never displayed

/// Trace slot for the once-per-change width report. See
/// [`DimensionGroupsUi::overflow_x`].
const OVERFLOW_SLOT: &str = "dimension-groups-width"; // ui-text-exempt: trace slot name, never displayed
/// The region the *Add group* button publishes.
pub const REGION_ADD: &str = "dimension-groups.add"; // ui-text-exempt: trace region name, never displayed
/// The region the new-group name field publishes, so a driven check can type
/// into it.
pub const REGION_NEW_NAME: &str = "dimension-groups.new_name"; // ui-text-exempt: trace region name, never displayed
/// The prefix of the per-row *Draw into* radio regions; the group's numeric id
/// is appended.
pub const REGION_DRAW_INTO_PREFIX: &str = "dimension-groups.draw_into."; // ui-text-exempt: trace region name, never displayed
/// The prefix of the per-row name regions — what a check clicks to make a group
/// the one the lower half of the window is configuring.
pub const REGION_ROW_PREFIX: &str = "dimension-groups.row."; // ui-text-exempt: trace region name, never displayed

/// The Manage-dimension-groups window's live state.
pub const REGION_HEADING_PREFIX: &str = "dimension-groups.heading."; // ui-text-exempt: trace region name, never displayed
/// The group unit combo.
pub const REGION_UNIT_COMBO: &str = "dimension-groups.unit.combo"; // ui-text-exempt: trace region name, never displayed
/// One unit in the open combo, suffixed with `Unit::token`.
pub const REGION_UNIT_OPTION_PREFIX: &str = "dimension-groups.unit.option."; // ui-text-exempt: trace region name, never displayed

/// The Manage-dimension-groups panel's live state.
pub struct DimensionGroupsUi {
    /// **How far the last frame's content ran past the dock column**, in
    /// points. Zero or negative is the healthy state.
    ///
    /// It exists because the defect it measures is **invisible**. A
    /// `ScrollArea::vertical()` clips horizontally and offers no bar in that
    /// axis, so a row wider than the column is simply cut off: no overflow
    /// indicator, no scroll, nothing on screen to say a control is out there.
    /// Part of a control goes missing with no scroll bar to show it, and
    /// without this number nothing in the application can contradict a claim
    /// that the panel is fine.
    ///
    /// Read by [`tests::no_row_in_this_panel_outruns_a_narrow_dock`] and traced
    /// once per change, so the same thing is checkable in a unit test and
    /// visible in a driven run.
    overflow_x: f32,
    /// The group whose settings the lower half of the panel is showing, or
    /// `None` to follow the authoring group.
    ///
    /// Distinct from the **authoring** group (the *Draw into* radio), and the
    /// distinction is worth the extra state: an operator setting up a detail
    /// group's appearance while still drawing into the plan group is an
    /// ordinary thing to want, and collapsing the two would make inspecting a
    /// group's settings silently redirect the next dimension they draw.
    selected: Option<GroupId>,
    /// What has been typed into the new-group name field.
    new_name: String,
    /// The rename draft for [`Self::selected`], and which group it is for.
    ///
    /// The `GroupId` is held **with** the text, not inferred from
    /// [`Self::selected`], and that is what stops a half-typed rename following
    /// the operator to a different row. Selecting another group makes the pair
    /// stale, [`Self::rename_draft_for`] notices, and the field re-seeds from
    /// the group actually on screen — rather than offering to rename *this*
    /// group to a name meant for the last one.
    rename: Option<(GroupId, String)>,
    /// Where a populated group's members would go if it were deleted.
    ///
    /// `None` until the operator presses Delete on a group that has members —
    /// so the destination picker is **absent** rather than sitting under every
    /// row, which is R9's rule and also the honest layout: it is a question
    /// nobody has been asked yet.
    delete_destination: Option<GroupId>,
    /// The unit the new group would start in.
    ///
    /// Millimetres, because it is the unit this operator's drawings are in and
    /// a unit is one combo away for anybody whose are not. See [`Self::default`]
    /// for why that is a hand-written `Default` rather than a derived one.
    new_unit: Unit,
    /// Set by the *Set scale…* button, drained by `crate::app::PdfcerApp`.
    ///
    /// **A request rather than a call**, and the reason changed shape when
    /// this became a panel without changing conclusion. As a window it was
    /// because both windows were fields of one `DialogsState` and neither could
    /// reach the other from inside its own `show`. As a panel it is because a
    /// panel body is handed `&OpenDoc` and `&mut PanelsState` and **nothing
    /// else** — it cannot see `DialogsState` at all, which is the seam that
    /// keeps a panel from opening arbitrary windows.
    ///
    /// Drained in `PdfcerApp::docks`, immediately after the dock body closes and
    /// releases its borrows, which is the one place that can see both halves.
    /// The Set-scale window's own guards (`open_scale`'s no-document and
    /// already-open checks) stay on the one path that builds it.
    scale_requested: Option<GroupId>,
}

impl Default for DimensionGroupsUi {
    /// Hand-written, and `pdfcer_core::dimension::Unit` is why.
    fn default() -> Self {
        Self {
            // Nothing has been laid out yet, so nothing has overflowed.
            overflow_x: 0.0,
            selected: None,
            new_name: String::new(),
            rename: None,
            delete_destination: None,
            new_unit: Unit::Millimeter,
            scale_requested: None,
        }
    }
}

impl DimensionGroupsUi {
    /// Take the pending *Set scale…* request, if the operator pressed it.
    pub fn take_scale_request(&mut self) -> Option<GroupId> {
        self.scale_requested.take()
    }

    /// How far the last frame's content ran past the column. See
    /// [`Self::overflow_x`]; `NaN` until a frame has drawn.
    #[cfg(test)]
    pub(crate) const fn overflow_for_test(&self) -> f32 {
        self.overflow_x
    }

    /// The whole panel body.
    fn show(&mut self, ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) {
        // Read once per frame, and read from the SESSION rather than from any
        // cache. `dimension_model()` clones out of the `/PieceInfo` sidecar, so
        // this is the model as the document currently stands including every
        // unsaved edit — which is what the operator is looking at.
        let model = doc.session.dimension_model();
        let ctx = ui.ctx().clone();
        let active = crate::canvas::measure::active_group(&ctx).unwrap_or(DEFAULT_GROUP_ID);

        // Seeded on the first frame that draws rather than in a constructor —
        // see the struct's own header. `None` keeps following the authoring
        // group until the operator picks a row, which is what an operator who
        // opened this while drawing meant.
        let selected = self.selected.unwrap_or(active);
        self.selected = Some(selected);
        // If the selected group was removed from under the panel, fall back
        // rather than drawing an empty lower half. Reachable since the delete
        // verb landed — and reachable from *this* panel, which is what makes it
        // worth a line rather than a comment: deleting the selected group is
        // the ordinary way to arrive here.
        if model.group(selected).is_none() {
            self.selected = Some(DEFAULT_GROUP_ID);
        }

        // **The overflow measurement, and why a panel takes one.**
        //
        // A control wider than the dock column has part of itself hidden with
        // no scroll bar to show the part that is missing, and the mechanism is
        // worth writing down because it is silent by construction. A
        // `ScrollArea::vertical()` clips
        // **horizontally** and offers no bar in that axis, so a row wider than
        // the dock column does not overflow visibly, does not scroll, and does
        // not report anything — it is simply cut off at the right edge. The
        // control that ends up outside is unreachable and there is nothing on
        // screen to say it exists.
        //
        // The row that reaches it: `"no scale set — showing raw page units"`
        // followed by the **Set scale…** button — about 310 pt of content in a
        // 250 pt column. In a `ui.horizontal` that is cut off; every row in
        // this panel is `horizontal_wrapped`, so the button drops to the next
        // line instead of off the edge.
        //
        // Wrapping is the fix; this is the **falsifier**. `content_size.x`
        // against the viewport's width is the one number that says whether it
        // has regressed, it costs nothing, and without it the next long
        // sentence added to a row here would reintroduce the defect in exactly
        // the same invisible way.
        let output = egui::ScrollArea::vertical()
            .id_salt("dimension-groups-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                crate::diag::ui_rect(REGION_BODY, ui.max_rect());
                ui.label(t::intro());
                ui.add_space(6.0);
                self.group_list(ui, &ctx, &model, active);
                ui.separator();
                // **Adding comes directly under the LIST, above the selected
                // group's settings**, and that is the window's own hard-won
                // lesson carried over rather than a fresh preference.
                //
                // In the window the Add controls sat at the bottom, under every
                // style, standard and layer control belonging to a group the
                // operator was not trying to add to — and the note that moved
                // them said why it was more than a reach problem: *"adding a
                // group is an action on the LIST, not on the selected group,
                // and a control's position is a claim about what it acts on."*
                // The window fixed the reach by hoisting Add out of the scroll
                // area into a reserved footer. This fixes the claim, which is
                // the part that was actually wrong, and gets the reach for
                // free — a fold three lines under the list cannot be pushed off
                // anything.
                self.add_group(ui, actions);
                ui.separator();
                self.selected_group(ui, &model, actions);
            });

        // How far the content ran past the column, if it did. `<= 0` is the
        // healthy state and the only one this panel may ship in.
        self.overflow_x = output.content_size.x - output.inner_rect.width();
        crate::diag::trace_changed(OVERFLOW_SLOT, || {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "dimension-groups-width content={:.0} viewport={:.0} overflow={:.0}",
                output.content_size.x,
                output.inner_rect.width(),
                self.overflow_x.max(0.0),
            )
        });
    }

    /// The list of groups: the authoring radio, the name, and the facts that
    /// distinguish one row from another.
    fn group_list(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        model: &pdfcer_core::dimension::DimensionModel,
        active: GroupId,
    ) {
        ui.label(t::groups_heading());
        ui.label(t::draw_into_hint());
        ui.add_space(4.0);

        // **A BLOCK PER GROUP, NOT A GRID ROW.**
        //
        // The four columns a group wants — radio | name | member count | scale
        // phrase — do not fit a dock column as an `egui::Grid` and cannot be
        // made to. The scale phrase alone is a sentence —
        // `"no scale set — showing raw page units"`, about 200 pt — and the
        // row totals some 390 pt against the navigator's ~250. A `Grid` does
        // not wrap, a `ScrollArea::vertical()` offers no horizontal bar, so the
        // right-hand columns are **cut off with nothing to say they existed**.
        //
        // Two lines per group instead. The controls — the authoring radio and
        // the row selector — go on the first line where they are always
        // reachable; the *facts* that distinguish one group from another go on
        // the second, small and weak, where they may wrap freely because
        // nothing there is clickable.
        //
        // That ordering is the rule worth keeping: **in a narrow column, put
        // what can be pressed on the line that cannot overflow, and what can
        // only be read on the line that can.**
        //
        // `no_row_in_this_panel_outruns_a_narrow_dock` is the falsifier, and a
        // grid of those four columns overruns it by 209 pt.
        for group in model.groups() {
            ui.horizontal_wrapped(|ui| {
                let response = ui.radio(group.id == active, "");
                crate::diag::ui_rect(
                    // ui-text-exempt: trace region name, never displayed
                    &format!("{REGION_DRAW_INTO_PREFIX}{}", group.id.0),
                    response.rect,
                );
                if response.clicked() {
                    crate::canvas::measure::set_active_group(ctx, group.id);
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed
                        format!("dimension-authoring-group id={}", group.id.0)
                    });
                }

                // The name doubles as the row selector for the lower half. A
                // separate "configure" button per row would be a second control
                // doing what clicking the row already means everywhere else in
                // this application.
                let row = ui.selectable_label(self.selected == Some(group.id), &group.name);
                crate::diag::ui_rect(
                    // ui-text-exempt: trace region name, never displayed
                    &format!("{REGION_ROW_PREFIX}{}", group.id.0),
                    row.rect,
                );
                if row.clicked() {
                    self.selected = Some(group.id);
                }
            });
            // The two facts that say *which group this is* when the names are
            // `Plan` and `Detail` and they were set up an hour ago. Indented
            // under the row they describe, and free to wrap — see above.
            ui.indent(("dimension-group-facts", group.id.0), |ui| {
                ui.small(t::member_count(model.member_count(group.id)));
                ui.small(t::scale_phrase(group.scale, group.format.unit));
            });
            ui.add_space(2.0);
        }
        ui.add_space(4.0);
    }

    /// The selected group's settings: scale, standard, layer, appearance.
    fn selected_group(
        &mut self,
        ui: &mut Ui,
        model: &pdfcer_core::dimension::DimensionModel,
        actions: &mut Vec<Action>,
    ) {
        let Some(group) = self.selected.and_then(|id| model.group(id)) else {
            return;
        };

        // --- identity: rename, and delete -------------------------------
        //
        // Folded shut, and this is the one section where that is a *safety*
        // statement rather than a length one. The two verbs it carries are the
        // only destructive ones on the panel: a rename an operator did not mean
        // is an undo entry, a delete they did not mean moves or destroys every
        // ce dimension in the group. R9 reserves greying for *temporarily*
        // unavailable, so neither may be greyed to make it feel safer; a fold
        // is the honest equivalent and costs one click to whoever came for it.
        section(ui, "identity", t::identity_heading(), false, |ui| {
            self.identity(ui, model, group, actions);
        });

        // --- scale and unit ---------------------------------------------
        //
        // One section for both, because they are one question. `set_group_scale`
        // takes the scale and the number format together — see the unit combo
        // below — so an operator who changes one is standing in front of the
        // verb that governs the other whether the layout says so or not.
        section(ui, "scale", t::scale_heading(), true, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(t::scale_phrase(group.scale, group.format.unit));
                if ui.button(t::set_scale_button()).clicked() {
                    self.scale_requested = Some(group.id);
                }
            });

            // --- unit -------------------------------------------------------
            //
            // Through `set_group_scale`, because a unit lives inside the group's
            // `NumberFormat` and there is no narrower verb. That is a
            // discoverability problem rather than a missing capability, and the
            // engine deliberately carries no sugar for it: the path works, it is
            // just not obvious, and making it obvious is a surface's job rather
            // than an API's.
            //
            // The scale is **re-expressed in the new unit**, never carried over:
            // a `Calibrated` scale is real units per point *in the group's unit*,
            // so 5 mm/pt kept as-is under feet reads 1000 mm as "1000 ft". The
            // real length every member measures must not move when only the unit
            // does (`units::scale_in_unit`).
            ui.horizontal_wrapped(|ui| {
                ui.label(t::unit_label());
                let mut unit = group.format.unit;
                let combo = egui::ComboBox::from_id_salt("dimension-group-unit")
                    .selected_text(crate::text::scale::unit_name(unit))
                    .show_ui(ui, |ui| {
                        for option in Unit::all().iter().copied() {
                            let row = ui.selectable_value(
                                &mut unit,
                                option,
                                crate::text::scale::unit_name(option),
                            );
                            crate::diag::ui_rect(
                                // ui-text-exempt: trace region name, never displayed
                                &format!("{REGION_UNIT_OPTION_PREFIX}{}", option.token()),
                                row.rect,
                            );
                        }
                    });
                crate::diag::ui_rect(REGION_UNIT_COMBO, combo.response.rect);
                if unit != group.format.unit {
                    actions.push(Action::Dimension(DimensionAction::SetGroupScale {
                        group: group.id,
                        scale: crate::units::scale_in_unit(group.scale, group.format.unit, unit),
                        // `default_format` rather than mutating the group's own
                        // `unit` field in place, because a `NumberFormat` is a unit
                        // AND how its fractional part is written — and those travel
                        // together for a reason. Millimetres in eighths, or inches
                        // to six decimal places, are formats nobody's drawing uses;
                        // carrying the old fraction mode across a unit change is
                        // how an operator gets one.
                        //
                        // The precision is per-ce-dimension overridable from the
                        // properties panel for the drawing that genuinely wants it,
                        // which is where that decision belongs.
                        format: unit.default_format(),
                    }));
                }
            });
            ui.weak(t::unit_hint());
        });

        // --- drafting standard ------------------------------------------
        section(ui, "standard", t::standard_heading(), false, |ui| {
            ui.label(t::standard_hint());
            let mut standard = group.standard;
            ui.horizontal_wrapped(|ui| {
                for option in [DimStandard::Ansi, DimStandard::Iso] {
                    ui.radio_value(&mut standard, option, t::standard_name(option));
                }
            });
            // The whole group moves, always — the standard has no per-ce-dimension
            // tier on `Group`, so no member can be following anything else. The
            // count is therefore the member count itself, and it is still shown,
            // because "all 40 will be redrawn" is exactly the sentence the operator
            // asked for and its absence here would read as "this one is different".
            let members = model.member_count(group.id);
            ui.weak(t::members_that_will_move(members, members));
            if standard != group.standard {
                actions.push(Action::Dimension(DimensionAction::SetGroupStandard {
                    group: group.id,
                    standard,
                }));
            }
        });

        // --- layer ------------------------------------------------------
        section(ui, "layer", t::layer_heading(), false, |ui| {
            if group.id == DEFAULT_GROUP_ID {
                // R9: the affordance is ABSENT, not greyed. The engine refuses to
                // hide the default group, so a switch here could never be honoured
                // — and the sentence in its place is why an omission does not read
                // as a bug.
                ui.weak(t::layer_default_group());
            } else {
                let mut visible = group.visible;
                if ui.checkbox(&mut visible, t::layer_visible()).changed() {
                    actions.push(Action::Dimension(DimensionAction::ToggleLayer {
                        group: group.id,
                        visible,
                    }));
                }
                ui.weak(t::layer_hint());
            }
        });

        // --- appearance defaults ----------------------------------------
        //
        // The longest section by a wide margin — five property rows, each with
        // its own override control and moving-count — and the one the operator's
        // report was really about: it is what made the window taller than his
        // screen. Folded SHUT, like four of the six.
        section(ui, "appearance", t::appearance_heading(), false, |ui| {
            style::show(ui, model, group, actions);
        });
    }

    /// The new-group controls.
    fn add_group(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        section(ui, "add", t::new_heading(), false, |ui| {
            self.add_group_body(ui, actions);
        });
    }

    /// The new-group controls proper, inside their fold.
    ///
    fn add_group_body(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        ui.horizontal_wrapped(|ui| {
            ui.label(t::new_name_label());
            let response =
                // escape-disposition: keeps-draft — the new group's name lives on this
                // panel and is committed by the button beside it.
                ui.add(egui::TextEdit::singleline(&mut self.new_name).desired_width(160.0));
            crate::diag::ui_rect(REGION_NEW_NAME, response.rect);

            ui.label(t::new_unit_label());
            egui::ComboBox::from_id_salt("dimension-groups-new-unit")
                .selected_text(crate::text::scale::unit_name(self.new_unit))
                .show_ui(ui, |ui| {
                    // `Unit::all()`, not a hand-written array. The engine's own
                    // doc for it says *"the GUI unit dropdown and the CLI unit
                    // parser iterate this"*, and `NO_SURFACE.md`'s sweep found a
                    // local copy in `dialogs::scale` that happened to match —
                    // a latent divergence rather than an active one, and this is
                    // the version that cannot acquire it.
                    for unit in Unit::all().iter().copied() {
                        ui.selectable_value(
                            &mut self.new_unit,
                            unit,
                            crate::text::scale::unit_name(unit),
                        );
                    }
                });
        });
        ui.weak(t::new_unit_hint());

        let name = self.new_name.trim().to_owned();
        if name.is_empty() {
            // Greying WITH an explanation: this is the *temporarily*
            // unavailable case R9 reserves it for, and the reason is one the
            // operator can act on in a single keystroke. Omitting the button
            // instead would make the name field look like it does nothing.
            let response = ui.add_enabled(false, egui::Button::new(t::new_button()));
            crate::diag::ui_rect(REGION_ADD, response.rect);
            // **`on_disabled_hover_text`, never `on_hover_text`.**
            //
            // In egui 0.35 `on_hover_text` builds `Tooltip::for_enabled`, which
            // opens only when `response.enabled()` — so on a response that is
            // already disabled it runs no content and paints nothing. The
            // promise above is *greyed WITH an explanation*; `on_hover_text`
            // here leaves the control greyed, silent and unexplainable by
            // hovering, which is R9 breached by a one-word method name.
            response.on_disabled_hover_text(t::new_needs_a_name());
        } else {
            let response = ui.button(t::new_button());
            crate::diag::ui_rect(REGION_ADD, response.rect);
            if response.clicked() {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "dimension-group-add unit={:?} chars={}",
                        self.new_unit,
                        name.len()
                    )
                });
                actions.push(Action::Dimension(DimensionAction::AddGroup {
                    name,
                    unit: self.new_unit,
                }));
                // Cleared so a second press cannot silently make a second group
                // with the same name — which the engine would accept, and which
                // would leave two indistinguishable rows in the picker.
                self.new_name.clear();
            }
        }
    }
}

/// A foldable section, and the reason this panel has them at all.
fn section(
    ui: &mut Ui,
    key: &str,
    heading: &str,
    open_by_default: bool,
    body: impl FnOnce(&mut Ui),
) {
    let response = egui::CollapsingHeader::new(heading)
        .id_salt(key)
        .default_open(open_by_default)
        .show(ui, body);
    crate::diag::ui_rect_visible(
        // ui-text-exempt: trace region name, never displayed
        &format!("{REGION_HEADING_PREFIX}{key}"),
        response.header_response.rect,
        ui.clip_rect(),
    );
    ui.add_space(2.0);
}

/// Draw the panel.
pub fn body(
    ui: &mut Ui,
    doc: &OpenDoc,
    state: &mut crate::panels::PanelsState,
    actions: &mut Vec<Action>,
) {
    state.dimension_groups.show(ui, doc, actions);
}

#[cfg(test)]
mod width_tests {
    use super::body;
    use eframe::egui;

    /// **The dock column this panel is designed for**, in points.
    const NARROW: f32 = 250.0;

    /// **No row in this panel outruns a narrow dock.**
    #[test]
    fn no_row_in_this_panel_outruns_a_narrow_dock() {
        let ctx = egui::Context::default();
        let doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);
        let mut state = crate::panels::PanelsState::default();
        let mut actions = Vec::new();
        let mut overflow = f32::NAN;

        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(NARROW, 900.0),
            )),
            ..Default::default()
        };
        // Two passes. `CollapsingHeader` state, the scroll area's own size and
        // the combo widths all settle on the second frame, and a one-frame
        // measurement of an immediate-mode layout is a measurement of its
        // first guess.
        for _ in 0..2 {
            let _ = ctx.run_ui(input.clone(), |ui| {
                body(ui, &doc, &mut state, &mut actions);
                overflow = state.dimension_groups.overflow_for_test();
            });
        }

        assert!(
            overflow.is_finite(),
            "the panel never reported a width, so this test measured nothing"
        );
        assert!(
            overflow <= 1.0,
            "the panel laid out {overflow:.0} pt wider than its {NARROW:.0} pt column"
        );
    }
}
