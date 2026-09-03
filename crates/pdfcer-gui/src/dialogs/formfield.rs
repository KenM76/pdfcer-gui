//! # `dialogs::formfield` — the details a placed form control needs
//!
//! **Operator request, 2026-08-26:** *"when I click one I should be able to
//! click on the canvas to place the position or drag a box for size then a pop
//! up lets me set the details for the feature."* This is the pop-up.
//!
//! It opens on `Action::BeginFormField`, which the canvas raises on the click or
//! release that finishes placing, and it **authors nothing until Accept**. That
//! is the whole reason a dialog is in this path rather than a properties pane
//! after the fact: a form field is invisible on a printed page and swallows
//! every keystroke aimed near it, so a mis-drag that left one behind would be
//! both hard to notice and annoying to find.
//!
//! ## ★★★ The tooltip field is not a nicety — it is the feature's blocker
//!
//! Every one of `pdfcer-core`'s five authoring verbs refuses a spec whose
//! tooltip is `TooltipChoice::Undecided`, because an interactive control owes a
//! screen reader a name and the engine will not invent one silently. That
//! refusal was recorded in this project's backlog as *"core's STRUCTURAL
//! certification gate"* and parked form authoring for nine days.
//!
//! There is no gate. The blocker is **this text box**. An empty one becomes
//! `Declined` — the operator saying *"this control needs no name"*, which is a
//! decision the engine accepts and is sometimes right — and a filled one becomes
//! `Text`. What the engine will not accept is nobody having been asked, and now
//! somebody has.
//!
//! ## ★★ Why one dialog for five kinds, and how it stays legible
//!
//! [`crate::canvas::formfield::draft::Draft`]'s header argues the model side:
//! the five engine specs share nine fields and differ in one to five, so five
//! GUI structs would mean writing the shared half five times. The same argument
//! holds for the surface, with one addition — **the shared half is the half an
//! operator adjusts.** Name, tooltip, required, read-only and border are asked
//! identically for all five, and only the kind-specific rows change.
//!
//! So the layout is: the common rows, a separator, then [`Self::specific`],
//! which is the only `match` on kind in the file. A reader looking for "what is
//! different about a check box" has exactly one place to look.
//!
//! ## What is remembered, and where
//!
//! Nothing, here. The dialog opens with a draft that
//! `Remembered::next` already prepared, and `Action::CommitFormField` is what
//! records the accepted one — **at the point it was accepted**, so a draft the
//! operator cancelled is not remembered. See `app::actions::apply`'s arm.

use crate::app::actions::forms::FieldAction;
use egui::Ui;

use crate::app::actions::Action;
use crate::canvas::formfield::draft::NAME_MAX;
use crate::canvas::formfield::{Draft, FormFieldKind};
use crate::text::formfield as t;

/// The dialog body's rect, for `ui-verify`.
///
/// ★ These names are a **cross-repo stability contract**: `tools/ui-verify`
/// asserts on them by string, so renaming one silently turns a check into a
/// skip rather than a failure. Treat them as published API.
const REGION_BODY: &str = "dialog.form_field.body";
/// The Accept control's rect.
const REGION_ACCEPT: &str = "dialog.form_field.accept";
/// The name field's rect — the one control that decides whether Accept is live.
const REGION_NAME: &str = "dialog.form_field.name";

/// How many characters a tooltip may run to.
///
/// A `/TU` is what a screen reader reads aloud, in one utterance. Past roughly
/// this length it stops being a label and becomes a paragraph nobody waits
/// through.
const TOOLTIP_MAX: usize = 240;

/// The placement dialog for one form control.
#[derive(Debug)]
pub struct FormFieldDialog {
    /// The 0-based page it will be authored onto.
    page: usize,
    /// Where it will go, in PDF user space, already normalised.
    rect: pdfcer_core::page_tree::Rect,
    /// Everything the operator is choosing.
    draft: Draft,
    /// Set by the Accept control; drained by [`Self::show`].
    accept_requested: bool,
    /// Set by Cancel; drained by [`Self::show`].
    close_requested: bool,
    /// Whether the name field has ever actually held focus.
    ///
    /// ★ Held, not *asked for* — the distinction the text-annotation dialog
    /// paid for twice. See [`Self::name_row`].
    focused_once: bool,
    /// How many focus requests have been spent while the window was focused.
    focus_attempts: u8,
}

/// How many frames a focus request may be retried for.
///
/// Long enough to outlast the pointer release that opened the window being
/// resolved; far short of a human reaching for the mouse, so a request cannot
/// fight the operator's own click on Cancel.
const FOCUS_ATTEMPT_FRAMES: u8 = 8;

impl FormFieldDialog {
    /// Open for a control of `draft.kind` about to be placed at `rect`.
    #[must_use]
    pub fn open(page: usize, rect: pdfcer_core::page_tree::Rect, draft: Draft) -> Self {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "form-field-open kind={:?} page={page} name={} w={:.2} h={:.2}",
                draft.kind,
                draft.name,
                rect.urx - rect.llx,
                rect.ury - rect.lly
            )
        });
        Self {
            page,
            rect,
            draft,
            accept_requested: false,
            close_requested: false,
            focused_once: false,
            focus_attempts: 0,
        }
    }

    /// Draw one frame. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        // ★★★ **The harness's way past a second OS window.**
        //
        // R1 says a feature is not done until it is asserted by driving the
        // running binary. `tools/ui-verify` drives ONE window — the one
        // `Session::launch` found — and this dialog is a deferred viewport with
        // a window of its own. So without this, everything downstream of
        // placing a field is unreachable: the five engine verbs, the narrowing
        // in `app::actions::forms::author`, and all four rule-4 disclosures
        // would be implemented, unit-tested, and never once exercised in a
        // running window.
        //
        // The precedent is exact and there are two of them. `PDFCER_DIAG_OPEN_PATH`
        // substitutes the answer to a native file picker, and
        // `PDFCER_DIAG_INSERT_PATH` the answer to another — both because a
        // dialog the harness cannot reach is a hard wall rather than a hard
        // problem. `app::dropped`'s note generalises it: *"without this,
        // drag-and-drop would be the one feature in this shell that R1 could
        // not reach."*
        //
        // ★ What it substitutes is the OPERATOR'S PRESS, not the authoring. It
        // sets the same flag the Add button sets, so everything after that
        // point — the readiness guard, the action, the remembering, the
        // narrowing and the engine call — is the path an operator takes. A seam
        // that pushed `CommitFormField` directly would be proving that a
        // different path works, which is the failure this whole channel exists
        // to avoid.
        if accept_requested_by_harness() && self.draft.is_authorable() {
            self.accept_requested = true;
        }
        // Taller than the text-annotation dialog because the kind-specific rows
        // stack under the common ones; the minimum is what the common rows plus
        // the buttons need, so a resized-small window still shows a usable
        // dialog rather than a scrollbar over an empty one.
        let size = egui::vec2(440.0, 420.0);
        let (frame, ()) = crate::dialogs::host::Host::new(
            "form-field", // ui-text-exempt: a viewport key, never displayed.
            t::title(self.draft.kind),
            size,
            egui::vec2(360.0, 260.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if self.accept_requested {
            self.accept_requested = false;
            actions.push(
                FieldAction::Commit {
                    page: self.page,
                    rect: self.rect,
                    draft: Box::new(self.draft.clone()),
                }
                .into(),
            );
            // ★★★ PUT THE TOOL DOWN. `OPERATOR_REQUESTS.md` **O53**.
            //
            // The tool stayed armed after a placement, so the operator's very
            // next click -- the one aimed at the checkbox they had just made,
            // to select it -- placed a SECOND checkbox instead. His report was
            // *"I can't select it on the canvas"*, and he was right: nothing he
            // could do reached the selection, because the click never got there.
            //
            // ★★★ **This project's own harness had been working around it for a
            // day.** `dragging_a_form_field_moves_it` presses Escape before it
            // selects, with a comment calling the arming normal *"exactly as a
            // markup pen does"*. => When a driven check needs a step the
            // operator would never know to take, that step is a bug report. It
            // was written down as scenery.
            //
            // ★★ Acrobat is the parity reference for forms and returns to the
            // selection tool after placing a field unless *Keep tool selected*
            // is ticked; Word, PowerPoint and Visio do the same for a drawn
            // shape. Illustrator and Inkscape keep the tool -- they are drawing
            // programs where placing twenty is the common case -- but **every
            // one of them leaves the new object selected**, which is the half
            // that is not a matter of taste.
            crate::canvas::tool::select(ctx, crate::canvas::tool::CanvasTool::Select);
            return false;
        }
        // ★ The window's own close button counts as Cancel and authors nothing:
        // the operator dismissed a question, and a dismissed question is not an
        // answer. The same reading the text-annotation dialog records.
        !(self.close_requested || !open)
    }

    /// The whole body: common rows, the kind-specific ones, then the buttons.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro(self.draft.kind));
        ui.add_space(8.0);

        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 40.0)
            .show(ui, |ui| {
                self.name_row(ui);
                self.tooltip_row(ui);
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);
                self.specific(ui);
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);
                self.common_flags(ui);
            });

        ui.add_space(6.0);
        ui.separator();
        ui.horizontal(|ui| {
            // ★ Greyed with the reason on hover, which is the one situation R9
            // reserves greying for: *temporarily* unavailable, and one keystroke
            // makes it live.
            let ready = self.draft.is_authorable();
            let accept = ui.add_enabled(ready, egui::Button::new(t::accept()));
            crate::diag::ui_rect(REGION_ACCEPT, accept.rect);
            if accept.clicked() {
                self.accept_requested = true;
            }
            if !ready {
                accept.on_disabled_hover_text(t::accept_disabled());
            }
            if ui.button(t::cancel()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// The name, which is the field's identity and the only hard requirement.
    fn name_row(&mut self, ui: &mut Ui) {
        ui.label(t::name_label(self.draft.kind));
        let response = ui.add(
            egui::TextEdit::singleline(&mut self.draft.name)
                .desired_width(f32::INFINITY)
                .char_limit(NAME_MAX),
        );
        crate::diag::ui_rect(REGION_NAME, response.rect);

        // ★★ Ask until the field actually HOLDS focus — not once. The
        // text-annotation dialog's header carries the full account; the short
        // version is that asking and holding are different facts, the frame
        // that opens this window is still resolving the pointer release that
        // placed the control, and a request that loses that race was never
        // retried. The operator's report of the same bug elsewhere was *"it
        // doesn't type anything in the box when I type."*
        //
        // ★ And the budget is only spent while the WINDOW is focused, because
        // an OS window's focus is granted by the platform and Windows refuses
        // the foreground to a process that does not already have it. Spending
        // the eight frames during that wait means every attempt is made at a
        // window that cannot hold focus.
        let window_focused = ui.ctx().input(|i| i.viewport().focused) != Some(false);
        crate::diag::trace_on_change("form-field-name", || {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "has_focus={} once={} attempts={} window_focused={window_focused}",
                response.has_focus(),
                self.focused_once,
                self.focus_attempts
            )
        });
        if !self.focused_once {
            if response.has_focus() {
                self.focused_once = true;
            } else if window_focused && self.focus_attempts < FOCUS_ATTEMPT_FRAMES {
                self.focus_attempts = self.focus_attempts.saturating_add(1);
                response.request_focus();
            }
        }
    }

    /// The tooltip — see the header for why this row is the whole blocker.
    fn tooltip_row(&mut self, ui: &mut Ui) {
        ui.add_space(6.0);
        ui.label(t::tooltip_label());
        ui.add(
            egui::TextEdit::singleline(&mut self.draft.tooltip)
                .desired_width(f32::INFINITY)
                .hint_text(t::tooltip_hint())
                .char_limit(TOOLTIP_MAX),
        );
        // ★ Stated ALWAYS, not only when empty, and off to the side rather than
        // as a warning. It is a consequence the operator cannot see — a screen
        // reader announcing only "edit box" — and rule 4's surviving half asks
        // for exactly that: report what cannot be seen, do not nag about it.
        ui.small(t::tooltip_note());
    }

    /// The rows that only one kind has. **The only `match` on kind in the file.**
    fn specific(&mut self, ui: &mut Ui) {
        match self.draft.kind {
            FormFieldKind::Text => self.text_rows(ui),
            FormFieldKind::CheckBox => self.check_rows(ui),
            FormFieldKind::Radio => self.radio_rows(ui),
            FormFieldKind::Choice => self.choice_rows(ui),
            FormFieldKind::PushButton => self.button_rows(ui),
        }
    }

    /// A text field's five extra choices.
    fn text_rows(&mut self, ui: &mut Ui) {
        ui.label(t::value_label());
        ui.add(
            egui::TextEdit::singleline(&mut self.draft.value)
                .desired_width(f32::INFINITY)
                .hint_text(t::value_hint()),
        );
        ui.add_space(6.0);
        ui.checkbox(&mut self.draft.multiline, t::multiline());
        ui.checkbox(&mut self.draft.password, t::password())
            .on_hover_text(t::password_hover());

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let mut limited = self.draft.max_len.is_some();
            if ui.checkbox(&mut limited, t::max_len()).changed() {
                // A number appears when the box is ticked and vanishes when it
                // is not, so there is never a maximum length sitting in the
                // draft that the operator has switched off and forgotten.
                self.draft.max_len = limited.then_some(20);
            }
            if let Some(n) = self.draft.max_len.as_mut() {
                ui.add(egui::DragValue::new(n).range(1..=1_000));
            }
        });
        // ★ Comb is offered only when it can be honoured. Its cells are
        // `max_len` divisions of the width, so without a maximum there is
        // nothing to divide by — `Draft::comb_ok` is the rule, asked here
        // rather than restated, so the dialog and the commit cannot disagree.
        //
        // R9: it renders NOTHING when unavailable rather than greying, because
        // this is not a temporary state a keystroke resolves — it is a property
        // of a choice made two rows up, and the row reappears the moment that
        // choice changes.
        if self.draft.comb_ok() {
            ui.checkbox(&mut self.draft.comb, t::comb())
                .on_hover_text(t::comb_hover());
        } else {
            self.draft.comb = false;
        }
    }

    /// A check box's two extra choices.
    fn check_rows(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.draft.checked, t::checked());
        ui.add_space(6.0);
        ui.label(t::export_label());
        ui.add(
            egui::TextEdit::singleline(&mut self.draft.export_value).desired_width(f32::INFINITY),
        );
        ui.small(t::export_note());
    }

    /// A radio button's two extra choices.
    ///
    /// ★★ The wording differs from the check box's even though the fields are
    /// the same two, and deliberately: for a radio the **name is the group**,
    /// so what tells two members apart is the export value. An operator who
    /// reads "export value" as a technical detail here will place three radios
    /// that are all the same answer.
    fn radio_rows(&mut self, ui: &mut Ui) {
        ui.small(t::radio_group_note());
        ui.add_space(6.0);
        ui.label(t::radio_export_label());
        ui.add(
            egui::TextEdit::singleline(&mut self.draft.export_value).desired_width(f32::INFINITY),
        );
        ui.add_space(6.0);
        ui.checkbox(&mut self.draft.checked, t::radio_selected());
    }

    /// A drop-down's options and four flags.
    fn choice_rows(&mut self, ui: &mut Ui) {
        ui.label(t::options_label());
        ui.add(
            egui::TextEdit::multiline(&mut self.draft.options)
                .desired_rows(4)
                .desired_width(f32::INFINITY)
                .hint_text(t::options_hint()),
        );
        ui.add_space(6.0);
        // A drop-down or a list box — one choice, two options, so a pair of
        // radio buttons rather than a checkbox whose unticked state has to be
        // read as "list box".
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.draft.combo, true, t::combo());
            ui.radio_value(&mut self.draft.combo, false, t::list_box());
        });
        // Editable is a property of a DROP-DOWN alone: a list box has no text
        // area to type into, so the flag would be authored and ignored. R9
        // again — the row is absent rather than greyed, because it reappears
        // the moment the choice above changes.
        if self.draft.combo {
            ui.checkbox(&mut self.draft.editable, t::editable());
        } else {
            self.draft.editable = false;
            ui.checkbox(&mut self.draft.multi_select, t::multi_select());
        }
        ui.checkbox(&mut self.draft.sort, t::sort())
            .on_hover_text(t::sort_hover());
    }

    /// A push button's caption, and what pressing it does.
    fn button_rows(&mut self, ui: &mut Ui) {
        ui.label(t::caption_label());
        ui.add(egui::TextEdit::singleline(&mut self.draft.caption).desired_width(f32::INFINITY));
        crate::dialogs::buttonaction::rows(ui, &mut self.draft.action);
        // ★★★ THE INERT NOTE IS GONE, and its deletion is the feature.
        //
        // Until 2026-09-01 this row ended with a sentence saying pdfcer *"can
        // place the button but cannot yet give it something to do"*, and the
        // ribbon command was greyed for the same reason. `pdfcer-core` shipped
        // `set_button_action` on 2026-08-30 and the reply said, in as many
        // words: *"if your surface tells the operator that pdfcer never authors
        // an action, it is now saying something untrue in the direction that
        // matters."*
        //
        // ★ Two days passed before anyone checked. That is the finding worth
        // keeping: the reply arrived, was read, and the sentence it warned
        // about stayed on screen — because nothing in this repository fails
        // when a capability lands. See `canvas::formfield::action`'s tripwire
        // for the shape that would have caught it.
    }

    /// Required and read-only — asked identically for all five kinds.
    fn common_flags(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.draft.required, t::required())
            .on_hover_text(t::required_hover());
        ui.checkbox(&mut self.draft.read_only, t::read_only())
            .on_hover_text(t::read_only_hover());
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(t::border_label());
            ui.add(
                egui::DragValue::new(&mut self.draft.border_width)
                    .range(0.0..=12.0)
                    .speed(0.1),
            )
            .on_hover_text(t::border_hover());
        });
    }
}

/// Whether `PDFCER_DIAG_FORM_ACCEPT` asks this dialog to accept itself.
///
/// ★ Read every frame rather than latched, unlike `scripted_invoke`'s counter,
/// and the difference is real: that one turns an env var into an **event**, so
/// it must fire once. This is a **standing instruction** — *"in this run, accept
/// every form-field dialog"* — and a check that places three fields wants all
/// three accepted. It is idempotent by construction, because accepting closes
/// the dialog.
///
/// Gated on `crate::diag::enabled()` like every other seam, so a stray
/// environment variable cannot change what the shipped program does for an
/// operator who is not running a harness.
fn accept_requested_by_harness() -> bool {
    crate::diag::enabled()
        && std::env::var("PDFCER_DIAG_FORM_ACCEPT").is_ok_and(|v| !v.is_empty() && v != "0")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect() -> pdfcer_core::page_tree::Rect {
        pdfcer_core::page_tree::Rect {
            llx: 10.0,
            lly: 10.0,
            urx: 170.0,
            ury: 30.0,
        }
    }

    /// **Accept is live exactly when the draft can be authored**, which is the
    /// contract the greying above promises.
    #[test]
    fn accept_tracks_authorability() {
        let mut d = Draft::fresh(FormFieldKind::Text);
        let dialog = FormFieldDialog::open(0, rect(), d.clone());
        assert!(
            !dialog.draft.is_authorable(),
            "a fresh draft has no name, so Accept must be greyed"
        );
        d.name = "Total".to_owned();
        let dialog = FormFieldDialog::open(0, rect(), d);
        assert!(dialog.draft.is_authorable());
    }

    /// ★★ **Accepting raises exactly one action, carrying the whole draft.**
    ///
    /// The guard against the shape this dialog exists to avoid: authoring on
    /// placement. Nothing reaches the document until this action does, so a
    /// cancelled dialog must produce none — asserted in the test below.
    #[test]
    fn accepting_raises_one_commit_with_the_draft() {
        let mut draft = Draft::fresh(FormFieldKind::CheckBox);
        draft.name = "Approved".to_owned();
        draft.checked = true;
        let mut dialog = FormFieldDialog::open(3, rect(), draft.clone());
        dialog.accept_requested = true;

        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            dialog.show(ui.ctx(), &mut actions);
        });

        assert_eq!(actions.len(), 1, "one commit, not two and not none");
        match &actions[0] {
            crate::app::actions::Action::Field(FieldAction::Commit {
                page,
                rect: r,
                draft: got,
            }) => {
                assert_eq!(*page, 3);
                assert!((r.urx - 170.0).abs() < f64::EPSILON);
                assert_eq!(**got, draft, "the whole draft travels, unaltered");
            }
            other => panic!("wrong action: {other:?}"),
        }
    }

    /// **Cancelling authors nothing**, which is what makes a mis-drag free.
    #[test]
    fn cancelling_raises_nothing() {
        let mut draft = Draft::fresh(FormFieldKind::Text);
        draft.name = "Discarded".to_owned();
        let mut dialog = FormFieldDialog::open(0, rect(), draft);
        dialog.close_requested = true;

        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        let mut still_open = true;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            still_open = dialog.show(ui.ctx(), &mut actions);
        });

        assert!(actions.is_empty(), "a dismissed question is not an answer");
        assert!(!still_open, "and the dialog closes");
    }

    /// ★ **Comb is cleared when it cannot be honoured**, rather than left set
    /// in a draft whose maximum length has since been switched off.
    ///
    /// Drives the real body so the clearing is asserted where it happens, not
    /// restated. Without it, a text field could be authored `comb` with no
    /// `/MaxLen`, which draws a box divided into no cells.
    #[test]
    fn comb_is_cleared_when_its_precondition_goes_away() {
        let mut draft = Draft::fresh(FormFieldKind::Text);
        draft.name = "Serial".to_owned();
        draft.max_len = None;
        draft.comb = true;
        let mut dialog = FormFieldDialog::open(0, rect(), draft);

        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            dialog.show(ui.ctx(), &mut actions);
        });

        assert!(
            !dialog.draft.comb,
            "comb with no maximum length divides the width into no cells"
        );
    }
}
