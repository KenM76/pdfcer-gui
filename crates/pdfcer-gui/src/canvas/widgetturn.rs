//! **A selected form field's box turned by its rotate grip.**
//!
//! A drag commits `turn_widget` with the box's new free angle — the verb takes
//! the angle absolute, so the current one is read first (`widgetread`, G177).
//! A drag that lands on a quarter turn commits `rotate_widget` instead, which
//! writes `/MK /R` where every viewer keeps it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/widgetturn.md`.

use std::path::PathBuf;
use std::sync::Arc;

use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::app::state::OpenDoc;
use pdfcer_gui_base::widgetread;

/// The egui memory slot the per-revision reading is cached in.
const KEY: &str = "pdfcer-canvas-widget-turn"; // ui-text-exempt: internal memory id, never displayed

/// The selected widget a rotate drag turns, as the file states it now.
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    /// The field's fully-qualified name.
    pub field: String,
    /// Which of its widgets.
    pub widget: usize,
    /// The free angle, counterclockwise degrees in (-180, 180]; `0.0` upright.
    pub free: f64,
    /// `/MK /R`, counterclockwise, in `0..360`.
    pub quarter: i64,
    /// The unturned box's width and height in points, in the `/Rect` frame;
    /// `None` when upright, where `/Rect` is that box.
    pub size: Option<(f64, f64)>,
}

impl Target {
    /// Whether the box stands at a free angle, which `rotate_widget` and a
    /// resize refuse.
    #[must_use]
    pub fn turned(&self) -> bool {
        self.free != 0.0
    }
}

type Key = (PathBuf, u64, String, usize);

/// **The selected widget's reading**, once per document revision.
#[must_use]
pub fn target(ctx: &egui::Context, doc: &OpenDoc) -> Option<Arc<Target>> {
    let selected = doc.selected_field.as_ref()?;
    let key: Key = (
        doc.path.clone(),
        doc.edit_epoch,
        selected.field.clone(),
        selected.widget,
    );
    let id = egui::Id::new(KEY);
    if let Some((cached, found)) = ctx.data(|d| d.get_temp::<(Key, Option<Arc<Target>>)>(id))
        && cached == key
    {
        return found;
    }
    let found = read(doc, &selected.field, selected.widget).map(Arc::new);
    ctx.data_mut(|d| d.insert_temp(id, (key, found.clone())));
    found
}

fn read(doc: &OpenDoc, field: &str, index: usize) -> Option<Target> {
    let view = doc.session.view();
    let form = pdfcer_core::forms::parse_acroform(&view)?;
    let widget = form
        .fields
        .iter()
        .find(|f| f.fully_qualified_name == field)?
        .widgets
        .get(index)?;
    let turn = widgetread::free_turn(&doc.session, widget);
    Some(Target {
        field: field.to_owned(),
        widget: index,
        free: turn.map_or(0.0, |t| t.degrees),
        quarter: widget.rotation.unwrap_or(0).rem_euclid(360),
        size: turn.map(|t| (t.w, t.h)),
    })
}

/// The grips a selected widget offers: all nine upright, and the rotate
/// handle alone while turned, because `resize_widget` refuses a turned box.
pub fn offer(ctx: &egui::Context, doc: &OpenDoc) -> crate::canvas::handles::GripSet {
    match target(ctx, doc) {
        Some(t) if t.turned() => crate::canvas::handles::GripSet::rotate_only(),
        _ => crate::canvas::handles::GripSet::all(),
    }
}

/// The rotate ghost's box: the unturned widget's screen extent and the
/// screen angle it stands at (y down, so a counterclockwise page angle is
/// negative). Upright, that is the grip box itself at angle zero.
#[must_use]
pub fn ghost_box(
    target: &Target,
    map: &crate::canvas::mapping::PageMapping,
    grip_box: egui::Rect,
) -> (egui::Vec2, f32) {
    match target.size {
        #[allow(clippy::cast_possible_truncation)]
        Some((w, h)) => (
            map.page_vec_to_screen(egui::vec2(w as f32, h as f32)).abs(),
            -(target.free as f32).to_radians(),
        ),
        None => (grip_box.size(), 0.0),
    }
}

/// **Raise the verb a release at `degrees` (counterclockwise, page space)
/// asks for.**
///
/// The new free angle is `free + degrees`. Landing on a non-zero quarter turn
/// it becomes `/MK /R` instead: the free angle is stood back up first when
/// there is one, so `rotate_widget` does not refuse a turned box.
pub fn commit(target: &Target, degrees: f64, actions: &mut Vec<Action>) {
    let total = widgetread::normalise(target.free + degrees);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "widget-turn-commit field={} widget={} free={:.2} deg={degrees:.2} total={total:.2} \
             quarter={}",
            target.field, target.widget, target.free, target.quarter
        )
    });
    if widgetread::is_quarter(total) && total.round() != 0.0 {
        if target.turned() {
            actions.push(turn(target, 0.0));
        }
        #[allow(clippy::cast_possible_truncation)]
        let steps = (total / 90.0).round() as i64;
        actions.push(Action::Field(FieldAction::RotateWidget {
            field: target.field.clone(),
            widget: target.widget,
            degrees: (target.quarter + steps * 90).rem_euclid(360),
        }));
        return;
    }
    let total = if widgetread::is_quarter(total) {
        0.0
    } else {
        total
    };
    actions.push(turn(target, total));
}

fn turn(target: &Target, degrees: f64) -> Action {
    Action::Field(FieldAction::TurnWidget {
        field: target.field.clone(),
        widget: target.widget,
        degrees,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upright() -> Target {
        Target {
            field: "Name".into(),
            widget: 0,
            free: 0.0,
            quarter: 0,
            size: None,
        }
    }

    fn sent(t: &Target, deg: f64) -> Vec<FieldAction> {
        let mut out = Vec::new();
        commit(t, deg, &mut out);
        out.into_iter()
            .map(|a| match a {
                Action::Field(f) => f,
                other => panic!("not a field action: {other:?}"),
            })
            .collect()
    }

    #[test]
    fn a_free_drag_turns_to_the_sum() {
        let t = Target {
            free: 30.0,
            size: Some((100.0, 20.0)),
            ..upright()
        };
        assert!(
            matches!(&sent(&t, 15.0)[..], [FieldAction::TurnWidget { degrees, .. }] if (*degrees - 45.0).abs() < 1e-9)
        );
    }

    #[test]
    fn a_drag_landing_on_a_quarter_turn_writes_mk_r() {
        assert!(matches!(
            &sent(&upright(), 90.0)[..],
            [FieldAction::RotateWidget { degrees: 90, .. }]
        ));
        let t = Target {
            free: 30.0,
            quarter: 90,
            size: Some((100.0, 20.0)),
            ..upright()
        };
        assert!(matches!(
            &sent(&t, 60.0)[..],
            [FieldAction::TurnWidget { degrees, .. }, FieldAction::RotateWidget { degrees: 180, .. }]
                if *degrees == 0.0
        ));
    }

    #[test]
    fn a_drag_back_to_upright_stands_it_up() {
        let t = Target {
            free: 30.0,
            size: Some((100.0, 20.0)),
            ..upright()
        };
        assert!(
            matches!(&sent(&t, -30.0)[..], [FieldAction::TurnWidget { degrees, .. }] if *degrees == 0.0)
        );
    }
}
