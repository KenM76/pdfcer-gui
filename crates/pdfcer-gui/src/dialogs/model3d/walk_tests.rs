use pdfcer_3d::{TriangleMesh, pick};

use super::*;

/// A 20 × 20 floor at z = 0 and a wall across x = 5, z up.
fn room() -> Vec<TriangleMesh> {
    let mut floor = TriangleMesh::default();
    floor.positions = vec![
        [-10.0, -10.0, 0.0],
        [10.0, -10.0, 0.0],
        [10.0, 10.0, 0.0],
        [-10.0, 10.0, 0.0],
    ];
    floor.triangles = vec![[0, 1, 2], [0, 2, 3]];
    let mut wall = TriangleMesh::default();
    wall.positions = vec![
        [5.0, -10.0, 0.0],
        [5.0, 10.0, 0.0],
        [5.0, 10.0, 10.0],
        [5.0, -10.0, 10.0],
    ];
    wall.triangles = vec![[0, 1, 2], [0, 2, 3]];
    vec![floor, wall]
}

/// Looking along +x, z up, radius 10: eye height 0.8.
fn walker(eye: [f64; 3]) -> Walk {
    let axes = Axes {
        front: [1.0, 0.0, 0.0],
        up: [0.0, 0.0, 1.0],
    };
    Walk {
        eye,
        yaw: 0.0,
        pitch: 0.0,
        axes,
        scale: 10.0,
        fall: 0.0,
    }
}

const SOLID: Rules = Rules {
    solid: true,
    level: true,
    speed: 1.0,
};

fn ahead(seconds: usize, walk: &mut Walk, rules: Rules, meshes: &[TriangleMesh]) {
    let intent = Intent {
        forward: 1.0,
        ..Intent::default()
    };
    for _ in 0..seconds * 20 {
        walk.step(intent, 0.05, rules, &mut |ray| pick(meshes, ray));
    }
}

#[test]
fn a_solid_walker_stops_at_the_wall_and_a_ghost_goes_through() {
    let meshes = room();
    let mut solid = walker([0.0, 0.0, 0.8]);
    ahead(10, &mut solid, SOLID, &meshes);
    assert!(solid.eye[0] < 5.0 && solid.eye[0] > 4.0, "{:?}", solid.eye);
    let mut ghost = walker([0.0, 0.0, 0.8]);
    let rules = Rules {
        solid: false,
        ..SOLID
    };
    ahead(10, &mut ghost, rules, &meshes);
    assert!(ghost.eye[0] > 10.0, "{:?}", ghost.eye);
}

#[test]
fn a_solid_walker_falls_to_eye_height_and_stays_there() {
    let meshes = room();
    let mut walk = walker([0.0, 0.0, 4.0]);
    for _ in 0..100 {
        walk.step(Intent::default(), 0.05, SOLID, &mut |ray| {
            pick(&meshes, ray)
        });
    }
    assert!((walk.eye[2] - 0.8).abs() < 1e-9, "{:?}", walk.eye);
    assert!(!walk.falling());
}

#[test]
fn a_level_walker_keeps_its_height_looking_down() {
    let meshes = room();
    let rules = Rules {
        solid: false,
        level: true,
        speed: 1.0,
    };
    let mut walk = walker([-5.0, 0.0, 3.0]);
    walk.turn(0.0, 0.5);
    ahead(2, &mut walk, rules, &meshes);
    assert!((walk.eye[2] - 3.0).abs() < 1e-9, "{:?}", walk.eye);
    let mut flying = walker([-5.0, 0.0, 3.0]);
    flying.turn(0.0, 0.5);
    ahead(
        2,
        &mut flying,
        Rules {
            level: false,
            ..rules
        },
        &meshes,
    );
    assert!(flying.eye[2] < 2.0, "{:?}", flying.eye);
}

#[test]
fn turning_round_the_axes_and_facing_agree() {
    let mut walk = walker([0.0; 3]);
    walk.face([0.0, 1.0, 0.0]);
    let d = walk.direction();
    assert!((d[1] - 1.0).abs() < 1e-9, "{d:?}");
    walk.turn(0.0, 10.0);
    assert!(walk.pitch <= PITCH_LIMIT);
}
