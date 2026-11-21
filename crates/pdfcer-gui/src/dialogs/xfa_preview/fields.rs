//! Form preview's fields: where each field lies on a drawn page, what entry
//! it takes, and the editor a click opens over it.
//!
//! The entry is the engine's `xfa::field_widget`: a text field opens for
//! typing, a check box turns on or off with one click, a radio button is
//! chosen by filling its exclusion group with the button's on value, and a
//! choice list opens its choices — below the field for a drop-down, over it
//! for a list always shown — and writes the save value of the one picked.
//! A `multiSelect` list takes several: a click picks one alone, as in any
//! list, and Ctrl+click adds or removes a choice and keeps the list open,
//! which writes its choices one per line when it closes.
//! A widget Form preview cannot fill (image, signature, button, barcode)
//! says so on hover and opens nothing.
//!
//! The hover outline is the cursor, not content: it is drawn only while the
//! pointer is over a field. Leaving a typed field by Enter, Esc or a click
//! elsewhere writes a changed value, as one undo step: typed words are never
//! thrown away, and Ctrl+Z takes them back.

use pdfcer_core::xfa::{
    ChoiceOpen, ContainerKind, FieldWidget, LayoutPage, MergedForm, field_widget,
};
use pdfcer_gui_base::formpreview::FormPreviewAction;

use crate::app::actions::Action;
use crate::text::xfapreview::{self as t, FieldClick};

/// What a click on a field opens.
enum Entry {
    /// Typing, in one line or several, masked for a password.
    Text {
        multiline: bool,
        max_chars: Option<usize>,
        password: bool,
    },
    /// One click writes `on` or `off`; a radio button writes `on` to its
    /// exclusion `group` instead, and only when it is not already chosen.
    Check {
        on: String,
        off: String,
        group: Option<Vec<usize>>,
    },
    /// A list of `(display, save)` choices, of which one is written, or
    /// several one per line when `multi`.
    Choice {
        items: Vec<(String, String)>,
        drop_down: bool,
        text_entry: bool,
        multi: bool,
    },
    /// A widget Form preview cannot fill, by element name.
    Inert(String),
}

impl Entry {
    fn of(merged: &MergedForm, path: &[usize]) -> Self {
        let text = |multiline, max_chars, password| Self::Text {
            multiline,
            max_chars,
            password,
        };
        match field_widget(&merged.template, path) {
            Some(FieldWidget::Text {
                multiline,
                max_chars,
            }) => text(multiline, max_chars, false),
            Some(FieldWidget::CheckButton { on, off, group, .. }) => Self::Check { on, off, group },
            Some(FieldWidget::ChoiceList {
                items,
                open,
                text_entry,
            }) => Self::Choice {
                items,
                drop_down: open.is_drop_down(),
                text_entry,
                multi: open == ChoiceOpen::MultiSelect,
            },
            Some(FieldWidget::Other(w)) if w == "passwordEdit" => text(false, None, true),
            Some(FieldWidget::Other(w)) => Self::Inert(w),
            _ => text(false, None, false),
        }
    }

    /// The trace's name for the entry.
    fn kind(&self) -> &'static str {
        match self {
            Self::Text { .. } => "text",
            Self::Check { group: None, .. } => "check",
            Self::Check { .. } => "radio",
            Self::Choice {
                drop_down: true, ..
            } => "drop-down",
            Self::Choice { multi: true, .. } => "multi",
            Self::Choice { .. } => "list",
            Self::Inert(_) => "inert",
        }
    }
}

/// One field on a drawn page; `rect` is in page points from the top-left.
pub(super) struct FieldBox {
    path: Vec<usize>,
    name: Option<String>,
    rect: egui::Rect,
    /// The bound value: the exclusion group's for a radio button.
    value: String,
    entry: Entry,
}

/// The visible fields of `page`, with the values `merged` binds to them.
pub(super) fn of_page(page: &LayoutPage, merged: &MergedForm) -> Vec<FieldBox> {
    let bound = |path: &[usize]| {
        merged
            .bound
            .iter()
            .find(|b| b.path == path)
            .map(|b| b.value.clone())
            .unwrap_or_default()
    };
    page.items
        .iter()
        .filter(|i| i.kind == ContainerKind::Field && i.visible)
        .map(|i| {
            let entry = Entry::of(merged, &i.path);
            let value = match &entry {
                Entry::Check { group: Some(g), .. } => bound(g),
                _ => bound(&i.path),
            };
            FieldBox {
                path: i.path.clone(),
                name: i.name.clone(),
                rect: egui::Rect::from_min_size(
                    egui::pos2(i.rect.x as f32, i.rect.y as f32),
                    egui::vec2(i.rect.w as f32, i.rect.h as f32),
                ),
                value,
                entry,
            }
        })
        .collect()
}

/// The field open for typing or picking: its page, its index in that page's
/// fields, the text so far, and a `multiSelect` list's choices so far.
pub(super) struct Editing {
    page: usize,
    index: usize,
    text: String,
    picks: Vec<String>,
    focused: bool,
}

/// The selections in a `multiSelect` list's value, read by the engine's
/// rule (`FieldWidget::selections`), which the painter and the fillable
/// planner share.
fn selections(value: &str, items: &[(String, String)]) -> Vec<String> {
    let list = FieldWidget::ChoiceList {
        items: items.to_vec(),
        open: ChoiceOpen::MultiSelect,
        text_entry: false,
    };
    list.selections(value)
        .unwrap_or_default()
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// The indices of `items` whose save value is among `picks`, comma-joined
/// for the trace; `none` when there are none.
fn ticked(items: &[(String, String)], picks: &[String]) -> String {
    let on: Vec<String> = items
        .iter()
        .enumerate()
        .filter(|(_, (_, save))| picks.contains(save))
        .map(|(k, _)| k.to_string())
        .collect();
    if on.is_empty() {
        "none".to_owned()
    } else {
        on.join(",")
    }
}

/// What the overlay of one page needs from the window.
pub(super) struct Overlay<'a> {
    /// Zero-based page index.
    pub page: usize,
    /// Where the page's picture is on screen.
    pub image: egui::Rect,
    /// Screen points per page point.
    pub scale: f32,
    /// `false` while a redraw is in flight: a click then opens nothing.
    pub enabled: bool,
    pub choices: pdfcer_gui_base::formpreview::ReadingChoices,
    pub actions: &'a mut Vec<Action>,
}

impl Overlay<'_> {
    /// Write `value` to the form's data at `path`.
    fn fill(&mut self, f: &FieldBox, path: &[usize], value: String) {
        self.actions
            .push(Action::FormPreview(FormPreviewAction::Fill {
                path: path.to_vec(),
                value,
                name: f.name.clone(),
                choices: self.choices,
            }));
    }
}

/// Draw `fields` over their page's picture: the hover outline, and the
/// editor of the field open for typing or picking.
pub(super) fn overlay(
    ui: &mut egui::Ui,
    mut o: Overlay<'_>,
    fields: &[FieldBox],
    editing: &mut Option<Editing>,
) {
    for (n, f) in fields.iter().enumerate() {
        let r = egui::Rect::from_min_size(
            o.image.min + f.rect.min.to_vec2() * o.scale,
            f.rect.size() * o.scale,
        );
        // ui-text-exempt: trace region name, never displayed
        crate::diag::ui_rect(&format!("form-preview.field.{}.{n}", o.page), r);
        if let Some(e) = editing
            .as_mut()
            .filter(|e| e.page == o.page && e.index == n)
        {
            let open = match &f.entry {
                Entry::Choice {
                    items, multi: true, ..
                } => pick_several(ui, &mut o, f, r, e, items),
                Entry::Choice {
                    items,
                    drop_down,
                    text_entry,
                    ..
                } => pick(ui, &mut o, f, r, e, (items, *drop_down, *text_entry)),
                _ => typing(ui, &mut o, f, r, e),
            };
            if !open {
                *editing = None;
            }
            continue;
        }
        if o.enabled && hovered_click(ui, &o, f, r, n) {
            pressed(&mut o, f, n, editing);
        }
    }
}

/// The hover outline and hover text; `true` when the field was clicked.
fn hovered_click(
    ui: &mut egui::Ui,
    o: &Overlay<'_>,
    f: &FieldBox,
    r: egui::Rect,
    n: usize,
) -> bool {
    let id = ui.id().with(("form-preview-field", o.page, n));
    let hover = match &f.entry {
        Entry::Inert(widget) => {
            let _ = ui
                .interact(r, id, egui::Sense::hover())
                .on_hover_text(t::unfillable_hover(f.name.as_deref(), widget));
            return false;
        }
        Entry::Text { .. } => FieldClick::Type,
        Entry::Check { group: None, .. } => FieldClick::Toggle,
        Entry::Check { .. } => FieldClick::Choose,
        Entry::Choice { multi: true, .. } => FieldClick::PickSeveral,
        Entry::Choice { .. } => FieldClick::Pick,
    };
    let hit = ui
        .interact(r, id, egui::Sense::click())
        .on_hover_text(t::fill_hover(f.name.as_deref(), hover));
    if hit.hovered() {
        let icon = if hover == FieldClick::Type {
            egui::CursorIcon::Text
        } else {
            egui::CursorIcon::PointingHand
        };
        ui.ctx().set_cursor_icon(icon);
        ui.painter().rect_stroke(
            r,
            0.0,
            egui::Stroke::new(
                1.5,
                egui_shell::theme::Theme::canvas_selection_ink(ui.ctx()),
            ),
            egui::StrokeKind::Outside,
        );
    }
    hit.clicked()
}

/// A click on field `n`: a check box or radio button writes at once, and
/// any other field opens for typing or picking.
fn pressed(o: &mut Overlay<'_>, f: &FieldBox, n: usize, editing: &mut Option<Editing>) {
    if let Entry::Check { on, off, group } = &f.entry {
        let checked = f.value == *on;
        let (path, value) = match group {
            Some(_) if checked => return,
            Some(g) => (g.as_slice(), on.clone()),
            None if checked => (f.path.as_slice(), off.clone()),
            None => (f.path.as_slice(), on.clone()),
        };
        let kind = f.entry.kind();
        let state = if value == *on { "on" } else { "off" };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "form-preview-pick page={} index={n} kind={kind} state={state}",
                o.page
            )
        });
        o.fill(f, path, value);
        return;
    }
    let mut picks = Vec::new();
    if let Entry::Choice { items, multi, .. } = &f.entry {
        picks = if *multi {
            selections(&f.value, items)
        } else {
            vec![f.value.clone()]
        };
        let kind = f.entry.kind();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "form-preview-list page={} index={n} kind={kind} items={} ticked={}",
                o.page,
                items.len(),
                ticked(items, &picks)
            )
        });
    }
    *editing = Some(Editing {
        page: o.page,
        index: n,
        text: f.value.clone(),
        picks,
        focused: false,
    });
}

/// The in-place text editor; `false` once it has closed.
fn typing(
    ui: &mut egui::Ui,
    o: &mut Overlay<'_>,
    f: &FieldBox,
    r: egui::Rect,
    e: &mut Editing,
) -> bool {
    let (multiline, max_chars, password) = match &f.entry {
        Entry::Text {
            multiline,
            max_chars,
            password,
        } => (*multiline, *max_chars, *password),
        _ => (false, None, false),
    };
    // escape-disposition: commits — Esc ends focus, and `lost_focus` writes.
    let editor = if multiline {
        egui::TextEdit::multiline(&mut e.text)
    } else {
        egui::TextEdit::singleline(&mut e.text)
    };
    let editor = editor.password(password);
    let editor = match max_chars {
        Some(n) => editor.char_limit(n),
        None => editor,
    };
    let edit = ui.put(r, editor);
    if !e.focused {
        edit.request_focus();
        e.focused = true;
    }
    if !edit.lost_focus() {
        return true;
    }
    if e.text != f.value {
        let value = std::mem::take(&mut e.text);
        o.fill(f, &f.path, value);
    }
    false
}

/// A choice list's open list, below the field for a drop-down and over it
/// otherwise, with a typed entry above it when the list takes one; `false`
/// once it has closed.
fn pick(
    ui: &mut egui::Ui,
    o: &mut Overlay<'_>,
    f: &FieldBox,
    r: egui::Rect,
    e: &mut Editing,
    (items, drop_down, text_entry): (&[(String, String)], bool, bool),
) -> bool {
    let at = if drop_down {
        r.left_bottom()
    } else {
        r.left_top()
    };
    let mut picked: Option<(usize, String)> = None;
    let mut typed = false;
    let area = egui::Area::new(ui.id().with(("form-preview-list", o.page, e.index)))
        .order(egui::Order::Foreground)
        .fixed_pos(at)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(r.width());
                if text_entry {
                    // escape-disposition: commits — Esc ends focus, and `lost_focus` writes.
                    let edit = ui.text_edit_singleline(&mut e.text);
                    typed = edit.lost_focus() && e.text != f.value;
                }
                for (k, (display, save)) in items.iter().enumerate() {
                    let item = ui.selectable_label(*save == f.value, display.as_str());
                    // ui-text-exempt: trace region name, never displayed
                    crate::diag::ui_rect(&format!("form-preview.item.{k}"), item.rect);
                    if item.clicked() {
                        picked = Some((k, save.clone()));
                    }
                }
            });
        });
    if let Some((k, save)) = picked {
        let kind = f.entry.kind();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "form-preview-pick page={} index={} kind={kind} item={k}",
                o.page, e.index
            )
        });
        if save != f.value {
            o.fill(f, &f.path, save);
        }
        return false;
    }
    if typed {
        let value = std::mem::take(&mut e.text);
        o.fill(f, &f.path, value);
        return false;
    }
    !area.response.clicked_elsewhere()
}

/// A `multiSelect` list's open list over the field: a click writes the one
/// item alone, a Ctrl+click adds or removes an item and keeps the list open,
/// and closing it by Enter, Esc or a click elsewhere writes the choices so
/// far when they changed; `false` once it has closed.
fn pick_several(
    ui: &mut egui::Ui,
    o: &mut Overlay<'_>,
    f: &FieldBox,
    r: egui::Rect,
    e: &mut Editing,
    items: &[(String, String)],
) -> bool {
    let mut alone: Option<(usize, String)> = None;
    let area = egui::Area::new(ui.id().with(("form-preview-list", o.page, e.index)))
        .order(egui::Order::Foreground)
        .fixed_pos(r.left_top())
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(r.width());
                for (k, (display, save)) in items.iter().enumerate() {
                    let on = e.picks.contains(save);
                    let item = ui.selectable_label(on, display.as_str());
                    // ui-text-exempt: trace region name, never displayed
                    crate::diag::ui_rect(&format!("form-preview.item.{k}"), item.rect);
                    if !item.clicked() {
                        continue;
                    }
                    if !ui.input(|i| i.modifiers.command) {
                        alone = Some((k, save.clone()));
                        continue;
                    }
                    if on {
                        e.picks.retain(|p| p != save);
                    } else {
                        e.picks.push(save.clone());
                    }
                    let state = if on { "off" } else { "on" };
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        format!(
                            "form-preview-toggle page={} index={} item={k} state={state}",
                            o.page, e.index
                        )
                    });
                }
            });
        });
    if let Some((k, save)) = alone {
        e.picks = vec![save];
        write_several(o, f, e, items, &k.to_string());
        return false;
    }
    // escape-disposition: commits — Esc closes the list and writes its choices.
    let keyed = ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Escape));
    if !keyed && !area.response.clicked_elsewhere() {
        return true;
    }
    let picked = ticked(items, &e.picks);
    write_several(o, f, e, items, &picked);
    false
}

/// Write a `multiSelect` list's choices so far, one per line in the list's
/// order with any choice the list does not name kept after them, when they
/// differ from its value; `picked` names the items for the trace.
fn write_several(
    o: &mut Overlay<'_>,
    f: &FieldBox,
    e: &Editing,
    items: &[(String, String)],
    picked: &str,
) {
    let listed = |p: &str| items.iter().any(|(_, save)| save == p);
    let mut value: Vec<&str> = items
        .iter()
        .map(|(_, save)| save.as_str())
        .filter(|save| e.picks.iter().any(|p| p == save))
        .collect();
    value.extend(e.picks.iter().map(String::as_str).filter(|p| !listed(p)));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "form-preview-pick page={} index={} kind=multi item={picked}",
            o.page, e.index
        )
    });
    if selections(&f.value, items) != value {
        o.fill(f, &f.path, value.join("\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::{selections, ticked};

    fn towns() -> Vec<(String, String)> {
        ["Toronto", "New York", "Boston"]
            .iter()
            .map(|t| ((*t).to_owned(), (*t).to_owned()))
            .collect()
    }

    #[test]
    fn selections_are_one_per_line() {
        assert_eq!(
            selections("New York\nBoston", &towns()),
            ["New York", "Boston"]
        );
    }

    #[test]
    fn a_save_value_with_a_space_is_one_selection() {
        assert_eq!(selections("New York", &towns()), ["New York"]);
    }

    #[test]
    fn a_one_line_value_that_is_no_save_value_reads_space_separated() {
        assert_eq!(
            selections("Toronto Boston", &towns()),
            ["Toronto", "Boston"]
        );
        assert!(selections("", &towns()).is_empty());
    }

    #[test]
    fn ticked_names_the_items_in_list_order() {
        let picks = ["Boston".to_owned(), "Toronto".to_owned()];
        assert_eq!(ticked(&towns(), &picks), "0,2");
        assert_eq!(ticked(&towns(), &[]), "none");
    }
}
