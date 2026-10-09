//! # `panels::properties::strokestyle` — line width, dash and opacity of the
//! selected page objects, or of the selected parts of one placed drawing
//!
//! Paths get Width, Style, Line opacity and Fill opacity; pictures and placed
//! drawings get one Picture opacity. Width and dash are shown and sent in
//! **points** (`app::actions::strokestyle` converts to each path's user space).
//! A row whose members disagree shows the first member's value and says so; a
//! new value sets them all.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/strokestyle.md`.

use egui::Ui;
use pdfcer_core::vector::{Dash, StrokeStyle, VectorObject};
use pdfcer_gui_base::entry;
use pdfcer_gui_base::linestyle::{DashReading, LineStyle};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::target::CanvasTargetProvider as _;
use crate::text::strokestyle as t;

/// The trace slot, and the prefix of every region this section publishes.
const REGION: &str = "properties.stroke"; // ui-text-exempt: a trace region name
const REGION_WIDTH: &str = "properties.stroke.width"; // ui-text-exempt: a trace region name
const REGION_DASH: &str = "properties.stroke.dash"; // ui-text-exempt: a trace region name
const REGION_LINE_ALPHA: &str = "properties.stroke.line-opacity"; // ui-text-exempt: a trace region name
const REGION_FILL_ALPHA: &str = "properties.stroke.fill-opacity"; // ui-text-exempt: a trace region name
const REGION_PICTURE_ALPHA: &str = "properties.stroke.picture-opacity"; // ui-text-exempt: a trace region name
const REGION_REPLACE: &str = "properties.stroke.replace-image"; // ui-text-exempt: a trace region name

/// The widest line the field offers, in points.
const MAX_WIDTH_PT: f64 = 144.0;
/// Dash lengths in points are compared to the offered patterns at this
/// resolution, so a pattern scaled through a CTM still reads as itself.
const DASH_RESOLUTION: f64 = 1e-3;

/// One value across a selection: the first member's, and whether any differ.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Reading {
    first: f64,
    mixed: bool,
}

impl Reading {
    fn of(values: impl IntoIterator<Item = f64>) -> Option<Self> {
        let mut it = values.into_iter();
        let first = it.next()?;
        let mixed = it.any(|v| (v - first).abs() > 1e-6);
        Some(Self { first, mixed })
    }

    fn token(self) -> String {
        if self.mixed {
            "mixed".to_owned() // ui-text-exempt: trace token
        } else {
            format!("{:.3}", self.first)
        }
    }
}

/// A selected path as the rows show it.
struct PathRead {
    index: usize,
    width_pt: f64,
    dash: DashReading,
    stroke_alpha: f64,
    fill_alpha: f64,
}

/// What the selection holds that this section can act on.
struct Subjects {
    paths: Vec<PathRead>,
    /// Pictures and placed drawings: `(index, fill_alpha)`.
    pictures: Vec<(usize, f64)>,
}

/// Draw the section. `true` if anything was drawn.
pub fn section(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    if doc.selection.annot().is_some() {
        return false;
    }
    let page = doc.view.page_index;
    let Some((objects, leaves)) = selected(doc, page) else {
        return false;
    };
    let Some(subjects) = read(doc, page, &objects, leaves) else {
        return false;
    };
    if subjects.paths.is_empty() && subjects.pictures.is_empty() {
        return false;
    }
    trace(&subjects, leaves);

    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
    ui.label(t::heading());

    let mut chosen: Option<(Vec<usize>, StrokeStyle)> = None;
    if !subjects.paths.is_empty() {
        let indices: Vec<usize> = subjects.paths.iter().map(|p| p.index).collect();
        let mut send = |style: StrokeStyle| chosen = Some((indices.clone(), style));
        path_rows(ui, &subjects.paths, &mut send);
    }
    if let Some(reading) = Reading::of(subjects.pictures.iter().map(|&(_, a)| a))
        && let Some(a) = opacity_row(
            ui,
            t::picture_opacity_label(),
            reading,
            REGION_PICTURE_ALPHA,
        )
    {
        chosen = Some((
            subjects.pictures.iter().map(|&(i, _)| i).collect(),
            // A placed drawing starts from both alphas; an image reads only
            // the fill one, and the stroke one is inert there.
            StrokeStyle {
                fill_alpha: Some(a),
                stroke_alpha: Some(a),
                ..StrokeStyle::default()
            },
        ));
    }
    if let Some((objects, style)) = chosen {
        actions.push(Action::SetObjectStrokeStyle {
            page,
            objects,
            leaves,
            style,
        });
    }
    replace_row(ui, doc, actions);
    true
}

/// Format ▸ Replace image's panel twin, drawn on the dispatcher's own test.
fn replace_row(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) {
    use crate::app::dispatch::replaceimage::{ID, replaceable_image};
    if !crate::canvas::tool::capabilities(ui.ctx()).edit_content || replaceable_image(doc).is_none()
    {
        return;
    }
    let words = crate::text::commands::format_replace_image();
    let button = ui.button(words.label).on_hover_text(words.tooltip);
    crate::diag::ui_rect_visible(REGION_REPLACE, button.rect, ui.clip_rect());
    if button.clicked() {
        actions.push(Action::Command(ID.to_owned()));
    }
}

/// The four path rows; `send` takes the one edit a frame can raise.
fn path_rows(ui: &mut Ui, paths: &[PathRead], send: &mut impl FnMut(StrokeStyle)) {
    if let Some(reading) = Reading::of(paths.iter().map(|p| p.width_pt))
        && let Some(w) = width_row(ui, reading)
    {
        send(StrokeStyle {
            width: Some(w),
            ..StrokeStyle::default()
        });
    }
    if let Some(style) = dash_row(ui, dash_of(paths)) {
        send(StrokeStyle {
            dash: Some(Dash::new(
                style.pattern().map_or_else(Vec::new, <[f64]>::to_vec),
                0.0,
            )),
            ..StrokeStyle::default()
        });
    }
    if let Some(reading) = Reading::of(paths.iter().map(|p| p.stroke_alpha))
        && let Some(a) = opacity_row(ui, t::line_opacity_label(), reading, REGION_LINE_ALPHA)
    {
        send(StrokeStyle {
            stroke_alpha: Some(a),
            ..StrokeStyle::default()
        });
    }
    if let Some(reading) = Reading::of(paths.iter().map(|p| p.fill_alpha))
        && let Some(a) = opacity_row(ui, t::fill_opacity_label(), reading, REGION_FILL_ALPHA)
    {
        send(StrokeStyle {
            fill_alpha: Some(a),
            ..StrokeStyle::default()
        });
    }
}

/// What the style sections act on: the selected page objects, else the
/// selected parts of a placed drawing (`true`). A selection holding both acts
/// on its page objects, because one verb call cannot take both index spaces.
pub(super) fn selected(doc: &OpenDoc, page: usize) -> Option<(Vec<usize>, bool)> {
    let objects = doc.selection.object_indices_on(page);
    if !objects.is_empty() {
        return Some((objects, false));
    }
    let leaves = doc.selection.leaf_indices_on(page);
    (!leaves.is_empty()).then_some((leaves, true))
}

/// Read every selected object in one borrow of the provider, dropped before
/// anything is drawn.
fn read(doc: &OpenDoc, page: usize, objects: &[usize], leaves: bool) -> Option<Subjects> {
    let provider = doc.page_objects()?;
    let model = provider.page_objects_model(page)?;
    let mut subjects = Subjects {
        paths: Vec::new(),
        pictures: Vec::new(),
    };
    for &index in objects {
        match crate::app::actions::styled_object(model, index, leaves) {
            Some(object @ VectorObject::Path(path)) => {
                let k = crate::app::actions::stroke_scale(object);
                subjects.paths.push(PathRead {
                    index,
                    width_pt: path.line_width * k,
                    dash: dash_reading(&path.dash, k),
                    stroke_alpha: path.stroke_alpha,
                    fill_alpha: path.fill_alpha,
                });
            }
            Some(VectorObject::Image(image)) => subjects.pictures.push((index, image.fill_alpha)),
            _ => {}
        }
    }
    Some(subjects)
}

/// A path's dash, scaled to points, as the chooser names it.
fn dash_reading(dash: &Dash, k: f64) -> DashReading {
    if dash.is_solid() {
        return DashReading::Solid;
    }
    let pt: Vec<f64> = dash
        .array
        .iter()
        .map(|v| (v * k / DASH_RESOLUTION).round() * DASH_RESOLUTION)
        .collect();
    LineStyle::of_pattern(&pt).map_or(DashReading::Foreign, DashReading::Offered)
}

/// The selection's dash: the members' common reading, or [`DashReading::Mixed`].
fn dash_of(paths: &[PathRead]) -> DashReading {
    let first = paths.first().map_or(DashReading::Solid, |p| p.dash);
    if paths.iter().all(|p| p.dash == first) {
        first
    } else {
        DashReading::Mixed
    }
}

/// The width field, in points. `Some` once, when a drag or an edit ends on a
/// new value.
fn width_row(ui: &mut Ui, reading: Reading) -> Option<f64> {
    let mut width = reading.first;
    let mut chosen = None;
    ui.horizontal(|ui| {
        ui.label(t::width_label());
        let (widget, refusal) = entry::drag_value(
            ui,
            &mut width,
            entry::Kind::Length(entry::LengthUnit::Point),
        );
        let response = refusal.show(ui.add(widget.range(0.0..=MAX_WIDTH_PT).speed(0.1)));
        crate::diag::ui_rect_visible(REGION_WIDTH, response.rect, ui.clip_rect());
        // Once, at the end of the gesture: every pixel of a drag is otherwise a
        // content rewrite and an undo entry.
        if (response.drag_stopped() || response.lost_focus())
            && (width - reading.first).abs() > 1e-6
        {
            chosen = Some(width);
        }
        mixed_note(ui, reading.mixed);
    });
    chosen
}

/// The dash chooser, with each entry's rect published for a driven check.
fn dash_row(ui: &mut Ui, current: DashReading) -> Option<LineStyle> {
    let mut picked = None;
    ui.horizontal(|ui| {
        ui.label(t::dash_label());
        let combo = egui::ComboBox::from_id_salt(REGION_DASH)
            .selected_text(current.label())
            .show_ui(ui, |ui| {
                for (i, &style) in LineStyle::ALL.iter().enumerate() {
                    let selected = current.selected() == Some(style);
                    let entry = ui.selectable_label(selected, style.label());
                    crate::diag::ui_rect_visible(
                        &format!("{REGION_DASH}.{i}"), // ui-text-exempt: a trace region name
                        entry.rect,
                        ui.clip_rect(),
                    );
                    if entry.clicked() && !selected {
                        picked = Some(style);
                    }
                }
            });
        crate::diag::ui_rect_visible(REGION_DASH, combo.response.rect, ui.clip_rect());
    });
    picked
}

/// A 0–100 % opacity field. `Some` (as 0..=1) once, when an edit ends on a
/// new value.
fn opacity_row(ui: &mut Ui, label: String, reading: Reading, region: &str) -> Option<f64> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let shown = (reading.first * 100.0).round().clamp(0.0, 100.0) as u8;
    let mut percent = shown;
    let mut chosen = None;
    ui.horizontal(|ui| {
        ui.label(label);
        let (widget, refusal) = entry::drag_value(ui, &mut percent, entry::Kind::Number(&["%"]));
        let response = refusal.show(
            ui.add(
                widget
                    .range(0..=100)
                    .speed(1.0)
                    .suffix(crate::text::panels::properties::markup_opacity_suffix()),
            ),
        );
        crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());
        if (response.drag_stopped() || response.lost_focus()) && percent != shown {
            chosen = Some(f64::from(percent) / 100.0);
        }
        mixed_note(ui, reading.mixed);
    });
    chosen
}

fn mixed_note(ui: &mut Ui, mixed: bool) {
    if mixed {
        ui.label(egui::RichText::new(t::mixed()).small().weak());
    }
}

/// One line per change of what the section shows, for a driven check.
fn trace(subjects: &Subjects, leaves: bool) {
    let none = || "none".to_owned(); // ui-text-exempt: trace token
    let paths = &subjects.paths;
    crate::diag::trace_changed(REGION, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            "stroke-style-shown leaves={leaves} paths={} pictures={} width_pt={} dash={} \
             line_alpha={} fill_alpha={} picture_alpha={}",
            paths.len(),
            subjects.pictures.len(),
            Reading::of(paths.iter().map(|p| p.width_pt)).map_or_else(none, Reading::token),
            if paths.is_empty() {
                "none" // ui-text-exempt: trace token
            } else {
                crate::panels::properties::widgetdash::token(dash_of(paths))
            },
            Reading::of(paths.iter().map(|p| p.stroke_alpha)).map_or_else(none, Reading::token),
            Reading::of(paths.iter().map(|p| p.fill_alpha)).map_or_else(none, Reading::token),
            Reading::of(subjects.pictures.iter().map(|&(_, a)| a))
                .map_or_else(none, Reading::token),
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dash_drawn_at_half_scale_reads_as_the_pattern_in_points() {
        let dash = Dash::new(vec![16.0, 8.0], 0.0);
        assert_eq!(
            dash_reading(&dash, 0.5),
            DashReading::Offered(LineStyle::LongDash)
        );
    }

    #[test]
    fn a_pattern_off_by_rounding_still_reads_as_itself() {
        let k = 0.24;
        let dash = Dash::new(vec![3.0 / k], 0.0);
        assert_eq!(
            dash_reading(&dash, k),
            DashReading::Offered(LineStyle::Dashed)
        );
    }

    #[test]
    fn an_unoffered_pattern_is_foreign_and_no_dash_is_solid() {
        assert_eq!(
            dash_reading(&Dash::new(vec![5.0, 2.0], 0.0), 1.0),
            DashReading::Foreign
        );
        assert_eq!(dash_reading(&Dash::default(), 1.0), DashReading::Solid);
    }

    fn path(dash: DashReading) -> PathRead {
        PathRead {
            index: 0,
            width_pt: 1.0,
            dash,
            stroke_alpha: 1.0,
            fill_alpha: 1.0,
        }
    }

    /// Mixed only when members disagree; the counterpart test keeps `Mixed`
    /// from being returned unconditionally.
    #[test]
    fn disagreeing_dashes_read_as_mixed_and_agreeing_ones_do_not() {
        let solid = path(DashReading::Solid);
        let dashed = path(DashReading::Offered(LineStyle::Dashed));
        assert_eq!(dash_of(&[solid, dashed]), DashReading::Mixed);
        let a = path(DashReading::Offered(LineStyle::Dashed));
        let b = path(DashReading::Offered(LineStyle::Dashed));
        assert_eq!(dash_of(&[a, b]), DashReading::Offered(LineStyle::Dashed));
    }

    #[test]
    fn a_reading_is_mixed_only_when_values_differ() {
        assert_eq!(
            Reading::of([2.0, 2.0]),
            Some(Reading {
                first: 2.0,
                mixed: false
            })
        );
        assert!(Reading::of([2.0, 3.0]).is_some_and(|r| r.mixed));
        assert_eq!(Reading::of(std::iter::empty()), None);
    }
}
