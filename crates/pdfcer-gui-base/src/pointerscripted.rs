//! # `pointerscripted` — clicks, drags and wheel turns without the OS mouse
//!
//! `PDFCER_DIAG_POINTER=<path>` names a file a harness appends steps to, one
//! per line. Each step becomes egui pointer events injected into `RawInput`
//! through an [`egui::Plugin::input_hook`], so it passes through egui's own
//! hit-testing, click counting and drag threshold exactly as a real press
//! does, in any viewport. Only honoured when [`crate::diag::enabled`].
//!
//! ```text
//! <seq> move X Y [vp=V]
//! <seq> click X Y [btn=l|r|m] [mods=ctrl+shift+alt] [vp=V]
//! <seq> dclick X Y [mods=…] [vp=V]
//! <seq> down X Y [btn=…] [mods=…] [vp=V]      <seq> up X Y [btn=…] [mods=…] [vp=V]
//! <seq> drag X0 Y0 X1 Y1 [steps=N] [btn=…] [mods=…] [vp=V]
//! <seq> wheel X Y DY [mods=…] [vp=V]
//! <seq> gone [vp=V]
//! ```
//!
//! Points are egui logical points of the target viewport — the space
//! `ui-rect` lines are written in. `vp=` is `root` (the default) or the
//! `viewport=` token of a `ui-rect` line, copied verbatim.
//!
//! When a step's last event has been delivered it answers:
//!
//! ```text
//! diag-pointer seq=3 verb=click vp=root frames=3
//! ```
//!
//! and an unreadable line answers `diag-pointer-refused seq=… line=…`. The
//! acknowledgement says the events were **delivered**, never what they did:
//! a check reads the effect from the application's own trace.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/pointerscripted.md`.

use std::collections::VecDeque;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::Duration;

use egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput, ViewportId, vec2};

/// How often an idle app re-reads the file for new steps.
const POLL: Duration = Duration::from_millis(50);

/// Frames a drag spends moving when the step does not say.
const DEFAULT_DRAG_STEPS: usize = 6;

/// Which viewport a step is for.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Target {
    Root,
    /// The `viewport=` token of a `ui-rect` line: the id's `Debug` form.
    Named(String),
}

impl Target {
    fn matches(&self, id: ViewportId) -> bool {
        match self {
            Self::Root => id == ViewportId::ROOT,
            // The id's `Debug` form is a quoted string, and a harness's trace
            // reader hands the value back unquoted; either spelling names it.
            Self::Named(name) => format!("{id:?}").trim_matches('"') == name.trim_matches('"'), // ui-text-exempt: a viewport id compared, never displayed
        }
    }

    fn token(&self) -> &str {
        match self {
            Self::Root => "root",
            Self::Named(name) => name,
        }
    }
}

/// One parsed step, expanded into the events of each frame it occupies.
#[derive(Debug, Clone, PartialEq)]
struct Step {
    seq: u64,
    verb: String,
    target: Target,
    modifiers: Modifiers,
    frames: VecDeque<Vec<Event>>,
    /// Frames delivered so far.
    sent: usize,
}

/// Parse one line; `Err` carries nothing, the caller quotes the line.
fn parse(line: &str) -> Result<Step, ()> {
    let mut words = line.split_whitespace();
    let seq: u64 = words.next().ok_or(())?.parse().map_err(|_| ())?;
    let verb = words.next().ok_or(())?.to_owned();
    let mut numbers: Vec<f32> = Vec::new();
    let mut target = Target::Root;
    let mut modifiers = Modifiers::NONE;
    let mut button = PointerButton::Primary;
    let mut steps = DEFAULT_DRAG_STEPS;
    for word in words {
        if let Some(v) = word.strip_prefix("vp=") {
            target = if v == "root" {
                Target::Root
            } else {
                Target::Named(v.to_owned())
            };
        } else if let Some(v) = word.strip_prefix("mods=") {
            modifiers = parse_mods(v)?;
        } else if let Some(v) = word.strip_prefix("btn=") {
            button = match v {
                "l" => PointerButton::Primary,
                "r" => PointerButton::Secondary,
                "m" => PointerButton::Middle,
                _ => return Err(()),
            };
        } else if let Some(v) = word.strip_prefix("steps=") {
            steps = v.parse().map_err(|_| ())?;
        } else {
            numbers.push(word.parse().map_err(|_| ())?);
        }
    }
    let frames = expand(&verb, &numbers, button, modifiers, steps.max(1))?;
    Ok(Step {
        seq,
        verb,
        target,
        modifiers,
        frames,
        sent: 0,
    })
}

fn parse_mods(spelling: &str) -> Result<Modifiers, ()> {
    let mut m = Modifiers::NONE;
    for part in spelling.split('+') {
        match part {
            // egui-winit reports Ctrl on Windows as both flags; a reader of
            // either must see it.
            "ctrl" => {
                m.ctrl = true;
                m.command = true;
            }
            "shift" => m.shift = true,
            "alt" => m.alt = true,
            _ => return Err(()),
        }
    }
    Ok(m)
}

/// The per-frame events of a verb. A press is always preceded by a frame
/// that only moves: egui resolves a press against the previous frame's
/// widget rects, and hover must have landed first.
fn expand(
    verb: &str,
    n: &[f32],
    button: PointerButton,
    modifiers: Modifiers,
    steps: usize,
) -> Result<VecDeque<Vec<Event>>, ()> {
    let at = |i: usize| -> Result<Pos2, ()> {
        Ok(Pos2::new(*n.get(i).ok_or(())?, *n.get(i + 1).ok_or(())?))
    };
    let press = |pos: Pos2, pressed: bool| Event::PointerButton {
        pos,
        button,
        pressed,
        modifiers,
    };
    let want = |count: usize| if n.len() == count { Ok(()) } else { Err(()) };
    let frames: Vec<Vec<Event>> = match verb {
        "move" => {
            want(2)?;
            vec![vec![Event::PointerMoved(at(0)?)]]
        }
        "down" | "up" => {
            want(2)?;
            let p = at(0)?;
            vec![vec![Event::PointerMoved(p), press(p, verb == "down")]]
        }
        "click" => {
            want(2)?;
            let p = at(0)?;
            vec![
                vec![Event::PointerMoved(p)],
                vec![press(p, true)],
                vec![press(p, false)],
            ]
        }
        "dclick" => {
            want(2)?;
            let p = at(0)?;
            vec![
                vec![Event::PointerMoved(p)],
                vec![press(p, true)],
                vec![press(p, false)],
                vec![press(p, true)],
                vec![press(p, false)],
            ]
        }
        "drag" => {
            want(4)?;
            let (a, b) = (at(0)?, at(2)?);
            let mut out = vec![vec![Event::PointerMoved(a)], vec![press(a, true)]];
            for i in 1..=steps {
                #[allow(clippy::cast_precision_loss)]
                let t = i as f32 / steps as f32;
                out.push(vec![Event::PointerMoved(a.lerp(b, t))]);
            }
            out.push(vec![press(b, false)]);
            out
        }
        "wheel" => {
            want(3)?;
            let p = at(0)?;
            vec![
                vec![Event::PointerMoved(p)],
                vec![Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Line,
                    delta: vec2(0.0, n[2]),
                    phase: egui::TouchPhase::Move,
                    modifiers,
                }],
            ]
        }
        "gone" => {
            want(0)?;
            vec![vec![Event::PointerGone]]
        }
        _ => return Err(()),
    };
    Ok(frames.into())
}

/// The plugin: the file it follows, what it has read, what it owes.
pub struct PointerScript {
    path: PathBuf,
    offset: u64,
    partial: String,
    queue: VecDeque<Step>,
    /// `(viewport, frame)` of the last delivery, so a discarded pass that
    /// re-runs `begin_pass` in the same frame is not handed the step again.
    last: Option<(ViewportId, u64)>,
    /// Every viewport that has begun a pass.
    seen: Vec<ViewportId>,
}

impl PointerScript {
    /// The plugin, when the trace is on and the variable names a file.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        if !crate::diag::enabled() {
            return None;
        }
        // ui-text-exempt: an environment variable name, never displayed.
        let path = std::env::var_os("PDFCER_DIAG_POINTER")?;
        Some(Self {
            path: path.into(),
            offset: 0,
            partial: String::new(),
            queue: VecDeque::new(),
            last: None,
            seen: Vec::new(),
        })
    }

    /// Read what the harness appended since last time; queue whole lines.
    fn poll(&mut self) {
        let Ok(mut file) = std::fs::File::open(&self.path) else {
            return;
        };
        if file.seek(SeekFrom::Start(self.offset)).is_err() {
            return;
        }
        let mut bytes = Vec::new();
        if file.read_to_end(&mut bytes).is_err() || bytes.is_empty() {
            return;
        }
        self.offset += bytes.len() as u64;
        self.partial.push_str(&String::from_utf8_lossy(&bytes));
        while let Some(end) = self.partial.find('\n') {
            let line: String = self.partial.drain(..=end).collect();
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match parse(line) {
                Ok(step) => self.queue.push_back(step),
                Err(()) => {
                    let seq = line.split_whitespace().next().unwrap_or("?");
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        format!("diag-pointer-refused seq={seq} line={line}")
                    });
                }
            }
        }
    }
}

impl egui::Plugin for PointerScript {
    fn debug_name(&self) -> &'static str {
        "pdfcer-diag-pointer"
    }

    fn setup(&mut self, _ctx: &Context) {
        let path = self.path.display().to_string();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("diag-pointer-armed path={path}")
        });
    }

    /// Wake-ups are requested inside the pass: eframe ignores the output's
    /// `repaint_delay`, and a request made from `input_hook` precedes the
    /// pass and is cleared by its own begin.
    fn on_end_pass(&mut self, ui: &mut egui::Ui) {
        let viewport = ui.ctx().viewport_id();
        let Some(step) = self.queue.front() else {
            if viewport == ViewportId::ROOT {
                ui.ctx().request_repaint_after(POLL);
            }
            return;
        };
        ui.ctx().request_repaint();
        for id in self.seen.iter().filter(|id| step.target.matches(**id)) {
            ui.ctx().request_repaint_of(*id);
        }
    }

    fn input_hook(&mut self, ctx: &Context, input: &mut RawInput) {
        let viewport = input.viewport_id;
        if !self.seen.contains(&viewport) {
            self.seen.push(viewport);
        }
        if viewport == ViewportId::ROOT {
            self.poll();
        }
        let Some(step) = self.queue.front_mut() else {
            return;
        };
        if !step.target.matches(viewport) {
            return;
        }
        let frame = ctx.cumulative_frame_nr_for(viewport);
        if self.last == Some((viewport, frame)) {
            return;
        }
        let Some(events) = step.frames.pop_front() else {
            return;
        };
        self.last = Some((viewport, frame));
        step.sent += 1;
        input.events.extend(events);
        if step.modifiers != Modifiers::NONE {
            input.modifiers = step.modifiers;
        }
        if step.frames.is_empty() {
            let (seq, sent) = (step.seq, step.sent);
            let verb = step.verb.clone();
            let vp = step.target.token().to_owned();
            self.queue.pop_front();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("diag-pointer seq={seq} verb={verb} vp={vp} frames={sent}")
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_click_is_a_move_then_a_press_then_a_release_on_three_frames() {
        let step = parse("4 click 10 20 btn=r mods=ctrl").unwrap();
        assert_eq!(step.seq, 4);
        assert_eq!(step.frames.len(), 3);
        assert!(matches!(step.frames[0][..], [Event::PointerMoved(_)]));
        assert!(matches!(
            step.frames[1][..],
            [Event::PointerButton {
                button: PointerButton::Secondary,
                pressed: true,
                ..
            }]
        ));
        assert!(step.modifiers.ctrl && step.modifiers.command);
    }

    #[test]
    fn a_drag_moves_in_steps_between_press_and_release() {
        let step = parse("1 drag 0 0 60 0 steps=3").unwrap();
        // move, press, three moves, release.
        assert_eq!(step.frames.len(), 6);
        let Event::PointerMoved(p) = step.frames[3][0] else {
            panic!("not a move")
        };
        assert!((p.x - 40.0).abs() < 1e-3);
    }

    #[test]
    fn a_malformed_line_is_refused_not_guessed() {
        for line in [
            "click 1 2",
            "1 click 1",
            "1 click 1 2 3",
            "1 fling 1 2",
            "1 click 1 2 btn=x",
            "1 click 1 2 mods=hyper",
        ] {
            assert!(parse(line).is_err(), "{line}");
        }
        assert_eq!(parse("2 gone vp=abc").unwrap().target.token(), "abc");
    }

    #[test]
    fn a_viewport_is_named_with_or_without_its_quotes() {
        let id = ViewportId::from_hash_of("dialog");
        let bare = format!("{id:?}").trim_matches('"').to_owned();
        for spelling in [bare.clone(), format!("\"{bare}\"")] {
            assert!(Target::Named(spelling.clone()).matches(id), "{spelling}");
        }
        assert!(!Target::Named(bare).matches(ViewportId::ROOT));
    }
}
