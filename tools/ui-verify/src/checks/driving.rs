//! `checks::driving` — the moves that **every check which drives the ribbon**
//! has to make, in one place.
//!
//! # Why this module exists
//!
//! [`crate::checks::markup_rectangle`] was the first check to click a ribbon
//! control, and it had to invent five small things to do it: read the last
//! rect the application declared for a name, list the names it *did* declare
//! for a SKIP reason, re-parse the same captured stderr under the **shell's**
//! line prefix, measure a control's fill out of a capture, and compare two
//! fills. All five are properties of *driving an `egui-shell` ribbon*, not of
//! markup.
//!
//! The second and third such checks — [`crate::checks::measure_linear`] and
//! [`crate::checks::read_mode`] — needed the same five, and a third copy of a
//! function is where copies start to disagree. So they live here, with the
//! reasoning that shaped them carried across rather than summarised.
//!
//! `markup_rectangle` deliberately keeps its own copies. Rewriting a check
//! that is already known to detect its defect, in the same change that adds
//! two new ones, would mean the three checks stopped being independent
//! evidence of each other at exactly the moment the harness grew. This module
//! is a *widening*, not a refactor; when someone next has cause to touch
//! `markup_rectangle` for its own sake, folding it onto these is a one-line
//! change per helper.
//!
//! # The two diagnostic channels, and why a check reads both
//!
//! One captured stderr file, two vocabularies:
//!
//! | Channel | Switch | Prefix | Says |
//! |---|---|---|---|
//! | the shell | [`SHELL_DIAG_ENV`] | [`SHELL_TRACE_PREFIX`] | a segment/tab/control took a click |
//! | the application | the profile's `diag_env` | the profile's `trace_prefix` | what the application did about it |
//!
//! The split is not an accident of this build. `egui_shell::verify`'s header
//! explains that one environment variable name lets a harness arm tracing on
//! *any* `egui-shell` application without first discovering its name, and the
//! prefix is the application's so two crates' lines never blur together. The
//! consequence for a check is the thing that makes a failure attributable: a
//! present `ribbon-command-invoked` with an absent application-side effect
//! names the application's dispatch and nothing else, and an absent
//! `ribbon-command-invoked` means no click was ever delivered — which is a
//! SKIP, because a check that could not deliver a click has learned nothing.

use crate::coords::WindowFrame;
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::image::{Image, Rgb};
use crate::input::Driver;
use crate::launch::Session;
use crate::trace::Trace;

/// The shell's own diagnostic switch, and its value.
///
/// See the module header. `pdfcer-gui` does not call
/// `egui_shell::verify::set_prefix`, so the shell's lines arrive under the
/// crate's default prefix, [`SHELL_TRACE_PREFIX`].
pub const SHELL_DIAG_ENV: (&str, &str) = ("EGUI_SHELL_DIAG", "1");

/// The line prefix `egui-shell` uses when the application has not set one.
pub const SHELL_TRACE_PREFIX: &str = "egui-shell-diag";

/// `ribbon-mode-selected mode=…` — the shell reporting a mode-segment click.
///
/// Emitted on **every** click of a segment, including a click on the segment
/// that is already selected (`ribbon::mode_selector` sets `chosen` from the
/// click and filters for "did this change anything" only in its *return
/// value*, after the line is written). That is what makes it usable as the
/// input-channel proof for a mode a check merely wants to be *in*, rather than
/// only for a mode it is switching *to*.
pub const MODE_EVENT: &str = "ribbon-mode-selected";

/// `ribbon-tab-activated tab=…` — the shell reporting a tab click.
pub const TAB_EVENT: &str = "ribbon-tab-activated";

/// `ribbon-command-invoked id=… handler=…` — the shell reporting that a band
/// control was clicked and its token handed to the application.
pub const INVOKE_EVENT: &str = "ribbon-command-invoked";

/// `command-unimplemented id=…` — `app/dispatch.rs`'s fall-through arm.
///
/// Read only to *improve a failure message*: its presence alongside a missing
/// application-side effect is the signature of a dispatch that received the
/// command and had no arm for it, which is a different fix from a dispatch
/// that never received it at all.
pub const UNIMPLEMENTED_EVENT: &str = "command-unimplemented";

/// The namespace one ribbon command control's rect is published under.
pub const ITEM_PREFIX: &str = "ribbon.item.";

/// The last rect the application declared under `name`, if any.
///
/// **Last wins.** A region is re-declared whenever it moves, and an early
/// frame can carry a rect from before the layout settled — the find bar's
/// one-frame misplacement was exactly that, and taking the first occurrence
/// would aim a check's clicks at it.
#[must_use]
pub fn declared(trace: &Trace, ui_rect: &str, name: &str) -> Option<LRect> {
    // ★ A region that was RETIRED after its last declaration is not declared.
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

/// **The frame a declared region's coordinates are relative to.**
///
/// # ★★★ Why a region needs this at all, as of 2026-08-20
///
/// Every `ui-rect` rectangle is relative to **the viewport that drew it**.
/// There was one viewport until `crate::checks` was written and until this
/// harness's whole coordinate model was built, so
/// `session.frame()?.declared_center(rect)` — add the application window's
/// client origin — was right everywhere.
///
/// `dialogs::host` made a dialog a real OS window. Its regions publish
/// rectangles that look **exactly like** the ones this harness has always
/// converted and name a place several hundred points away, because the origin
/// they are relative to is the dialog's client area rather than the
/// application's.
///
/// That is a coordinate-space defect with plausible numbers, which is the
/// single most expensive shape of bug in this project's record — `D:/dev/rag/egui/`
/// carries three instances, every one presenting as *"the click lands somewhere
/// else"*. The application therefore tags each region with the viewport that
/// drew it and publishes that viewport's own origin, and this function joins
/// the two.
///
/// # What it returns
///
/// The `WindowFrame` to convert with. For an untagged region — the application
/// window, which is every region that existed before this — that is
/// `session.frame()`, unchanged, so no existing call site changes behaviour.
///
/// # ★ Why an absent `viewport-inner` is an ERROR and not a fallback
///
/// Because falling back to the main window's frame would produce a **click at
/// a plausible wrong place**, which is precisely the failure this exists to
/// prevent, arriving through the code written to prevent it. A tagged region
/// with no published origin means the application drew a dialog and did not say
/// where — a defect worth reporting, not one worth guessing around.
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
    // ★ The dialog's own client origin, in DESKTOP PIXELS.
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
///
/// [`declared`]'s twin, for a caller that is going to CLICK the region rather
/// than measure it. The rectangle alone is not enough to aim with once dialogs
/// have their own windows — see [`frame_for`].
///
/// Shares `declared`'s retirement rule, and shares it by calling it: a region
/// that has been retired is not declared, whichever viewport drew it.
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
/// # ★★★ Why almost every dialog-driving check needed this on 2026-08-21
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
/// ★ It is **safe on a main-window region** and that is the point: an untagged
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
///
/// Reads the region, settles, reads it again, and repeats until two consecutive
/// reads agree — or until it gives up and returns the last one it saw.
///
/// # ★★ Why this exists, and it is a defect report
///
/// `ui-rect` is a **change log**: the application emits a line when a rect
/// moves, so [`declared`] answers *where that control was as of the last frame
/// the application drew*. That is exactly right for a settled window and
/// exactly wrong for one in motion, and the difference is invisible — a stale
/// coordinate is a number, not an error.
///
/// Measured, on `dimension_groups_panel_makes_a_group`, 2026-08-19: raising a
/// dock panel changes the **dock's own** layout, and it lands over several
/// frames. The check read a fold heading at `x=786..1009, y=610`, the dock
/// then re-laid out, and by the time the click was injected the panel's left
/// edge had moved past the point being aimed at — so the click landed **on the
/// canvas** and the check reported the fold as broken. Adding settle time did
/// not fix it, because the motion is triggered by the very act being measured
/// rather than by the passage of time.
///
/// > **A harness that reads a coordinate and then acts on it owns the interval
/// > between the two.** The only honest way to close that interval is to watch
/// > the coordinate until it stops.
///
/// # Why it gives up rather than failing
///
/// A rect that never settles is a real state — an animation, a spinner, a
/// progress bar — and this helper cannot know whether the caller is aiming at
/// one. Returning the last observation lets the caller's own assertion produce
/// the verdict, in its own words, with its own diagnosis. A `Result` here would
/// make every call site handle a failure mode most of them cannot describe.
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
///
/// # ★★ Why [`declared`] is the wrong question for a gesture-only overlay
///
/// `declared` asks *"is this on screen now?"*, and it is right to: a region
/// retired after its last declaration is a fossil, and reading one produced a
/// confident, detailed, entirely wrong layout-defect report once already.
///
/// But an overlay that exists **only while the pointer is down** — a drop
/// caret, a rubber band, a snap indicator — is *guaranteed* to be retired by
/// the time a check can look at it. The harness cannot read the trace mid-drag:
/// `Driver::drag` presses, moves and releases before it returns. So `declared`
/// answers `None` for a caret that drew perfectly, and the check reports the
/// feature missing.
///
/// That is not hypothetical either. It is exactly what happened on
/// 2026-08-19: `pages_drag_shows_where_it_lands` failed with *"NO
/// `panel-pages-drop-caret` region was ever published"* while the trace
/// carried `ui-rect name=panel-pages-drop-caret rect=[[258.0 239.1] - [262.0
/// 331.9]]` four lines above the release. The indicator worked. The check was
/// reading a change log as a snapshot in the other direction — asking for
/// presence *now* about a thing whose whole nature is to be gone now.
///
/// # What this asks instead, and why the anchor is required rather than optional
///
/// *"Was it published during THIS gesture?"* The `after` line number is the
/// gesture's own start event, so a caret left over from an earlier drag in the
/// same run cannot satisfy it. Without that anchor this would be
/// `last-rect-ignoring-retirement`, which is the fossil-reading bug wearing a
/// helpful name — and it would pass on a build where the caret drew once at
/// startup and never again.
///
/// `TraceLine::lineno` is the position in the file, so the two streams are
/// directly comparable — the same property [`declared`] relies on.
#[must_use]
pub fn declared_since(trace: &Trace, ui_rect: &str, name: &str, after: usize) -> Option<LRect> {
    trace
        .events(ui_rect)
        .filter(|l| l.lineno > after && l.get("name") == Some(name))
        .filter_map(|l| l.get_rect("rect"))
        .last()
}

/// The event the application emits for a region it has stopped drawing.
///
/// Matched literally, like every other event name in this crate, so renaming
/// it in `crate::diag` without changing it here silently returns [`declared`]
/// to reading fossils.
pub const UI_RECT_GONE_EVENT: &str = "ui-rect-gone";

/// Every region name beginning with `prefix` that is **on screen now**.
///
/// # ★★ Why this exists beside [`declared_names`], which counts fossils
///
/// [`declared_names`]'s own documentation says *"used only for SKIP reasons"*,
/// and it means it: it collects every name that has **ever** appeared, because
/// an error message listing what the application *did* declare is more useful
/// the more it lists. Retirement is irrelevant to that job.
///
/// It is exactly wrong for a **count**. The `ui-rect` channel is a change log,
/// so a row that was deleted leaves its last declaration standing for ever, and
/// counting names therefore counts rows that are gone.
///
/// That is not hypothetical. On 2026-08-19 the Manage-groups check reported
/// *"the round trip did not close: 1 row before, 2 after the delete"* over a
/// trace containing `dimension-group-delete id=1`, `delete-dimension-group
/// epoch=3` **and** `ui-rect-gone name=dimension-groups.draw_into.1`. The
/// delete had worked, at every level, and the check said it had not — a
/// confident, specific, entirely wrong defect report about a feature that was
/// correct, produced by a helper being used outside the job its own doc comment
/// names.
///
/// So: **[`declared_names`] to say what was seen, this to say what is there.**
/// If a check compares two numbers, it wants this one.
#[must_use]
pub fn live_names(trace: &Trace, ui_rect: &str, prefix: &str) -> Vec<String> {
    declared_names(trace, ui_rect, prefix)
        .into_iter()
        .filter(|name| declared(trace, ui_rect, name).is_some())
        .collect()
}

/// Every distinct region name the application declared beginning with
/// `prefix`, in first-seen order.
///
/// Used only for SKIP reasons. A reason that says "I did not find X" and does
/// not say what it *did* find sends its reader to guess; this crate has a
/// standing rule about that ([`crate::checks`] rule 5).
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
///
/// One file, two vocabularies — see the module header. `Session::trace` parses
/// with the profile's prefix; everything `egui-shell` writes carries its own
/// and lands in [`Trace::other`] on that parse. Re-parsing is cheap next to a
/// click and keeps both streams honest: a line is attributed to whichever
/// crate actually wrote it.
///
/// # Errors
///
/// If the captured stderr cannot be read at all.
/// The control that holds the groups a narrow ribbon could not fit.
///
/// ★★ The name is a fossil that was kept on purpose. Until 2026-08-25 this was
/// a `⏷ N more` **dropdown**; it is now the **right scroll arrow**, and
/// `egui-shell`'s `ribbon::overflow` explains at length why the published name
/// did not change with the mechanism — four checks name it and what they assert
/// about it is still true. What *is* no longer true is the mental model a
/// reader brings to the word "overflow", and that cost this project three false
/// SKIPs. See [`declared_or_in_overflow`].
pub const OVERFLOW: &str = "ribbon.overflow";

/// The arrow that scrolls the band back towards its **first** group.
///
/// Drawn only while the band is scrolled off its start, so its presence is the
/// predicate *"this band is not at position zero"* — which is how `rewind_band`
/// knows when to stop.
pub const SCROLL_LEFT: &str = "ribbon.scroll.left";

/// How many band scrolls this helper performs before it gives up.
///
/// A bound rather than a `while true`, because a harness that hangs reports
/// nothing at all. It is deliberately far above any real tab — the widest tab
/// in `RIBBON_IA.md` has nine groups — so hitting it is a defect in the
/// application (an arrow that never retires) rather than a tab this helper
/// cannot search, and the two must not be confused.
const MAX_BAND_SCROLLS: usize = 32;

/// **Find a ribbon item wherever the responsive band has put it.**
///
/// ★ The fix for a whole class of false SKIPs, and it is worth understanding
/// why they were false rather than treating this as a convenience.
///
/// The harness drives a **1100 pt** window. At that width the ribbon correctly
/// re-wraps, collapses and finally scrolls its rightmost groups — that is the
/// responsive behaviour working, not failing. A check that looked only at the
/// tab surface then reported *"no `ribbon.item.file.print` region on the File
/// tab"*, which is true and reads as *"the command is missing"*, which is
/// false. It cost `print_dialog_reaches_the_spooler` a standing FAIL that was
/// written up as a harness gap and left, and it would have cost `about` the
/// same.
///
/// # ★★★ The overflow is a SCROLL, not a menu — and this helper did not know
///
/// Corrected 2026-09-03, and the shape is the one this project keeps meeting:
/// **prose and mechanism agreed when the prose was written, and then the
/// mechanism changed underneath it.**
///
/// This helper was written against the `⏷ N more` **dropdown**: click it once
/// and *everything* hidden appears at once, so one click is the whole search.
/// On 2026-08-25 the dropdown became a Word-style `›` arrow on the operator's
/// instruction, and one click now moves the band by **exactly one group**
/// (`egui-shell`'s `ribbon::band` — `set_first(.., scrolled + 1)`). The helper
/// kept clicking once, and — the part that actually bit — looked at the band
/// **bare** afterwards, having searched collapsed groups only at the band's
/// starting position. So the hole was *any command needing a collapsed group
/// opened at a stop past the first*, which is a superset of "two or more
/// scrolls" and is what the three checks below actually met. Either way the
/// command was reported absent, with a confident message naming it.
///
/// It was measured on 2026-09-02: `about_reports_the_build`,
/// `shortcuts_reference_is_live` and `properties_metadata_round_trips` all
/// SKIPPED reporting a lost command, on a File tab whose **Document** and
/// **pdfcer** groups were two and three scroll stops away. All three were worked
/// around with `session.maximize()` — a workaround that is fine for those three
/// and does nothing for the next check to meet this.
///
/// ⇒ The published *name* being a stability contract is right, and it is
/// exactly what made this survive: nothing renamed, nothing failed to compile,
/// and the one caller that would have noticed reported the application as
/// broken instead.
///
/// # What it does now
///
/// 1. Look where the band is standing. Return immediately if the item is
///    there — the overwhelmingly common case, and it costs one trace read.
/// 2. Otherwise **rewind** the band to its first group, so the search covers
///    the whole row rather than the part of it to the right of wherever a
///    previous call left it. ★ This is what makes the helper *idempotent*:
///    without it, a check that asks for an item in the last group and then for
///    one in the first would be told the second does not exist.
/// 3. Walk left to right one stop at a time. At each stop, look on the band,
///    then open each **collapsed** group in turn and look inside it.
/// 4. Give up only when the right arrow has retired — the band is showing its
///    last group — and rewind before answering, so a `None` leaves the ribbon
///    where it was found.
///
/// # Why this returns the rect rather than clicking
///
/// Because *"where is it"* and *"press it"* are different decisions, and some
/// callers want to measure a control rather than invoke it. On success the band
/// is left **scrolled to where the item is visible** and a collapsed group's
/// popup is left **open**, which is what a caller that is about to click wants;
/// a caller that is not can dismiss the popup with Escape.
///
/// # Errors
///
/// If the trace cannot be read, or a click cannot be delivered.
pub fn declared_or_in_overflow(
    session: &Session,
    driver: &crate::input::Driver,
    ui_rect: &str,
    name: &str,
) -> Result<Option<LRect>> {
    Ok(search_the_band(session, driver, ui_rect, name)?.0)
}

/// **How far [`search_the_band`] had to go** to answer.
///
/// # ★★★ Why the search is instrumented at all
///
/// Because otherwise the fix to the one-click bug is **unfalsifiable from
/// outside**. Every existing caller asks a yes/no question — *is the command
/// reachable* — and on a correct build the answer is `Some` whether the helper
/// scrolled once or five times. A check written against that answer alone would
/// pass just as happily on the broken build for any command that happens to sit
/// one stop past the fold, which is most of them.
///
/// `scrolls` is the number the assertion wants: a run that found the item after
/// **two or more** stops is a run the single-click implementation could not have
/// completed. See `crate::checks::band_scroll`.
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
    /// ★★★ With [`Self::scrolls`], this is the pair that says *"the old
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
///
/// The whole search lives here and the plain form is a one-line wrapper, so
/// there is exactly one implementation and the instrumented answer describes
/// the code every caller runs. Two implementations would be two behaviours the
/// day one of them was edited.
///
/// # Errors
///
/// If the trace cannot be read, or a click cannot be delivered.
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
///
/// ★★★ A COLLAPSED GROUP IS A THIRD PLACE A COMMAND CAN BE, and until
/// 2026-08-26 the caller knew about two.
///
/// S3 gave the band a middle rung: when it runs short of width a whole group
/// folds into a single captioned button, its items reachable through that
/// button's popup. They are on the ribbon, they are one click away — and they
/// **publish no rect**, exactly like a scrolled-away group's, which is why this
/// search exists at all.
///
/// `export_dxf_writes_the_pages_geometry` went red on it and the failure read
/// as a lost command: *"the File tab declares no
/// `ribbon.item.file.export_dxf`, on the band or in the overflow."* The command
/// was not lost. The harness was asking about two of the three places it could
/// be, and the window the harness opens is 1,100 pt wide — precisely the width
/// at which the Export group collapses.
///
/// ★ It is checked at **every scroll stop**, not once, because collapsing
/// happens before scrolling: a group that scrolls into view can arrive already
/// collapsed, and looking for its items on the band would find nothing.
///
/// # Errors
///
/// If the trace cannot be read, or a click cannot be delivered.
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
///
/// The left arrow is drawn **only** while the band is scrolled off its start
/// (`egui-shell`'s `ribbon::band`: `if scrolled > 0`), so its absence is the
/// termination condition rather than a count this helper would have to keep in
/// step with the application's.
///
/// ★ On a band that is already at position zero this costs one trace read and
/// no clicks, which is why `declared_or_in_overflow` can call it
/// unconditionally.
///
/// # Errors
///
/// If the trace cannot be read, or a click cannot be delivered.
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
///
/// A collapsed group publishes `ribbon.group.<tab>.<id>.collapsed` — a
/// deliberately distinct name from the expanded `ribbon.group.<tab>.<id>`, so
/// that a check can tell *"on the band, collapsed"* from *"on the band"* and
/// from *"gone"*. This is the consumer that distinction was created for.
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

/// The dominant colour of a declared region in a capture — a control's fill.
///
/// `None` when the region resolved to no pixels, which means the application
/// declared it outside its own client area. That is a finding rather than a
/// measurement and the caller reports it as one.
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
///
/// The derivation is [`crate::checks::markup_rectangle`]'s `MIN_PRESSED_DELTA`
/// and is not restated here, because restating it would create two accounts of
/// one measurement that can drift apart. In summary, and only as a pointer
/// into that argument:
///
/// * `egui`'s stock light palette — which is what the built binary actually
///   paints with, because nothing in `crates/pdfcer-gui` calls
///   `egui_shell::theme::Theme::apply` — separates unpressed `#E5E5E5` from
///   pressed `#90D1FF` by **85**;
/// * `egui-shell`'s `quiet` preset, if it were installed, would separate them
///   by **39**;
/// * two identically filled controls in a lossless BGRA capture differ by
///   **0**, not by a small number.
///
/// Twelve sits above zero and a factor of three below the smaller of the two
/// real differences, so the verdict is the same whichever palette is in force.
///
/// A channel difference rather than a contrast ratio, because both pairs are
/// near-equal in luminance (about 1.5:1 and 1.3:1) and would therefore be
/// called *identical* by [`crate::pixels::AA_LARGE`]. Contrast answers "can
/// this be read"; the question here is "is this a different colour".
pub const MIN_PRESSED_DELTA: u16 = 12;

/// **Click a mode segment and confirm the shell saw the click.**
///
/// The move both new checks make repeatedly, with the counting that makes it
/// honest folded in.
///
/// # Why the count rather than "is there a line for this mode?"
///
/// Because a run switches modes more than once, and a check that asked
/// "did the shell ever report `mode=read`?" would be satisfied by a click it
/// made a minute ago. The event is emitted on every segment click — including
/// a click on the already-selected segment — so the number of them is the only
/// thing that distinguishes *this* click from the previous one.
///
/// # Why a failure here is a SKIP and not a FAIL
///
/// Same reason [`crate::checks::find_bar`]'s chord control exists: a check
/// that could not deliver a click has learned nothing about the application,
/// and naming a feature as the culprit when nothing was ever clicked at it is
/// worse than no check at all. The two readings — pointer injection is not
/// reaching this window, or the shell diagnostic switch did not reach the
/// process — are both stated, and this function declines to choose between
/// them.
///
/// # Errors
///
/// * the application declared no rect for the segment, so there is nothing to
///   aim at (the reason lists the segments it *did* declare);
/// * the segment was declared at no usable size;
/// * the pointer could not be driven;
/// * the shell traced no new [`MODE_EVENT`] for this mode after the click.
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

// ★ The reachability family lives in [`super::reaching`] and is re-exported
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
///
/// # ★★★ Why this exists: a keystroke is not a harness primitive with a raised
/// panel on screen
///
/// A check that authors something with a markup or measure tool has to disarm
/// before it can select what it just made — with a tool armed, a click on the
/// page is a PICK rather than a selection. Every such check used to do that
/// with a key: `V` (the `view.tool_select` chord) or Escape.
///
/// On 2026-08-28 that cost `the_line_weight_switch_reaches_the_resize` three
/// failed runs out of six, and the measurements are worth keeping:
///
/// | attempt | result |
/// |---|---|
/// | `V` | **never arrived** — no invocation traced at all |
/// | one Escape | arrived sometimes |
/// | five Escapes, polling for the region | attempt 1, or not in five |
///
/// ⇒ **A chord is routed through whatever holds keyboard focus**, and that
/// check raises a dock panel by construction. The failure's shape is the
/// dangerous part: `V` produced *no line anywhere*, so the check reported the
/// Tool panel as drawing the wrong block when nothing had ever reached the
/// application. A harness primitive that can fail silently will eventually be
/// believed.
///
/// A click does not depend on focus, it is this suite's most exercised
/// primitive, and it has an oracle — the shell writes
/// `ribbon-command-invoked id=view.tool_select`.
///
/// # ★★ Why `view.tool_select` specifically, and not its neighbours
///
/// `app::dispatch`'s arm calls `canvas::tool::arm::select`, a plain write into
/// tool memory. Its two neighbours on the same band — `view.tool_hand` and
/// `view.tool_text` — are **toggles** (`toggle_hand`, and its text twin), so a
/// second press of either flips back. This is the one control on that row that
/// cannot be wrong about its own state, which is what makes the step
/// deterministic rather than merely more reliable.
///
/// # ★ Why it returns `bool` rather than failing
///
/// Because what an unavailable route *means* is the caller's to say, and the
/// two callers disagree. A check whose subject sits inside a raised panel must
/// SKIP — falling back to the chord there would restore exactly the flake this
/// removes, silently. A check that merely needs the pen down, and that has
/// been passing on `V` for weeks, should press `V` and say so. Neither verdict
/// belongs here; both messages do belong in their own check.
///
/// `false` means *the pointer route was not available or did not land* — the
/// View tab is missing, the control is on none of the band, a collapsed group
/// or the overflow, or the click produced no invoke. Each of those writes a
/// note before returning, so a caller's own message never has to guess which.
///
/// # Errors
///
/// If the trace cannot be read or the pointer cannot be driven. Note that a
/// *missing* control is not an error — it is `Ok(false)`.
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
///
/// ★ Four rather than two, and the number comes from a measurement rather than
/// from taste: `scale_switch`'s header records a bare `V` **arriving zero times
/// in six runs** with a dock panel raised. A non-arrival on this machine is not
/// a rare coincidence to be papered over with one retry — it is a routine
/// outcome of a window-manager transition landing between the raise and the
/// key — so a loop that gives up at two would report "not delivered" on runs
/// where a third press would have landed, and the suite would learn nothing on
/// those runs either.
///
/// It is bounded, and small, because a press is not free of consequence: see
/// this function's contract about repeatability below.
pub const PRESS_TRIES: usize = 4;

/// **Press a key until the application's own trace shows it was heard**, and
/// answer whether it ever was.
///
/// Returns `true` as soon as any line named in `evidence` appears that was not
/// there before the first press, and `false` after [`PRESS_TRIES`] presses with
/// no new line.
///
/// # ★★★ THE RULE THIS ENCODES
///
/// > **Nothing measured after a press is evidence about the program until the
/// > press is shown to have arrived.**
///
/// `Driver::press` answers `Ok(())` when the **keystroke was sent**. It refuses
/// with no target window and it raises the target first — both real guards, and
/// neither of them is the statement *"the application processed that key"*.
/// Between the two lie a foreground transition, an egui frame boundary, and, on
/// this machine, a measured failure rate that is not small: `scale_switch`'s
/// header records a bare `V` arriving **zero times in six** runs with a dock
/// panel raised.
///
/// A check that presses once, measures nothing, and reports a defect has
/// reported a defect about a program it never spoke to. That is strictly worse
/// than reporting nothing, because it is a confident accusation naming a
/// specific line — and this suite has now produced one of those (`annot_delete_gate`
/// phase D on 2026-08-29 said *"the keystroke did not reach `canvas::keys` at
/// all"* about a keystroke whose effect was four lines further up the same
/// trace). ⇒ A press that cannot be shown to have landed is a **SKIP**.
///
/// # ★★ The caller owes two things, and both are contracts rather than advice
///
/// 1. **`evidence` must name every line the key could produce**, including the
///    ones that mean the program is wrong. A list containing only the
///    good outcome turns a broken build into "the key did not arrive", which is
///    the same false negative in a new place. `annot_delete_gate` lists three:
///    the decline it wants, the funnel line that means the gate was walked past,
///    and the funnel's refusal line that means it was walked past and the engine
///    caught it.
/// 2. **The key must be safe to press more than once.** Every press after the
///    one that lands is suppressed — the loop returns immediately — but presses
///    before it are real and may repeat. Delete on a document that refuses every
///    delete is safe by construction; Delete on a document that performs them is
///    not, and such a caller must press once and take the SKIP.
///
/// # Relation to `read_mode_chrome::press_until_invoked`
///
/// That is this function's **click** half — same rule, same shape, and it
/// additionally re-reads the control's rect between attempts because a click
/// needs a target and a ribbon relays itself out when the window resizes. A key
/// has no rect, so there is nothing to re-read and the two cannot share a body
/// without one of them carrying a parameter it ignores. They are kept as a pair
/// deliberately, and this module's own rule applies to folding them: *"a third
/// copy is the point at which folding becomes worth doing on its own rather
/// than in the change that happens to need it."*
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
    ///
    /// The same property [`crate::checks::markup_rectangle`] pins for its own
    /// copy. Pinned twice on purpose: the two copies exist to be independent,
    /// and an independent copy with no test of its own is not independent
    /// evidence, it is an untested duplicate.
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
    ///
    /// If a future prefix change made one a prefix of the other, this test is
    /// what says so — and the symptom otherwise would be a check that reads a
    /// `ribbon-command-invoked` that is not there, or misses one that is.
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
    ///
    /// The second assertion is the one that matters: `AA_LARGE` is 3.0 and
    /// these fills are 1.5:1 and 1.3:1 apart, so a check written against the
    /// harness's usual legibility oracle would report "no difference" about a
    /// control that is visibly blue.
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
