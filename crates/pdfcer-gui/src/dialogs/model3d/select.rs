//! The Measure tab's *Face, edge, corner* measure. Each pick is the face, the
//! model edge or the corner a click lands on, as the operator chooses, and
//! the engine measures it (`face_figures`, `edge_at`, `vertex_at`); a second
//! pick also gives the smallest, largest and centre distances between the two
//! (`entity_distance`). A third pick starts again.
//!
//! Faces and curved edges are fitted from triangles (rule 4), so every fitted
//! figure is said with its fit, and a file that separates no faces is said to
//! give the whole part's figures. Everything is said off the picture; the
//! picture marks only the picks, the picked edge and the smallest span.
//! Nothing is written to the file. The unit, the coordinate system and the
//! projection the figures are said in are `readout`'s.

use egui::{RichText, Ui};
use pdfcer_3d::vec3::sub;
use pdfcer_3d::{
    CoordinateFrame, DEFAULT_CORNER_DEGREES, DEFAULT_FLAT_DEGREES, EdgeFigures, EdgeShape,
    EntityDistance, FaceFigures, FaceShape, FaceSource, Hit, MeasureEntity, ProjectedArea,
    Selection, SurfacePick, TriangleMesh, edge_at, entity_distance, face_figures,
    face_projected_area, vertex_at,
};

use super::ModelView;
use super::measure::{SELECT, Said};
use super::measure_tab::REGION_START_AGAIN;
use super::readout::{self, Change, Readout, triple};
use super::surfaces::row;
use crate::text::model3dreadout::{self as tr, Lengths};
use crate::text::model3dselect as t;

/// The measure's button in the kind row.
pub const REGION_SELECT: &str = "model3d.measure.select"; // ui-text-exempt: trace region name, never displayed
/// Each choice of what a click picks, suffixed with its place in [`ENTITIES`].
pub const REGION_ENTITY_PREFIX: &str = "model3d.select.entity."; // ui-text-exempt: trace region name, never displayed

/// What a click may pick, in [`t::ENTITIES`]' order.
pub(super) const ENTITIES: [MeasureEntity; 3] = [
    MeasureEntity::Face,
    MeasureEntity::Edge,
    MeasureEntity::Vertex,
];
/// The headline's text size, in points.
const VALUE_SIZE: f32 = 16.0;
/// A row that says nothing and keeps its height.
const BLANK: &str = " "; // ui-text-exempt: an empty row, no words

/// What one pick measures.
#[derive(Debug, Clone)]
enum Figures {
    Corner([f64; 3]),
    Edge(EdgeFigures),
    /// Where the figures came from is `figures.source`, said under them when
    /// it is not the file's own face.
    Face(FaceFigures),
}

/// One pick and what the engine made of it.
#[derive(Debug, Clone)]
struct Picked {
    selection: Selection,
    figures: Result<Figures, String>,
}

/// What the chosen projection made of the picks, and the plane's normal and
/// the number of picks it was made for, so a turn or a pick remakes it.
#[derive(Debug, Clone, Default)]
struct Projected {
    normal: Option<[f64; 3]>,
    picks: usize,
    /// The smallest span's projected length and the signed part removed.
    minimum: Option<(f64, f64)>,
    /// Each pick's projected area, for a face pick.
    faces: Vec<Option<Result<ProjectedArea, String>>>,
}

/// The measure's state; the choice of what a click picks and the readout's
/// choices outlive the picks.
#[derive(Debug, Clone, Default)]
pub(super) struct Selected {
    /// What a click picks, a place in [`ENTITIES`].
    pub entity: usize,
    /// How the figures are said.
    pub readout: Readout,
    picks: Vec<Picked>,
    distance: Option<Result<EntityDistance, String>>,
    projected: Projected,
}

impl Selected {
    /// Forget the picks and what they measured.
    pub(super) fn clear(&mut self) {
        self.picks.clear();
        self.distance = None;
        self.projected = Projected::default();
    }

    /// The system two straight edge picks set, when they can.
    fn frame_of_picks(&self) -> Option<CoordinateFrame> {
        let (Some(Ok(d)), [a, b]) = (&self.distance, self.picks.as_slice()) else {
            return None;
        };
        let (Ok(Figures::Edge(first)), Ok(Figures::Edge(second))) = (&a.figures, &b.figures) else {
            return None;
        };
        readout::frame_from(first, second, &d.minimum)
    }

    /// Remake the projection when the plane or the picks changed.
    fn project(&mut self, meshes: &[TriangleMesh], toward_eye: Option<[f64; 3]>) {
        let normal = self.readout.normal(toward_eye);
        if self.projected.normal == normal && self.projected.picks == self.picks.len() {
            return;
        }
        let minimum = match (&self.distance, normal) {
            (Some(Ok(d)), Some(n)) => Readout::flattened(&d.minimum, n),
            _ => None,
        };
        let faces = self
            .picks
            .iter()
            .map(|p| match (&p.figures, normal) {
                (Ok(Figures::Face(_)), Some(n)) => Some(
                    face_projected_area(meshes, &p.selection.pick, n).map_err(|e| e.to_string()),
                ),
                _ => None,
            })
            .collect();
        self.projected = Projected {
            normal,
            picks: self.picks.len(),
            minimum,
            faces,
        };
        trace_projected(self.readout.plane_word(), &self.projected);
    }

    fn holds(&self) -> bool {
        !self.picks.is_empty()
    }

    fn entity(&self) -> MeasureEntity {
        ENTITIES[self.entity.min(ENTITIES.len() - 1)]
    }

    /// The picked points, to mark on the picture.
    pub(super) fn marks(&self) -> impl Iterator<Item = [f64; 3]> + '_ {
        self.picks.iter().map(|p| match &p.figures {
            Ok(Figures::Corner(at)) => *at,
            _ => p.selection.pick.point,
        })
    }

    /// The picked edges' corners in order, a closed edge's first again at
    /// its end, to trace on the picture.
    pub(super) fn edges(&self) -> impl Iterator<Item = Vec<[f64; 3]>> + '_ {
        self.picks.iter().filter_map(|p| match &p.figures {
            Ok(Figures::Edge(e)) => {
                let mut points = e.points.clone();
                if let (true, Some(&first)) = (e.closed, e.points.first()) {
                    points.push(first);
                }
                Some(points)
            }
            _ => None,
        })
    }

    /// The smallest span between the two picks, to draw.
    pub(super) fn span(&self) -> Option<([f64; 3], [f64; 3])> {
        match &self.distance {
            Some(Ok(d)) => Some((d.minimum.from, d.minimum.to)),
            _ => None,
        }
    }

    /// Take a pick; the second also measures between the two.
    fn pick(&mut self, meshes: &[TriangleMesh], hit: &Hit) {
        if self.picks.len() >= 2 {
            self.clear();
        }
        let selection = Selection {
            pick: SurfacePick::from(hit),
            entity: self.entity(),
        };
        let n = self.picks.len() + 1;
        let figures = figures_of(meshes, &selection);
        trace_pick(n, &selection, &figures, self.readout.frame.as_ref());
        self.picks.push(Picked { selection, figures });
        if let [a, b] = self.picks.as_slice() {
            let d = entity_distance(
                meshes,
                &a.selection,
                &b.selection,
                DEFAULT_FLAT_DEGREES,
                DEFAULT_CORNER_DEGREES,
            )
            .map_err(|e| e.to_string());
            trace_distance(&d);
            self.distance = Some(d);
        }
    }
}

fn figures_of(meshes: &[TriangleMesh], s: &Selection) -> Result<Figures, String> {
    let figures = match s.entity {
        MeasureEntity::Vertex => vertex_at(meshes, &s.pick).map(Figures::Corner),
        MeasureEntity::Edge => edge_at(
            meshes,
            &s.pick,
            DEFAULT_FLAT_DEGREES,
            DEFAULT_CORNER_DEGREES,
        )
        .map(Figures::Edge),
        _ => face_figures(meshes, &s.pick, DEFAULT_FLAT_DEGREES).map(Figures::Face),
    };
    figures.map_err(|e| e.to_string())
}

fn source_word(s: FaceSource) -> &'static str {
    match s {
        FaceSource::File => "file",
        FaceSource::Grown => "grown",
        _ => "whole-mesh",
    }
}

fn entity_word(e: MeasureEntity) -> &'static str {
    match e {
        MeasureEntity::Vertex => "corner",
        MeasureEntity::Edge => "edge",
        _ => "face",
    }
}

fn trace_pick(
    n: usize,
    s: &Selection,
    figures: &Result<Figures, String>,
    frame: Option<&CoordinateFrame>,
) {
    let [x, y, z] = s.pick.point;
    let local = match (figures, frame) {
        (Ok(Figures::Corner(at)), Some(f)) => format!(" local={}", triple(f.point_to_local(*at))),
        _ => String::new(),
    };
    let head = format!(
        "n={n} entity={} mesh={} triangle={} at={x:.6},{y:.6},{z:.6}", // ui-text-exempt: diagnostic trace, never displayed
        entity_word(s.entity),
        s.pick.mesh,
        s.pick.triangle
    );
    // ui-text-exempt: diagnostic trace, never displayed
    crate::diag::trace(|| match figures {
        Ok(f) => format!("model-select-figures {head} {}{local}", trace_figures(f)),
        Err(why) => format!("model-select-refused {head} why={why}"),
    });
}

fn trace_figures(f: &Figures) -> String {
    match f {
        Figures::Corner([x, y, z]) => format!("corner={x:.6},{y:.6},{z:.6}"),
        Figures::Edge(e) => {
            let (shape, radius) = match e.shape {
                EdgeShape::Line { .. } => ("line", String::from("-")),
                EdgeShape::Circle { radius, .. } => ("circle", format!("{radius:.6}")),
                _ => ("curve", String::from("-")),
            };
            format!(
                "length={:.6} chord={:.6} closed={} shape={shape} radius={radius}", // ui-text-exempt: diagnostic trace, never displayed
                e.length,
                e.chord,
                u8::from(e.closed)
            )
        }
        Figures::Face(f) => {
            let (shape, radius) = match f.shape {
                FaceShape::Plane { .. } => ("plane", String::from("-")),
                FaceShape::Cylinder { radius, .. } => ("cylinder", format!("{radius:.6}")),
                _ => ("unfitted", String::from("-")),
            };
            format!(
                "area={:.6} perimeter={:.6} shape={shape} radius={radius} source={} triangles={}", // ui-text-exempt: diagnostic trace, never displayed
                f.area,
                f.perimeter,
                source_word(f.source),
                f.triangles
            )
        }
    }
}

fn trace_distance(d: &Result<EntityDistance, String>) {
    // ui-text-exempt: diagnostic trace, never displayed
    crate::diag::trace(|| match d {
        Ok(d) => {
            let [dx, dy, dz] = d.minimum.delta();
            let centres = d
                .centres
                .map_or_else(|| "-".to_owned(), |c| format!("{:.6}", c.length));
            format!(
                "model-select-distance minimum={:.6} maximum={:.6} centres={centres} \
                 dx={dx:.6} dy={dy:.6} dz={dz:.6}",
                d.minimum.length, d.maximum.length
            )
        }
        Err(why) => format!("model-select-distance-refused why={why}"),
    });
}

fn trace_projected(plane: &str, p: &Projected) {
    if p.minimum.is_none() && p.faces.iter().all(Option::is_none) {
        return;
    }
    let (minimum, out) = p.minimum.map_or_else(
        || ("-".to_owned(), "-".to_owned()),
        |(m, o)| (format!("{m:.6}"), format!("{o:.6}")),
    );
    let faces: String = p
        .faces
        .iter()
        .enumerate()
        .map(|(i, f)| match f {
            Some(Ok(a)) => format!(" face{}={:.6}/{:.6}", i + 1, a.toward, a.away),
            Some(Err(_)) => format!(" face{}=refused", i + 1),
            None => format!(" face{}=-", i + 1),
        })
        .collect();
    // ui-text-exempt: diagnostic trace, never displayed
    crate::diag::trace(|| {
        format!("model-select-projected plane={plane} minimum={minimum} out={out}{faces}")
    });
}

/// The lines one pick's figures are said in, headline first; a corner in
/// `r`'s system.
fn figure_lines(f: &Figures, unit: &Lengths<'_>, r: &Readout) -> Vec<String> {
    match f {
        Figures::Corner(at) => vec![t::corner(r.point(*at), unit)],
        Figures::Edge(e) => {
            let mut lines = vec![t::edge_length(e.length, unit)];
            match e.shape {
                EdgeShape::Line { .. } => lines.push(t::STRAIGHT.to_owned()),
                EdgeShape::Circle {
                    radius,
                    residual,
                    sweep_degrees,
                    arc_length,
                    ..
                } => {
                    lines.push(t::circle(radius, residual, unit));
                    lines.push(t::arc(sweep_degrees, arc_length, unit));
                }
                _ => lines.push(t::CURVE.to_owned()),
            }
            lines.push(t::edge_chord(e.chord, e.closed, unit));
            lines
        }
        Figures::Face(f) => {
            let mut lines = vec![t::face_area(f.area, unit), t::perimeter(f.perimeter, unit)];
            match f.shape {
                FaceShape::Plane {
                    flatness_degrees, ..
                } => lines.push(t::flat(flatness_degrees)),
                FaceShape::Cylinder {
                    radius,
                    residual,
                    length,
                    sweep_degrees,
                    ..
                } => {
                    lines.push(t::circle(radius, residual, unit));
                    lines.push(t::cylinder(length, sweep_degrees, unit));
                }
                _ => lines.push(t::UNFITTED_FACE.to_owned()),
            }
            match f.source {
                FaceSource::File => {}
                FaceSource::Grown => lines.push(t::GROWN_FACE.to_owned()),
                _ => lines.push(t::WHOLE_PART.to_owned()),
            }
            lines
        }
    }
}

fn pick_lines(p: &Picked, unit: &Lengths<'_>, r: &Readout) -> Vec<String> {
    match &p.figures {
        Ok(f) => figure_lines(f, unit, r),
        Err(why) => vec![t::refused(why)],
    }
}

fn distance_lines(
    d: &Result<EntityDistance, String>,
    unit: &Lengths<'_>,
    r: &Readout,
) -> Vec<String> {
    let d = match d {
        Ok(d) => d,
        Err(why) => return vec![t::refused(why)],
    };
    let mut lines = vec![
        t::minimum(d.minimum.length, unit),
        tr::deltas(unit, r.vector(d.minimum.delta())),
        t::maximum(d.maximum.length, unit),
    ];
    if let Some(c) = d.centres {
        lines.push(t::centres(c.length, unit));
    }
    lines
}

/// What the projection made of the smallest span, said on `plane`.
fn projected_distance_line(p: &Projected, plane: &str, unit: &Lengths<'_>) -> Option<String> {
    let (length, out) = p.minimum?;
    Some(tr::projected_distance(
        plane,
        &unit.length(length),
        &unit.length(out.abs()),
    ))
}

/// What the projection made of pick `i`'s face, if it is one.
fn projected_face_lines(
    p: &Projected,
    i: usize,
    plane: &str,
    unit: &Lengths<'_>,
    grown: bool,
) -> Vec<String> {
    let mut lines = match p.faces.get(i) {
        Some(Some(Ok(a))) if a.toward.min(a.away) > 0.0 => vec![tr::projected_area_sides(
            plane,
            &unit.area(a.toward),
            &unit.area(a.away),
        )],
        Some(Some(Ok(a))) => vec![tr::projected_area(plane, &unit.area(a.toward + a.away))],
        Some(Some(Err(why))) => vec![tr::not_projected(why)],
        _ => return Vec::new(),
    };
    if grown {
        lines.push(tr::LARGER_FACE_PROJECTED.to_owned());
    }
    lines
}

impl ModelView {
    /// The line from the view's target to its eye, the screen's normal.
    fn toward_eye(&self) -> Option<[f64; 3]> {
        self.view_camera(1.0).ok().map(|c| sub(c.eye, c.target))
    }

    /// The readout's rows, and what a change made.
    fn readout_controls(&mut self, ui: &mut Ui) {
        let units = self.model.drawn.units;
        let stated = units.is_some_and(|u| u.from_cad_file);
        let can_set = self.measure.select.frame_of_picks().is_some();
        let s = &mut self.measure.select;
        let Some(change) = readout::controls(ui, &mut s.readout, stated, can_set) else {
            return;
        };
        if change == Change::SetFrame {
            s.readout.frame = s.frame_of_picks();
        }
        let r = &s.readout;
        let l = r.lengths(units);
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| match change {
            Change::Unit | Change::Second => format!(
                "model-select-unit unit={} second={} factor={:.6} shown={}",
                r.unit.map_or("file", |u| u.symbol()),
                r.second.map_or("none", |u| u.symbol()),
                l.factor,
                l.length(1.0).replace(' ', "_")
            ),
            Change::Decimals => format!("model-select-decimals n={}", l.decimals),
            Change::Projection => format!("model-select-projection plane={}", r.plane_word()),
            Change::SetFrame | Change::ModelFrame => match &r.frame {
                Some(f) => {
                    let [x, y, z] = f.axes();
                    format!(
                        "model-select-frame set=1 origin={} x={} y={} z={}",
                        triple(f.origin()),
                        triple(x),
                        triple(y),
                        triple(z)
                    )
                }
                None => "model-select-frame set=0".to_owned(),
            },
        });
    }

    /// Remake the projection for this frame's view.
    fn select_project(&mut self) {
        let toward = self.toward_eye();
        self.measure
            .select
            .project(&self.model.drawn.meshes, toward);
    }

    /// A click on the model while this measure is chosen.
    pub(super) fn select_pick(&mut self, hit: &Hit) {
        self.measure.select.pick(&self.model.drawn.meshes, hit);
    }

    /// The measure's button, after the surface measure.
    pub(super) fn select_button(&mut self, ui: &mut Ui) {
        let chosen = self.measure.kind == SELECT;
        let button = ui
            .add(egui::Button::new(t::SELECT).selected(chosen))
            .on_hover_text(t::SELECT_TIP);
        crate::diag::ui_rect_visible(REGION_SELECT, button.rect, ui.clip_rect());
        if button.clicked() && !chosen {
            self.measure.kind = SELECT;
            self.measure.clear();
            self.meter.clear();
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| "model-select-chosen".to_owned());
        }
    }

    /// The tab's row while this measure is chosen: what a click picks. A
    /// change forgets the picks, which were of the other kind.
    pub(super) fn select_controls(&mut self, ui: &mut Ui) {
        let s = &mut self.measure.select;
        let mut changed = None;
        ui.horizontal_wrapped(|ui| {
            ui.label(t::PICKS);
            for (i, name) in t::ENTITIES.into_iter().enumerate() {
                let choice = ui
                    .selectable_label(s.entity == i, name)
                    .on_hover_text(t::ENTITY_TIPS[i]);
                // ui-text-exempt: trace region name, never displayed
                let region = format!("{REGION_ENTITY_PREFIX}{i}");
                crate::diag::ui_rect_visible(&region, choice.rect, ui.clip_rect());
                if choice.clicked() && s.entity != i {
                    changed = Some(i);
                }
            }
        });
        if let Some(i) = changed {
            s.entity = i;
            s.clear();
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| {
                format!("model-select-entity entity={}", entity_word(ENTITIES[i]))
            });
        }
        self.readout_controls(ui);
        self.select_project();
    }

    /// Everything the picks measured, in the Measure tab.
    pub(super) fn select_details(&self, ui: &mut Ui) {
        let s = &self.measure.select;
        if !s.holds() {
            return;
        }
        let (r, unit) = (&s.readout, s.readout.lengths(self.model.drawn.units));
        let plane = tr::plane_name(r.projection);
        if let Some(d) = &s.distance {
            let mut lines = distance_lines(d, &unit, r);
            lines.extend(projected_distance_line(&s.projected, plane, &unit));
            for (i, line) in lines.iter().enumerate() {
                if i == 0 {
                    ui.label(RichText::new(line).size(VALUE_SIZE));
                } else {
                    ui.small(line);
                }
            }
        }
        for (i, (which, p)) in [t::FIRST, t::SECOND].into_iter().zip(&s.picks).enumerate() {
            ui.label(which);
            let grown = matches!(&p.figures, Ok(Figures::Face(f)) if f.source == FaceSource::Grown);
            let projected = projected_face_lines(&s.projected, i, plane, &unit, grown);
            for line in pick_lines(p, &unit, r).into_iter().chain(projected) {
                ui.small(line);
            }
        }
        if unit.unit.is_empty() {
            ui.small(t::NO_UNIT);
        }
    }

    /// The measurement under the picture: the headline and two rows whatever
    /// it holds, so the picture keeps its size from one pick to the next.
    pub(super) fn select_readout(&mut self, ui: &mut Ui) {
        self.select_project();
        let s = &self.measure.select;
        let (r, unit) = (&s.readout, s.readout.lengths(self.model.drawn.units));
        let mut lines = match (&s.distance, s.picks.last()) {
            (Some(d), _) => distance_lines(d, &unit, r),
            (None, Some(p)) => {
                let mut l = pick_lines(p, &unit, r);
                l.insert(1, t::next(1, s.entity));
                l
            }
            (None, None) => vec![t::next(0, s.entity)],
        };
        if self.measure.said == Some(Said::Missed) {
            lines.insert(1, crate::text::model3dmeasure::MISSED.to_owned());
        }
        lines.resize(3, BLANK.to_owned());
        ui.add(egui::Label::new(RichText::new(&lines[0]).size(VALUE_SIZE)).truncate());
        row(ui, &lines[1]);
        row(ui, &lines[2]);
        let again = ui
            .add_enabled(
                s.holds(),
                egui::Button::new(crate::text::model3dmeasure::START_AGAIN),
            )
            .on_hover_text(crate::text::model3dmeasure::START_AGAIN_TOOLTIP)
            .on_disabled_hover_text(t::next(0, s.entity));
        crate::diag::ui_rect_visible(REGION_START_AGAIN, again.rect, ui.clip_rect());
        if again.clicked() {
            self.measure.clear();
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| "model-select-cleared".to_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_measure_picks_faces_and_holds_nothing() {
        let s = Selected::default();
        assert_eq!(s.entity(), MeasureEntity::Face);
        assert!(!s.holds());
        assert!(s.span().is_none());
    }

    #[test]
    fn a_corner_is_said_in_one_line() {
        let f = Figures::Corner([1.0, 0.0, 0.0]);
        let (unit, r) = (Lengths::plain("mm"), Readout::default());
        assert_eq!(
            figure_lines(&f, &unit, &r),
            vec![t::corner([1.0, 0.0, 0.0], &unit)]
        );
    }
}
