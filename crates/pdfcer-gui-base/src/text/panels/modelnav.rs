//! # `text::panels::modelnav` — the 3D viewer's navigation
//!
//! Every string `pdfcer_gui::dialogs::model3d`'s walking, gimbal and full
//! screen controls show.

/// The gimbal's axis labels, X, Y, Z.
#[must_use]
pub fn axis_letters() -> [&'static str; 3] {
    ["X", "Y", "Z"]
}

/// The switch into walking.
#[must_use]
pub fn walk() -> &'static str {
    "Walk"
}

/// Its tip.
#[must_use]
pub fn walk_tooltip() -> &'static str {
    "Walk through the model in the first person."
}

/// The switch back to turning the model.
#[must_use]
pub fn orbit() -> &'static str {
    "Orbit"
}

/// Its tip.
#[must_use]
pub fn orbit_tooltip() -> &'static str {
    "Turn the model about its centre again."
}

/// Ghost mode.
#[must_use]
pub fn ghost() -> &'static str {
    "Ghost"
}

/// Its tip.
#[must_use]
pub fn ghost_tooltip() -> &'static str {
    "Walk through walls and floors. Off, walls stop you, you stand on floors and climb steps."
}

/// Level walking.
#[must_use]
pub fn level() -> &'static str {
    "Level"
}

/// Its tip.
#[must_use]
pub fn level_tooltip() -> &'static str {
    "Keep moving level, wherever you look, so you never drift up or down. Turning it on levels the view."
}

/// Stair climbing.
#[must_use]
pub fn stairs() -> &'static str {
    "Stairs"
}

/// Its tip.
#[must_use]
pub fn stairs_tooltip() -> &'static str {
    "Climb over anything lower than a third of your eye height. Off, everything you meet stops you."
}

/// Show the gimbal.
#[must_use]
pub fn gimbal() -> &'static str {
    "Gimbal"
}

/// Its tip.
#[must_use]
pub fn gimbal_tooltip() -> &'static str {
    "Show the axes in the picture's corner. Tap an axis end to view from that side, or to face that way when walking; tap the centre to fit, or to level the view; tap it twice to save a picture of the scene."
}

/// The switch that shows or hides the navigation controls.
#[must_use]
pub fn controls() -> &'static str {
    "Show navigation controls"
}

/// Its tip.
#[must_use]
pub fn controls_tooltip() -> &'static str {
    "Draw the gimbal, the drone sticks and the full-screen buttons over the picture. A single tap on the picture also shows or hides them; while they are hidden, the camera button at the bottom saves a picture of the view."
}

/// The camera button's tip while the controls are hidden.
#[must_use]
pub fn shot_tooltip() -> &'static str {
    "Save a picture of the view, without the buttons over it"
}

/// The walking speed.
#[must_use]
pub fn speed() -> &'static str {
    "Speed"
}

/// Its tip.
#[must_use]
pub fn speed_tooltip() -> &'static str {
    "How fast walking moves, as a multiple of the usual pace."
}

/// Fill the screen, where the viewer is drawn inside the application.
#[must_use]
pub fn immersive() -> &'static str {
    "Full screen"
}

/// Its tip.
#[must_use]
pub fn immersive_tooltip() -> &'static str {
    "Fill the screen with the model, with game-style controls."
}

/// Leave it.
#[must_use]
pub fn immersive_leave() -> &'static str {
    "Leave full screen"
}

/// Rise.
#[must_use]
pub fn rise() -> &'static str {
    "Rise"
}

/// Sink.
#[must_use]
pub fn sink() -> &'static str {
    "Sink"
}

/// Fit the model.
#[must_use]
pub fn home() -> &'static str {
    "Fit the model in view"
}

/// How to walk with keys and a mouse.
#[must_use]
pub fn walk_hint() -> &'static str {
    "W and S rise and sink, A and D turn; the arrows or keypad fly and strafe; I J K L turn the camera, U centres it and O levels it; Shift runs; drag to look; the wheel steps, Ctrl with the wheel zooms; double-click to go there."
}

/// How to walk with fingers.
#[must_use]
pub fn walk_hint_touch() -> &'static str {
    "As a drone's remote: the left stick rises, sinks and turns, the right flies and strafes, the middle turns the camera; pinch to zoom; double-tap to go there."
}

/// How the orbit keys work, beside the existing pointer hint.
#[must_use]
pub fn orbit_keys_hint() -> &'static str {
    "Arrows turn, + and - zoom; double-click centres on a point."
}

/// The pointer hint `orbiting` with the orbit keys after it.
#[must_use]
pub fn orbit_hint_with_keys(orbiting: &str) -> String {
    format!("{orbiting} {}", orbit_keys_hint())
}

/// Walking at this speed, for the speed slider's value.
#[must_use]
pub fn speed_value(speed: f64) -> String {
    format!("{speed:.2}×")
}

/// Walls and floors make walking slow on this model: its collision time
/// per step and its triangle count.
#[must_use]
pub fn slow_collision(ms: f64, triangles: usize) -> String {
    format!(
        "Walls and floors take {ms:.0} ms a step on this model's {triangles} triangles, so walking is jerky; Ghost mode moves freely."
    )
}

/// The options glyph's tip.
#[must_use]
pub fn options_tooltip() -> &'static str {
    "View options: views, projection, axes, mesh fit, navigation and full screen."
}

/// The walker's eye height, set with ghost mode off.
#[must_use]
pub fn eye_height() -> &'static str {
    "Eye height"
}

/// Its tip.
#[must_use]
pub fn eye_height_tooltip() -> &'static str {
    "How high above the floor you walk. Steps you climb and the floor you land on follow it."
}

/// The feet field's unit.
#[must_use]
pub fn feet_suffix() -> &'static str {
    " ft"
}

/// The inches field's unit.
#[must_use]
pub fn inches_suffix() -> &'static str {
    " in"
}

/// The model's file names no unit of its own.
#[must_use]
pub fn unit_assumed() -> &'static str {
    "unit assumed"
}

/// Its tip.
#[must_use]
pub fn unit_assumed_tooltip() -> &'static str {
    "The file does not say the model's real unit, so heights use the exporter's default and may be off by its scale."
}

/// Go back to the automatic eye height.
#[must_use]
pub fn eye_auto() -> &'static str {
    "Auto"
}

/// Its tip.
#[must_use]
pub fn eye_auto_tooltip() -> &'static str {
    "Walk at a height set from the model's size again."
}
