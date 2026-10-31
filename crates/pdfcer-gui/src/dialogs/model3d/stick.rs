//! Game-style touch controls over the 3D picture, read from the raw touch
//! events so that every finger counts: egui's own pointer follows only the
//! first, which cannot both steer and look.
//!
//! A finger put down on a zone (a rise or sink button, a gimbal tip) holds
//! that zone; on the picture's left half it is a floating move stick
//! centred where it landed; anywhere else it drags the look. A finger lifted
//! soon after landing, having barely moved, is a tap. Without a touch
//! screen the mouse is one finger with no stick, so a drag anywhere looks.

use egui::{Event, Pos2, Rect, TouchPhase, Ui, Vec2};

/// The stick's full tilt, in points from where the finger landed.
pub(super) const STICK_RADIUS: f32 = 56.0;
/// The most a tap may move, in points.
const TAP_SLOP: f32 = 12.0;
/// The longest a tap may last, in seconds.
const TAP_TIME: f64 = 0.35;
/// The mouse's finger id, apart from any touch's.
const MOUSE: u64 = u64::MAX;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Role {
    Move,
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

/// One frame's reading.
#[derive(Debug, Default)]
pub(super) struct Reading {
    /// The move stick's tilt: x right, y forward, each -1 to 1.
    pub stick: Option<(Pos2, Vec2)>,
    /// Points the look was dragged.
    pub look: Vec2,
    /// The zones a finger holds.
    pub held: Vec<usize>,
    /// Zones tapped.
    pub tapped_zones: Vec<usize>,
    /// Taps on the picture outside any zone.
    pub taps: Vec<Pos2>,
}

impl Sticks {
    /// Whether this is a touch screen, so the stick and the touch buttons
    /// are offered.
    pub fn touch(&self) -> bool {
        self.touched
    }

    /// Whether any finger is down.
    pub fn active(&self) -> bool {
        !self.contacts.is_empty()
    }

    /// Read this frame's fingers on `rect`. `zones` are hot areas over it, in
    /// order; `ignored` are areas whose own widgets take the finger.
    pub fn read(&mut self, ui: &Ui, rect: Rect, zones: &[Rect], ignored: &[Rect]) -> Reading {
        let ctx = ui.ctx();
        let layer = ui.layer_id();
        let starts_here = |p: Pos2| {
            rect.contains(p)
                && ctx.layer_id_at(p) == Some(layer)
                && !ignored.iter().any(|r| r.contains(p))
        };
        let (events, now) = ui.input(|i| (i.events.clone(), i.time));
        let mut reading = Reading::default();
        for event in events {
            let Some((id, phase, pos)) = self.contact_of(event) else {
                continue;
            };
            match phase {
                TouchPhase::Start if starts_here(pos) => self.begin(id, pos, now, rect, zones),
                TouchPhase::Start => {}
                TouchPhase::Move => {
                    if let Some(c) = self.contacts.iter_mut().find(|c| c.id == id) {
                        if c.role == Role::Look {
                            reading.look += pos - c.pos;
                        }
                        c.pos = pos;
                        c.moved |= (pos - c.start).length() > TAP_SLOP;
                    }
                }
                TouchPhase::End | TouchPhase::Cancel => {
                    self.end(id, phase == TouchPhase::End, now, &mut reading);
                }
            }
        }
        self.held(&mut reading);
        reading
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

    /// A finger down: on a zone it is that zone's; elsewhere a touch on the
    /// left half moves and anything else looks.
    fn begin(&mut self, id: u64, pos: Pos2, now: f64, rect: Rect, zones: &[Rect]) {
        self.contacts.retain(|c| c.id != id);
        let role = match zones.iter().position(|z| z.contains(pos)) {
            Some(z) => Role::Zone(z),
            None if self.touched && pos.x < rect.center().x => Role::Move,
            None => Role::Look,
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
            (true, _) => reading.taps.push(c.start),
            _ => {}
        }
    }

    /// The fingers still down: zones held, and the stick's tilt.
    fn held(&self, reading: &mut Reading) {
        for c in &self.contacts {
            match c.role {
                Role::Zone(z) => reading.held.push(z),
                Role::Move => {
                    let tilt = (c.pos - c.start) / STICK_RADIUS;
                    let tilt = if tilt.length() > 1.0 {
                        tilt.normalized()
                    } else {
                        tilt
                    };
                    reading.stick = Some((c.start, Vec2::new(tilt.x, -tilt.y)));
                }
                Role::Look => {}
            }
        }
    }
}
