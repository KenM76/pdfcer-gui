//! The figure's two shadows, each an option: a soft patch on the floor under
//! its feet, and its own shape cast on the floor by a light high to one side.
//! Both are drawn over the picture before the figure, like the figure itself.

use egui::{Color32, Mesh, Pos2};
use pdfcer_3d::Camera;
use pdfcer_3d::vec3::{add, dot, normalize, scale, sub, unit_cross};

/// The patch's darkness at its centre and the cast shadow's.
const PATCH: u8 = 110;
const CAST: u8 = 80;
/// Rings of the patch, darkest in, and corners round each.
const RINGS: usize = 4;
const AROUND: usize = 24;

/// Where the shadows lie: the floor point under the figure, the up, and the
/// figure's right and forward, unit vectors.
pub(super) struct Floor {
    pub at: [f64; 3],
    pub up: [f64; 3],
    pub right: [f64; 3],
    pub forward: [f64; 3],
}

fn place(camera: &Camera, origin: Pos2, size: [u32; 2], p: [f64; 3]) -> Option<Pos2> {
    let [x, y] = camera.pixel_of(p, size[0], size[1])?;
    #[allow(clippy::cast_possible_truncation)]
    Some(origin + egui::vec2(x as f32, y as f32))
}

/// A soft ellipse `half` model units across and `half * 0.7` front to back,
/// fading out to its rim, into `mesh`.
pub(super) fn patch(
    mesh: &mut Mesh,
    floor: &Floor,
    half: f64,
    camera: &Camera,
    origin: Pos2,
    size: [u32; 2],
) {
    #[allow(clippy::cast_precision_loss)]
    let ring = |r: usize| -> Option<Vec<Pos2>> {
        let k = r as f64 / RINGS as f64;
        (0..AROUND)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / AROUND as f64;
                let p = add(
                    add(floor.at, scale(floor.right, half * k * a.cos())),
                    scale(floor.forward, 0.7 * half * k * a.sin()),
                );
                place(camera, origin, size, p)
            })
            .collect()
    };
    let Some(centre) = place(camera, origin, size, floor.at) else {
        return;
    };
    let mut inner = vec![centre; AROUND];
    for r in 1..=RINGS {
        let Some(outer) = ring(r) else { return };
        #[allow(clippy::cast_possible_truncation)]
        let dark = |r: usize| {
            let alpha = (f64::from(PATCH) * (1.0 - r as f64 / RINGS as f64)) as u8;
            Color32::from_black_alpha(alpha) // NOT A THEME COLOUR: a shadow in the 3D scene
        };
        for i in 0..AROUND {
            let j = (i + 1) % AROUND;
            let first = u32::try_from(mesh.vertices.len()).unwrap_or(u32::MAX);
            mesh.colored_vertex(inner[i], dark(r - 1));
            mesh.colored_vertex(inner[j], dark(r - 1));
            mesh.colored_vertex(outer[j], dark(r));
            mesh.colored_vertex(outer[i], dark(r));
            mesh.add_triangle(first, first + 1, first + 2);
            mesh.add_triangle(first, first + 2, first + 3);
        }
        inner = outer;
    }
}

/// The light's direction, travelling from it: down, and fixed to the model
/// rather than the figure, so the shadow keeps its side as the figure turns.
pub(super) fn light(up: [f64; 3], level: [f64; 3]) -> [f64; 3] {
    normalize(add(scale(up, -1.0), scale(level, 0.6))).unwrap_or(scale(up, -1.0))
}

/// `p` slid along `light` onto the floor.
fn onto(floor: &Floor, light: [f64; 3], p: [f64; 3]) -> [f64; 3] {
    let down = dot(light, floor.up);
    if down > -1e-6 {
        return p;
    }
    add(p, scale(light, -dot(sub(p, floor.at), floor.up) / down))
}

/// The faces lit by `light`, each laid on the floor along it, into `mesh`.
pub(super) fn cast(
    mesh: &mut Mesh,
    floor: &Floor,
    light: [f64; 3],
    faces: &[([[f64; 3]; 3], Color32)],
    camera: &Camera,
    origin: Pos2,
    size: [u32; 2],
) {
    let ink = Color32::from_black_alpha(CAST); // NOT A THEME COLOUR: a shadow in the 3D scene
    for (face, _) in faces {
        let Some(n) = unit_cross(sub(face[1], face[0]), sub(face[2], face[0])) else {
            continue;
        };
        if dot(n, light) >= 0.0 {
            continue;
        }
        let placed: Option<Vec<Pos2>> = face
            .iter()
            .map(|p| place(camera, origin, size, onto(floor, light, *p)))
            .collect();
        let Some(placed) = placed else { continue };
        let first = u32::try_from(mesh.vertices.len()).unwrap_or(u32::MAX);
        for p in placed {
            mesh.colored_vertex(p, ink);
        }
        mesh.add_triangle(first, first + 1, first + 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floor() -> Floor {
        Floor {
            at: [0.0, 0.0, 0.0],
            up: [0.0, 0.0, 1.0],
            right: [1.0, 0.0, 0.0],
            forward: [0.0, 1.0, 0.0],
        }
    }

    #[test]
    fn a_point_is_laid_on_the_floor_away_from_the_light() {
        let l = light([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        let p = onto(&floor(), l, [0.0, 0.0, 1.0]);
        assert!(p[2].abs() < 1e-9 && p[0] > 0.5, "{p:?}");
        let at_rest = onto(&floor(), l, [0.3, 0.2, 0.0]);
        assert!((at_rest[0] - 0.3).abs() < 1e-9 && (at_rest[1] - 0.2).abs() < 1e-9);
    }
}
