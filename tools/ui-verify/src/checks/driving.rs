//! `checks::driving` — the moves that **every check which drives the ribbon**
//! has to make, in one place.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/driving.md`.

use crate::coords::WindowFrame;
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::image::{Image, Rgb};
use crate::input::Driver;
use crate::launch::Session;
use crate::trace::Trace;

/// The shell's own diagnostic switch, and its value.
pub const SHELL_DIAG_ENV: (&str, &str) = ("EGUI_SHELL_DIAG", "1");

/// The line prefix `egui-shell` uses when the application has not set one.
pub const SHELL_TRACE_PREFIX: &str = "egui-shell-diag";

/// `ribbon-mode-selected mode=…` — the shell reporting a mode-segment click.
pub const MODE_EVENT: &str = "ribbon-mode-selected";

/// `ribbon-tab-activated tab=…` — the shell reporting a tab click.
pub const TAB_EVENT: &str = "ribbon-tab-activated";

/// `ribbon-command-invoked id=… handler=…` — the shell reporting that a band
/// control was clicked and its token handed to the application.
pub const INVOKE_EVENT: &str = "ribbon-command-invoked";

/// `command-unimplemented id=…` — `app/dispatch.rs`'s fall-through arm.
pub const UNIMPLEMENTED_EVENT: &str = "command-unimplemented";

/// The namespace one ribbon command control's rect is published under.
pub const ITEM_PREFIX: &str = "ribbon.item.";

/// The last rect the application declared under `name`, if any.
#[must_use]
pub fn declared(trace: &Trace, ui_rect: &str, name: &str) -> Option<LRect> {
    // A region that was RETIRED after its last declaration is not declared.
    //
    // The application's `ui-rect` channel is a CHANGE LOG — it emits only when
    // a rect moves — so a control that stops being drawn leaves its last rect
    // standing in the trace with nothing after it. Reading `.last()` alone
    // therefore returns a fossil, and a caller cannot tell it from a live
    // region.
    //
    // That is not hypothetical: it made the UI-scale check report eighteen
    // ribbon controls as lying outside the window at a large scale, when the
    // ribbon's overflow had correctly swallowed every one of them and the
    // screenshot showed a clean layout with a *5 more* button. A confident,
    // detailed, entirely wrong layout-defect report, produced by reading a
    // change log as a snapshot.
    //
    // The application now closes each frame with a `ui-rect-gone name=…` line
    // per region it stopped drawing, so the log reports both directions. This
    // compares positions in the trace: a `gone` after the last `ui-rect` means
    // the region is not on screen, whatever rect it last had.
    //
    // Older traces — captured before that line existed — carry no `gone`
    // events at all, so this degrades to the previous behaviour rather than
    // to an error. That matters because `--image` runs replay dated captures.
    // `TraceLine::lineno` is the position in the FILE, so the two event
    // streams are comparable. Enumerating each iterator separately would give
    // two independent counters and compare a ui-rect's ordinal against a
    // gone-event's ordinal, which is meaningless — and would silently be
    // *mostly* right, since there are far more of the former.
    let (line_of_last_rect, rect) = trace
        .events(ui_rect)
        .filter(|l| l.get("name") == Some(name))
        .filter_map(|l| l.get_rect("rect").map(|r| (l.lineno, r)))
        .last()?;
    let retired_after = trace
        .events(UI_RECT_GONE_EVENT)
        .any(|l| l.lineno > line_of_last_rect && l.get("name") == Some(name));
    if retired_after { None } else { Some(rect) }
}

/// The `viewport-inner` event: a child viewport's client rectangle, in
/// **desktop logical points**.
pub const VIEWPORT_INNER_EVENT: &str = "viewport-inner";

/// The `viewport-outer` event: a child viewport's whole window — decoration
/// included — in **desktop logical points**.
pub const VIEWPORT_OUTER_EVENT: &str = "viewport-outer";

/// **The frame a declared region's coordinates are relative to.**
pub fn frame_for(session: &Session, trace: &Trace, viewport: Option<&str>) -> Result<WindowFrame> {
    let main = session.frame()?;
    let Some(id) = viewport else {
        return Ok(main);
    };
    let inner = trace
        .events(VIEWPORT_INNER_EVENT)
        .filter(|l| l.get("id") == Some(id))
        .filter_map(|l| l.get_rect("rect"))
        .last()
        .ok_or_else(|| {
            Error::new(format!(
                "a region was published in viewport `{id}` and no \
                 `{VIEWPORT_INNER_EVENT} id={id}` line says where that viewport is. The \
                 harness refuses to convert against the application window instead: the \
                 numbers would be plausible and the click would land somewhere else, which is \
                 the exact defect the tag exists to prevent. Look at \
                 `dialogs::host::Host::show`, which publishes it."
            ))
        })?;
    // The dialog's own client origin, in DESKTOP PIXELS.
    //
    // `viewport-inner` is in egui's logical points of monitor space, and
    // `WindowFrame::client_origin` is in pixels — the same relationship
    // `to_screen` already applies to a window point, applied once here to the
    // origin instead of once per point. The scale is the application's, which
    // is correct: a child viewport of the same application renders at the same
    // scale, and a per-monitor difference is a case neither this harness nor
    // the application handles yet.
    Ok(WindowFrame {
        client_origin: (
            (inner.min.x * main.scale).round() as i32,
            (inner.min.y * main.scale).round() as i32,
        ),
        client_size: (
            ((inner.max.x - inner.min.x) * main.scale).round() as u32,
            ((inner.max.y - inner.min.y) * main.scale).round() as u32,
        ),
        scale: main.scale,
    })
}

/// **A region's rectangle and the viewport it was drawn in.**
#[must_use]
pub fn declared_in(trace: &Trace, ui_rect: &str, name: &str) -> Option<(LRect, Option<String>)> {
    let rect = declared(trace, ui_rect, name)?;
    let viewport = trace
        .events(ui_rect)
        .filter(|l| l.get("name") == Some(name))
        .last()
        .and_then(|l| l.get("viewport").map(str::to_owned));
    Some((rect, viewport))
}

/// **The frame to convert a named region against**, whichever window drew it.
///
///
/// The idiom every check used was:
///
/// ```ignore
/// let button = declared(&trace, ui_rect, "dialog:export-dxf.export")?;
/// driver.click_at(session.frame()?.declared_center(button))?;
/// ```
///
/// which is correct for as long as every region is drawn in the application's
/// own window. The day the other thirteen dialogs became real OS windows, six
/// checks failed and six more skipped — **every one of them clicking hundreds
/// of pixels from the control it named**, with no error anywhere, because a
/// child viewport's rectangles are relative to ITS origin and the numbers stay
/// perfectly plausible.
///
/// That is the defect `a_child_viewports_ui_rects_are_relative_to_ITS_origin`
/// records, arriving in bulk. [`frame_for`] was written for the print dialog
/// and does the conversion; this is the two-argument form that finds the
/// viewport for you, so a call site changes by one word rather than by four
/// lines:
///
/// ```ignore
/// driver.click_at(frame_of(&session, &trace, ui_rect, NAME)?.declared_center(button))?;
/// ```
///
/// It is **safe on a main-window region** and that is the point: an untagged
/// region answers with `session.frame()`, unchanged. So a call site converted
/// pre-emptively costs nothing and survives its surface being moved into a
/// dialog later — which is the direction this shell keeps moving.
///
/// # Errors
///
/// When the region names a viewport whose origin was never published. See
/// [`frame_for`] for why that is refused rather than guessed around.
pub fn frame_of(
    session: &Session,
    trace: &Trace,
    ui_rect: &str,
    name: &str,
) -> Result<crate::coords::WindowFrame> {
    let viewport = declared_in(trace, ui_rect, name).and_then(|(_, v)| v);
    frame_for(session, trace, viewport.as_deref())
}

/// **A region's rectangle, once it has stopped moving.**
pub fn stable_rect(
    session: &Session,
    ui_rect: &str,
    name: &str,
    tries: u32,
) -> Result<Option<LRect>> {
    let mut previous = declared(&session.trace()?, ui_rect, name);
    for _ in 0..tries {
        session.settle(8);
        let now = declared(&session.trace()?, ui_rect, name);
        // `None` twice is stable too, and is the honest answer for a region
        // that is not on screen — the caller's own message is what says so.
        if now == previous {
            return Ok(now);
        }
        previous = now;
    }
    Ok(previous)
}

/// The last rect a region was published with **after** a given trace line,
/// whether or not it has since been retired.
#[must_use]
pub fn declared_since(trace: &Trace, ui_rect: &str, name: &str, after: usize) -> Option<LRect> {
    trace
        .events(ui_rect)
        .filter(|l| l.lineno > after && l.get("name") == Some(name))
        .filter_map(|l| l.get_rect("rect"))
        .last()
}

/// The event the application emits for a region it has stopped drawing.
pub const UI_RECT_GONE_EVENT: &str = "ui-rect-gone";

/// The event the application emits for a region it DECLINED to publish
/// because too little of it survived the clip.
pub const UI_RECT_CLIPPED_EVENT: &str = "ui-rect-clipped";

/// **Why `name` is missing, when the answer is “it drew, off the edge”.**
#[must_use]
pub fn clipped_away(trace: &Trace, ui_rect: &str, name: &str) -> Option<String> {
    let (line_of_clip, detail) = trace
        .events(UI_RECT_CLIPPED_EVENT)
        .filter(|l| l.get("name") == Some(name))
        .map(|l| (l.lineno, l.raw.clone()))
        .last()?;
    let published_after = trace
        .events(ui_rect)
        .any(|l| l.lineno > line_of_clip && l.get("name") == Some(name));
    if published_after { None } else { Some(detail) }
}

/// Every region name beginning with `prefix` that is **on screen now**.
#[must_use]
pub fn live_names(trace: &Trace, ui_rect: &str, prefix: &str) -> Vec<String> {
    declared_names(trace, ui_rect, prefix)
        .into_iter()
        .filter(|name| declared(trace, ui_rect, name).is_some())
        .collect()
}

/// Every distinct region name the application declared beginning with
/// `prefix`, in first-seen order.
#[must_use]
pub fn declared_names(trace: &Trace, ui_rect: &str, prefix: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in trace.events(ui_rect) {
        let Some(name) = line.get("name") else {
            continue;
        };
        if name.starts_with(prefix) && !out.iter().any(|n| n == name) {
            out.push(name.to_owned());
        }
    }
    out
}

/// Read the same captured stderr a second time, under the **shell's** line
/// prefix.
pub const OVERFLOW: &str = "ribbon.overflow";

/// The arrow that scrolls the band back towards its **first** group.
pub const SCROLL_LEFT: &str = "ribbon.scroll.left";

/// How many band scrolls this helper performs before it gives up.
const MAX_BAND_SCROLLS: usize = 32;

/// **Find a ribbon item wherever the responsive band has put it.**
pub fn declared_or_in_overflow(
    session: &Session,
    driver: &crate::input::Driver,
    ui_rect: &str,
    name: &str,
) -> Result<Option<LRect>> {
    Ok(search_the_band(session, driver, ui_rect, name)?.0)
}

/// **How far [`search_the_band`] had to go** to answer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BandSearch {
    /// Right-arrow clicks — how many groups the band was moved on by.
    pub scrolls: usize,
    /// Left-arrow clicks, across both rewinds.
    pub rewinds: usize,
    /// Collapsed-group popups opened while looking.
    pub popups: usize,
    /// Whether the item was found without moving the band at all.
    pub found_where_it_stood: bool,
    /// Whether the item only became visible when a **collapsed group's popup**
    /// was opened, as opposed to being on the band at that stop.
    ///
    /// With [`Self::scrolls`], this is the pair that says *"the old
    /// single-click search could not have completed this run"*, and it is the
    /// pair rather than either half. Measured 2026-09-03: at 1,100 pt the File
    /// tab's About sits **one** scroll away and **inside a collapsed group** —
    /// so a `scrolls >= 2` assertion alone would have skipped, and a
    /// `found_in_popup` assertion alone would be satisfied by a popup at the
    /// band's starting position, which the old code searched perfectly well.
    ///
    /// The old order was: popups at stop 0, one scroll, then a **bare** look at
    /// the band. Anything needing a popup at any stop past the first was
    /// therefore invisible to it, and that is precisely what happened to About,
    /// Shortcuts and Properties.
    pub found_in_popup: bool,
}

/// [`declared_or_in_overflow`], reporting how it got there.
pub fn search_the_band(
    session: &Session,
    driver: &crate::input::Driver,
    ui_rect: &str,
    name: &str,
) -> Result<(Option<LRect>, BandSearch)> {
    let mut seen = BandSearch::default();

    // Step 1. Where the band already stands. Deliberately before the rewind:
    // an item that is on screen is on screen, and moving the band to find
    // something already found would change coordinates for no reason.
    if let Some(rect) = declared(&session.trace()?, ui_rect, name) {
        seen.found_where_it_stood = true;
        return Ok((Some(rect), seen));
    }

    // Step 2. Level the band, so "not found" means "not on this tab" rather
    // than "not to the right of where the last caller left things".
    seen.rewinds += rewind_band(session, driver, ui_rect)?;

    // Step 3. One stop at a time.
    for _ in 0..=MAX_BAND_SCROLLS {
        let (found, popups) = at_this_stop(session, driver, ui_rect, name)?;
        seen.popups += popups;
        if let Some((rect, via_popup)) = found {
            seen.found_in_popup = via_popup;
            return Ok((Some(rect), seen));
        }
        let trace = session.trace()?;
        let Some(arrow) = declared(&trace, ui_rect, OVERFLOW) else {
            // The right arrow has retired: the band is showing its last group,
            // so there is nowhere further to look. Step 4.
            break;
        };
        driver.click_at(frame_of(session, &trace, ui_rect, OVERFLOW)?.declared_center(arrow))?;
        session.settle(16);
        seen.scrolls += 1;
    }

    seen.rewinds += rewind_band(session, driver, ui_rect)?;
    Ok((None, seen))
}

/// Look for `name` with the band where it is standing, opening every collapsed
/// group on it in turn.
fn at_this_stop(
    session: &Session,
    driver: &crate::input::Driver,
    ui_rect: &str,
    name: &str,
) -> Result<(Option<(LRect, bool)>, usize)> {
    let trace = session.trace()?;
    if let Some(rect) = declared(&trace, ui_rect, name) {
        return Ok((Some((rect, false)), 0));
    }
    let mut popups = 0;
    for group in collapsed_groups(&trace, ui_rect) {
        driver.click_at(session.frame()?.declared_center(group))?;
        session.settle(16);
        popups += 1;
        if let Some(rect) = declared(&session.trace()?, ui_rect, name) {
            return Ok((Some((rect, true)), popups));
        }
        // Shut it again, so the next candidate is not clicked through an open
        // popup — and so a caller that goes on to measure the band sees the
        // band rather than a menu lying over it.
        driver.press(crate::sys::vk::ESCAPE)?;
        session.settle(8);
    }
    Ok((None, popups))
}

/// Scroll the band back to its first group, and leave it there.
fn rewind_band(session: &Session, driver: &crate::input::Driver, ui_rect: &str) -> Result<usize> {
    for clicks in 0..MAX_BAND_SCROLLS {
        let trace = session.trace()?;
        let Some(arrow) = declared(&trace, ui_rect, SCROLL_LEFT) else {
            return Ok(clicks);
        };
        driver.click_at(frame_of(session, &trace, ui_rect, SCROLL_LEFT)?.declared_center(arrow))?;
        session.settle(16);
    }
    Ok(MAX_BAND_SCROLLS)
}

/// Every collapsed group's button on the current tab, in the order reported.
fn collapsed_groups(trace: &Trace, ui_rect: &str) -> Vec<LRect> {
    declared_names(trace, ui_rect, "ribbon.group.")
        .into_iter()
        .filter(|n| n.ends_with(".collapsed"))
        .filter_map(|n| declared(trace, ui_rect, &n))
        .collect()
}

pub fn shell_trace(session: &Session) -> Result<Trace> {
    Trace::read(session.trace_path(), SHELL_TRACE_PREFIX)
}

/// The event name under which a ribbon control publishes **whether it was drawn
/// pressable**, in both crates.
pub const ENABLEMENT_EVENT: &str = "ribbon-item-enablement";

/// What the last enablement line said about one control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Enablement {
    /// `enabled=` — the command's predicate over the published conditions.
    pub enabled: bool,
    /// `live=` — the renderer's own second test, when it has one.
    pub live: Option<bool>,
}

impl Enablement {
    /// Whether the control was actually drawn pressable.
    #[must_use]
    pub fn pressable(self) -> bool {
        self.enabled && self.live.unwrap_or(true)
    }

    /// Whether the two predicates DISAGREE about this control.
    #[must_use]
    pub fn disagrees(self) -> bool {
        self.enabled && self.live == Some(false)
    }
}

/// Every control's LAST enablement line, from both crates' traces at once.
pub fn enablement(session: &Session) -> Result<std::collections::BTreeMap<String, Enablement>> {
    let app = session.trace()?;
    let shell = shell_trace(session)?;
    let mut lines: Vec<(usize, String, Enablement)> = app
        .events(ENABLEMENT_EVENT)
        .chain(shell.events(ENABLEMENT_EVENT))
        .filter_map(|l| {
            let id = l.get("id")?.to_owned();
            let enabled = l.get("enabled")? == "1";
            let live = l.get("live").map(|v| v == "1");
            Some((l.lineno, id, Enablement { enabled, live }))
        })
        .collect();
    lines.sort_by_key(|(lineno, _, _)| *lineno);
    let mut out = std::collections::BTreeMap::new();
    for (_, id, state) in lines {
        out.insert(id, state);
    }
    Ok(out)
}

/// Render a list of names for a reason string, or say plainly that there were
/// none.
///
/// `"none"` rather than `""`, because an empty list printed as nothing reads
/// as a formatting bug and hides the fact that was being reported.
#[must_use]
pub fn list(names: &[String]) -> String {
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}

/// [`list`] for borrowed strings.
#[must_use]
pub fn list_str(names: &[&str]) -> String {
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}

/// **Resolve a fixture from this repository, refusing to guess.**
///
/// # Why it is resolved at COMPILE time and never from `--source-root`
///
///
/// ```text
/// [SKIP] -> the fixture crates/fixtures/<name>.pdf is missing
/// ```
///
/// ⚠ A SKIP is not red. Those checks would have skipped **for ever while
/// looking healthy** -- the exact shape this project has filed against itself
/// more than once. `CARGO_MANIFEST_DIR` is `tools/ui-verify`, so two parents up
/// is the workspace root, and being resolved at compile time it cannot be got
/// wrong by an invocation, which is the property `--source-root` lacked.
///
/// # Why it is an `Err` and not a SKIP
///
/// A SKIP reads as *"this build does not have the feature"*. A missing fixture
/// is a fact about the **checkout**, and reporting it as a SKIP sends the next
/// reader to look at the application. The error names the absolute path so the
/// right thing gets fixed.
///
/// # The `method` argument
///
/// One sentence saying what the calling check's method IS -- *"one build's
/// positive reading on a file that contradicts itself, denied on a file that
/// does not"*. It is appended to the message so that a reader who hits this
/// learns why the fixture is not substitutable for whatever is nearest to hand.
/// Checks that pin their own documents ignore `--pdf` by design, and that is
/// surprising enough to be worth restating at the point it bites.
///
pub fn repo_fixture(name: &str, method: &str) -> Result<std::path::PathBuf> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join(name);
    if !path.is_file() {
        return Err(Error::new(format!(
            "the fixture {} is missing. {method}",
            path.display()
        )));
    }
    Ok(path)
}

/// The dominant colour of a declared region in a capture — a control's fill.
#[must_use]
pub fn fill_of(image: &Image, frame: &WindowFrame, rect: LRect) -> Option<Rgb> {
    let px = frame.logical_to_capture_pixels(rect);
    if px.area() == 0 {
        return None;
    }
    let report = crate::pixels::contrast_at(image, px);
    (report.sampled > 0).then_some(report.background)
}

/// Maximum absolute per-channel difference between two colours.
#[must_use]
pub fn delta(a: Rgb, b: Rgb) -> u16 {
    let d = |x: u8, y: u8| u16::from(x.abs_diff(y));
    d(a.r, b.r).max(d(a.g, b.g)).max(d(a.b, b.b))
}

/// How far apart two dominant fills must be to count as "one of these is
/// pressed", as a maximum absolute per-channel difference in 0–255.
pub const MIN_PRESSED_DELTA: u16 = 12;

/// **Click a mode segment and confirm the shell saw the click.**
pub fn click_mode_segment(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    mode_id: &str,
) -> Result<()> {
    let region = format!("ribbon.mode.{mode_id}");
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, &region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region, so there is no mode segment to \
             click and this check cannot put the application into the mode it is about. \
             Regions it did declare under `ribbon.mode.`: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.mode."))
        ))
    })?;
    if !rect.is_substantial() {
        return Err(Error::new(format!(
            "`{region}` was declared at {rect:?}, which has no usable area. A click aimed at a \
             degenerate rectangle proves nothing, so this is reported rather than driven — and \
             it is itself the finding: `MODES_AND_PANELS.md` Part 1 requires the selector to \
             render as a real segmented control with every label visible."
        )));
    }

    let before = shell_trace(session)?
        .events(MODE_EVENT)
        .filter(|l| l.get("mode") == Some(mode_id))
        .count();
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(12);
    let after = shell_trace(session)?
        .events(MODE_EVENT)
        .filter(|l| l.get("mode") == Some(mode_id))
        .count();
    if after <= before {
        let shell = shell_trace(session)?;
        return Err(Error::new(format!(
            "the click on `{region}` produced no new `{MODE_EVENT} mode={mode_id}` line, so no \
             click reached the ribbon and nothing after it would mean anything. Two readings, \
             and this check declines to choose between them: the pointer injection is not \
             reaching this window, or the shell diagnostic switch {}={} did not reach the \
             process — the shell trace carries {} line(s) under `{SHELL_TRACE_PREFIX}`. \
             Trace: {}.",
            SHELL_DIAG_ENV.0,
            SHELL_DIAG_ENV.1,
            shell.lines.len(),
            session.trace_path().display()
        )));
    }
    Ok(())
}

// The reachability family lives in [`super::reaching`] and is re-exported
// here so `driving::scroll_to`, `driving::raise_dock_tab` and
// `driving::bring_into_body` keep resolving at every call site. See that
// module's header for the seam and for R2.
pub use super::reaching::{bring_into_body, raise_dock_tab, scroll_to};

/// The select tool's ribbon control, on View ▸ Navigate.
pub const SELECT_TOOL_REGION: &str = "ribbon.item.view.tool_select";
/// The command id the shell reports for it on [`INVOKE_EVENT`].
pub const SELECT_TOOL_ID: &str = "view.tool_select";
/// The tab that carries it — View, which **every** mode is shown.
pub const VIEW_TAB: (&str, &str) = ("ribbon.tab.view", "view");

/// **Put the pen down with the POINTER, not with a key.** Answers whether the
/// ribbon route delivered the command.
pub fn arm_select_from_ribbon(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    report: &mut crate::report::CheckReport,
) -> Result<bool> {
    // The tab first, and only if the control is not already on the band: a tab
    // click is cheap but it is not free, and a check driven from View has the
    // control in front of it already.
    if declared(&session.trace()?, ui_rect, SELECT_TOOL_REGION).is_none() {
        let trace = session.trace()?;
        let Some(tab) = declared(&trace, ui_rect, VIEW_TAB.0) else {
            report.note(format!(
                "the pointer route to the select tool is unavailable: no `{}` region. Tabs \
                 declared: {}.",
                VIEW_TAB.0,
                list(&declared_names(&trace, ui_rect, "ribbon.tab."))
            ));
            return Ok(false);
        };
        driver.click_at(session.frame()?.declared_center(tab))?;
        session.settle(14);
        if !shell_trace(session)?
            .events(TAB_EVENT)
            .any(|l| l.get("tab") == Some(VIEW_TAB.1))
        {
            report.note(format!(
                "the click on `{}` produced no `{TAB_EVENT} tab={}` line, so the pointer route \
                 to the select tool could not be opened.",
                VIEW_TAB.0, VIEW_TAB.1
            ));
            return Ok(false);
        }
    }

    let Some(item) = declared_or_in_overflow(session, driver, ui_rect, SELECT_TOOL_REGION)? else {
        report.note(format!(
            "the View tab declares no `{SELECT_TOOL_REGION}`, on the band, in a collapsed group \
             or in the overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.view."
            ))
        ));
        return Ok(false);
    };

    // Before/after, not "did it happen at all": the application is free to have
    // armed the select tool earlier in the run for its own reasons, and what
    // this step needs to know is whether THIS click landed.
    let before = select_invokes(session)?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(20);
    if select_invokes(session)? <= before {
        report.note(format!(
            "the click on `{SELECT_TOOL_REGION}` produced no new `{INVOKE_EVENT} \
             id={SELECT_TOOL_ID}` line, so the pointer did not reach the control."
        ));
        return Ok(false);
    }
    Ok(true)
}

/// How many times the **shell** has reported [`SELECT_TOOL_ID`] invoked.
///
/// Read from [`shell_trace`], because `ribbon-command-invoked` is written by
/// `egui-shell` and `Session::trace` parses only the application's vocabulary.
fn select_invokes(session: &Session) -> Result<usize> {
    Ok(shell_trace(session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(SELECT_TOOL_ID))
        .count())
}

/// How many times [`press_until_traced`] will send a keystroke before
/// concluding it is not arriving.
pub const PRESS_TRIES: usize = 4;

/// **Press a key until the application's own trace shows it was heard**, and
/// answer whether it ever was.
pub fn press_until_traced(
    session: &Session,
    driver: &Driver,
    vk: u16,
    evidence: &[&str],
) -> Result<bool> {
    let count = |session: &Session| -> Result<usize> {
        let trace = session.trace()?;
        Ok(evidence.iter().map(|name| trace.events(name).count()).sum())
    };
    let before = count(session)?;
    for _ in 0..PRESS_TRIES {
        driver.press(vk)?;
        // The same settle a single-press check would have used. What the loop
        // adds is another look, not a longer one: a key that is going to be
        // processed is processed within a frame or two of arriving, and a key
        // that never arrived will not arrive by being waited for.
        session.settle(12);
        if count(session)? > before {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Pt;

    /// Last wins, per name, and a name that was never declared is `None`.
    #[test]
    fn a_regions_last_declaration_is_the_one_that_is_used() {
        let trace = Trace::parse(
            "pdfcer-diag start argv1=None\n\
             pdfcer-diag ui-rect name=ribbon.item.measure.linear rect=[[0.0 0.0] - [10.0 10.0]]\n\
             pdfcer-diag ui-rect name=ribbon.item.measure.two_line rect=[[20.0 0.0] - [30.0 10.0]]\n\
             pdfcer-diag ui-rect name=ribbon.item.measure.linear rect=[[4.0 30.0] - [84.0 54.0]]",
            "pdfcer-diag",
        );
        assert_eq!(
            declared(&trace, "ui-rect", "ribbon.item.measure.linear"),
            Some(LRect::new(Pt::new(4.0, 30.0), Pt::new(84.0, 54.0))),
            "an early frame can carry a rect from before the layout settled"
        );
        assert_eq!(
            declared(&trace, "ui-rect", "ribbon.item.measure.area"),
            None
        );
        assert_eq!(
            declared_names(&trace, "ui-rect", ITEM_PREFIX),
            vec![
                "ribbon.item.measure.linear".to_owned(),
                "ribbon.item.measure.two_line".to_owned()
            ],
            "each name once, in first-seen order"
        );
    }

    /// **The two channels are parsed out of one file without contaminating
    /// each other.**
    #[test]
    fn the_application_and_shell_streams_do_not_contaminate_each_other() {
        let text = "pdfcer-diag start argv1=None\n\
                    egui-shell-diag ribbon-mode-selected mode=review\n\
                    egui-shell-diag ribbon-command-invoked id=measure.linear handler=600\n\
                    pdfcer-diag measure-tool tool=Measure(Linear)\n";
        let app = Trace::parse(text, "pdfcer-diag");
        let shell = Trace::parse(text, SHELL_TRACE_PREFIX);

        assert!(app.started("start"));
        assert!(
            app.events(INVOKE_EVENT).next().is_none(),
            "the shell's line must not be read as the application's"
        );
        assert_eq!(
            app.last("measure-tool").and_then(|l| l.get("tool")),
            Some("Measure(Linear)")
        );
        assert!(
            shell
                .events(MODE_EVENT)
                .any(|l| l.get("mode") == Some("review"))
        );
        assert!(
            shell.events("measure-tool").next().is_none(),
            "the application's line must not be read as the shell's"
        );
    }

    /// The difference is symmetric and takes the largest channel, so a shift
    /// confined to one channel still registers.
    #[test]
    fn the_difference_is_the_largest_channel_and_is_symmetric() {
        let a = Rgb::new(200, 100, 50);
        let b = Rgb::new(190, 100, 90);
        assert_eq!(delta(a, b), 40);
        assert_eq!(delta(b, a), 40);
        assert_eq!(delta(a, a), 0, "identical fills differ by nothing at all");
    }

    /// **The threshold separates pressed from unpressed under both palettes
    /// this build might paint with — and a contrast ratio separates neither.**
    #[test]
    fn the_threshold_separates_pressed_from_unpressed_under_both_palettes() {
        let pairs = [
            (
                Rgb::new(229, 229, 229),
                Rgb::new(144, 209, 255),
                85_u16,
                "egui's stock light palette — MEASURED from a real capture",
            ),
            (
                Rgb::new(232, 232, 234),
                Rgb::new(193, 207, 230),
                39_u16,
                "egui-shell's `quiet` preset, composited — computed",
            ),
        ];
        for (unpressed, pressed, expected, what) in pairs {
            assert_eq!(delta(unpressed, pressed), expected, "{what}");
            assert!(
                expected > MIN_PRESSED_DELTA * 3,
                "the threshold must sit well below the difference produced by {what}"
            );
            let ratio = crate::pixels::contrast_ratio(unpressed, pressed);
            assert!(
                ratio < crate::pixels::AA_LARGE,
                "a contrast threshold would call these two fills the same colour \
                 ({ratio:.2}:1) under {what}, which is why this module measures a channel \
                 difference instead"
            );
        }
    }

    /// A list with nothing in it says so in words.
    #[test]
    fn an_empty_list_reads_as_none_rather_than_as_nothing() {
        assert_eq!(list(&[]), "none");
        assert_eq!(list_str(&[]), "none");
        assert_eq!(list_str(&["a", "b"]), "a, b");
    }
}
