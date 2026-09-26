//! # `dialogs::scale` — what a dimension's number *means*
//!
//! The Set-scale dialog. `measure.set_scale` was registered, drawn on
//! Measure ▸ Scale, and inert; `shell::commands::reach` recorded it as *"the
//! clearest statement of a missing arm in the crate"*, and the block on it was
//! never the model — it was this window.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/scale.md`.
//!
//! ## conventions: dialogs
//!
//! Corpus: `ui-conventions/dialogs.md`.
//!
//! - G1 is-an-os-window: **GAP, and it is the operator's report of 2026-08-20** —
//!   *"doesn't pop up in its own movable window. It is locked within the
//!   boundaries of the program's window."* Every dialog here is an
//!   `egui::Window`, which is an in-viewport panel. egui can already do the real
//!   thing through `show_viewport_immediate`; the panel was the path of least
//!   resistance and nothing pushed back.
//! - G2 use-the-os-dialog: the file and save pickers are the system's, and
//!   `pdfcer-print` opens the native printer-properties sheet owned by our
//!   window. The dialogs in this directory are pdfcer's own because they carry
//!   choices only pdfcer has — which is the right reason to draw one, and does
//!   not excuse G1.
//! - G3 owned-by-the-app: the native pickers are; an in-viewport panel cannot be
//!   anything else. This becomes a live question the moment G1 is fixed.
//! - G4 enter-accepts-escape-cancels: **PARTIAL** — Escape closes; Enter is not
//!   wired as the affirmative default and no button is drawn as the default, so
//!   an operator who types into the last field and presses Enter gets nothing.
//! - G5 keyboard-reachable: **GAP** — egui's tab order is positional and nothing
//!   here asserts that focus starts in a sensible field or that a modal traps
//!   it.
//! - G6 remembers-position: **GAP** — anchored `CENTER_CENTER` every time, so a
//!   dialog the operator moved comes back to the middle of the window.
//! - G7 destructive-verbs-named: the unsaved-changes dialog names the file and
//!   labels its buttons with verbs rather than Yes/No.
//! - G8 cancel-is-silent: a cancelled picker is a complete, correct,
//!   uninteresting outcome and is never reported as an error.
//! - G9 nothing-blocks-silently: a native picker blocks the UI thread by design,
//!   which is what a modal file dialog is. Long work behind a pdfcer dialog is
//!   not surfaced. **GAP.**

use egui::Ui;
use pdfcer_core::dimension::{DEFAULT_GROUP_ID, DimensionModel, FractionMode, GroupId, Unit};

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::OpenDoc;
use crate::canvas::measure::scale::ScaleEntryFields;
use crate::text::dimension_groups as gt;
use crate::text::scale as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:set-scale"; // ui-text-exempt: trace region name, never displayed

/// The region the Accept control publishes.
pub const REGION_ACCEPT: &str = "dialog:set-scale.accept"; // ui-text-exempt: trace region name, never displayed

/// The Set-scale dialog's live state.
///
/// Existence is the "open" state, as everywhere in this directory — there is no
/// `open: bool` that could disagree with whether the state exists.
pub struct ScaleDialog {
    /// The group being calibrated.
    ///
    /// Seeded when the dialog opens -- from the measure tool's active authoring
    /// group when the ribbon opened it, or from the selected row when the
    /// Manage-groups panel's *Set scale...* button did.
    ///
    /// # It is now OPERATOR-changeable, and the old contract still holds
    ///
    /// This field's note used to read *"not re-read per frame: a group picker
    /// that moved underneath an open dialog would let them type a number for
    /// one group and commit it to another."* That reasoning is **still exactly
    /// right and is still honoured** -- nothing re-reads the *authoring* group
    /// while this window is up, so changing what the canvas draws into cannot
    /// redirect a calibration in progress.
    ///
    /// What changed is that the operator may now aim the window deliberately,
    /// through the picker this field backs (`OPERATOR_REQUESTS.md` O193). That
    /// is the opposite situation: a picker moving **underneath** the operator
    /// is a silent redirection, and a picker moved **by** the operator is them
    /// saying which drawing they mean. The first is what the guard was written
    /// against; the second is what the guard was accidentally preventing,
    /// because a window that named no group at all could be aimed somewhere
    /// they were not looking and never say so.
    ///
    /// Every change to this field goes through [`Self::reseed`], which
    /// re-reads the new group's stored scale into [`Self::fields`]. Assigning
    /// it without reseeding would leave the entry controls describing the
    /// group the operator just navigated away from, which is the O192 defect
    /// reintroduced one row further down.
    group: GroupId,
    /// The two entry paths and everything typed into them.
    ///
    /// Seeded with [`ScaleEntryFields::for_group_panel`], which pre-selects the
    /// **ratio** path — the constructor exists precisely for "no reference line
    /// was drawn", which is this dialog's situation.
    fields: ScaleEntryFields,
    /// The last parse error from the real-length field, if any.
    ///
    /// Held rather than recomputed at draw time so the message and the preview
    /// cannot disagree: `sync_real_length` deliberately leaves the previous
    /// good value alone on a failed parse, so "what is shown" and "what would
    /// commit" have to be captured together.
    parse_error: Option<String>,
    /// Set by Accept, consumed after the window's closure returns.
    ///
    /// Same one-statement deferral the print dialog uses, and for a milder
    /// version of the same reason: `set_group_scale` re-propagates every
    /// member's baked appearance stream, which is a document edit, and the
    /// action funnel's invariant is that no code path runs from a widget to a
    /// document.
    accept_requested: bool,
    /// Set by Close, consumed by [`Self::show`].
    close_requested: bool,
    /// The reference line's measured length in PDF points, when the operator
    /// calibrated by picking two points on the drawing.
    ///
    ///
    /// It is threaded straight through to `ScaleEntryFields`, whose `entry`,
    /// `preview` and `commit` all take `drawn_pdf_length: Option<f64>` and have
    /// since the Phase 7 salvage. **No arithmetic was added anywhere for this**
    /// — the model always supported both paths and had no caller for the
    /// second.
    drawn_pdf_length: Option<f64>,
    /// Set by the calibrate button, consumed after the window's closure.
    ///
    /// It no longer means "close me". See [`Self::hidden`] for what replaced
    /// that, and why the difference is a defect the operator had not yet got
    /// around to reporting.
    calibrate_requested: bool,
}

/// The region the real-length field publishes, so a driven check can type into
/// it. Matched literally by `tools/ui-verify`.
pub const REGION_REAL_LENGTH: &str = "scale.real_length"; // ui-text-exempt: trace region name, never displayed
/// The region the calibrate button publishes.
pub const REGION_CALIBRATE: &str = "scale.calibrate"; // ui-text-exempt: trace region name, never displayed
/// The region the group picker publishes -- O193.
pub const REGION_GROUP: &str = "scale.group"; // ui-text-exempt: trace region name, never displayed
/// The region the current-scale line publishes -- O192.
///
/// A region rather than only a trace line, because the defect being closed is
/// that the operator could not **see** the scale. A trace proving the string
/// was computed would be the same evidence the old window could have produced;
/// what has to be asserted is that it was drawn, with a rect, inside the
/// window's body.
pub const REGION_CURRENT: &str = "scale.current"; // ui-text-exempt: trace region name, never displayed

impl ScaleDialog {
    /// **Open on `group`, showing the scale that group is already at.**
    ///
    /// # It reads the document now -- O192
    ///
    /// This constructor took a bare [`GroupId`] until 2026-09-13 and seeded its
    /// fields from [`ScaleEntryFields::for_group_panel`], which is seeded from
    /// nothing. So the window opened reading `1:100` in metres over a group
    /// calibrated to `1:50` in inches, and an operator who pressed *Set scale*
    /// without touching a control silently recalibrated the whole drawing to a
    /// number the window had invented. The operator's report named the visible
    /// half -- *"does not show me the scale that is already set"* -- and the
    /// invisible half is the one that could have damaged a file.
    ///
    /// [`ScaleEntryFields::for_group`] carries the whole inversion and the
    /// proof that it is exact. The only thing done here is choosing the path:
    /// **ratio**, because a cold-opened window has no drawn reference line and
    /// the real-length path cannot produce a scale without one.
    #[must_use]
    pub fn open(doc: &OpenDoc, group: GroupId) -> Self {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                //
                // This line exists so that a window's CONSTRUCTION can be
                // counted, which is the only externally visible difference
                // between the window being hidden for the duration of a pick
                // and the window being destroyed and rebuilt from defaults.
                // Both put a window back on the screen carrying the
                // measurement; only one of them runs a constructor twice.
                //
                // A count is only a measurement if both constructors are in
                // it, so `calibrated` emits the same event with a different
                // `path`.
                "scale-open group={group:?} path=cold"
            )
        });
        let mut dialog = Self {
            group,
            fields: ScaleEntryFields::for_group_panel(),
            parse_error: None,
            accept_requested: false,
            close_requested: false,
            drawn_pdf_length: None,
            calibrate_requested: false,
        };
        dialog.reseed(&doc.session.dimension_model());
        // The cold path: no line is drawn, so the ratio entry is the only one
        // that can produce a scale. Set AFTER the reseed, which inherits
        // `Default`'s `true` through its `..Self::default()` -- see that
        // constructor's note on why it does not choose a path itself.
        dialog.fields.use_real_length = false;
        dialog
    }

    /// **Open on `group` with a reference line already measured.**
    ///
    /// The calibration path's constructor. Raised by the application when
    /// `ScalePick::dialog_open()` turns true — i.e. on the click that completes
    /// the two-point pick.
    ///
    /// # It seeds the REAL-LENGTH path, not the ratio one
    ///
    /// `ScaleEntryFields::default()` rather than `for_group_panel()`, and that
    /// is the whole difference between the two constructors. `for_group_panel`
    /// exists to pre-select **ratio** because its situation is "no reference
    /// line was drawn"; here one was, so the path the operator just did the
    /// work for is the one that should be waiting for them.
    ///
    /// The ratio path stays available in the same window. An operator who
    /// picks two points and then decides they would rather type `1:100` can,
    /// and nothing is lost — `ScaleEntryFields::entry` chooses on the radio,
    /// not on whether a length exists.
    ///
    /// It also seeds from the group's stored scale (O192), for the same
    /// reason [`Self::open`] does and with one extra consequence: the unit the
    /// group is already in becomes the unit the real-length field is read in,
    /// so an operator calibrating a drawing that is already in inches types
    /// `25 ft` and gets feet, rather than typing into a field that silently
    /// meant metres.
    ///
    /// ⚠ **This is now the FALLBACK entry point, not the usual one.** The
    /// ordinary route is [`Self::deliver_measured`] on a window that never
    /// closed. This constructor is what runs when a pick completes with no
    /// window waiting -- which this application cannot currently reach, and
    /// which is kept rather than removed because removing it would make the
    /// two-point gesture depend on a window's continued existence for its
    /// result to be usable at all.
    #[must_use]
    pub fn calibrated(doc: &OpenDoc, group: GroupId, drawn_pdf_length: f64) -> Self {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "scale-open group={group:?} path=calibrated \
                 calibrated_pdf_length={drawn_pdf_length:.3}"
            )
        });
        let mut dialog = Self {
            group,
            fields: ScaleEntryFields::default(),
            parse_error: None,
            accept_requested: false,
            close_requested: false,
            drawn_pdf_length: Some(drawn_pdf_length),
            calibrate_requested: false,
        };
        dialog.reseed(&doc.session.dimension_model());
        dialog.fields.use_real_length = true;
        dialog
    }

    /// **Re-read [`Self::group`]'s stored scale into the entry fields.**
    fn reseed(&mut self, model: &DimensionModel) {
        let Some(group) = model.group(self.group) else {
            return;
        };
        let path = self.fields.use_real_length;
        let typed = std::mem::take(&mut self.fields.real_length_text);
        let parsed = self.fields.real_length;
        self.fields = ScaleEntryFields::for_group(group.scale, group.format);
        self.fields.use_real_length = path;
        self.fields.real_length_text = typed;
        self.fields.real_length = parsed;
        // A stale message about a field that has just been replaced is worse
        // than none: it reads as a complaint about what is on screen now.
        self.parse_error = None;
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                //
                // `ratio_paper` and `ratio_real` are SEPARATE keys, not one
                // `1:100` field. A driven check reads these to prove the window
                // was seeded from the document rather than from the defaults
                // (O192), and a packed field is one a reader has to split --
                // which is how a check in this suite once reported the opposite
                // of the truth while quoting the truth in its own message.
                //
                // `name` stays Debug-quoted because a group name may contain
                // spaces, which would end the key/value pair early. It is for a
                // human reading the trace; nothing parses it.
                "scale-seeded group={:?} name={:?} ratio_paper={} ratio_real={} \
                 basis={:?} unit={:?}",
                group.id,
                group.name,
                self.fields.ratio_paper,
                self.fields.ratio_real,
                self.fields.basis,
                self.fields.unit,
            )
        });
    }

    /// **Hand a measured reference line back to the window that asked for it.**
    ///
    /// The pick completed; this is the answer coming home. Everything the
    /// operator had already entered is still here, **because the window was
    /// never closed** -- it was only [`hidden`](Self::hidden).
    ///
    /// `use_real_length` is set because the operator has just done the work
    /// that path exists for; the ratio entry stays available in the same
    /// window, exactly as it does on the [`Self::calibrated`] path.
    ///
    /// Nothing here un-hides. Un-hiding is what the caller does by disarming
    /// the tool, and it is derived rather than done -- see [`Self::hidden`].
    pub fn deliver_measured(&mut self, drawn_pdf_length: f64) {
        self.drawn_pdf_length = Some(drawn_pdf_length);
        self.fields.use_real_length = true;
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "scale-delivered group={:?} measured_pt={drawn_pdf_length:.3}",
                self.group
            )
        });
    }

    /// **Is this window out on the page, waiting for the operator to point?**
    ///
    /// # Hidden is DERIVED, and that distinction is the third defect
    ///
    ///
    /// # Why a flag was written first, and then deleted
    ///
    /// The obvious repair is an `awaiting_pick: bool` set when the button is
    /// pressed and cleared on delivery, plus a once-a-frame invariant in
    /// `app::frame` to clear it when the operator abandons the pick. That was
    /// written, and then removed, because `canvas::placing`'s header already
    /// contains the ruling against it **and names this window as the broken
    /// precedent it was generalising away from**:
    ///
    /// > With a stored `hidden: bool` this arm would inherit that, five times
    /// > over: a mode change through `tool::arm::retire_forbidden`, the Tool
    /// > panel putting the pen down, a ribbon control arming a different tool,
    /// > the document closing, Escape. Every one is a route somebody has to
    /// > remember to clear a flag on. With `hidden` derived, **stranding is
    /// > unrepresentable**.
    ///
    /// So it is derived. The window is hidden for exactly as long as the scale
    /// pick is armed, and for no other reason. Every route that disarms the
    /// tool -- Escape, a mode change, the Tool panel, a ribbon control, the
    /// completed pick itself, and any route added next year by somebody who
    /// has never read this file -- brings the window back with every entry
    /// still in it, because there is no flag to forget.
    ///
    /// The derivation is only sound because `MeasureKind::Scale` has exactly
    /// **one** arming site in the crate: `app::frame`, on this window's own
    /// button. That is not an accident anybody has to maintain by hand --
    /// `canvas::measure`'s `every_variant_is_either_offered_or_deliberately_excluded`
    /// is a wildcard-free `match` that classifies `Scale` as `"elsewhere"`
    /// rather than `"ribbon"`, so a future ribbon control for it does not
    /// compile until somebody moves it between the two lists and reads why.
    ///
    /// [`crate::canvas::tool::selected`] and not `active`: `active`
    /// resolves the space-bar's temporary Hand override, and an operator who
    /// pans the page mid-pick must not have this window flash back over the
    /// drawing they are panning to look at.
    #[must_use]
    pub fn hidden(ctx: &egui::Context) -> bool {
        crate::canvas::tool::selected(ctx)
            == crate::canvas::tool::CanvasTool::Measure(crate::canvas::measure::MeasureKind::Scale)
    }

    /// Whether the operator asked to measure the reference line on the drawing.
    ///
    /// Consumed by the application, which arms `MeasureKind::Scale`. Read-and-
    /// clear rather than a returned flag, so the caller cannot forget to reset
    /// it and re-arm on every subsequent frame.
    ///
    pub fn take_calibrate_request(&mut self) -> bool {
        std::mem::take(&mut self.calibrate_requested)
    }

    /// Draw one frame. Returns `false` when it should close.
    ///
    /// # Screen-anchored, like every dialog here
    ///
    /// A surface an operator is typing into must stay where they put their
    /// eyes, and a position derived from the page moves on every zoom and
    /// scroll. `default_pos` rather than `anchor` so it can be dragged aside —
    /// this one sits over a drawing the operator may want to look at while
    /// deciding what the scale is.
    ///
    /// # The early return says `true`, and the `true` is load-bearing
    ///
    /// [`Self::hidden`] returns **without drawing**: the window still exists,
    /// it is simply out on the page while the operator points at a line. A
    /// `false` here would destroy exactly the state this change exists to
    /// preserve -- it is the old `close_scale()` behaviour, spelled one layer
    /// further in.
    pub fn show(&mut self, ctx: &egui::Context, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
        if Self::hidden(ctx) {
            return true;
        }
        // Read once per frame and read from the SESSION, the same rule the
        // Manage-groups panel records: `dimension_model()` clones out of the
        // `/PieceInfo` sidecar, so this is the model as the document currently
        // stands including every unsaved edit. A snapshot taken when the window
        // opened would be stale for exactly one frame after every change the
        // operator made -- which is the frame they are looking at.
        let model = doc.session.dimension_model();
        // The selected group can be deleted from under this window: the
        // Manage-groups panel is a dock panel and is reachable while this is
        // up. Falling back keeps the picker showing a real row rather than an
        // empty caption, and the reseed is what stops the entry controls
        // describing the group that has gone.
        if model.group(self.group).is_none() {
            self.group = DEFAULT_GROUP_ID;
            self.reseed(&model);
        }
        let screen = ctx.input(egui::InputState::content_rect);
        let size = egui::vec2(440.0_f32.min(screen.width() - 40.0), 300.0);
        let pos = egui::pos2(
            ((screen.width() - size.x).max(0.0) / 2.0).max(0.0),
            ((screen.height() - size.y).max(0.0) / 3.0).max(0.0),
        );

        let _ = pos;
        let (frame, ()) = crate::dialogs::host::Host::new(
            "set-scale", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            size,
            egui::vec2(360.0, 220.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui, &model);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.accept_requested) {
            self.commit(actions);
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The group picker, the current scale, the fields, the preview and the
    /// two buttons.
    fn body(&mut self, ui: &mut Ui, model: &DimensionModel) {
        ui.label(t::intro());
        ui.add_space(6.0);

        // O193 and O192, in this order and at the TOP of the window.
        //
        // The order is the sentence: *which drawing* before *what it is at
        // now* before *what you would like it to be*. Put the current scale
        // above the picker and it is a fact with no subject; put either below
        // the entry fields and the operator has already typed a number before
        // being told what it replaces.
        self.group_row(ui, model);
        ui.add_space(6.0);

        // BOTH PATHS NOW, and which one the window leads with depends on
        // whether the operator measured something first.
        //
        // This block used to say the ratio path was the only one, because
        // "the real-length path needs a drawn reference line, and drawing one
        // is a canvas gesture no command arms yet". That sentence was accurate
        // and it was also the whole gap the operator reported on 2026-08-17:
        // *"still missing the feature where we set the scale by selecting two
        // lines or points and defining what that distance represents."*
        //
        // The gesture now exists (`MeasureKind::Scale`), and the two states
        // this dialog can be in are genuinely different questions:
        match self.drawn_pdf_length {
            // Calibrated: a line was picked, so the useful question is what it
            // represents. The measured length is shown because it is the half
            // of the equation pdfcer contributed, and an operator checking their
            // work needs to see that pdfcer measured what they meant to pick.
            Some(measured) => {
                ui.label(t::calibrated_note(measured));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(t::real_length_label());
                    let response = ui.add(
                        // escape-disposition: dialog-cancels — `dialogs::host` owns the key for
                        // every field in this window: the first press leaves the box, the second
                        // cancels.
                        egui::TextEdit::singleline(&mut self.fields.real_length_text)
                            .hint_text(t::real_length_hint())
                            .desired_width(140.0),
                    );
                    crate::diag::ui_rect(REGION_REAL_LENGTH, response.rect);
                    if response.changed() {
                        // Parsed on every keystroke, and the error HELD rather
                        // than recomputed at draw time — `sync_real_length`
                        // deliberately leaves the last good value alone on a
                        // failed parse, so the message and the preview have to
                        // be captured together or they disagree.
                        self.parse_error = self
                            .fields
                            .sync_real_length()
                            .map(|e| t::length_parse_error(&e.to_string()));
                    }
                });
                if let Some(err) = &self.parse_error {
                    ui.label(
                        egui::RichText::new(err)
                            .small()
                            .color(egui_shell::theme::Theme::of(ui.ctx()).palette.danger),
                    );
                }
                ui.label(
                    egui::RichText::new(t::real_length_hint_long())
                        .small()
                        .weak(),
                );
            }
            // Cold: no line, so the ratio path is the only one that can produce
            // a scale — and the button is how the operator reaches the other.
            //
            // A BUTTON rather than a greyed radio. Greying is for temporarily
            // unavailable, and this is not unavailable: it is one click away.
            // The button says what that click does, because "Calibrate" is a
            // word from our side of the fence and "measure it on the drawing"
            // is what the operator is about to do.
            None => {
                ui.label(egui::RichText::new(t::ratio_only_note()).small().weak());
                ui.add_space(4.0);
                let calibrate = ui
                    .button(t::calibrate_button())
                    .on_hover_text(t::calibrate_tooltip());
                crate::diag::ui_rect(REGION_CALIBRATE, calibrate.rect);
                if calibrate.clicked() {
                    self.calibrate_requested = true;
                }
                ui.label(egui::RichText::new(t::calibrate_note()).small().weak());
            }
        }
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label(t::ratio_label());
            ui.add(
                egui::DragValue::new(&mut self.fields.ratio_paper)
                    .speed(0.01)
                    .range(0.0001..=10_000.0),
            );
            ui.label(t::ratio_separator());
            ui.add(
                egui::DragValue::new(&mut self.fields.ratio_real)
                    .speed(1.0)
                    .range(0.0001..=1_000_000.0),
            );
        });
        ui.label(egui::RichText::new(t::ratio_hint()).small().weak());
        ui.add_space(6.0);

        // The paper-unit basis. Disclosed rather than assumed, because a ratio
        // is meaningless without one: `1:100` on an inch basis and `1:100` on a
        // millimetre basis are different scales, and PDF's own paper unit is
        // 1/72", which is nobody's intuition.
        ui.horizontal(|ui| {
            ui.label(t::basis_label());
            unit_combo(ui, "scale.basis", &mut self.fields.basis);
        });
        ui.horizontal(|ui| {
            ui.label(t::unit_label());
            unit_combo(ui, "scale.unit", &mut self.fields.unit);
        });
        ui.horizontal(|ui| {
            ui.label(t::fraction_label());
            fraction_combo(ui, &mut self.fields.fraction);
        });

        ui.add_space(8.0);
        ui.separator();

        // The live preview, from the ENGINE's own back-calculation.
        //
        // `ScaleEntryFields::preview` calls `preview_group_scale`, which is the
        // same function the CLI calibrates through — so what this window shows
        // and what a scripted calibration produces are the same number by
        // construction rather than by two implementations agreeing.
        //
        // `None` means a degenerate entry (a zero somewhere), and Accept then
        // has nothing to commit. Shown as a refusal rather than as a blank, so
        // a greyed Accept has a reason beside it.
        let preview = self.fields.preview(self.drawn_pdf_length);
        match &preview {
            Some(p) => {
                // `ratio_label` is the engine's own `/R`-style display string —
                // `1:100`, or `25 ft = 42.3 pt`. Using it rather than formatting
                // the scale here keeps one implementation of a number the
                // operator is checking; two would eventually disagree about
                // rounding.
                ui.label(t::preview(&p.ratio_label, p.unit));
            }
            None => {
                ui.label(egui::RichText::new(t::degenerate()).color(ui.visuals().warn_fg_color));
            }
        }
        if let Some(error) = &self.parse_error {
            ui.label(
                egui::RichText::new(t::parse_failed(error)).color(ui.visuals().error_fg_color),
            );
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let accept = ui.add_enabled(preview.is_some(), egui::Button::new(t::accept()));
            crate::diag::ui_rect(REGION_ACCEPT, accept.rect);
            if accept.clicked() {
                self.accept_requested = true;
            }
            if preview.is_none() {
                accept.on_disabled_hover_text(t::accept_disabled_tooltip());
            }
            if ui
                .button(t::cancel())
                .on_hover_text(t::cancel_tooltip())
                .clicked()
            {
                self.close_requested = true;
            }
        });
    }

    /// **The picker and the current-scale line** -- O193 and O192.
    fn group_row(&mut self, ui: &mut Ui, model: &DimensionModel) {
        let before = self.group;
        ui.horizontal(|ui| {
            ui.label(t::group_label());
            let combo = egui::ComboBox::from_id_salt(REGION_GROUP)
                .selected_text(
                    model
                        .group(self.group)
                        .map_or("", |group| group.name.as_str()),
                )
                .show_ui(ui, |ui| {
                    // `model.groups()` directly, with no local list in front
                    // of it -- the same rule the unit combo below is read
                    // under. A group the operator adds in the panel beside this
                    // window appears here on the next frame without anybody
                    // remembering.
                    for group in model.groups() {
                        ui.selectable_value(&mut self.group, group.id, group.name.as_str());
                    }
                });
            crate::diag::ui_rect(REGION_GROUP, combo.response.rect);
        });
        if self.group != before {
            self.reseed(model);
        }

        // O192. Drawn for every group including an uncalibrated one, because
        // *"no scale set"* is the answer the operator most needs and the one
        // the old window was least able to give: it looked identical whether
        // the group was at 1:50 or at nothing at all.
        if let Some(group) = model.group(self.group) {
            let line = ui.label(t::current_scale(&gt::scale_phrase(
                group.scale,
                group.format.unit,
            )));
            crate::diag::ui_rect(REGION_CURRENT, line.rect);
        }
    }

    /// Turn the entry into the one action that changes the document.
    fn commit(&self, actions: &mut Vec<Action>) {
        // `None` for the drawn length: this dialog offers the ratio path, which
        // needs no line. `ScaleEntryFields::entry` routes on exactly that.
        let Some((scale, format)) = self.fields.commit(self.drawn_pdf_length) else {
            // Unreachable from the UI — Accept is disabled without a preview,
            // and a preview exists exactly when `commit` will. Handled rather
            // than unwrapped because an `Action` is plain data a test can
            // build, and declining by name beats a panic in a frame that is
            // trying to draw.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "scale-commit-refused reason=degenerate".to_owned()
            });
            return;
        };
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "scale-commit group={:?} scale={scale:?} format={format:?} \
                 ratio={}:{} basis={:?}",
                self.group, self.fields.ratio_paper, self.fields.ratio_real, self.fields.basis,
            )
        });
        actions.push(Action::Dimension(DimensionAction::SetGroupScale {
            group: self.group,
            scale,
            format,
        }));
    }
}

/// A unit picker.
fn unit_combo(ui: &mut Ui, id: &str, unit: &mut Unit) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(t::unit_name(*unit))
        .show_ui(ui, |ui| {
            for option in Unit::all().iter().copied() {
                ui.selectable_value(unit, option, t::unit_name(option));
            }
        });
}

/// The number styles offered.
const FRACTIONS: &[FractionMode] = &[
    FractionMode::Decimal { places: 0 },
    FractionMode::Decimal { places: 1 },
    FractionMode::Decimal { places: 2 },
    FractionMode::Decimal { places: 3 },
    FractionMode::Fraction {
        denominator: 8,
        reduce: false,
    },
    FractionMode::Fraction {
        denominator: 16,
        reduce: false,
    },
    FractionMode::Fraction {
        denominator: 32,
        reduce: false,
    },
];

/// A fraction-display picker, including the *"use the unit's default"* state.
fn fraction_combo(ui: &mut Ui, fraction: &mut Option<FractionMode>) {
    egui::ComboBox::from_id_salt("scale.fraction")
        .selected_text(t::fraction_name(*fraction))
        .show_ui(ui, |ui| {
            ui.selectable_value(fraction, None, t::fraction_name(None));
            for mode in FRACTIONS {
                ui.selectable_value(fraction, Some(*mode), t::fraction_name(Some(*mode)));
            }
        });
}
