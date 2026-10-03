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
//! <seq> dclick X Y [mods=…] [vp=V]         <seq> tclick X Y [mods=…] [vp=V]
//! <seq> down X Y [btn=…] [mods=…] [vp=V]      <seq> up X Y [btn=…] [mods=…] [vp=V]
//! <seq> drag X0 Y0 X1 Y1 [steps=N] [btn=…] [mods=…] [vp=V]
//! <seq> wheel X Y DY [mods=…] [vp=V]
//! <seq> gone [vp=V]
//! <seq> shot [vp=V]
//! <seq> key NAME [mods=…] [vp=V]
//! <seq> type [vp=V] TEXT
//! <seq> paste [vp=V] TEXT
//! <seq> preedit [vp=V] TEXT                <seq> commit [vp=V] TEXT
//! <seq> copy [vp=V]                      <seq> cut [vp=V]
//! <seq> drop X Y [mods=…] [vp=V] PATH[|PATH…]
//! ```
//!
//! `key` presses and releases one key named as `egui::Key::from_name` spells
//! it (`A`, `Enter`, `Tab`, `Escape`, `Backspace`, …), so `key A mods=ctrl`
//! selects all in a focused field. `type` delivers everything after the verb
//! (and an optional `vp=`), spaces included, as one text event to whatever
//! holds keyboard focus: click the field first. `copy` and `cut` deliver the
//! platform's Copy and Cut commands, as Ctrl+C and Ctrl+X reach the app.
//! `preedit` and `commit` deliver an input method's composition in progress
//! and its result, as egui-winit posts them.
//! `drop` moves to X Y, then lands the paths (spaces kept, `|` between them)
//! as a file drop with `mods` held; while a script drives the window, a drop's
//! position is the pointer's ([`scripted`]).
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
//! `shot` asks egui for the viewport's own rendered frame — the one oracle
//! for a window placed off the desktop, where an OS capture sees whatever is
//! on screen there — and writes it beside the step file as a binary PPM,
//! `<step file>.shot-<seq>.ppm`. Its acknowledgement waits for the file and
//! adds `path= w= h=`. eframe never screenshots an immediate viewport (its
//! `render_immediate_viewport` passes no screenshot commands), so a `shot`
//! aimed at a dialog shown with `show_viewport_immediate` is never answered.
//! A step whose viewport closes before its last frame
//! (Enter that closes its own dialog) is acknowledged with `closed=1`.
//!
//! An unreadable line answers `diag-pointer-refused seq=… line=…`. The
//! acknowledgement says the events were **delivered**, never what they did:
//! a check reads the effect from the application's own trace.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/pointerscripted.md`.

use std::collections::VecDeque;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput, ViewportId, vec2};

/// How often an idle app re-reads the file for new steps.
const POLL: Duration = Duration::from_millis(50);

/// The most egui's clock advances between two frames of one step, in
/// seconds. A slow frame must not stretch a scripted double-click past
/// egui's double-click delay or a click past its longest press; a real
/// hand's timing is the operator's, a script's is not a thing under test.
const STEP_FRAME_S: f64 = 1.0 / 60.0;

/// Whether a script drives this process's pointer.
static SCRIPTED: AtomicBool = AtomicBool::new(false);

/// Whether a harness drives the pointer, so a file drop's position is the
/// scripted pointer's rather than the operating system cursor's.
#[must_use]
pub fn scripted() -> bool {
    SCRIPTED.load(Ordering::Relaxed)
}

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
    /// The input time given to this step's last frame, from which the next
    /// frame's is capped by [`STEP_FRAME_S`].
    clock: Option<f64>,
    /// Files a `drop` lands with its last frame.
    files: Vec<PathBuf>,
}

/// Parse one line; `Err` carries nothing, the caller quotes the line.
fn parse(line: &str) -> Result<Step, ()> {
    let mut words = line.split_whitespace();
    let seq: u64 = words.next().ok_or(())?.parse().map_err(|_| ())?;
    let verb = words.next().ok_or(())?.to_owned();
    if matches!(verb.as_str(), "type" | "paste" | "preedit" | "commit") {
        return typed(seq, line, &verb);
    }
    if verb == "drop" {
        return dropped(seq, line);
    }
    let mut key = None;
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
        } else if verb == "key" && key.is_none() {
            key = Some(egui::Key::from_name(word).ok_or(())?);
        } else {
            numbers.push(word.parse().map_err(|_| ())?);
        }
    }
    let frames = if verb == "key" {
        if !numbers.is_empty() {
            return Err(());
        }
        let key = key.ok_or(())?;
        let event = |pressed| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        };
        VecDeque::from([vec![event(true)], vec![event(false)]])
    } else {
        expand(&verb, &numbers, button, modifiers, steps.max(1))?
    };
    Ok(Step {
        seq,
        verb,
        target,
        modifiers,
        frames,
        sent: 0,
        clock: None,
        files: Vec::new(),
    })
}

/// `\n`, `\t` and `\\` in a `paste` step, as the characters they name.
fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

/// A `type`, `paste`, `preedit` or `commit` step: the text is the rest of the line after the verb
/// and an optional leading `vp=`, separated by single spaces. A paste unescapes
/// `\n`, `\t` and `\\`, so a line break fits on the step's one line.
fn typed(seq: u64, line: &str, verb: &str) -> Result<Step, ()> {
    let rest = line.trim().splitn(3, ' ').nth(2).ok_or(())?;
    let (target, text) = match rest.split_once(' ') {
        Some((v, text)) if v.starts_with("vp=") => {
            let v = &v[3..];
            (
                if v == "root" {
                    Target::Root
                } else {
                    Target::Named(v.to_owned())
                },
                text,
            )
        }
        _ => (Target::Root, rest),
    };
    if text.is_empty() {
        return Err(());
    }
    let event = match verb {
        "paste" => Event::Paste(unescape(text)),
        "preedit" => Event::Ime(egui::ImeEvent::Preedit {
            text: text.to_owned(),
            active_range_chars: None,
        }),
        "commit" => Event::Ime(egui::ImeEvent::Commit(text.to_owned())),
        _ => Event::Text(text.to_owned()),
    };
    Ok(Step {
        seq,
        verb: verb.to_owned(),
        target,
        modifiers: Modifiers::NONE,
        frames: VecDeque::from([vec![event]]),
        sent: 0,
        clock: None,
        files: Vec::new(),
    })
}

/// A `drop` step: X and Y, optional `mods=` and `vp=`, then the paths.
fn dropped(seq: u64, line: &str) -> Result<Step, ()> {
    let mut rest = line.trim().splitn(3, ' ').nth(2).ok_or(())?;
    let mut numbers = Vec::new();
    let mut target = Target::Root;
    let mut modifiers = Modifiers::NONE;
    loop {
        let (word, tail) = rest.split_once(' ').unwrap_or((rest, ""));
        if numbers.len() < 2 {
            numbers.push(word.parse::<f32>().map_err(|_| ())?);
        } else if let Some(v) = word.strip_prefix("mods=") {
            modifiers = parse_mods(v)?;
        } else if let Some(v) = word.strip_prefix("vp=") {
            target = if v == "root" {
                Target::Root
            } else {
                Target::Named(v.to_owned())
            };
        } else {
            break;
        }
        rest = tail;
    }
    let files: Vec<PathBuf> = rest
        .split('|')
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .collect();
    if files.is_empty() {
        return Err(());
    }
    let p = Pos2::new(numbers[0], numbers[1]);
    Ok(Step {
        seq,
        verb: "drop".to_owned(),
        target,
        modifiers,
        frames: VecDeque::from([vec![Event::PointerMoved(p)], Vec::new()]),
        sent: 0,
        clock: None,
        files,
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
        "dclick" | "tclick" => {
            want(2)?;
            let p = at(0)?;
            let presses = if verb == "dclick" { 2 } else { 3 };
            let mut out = vec![vec![Event::PointerMoved(p)]];
            for _ in 0..presses {
                out.push(vec![press(p, true)]);
                out.push(vec![press(p, false)]);
            }
            out
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
        "shot" => {
            want(0)?;
            vec![Vec::new()]
        }
        "copy" => {
            want(0)?;
            vec![vec![Event::Copy]]
        }
        "cut" => {
            want(0)?;
            vec![vec![Event::Cut]]
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
    /// A `shot` delivered and not yet written.
    shot: Option<Shot>,
}

/// A screenshot owed to the harness.
struct Shot {
    seq: u64,
    viewport: ViewportId,
    /// Whether the viewport command has been sent; it is sent from inside a
    /// pass, because commands issued before `begin_pass` are discarded by it.
    requested: bool,
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
        SCRIPTED.store(true, Ordering::Relaxed);
        Some(Self {
            path: path.into(),
            offset: 0,
            partial: String::new(),
            queue: VecDeque::new(),
            last: None,
            seen: Vec::new(),
            shot: None,
        })
    }

    /// Write the owed screenshot if this viewport's input carries it, and
    /// acknowledge its step.
    fn write_shot(&mut self, viewport: ViewportId, input: &RawInput) {
        let Some(seq) = self
            .shot
            .as_ref()
            .filter(|s| s.viewport == viewport)
            .map(|s| s.seq)
        else {
            return;
        };
        let Some(image) = input.events.iter().find_map(|e| match e {
            Event::Screenshot {
                viewport_id, image, ..
            } if *viewport_id == viewport => Some(image.clone()),
            _ => None,
        }) else {
            return;
        };
        self.shot = None;
        let path = format!("{}.shot-{seq}.ppm", self.path.display());
        let [w, h] = image.size;
        let mut bytes = format!("P6\n{w} {h}\n255\n").into_bytes(); // ui-text-exempt: a PPM file header, never displayed
        bytes.reserve(w * h * 3);
        for px in &image.pixels {
            bytes.extend_from_slice(&[px.r(), px.g(), px.b()]);
        }
        let outcome = match std::fs::write(&path, bytes) {
            Ok(()) => format!("path={path} w={w} h={h}"), // ui-text-exempt: a trace field list, never displayed
            Err(e) => format!("error={}", e.to_string().replace(' ', "_")),
        };
        let vp = if viewport == ViewportId::ROOT {
            "root".to_owned()
        } else {
            format!("{viewport:?}")
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("diag-pointer seq={seq} verb=shot vp={vp} frames=1 {outcome}")
        });
    }

    /// Acknowledge, with `closed=1`, a started step whose viewport has closed:
    /// a key that closes its own window leaves frames no viewport will take.
    fn retire_orphan(&mut self, root: &RawInput) {
        let Some(step) = self.queue.front() else {
            return;
        };
        let alive = root.viewports.keys().any(|id| step.target.matches(*id));
        if step.sent == 0 || alive || matches!(step.target, Target::Root) {
            return;
        }
        let (seq, sent, verb) = (step.seq, step.sent, step.verb.clone());
        let vp = step.target.token().to_owned();
        self.queue.pop_front();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("diag-pointer seq={seq} verb={verb} vp={vp} frames={sent} closed=1")
        });
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
        if let Some(shot) = self
            .shot
            .as_mut()
            .filter(|s| s.viewport == viewport && !s.requested)
        {
            shot.requested = true;
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            ui.ctx().request_repaint();
        }
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
            self.retire_orphan(input);
        }
        self.write_shot(viewport, input);
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
        // Capping keeps time monotonic: each capped time exceeds the last,
        // and the step's end hands back the real clock, which is later still.
        if let Some(now) = input.time {
            let t = step.clock.map_or(now, |last| now.min(last + STEP_FRAME_S));
            input.time = Some(t);
            step.clock = Some(t);
        }
        input.events.extend(events);
        if step.frames.is_empty() {
            input
                .dropped_files
                .extend(step.files.drain(..).map(|path| egui::DroppedFile {
                    path: Some(path),
                    ..Default::default()
                }));
        }
        if step.modifiers != Modifiers::NONE {
            input.modifiers = step.modifiers;
        }
        if step.frames.is_empty() {
            let (seq, sent) = (step.seq, step.sent);
            let verb = step.verb.clone();
            let vp = step.target.token().to_owned();
            self.queue.pop_front();
            if verb == "shot" {
                // Acknowledged by `write_shot`, once the file exists.
                self.shot = Some(Shot {
                    seq,
                    viewport,
                    requested: false,
                });
                return;
            }
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

    fn script_with(step: Step) -> PointerScript {
        PointerScript {
            path: PathBuf::new(),
            offset: 0,
            partial: String::new(),
            queue: VecDeque::from([step]),
            last: None,
            seen: Vec::new(),
            shot: None,
        }
    }

    #[test]
    fn a_started_step_whose_viewport_closed_is_retired_and_an_unstarted_one_waits() {
        let root_only = RawInput::default();
        let mut started = parse("8 key Enter vp=1071").unwrap();
        started.sent = 1;
        let mut script = script_with(started);
        script.retire_orphan(&root_only);
        assert!(script.queue.is_empty());

        let mut waiting = script_with(parse("8 key Enter vp=1071").unwrap());
        waiting.retire_orphan(&root_only);
        assert_eq!(waiting.queue.len(), 1);
    }

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
    fn a_key_is_a_press_then_a_release_with_its_modifiers() {
        let step = parse("2 key A mods=ctrl").unwrap();
        assert_eq!(step.frames.len(), 2);
        assert!(matches!(
            step.frames[0][..],
            [Event::Key { key: egui::Key::A, pressed: true, modifiers, .. }] if modifiers.command
        ));
        assert!(parse("2 key NoSuchKey").is_err());
        assert!(parse("2 key").is_err());
        assert!(parse("2 key A 5").is_err());
    }

    #[test]
    fn type_carries_the_rest_of_the_line_spaces_included() {
        let step = parse("3 type http://127.0.0.1:80/ a b").unwrap();
        assert!(matches!(&step.frames[0][..], [Event::Text(t)] if t == "http://127.0.0.1:80/ a b"));
        assert_eq!(step.target, Target::Root);
        let step = parse("4 type vp=abc hello").unwrap();
        assert_eq!(step.target, Target::Named("abc".to_owned()));
        assert!(matches!(&step.frames[0][..], [Event::Text(t)] if t == "hello"));
        assert!(parse("5 type").is_err());
    }

    #[test]
    fn paste_carries_an_escaped_line_break() {
        let step = parse("6 paste a\\nb\\\\c").unwrap();
        assert_eq!(step.verb, "paste");
        assert!(matches!(&step.frames[0][..], [Event::Paste(t)] if t == "a\nb\\c"));
    }

    #[test]
    fn preedit_and_commit_are_input_method_events() {
        let step = parse("10 preedit zq").unwrap();
        assert_eq!(step.verb, "preedit");
        assert!(
            matches!(&step.frames[0][..], [Event::Ime(egui::ImeEvent::Preedit { text, .. })] if text == "zq")
        );
        let step = parse("11 commit vp=abc é").unwrap();
        assert!(matches!(&step.frames[0][..], [Event::Ime(egui::ImeEvent::Commit(t))] if t == "é"));
        assert!(parse("12 commit").is_err());
    }

    #[test]
    fn copy_and_cut_are_one_platform_command_each() {
        let step = parse("7 copy").unwrap();
        assert!(matches!(&step.frames[0][..], [Event::Copy]));
        let step = parse("8 cut vp=abc").unwrap();
        assert!(matches!(&step.frames[0][..], [Event::Cut]));
        assert_eq!(step.target, Target::Named("abc".to_owned()));
        assert!(parse("9 copy 5").is_err());
    }

    #[test]
    fn a_drop_moves_then_lands_every_path_with_its_modifiers() {
        let step = parse("7 drop 10 20 mods=alt C:/a b.png|C:/c.png").expect("parses");
        assert_eq!(step.verb, "drop");
        assert!(step.modifiers.alt);
        assert_eq!(step.frames.len(), 2);
        assert!(
            matches!(&step.frames[0][..], [Event::PointerMoved(p)] if *p == Pos2::new(10.0, 20.0))
        );
        assert_eq!(
            step.files,
            [PathBuf::from("C:/a b.png"), PathBuf::from("C:/c.png")]
        );
        assert!(parse("8 drop 10 20").is_err(), "a drop names a file");
        assert!(
            parse("9 drop 10 C:/a.png").is_err(),
            "a drop names both coordinates"
        );
    }

    #[test]
    fn a_shot_is_one_frame_with_no_events() {
        let step = parse("9 shot").unwrap();
        assert_eq!(step.frames.len(), 1);
        assert!(step.frames[0].is_empty());
        assert!(parse("9 shot 1 2").is_err());
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
