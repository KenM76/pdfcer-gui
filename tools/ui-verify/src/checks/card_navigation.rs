//! `card_navigation` — **navigating with the page drawn on the graphics card
//! never holds the window longer than the processor does.** The same zoom and
//! pan script is driven twice on one drawing, Settings ▸ Display ▸ *Draw pages
//! on the graphics card* off and on. Each run reports the frames over 50 ms
//! (`frame-long`), the longest, their sum, the renders started, the card's own
//! draw time and the time until the page stops re-rendering. Driven off the
//! desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/card_navigation.md`.

use std::time::{Duration, Instant};

use super::dimdrive::{Fixture, launch_path, ui_rect_event};
use crate::checks::driving::declared;
use crate::checks::scripteddrive::userdata;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "a1-titleblock.pdf",
    method: "It is checked in under fixtures/.",
    page: PageGeometry {
        width_pt: 2383.94,
        height_pt: 1683.78,
    },
};
const VIEWPORT: &str = "canvas-viewport"; // ui-text-exempt: a trace region name
const KEY: &str = "draw_pages_on_graphics_card"; // ui-text-exempt: a preferences-file key
const LONG: &str = "frame-long"; // ui-text-exempt: a trace event name
const SPAWN: &str = "render-spawn"; // ui-text-exempt: a trace event name
const CARD: &str = "render-card"; // ui-text-exempt: a trace event name
const DONE: &str = "render-async-done"; // ui-text-exempt: a trace event name
const INLINE: &str = "render-inline"; // ui-text-exempt: a trace event name
/// Ctrl+wheel notches per burst, and the gap between notches: a hand.
const NOTCHES: usize = 4;
const NOTCH_GAP: Duration = Duration::from_millis(40);
/// The pause between gestures: long enough for a render to start.
const BETWEEN: Duration = Duration::from_millis(350);
/// Rounds of in, pan, in, pan, out, out.
const ROUNDS: usize = 2;
/// The page is settled once no render has started for this long.
const QUIET: Duration = Duration::from_millis(1500);
const SETTLE_CAP: Duration = Duration::from_secs(60);
/// On fails when its longest frame is past both this and the off run's
/// longest times [`WORSE`].
const FLOOR_MS: u64 = 100;
const WORSE: u64 = 2;

/// See the module documentation.
pub struct CardNavigation;

impl Check for CardNavigation {
    fn name(&self) -> &'static str {
        "navigating_on_the_graphics_card_never_holds_the_window_longer_than_the_processor"
    }

    fn defect(&self) -> &'static str {
        "with Draw pages on the graphics card on, zooming and panning freezes the window for \
         longer than the processor's drawing does"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = drive(ctx, &mut report);
        if let Ok(dir) = userdata(ctx) {
            let _ = crate::sandbox::reset_prefs(&dir);
        }
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// One run's figures.
struct Run {
    long_frames: usize,
    longest_ms: u64,
    held_ms: u64,
    spawns: usize,
    card_drawn: usize,
    card_draw_ms: u64,
    card_longest_draw_ms: u64,
    settle_ms: u128,
}

impl Run {
    fn line(&self, tag: &str) -> String {
        format!(
            "{tag}: {} frames over 50 ms, longest {} ms, {} ms held in all; {} renders started; \
             card drew {} ({} ms in all, longest {} ms); last picture {} ms after the gestures",
            self.long_frames,
            self.longest_ms,
            self.held_ms,
            self.spawns,
            self.card_drawn,
            self.card_draw_ms,
            self.card_longest_draw_ms,
            self.settle_ms
        )
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let off = bench(ctx, report, "off", "")?;
    let on = bench(ctx, report, "on", &format!("{KEY} = true"))?;
    report.note(off.line("off"));
    report.note(on.line("on"));
    if off.spawns == 0 || on.spawns == 0 {
        return Err(Error::new(
            "a run started no render: the wheel did not reach the canvas, so nothing was \
             measured. SKIPPED.",
        ));
    }
    if on.card_drawn == 0 {
        return Err(Error::new(format!(
            "with {KEY} = true the card drew nothing: this run measured the processor twice. \
             SKIPPED."
        )));
    }
    let limit = (off.longest_ms * WORSE).max(FLOOR_MS);
    if on.longest_ms > limit {
        return Ok(Some(format!(
            "with the card on the window was held {} ms in one frame, past {limit} ms (the \
             processor's longest, {} ms, times {WORSE}, floor {FLOOR_MS} ms). {}. {}.",
            on.longest_ms,
            off.longest_ms,
            on.line("on"),
            off.line("off")
        )));
    }
    Ok(None)
}

/// The drawing driven: `--pdf` when given, else the A1 title block; always a
/// copy, because a session may write the file it opened.
fn document(ctx: &CheckContext, stem: &str) -> Result<std::path::PathBuf> {
    let doc = ctx.out(&format!("{stem}.pdf"));
    let source = match &ctx.pdf {
        Some(p) => p.clone(),
        None => crate::checks::driving::repo_fixture(FIXTURE.file, FIXTURE.method)?,
    };
    std::fs::copy(&source, &doc)
        .map_err(|e| Error::new(format!("copying {}: {e}", source.display())))?;
    Ok(doc)
}

fn bench(ctx: &CheckContext, report: &mut CheckReport, tag: &str, prefs: &str) -> Result<Run> {
    crate::sandbox::write_prefs(&userdata(ctx)?, prefs)
        .map_err(|e| Error::new(format!("could not plant the preferences: {e}")))?;
    let stem = format!("card_navigation.{tag}");
    let doc = document(ctx, &stem)?;
    let (session, pointer) = launch_path(ctx, report, doc, "mode.read", &stem, &[])?;
    session.settle(60);
    let trace = session.trace()?;
    let Some(canvas) = declared(&trace, ui_rect_event(ctx)?, VIEWPORT) else {
        return Err(Error::new(format!("no `{VIEWPORT}` region. SKIPPED.")));
    };
    let at = WindowPoint::centre_of(canvas);
    pointer.hover(&session, at)?;
    session.settle(20);
    let mark = session.trace()?.mark();
    script(&session, &pointer, at)?;
    let settle_ms = settle(&session)?.as_millis();
    let trace = session.trace()?;
    let _ = pointer.gone(&session);
    let after = |l: &&crate::trace::TraceLine| l.lineno > mark;
    let ms = |l: &crate::trace::TraceLine, k: &str| {
        l.get(k).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0)
    };
    let long: Vec<u64> = trace
        .events(LONG)
        .filter(after)
        .map(|l| ms(l, "ms"))
        .collect();
    let card: Vec<u64> = trace
        .events(CARD)
        .filter(after)
        .filter(|l| l.get("drew") == Some("1"))
        .map(|l| ms(l, "draw_ms"))
        .collect();
    for l in trace.events(LONG).filter(after).take(5) {
        report.note(format!("{tag}: {}", l.raw));
    }
    Ok(Run {
        long_frames: long.len(),
        longest_ms: long.iter().copied().max().unwrap_or(0),
        held_ms: long.iter().sum(),
        spawns: trace.events(SPAWN).filter(after).count(),
        card_drawn: card.len(),
        card_draw_ms: card.iter().sum(),
        card_longest_draw_ms: card.iter().copied().max().unwrap_or(0),
        settle_ms,
    })
}

/// In, pan, in, pan, out, out; [`ROUNDS`] times.
fn script(session: &Session, pointer: &ScriptedPointer, at: WindowPoint) -> Result<()> {
    let burst = |dy: f32, ctrl: bool| -> Result<()> {
        for _ in 0..NOTCHES {
            pointer.wheel(session, at, dy, ctrl)?;
            std::thread::sleep(NOTCH_GAP);
        }
        std::thread::sleep(BETWEEN);
        Ok(())
    };
    for _ in 0..ROUNDS {
        burst(1.0, true)?;
        burst(-3.0, false)?;
        burst(1.0, true)?;
        burst(3.0, false)?;
        burst(-1.0, true)?;
        burst(-1.0, true)?;
    }
    Ok(())
}

/// Wait until no render has started for [`QUIET`] and the last one started
/// has landed (`render-async-done` or `render-inline`); the time from now to that landing, which
/// is when the final picture arrived.
fn settle(session: &Session) -> Result<Duration> {
    let began = Instant::now();
    let mut count = session.trace()?.events(SPAWN).count();
    let mut last = began;
    while began.elapsed() < SETTLE_CAP {
        std::thread::sleep(Duration::from_millis(50));
        let trace = session.trace()?;
        let now = trace.events(SPAWN).count();
        if now != count {
            count = now;
            last = Instant::now();
            continue;
        }
        let spawned = trace.events(SPAWN).last().map_or(0, |l| l.lineno);
        let landed = [DONE, INLINE]
            .iter()
            .any(|e| trace.events(e).any(|l| l.lineno > spawned));
        if landed && last.elapsed() >= QUIET {
            return Ok(began.elapsed());
        }
        if landed {
            let at = began.elapsed();
            while began.elapsed() < SETTLE_CAP && last.elapsed() < QUIET {
                std::thread::sleep(Duration::from_millis(50));
                if session.trace()?.events(SPAWN).count() != count {
                    return settle(session);
                }
            }
            return Ok(at);
        }
    }
    Err(Error::new(format!(
        "the last render had not landed {} s after the last gesture. SKIPPED.",
        SETTLE_CAP.as_secs()
    )))
}
