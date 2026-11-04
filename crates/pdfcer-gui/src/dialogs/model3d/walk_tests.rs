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
        look_yaw: 0.0,
        pitch: 0.0,
        zoom: 1.0,
        axes,
        height: None,
        scale: 10.0,
        fall: 0.0,
    }
}

const SOLID: Rules = Rules {
    solid: true,
    level: true,
    speed: 1.0,
    stairs: false,
};

/// [`room`] with a box across x = 2..3 of height `height`.
fn room_with_box(height: f64) -> Vec<TriangleMesh> {
    let mut meshes = room();
    let mut block = TriangleMesh::default();
    let (a, b) = (2.0, 3.0);
    block.positions = vec![
        [a, -10.0, 0.0],
        [a, 10.0, 0.0],
        [a, 10.0, height],
        [a, -10.0, height],
        [b, -10.0, height],
        [b, 10.0, height],
        [b, 10.0, 0.0],
        [b, -10.0, 0.0],
    ];
    block.triangles = vec![
        [0, 1, 2],
        [0, 2, 3],
        [3, 2, 5],
        [3, 5, 4],
        [4, 5, 6],
        [4, 6, 7],
    ];
    meshes.push(block);
    meshes
}

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
        stairs: false,
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

#[test]
fn stair_climbing_climbs_under_a_third_of_the_eye_height_and_off_it_bumps() {
    // Eye height 0.8: a 0.2 box is under a third of it, a 0.4 box over.
    let low = room_with_box(0.2);
    let climbing = Rules {
        stairs: true,
        ..SOLID
    };
    let mut over = walker([0.0, 0.0, 0.8]);
    ahead(3, &mut over, climbing, &low);
    assert!(over.eye[0] > 3.0, "{:?}", over.eye);
    let mut bumped = walker([0.0, 0.0, 0.8]);
    ahead(3, &mut bumped, SOLID, &low);
    assert!(bumped.eye[0] < 2.0, "{:?}", bumped.eye);
    let high = room_with_box(0.4);
    let mut stopped = walker([0.0, 0.0, 0.8]);
    ahead(3, &mut stopped, climbing, &high);
    assert!(stopped.eye[0] < 2.0, "{:?}", stopped.eye);
}

#[test]
fn a_solid_walker_cannot_rise_over_a_box_and_a_ghost_rises() {
    let high = room_with_box(0.4);
    let lifting = Intent {
        forward: 1.0,
        rise: 1.0,
        ..Intent::default()
    };
    let mut solid = walker([0.0, 0.0, 0.8]);
    for _ in 0..60 {
        solid.step(lifting, 0.05, SOLID, &mut |ray| pick(&high, ray));
    }
    assert!(solid.eye[0] < 2.0, "{:?}", solid.eye);
    assert!((solid.eye[2] - 0.8).abs() < 1e-9, "{:?}", solid.eye);
    let ghost = Rules {
        solid: false,
        ..SOLID
    };
    let mut flying = walker([0.0, 0.0, 0.8]);
    flying.step(lifting, 0.05, ghost, &mut |ray| pick(&high, ray));
    assert!(flying.eye[2] > 0.8, "{:?}", flying.eye);
}

/// [`room`] with a slab across x = 2..3 from `z0` up to `z1`, standing on
/// nothing: a table top, passing between the wall rays.
fn room_with_slab(z0: f64, z1: f64) -> Vec<TriangleMesh> {
    let mut meshes = room();
    let mut slab = TriangleMesh::default();
    slab.positions = vec![
        [2.0, -10.0, z0],
        [2.0, 10.0, z0],
        [2.0, 10.0, z1],
        [2.0, -10.0, z1],
        [3.0, -10.0, z1],
        [3.0, 10.0, z1],
        [3.0, 10.0, z0],
        [3.0, -10.0, z0],
    ];
    slab.triangles = vec![
        [0, 1, 2],
        [0, 2, 3],
        [3, 2, 5],
        [3, 5, 4],
        [4, 5, 6],
        [4, 6, 7],
        [7, 6, 1],
        [7, 1, 0],
    ];
    meshes.push(slab);
    meshes
}

#[test]
fn a_solid_walker_never_climbs_onto_a_table_it_walks_under() {
    // Eye height 0.8: both wall rays miss a slab from 0.3 to 0.4.
    let table = room_with_slab(0.3, 0.4);
    for rules in [
        SOLID,
        Rules {
            stairs: true,
            ..SOLID
        },
    ] {
        let mut walk = walker([0.0, 0.0, 0.8]);
        ahead(3, &mut walk, rules, &table);
        assert!((walk.eye[2] - 0.8).abs() < 1e-9, "{rules:?} {:?}", walk.eye); // ui-text-exempt: test failure message
        assert!(walk.eye[0] < 2.0, "{rules:?} {:?}", walk.eye); // ui-text-exempt: test failure message
    }
}

#[test]
fn the_camera_turns_apart_from_the_heading_and_centres_back() {
    let meshes = room();
    let mut walk = walker([-5.0, 0.0, 0.8]);
    walk.look(std::f64::consts::FRAC_PI_2, 0.3);
    ahead(1, &mut walk, SOLID, &meshes);
    assert!(
        walk.eye[0] > -5.0 && walk.eye[1].abs() < 1e-9,
        "{:?}",
        walk.eye
    );
    assert!(walk.direction()[1].abs() > 0.9, "{:?}", walk.direction());
    walk.centre_look();
    walk.level();
    let d = walk.direction();
    assert!((d[0] - 1.0).abs() < 1e-9, "{d:?}");
}

#[test]
fn zoom_narrows_the_view_within_its_range() {
    let mut walk = walker([0.0; 3]);
    walk.zoom_by(2.0);
    let Projection::Perspective { fov_y } = walk.camera(1.0).projection else {
        unreachable!();
    };
    assert!((fov_y - FOV / 2.0).abs() < 1e-9, "{fov_y}");
    walk.zoom_by(1000.0);
    assert!((walk.zoom - ZOOM.1).abs() < 1e-9);
    walk.zoom_by(1e-6);
    let Projection::Perspective { fov_y } = walk.camera(1.0).projection else {
        unreachable!();
    };
    assert!(fov_y <= 70.0 + 1e-9, "zoomed out to {fov_y} degrees"); // ui-text-exempt: test failure message
}

#[test]
fn a_wide_window_keeps_a_16_9_sideways_field() {
    let walk = walker([0.0; 3]);
    let across = |aspect: f64| {
        let Projection::Perspective { fov_y } = walk.camera(aspect).projection else {
            unreachable!();
        };
        2.0 * ((fov_y.to_radians() / 2.0).tan() * aspect)
            .atan()
            .to_degrees()
    };
    let screen = across(WIDEST);
    assert!((across(3.0) - screen).abs() < 1e-9, "{}", across(3.0));
    assert!((across(5.0) - screen).abs() < 1e-9, "{}", across(5.0));
    let Projection::Perspective { fov_y } = walk.camera(4.0 / 3.0).projection else {
        unreachable!();
    };
    assert!(
        (fov_y - FOV).abs() < 1e-9,
        "a narrower window keeps the vertical field" // ui-text-exempt: test assertion message
    );
}
