//! # `dialogs::buttonaction` — the *What pressing it does* chooser
//!
//! One control, drawn into the form-field placement dialog when the kind being
//! placed is a push button. Lifted out of `dialogs::formfield` rather than
//! written inline for two reasons: R2 (that file has five kinds' worth of rows
//! already), and because this is the only row group in the dialog that carries
//! a **disclosure obligation**, which is easier to review when it is not
//! interleaved with a comb-cell checkbox.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/buttonaction.md`.

use egui::Ui;

use crate::canvas::formfield::action::{
    ButtonDoes, ButtonDoesKind, NamedChoice, PageViewChoice, url_is_unencrypted,
};
use crate::text::buttonaction as t;

/// The trace event the chooser writes when the operator changes it.
const CHOSE: &str = "button-action-chose"; // ui-text-exempt: a trace event name, never displayed

/// The closed chooser's rectangle, so a driven check can open it.
const COMBO_REGION: &str = "form.button.action"; // ui-text-exempt: a trace region name, never displayed

/// The prefix each popup row is published under, suffixed with the kind.
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
                // **A POPUP ROW'S RECTANGLE CAN ONLY BE PUBLISHED FROM
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
                // Named per KIND rather than by index. An index-named region
                // would keep passing after the order of `ALL` changed, aiming a
                // check at whatever now sits third.
                crate::diag::ui_rect(&format!("{ROW_REGION}.{kind:?}"), row.rect);
            }
        });
    // The closed control's own rectangle, published unconditionally, because
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

    // The reach sentence, for every choice including the inert ones. See
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

    // The blocker sentence sits under the boxes it is about, and is drawn
    // whenever it applies rather than only after a failed press. The dialog
    // greys Add on the same predicate, so an operator who cannot press it can
    // always see the reason without pressing anything.
    if let Some(reason) = does.blocker() {
        ui.add_space(4.0);
        ui.small(t::blocker(reason));
    }
}

/// **Go to a page** — the number, and where on it to land.
fn page_rows(ui: &mut Ui, does: &mut ButtonDoes) {
    ui.horizontal(|ui| {
        ui.label(t::page_number_label());
        let response = ui.add(
            // escape-disposition: dialog-cancels — `dialogs::host` owns the key for
            // every field in this window: the first press leaves the box, the second
            // cancels.
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
fn show_hide_rows(ui: &mut Ui, does: &mut ButtonDoes) {
    ui.horizontal(|ui| {
        ui.radio_value(&mut does.hide, true, t::hide_them());
        ui.radio_value(&mut does.hide, false, t::show_them());
    });
    ui.add_space(4.0);
    ui.label(t::targets_label());
    ui.add(
        // escape-disposition: dialog-cancels — `dialogs::host` owns the key for
        // every field in this window: the first press leaves the box, the second
        // cancels.
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
    // escape-disposition: dialog-cancels — `dialogs::host` owns the key for
    // every field in this window: the first press leaves the box, the second
    // cancels.
    ui.add(egui::TextEdit::singleline(&mut does.url).desired_width(f32::INFINITY));
}

/// **The submit's disclosure**, drawn under the address it is about.
fn submit_disclosure(ui: &mut Ui, does: &ButtonDoes) {
    ui.add_space(6.0);
    ui.small(t::submit_disclosure());
    if url_is_unencrypted(&does.url) {
        ui.add_space(4.0);
        ui.small(t::submit_unencrypted());
    }
}
