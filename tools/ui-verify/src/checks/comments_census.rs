//! `checks::comments_census` — reading the Comments panel's count **honestly**,
//! for every check that uses it as an oracle.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/comments_census.md`.

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
