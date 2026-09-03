//! # `panels::dimension_groups` — where dimension groups are made, chosen and
//! configured
//!
//! ## The gap this closes
//!
//! `measure.manage_groups` was registered, drawn on Measure ▸ Scale, listed in
//! `shell::commands::reach`'s `SCAFFOLDED` set, and **inert for the whole life
//! of this build**. The operator hit it by name on 2026-08-18: *"I still can't
//! get to edit dimension groups when I click on it."*
//!
//! The recorded blocker was *"needs a window, not an arm"* plus *"two of four
//! verbs do not exist"*, and re-measuring it on 2026-08-18 found the second
//! half had shrunk. Of the six things a group manager wants to do, **four are
//! shipped engine verbs**:
//!
//! | | verb | `edit.rs` |
//! |---|---|---|
//! | create | `add_dimension_group` | 17692 |
//! | calibrate | `set_group_scale` | 17718 |
//! | drafting standard | `set_group_standard` | 18220 |
//! | appearance defaults | `set_group_style` | 18289 |
//! | show / hide the layer | `toggle_dimension_layer` | 17769 |
//! | rename | `rename_dimension_group` | 19295 |
//! | delete | `delete_dimension_group_with` | 19360 |
//!
//! ## ★ The control that was missing from the whole feature, not just from this
//! surface
//!
//! `MeasureState::group` — *"the active authoring group the next dimension
//! joins"* — has existed since the Phase 7 salvage, is documented as *"ui-spec
//! §2.6 group picker"*, is seeded to `DEFAULT_GROUP_ID`, and **nothing in this
//! build ever wrote to it**. So a second group could be created from the CLI,
//! carry its own scale, and be joinable by nothing: every dimension the shell
//! authored went into the default group, forever.
//!
//! The *Draw into* column is that picker. It is the first control here not
//! because it is the most elaborate but because without it every other control
//! governs a group nothing can reach.
//!
//! ## ★★ Why this is a PANEL, and why it was a window until 2026-08-19
//!
//! It shipped as an [`egui::Window`] on 2026-08-18, on [`crate::dialogs`]'
//! own test — *a dialog is one transaction with a start and an end; a panel is
//! somewhere an operator dips in and out of while working* — and the argument
//! given was that setting up a drawing's groups *"happens once at the start of
//! a sheet and then not again for hours."*
//!
//! **That was wrong on the facts and the operator said so.** His words, of
//! 2026-08-19:
//!
//! > *"the groups editor popup is too long for some screens so can't close it
//! > … should come up in the side bar and be scrollable and each section should
//! > be able to fold up like the settings one"*
//!
//! Three separate findings are packed into that sentence and each of them
//! outranks the taxonomy argument:
//!
//! 1. **A window taller than the screen cannot be closed.** Its title bar can
//!    leave the desktop, and with it the only ✕. That is not a layout
//!    complaint, it is a **trap** — the surface captures the operator and the
//!    application offers no way back. A dock panel cannot do it: the dock
//!    bounds its own body and the tab strip carrying the close control is
//!    pinned to the top of that body, on screen, always.
//! 2. **The content is long and stays long.** Six subject blocks for the
//!    selected group, plus the list, plus the new-group controls. A surface
//!    whose natural height exceeds a laptop screen wants a **scroll region with
//!    a bounded parent**, and a dock column is exactly that. A free-floating
//!    window sized to its content is not — see the growth loop recorded below.
//! 3. **The sections must fold**, and the model he named is
//!    [`crate::dialogs::settings`]. That is the same reasoning that module's
//!    header gives for its own seven groups: an operator arrives with a
//!    *symptom* — "this group's arrowheads are wrong", "this one is measuring
//!    in inches" — and headings are how a symptom finds its control. Folding
//!    turns a 700 pt column into a six-line table of contents.
//!
//! The taxonomy is therefore **amended openly** rather than quietly bent. The
//! dock's own test in `crate::app::modes` is *"selection state is watched,
//! workflows are entered"*, and group setup is watched: which group the next
//! ce dimension joins is a fact an operator consults while drawing, in the same
//! breath as which layer is visible. That is the Layers panel's question with a
//! different noun, and Layers has never been a window.
//!
//! ## ★ The growth loop the window had, kept because it is the reason
//!
//! The window's body was a vertical `ScrollArea` and the window was given a
//! `default_width` with no height, so it sized itself to its content. The
//! scroll area asked for the height of everything inside it; the window grew to
//! fit; the scroll area got more room and asked for more. A feedback loop
//! between a measured size and the thing being measured — `D:/dev/rag/egui/`'s
//! R128 in a second place. The driven run caught `dimension-groups.new_name`
//! laid out at y=958 inside a body that ended at y=793: **the Add button was
//! rendered below the bottom of the window and could not be clicked.**
//!
//! A `default_size` with a height stopped it. This landing removes the
//! condition instead: **a dock panel's height is the dock's, decided before the
//! body draws**, so no content this module lays out can influence it. The class
//! of defect is gone rather than tuned, which is the difference `CONTINUE.md`
//! §7 asks for — *"reserve-and-hope is the same defect with a tuning
//! parameter."*
//!
//! ## ★ Which folds start open — one of six, and the rule behind it
//!
//! **Scale and unit** alone. Everything else — Add a group, Rename or remove,
//! Drafting standard, Layer, Appearance defaults — starts shut.
//!
//! The rule is *what does an operator need to READ without asking*, not *what
//! do they most often change*. Those are different questions and the second one
//! is the trap: Appearance is the most-changed section and the longest, and
//! opening it by default is precisely what made this surface taller than the
//! screen. A fold that is open because the content behind it is popular is a
//! fold that is never closed.
//!
//! What has to be readable at a glance is **which group is which**, and the
//! list answers that — with the scale phrase already on every row. The Scale
//! and unit fold stays open because it is the one section that is *about the
//! selected row's identity* rather than about editing it: an operator who has
//! just clicked a row is asking "what is this group calibrated to", and making
//! them click again to find out would be the panel answering a question with a
//! question.
//!
//! Folds persist in `egui::Memory` for the session, keyed by the section's
//! stable key, so this is a *starting* arrangement and not a policy. An
//! operator who works in Appearance for an hour opens it once.
//!
//! ## Everything is inside the one `ScrollArea`, and that is deliberate
//!
//! `CONTINUE.md` §7's application-side pattern, which recurred four times in
//! one day: *"a control that must be reachable cannot be placed after an
//! unbounded `ScrollArea`."* The window's answer was to hoist Add and Close out
//! of the scroll area and reserve a footer for them. This panel has **no
//! footer**: there is no Close button, because the dock tab carries one, and
//! Add sits at the bottom of the scroll region inside a fold of its own. With
//! nothing after the scroll area there is nothing to be pushed off the end of
//! it, and no `FOOTER_RESERVE` constant to be tuned wrong.
//!
//! ## What it does NOT do, deliberately
//!
//! - **It does not pick which group a placed ce dimension belongs to.** There
//!   is no engine verb for that
//!   (`request_a_placed_ce_dimension_cannot_be_moved_to_another_group.md`), and
//!   more importantly it is a *per-ce-dimension* question — it belongs on the
//!   selection surface, beside the other per-ce-dimension overrides.
//! - **It does not set a per-ce-dimension anything.** Every control here is
//!   group-scoped, which is what makes the reach-backwards disclosure on each
//!   of them true and uniform.
//! - **It does not offer a scale field.** The Set-scale window already exists,
//!   already owns the two entry paths and the calibration gesture, and already
//!   raises the one action. A second scale entry here would be a second
//!   implementation of the hardest arithmetic in the feature — see
//!   [`DimensionGroupsUi::take_scale_request`] for how the button hands over.

/// ★ Renaming a group and removing one — the two controls this window shipped
/// WITHOUT on 2026-08-18, with a sentence where they should have been.
///
/// Its header carries the interesting half: deleting a populated group is the
/// **orphan question**, the engine refuses by default with the member count in
/// the refusal, and putting that question in front of an operator is a thing
/// only a surface can do.
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
///
/// Indexed by the **`GroupId`**, not by the row's position in the list. A row
/// index would change under a check the moment a group was added, which is
/// exactly what a check that adds a group is doing.
pub const REGION_DRAW_INTO_PREFIX: &str = "dimension-groups.draw_into."; // ui-text-exempt: trace region name, never displayed
/// The prefix of the per-row name regions — what a check clicks to make a group
/// the one the lower half of the window is configuring.
///
/// Distinct from [`REGION_DRAW_INTO_PREFIX`] beside it, and the distinction is
/// the window's own: the radio chooses where the **next dimension** goes, the
/// name chooses which group's **settings are on screen**. Collapsing them would
/// make inspecting a group silently redirect the next dimension drawn.
pub const REGION_ROW_PREFIX: &str = "dimension-groups.row."; // ui-text-exempt: trace region name, never displayed

/// The Manage-dimension-groups window's live state.
///
/// Existence is the "open" state, as everywhere in [`super`] — there is no
/// `open: bool` that could disagree with whether the state exists.
///
/// ★ **Almost nothing is held here**, and that is the design. The groups, their
/// scales, standards, styles and member counts are all read from
/// `EditSession::dimension_model()` on every frame. A local copy would be a
/// second source of truth for a model that this very window edits through an
/// action queue applied *after* the frame — so the copy would be stale for
/// exactly one frame after every change the operator made, which is the frame
/// they are looking at.
///
/// What is held is the four things the *document* does not know: which row the
/// operator is configuring, what they have typed into the new-group fields, and
/// the two one-shot requests that have to survive past the window closure.
/// The region a fold heading publishes; the section's stable key is appended.
///
/// The key is deliberately **not** derived from the caption, for the reason
/// [`crate::dialogs::settings::widgets::group`] records: a caption is operator
/// copy and may be reworded, and a check aimed at a region named after it would
/// then report a heading that is not there rather than a heading that is
/// illegible. Those are different verdicts and only one of them is true.
pub const REGION_HEADING_PREFIX: &str = "dimension-groups.heading."; // ui-text-exempt: trace region name, never displayed

/// The Manage-dimension-groups panel's live state.
///
/// ★ **Almost nothing is held here**, and that is the design. The groups, their
/// scales, standards, styles and member counts are all read from
/// `EditSession::dimension_model()` on every frame. A local copy would be a
/// second source of truth for a model that this very panel edits through an
/// action queue applied *after* the frame — so the copy would be stale for
/// exactly one frame after every change the operator made, which is the frame
/// they are looking at.
///
/// What is held is the four things the *document* does not know: which row the
/// operator is configuring, what they have typed into the new-group fields, and
/// the one-shot request that has to survive past the frame that raised it.
///
/// # ★ Why `Default` rather than a constructor taking the authoring group
///
/// As a window this was built by `open(active)` and seeded its selection with
/// the group the operator was drawing into — *"an operator who opens this while
/// working has a group in mind and it is the one they are drawing into."* That
/// reasoning is still right and is still honoured, but it cannot live in a
/// constructor any more: a panel is not constructed when it is shown. It lives
/// on [`crate::panels::PanelsState`] for the life of the document and is reset
/// by `forget_document`, exactly like the Redact panel's query and the
/// Bookmarks panel's draft.
///
/// So [`Self::selected`] is an `Option` and the seeding happens on the first
/// frame that draws — see [`Self::show`]. `None` means *"whatever the operator
/// is drawing into"*, which is a better default than any `GroupId` because it
/// keeps following them until they say otherwise.
pub struct DimensionGroupsUi {
    /// **How far the last frame's content ran past the dock column**, in
    /// points. Zero or negative is the healthy state.
    ///
    /// ★ It exists because the defect it measures is **invisible**. A
    /// `ScrollArea::vertical()` clips horizontally and offers no bar in that
    /// axis, so a row wider than the column is simply cut off: no overflow
    /// indicator, no scroll, nothing on screen to say a control is out there.
    /// The operator's report on 2026-08-20 was *"part of the control is hidden
    /// … there's no scroll bar to show the part that is missing"*, and there
    /// was no number anywhere in the application that could have contradicted
    /// a claim that the panel was fine.
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
    /// ★ The `GroupId` is held **with** the text, not inferred from
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
    /// ★ **A request rather than a call**, and the reason changed shape when
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
    /// ★ Hand-written, and `pdfcer_core::dimension::Unit` is why.
    ///
    /// `Unit` implements no `Default`, deliberately — the engine declines to
    /// have an opinion about which unit a drawing is in, which is exactly the
    /// stance `crate::dialogs::settings`' header describes for every ambiguity
    /// the spec leaves open. So the surface must have one, and a surface's
    /// choice is a statement about *this operator's* drawings rather than about
    /// the type.
    ///
    /// The rest is `Option::None` and an empty `String`, which a derive would
    /// have given for free. It is worth the eleven lines to keep the one real
    /// decision visible instead of buried in a field initialiser.
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
    ///
    /// Called by `PdfcerApp::docks` immediately after the dock draws. Returning
    /// it rather than acting on it is what keeps this module free of any
    /// knowledge of the dialog layer.
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
    ///
    /// # ★ One `ScrollArea` and nothing after it
    ///
    /// The single most important line of layout in this file. `CONTINUE.md`
    /// §7: *"a control that must be reachable cannot be placed after an
    /// unbounded `ScrollArea`, and reserve-and-hope is the same defect with a
    /// tuning parameter."* Four surfaces shipped that defect in one day. This
    /// one has no footer at all — the dock tab carries the close control and
    /// the Add button lives inside a fold at the bottom of the scroll region —
    /// so there is nothing that *can* be pushed past the end.
    ///
    /// `auto_shrink([false, false])` so the region claims the dock column even
    /// when its folded content is six lines high. Without it a fully folded
    /// panel would shrink to a stripe and the tab would look half-drawn.
    fn show(&mut self, ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) {
        // ★ Read once per frame, and read from the SESSION rather than from any
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

        // ★★ **The overflow measurement, and why a panel takes one.**
        //
        // Operator, 2026-08-20: *"the measuring tool group option changes the
        // width of the side bar so that part of the control is hidden. there's
        // no scroll bar to show the part that is missing."*
        //
        // He was right, and the mechanism is worth writing down because it is
        // silent by construction. A `ScrollArea::vertical()` clips
        // **horizontally** and offers no bar in that axis, so a row wider than
        // the dock column does not overflow visibly, does not scroll, and does
        // not report anything — it is simply cut off at the right edge. The
        // control that ends up outside is unreachable and there is nothing on
        // screen to say it exists.
        //
        // The row that did it: `"no scale set — showing raw page units"`
        // followed by the **Set scale…** button, in a `ui.horizontal` — about
        // 310 pt of content in a 250 pt column. The rows in this panel are
        // `horizontal_wrapped` now, so the button drops to the next line
        // instead of off the edge.
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
                // ★ **Adding comes directly under the LIST, above the selected
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

        // ★ How far the content ran past the column, if it did. `<= 0` is the
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
    ///
    /// # Why the row shows the scale and the member count
    ///
    /// Because those are the two things that tell an operator *which group this
    /// is* when the names are `Plan` and `Detail` and they set them up an hour
    /// ago. A list of names alone would be a list of words.
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

        // ★★ **A BLOCK PER GROUP, NOT A GRID ROW** — 2026-08-20, on the
        // operator's report.
        //
        // This was a four-column `egui::Grid`: radio | name | member count |
        // scale phrase. In a dock column that does not fit and cannot be made
        // to. The scale phrase alone is a sentence —
        // `"no scale set — showing raw page units"`, about 200 pt — and the
        // row totalled some 390 pt against the navigator's ~250. A `Grid` does
        // not wrap, a `ScrollArea::vertical()` offers no horizontal bar, so the
        // right-hand columns were **cut off with nothing to say they existed**:
        //
        // > *"the measuring tool group option changes the width of the side bar
        // > so that part of the control is hidden. there's no scroll bar to
        // > show the part that is missing."*
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
        // `no_row_in_this_panel_outruns_a_narrow_dock` is the falsifier, and it
        // failed at 209 pt against this grid before the change.
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
        // ★ Folded shut, and this is the one section where that is a *safety*
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
            // ★ Through `set_group_scale`, because a unit lives inside the group's
            // `NumberFormat` and there is no narrower verb. The engine's reply of
            // 2026-08-19 called that *"a discoverability problem, not a missing
            // capability"* and declined to add sugar for it on speculation, which
            // is right — the path works, it is just not obvious, and making it
            // obvious is a surface's job rather than an API's.
            //
            // The group's **scale is carried through unchanged**. That is the whole
            // subtlety: `set_group_scale` takes both, so passing anything but the
            // group's own `scale` here would silently recalibrate a drawing while
            // the operator was changing a unit — a far larger act than the one they
            // asked for, in a control that does not mention it.
            ui.horizontal_wrapped(|ui| {
                ui.label(t::unit_label());
                let mut unit = group.format.unit;
                egui::ComboBox::from_id_salt("dimension-group-unit")
                    .selected_text(crate::text::scale::unit_name(unit))
                    .show_ui(ui, |ui| {
                        for option in Unit::all() {
                            ui.selectable_value(
                                &mut unit,
                                option,
                                crate::text::scale::unit_name(option),
                            );
                        }
                    });
                if unit != group.format.unit {
                    actions.push(Action::Dimension(DimensionAction::SetGroupScale {
                        group: group.id,
                        scale: group.scale,
                        // ★ `default_format` rather than mutating the group's own
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
            // ★ The whole group moves, always — the standard has no per-ce-dimension
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
    /// Split from [`Self::add_group`] rather than written as a closure body so
    /// that `self` is borrowed once, by a method call, instead of captured by a
    /// closure that also needs `actions` — which is the borrow the compiler
    /// refuses.
    fn add_group_body(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        ui.horizontal_wrapped(|ui| {
            ui.label(t::new_name_label());
            let response =
                ui.add(egui::TextEdit::singleline(&mut self.new_name).desired_width(160.0));
            crate::diag::ui_rect(REGION_NEW_NAME, response.rect);

            ui.label(t::new_unit_label());
            egui::ComboBox::from_id_salt("dimension-groups-new-unit")
                .selected_text(crate::text::scale::unit_name(self.new_unit))
                .show_ui(ui, |ui| {
                    // ★ `Unit::all()`, not a hand-written array. The engine's own
                    // doc for it says *"the GUI unit dropdown and the CLI unit
                    // parser iterate this"*, and `NO_SURFACE.md`'s sweep found a
                    // local copy in `dialogs::scale` that happened to match —
                    // a latent divergence rather than an active one, and this is
                    // the version that cannot acquire it.
                    for unit in Unit::all() {
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
            // ★★★ **`on_disabled_hover_text`, since 2026-08-31** —
            // `OPERATOR_REQUESTS.md` O77's sweep.
            //
            // This read `on_hover_text`, and in egui 0.35 that builds
            // `Tooltip::for_enabled`, which opens only when
            // `response.enabled()` — so on a response that is already disabled
            // it runs no content and paints nothing. The comment above
            // promised *"greyed WITH an explanation"* and there was no
            // explanation: the control was greyed, silent, and unexplainable
            // by hovering, which is R9 breached by a one-word method name.
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
///
/// The operator asked for it by name — *"each section should be able to fold up
/// like the settings one"* — and [`crate::dialogs::settings`]'s `widgets::group`
/// is the model. This is a second implementation rather than a call to that one
/// for a single reason worth stating: that function publishes its rect under
/// `settings.heading.<key>`, and a check aimed at the settings window would then
/// find headings belonging to a panel in a different surface entirely. A shared
/// helper taking a prefix would be the tidier answer and is worth doing the day
/// a third surface wants folds; two is not yet a pattern.
///
/// ★ `ui_rect_visible`, not `ui_rect`, for the reason the settings window
/// learned the hard way: these headings live in a `ScrollArea`, `egui` lays out
/// the ones below the fold before clipping them, and publishing a rect for a
/// heading nobody can see makes a contrast check measure whatever is genuinely
/// at those coordinates — which on the settings window's first live run was the
/// Pages panel and the drawing behind the dialog.
///
/// The caption is **plain text**, never `.strong()`. `DEFECTS.md` D11: no theme
/// this project ships renders `.strong()` legibly on a panel, and
/// `tools/gates/check-strong-text.sh` refuses a bare one. The disclosure
/// triangle beside the caption is the whole of the emphasis and it is enough.
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
///
/// The entry point [`crate::panels::Panel::show`] calls, in the shape every
/// panel body has: the empty-document case never arrives here, because it is
/// answered once for all panels rather than eleven times.
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
    ///
    /// `crate::app::modes::defaults::NAVIGATOR_WIDTH` is 280; the panel gets
    /// that less the dock's own margins and the scroll bar it reserves. 250 is
    /// the number `panels::pages`' own column test uses for the same reason.
    const NARROW: f32 = 250.0;

    /// ★★ **No row in this panel outruns a narrow dock.**
    ///
    /// The operator, 2026-08-20: *"the measuring tool group option changes the
    /// width of the side bar so that part of the control is hidden. there's no
    /// scroll bar to show the part that is missing."*
    ///
    /// He was right, and the defect is **invisible by construction**, which is
    /// why it needed a number rather than a look. A `ScrollArea::vertical()`
    /// clips horizontally and offers no bar in that axis: a row wider than the
    /// column is cut off at the right edge, does not scroll, and reports
    /// nothing. The control that ends up outside is unreachable and there is
    /// nothing on screen to say it exists.
    ///
    /// The row that did it was `"no scale set — showing raw page units"`
    /// followed by the **Set scale…** button — about 310 pt of content in a
    /// 250 pt column, in a `ui.horizontal`, which does not wrap. Every row in
    /// this panel is `horizontal_wrapped` now.
    ///
    /// ## Why the assertion is on a measured overflow rather than on a
    /// screenshot
    ///
    /// Because the panel can measure itself exactly — `content_size.x` against
    /// the scroll viewport's width — and a number that the application
    /// computes is a better oracle than a rendering a test has to interpret.
    /// The screenshot rule (`D:/dev/rag/egui/`) is about *reachability*
    /// defects a trace cannot see; this one the application can see, so it is
    /// made to say so.
    ///
    /// ## What it does NOT prove
    ///
    /// That every control is legible, or that wrapping put things somewhere
    /// sensible. It proves nothing is off the edge, which is the operator's
    /// complaint exactly.
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
