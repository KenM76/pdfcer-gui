//! The canvas object menu's **Redact selection** row, and the bounds it asks for.
//!
//! # Why this is a module and not a function inside one check
//!
//! Two checks press this row, and they differ in exactly one thing: the
//! selection standing when they press it.
//!
//! - `redacting_a_clicked_chunk_marks_only_that_chunk` presses it twice, on a
//!   whole block and then on one line, and compares the two boxes.
//! - `marking_two_chunks_makes_one_mark_and_two_regions` presses it on one line
//!   and then on a set of two, and compares the counts.
//!
//! The gesture between those presses — right-click, confirm the OBJECT menu
//! resolved, find the row's published rect, click its centre, read the verb's
//! own trace line — is a hundred lines of refusal prose, every sentence of which
//! names a specific way the route can be broken and what to suspect first. A
//! second copy would be a second place for the route to drift, and the drift
//! would be invisible: both copies would keep passing, against different
//! programs.
//!
//! # What a caller gets, and what it must still decide
//!
//! [`mark_through_the_menu`] returns [`Marked`] — the **number of quads** the
//! verb built and their **union**. It asserts nothing about either. Whether one
//! quad is right or two are is the caller's subject, and the two callers want
//! opposite answers, so an assertion here would have to be satisfied by both
//! and would therefore measure neither.
//!
//! ⚠ **The union is not the quads.** The trace publishes one `bbox=` for the
//! whole request, so a check reading it can prove *how many* regions were built
//! and *what they span*, never that each one is the box of the thing that was
//! selected. The end-to-end proof is the apply report, which lists the text it
//! will destroy region by region.
//!
//! # The three-way return, and why it is not two
//!
//! `Err(..)` is a finding about the **run** — input disabled, a window that
//! never came up, a trace that cannot be read — and every caller reports it as
//! SKIPPED. `Ok(Err(..))` is a finding about the **program** and is a failure.
//! `Ok(Ok(..))` is a measurement. Collapsing the first two would make a broken
//! harness indistinguishable from a broken build, which is the reading this
//! project has been wrong about most often.

use crate::checks::driving::{declared, declared_names, list};
use crate::coords::ScreenPoint;
use crate::error::Result;
use crate::input::Driver;
use crate::launch::Session;

/// The shell's line for the verb, carrying `quads=` and `bbox=`.
pub(super) const REQUESTED: &str = "redact-mark-selection-requested"; // ui-text-exempt: a trace event name

/// What the canvas writes on every frame carrying a secondary click.
const MENU_EVENT: &str = "canvas-menu"; // ui-text-exempt: a trace event name

/// The context the object menu declares.
const OBJECT_CONTEXT: &str = "canvas.object"; // ui-text-exempt: a trace token

/// The context-menu row these checks press — **O53's half of the row**.
const ROW_REGION: &str = "menu.item.canvas.object.edit.redact_selection";

/// Everything the object menu publishes, for a failure that can name what IS
/// there rather than only what is not.
const ROW_PREFIX: &str = "menu.item.canvas.object.";

/// How far outside another rectangle a rectangle may fall and still be inside
/// it, in points.
///
/// The trace publishes one decimal place, and the boxes compared by callers are
/// computed from the same outlines, so this absorbs rounding and nothing else.
/// A real containment failure is tens of points, not tenths.
const CONTAINMENT_SLACK_PT: f64 = 1.0;

/// A rectangle read off a `bbox=` field, in PDF user space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Bounds {
    llx: f64,
    lly: f64,
    urx: f64,
    ury: f64,
}

impl Bounds {
    /// Parse `llx,lly,urx,ury`. `None` for `bbox=none`, a short field or any
    /// component that is not a number — all of which mean the same thing to
    /// every caller: **this line cannot say what was marked.**
    pub(super) fn parse(field: &str) -> Option<Self> {
        let mut parts = field.split(',');
        let mut next = || parts.next()?.parse::<f64>().ok();
        let (llx, lly, urx, ury) = (next()?, next()?, next()?, next()?);
        if parts.next().is_some() {
            return None;
        }
        Some(Self { llx, lly, urx, ury })
    }

    pub(super) fn height(self) -> f64 {
        self.ury - self.lly
    }

    /// Whether `self` lies inside `outer`, allowing [`CONTAINMENT_SLACK_PT`].
    pub(super) fn inside(self, outer: Self) -> bool {
        self.llx >= outer.llx - CONTAINMENT_SLACK_PT
            && self.lly >= outer.lly - CONTAINMENT_SLACK_PT
            && self.urx <= outer.urx + CONTAINMENT_SLACK_PT
            && self.ury <= outer.ury + CONTAINMENT_SLACK_PT
    }
}

impl std::fmt::Display for Bounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.1},{:.1},{:.1},{:.1}",
            self.llx, self.lly, self.urx, self.ury
        )
    }
}

/// What one press of the redact row asked the document for.
#[derive(Debug, Clone, Copy)]
pub(super) struct Marked {
    /// How many separate regions the verb built — `quads=` on [`REQUESTED`].
    ///
    /// **The design's unit of disclosure.** One gesture over a set of lines is
    /// one mark holding one region per line, so this number is what separates
    /// a build that honours the set from one that unioned it into a single
    /// rectangle over text the operator never selected.
    pub(super) quads: usize,
    /// The union of those regions — `bbox=` on the same line.
    pub(super) bbox: Bounds,
}

/// Right-click at `at`, press the redaction row, and read what it requested.
///
/// `what` names the selection standing at the time, in the operator's terms
/// ("the whole block", "one line"), and is quoted in every refusal so a reader
/// knows which press failed without counting them.
///
/// `Ok(Err(..))` is a finding about the program; `Err(..)` is a finding about
/// the run and is reported as SKIPPED.
pub(super) fn mark_through_the_menu(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    at: ScreenPoint,
    what: &str,
) -> Result<std::result::Result<Marked, String>> {
    driver.right_click_at(at)?;
    session.settle(35);

    let trace = session.trace()?;
    let Some(menu) = trace.events(MENU_EVENT).last() else {
        return Ok(Err(format!(
            "THE RIGHT-CLICK ON {what} RESOLVED NO MENU AT ALL: no `{MENU_EVENT}` line after a \
             secondary click on the page. `canvas::menus::attach` writes that line on every frame \
             carrying a secondary click, so its absence means the click never reached the canvas \
             response. Trace: {}",
            session.trace_path().display()
        )));
    };
    let context = menu.get("context").unwrap_or_default();
    if context != OBJECT_CONTEXT {
        return Ok(Err(format!(
            "THE RIGHT-CLICK ON {what} RESOLVED `{context}`, NOT `{OBJECT_CONTEXT}`: `{}`.\n\
             A selection standing at either the Object or the Part rung is an OBJECT selection as \
             far as the menu is concerned, so the object menu is the one that must appear. \
             Resolving the view menu here means the secondary hit test lost the selection the \
             left click made. Trace: {}",
            menu.raw,
            session.trace_path().display()
        )));
    }

    let Some(row) = declared(&trace, ui_rect, ROW_REGION) else {
        return Ok(Err(format!(
            "★★★ THE REDACT ROW IS NOT IN THE CANVAS OBJECT MENU: no `{ROW_REGION}` region after \
             the menu opened on {what}. Rows it DID publish: {}.\n\
             Three readings, and all three are defects: `edit.redact_selection` is registered on \
             the Edit ribbon tab only, which is exactly what **O53** forbids; the row is drawn \
             but disabled, because a disabled command is dropped before it is drawn and \
             `selection.any` is not being set for this selection; or `MenuHost::attach_with` has \
             stopped supplying a rect sink, in which case no context-menu row anywhere in this \
             application can be pressed by a check. Trace: {}",
            list(&declared_names(&trace, ui_rect, ROW_PREFIX)),
            session.trace_path().display()
        )));
    };
    if !row.is_substantial() {
        return Ok(Err(format!(
            "`{ROW_REGION}` was published at {row:?}, which has no usable area — so the row \
             exists in the plan and was laid out to nothing. A click aimed at a degenerate \
             rectangle proves nothing, and this is itself the finding."
        )));
    }

    let mark = session.trace()?.mark();
    driver.click_at(session.frame()?.declared_center(row))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(line) = trace.last_after(REQUESTED, mark) else {
        return Ok(Err(format!(
            "★★★ THE ROW WAS PRESSED AND THE VERB DID NOT RUN: no `{REQUESTED}` line after \
             pressing the redact row on {what}.\n\
             ★★ Ask first whether the press dispatched at all. The menu dies on the pointer MOVE \
             if the canvas is choosing between two responses per frame — egui derives a popup's \
             identity from the response it was attached to — and the tell is the row's \
             `ui-rect-gone` lines arriving in the same frame as `canvas-pointer`, with no button \
             ever going down.\n\
             IF the verb WAS entered, the next suspect is the page filter: \
             `app::actions::redactsel::mark_selection` keeps only outlines whose page is the \
             current one, and a build that kept none writes \
             `redact-mark-selection-declined … reason=no-bounds` instead. Grep for it. Trace: {}",
            session.trace_path().display()
        )));
    };
    let Some(field) = line.get("bbox") else {
        return Ok(Err(format!(
            "★★★ THE VERB RAN AND DID NOT SAY WHAT IT MARKED: `{}` carries no `bbox=` field.\n\
             The count alone is written identically for a chunk-sized mark and a block-sized one, \
             which is the pair these checks exist to separate, so without the bounds there is \
             nothing here to measure. Somebody narrowed the trace line; widen it again.",
            line.raw
        )));
    };
    let Some(bbox) = Bounds::parse(field) else {
        return Ok(Err(format!(
            "★★ THE MARKED BOUNDS ARE NOT A RECTANGLE: `{}`.\n\
             `bbox=none` means the union of the marked rectangles was empty on a frame that went \
             on to build quads — which `mark_selection`'s own early return is supposed to make \
             unreachable. Anything else is a malformed field.",
            line.raw
        )));
    };
    // ★ The absence of `quads=` is a failure rather than a zero. A caller
    // asserting `quads == 2` against a defaulted zero would report "the verb
    // built one region" about a line that never said how many it built, and
    // send the next reader to `mark_selection`'s geometry instead of to the
    // trace line somebody narrowed.
    let Some(quads) = line.get_usize("quads") else {
        return Ok(Err(format!(
            "★★★ THE VERB RAN AND DID NOT SAY HOW MANY REGIONS IT BUILT: `{}` carries no usable \
             `quads=` field.\n\
             That count is the only published difference between one mark covering a set of \
             lines region by region and one mark unioned into a single rectangle over text the \
             operator did not select — and the second silently destroys the lines between them. \
             Without it there is nothing here to measure.",
            line.raw
        )));
    };
    Ok(Ok(Marked { quads, bbox }))
}
