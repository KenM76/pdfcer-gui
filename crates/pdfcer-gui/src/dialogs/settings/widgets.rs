//! # `dialogs::settings::widgets` — the three shapes every setting is made of
//!
//! Seven group modules draw thirteen settings, and every one of them is built
//! from the three functions here. That is deliberate: a settings window whose
//! entries are hand-laid-out drifts into thirteen slightly different layouts
//! within a year, and the reader notices the inconsistency before they notice
//! the content.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/settings/widgets.md`.

use egui::{RichText, Ui};

/// One collapsible subject group.
pub fn group(
    ui: &mut Ui,
    key: &str,
    heading: &str,
    open_by_default: bool,
    body: impl FnOnce(&mut Ui),
) {
    group_focused(ui, key, heading, open_by_default, false, body);
}

/// [`group`], and whether this is the group the window was opened **for**.
pub fn group_focused(
    ui: &mut Ui,
    key: &str,
    heading: &str,
    open_by_default: bool,
    focused: bool,
    body: impl FnOnce(&mut Ui),
) {
    let mut header = egui::CollapsingHeader::new(RichText::new(heading));
    if focused {
        header = header.open(Some(true));
    }
    let response = header.default_open(open_by_default).show(ui, body);
    if focused {
        // The HEADER's response, so the window lands with the heading at the
        // top of the view rather than the body's last row. `Align::TOP` for the
        // same reason — the operator asked for this group and wants to read it
        // downward, not to arrive at its end.
        response
            .header_response
            .scroll_to_me(Some(egui::Align::TOP));
    }
    // The HEADER's rect, not the whole collapsible's.
    //
    // `CollapsingHeaderResponse::header_response` is the row carrying the text;
    // the outer rect would include the expanded body, and a contrast check
    // measuring that would sample a hundred lines of prose and average the
    // heading away. D2 was a defect in one row of pixels, and it measured about
    // 1.1:1 — a figure only obtainable from the row itself.
    // `ui_rect_visible`, not `ui_rect` — these headings live in a
    // `ScrollArea` and `egui` lays out the ones below the fold before clipping
    // them. Publishing a rect for a heading nobody can see makes a contrast
    // check measure whatever is genuinely at those coordinates, which on the
    // first live run of `settings_headings_legible` was the Pages panel and
    // the drawing behind the dialog — reported as three illegible headings in
    // a dialog whose visible headings measured 13.91:1. See
    // `crate::diag::ui_rect_visible`.
    crate::diag::ui_rect_visible(
        &format!("{}{key}", super::REGION_HEADING_PREFIX),
        response.header_response.rect,
        ui.clip_rect(),
    );
    ui.add_space(2.0);
}

/// One setting's three lines: what it is, what is open, and what it costs.
pub fn header(ui: &mut Ui, title: &str, silence: &str, radius: &str) {
    ui.label(RichText::new(title));
    ui.label(RichText::new(silence).small().weak());
    ui.label(RichText::new(radius).small().weak());
    ui.add_space(2.0);
}

/// One radio option, with an optional gloss under it.
pub fn option<T: PartialEq>(
    ui: &mut Ui,
    current: &mut T,
    value: T,
    label: &str,
    note: Option<&str>,
) {
    ui.radio_value(current, value, label);
    if let Some(note) = note
        && !note.is_empty()
    {
        ui.label(RichText::new(note).small().weak());
    }
}

/// One switch, with an optional gloss under it.
pub fn toggle(ui: &mut Ui, value: &mut bool, label: &str, note: Option<&str>) {
    ui.checkbox(value, label);
    if let Some(note) = note
        && !note.is_empty()
    {
        ui.label(RichText::new(note).small().weak());
    }
}

/// **A free-text setting, with the parse shown rather than enforced.**
pub fn text_value<T: Clone + PartialEq>(
    ui: &mut Ui,
    id: &str,
    value: &mut T,
    label: &str,
    note: Option<&str>,
    format: impl Fn(&T) -> String,
    parse: impl Fn(&str) -> Option<T>,
) {
    ui.label(label);
    // The buffer lives in `egui::Memory` keyed on this control's id, not in the
    // draft: the draft holds a parsed VALUE and this holds the operator's
    // keystrokes, and the two are legitimately different while a number is
    // half-typed. Seeded from the value the first time the control is drawn, so
    // reopening the window shows what is stored rather than an empty box.
    let id = egui::Id::new(id);
    let mut buffer = ui
        .ctx()
        .data_mut(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| format(value));
    // escape-disposition: dialog-cancels — `dialogs::host` owns the key for
    // every field in this window: the first press leaves the box, the second
    // cancels.
    let response = ui.add(egui::TextEdit::singleline(&mut buffer).desired_width(140.0));
    if response.changed()
        && let Some(parsed) = parse(&buffer)
    {
        *value = parsed;
    }
    ui.ctx().data_mut(|d| d.insert_temp(id, buffer.clone()));

    match parse(&buffer) {
        Some(parsed) => {
            let echo = format(&parsed);
            // Echoed only when the operator's spelling and the canonical one
            // differ. `256 MiB` typed back as `256 MiB` is noise; `0.25gb`
            // answered with `256 MiB` is the whole reason this line exists.
            if echo != buffer {
                ui.label(
                    RichText::new(crate::text::settings::parsed_as(&echo))
                        .small()
                        .weak(),
                );
            }
        }
        None => {
            ui.label(
                RichText::new(crate::text::settings::unparsed_value_note())
                    .small()
                    // `notice` rather than `danger`: nothing is broken and
                    // nothing was lost — the stored value still stands and the
                    // operator is mid-keystroke. `danger` is for an act that
                    // destroys something.
                    .color(egui_shell::theme::Theme::of(ui.ctx()).palette.notice),
            );
        }
    }
    if let Some(note) = note
        && !note.is_empty()
    {
        ui.label(RichText::new(note).small().weak());
    }
}

/// A sentence the operator must see but that belongs to the **setting**, not to
/// any one of its options.
pub fn disclosure(ui: &mut Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(RichText::new(text).small());
}
