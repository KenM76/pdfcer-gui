//! Game-style touch controls over the 3D picture, read from the raw touch
//! events so that every finger counts: egui's own pointer follows only the
//! first, which cannot both steer and look, and egui's pinch counts every
//! finger, a thumb on a stick included.
//!
//! A finger put down on a stick's base holds that stick, its tilt measured
//! from where it landed so a thumb set down off-centre does not jump; on a
//! zone (a button, a gimbal tip) it holds that zone; anywhere else it drags
//! the look, and two such fingers pinch. A finger lifted soon after landing,
//! having barely moved, is a tap. The mouse is one more finger, so it can
//! work a stick too.

use egui::{Event, Pos2, Rect, TouchPhase, Ui, Vec2};

/// The most a tap may move, in points.
const TAP_SLOP: f32 = 12.0;
/// The longest a tap may last, in seconds.
const TAP_TIME: f64 = 0.35;
/// The mouse's finger id, apart from any touch's.
const MOUSE: u64 = u64::MAX;
/// How far beyond a stick's drawn base a finger still takes it, as a
/// multiple of its radius: a thumb lands roughly.
const GRAB: f32 = 1.3;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Role {
    Stick(usize),
    Look,
    Zone(usize),
}

#[derive(Debug, Clone, Copy)]
struct Contact {
    id: u64,
    role: Role,
    start: Pos2,
    pos: Pos2,
    since: f64,
    /// It wandered beyond a tap.
    moved: bool,
}

/// The fingers on the picture, kept from frame to frame.
#[derive(Debug, Default)]
pub(super) struct Sticks {
    contacts: Vec<Contact>,
    /// A touch event has been seen, so the mouse events egui makes from the
    /// first finger are ignored.
    touched: bool,
}

/// The sticks offered: their bases' centres and radii, points, in order.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Bases<'a> {
    pub centres: &'a [Pos2],
    pub radii: &'a [f32],
}

impl Bases<'_> {
    /// Stick `s`'s radius, points; 0 past the list.
    fn radius(&self, s: usize) -> f32 {
        self.radii.get(s).copied().unwrap_or(0.0)
    }
}

/// One frame's reading.
#[derive(Debug, Default)]
pub(super) struct Reading {
    /// Each offered stick's tilt, in [`Bases::centres`] order: x right, y up,
    /// each -1 to 1, zero when let go.
    pub sticks: Vec<Vec2>,
    /// Points the look was dragged by a single finger.
    pub look: Vec2,
    /// Two fingers off the sticks pinching, as a factor; 1 when not.
    pub pinch: f32,
    /// The zones a finger holds.
    pub held: Vec<usize>,
    /// Zones tapped.
    pub tapped_zones: Vec<usize>,
    /// Taps on the picture outside any zone.
    pub taps: Vec<Pos2>,
}

impl Sticks {
    /// Whether this is a touch screen, so the sticks are offered.
    pub fn touch(&self) -> bool {
        self.touched
    }

    /// Whether any finger is down.
    pub fn active(&self) -> bool {
        !self.contacts.is_empty()
    }

    /// Read this frame's fingers on `rect`. `zones` are hot areas over it, in
    /// order; `ignored` are areas whose own widgets take the finger.
    pub fn read(
        &mut self,
        ui: &Ui,
        rect: Rect,
        bases: Bases<'_>,
        zones: &[Rect],
        ignored: &[Rect],
    ) -> Reading {
        let ctx = ui.ctx();
        let layer = ui.layer_id();
        let starts_here = |p: Pos2| {
            rect.contains(p)
                && ctx.layer_id_at(p) == Some(layer)
                && !ignored.iter().any(|r| r.contains(p))
        };
        let (events, now) = ui.input(|i| (i.events.clone(), i.time));
        let mut reading = Reading {
            pinch: 1.0,
            ..Reading::default()
        };
        let spread = self.spread();
        for event in events {
            let Some((id, phase, pos)) = self.contact_of(event) else {
                continue;
            };
            match phase {
                TouchPhase::Start if starts_here(pos) => self.begin(id, pos, now, bases, zones),
                TouchPhase::Start => {}
                TouchPhase::Move => self.moved(id, pos, &mut reading),
                TouchPhase::End | TouchPhase::Cancel => {
                    self.end(id, phase == TouchPhase::End, now, &mut reading);
                }
            }
        }
        if let (Some(before), Some(after)) = (spread, self.spread()) {
            reading.pinch = after / before.max(1.0);
        }
        self.held(bases, &mut reading);
        reading
    }

    /// The distance between the two looking fingers, when exactly two are.
    fn spread(&self) -> Option<f32> {
        let mut looking = self.contacts.iter().filter(|c| c.role == Role::Look);
        match (looking.next(), looking.next(), looking.next()) {
            (Some(a), Some(b), None) => Some((a.pos - b.pos).length()),
            _ => None,
        }
    }

    /// A finger moved: a lone looking finger drags the look.
    fn moved(&mut self, id: u64, pos: Pos2, reading: &mut Reading) {
        let looking = self
            .contacts
            .iter()
            .filter(|c| c.role == Role::Look)
            .count();
        if let Some(c) = self.contacts.iter_mut().find(|c| c.id == id) {
            if c.role == Role::Look && looking == 1 {
                reading.look += pos - c.pos;
            }
            c.pos = pos;
            c.moved |= (pos - c.start).length() > TAP_SLOP;
        }
    }

    /// The finger an event is about: a touch, or the mouse until a touch has
    /// been seen (a touch screen also synthesises mouse events).
    fn contact_of(&mut self, event: Event) -> Option<(u64, TouchPhase, Pos2)> {
        Some(match event {
            Event::Touch { id, phase, pos, .. } => {
                self.touched = true;
                (id.0, phase, pos)
            }
            _ if self.touched => return None,
            Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                ..
            } => {
                let phase = if pressed {
                    TouchPhase::Start
                } else {
                    TouchPhase::End
                };
                (MOUSE, phase, pos)
            }
            Event::PointerMoved(pos) => (MOUSE, TouchPhase::Move, pos),
            Event::PointerGone => (MOUSE, TouchPhase::Cancel, Pos2::ZERO),
            _ => return None,
        })
    }

    /// A finger down: on a zone it is that zone's, on a free stick's base
    /// that stick's, and anything else looks.
    fn begin(&mut self, id: u64, pos: Pos2, now: f64, bases: Bases<'_>, zones: &[Rect]) {
        self.contacts.retain(|c| c.id != id);
        let taken = |s: usize| self.contacts.iter().any(|c| c.role == Role::Stick(s));
        let stick = bases
            .centres
            .iter()
            .enumerate()
            .filter(|&(s, c)| !taken(s) && (pos - *c).length() <= bases.radius(s) * GRAB)
            .min_by(|a, b| (pos - *a.1).length().total_cmp(&(pos - *b.1).length()))
            .map(|(s, _)| s);
        let role = match (zones.iter().position(|z| z.contains(pos)), stick) {
            (Some(z), _) => Role::Zone(z),
            (None, Some(s)) => Role::Stick(s),
            (None, None) => Role::Look,
        };
        self.contacts.push(Contact {
            id,
            role,
            start: pos,
            pos,
            since: now,
            moved: false,
        });
    }

    /// A finger up (`lifted`) or cancelled: a short, still lift is a tap.
    fn end(&mut self, id: u64, lifted: bool, now: f64, reading: &mut Reading) {
        let Some(at) = self.contacts.iter().position(|c| c.id == id) else {
            return;
        };
        let c = self.contacts.remove(at);
        let tap = lifted && !c.moved && now - c.since <= TAP_TIME;
        match (tap, c.role) {
            (true, Role::Zone(z)) => reading.tapped_zones.push(z),
            (true, Role::Look) => reading.taps.push(c.start),
            _ => {}
        }
    }

    /// The fingers still down: zones held, and the sticks' tilts.
    fn held(&self, bases: Bases<'_>, reading: &mut Reading) {
        reading.sticks = vec![Vec2::ZERO; bases.centres.len()];
        for c in &self.contacts {
            match c.role {
                Role::Zone(z) => reading.held.push(z),
                Role::Stick(s) => {
                    let tilt = (c.pos - c.start) / bases.radius(s).max(1.0);
                    let tilt = if tilt.length() > 1.0 {
                        tilt.normalized()
                    } else {
                        tilt
                    };
                    if let Some(slot) = reading.sticks.get_mut(s) {
                        *slot = Vec2::new(tilt.x, -tilt.y);
                    }
                }
                Role::Look => {}
            }
        }
    }
}
