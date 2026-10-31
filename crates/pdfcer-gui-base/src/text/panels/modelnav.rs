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

/// Show the gimbal.
#[must_use]
pub fn gimbal() -> &'static str {
    "Gimbal"
}

/// Its tip.
#[must_use]
pub fn gimbal_tooltip() -> &'static str {
    "Show the axes in the picture's corner. Tap an axis end to view from that side, or to face that way when walking; tap the centre to fit, or to level the view."
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
    "W A S D or the arrows move and turn, E and Q rise and sink, Shift runs; drag to look; the wheel steps; double-click to go there."
}

/// How to walk with fingers.
#[must_use]
pub fn walk_hint_touch() -> &'static str {
    "Left thumb moves, right thumb looks; the arrows rise and sink; double-tap to go there."
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
