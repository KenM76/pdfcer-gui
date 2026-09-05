//! `checks::comments_census` — reading the Comments panel's count **honestly**,
//! for every check that uses it as an oracle.
//!
//! # ★★★ Why this module exists: two checks, one copied defect, two false
//! reports — 2026-09-05
//!
//! `save_copy_round_trip` and `undo_redo_round_trip` both use the Comments
//! panel's per-frame census as their proof that an annotation reached the
//! document. On the driven sweep of 2026-09-05 **both failed, in the same
//! words**, and the sweep report treated them as *"one application defect with
//! two independent witnesses"*:
//!
//! > *"THE COMMENTS PANEL DOES NOT SEE THE ANNOTATION THAT WAS JUST AUTHORED:
//! > it listed 12 before the drag and 12 after it. The engine traced
//! > `add-markup`, so the annotation IS on the session."*
//!
//! The panel was fine. The witnesses were not independent — each carried a
//! **copy** of the same eight-line helper:
//!
//! ```ignore
//! fn listed(trace: &Trace) -> Option<usize> {
//!     trace.last(COMMENTS_EVENT)?.get_usize(LISTED_FIELD)
//! }
//! ```
//!
//! `Trace::last` searches the whole capture. A docked pane that is not the
//! front tab **draws nothing and therefore traces nothing** — this project's
//! own `RESUME.md` records that as a finding — so when a persisted
//! `userdata/layout.ron` put Document properties in front of Comments, the
//! panel fell silent three hundred frames before the drag and its last census
//! stood for ever. Both checks read that fossil, twice, and subtracted it from
//! itself.
//!
//! ⇒ ★★ **Two checks sharing a helper can share a defect; two checks sharing a
//! COPY of a helper are worse, because the duplication is what makes the two
//! failures look like corroboration.** One module now, imported by both, so a
//! future repair cannot land in one and miss the other.
//!
//! # What a caller gets instead
//!
//! | | |
//! |---|---|
//! | [`Census::since`] | a census the panel published **after** a named cause, or `None` |
//! | [`refresh`] | the same, and if the panel is silent, **puts it back in front** and asks again |
//! | [`baseline`] | enter the mode, front the panel, and report the starting census |
//!
//! `None` is never folded into a number. *"The panel said nothing since the
//! edit"* and *"the panel said the same number"* are different verdicts about
//! different subjects — the first is a layout fact and reports SKIP, the second
//! is a defect and reports FAIL — and the whole of the 2026-09-05 misreport was
//! the first being printed as the second.
//!
//! # ★ What a census asserts, and what it does not
//!
//! The line carries no annotation identity, so a caller cannot name the object
//! it is looking for. What it can do, and what [`Census::describes_one_more`]
//! does, is assert the **shape** of the change a freshly drawn markup makes:
//!
//! * `listed` rises by exactly one — a row appeared;
//! * `with_note` does **not** move — the new row has no `/Contents`, because
//!   nothing in this shell can write words onto a shape at the moment it is
//!   drawn;
//! * `authors` does **not** move — nor a `/T`.
//!
//! A build whose census moved because something *else* changed (a widget
//! stopped being excluded, a reply was counted, a ce dimension was
//! reclassified) fails that conjunction, which is what stops the assertion
//! being the vacuous *"a number went up"*. The caller states the fixture's
//! starting census in its own report so the arithmetic is auditable from the
//! output alone.
//!
//! # ★★★ Driven and falsified, 2026-09-05 — four runs, and what each settled
//!
//! | run | condition | result |
//! |---|---|---|
//! | 1 | `userdata/` cleared, unmodified build | `save_copy_round_trip` **PASS**, 12 → 13 |
//! | 2 | the same | `undo_redo_round_trip` **PASS**, 12 → 13 → 12 → 13 |
//! | 3 | **the hostile layout seeded by hand** — `mode:review`'s right stack rewritten with `active: 4`, so `file.document_properties` is the front tab and Comments is in the overflow, which is the exact state of the sweep that produced the two false reports | both **PASS**, each printing *"the Comments panel published no census … Bringing it forward"* and then *"the panel came forward on a press of `markup.comments`"* |
//! | 4 | **a planted application defect** — the panel's listing truncated to the row count of its first frame, a census frozen on something that never moves | both **FAIL**, naming the arithmetic: *"`listed` 12 → 12, `with_note` 12 → 12, `authors` 12 → 12 (expected 12 → 13, 12 → 12, 12 → 12)"*, exit 1 |
//!
//! Run 3 is the one that matters most, because it is the only one that could
//! have shown the repair to be cosmetic. Run 4 is what stops the repair being
//! *"make the check pass"*: with the panel genuinely blind, both checks still
//! say so, and they say it about the panel rather than about the save or the
//! undo.
//!
//! ★ Run 3 also corrected the repair **while it was being made**. The first
//! version anchored the baseline on a mark taken before the mode click, and
//! that was still wrong: a launch restores its remembered mode, the harness
//! clicks a segment out from under it, and a perfectly fresh census from the
//! *previous* mode satisfies the anchor. The driven run showed the baseline
//! coming from a Read frame while the check believed it had measured Review.
//! The anchor is `mode-changed … to=<mode>` because of that run and not
//! because of any reasoning that preceded it.
//!
//! # ★★ `filtered=` is checked, and the reason is this panel's own rule
//!
//! `panels::comments` gained filtering by author, type and has-words on
//! 2026-09-05. Its founding discipline is that *"nothing is silently
//! omitted"*, and a filter is an omission the **operator** caused. The panel
//! states it on screen; since the same day it states it on the trace as
//! `filtered=1`, and every function here refuses a filtered census rather than
//! comparing it. A narrowed list is not a census of the document, and reading
//! one as though it were is exactly how a check reports a document as having
//! lost annotations it still has.

use crate::checks::driving::{
    self, ITEM_PREFIX, TAB_EVENT, declared, declared_names, list, shell_trace,
};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::Trace;

/// `comments-panel pages=… listed=N …` — one line per frame the panel draws.
pub const EVENT: &str = "comments-panel";

/// Rows [`crate::checks`]' subject documents call "the census" — every
/// annotation the panel found, **before** the operator's filter.
const LISTED: &str = "listed";

/// Rows whose `/Contents` (or ce-dimension description) carries words.
const WITH_NOTE: &str = "with_note";

/// Rows carrying a `/T`.
const AUTHORS: &str = "authors";

/// Annotations left out by editorial rule — widgets, pop-ups, `/TrapNet`.
const EXCLUDED: &str = "excluded_total";

/// `1` while the operator's filter is narrowing the list.
const FILTERED: &str = "filtered";

/// The Comments panel's dock tab, when the stack is wide enough to draw it.
const DOCK_TAB: &str = "dock.tab.markup.comments";

/// The Comments panel's ribbon control. `app/panels.rs` makes it a **show**,
/// not a toggle, so pressing it when the panel is already up is a no-op — which
/// is what makes it safe to press without first knowing the state.
const COMMAND: (&str, &str) = ("ribbon.item.markup.comments", "markup.comments");

/// The tab carrying that control.
const MARKUP_TAB: (&str, &str) = ("ribbon.tab.markup", "markup");

/// `mode-changed from=… to=… remembered=… panels=…` — the **application's**
/// line, written by `crate::app::modes` when a mode's arrangement is applied.
///
/// Not the shell's `ribbon-mode-selected`, which says a segment was pressed.
/// The distinction matters here: the anchor has to be the moment the dock this
/// check reads was rearranged, not the moment the click landed.
const MODE_CHANGED_EVENT: &str = "mode-changed";

/// One frame's reading of the Comments panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Census {
    /// Where in the capture it was published — so a caller can anchor the
    /// *next* read on this one.
    pub lineno: usize,
    /// Annotations the panel found, before any filter.
    pub listed: usize,
    /// How many of them carry words.
    pub with_note: usize,
    /// How many of them carry a `/T`.
    pub authors: usize,
    /// How many the panel left out by editorial rule.
    pub excluded: usize,
    /// The line as the application wrote it, for the report.
    pub raw: String,
}

impl Census {
    /// The newest census the panel published **after** `after`, or `None` if it
    /// has said nothing since.
    ///
    /// A filtered census is `Err`, never `Ok`: see this module's header.
    pub fn since(trace: &Trace, after: usize) -> Result<Option<Self>> {
        let Some(line) = trace.last_after(EVENT, after) else {
            return Ok(None);
        };
        // Absent on a build older than 2026-09-05, which is the only reason
        // this is not a hard requirement: an old capture read by a new harness
        // reports "not narrowing", which is what it was.
        if line.get_usize(FILTERED).unwrap_or(0) != 0 {
            return Err(Error::new(format!(
                "the Comments panel's filter is narrowing the list, so `{LISTED}` is a count of \
                 the operator's current selection rather than a census of the document, and no \
                 comparison against it means anything. Line: `{}`. Nothing in this harness sets \
                 that filter, so it came from a persisted panel state beside the binary — clear \
                 `userdata/` and run again.",
                line.raw
            )));
        }
        Ok(Some(Self {
            lineno: line.lineno,
            listed: line.get_usize(LISTED).ok_or_else(|| {
                Error::new(format!(
                    "the Comments panel traced a census with no `{LISTED}` field, so this check \
                     has no oracle: `{}`",
                    line.raw
                ))
            })?,
            with_note: line.get_usize(WITH_NOTE).unwrap_or(0),
            authors: line.get_usize(AUTHORS).unwrap_or(0),
            excluded: line.get_usize(EXCLUDED).unwrap_or(0),
            raw: line.raw.clone(),
        }))
    }

    /// Refuse a fixture whose annotations the panel excludes.
    ///
    /// Carried here from the two checks that each had their own copy of it. The
    /// arithmetic every caller performs is *"the census moves by exactly one"*,
    /// and on a drawing full of form fields that arithmetic is measuring the
    /// panel's editorial rules rather than the caller's subject.
    pub fn require_a_clean_fixture(&self, what: &str, subject: &str) -> Result<()> {
        if self.excluded == 0 {
            return Ok(());
        }
        Err(Error::new(format!(
            "the Comments panel excluded {} annotation(s) on {what} — widgets, pop-ups or trap \
             nets, which it leaves out by editorial rule. This check's verdict is that `{LISTED}` \
             moves by exactly one, and on a document with excluded annotations that arithmetic is \
             measuring the panel's rules rather than {subject}. Point --pdf at a drawing without \
             form fields.",
            self.excluded
        )))
    }

    /// Is this census `before` plus **one freshly authored, wordless,
    /// unsigned** markup?
    ///
    /// See the module header for why all three clauses are asserted and not
    /// just the first.
    #[must_use]
    pub fn describes_one_more(&self, before: &Self) -> bool {
        self.listed == before.listed + 1
            && self.with_note == before.with_note
            && self.authors == before.authors
    }

    /// Why [`Census::describes_one_more`] said no, in the operator's arithmetic.
    #[must_use]
    pub fn disagreement(&self, before: &Self) -> String {
        format!(
            "`{LISTED}` {} → {}, `{WITH_NOTE}` {} → {}, `{AUTHORS}` {} → {} (expected {} → {}, {} \
             → {}, {} → {}: one more row, carrying neither words nor an author, because nothing \
             in this shell writes either at the moment a shape is drawn).",
            before.listed,
            self.listed,
            before.with_note,
            self.with_note,
            before.authors,
            self.authors,
            before.listed,
            before.listed + 1,
            before.with_note,
            before.with_note,
            before.authors,
            before.authors,
        )
    }
}

/// Read a census published after `after`, **bringing the panel back to the
/// front first if it has gone quiet**.
///
/// `Ok(None)` means the panel could not be put on screen at all, which every
/// caller reports as SKIP: a check that could not see its own oracle has
/// learned nothing about its subject.
///
/// # The two routes, in the order they are tried
///
/// 1. **Its dock tab**, `dock.tab.markup.comments`, when the stack is wide
///    enough to draw one. The cheapest gesture and the one an operator makes.
/// 2. **Its ribbon control**, `markup.comments`, which is a *show* rather than
///    a toggle. This is the route that survives the case that produced the
///    2026-09-05 misreport: a stack whose tab strip has overflowed publishes no
///    `dock.tab.*` region for the panels in the overflow menu, and a menu's
///    contents are not published as regions at all, so route 1 has nothing to
///    aim at and route 2 is the only one left.
///
/// ⚠ Route 2 needs the Markup tab, and an earlier version of this reasoning
/// recorded that on a 2384 pt-wide sheet the Markup band overflowed and the
/// Comments group was never drawn. Measured again on 2026-09-05 at a 1400 pt
/// window that is **no longer true** — `ribbon.item.markup.comments` is
/// declared — but the fallback still reports what it found rather than
/// asserting, because the claim is about band geometry and band geometry moved
/// twice this week.
pub fn refresh(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    after: usize,
    report: &mut CheckReport,
) -> Result<Option<Census>> {
    if let Some(census) = Census::since(&session.trace()?, after)? {
        return Ok(Some(census));
    }

    report.note(
        "the Comments panel published no census — it is mounted but not the front tab of its \
         stack, and a dock draws only its active tab. Bringing it forward before reading."
            .to_owned(),
    );

    // Route 1 — its own tab, if the strip is drawing one.
    let trace = session.trace()?;
    if let Some(rect) = declared(&trace, ui_rect, DOCK_TAB) {
        driver.click_at(session.frame()?.declared_center(rect))?;
        session.settle(14);
        if let Some(census) = Census::since(&session.trace()?, after)? {
            report.note(format!("the panel came forward on a click of `{DOCK_TAB}`"));
            return Ok(Some(census));
        }
    }

    // Route 2 — its command, which shows rather than toggles.
    let trace = session.trace()?;
    let Some(tab) = declared(&trace, ui_rect, MARKUP_TAB.0) else {
        report.note(format!(
            "no `{}` region either, so the ribbon route is unavailable. Tabs declared: {}.",
            MARKUP_TAB.0,
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ));
        return Ok(None);
    };
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    if shell_trace(session)?
        .events(TAB_EVENT)
        .all(|l| l.get("tab") != Some(MARKUP_TAB.1))
    {
        report.note(format!(
            "the click on `{}` produced no `{TAB_EVENT} tab={}`.",
            MARKUP_TAB.0, MARKUP_TAB.1
        ));
        return Ok(None);
    }

    let trace = session.trace()?;
    let Some(rect) = declared(&trace, ui_rect, COMMAND.0) else {
        report.note(format!(
            "the Markup tab is active and declares no `{}`, so the panel cannot be raised from \
             the band on this build. Controls declared: {}.",
            COMMAND.0,
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        ));
        return Ok(None);
    };
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(18);
    let census = Census::since(&session.trace()?, after)?;
    if census.is_some() {
        report.note(format!(
            "the panel came forward on a press of `{}`",
            COMMAND.1
        ));
    }
    Ok(census)
}

/// Enter `mode`, bring the Comments panel forward, and report the census it
/// publishes there.
///
/// The baseline every later comparison is measured against, and the reason it
/// is a function rather than four lines at each call site: the two counts a
/// round-trip check compares have to be produced by the identical sequence, or
/// the comparison at the end is between two different measurements.
///
/// # ★★★ The anchor is the application's own `mode-changed` line
///
/// The first repair of 2026-09-05 took the anchor **before** the mode click,
/// which is one step better than reading the whole capture and still wrong: a
/// launch restores its remembered mode, the harness clicks Read's segment out
/// from under it, and the panel publishes a perfectly fresh census *in the
/// wrong mode* between the anchor and the click. Driven on a seeded hostile
/// layout the same afternoon, that is exactly what happened — the baseline came
/// from a Read frame while the check believed it had measured Review.
///
/// So the anchor is `mode-changed … to=<mode>`, written by
/// `crate::app::modes` at the moment the arrangement is applied. Every census
/// after it was drawn by the dock this check is about, and nothing before it
/// can be mistaken for one.
///
/// ⚠ Falls back to a pre-click mark if the application traced no such line,
/// and **says so in the report** rather than silently weakening the anchor: a
/// mode that changed without announcing it is a finding of its own, and
/// `click_mode_segment` has already proved the *shell* saw the click.
pub fn baseline(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    mode: &str,
    what: &str,
    subject: &str,
    report: &mut CheckReport,
) -> Result<Census> {
    let before_click = session.trace()?.mark();
    driving::click_mode_segment(session, driver, ui_rect, mode)?;
    session.settle(16);

    let trace = session.trace()?;
    let anchor = match trace
        .events(MODE_CHANGED_EVENT)
        .filter(|l| l.get("to") == Some(mode))
        .last()
    {
        Some(l) => l.lineno,
        None => {
            report.note(format!(
                "the application traced no `{MODE_CHANGED_EVENT} … to={mode}` line, so the \
                 census below is anchored on the frame before the mode click rather than on the \
                 mode change itself — a weaker anchor, stated rather than assumed"
            ));
            before_click
        }
    };

    let census = refresh(session, driver, ui_rect, anchor, report)?.ok_or_else(|| {
        Error::new(format!(
            "the Comments panel traced no `{EVENT}` line for {what} after the switch to `{mode}`, \
             and neither its dock tab nor its ribbon control could raise it — so this check has no \
             oracle. It is mounted (`app::modes::defaults`' `{mode}` arrangement puts it first in \
             the right stack), which makes this a LAYOUT fact rather than a panel one: a persisted \
             `userdata/layout.ron` beside the binary can name a different active tab, and a stack \
             whose strip has overflowed hides the rest behind a menu whose contents are not \
             published as regions. Clear `userdata/` and run again. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    census.require_a_clean_fixture(what, subject)?;
    report.note(format!(
        "{what}: the Comments panel lists {} annotation(s) — `{}`",
        census.listed, census.raw
    ));
    Ok(census)
}

#[cfg(test)]
mod tests {
    use super::{Census, EVENT};
    use crate::trace::Trace;

    const PREFIX: &str = "pdfcer-diag";

    fn census(listed: usize, with_note: usize, authors: usize) -> String {
        format!(
            "{PREFIX} {EVENT} pages=1 listed={listed} with_note={with_note} authors={authors} \
             excluded_total=0 filtered=0 shown={listed}"
        )
    }

    #[test]
    fn a_census_published_before_the_cause_does_not_count() {
        let t = Trace::parse(
            &format!("{}\n{PREFIX} add-markup page=0", census(12, 12, 12)),
            PREFIX,
        );
        let cause = t.last("add-markup").unwrap().lineno;
        assert_eq!(
            Census::since(&t, cause).unwrap(),
            None,
            "the panel went quiet at the edit; reading its last line would report 12 as though it \
             were a fresh measurement"
        );
    }

    #[test]
    fn a_fresh_census_is_read_with_all_three_counts() {
        let t = Trace::parse(
            &format!("{PREFIX} add-markup page=0\n{}", census(13, 12, 12)),
            PREFIX,
        );
        let cause = t.first("add-markup").unwrap().lineno;
        let c = Census::since(&t, cause).unwrap().expect("a fresh census");
        assert_eq!((c.listed, c.with_note, c.authors), (13, 12, 12));
    }

    #[test]
    fn one_more_wordless_unsigned_row_is_the_only_accepted_shape() {
        let t = |s: &str| Trace::parse(s, PREFIX);
        let before = Census::since(&t(&census(12, 12, 12)), 0).unwrap().unwrap();

        let good = Census::since(&t(&census(13, 12, 12)), 0).unwrap().unwrap();
        assert!(good.describes_one_more(&before));

        let unchanged = Census::since(&t(&census(12, 12, 12)), 0).unwrap().unwrap();
        assert!(!unchanged.describes_one_more(&before));

        // The vacuity this guards: a census that moved by one for a reason
        // that has nothing to do with a shape being drawn.
        let wrong_kind = Census::since(&t(&census(13, 13, 13)), 0).unwrap().unwrap();
        assert!(
            !wrong_kind.describes_one_more(&before),
            "a row that arrived carrying words AND an author is not the rectangle this harness \
             drew, and accepting it would make the assertion `a number went up`"
        );
    }

    #[test]
    fn a_narrowed_panel_is_refused_rather_than_compared() {
        let t = Trace::parse(
            &format!(
                "{PREFIX} {EVENT} pages=1 listed=12 with_note=12 authors=12 excluded_total=0 \
                 filtered=1 shown=3"
            ),
            PREFIX,
        );
        let why = Census::since(&t, 0).expect_err("a filtered list is not a census");
        assert!(
            why.to_string().contains("filter"),
            "the refusal must name the filter, or the next reader spends an hour on the count: {why}"
        );
    }

    #[test]
    fn an_excluding_fixture_is_refused_by_name() {
        let t = Trace::parse(
            &format!(
                "{PREFIX} {EVENT} pages=1 listed=12 with_note=12 authors=12 excluded_total=4 \
                 filtered=0 shown=12"
            ),
            PREFIX,
        );
        let c = Census::since(&t, 0).unwrap().unwrap();
        let why = c
            .require_a_clean_fixture("the fixture", "the save")
            .expect_err("four excluded annotations make the arithmetic meaningless");
        assert!(why.to_string().contains('4'));
    }
}
