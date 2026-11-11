//! The sun disc: the sky seen from straight above, with a ball at the sun.
//! The ball's angle round the centre is the bearing; its distance in from the
//! rim is the height, linear in degrees — the rim 0°, the centre 90°. The disc
//! is turned so its bottom edge is the bearing the camera looks from, so the
//! ball shows where the sun stands relative to the picture as the model turns.
//! It is a control, not model content: the picture carries no mark of it.

use egui::{Color32, Pos2, Sense, Stroke, Ui, Vec2};

/// The disc's side, in points, at its largest.
const SIDE: f32 = 128.0;
/// The ball's radius, in points.
const BALL: f32 = 7.0;
/// The ball's fill: the sun's own yellow, the same in every theme.
const SUN: Color32 = Color32::from_rgb(0xF5, 0xC0, 0x2C); // NOT A THEME COLOUR: the sun's conventional yellow
/// Below this share of the radius the ball is at the centre, where every
/// bearing is the same place; a press there keeps the bearing it had.
const CENTRE: f32 = 0.02;

/// Where the ball sits, as a share of the radius, screen axes (y down), for
/// a sun at `bearing` and `height` degrees on a disc whose bottom is
/// `facing`. Bearing runs counter-clockwise seen from above, so on the disc
/// it runs from the bottom towards the right.
#[must_use]
pub(super) fn ball_offset(bearing: f64, height: f64, facing: f64) -> Vec2 {
    let phi = (bearing - facing).to_radians();
    let r = (1.0 - height.clamp(0.0, 90.0) / 90.0) as f32;
    Vec2::new(r * phi.sin() as f32, r * phi.cos() as f32)
}

/// The whole-degree bearing and height for a press at `offset` (a share of
/// the radius, screen axes) on a disc whose bottom is `facing`; the bearing
/// is `None` at the centre. Past the rim is the rim.
#[must_use]
pub(super) fn aim_at(offset: Vec2, facing: f64) -> (Option<f64>, f64) {
    let r = offset.length().min(1.0);
    let height = (90.0 * (1.0 - f64::from(r))).round();
    if r < CENTRE {
        return (None, height);
    }
    let phi = f64::from(offset.x).atan2(f64::from(offset.y)).to_degrees();
    let bearing = (phi + facing).rem_euclid(360.0).round() % 360.0;
    (Some(bearing), height)
}

/// Draw the disc under `region` and move the sun to a press or drag on it.
/// `facing` is the bearing at the disc's bottom edge. True when it moved.
pub(super) fn sun_disc(
    ui: &mut Ui,
    region: &str,
    tip: &str,
    (bearing, height): (&mut f64, &mut f64),
    facing: f64,
) -> bool {
    let side = SIDE.min(ui.available_width()).max(4.0 * BALL);
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click_and_drag());
    crate::diag::ui_rect_visible(region, rect, ui.clip_rect());
    let centre = rect.center();
    let radius = side / 2.0 - BALL - 1.0;
    let mut moved = false;
    if (response.is_pointer_button_down_on() || response.dragged() || response.clicked())
        && let Some(at) = response.interact_pointer_pos()
    {
        let (b, h) = aim_at((at - centre) / radius, facing);
        if let Some(b) = b {
            moved |= b != *bearing;
            *bearing = b;
        }
        moved |= h != *height;
        *height = h;
    }
    paint(
        ui,
        centre,
        radius,
        ball_offset(*bearing, *height, facing),
        facing,
    );
    response.on_hover_text(tip);
    moved
}

/// The rim, the 45° ring, the model's front tick and the ball.
fn paint(ui: &Ui, centre: Pos2, radius: f32, ball: Vec2, facing: f64) {
    let visuals = ui.visuals();
    let painter = ui.painter();
    let rim = visuals.widgets.noninteractive.fg_stroke;
    painter.circle(
        centre,
        radius,
        visuals.extreme_bg_color,
        Stroke::new(1.0, rim.color),
    );
    painter.circle_stroke(
        centre,
        radius / 2.0,
        Stroke::new(1.0, visuals.weak_text_color()),
    );
    let front = ball_offset(0.0, 0.0, facing);
    painter.line_segment(
        [centre + front * radius, centre + front * (radius - 8.0)],
        Stroke::new(2.0, rim.color),
    );
    let at = centre + ball * radius;
    painter.line_segment([centre, at], Stroke::new(1.0, visuals.weak_text_color()));
    painter.circle(at, BALL, SUN, Stroke::new(1.0, rim.color));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: Vec2, b: Vec2) -> bool {
        (a - b).length() < 1e-5
    }

    #[test]
    fn the_rim_is_the_horizon_and_the_centre_overhead() {
        assert!(near(ball_offset(0.0, 0.0, 0.0), Vec2::new(0.0, 1.0)));
        assert!(near(ball_offset(123.0, 90.0, 40.0), Vec2::ZERO));
        assert!((ball_offset(0.0, 45.0, 0.0).length() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn bearing_runs_from_the_bottom_towards_the_right() {
        assert!(near(ball_offset(90.0, 0.0, 0.0), Vec2::new(1.0, 0.0)));
        assert!(near(ball_offset(180.0, 0.0, 0.0), Vec2::new(0.0, -1.0)));
    }

    #[test]
    fn the_disc_turns_with_the_view() {
        // Looking from the model's right, its front is on the disc's left.
        assert!(near(ball_offset(0.0, 0.0, 90.0), Vec2::new(-1.0, 0.0)));
    }

    #[test]
    fn a_press_reads_back_the_sun_it_shows() {
        for facing in [0.0, 37.0, 200.0] {
            for (b, h) in [(0.0, 0.0), (45.0, 45.0), (300.0, 10.0), (90.0, 80.0)] {
                let (got_b, got_h) = aim_at(ball_offset(b, h, facing), facing);
                assert_eq!((got_b, got_h), (Some(b), h), "facing {facing}");
            }
        }
    }

    #[test]
    fn past_the_rim_is_the_horizon_and_the_centre_keeps_the_bearing() {
        assert_eq!(aim_at(Vec2::new(0.0, 3.0), 0.0), (Some(0.0), 0.0));
        assert_eq!(aim_at(Vec2::ZERO, 0.0), (None, 90.0));
    }
}
