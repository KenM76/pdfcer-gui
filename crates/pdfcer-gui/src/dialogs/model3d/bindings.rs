//! What each key and mouse button does while walking, and the three ready
//! layouts: *Default* (the drone sticks' keys), *Shooter* (as Counter-Strike:
//! W A S D move and strafe, the mouse looks without a button) and *Avatar*
//! (as World of Warcraft: A and D turn, Q and E strafe, the right button
//! turns the body, the left turns the camera, both walk). Design:
//! `DESIGNS.md` § O411 / O412.
//!
//! *Swap sides* exchanges turning and strafing — for the keys and the
//! on-screen sticks alike. It is applied where stick tilts become motion,
//! never to the tilts, so a drawn knob stays under the finger that moved it.

use egui::{Key, Modifiers, Vec2};

/// A walking action a key does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Act {
    Forward,
    Back,
    StrafeLeft,
    StrafeRight,
    TurnLeft,
    TurnRight,
    Rise,
    Sink,
    LookUp,
    LookDown,
    LookLeft,
    LookRight,
    CentreYaw,
    CentrePitch,
}

impl Act {
    pub(crate) const ALL: [Self; 14] = [
        Self::Forward,
        Self::Back,
        Self::StrafeLeft,
        Self::StrafeRight,
        Self::TurnLeft,
        Self::TurnRight,
        Self::Rise,
        Self::Sink,
        Self::LookUp,
        Self::LookDown,
        Self::LookLeft,
        Self::LookRight,
        Self::CentreYaw,
        Self::CentrePitch,
    ];

    /// The action's saved-key word.
    pub(crate) fn word(self) -> &'static str {
        // ui-text-exempt: saved-key words, never displayed
        [
            "forward",
            "back",
            "strafe_left",
            "strafe_right",
            "turn_left",
            "turn_right",
            "rise",
            "sink",
            "look_up",
            "look_down",
            "look_left",
            "look_right",
            "centre_yaw",
            "centre_pitch",
        ][self as usize]
    }
}

/// Keys each action takes.
pub(crate) const SLOTS: usize = 2;

/// Every key a binding may name, in saved order: a saved binding is an
/// index into this list, 0 for none, so the list only ever grows at its end.
/// Escape is left out: it always lets go of the mouse and leaves full screen.
pub(crate) const KEYS: [Key; 62] = [
    Key::A,
    Key::B,
    Key::C,
    Key::D,
    Key::E,
    Key::F,
    Key::G,
    Key::H,
    Key::I,
    Key::J,
    Key::K,
    Key::L,
    Key::M,
    Key::N,
    Key::O,
    Key::P,
    Key::Q,
    Key::R,
    Key::S,
    Key::T,
    Key::U,
    Key::V,
    Key::W,
    Key::X,
    Key::Y,
    Key::Z,
    Key::Num0,
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
    Key::ArrowUp,
    Key::ArrowDown,
    Key::ArrowLeft,
    Key::ArrowRight,
    Key::Space,
    Key::Tab,
    Key::Enter,
    Key::Backspace,
    Key::Insert,
    Key::Delete,
    Key::Home,
    Key::End,
    Key::PageUp,
    Key::PageDown,
    Key::Comma,
    Key::Period,
    Key::Semicolon,
    Key::Quote,
    Key::Slash,
    Key::Backslash,
    Key::Minus,
    Key::Equals,
    Key::OpenBracket,
    Key::CloseBracket,
    Key::Backtick,
    Key::F1,
];

/// What a drag with one mouse button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Drag {
    /// Turns the camera while the drone sticks are shown, the body otherwise.
    Auto,
    /// Turns the body, so forward follows the view.
    Turn,
    /// Turns the camera apart from the heading.
    Look,
    Nothing,
}

impl Drag {
    pub(crate) const ALL: [Self; 4] = [Self::Auto, Self::Turn, Self::Look, Self::Nothing];
}

/// The modifier held to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Run {
    Shift,
    Ctrl,
    Alt,
}

impl Run {
    pub(crate) const ALL: [Self; 3] = [Self::Shift, Self::Ctrl, Self::Alt];

    pub(crate) fn held(self, m: Modifiers) -> bool {
        match self {
            Self::Shift => m.shift,
            Self::Ctrl => m.ctrl,
            Self::Alt => m.alt,
        }
    }
}

/// A ready layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Preset {
    Default,
    Shooter,
    Avatar,
}

impl Preset {
    pub(crate) const ALL: [Self; 3] = [Self::Default, Self::Shooter, Self::Avatar];

    /// Its word in traces.
    pub(crate) fn word(self) -> &'static str {
        // ui-text-exempt: trace words, never displayed
        ["default", "shooter", "avatar"][self as usize]
    }
}

/// Every walking binding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Bindings {
    /// Each action's keys, in [`Act::ALL`] order.
    pub keys: [[Option<Key>; SLOTS]; Act::ALL.len()],
    pub left_drag: Drag,
    pub right_drag: Drag,
    /// Both buttons held walk forward.
    pub both_forward: bool,
    /// The mouse turns the body with no button held, once a click on the
    /// picture has taken it; Escape lets it go.
    pub free_look: bool,
    /// Mouse and drag looking upward drags down.
    pub invert_y: bool,
    /// Mouse and drag looking's rate, times the usual.
    pub sensitivity: f64,
    pub run: Run,
    /// Turning and strafing exchanged, on the keys and the sticks.
    pub swap_sides: bool,
}

impl Default for Bindings {
    fn default() -> Self {
        Self::preset(Preset::Default)
    }
}

/// The sensitivity's range.
pub(crate) const SENSITIVITY: std::ops::RangeInclusive<f64> = 0.25..=4.0;

impl Bindings {
    /// `preset`'s layout, unswapped.
    pub(crate) fn preset(preset: Preset) -> Self {
        use Key as K;
        let k = |a: K, b: Option<K>| [Some(a), b];
        let looks = [k(K::I, None), k(K::K, None), k(K::J, None), k(K::L, None)];
        let centres = [k(K::U, None), k(K::O, None)];
        let (moves, mouse) = match preset {
            Preset::Default => (
                [
                    k(K::ArrowUp, Some(K::Num8)),
                    k(K::ArrowDown, Some(K::Num2)),
                    k(K::ArrowLeft, Some(K::Num4)),
                    k(K::ArrowRight, Some(K::Num6)),
                    k(K::A, None),
                    k(K::D, None),
                    k(K::W, None),
                    k(K::S, None),
                ],
                (Drag::Auto, Drag::Nothing, false, false),
            ),
            Preset::Shooter => (
                [
                    k(K::W, Some(K::ArrowUp)),
                    k(K::S, Some(K::ArrowDown)),
                    k(K::A, None),
                    k(K::D, None),
                    k(K::ArrowLeft, None),
                    k(K::ArrowRight, None),
                    k(K::Space, None),
                    k(K::C, None),
                ],
                (Drag::Turn, Drag::Nothing, false, true),
            ),
            Preset::Avatar => (
                [
                    k(K::W, Some(K::ArrowUp)),
                    k(K::S, Some(K::ArrowDown)),
                    k(K::Q, None),
                    k(K::E, None),
                    k(K::A, Some(K::ArrowLeft)),
                    k(K::D, Some(K::ArrowRight)),
                    k(K::Space, None),
                    k(K::X, None),
                ],
                (Drag::Look, Drag::Turn, true, false),
            ),
        };
        let mut keys = [[None; SLOTS]; Act::ALL.len()];
        for (slot, set) in keys
            .iter_mut()
            .zip(moves.iter().chain(&looks).chain(&centres))
        {
            *slot = *set;
        }
        Self {
            keys,
            left_drag: mouse.0,
            right_drag: mouse.1,
            both_forward: mouse.2,
            free_look: mouse.3,
            invert_y: false,
            sensitivity: 1.0,
            run: Run::Shift,
            swap_sides: false,
        }
    }

    /// The preset these bindings are, swap, sensitivity and inversion aside;
    /// `None` once edited.
    pub(crate) fn matches(&self) -> Option<Preset> {
        Preset::ALL.into_iter().find(|p| {
            let mut ready = Self::preset(*p);
            ready.swap_sides = self.swap_sides;
            ready.sensitivity = self.sensitivity;
            ready.invert_y = self.invert_y;
            ready == *self
        })
    }

    /// Take up `preset`, keeping swap, sensitivity and inversion.
    pub(crate) fn apply(&mut self, preset: Preset) {
        *self = Self {
            swap_sides: self.swap_sides,
            sensitivity: self.sensitivity,
            invert_y: self.invert_y,
            ..Self::preset(preset)
        };
    }

    /// Bind `key` to `act`'s `slot`, taking it off every other action, so a
    /// key does one thing.
    pub(crate) fn bind(&mut self, act: Act, slot: usize, key: Option<Key>) {
        if let Some(key) = key {
            for slots in &mut self.keys {
                for s in slots.iter_mut().filter(|s| **s == Some(key)) {
                    *s = None;
                }
            }
        }
        self.keys[act as usize][slot] = key;
    }

    /// The keys held this frame as stick tilts — left (x turns, y rises),
    /// right (x strafes, y walks), camera — with turning and strafing
    /// exchanged when swapped, so the knob that moves is the one that does
    /// that job.
    pub(crate) fn tilts(&self, down: impl Fn(Key) -> bool) -> [Vec2; 3] {
        let held = |act: Act| f32::from(u8::from(self.fired(act, &down)));
        let axis = |plus: Act, minus: Act| held(plus) - held(minus);
        let turn = axis(Act::TurnRight, Act::TurnLeft);
        let strafe = axis(Act::StrafeRight, Act::StrafeLeft);
        let (left_x, right_x) = if self.swap_sides {
            (strafe, turn)
        } else {
            (turn, strafe)
        };
        [
            Vec2::new(left_x, axis(Act::Rise, Act::Sink)),
            Vec2::new(right_x, axis(Act::Forward, Act::Back)),
            Vec2::new(
                axis(Act::LookRight, Act::LookLeft),
                axis(Act::LookUp, Act::LookDown),
            ),
        ]
    }

    /// Whether any of `act`'s keys passes `test` — held, or pressed.
    pub(crate) fn fired(&self, act: Act, test: impl Fn(Key) -> bool) -> bool {
        self.keys[act as usize].iter().flatten().any(|k| test(*k))
    }

    /// The left and right sticks' x, read as (turn, strafe).
    pub(crate) fn turn_strafe(&self, left_x: f32, right_x: f32) -> (f32, f32) {
        if self.swap_sides {
            (right_x, left_x)
        } else {
            (left_x, right_x)
        }
    }

    /// A mouse look of `delta` points as (yaw, pitch) radians at `rate` per
    /// point.
    pub(crate) fn look(&self, delta: Vec2, rate: f64) -> (f64, f64) {
        let rate = rate * self.sensitivity;
        let y = if self.invert_y { -delta.y } else { delta.y };
        (-f64::from(delta.x) * rate, f64::from(y) * rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_preset_is_the_drone_sticks_keys() {
        let b = Bindings::default();
        let t = b.tilts(|k| k == Key::A || k == Key::ArrowUp);
        assert_eq!(t[0], Vec2::new(-1.0, 0.0));
        assert_eq!(t[1], Vec2::new(0.0, 1.0));
        assert_eq!(b.matches(), Some(Preset::Default));
    }

    #[test]
    fn swapping_moves_strafing_to_the_left_stick() {
        let b = Bindings {
            swap_sides: true,
            ..Bindings::default()
        };
        let t = b.tilts(|k| k == Key::A);
        assert_eq!(t[0].x, 0.0);
        assert_eq!(t[1].x, -1.0);
        assert_eq!(b.turn_strafe(t[0].x, t[1].x), (-1.0, 0.0));
        assert_eq!(b.matches(), Some(Preset::Default));
    }

    #[test]
    fn a_shooter_strafes_on_a_and_d() {
        let b = Bindings::preset(Preset::Shooter);
        let t = b.tilts(|k| k == Key::D);
        assert_eq!(b.turn_strafe(t[0].x, t[1].x), (0.0, 1.0));
        assert!(b.free_look);
    }

    #[test]
    fn binding_a_key_takes_it_off_its_old_action() {
        let mut b = Bindings::preset(Preset::Avatar);
        b.bind(Act::Rise, 1, Some(Key::Q));
        assert_eq!(b.keys[Act::StrafeLeft as usize], [None, None]);
        assert_eq!(b.matches(), None);
        b.apply(Preset::Avatar);
        assert_eq!(b.matches(), Some(Preset::Avatar));
    }

    #[test]
    fn every_preset_key_is_saveable_and_none_is_escape() {
        assert!(!KEYS.contains(&Key::Escape));
        for p in Preset::ALL {
            for k in Bindings::preset(p).keys.iter().flatten().flatten() {
                assert!(KEYS.contains(k), "{k:?}");
            }
        }
    }
}
