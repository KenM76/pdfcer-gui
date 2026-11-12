//! The 3D viewer's Controls tab: ready layouts, the keys, the mouse, and
//! the matching settings-page rows.

/// The settings page's heading over the walking keys and mouse.
#[must_use]
pub fn title() -> &'static str {
    "Walking controls"
}

/// The ready layouts' heading.
#[must_use]
pub fn presets() -> &'static str {
    "Ready layouts"
}

/// Each ready layout's name, in `Preset::ALL` order.
#[must_use]
pub fn preset_names() -> [&'static str; 3] {
    ["Default", "Shooter", "Avatar"]
}

/// Each ready layout's description, in `Preset::ALL` order.
#[must_use]
pub fn preset_notes() -> [&'static str; 3] {
    [
        "As the drone sticks: W and S rise and sink, A and D turn, the arrows walk and strafe. A drag turns the view.",
        "As a first-person shooter: W A S D walk and strafe, the mouse looks with no button held once you click the picture, Space rises and C sinks.",
        "As a role-playing game seen from behind: W and S walk, A and D turn, Q and E strafe, the right button turns the body, the left turns the camera, both buttons walk. Turns on the view from behind.",
    ]
}

/// The name shown once a binding differs from every ready layout.
#[must_use]
pub fn custom() -> &'static str {
    "Custom"
}

/// The line naming the layout in use.
#[must_use]
pub fn in_use(name: &str) -> String {
    format!("In use: {name}")
}

/// A ready layout card's tip.
#[must_use]
pub fn preset_tooltip() -> &'static str {
    "Use this layout's keys and mouse. The swap, the mouse's sensitivity and its up and down stay as they are."
}

/// The Controls tab's note while not walking.
#[must_use]
pub fn walk_only() -> &'static str {
    "These keys and mouse buttons move you while walking."
}

/// The swap switch.
#[must_use]
pub fn swap() -> &'static str {
    "Swap the turning and strafing sides"
}

/// Its tip.
#[must_use]
pub fn swap_tooltip() -> &'static str {
    "Turning moves to the right stick and to the keys that strafed; strafing moves to the left stick and to the keys that turned."
}

/// The keys' heading.
#[must_use]
pub fn keys() -> &'static str {
    "Keys"
}

/// Each walking action's name, in `Act::ALL` order.
#[must_use]
pub fn acts() -> [&'static str; 14] {
    [
        "Walk forward",
        "Walk back",
        "Strafe left",
        "Strafe right",
        "Turn left",
        "Turn right",
        "Rise",
        "Sink",
        "Look up",
        "Look down",
        "Look left",
        "Look right",
        "Centre the look",
        "Level the look",
    ]
}

/// A key chip's tip.
#[must_use]
pub fn chip_tooltip() -> &'static str {
    "Click, then press the key to use. Escape cancels; Delete clears it."
}

/// A key chip waiting for its key.
#[must_use]
pub fn press_key() -> &'static str {
    "Press a key…"
}

/// A key chip with no key.
#[must_use]
pub fn no_key() -> &'static str {
    "—"
}

/// A key's name on its chip.
#[must_use]
pub fn key(key: egui::Key) -> &'static str {
    key.symbol_or_name()
}

/// The mouse's heading.
#[must_use]
pub fn mouse() -> &'static str {
    "Mouse"
}

/// The left button's drag.
#[must_use]
pub fn left_drag() -> &'static str {
    "Left-button drag"
}

/// The right button's drag.
#[must_use]
pub fn right_drag() -> &'static str {
    "Right-button drag"
}

/// What a drag can do, in `Drag::ALL` order.
#[must_use]
pub fn drags() -> [&'static str; 4] {
    [
        "Turns the camera while the sticks show, else the body",
        "Turns the body",
        "Turns the camera only",
        "Nothing",
    ]
}

/// Both buttons walk.
#[must_use]
pub fn both_forward() -> &'static str {
    "Both buttons walk forward"
}

/// Free look.
#[must_use]
pub fn free_look() -> &'static str {
    "The mouse looks with no button held"
}

/// Its tip.
#[must_use]
pub fn free_look_tooltip() -> &'static str {
    "While walking, click the picture to give it the mouse: moving the mouse turns the view and the pointer hides. Escape gives the mouse back."
}

/// Said under the picture while the picture holds the mouse.
#[must_use]
pub fn mouse_held() -> &'static str {
    "The mouse is turning the view. Escape gives it back."
}

/// Inverted up and down.
#[must_use]
pub fn invert_y() -> &'static str {
    "Invert the mouse's up and down"
}

/// The mouse look's rate.
#[must_use]
pub fn sensitivity() -> &'static str {
    "Mouse sensitivity"
}

/// The run modifier.
#[must_use]
pub fn run() -> &'static str {
    "Run while holding"
}

/// The run modifiers, in `Run::ALL` order.
#[must_use]
pub fn runs() -> [&'static str; 3] {
    ["Shift", "Ctrl", "Alt"]
}

/// A key slot's row on the settings page.
#[must_use]
pub fn slot_label(act: &str, slot: usize) -> String {
    format!("{act}, key {}", slot + 1)
}
