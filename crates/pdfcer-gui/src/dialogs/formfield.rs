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

// ═════════════════════════════════════════════════════════════════════════════
// HOW BIG THE WINDOW OPENS — review finding A16a, fixed 2026-09-04
// ═════════════════════════════════════════════════════════════════════════════
//
// ★★★ THE REPORT, AND WHY IT WAS HALF RIGHT
//
// The review said this dialog *"clips"*. It does not: the body has scrolled
// since it was written, so every control is reachable. What it does is open at
// a size that is **too small for its own content**, with the one affordance
// that would say so — a scrollbar — drawn as egui's default floating sliver:
// two points wide, allocating no space, and faded out whenever the pointer is
// somewhere else. So the operator sees a dialog that appears to end after the
// tooltip box, with nothing on screen suggesting there is more below it. From
// the outside that is indistinguishable from clipping, which is exactly what
// the reviewer wrote down.
//
// The fix is therefore in two halves, and neither is sufficient alone:
//
//   1. the window opens at a size derived from what this kind of field actually
//      asks — the constants and the inventory below;
//   2. when it *does* scroll — a push button whose action grows extra rows, an
//      operator who dragged the window small — the bar is **solid and drawn in
//      the text colour**, so the affordance exists. That is `print/layout.rs`'s
//      remedy for its own invisible-scrollbar defect, reached the same way.
//
// ★★★ AND IT IS NOT MEASURED FROM THE BODY. R128, and this project has been
// bitten three times — the fit-zoom loop, the About window growing 560 → 1,624
// px in a few frames, and the print dialog's two mutually-causing scrollbars.
// `print/layout.rs`'s header states the rule this section obeys:
//
//   > Every width and height is derived from the space OUTSIDE the scroll area
//   > and from constants. Nothing is measured from inside it.
//
// A height taken from `ui.min_rect()` of the laid-out body would be a
// measurement of content that was laid out to fit the height being computed.
// It cannot be right, and its wrongness is a loop rather than an offset. So
// what follows is an **inventory**: a declared count of the rows each kind
// draws, priced with constants. It can go stale — a row added without a line
// here opens a window one row too short — and that is a strictly better failure
// than a window that grows while you look at it. `the_inventory_prices_every_kind`
// below is what makes staleness visible rather than silent.

/// A caption line above a control, including the spacing under it.
const LABEL_PTS: f32 = 22.0;

/// One interactive control on its own row — a single-line box, a check box, a
/// row of radio buttons, a combo — including the spacing under it.
///
/// This shell's theme presets draw controls at 28 pt (see `FEATURES.md` on the
/// status bar's height constant, which was written for 24 and was wrong for
/// exactly this reason), and the inventory prices the row rather than the
/// control, so the item spacing is in the number.
const CONTROL_PTS: f32 = 32.0;

/// A `ui.small` explanatory sentence.
///
/// Priced at **three** lines rather than the two most of them take at
/// [`WINDOW_PTS`]'s width. Deliberately generous: the wrapped height of a
/// sentence is a function of the font, the preset and the width, none of which
/// this arithmetic may read without becoming a measurement — so the honest
/// thing is to over-price it and be a few points tall rather than to price it
/// exactly at one preset and be a line short in another.
const NOTE_PTS: f32 = 44.0;

/// A horizontal rule with the 6 pt of air this file puts on each side of one.
const SEPARATOR_PTS: f32 = 20.0;

/// One `ui.add_space(6.0)`, as the body writes it.
const GAP_PTS: f32 = 6.0;

/// One row of a multi-line text box.
const TEXT_ROW_PTS: f32 = 18.0;

/// The border and padding a multi-line text box adds around its rows.
const TEXT_BOX_CHROME_PTS: f32 = 16.0;

/// Everything drawn ABOVE the scrolling body: the intro sentence and its space.
///
/// Priced at two lines because `text::formfield::intro`'s longest — the radio
/// button's *"One of a set of alternatives — picking one clears the others."* —
/// wraps to two at this width.
const INTRO_PTS: f32 = 48.0;

/// Height reserved UNDER the scrolling body for the separator and the button
/// row.
///
/// # ★ It is one constant used twice, and it was a literal `40.0` in one of the
/// # two places
///
/// The body reserves this out of the scroll area's `max_height` because the
/// buttons are drawn *after* it, and the window's opening height adds it back
/// because the buttons need somewhere to be. Those two uses must agree — a
/// reservation smaller than the row pushes Accept off the bottom of a dialog
/// whose entire purpose is Accept — so they read one number.
///
/// 46 pt, matching `print::layout::FOOTER_HEIGHT_PTS`, which reserves the same
/// thing for the same reason.
const FOOTER_PTS: f32 = 46.0;

/// The window's opening size before the content inventory and the application
/// window have their say.
///
/// The width is the whole of the horizontal story: the body is a stack of
/// full-width controls, so nothing here competes for width and the number only
/// has to be wide enough that the explanatory sentences wrap to two lines
/// rather than four. 480 rather than the previous 440 buys about ten characters
/// a line, which is one line off each of the four notes.
const WINDOW_PTS: egui::Vec2 = egui::vec2(480.0, 420.0);

/// The smallest this window may be, by resize or by squeeze.
///
/// What the common rows plus the buttons need. Below it the dialog is a
/// scrollbar over an empty window rather than a usable form.
const MIN_WINDOW_PTS: egui::Vec2 = egui::vec2(360.0, 260.0);

/// How much of the application window is left clear above and below when this
/// dialog's content is taller than the screen it has to open on.
const SCREEN_MARGIN_PTS: f32 = 60.0;

/// The width a solid scrollbar is drawn at, when the body does scroll.
///
/// The same 10 pt `print::layout::SCROLLBAR_WIDTH_PTS` uses, so the two dialogs
/// that draw one draw the same one.
const SCROLLBAR_WIDTH_PTS: f32 = 10.0;

/// **How tall the rows this kind draws are**, in points — the declared
/// inventory.
///
/// # What is counted, and the one thing that is not
///
/// Everything inside the scroll area, for the dialog **as it opens**: the two
/// common rows at the top, the kind's own rows, and the two common flags plus
/// the border row at the bottom.
///
/// ★★ A push button's *action* rows are deliberately **excluded**, and that is
/// the one judgement in this function. `dialogs::buttonaction::rows` draws a
/// different set of controls for each of seven choices — from nothing at all
/// for *Do nothing* to a radio pair, a label, a three-row box and a note for
/// *Show or hide fields* — and the choice is made by the operator **after** the
/// window has been created. Sizing the opening window for the tallest of them
/// would open every push-button dialog 160 pt taller than the rows it is
/// showing, to pre-pay for a branch most operators never take. Those rows are
/// what the scroll area is for, and they are why the bar is now visible.
#[must_use]
fn content_height(kind: FormFieldKind) -> f32 {
    // The name row, the tooltip row and its note.
    let name = LABEL_PTS + CONTROL_PTS;
    let tooltip = GAP_PTS + LABEL_PTS + CONTROL_PTS + NOTE_PTS;
    // Required, read-only, then the border row behind a gap.
    let flags = CONTROL_PTS + CONTROL_PTS + GAP_PTS + CONTROL_PTS;
    let specific = match kind {
        // Starting value, multiline, password, the max-length row, and comb —
        // which appears only once a maximum has been set, and is counted
        // because the window must not start clipping when it does.
        FormFieldKind::Text => {
            LABEL_PTS + CONTROL_PTS + GAP_PTS + CONTROL_PTS * 3.0 + GAP_PTS + CONTROL_PTS
        }
        // Ticked-by-default, then the export value and what it means.
        FormFieldKind::CheckBox => CONTROL_PTS + GAP_PTS + LABEL_PTS + CONTROL_PTS + NOTE_PTS,
        // The group note first — it is the sentence that stops the commonest
        // form-authoring mistake — then the value and the starts-selected box.
        FormFieldKind::Radio => {
            NOTE_PTS + GAP_PTS + LABEL_PTS + CONTROL_PTS + GAP_PTS + CONTROL_PTS
        }
        // The options box is four rows tall; then the drop-down/list pair,
        // one conditional flag, and sort.
        FormFieldKind::Choice => {
            LABEL_PTS + TEXT_ROW_PTS * 4.0 + TEXT_BOX_CHROME_PTS + GAP_PTS + CONTROL_PTS * 3.0
        }
        // The caption, then `buttonaction::rows`' own opening rows: its space,
        // its label, the combo, and the reach sentence under it.
        FormFieldKind::PushButton => {
            LABEL_PTS + CONTROL_PTS + GAP_PTS + LABEL_PTS + CONTROL_PTS + NOTE_PTS + GAP_PTS
        }
    };
    name + tooltip + SEPARATOR_PTS + specific + SEPARATOR_PTS + flags
}

/// **How big the window opens** for `kind`, on an application window whose
/// content rectangle is `screen`.
///
/// The height is the inventory plus the chrome outside the scroll area, capped
/// so the dialog cannot open taller than the screen it has to appear on and
/// floored at [`MIN_WINDOW_PTS`]. When the cap bites, the body scrolls — with a
/// visible bar — which is the right answer and the reason the scroll area was
/// never the defect.
///
/// ★ `screen` is the *application* window's content rectangle, used as the
/// stand-in for the monitor's usable height. It is not the same quantity —
/// this dialog is an OS window and may legally be taller than its parent — but
/// it is the only one `eframe 0.35` offers without a work-area query, it is
/// never larger than the monitor, and erring small here costs a scrollbar
/// rather than a window with its buttons below the taskbar. The Settings window
/// makes the same substitution and says so.
#[must_use]
fn window_size(kind: FormFieldKind, screen: egui::Rect) -> egui::Vec2 {
    let wanted = INTRO_PTS + content_height(kind) + FOOTER_PTS;
    let cap = (screen.height() - SCREEN_MARGIN_PTS).max(MIN_WINDOW_PTS.y);
    egui::vec2(
        WINDOW_PTS
            .x
            .min(screen.width() - SCREEN_MARGIN_PTS)
            .max(MIN_WINDOW_PTS.x),
        wanted.min(cap).max(MIN_WINDOW_PTS.y),
    )
}

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
        // ★ Sized to the rows THIS kind draws — see the inventory above and
        // review finding A16a. It used to be a flat `440 x 420` for all five,
        // which is too short for every one of them: the operator saw a dialog
        // that appeared to end after the tooltip box, because the only thing
        // saying otherwise was egui's two-point floating scrollbar.
        let size = window_size(self.draft.kind, ctx.input(egui::InputState::content_rect));
        let (frame, ()) = crate::dialogs::host::Host::new(
            "form-field", // ui-text-exempt: a viewport key, never displayed.
            t::title(self.draft.kind),
            size,
            MIN_WINDOW_PTS,
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

        // ★★ SOLID SCROLLBARS, not egui's floating default — the second half of
        // A16a, and the half that makes the first half honest.
        //
        // `ScrollStyle::default()` is `floating()`: a two-point sliver that
        // allocates no space and fades out whenever the pointer is elsewhere.
        // The body has always scrolled, so nothing was ever unreachable — but
        // the reviewer wrote *"clipped"*, and they were reading the screen
        // correctly. A dialog that ends mid-sentence with no visible bar looks
        // like a rendering fault, not like a scroll.
        //
        // `foreground_color` on top of `solid()` for the reason
        // `print/layout.rs` records: a solid handle defaults to
        // `widgets.inactive.bg_fill`, which in a light preset is a near-white
        // against a near-white panel — measured there as *present, opaque,
        // correctly sized and invisible*. Drawing it from the same visuals'
        // TEXT colour inherits whatever contrast the active theme gives text.
        //
        // ★ Set on a scoped `style_mut` of the body's `Ui`, so it reaches this
        // scroll area and nothing else in the program.
        let mut scroll = egui::style::ScrollStyle::solid();
        scroll.foreground_color = true;
        scroll.bar_width = SCROLLBAR_WIDTH_PTS;
        ui.style_mut().spacing.scroll = scroll;

        egui::ScrollArea::vertical()
            // ★ The SAME constant the opening height adds back for the button
            // row. It was a literal `40.0` here against nothing at all there,
            // which is how the two halves of a reservation drift apart.
            .max_height(ui.available_height() - FOOTER_PTS)
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

    /// A roomy application window — 1,280 x 800, this shell's own default plus
    /// the DPI inflation a real desktop adds — so the screen cap is not the
    /// thing under test.
    fn roomy() -> egui::Rect {
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 900.0))
    }

    /// ★★★ **Every kind's window opens tall enough for the rows that kind
    /// draws** — review finding A16a.
    ///
    /// The defect in one assertion. The window was a flat `440 x 420` for all
    /// five kinds and every one of them needs more than 420, so the dialog
    /// always opened with content below the fold — behind a two-point floating
    /// scrollbar nobody could see, which is why it was reported as clipping
    /// rather than as scrolling.
    ///
    /// ★ Both sides of this comparison come from this file's own constants, so
    /// it does not prove the constants are *right* — no unit test can, because
    /// the true row heights exist only in a laid-out frame under a theme. What
    /// it proves is that the window and the inventory cannot drift apart, which
    /// is the failure that shipped: a size chosen once, by hand, against
    /// content that then grew. The same shape as
    /// `print::layout`'s `the_content_floor_is_the_sum_of_the_column_floors`.
    #[test]
    fn every_kind_opens_tall_enough_for_its_own_rows() {
        for kind in FormFieldKind::ALL {
            let needs = INTRO_PTS + content_height(kind) + FOOTER_PTS;
            let got = window_size(kind, roomy()).y;
            assert!(
                got + f32::EPSILON >= needs,
                "{kind:?} opens {got} pt tall for {needs} pt of content — the operator sees \
                 a dialog that appears to end early, which is exactly what A16a reported"
            );
        }
    }

    /// **The inventory prices every kind separately**, so it cannot quietly
    /// collapse back into one number.
    ///
    /// The failure this guards is the one that shipped: a single hand-chosen
    /// size standing in for five different bodies. A drop-down has a four-row
    /// options box and three flags; a check box has one tick and one value.
    /// If those two ever price the same, the inventory has stopped being an
    /// inventory and is a constant wearing a `match`.
    #[test]
    fn the_inventory_prices_every_kind() {
        let choice = content_height(FormFieldKind::Choice);
        let check = content_height(FormFieldKind::CheckBox);
        assert!(
            choice > check,
            "a drop-down ({choice} pt) draws more rows than a check box ({check} pt); \
             pricing them the same means the inventory is not reading the kind"
        );
        for kind in FormFieldKind::ALL {
            assert!(
                content_height(kind) > 0.0,
                "{kind:?} priced its rows at nothing"
            );
        }
    }

    /// **A short screen caps the window rather than opening it off the
    /// bottom**, and the floor still wins under the cap.
    ///
    /// The cap is what keeps the inventory from becoming a licence to open a
    /// window taller than the desktop. When it bites the body scrolls, which is
    /// what the scroll area is for and why the bar was made visible in the same
    /// change.
    #[test]
    fn a_short_screen_caps_the_window_and_the_body_scrolls() {
        let short = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 400.0));
        let got = window_size(FormFieldKind::Text, short);
        assert!(
            got.y <= short.height() - SCREEN_MARGIN_PTS + f32::EPSILON,
            "the window ({} pt) must fit the screen it opens on ({} pt)",
            got.y,
            short.height()
        );
        assert!(
            got.y >= MIN_WINDOW_PTS.y,
            "the floor outranks the cap: a window below {} pt is a scrollbar over nothing",
            MIN_WINDOW_PTS.y
        );

        let tiny = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(80.0, 80.0));
        let got = window_size(FormFieldKind::Text, tiny);
        assert!(got.x >= MIN_WINDOW_PTS.x && got.y >= MIN_WINDOW_PTS.y);
    }

    /// The window never opens smaller than the size it refuses to be dragged
    /// to.
    ///
    /// A default below the floor is silently clamped by the window manager, so
    /// the dialog would open at a size no constant in this file names — and the
    /// operator could never get back to the one that was intended.
    #[test]
    fn the_opening_size_is_never_under_the_floor_it_declares() {
        for kind in FormFieldKind::ALL {
            let got = window_size(kind, roomy());
            assert!(
                got.x >= MIN_WINDOW_PTS.x && got.y >= MIN_WINDOW_PTS.y,
                "{kind:?} opens at {got:?}, under its own floor {MIN_WINDOW_PTS:?}"
            );
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
