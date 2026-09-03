//! # `dialogs::buttonaction` — the *What pressing it does* chooser
//!
//! One control, drawn into the form-field placement dialog when the kind being
//! placed is a push button. Lifted out of `dialogs::formfield` rather than
//! written inline for two reasons: R2 (that file has five kinds' worth of rows
//! already), and because this is the only row group in the dialog that carries
//! a **disclosure obligation**, which is easier to review when it is not
//! interleaved with a comb-cell checkbox.
//!
//! ## ★★★ The interaction is Acrobat's, deliberately
//!
//! *Field Properties ▸ Actions* is a chooser of behaviours with the chosen
//! one's parameters beneath it. Every form editor that came after copied it,
//! which makes it the convention, and this project's standing rule is that the
//! convergence of the product class **is** the spec. So: one drop-down, the
//! parameters for the chosen behaviour under it, and nothing else.
//!
//! What is **not** copied is Acrobat's disclosure, which was measured on this
//! machine on 2026-08-26 through UI Automation and names **scheme and host
//! only**, says nothing about the payload, and pre-ticks *remember this site*.
//! Copying that would be a regression against what pdfcer can say. See
//! `text::buttonaction`'s header.
//!
//! ## ★★ Why the parameters are held even when they are not shown
//!
//! [`ButtonDoes`] carries every parameter for every kind at once, so switching
//! the chooser to *Nothing* and back does not lose the page number that was
//! typed. That is a property of the model rather than of this file, and it is
//! restated here because the obvious way to draw this control — build the
//! engine's `ButtonAction` in the closure — silently has the opposite
//! behaviour.
//!
//! ## ★ Nothing is greyed, and nothing is hidden
//!
//! All seven choices are always offered. R9's rule is that an unavailable
//! capability renders **nothing** and greying is for the *temporarily*
//! unavailable — and none of the seven is either: the engine can write all of
//! them into any document a push button can be placed in. A draft that is
//! incomplete blocks the dialog's **Add** button and says why, which is a
//! different mechanism and the right one, because the remedy is typing rather
//! than waiting.

use egui::Ui;

use crate::canvas::formfield::action::{
    ButtonDoes, ButtonDoesKind, NamedChoice, PageViewChoice, url_is_unencrypted,
};
use crate::text::buttonaction as t;

/// The trace event the chooser writes when the operator changes it.
///
/// ★ Written on **change**, not every frame. A driven check needs to know the
/// chooser was reached and what it was set to, and a per-frame line would bury
/// that in thousands of identical ones.
const CHOSE: &str = "button-action-chose"; // ui-text-exempt: a trace event name, never displayed

/// The closed chooser's rectangle, so a driven check can open it.
const COMBO_REGION: &str = "form.button.action"; // ui-text-exempt: a trace region name, never displayed

/// The prefix each popup row is published under, suffixed with the kind.
///
/// ★ A driven check reads `form.button.action.row.ResetForm`, not `…row.1`.
/// See the publisher for why: an index survives a reordering of
/// `ButtonDoesKind::ALL` and goes on passing while aiming at the wrong row.
const ROW_REGION: &str = "form.button.action.row"; // ui-text-exempt: a trace region name, never displayed

/// Draw the chooser and its parameters into `ui`, editing `does` in place.
///
/// Returns nothing: the draft is the output, and the dialog reads
/// [`ButtonDoes::blocker`] itself when it decides whether Add may be pressed.
/// A `bool` return would be a second opinion about the same question.
pub fn rows(ui: &mut Ui, does: &mut ButtonDoes) {
    ui.add_space(8.0);
    ui.label(t::does_label());

    let before = does.kind;
    let combo = egui::ComboBox::from_id_salt("form_button_action_kind")
        .width(ui.available_width())
        .selected_text(t::does_choice(does.kind))
        .show_ui(ui, |ui| {
            for kind in ButtonDoesKind::ALL {
                let row = ui.selectable_value(&mut does.kind, kind, t::does_choice(kind));
                // ★★★ **A POPUP ROW'S RECTANGLE CAN ONLY BE PUBLISHED FROM
                // INSIDE THE POPUP**, which is why this is here rather than in
                // the harness.
                //
                // `egui`'s combo popup is an `Area` laid out at paint time. It
                // exists for the frames it is open and nowhere else, so a driven
                // check has no way to compute where the rows are — it can only
                // read what the application says. Recorded in `D:/dev/rag/egui/`
                // as `a_combobox_popup_is_an_area_laid_out_at_paint_time…`, and
                // this is the third control in this shell to need it.
                //
                // ★ Named per KIND rather than by index. An index-named region
                // would keep passing after the order of `ALL` changed, aiming a
                // check at whatever now sits third.
                crate::diag::ui_rect(&format!("{ROW_REGION}.{kind:?}"), row.rect);
            }
        });
    // ★★ The closed control's own rectangle, published unconditionally, because
    // a check has to click it to open the popup in the first place. `response`
    // is the button; `inner` is `Some` only while the popup is open.
    crate::diag::ui_rect(COMBO_REGION, combo.response.rect);
    if does.kind != before {
        let kind = does.kind;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "{CHOSE} kind={kind:?} reaches_outside={}",
                kind.reaches_outside()
            )
        });
    }

    // ★ The reach sentence, for every choice including the inert ones. See
    // `text::buttonaction::does_note` for why the inert ones carry one: a
    // sentence that appears only on the two addressed choices is a sentence an
    // operator learns to skip.
    ui.add_space(2.0);
    ui.small(t::does_note(does.kind));

    ui.add_space(6.0);
    match does.kind {
        ButtonDoesKind::Nothing | ButtonDoesKind::ResetForm => {}
        ButtonDoesKind::GoToPage => page_rows(ui, does),
        ButtonDoesKind::Named => named_rows(ui, does),
        ButtonDoesKind::ShowHide => show_hide_rows(ui, does),
        ButtonDoesKind::Uri => url_row(ui, does),
        ButtonDoesKind::SubmitForm => {
            url_row(ui, does);
            submit_disclosure(ui, does);
        }
    }

    // ★★ The blocker sentence sits under the boxes it is about, and is drawn
    // whenever it applies rather than only after a failed press. The dialog
    // greys Add on the same predicate, so an operator who cannot press it can
    // always see the reason without pressing anything.
    if let Some(reason) = does.blocker() {
        ui.add_space(4.0);
        ui.small(t::blocker(reason));
    }
}

/// **Go to a page** — the number, and where on it to land.
///
/// The number box is a plain text field rather than a `DragValue`, and that is
/// deliberate: a `DragValue` cannot be empty, so it would have to open at some
/// page, and opening at page 1 is pdfcer choosing a destination. An empty box is
/// the honest starting state and [`ButtonDoes::blocker`] refuses it.
fn page_rows(ui: &mut Ui, does: &mut ButtonDoes) {
    ui.horizontal(|ui| {
        ui.label(t::page_number_label());
        let response = ui.add(
            egui::TextEdit::singleline(&mut does.page_number)
                .desired_width(60.0)
                .char_limit(6),
        );
        if response.changed() {
            // Digits only. Filtering on change rather than validating on commit
            // means a stray letter never reaches the box, which is one fewer
            // refusal to word.
            does.page_number.retain(|c| c.is_ascii_digit());
        }
    });
    ui.add_space(4.0);
    egui::ComboBox::from_id_salt("form_button_action_view")
        .width(ui.available_width())
        .selected_text(t::page_view_choice(does.view))
        .show_ui(ui, |ui| {
            for view in PageViewChoice::ALL {
                ui.selectable_value(&mut does.view, view, t::page_view_choice(view));
            }
        });
}

/// **Move through the pages** — which of the four.
fn named_rows(ui: &mut Ui, does: &mut ButtonDoes) {
    egui::ComboBox::from_id_salt("form_button_action_named")
        .width(ui.available_width())
        .selected_text(t::named_choice(does.named))
        .show_ui(ui, |ui| {
            for named in NamedChoice::ALL {
                ui.selectable_value(&mut does.named, named, t::named_choice(named));
            }
        });
}

/// **Show or hide fields** — the names, and which direction.
///
/// ★★ Two radio buttons rather than one *Hidden* checkbox, and the engine's own
/// CLI made the same choice for the same reason: *show* is the value that has
/// to be written out to exist (Table 210's `/H` defaults to **true**, so an
/// absent entry means HIDE), and a single `Hidden` checkbox left unticked is
/// one misreading away from an operator believing they configured "show" when
/// they configured nothing.
fn show_hide_rows(ui: &mut Ui, does: &mut ButtonDoes) {
    ui.horizontal(|ui| {
        ui.radio_value(&mut does.hide, true, t::hide_them());
        ui.radio_value(&mut does.hide, false, t::show_them());
    });
    ui.add_space(4.0);
    ui.label(t::targets_label());
    ui.add(
        egui::TextEdit::multiline(&mut does.targets)
            .desired_width(f32::INFINITY)
            .desired_rows(3),
    );
    ui.add_space(2.0);
    ui.small(t::targets_note());
}

/// The address box, shared by *Open a web address* and *Send the form's data*.
fn url_row(ui: &mut Ui, does: &mut ButtonDoes) {
    ui.label(t::url_label());
    ui.add(egui::TextEdit::singleline(&mut does.url).desired_width(f32::INFINITY));
}

/// ★★★ **The submit's disclosure**, drawn under the address it is about.
///
/// Two blocks, and the second is conditional:
///
/// - **Always**: what the declaration would cover — the six §12.7.5.2 facts
///   nobody can guess, in `text::buttonaction::submit_disclosure`.
/// - **When the address is not `https:`**: that it is unencrypted. A
///   **statement**, never a refusal — no scheme is blocked anywhere, because
///   `https` appears zero times in ISO 32000-1 and refusing one would be pdfcer
///   inventing a conformance requirement.
///
/// Both are off-canvas by construction: they are in a dialog, and the button
/// they describe is drawn on the page exactly as the saved file will draw it.
fn submit_disclosure(ui: &mut Ui, does: &ButtonDoes) {
    ui.add_space(6.0);
    ui.small(t::submit_disclosure());
    if url_is_unencrypted(&does.url) {
        ui.add_space(4.0);
        ui.small(t::submit_unencrypted());
    }
}
