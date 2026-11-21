//! # `app::actions::forms::radiogroup` — putting a radio button into a group
//!
//! Properties ▸ *Radio group* moves the selected button into the named group,
//! or into a new one. (Another button pasted into its group is
//! `super::paste`'s, by `paste_field`.)
//!
//! Into an existing group it is the engine's `move_widget_to_field`, which
//! picks an on-state no member uses (§12.7.3.2: a shared name is what makes
//! radios exclusive). Into a new name it is `add_radio_button`, which takes
//! the source button's mark, colours and border, then `delete_widget`, folded
//! into one undo step (`coalesce_last`).

use pdfcer_core::annot_author::{CheckStyle, WidgetChrome};
use pdfcer_core::edit::{BorderSpec, BorderStyle, CommandKind, NewRadioButton, TooltipChoice};
use pdfcer_core::forms::{Field, FieldFlags, Widget};
use pdfcer_core::page_tree::Rect;

use crate::app::state::{OpenDoc, SelectedField};

/// Every export value the radio group `fqn` uses, empty when there is none.
#[must_use]
pub fn exports(doc: &OpenDoc, fqn: &str) -> Vec<String> {
    with_field(doc, fqn, |f| {
        f.widgets
            .iter()
            .flat_map(|w| w.on_states.iter())
            .map(|s| String::from_utf8_lossy(s).into_owned())
            .collect()
    })
    .unwrap_or_default()
}

/// What a new member copies from the button it is made from.
#[derive(Debug, Clone)]
struct Look {
    style: CheckStyle,
    chrome: WidgetChrome,
    border: BorderSpec,
    read_only: bool,
    required: bool,
    tooltip: TooltipChoice,
    export: String,
    visibility: Option<pdfcer_core::edit::Visibility>,
}

/// **Move the button `from`'s widget `widget` into the radio group `to`**,
/// creating `to` when no field bears that name. One undo step.
///
/// Into an existing group it is one `move_widget_to_field`: the widget keeps
/// its object and its look, and the engine picks a free on-state. Into a new
/// name it is `add_radio_button` then `delete_widget`, folded, because the
/// engine's verb needs a target field that exists.
pub(super) fn move_to(doc: &mut OpenDoc, from: &SelectedField, to: &str) {
    let to = to.trim();
    if to.is_empty() || to == from.field {
        return;
    }
    if with_field(doc, to, |_| ()).is_some() {
        regroup(doc, from, to);
    } else {
        move_to_new(doc, from, to);
    }
}

/// [`move_to`] into the existing field `to`.
fn regroup(doc: &mut OpenDoc, from: &SelectedField, to: &str) {
    let page = from.page;
    let (field, widget) = (from.field.clone(), from.widget);
    let before = doc.edit_epoch;
    crate::app::actions::apply::vector_edit(doc, "radio-move", page, 1, |session| {
        let done = session.move_widget_to_field(&field, widget, to)?;
        let export = done.on_state.clone().unwrap_or_default();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "radio-move-applied group={to} export={export} page={page} \
                 engine=move_widget_to_field source_removed={} orphaned={}",
                done.source_removed, done.action_targets_orphaned
            )
        });
        let mut lines = vec![crate::text::radiogroup::moved(to, &export)];
        lines.extend(done.disclosures);
        if done.action_targets_orphaned > 0 {
            lines.push(crate::text::forms::form_field_actions_orphaned(
                done.action_targets_orphaned,
            ));
        }
        Ok::<_, pdfcer_core::edit::EditError>(lines)
    });
    select_last(doc, before, to, page);
}

/// [`move_to`] into a group that does not exist yet.
fn move_to_new(doc: &mut OpenDoc, from: &SelectedField, to: &str) {
    let Some(Some((look, rect))) = with_field(doc, &from.field, |f| {
        let w = f.widgets.get(from.widget)?;
        Some((look_of(f, w), w.rect?))
    }) else {
        return;
    };
    let export = free_export(&look.export, &[]);
    let page = from.page;
    let (field, widget) = (from.field.clone(), from.widget);
    let before = doc.edit_epoch;
    crate::app::actions::apply::vector_edit(doc, "radio-move", page, 1, |session| {
        session.add_radio_button(&spec(page, to, rect, &look, &export))?;
        if let Err(e) = session.delete_widget(&field, widget) {
            session.undo();
            return Err(e);
        }
        let folded = session.coalesce_last(2, CommandKind::AddFormField);
        trace_join("radio-move", to, &export, page);
        let mut lines = vec![crate::text::radiogroup::moved(to, &export)];
        if !folded {
            lines.push(crate::text::radiogroup::TWO_UNDOS.to_owned());
        }
        Ok(lines)
    });
    select_last(doc, before, to, page);
}

fn trace_join(label: &str, group: &str, export: &str, page: usize) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("{label}-applied group={group} export={export} page={page}")
    });
}

/// Leave the new member selected, if the edit happened.
fn select_last(doc: &mut OpenDoc, before: u64, group: &str, page: usize) {
    if doc.edit_epoch == before {
        return;
    }
    let widget = with_field(doc, group, |f| f.widgets.len().saturating_sub(1)).unwrap_or(0);
    doc.selected_field = Some(SelectedField {
        field: group.to_owned(),
        widget,
        page,
    });
}

fn spec(page: usize, group: &str, rect: Rect, look: &Look, export: &str) -> NewRadioButton {
    let mut spec = NewRadioButton::new(page, group, rect, export.to_owned());
    spec.tooltip = look.tooltip.clone();
    spec.read_only = look.read_only;
    spec.required = look.required;
    spec.border = look.border;
    spec.chrome = look.chrome.clone();
    spec.style = look.style;
    if let Some(v) = look.visibility {
        spec.visibility = v;
    }
    spec
}

fn look_of(field: &Field, widget: &Widget) -> Look {
    let style = widget
        .caption
        .as_deref()
        .and_then(|c| c.first().copied())
        .and_then(CheckStyle::from_mk_caption_char)
        .unwrap_or(CheckStyle::Circle);
    let tooltip = field
        .alternate_name
        .as_deref()
        .map(|t| String::from_utf8_lossy(t).trim().to_owned())
        .filter(|t| !t.is_empty())
        .map_or(TooltipChoice::Declined, TooltipChoice::Text);
    let chrome = WidgetChrome::new(widget.background, widget.border_color);
    Look {
        style,
        chrome: match widget.border_dash.clone() {
            Some(dash) => chrome.with_border_dash(dash),
            None => chrome,
        },
        visibility: widget.visibility,
        border: widget.border.unwrap_or(BorderSpec {
            style: BorderStyle::Solid,
            width: 1.0,
        }),
        read_only: field.flags.has(FieldFlags::READ_ONLY),
        required: field.flags.has(FieldFlags::REQUIRED),
        tooltip,
        export: widget
            .on_states
            .first()
            .map(|s| String::from_utf8_lossy(s).into_owned())
            .unwrap_or_default(),
    }
}

/// `preferred` when no member uses it, else its stem with the lowest free
/// number: `Choice1` beside `Choice1` and `Choice2` becomes `Choice3`.
#[must_use]
pub fn free_export(preferred: &str, taken: &[String]) -> String {
    let preferred = if preferred.is_empty() || preferred == "Off" {
        "Choice1" // ui-text-exempt: a PDF /AS name written into the file
    } else {
        preferred
    };
    if !taken.iter().any(|t| t == preferred) {
        return preferred.to_owned();
    }
    let stem = preferred.trim_end_matches(|c: char| c.is_ascii_digit());
    let stem = if stem.is_empty() { preferred } else { stem };
    (1..=taken.len() + 1)
        .map(|n| format!("{stem}{n}"))
        .find(|c| !taken.iter().any(|t| t == c))
        .unwrap_or_else(|| format!("{stem}{}", taken.len() + 2))
}

fn with_field<T>(doc: &OpenDoc, fqn: &str, read: impl FnOnce(&Field) -> T) -> Option<T> {
    let view = doc.session.view();
    let form = pdfcer_core::forms::parse_acroform(&view)?;
    form.field_by_name(fqn).map(read)
}

#[cfg(test)]
mod tests {
    use super::free_export;

    #[test]
    fn a_free_export_skips_every_member_not_only_the_last() {
        let taken = ["Choice1".to_owned(), "Choice2".to_owned()];
        assert_eq!(free_export("Choice2", &taken), "Choice3");
        assert_eq!(free_export("A", &["A".to_owned(), "B".to_owned()]), "A1");
        assert_eq!(free_export("Yes", &[]), "Yes");
        assert_eq!(free_export("", &[]), "Choice1");
    }
}
