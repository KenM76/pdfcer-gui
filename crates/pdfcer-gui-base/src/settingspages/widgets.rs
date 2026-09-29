//! # `dialogs::settings::widgets` — the shapes every setting is made of
//!
//! Every setting is drawn with these functions, so the pages share one layout,
//! and every option name passes through them, which is what the search reads.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/settingspages/widgets.md`.

use std::cell::RefCell;

use egui::{RichText, Ui};

thread_local! {
    /// The search, lower-cased and trimmed; empty when there is none.
    static QUERY: RefCell<String> = const { RefCell::new(String::new()) };
    /// The names handed to the widgets below while [`collect`] runs.
    static NAMES: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
}

/// Set the search the widgets below mark matches of, for this frame.
pub fn set_query(query: &str) {
    QUERY.with(|q| *q.borrow_mut() = query.trim().to_lowercase());
}

/// Run `draw` and return, lower-cased, every option name the widgets below
/// were handed while it ran. The settings search is built from this.
pub fn collect(draw: impl FnOnce()) -> Vec<String> {
    let was = NAMES.with(|n| n.replace(Some(Vec::new())));
    draw();
    NAMES.with(|n| n.replace(was)).unwrap_or_default()
}

/// Record `name` for [`collect`] and return it styled: underlined in the
/// `notice` colour when it contains the search. Not `.strong()`, which no
/// theme here renders legibly on a panel (`DEFECTS.md` D11).
fn name(ui: &Ui, name: &str) -> RichText {
    let lower = name.to_lowercase();
    let hit = QUERY.with(|q| {
        let q = q.borrow();
        !q.is_empty() && lower.contains(q.as_str())
    });
    NAMES.with(|n| {
        if let Some(names) = n.borrow_mut().as_mut() {
            names.push(lower);
        }
    });
    let text = RichText::new(name);
    if hit {
        text.underline()
            .color(egui_shell::theme::Theme::of(ui.ctx()).palette.notice)
    } else {
        text
    }
}

/// One setting's three lines: what it is, what is open, and what it costs.
pub fn header(ui: &mut Ui, title: &str, silence: &str, radius: &str) {
    ui.label(name(ui, title));
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
    let label = name(ui, label);
    ui.radio_value(current, value, label);
    if let Some(note) = note
        && !note.is_empty()
    {
        ui.label(RichText::new(note).small().weak());
    }
}

/// One switch, with an optional gloss under it.
pub fn toggle(ui: &mut Ui, value: &mut bool, label: &str, note: Option<&str>) {
    let label = name(ui, label);
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
    ui.label(name(ui, label));
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
