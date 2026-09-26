//! # `canvas::textedit::report` — what an edit report is worth telling anyone
//!
//! ## The seam
//!
//!
//! ## The rule this module applies, stated once
//!
//! | goes to | when |
//! |---|---|
//! | **the status row**, verbatim from `report.disclosures` | the engine wrote a sentence for an operator to read. pdfcer owns that wording; re-phrasing it here would be a second account of one fact, free to drift |
//! | **the diagnostic channel** | the fact is a number about a content stream. An operator cannot act on *"1,676 followers were repositioned"*; a driven check can, and a regression then names itself |
//! | **nowhere** | it restates something already visible on the page |
//!
//! The middle row is the one that earns this module. R8b rule 4 says a
//! disclosure must be in terms of what the operator can see — so a number about
//! operator counts is not a disclosure, it is *evidence*, and evidence belongs
//! where a check can read it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/report.md`.

/// **The forms THIS PAGE invokes directly** — the set that decides whether the
/// shared-content disclosure may name a remedy.
pub struct PageLevelForms(std::collections::BTreeSet<u32>);

impl PageLevelForms {
    /// Gather the set from the current page's decomposition.
    #[must_use]
    pub fn of(doc: &crate::app::state::OpenDoc) -> Self {
        Self(doc.page_objects().map_or_else(Default::default, |objects| {
            objects
                .page_objects()
                .leaves
                .iter()
                .filter_map(|leaf| leaf.containment.first())
                .map(|id| id.num)
                .collect()
        }))
    }

    /// The remedy sentence, if this edit's fan-out is one the shell can offer
    /// to undo.
    #[must_use]
    pub fn remedy_for(&self, report: &pdfcer_core::text_edit::EditReport) -> Option<String> {
        (report.form_invocations > 1
            && report
                .form_object
                .is_some_and(|form| self.0.contains(&form)))
        .then(crate::text::unshare::shared_content_remedy)
    }
}

/// **Which content stream the commit rewrote, and how many places paint it.**
pub fn trace_target(page: usize, run: usize, report: &pdfcer_core::text_edit::EditReport) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "edit-text-target page={page} run={run} form={} invocations={} pages={} \
             followers={} disposition={:?}",
            report
                .form_object
                .map_or_else(|| "none".to_owned(), |o| o.to_string()),
            report.form_invocations,
            report.form_pages.len(),
            // THE REFLOW'S REACH, on the channel because the engine asked
            // for it by name and because of what it caught.
            //
            //
            // The engine's request, verbatim: *"if you show one number from an
            // edit report beyond the disclosures, make it that one."* On
            // absolutely-placed content it should be `0`; a large number means
            // the edited "line" ran further than the line.
            //
            // It is on the trace and NOT on the status row, and that is a
            // decision. The operator cannot act on "1,676 followers were
            // repositioned" — it is a number about a content stream, and rule 4
            // says a disclosure must be in terms of what he can see. What he
            // CAN see is the page, and this shell's answer to "did the edit move
            // more than it should" is the render diff a driven check measures.
            // The number is here so that a check has a cheap oracle and a
            // regression names itself, rather than being found by him again.
            report.followers_repositioned,
            report.disposition,
        )
    });
}

/// **Where a line sits, and how many lines there are to count from** — one
/// reading of a page, taken either side of an edit.
#[derive(Clone, Copy)]
pub struct LineReading {
    /// The run's left edge in PDF user-space points, or `None` when the run
    /// carries no bounding box.
    pub left: Option<f64>,
    /// How many runs the page decomposed into for this reading.
    pub runs: usize,
}

/// **Read `run`'s left edge on `page`** — the number the operator's own report
/// is phrased in.
#[must_use]
pub fn read_line(doc: &crate::app::state::OpenDoc, page: usize, run: usize) -> Option<LineReading> {
    if !crate::diag::enabled() {
        return None;
    }
    let text = doc.provenance_page_text(page)?;
    Some(LineReading {
        left: text.runs.get(run).and_then(|r| r.bbox).map(|b| b.llx),
        runs: text.runs.len(),
    })
}

/// **Did the edit move the line it corrected?** — `OPERATOR_REQUESTS.md` O213,
/// on the channel a driven check can read.
///
/// # Why this number and not `followers_repositioned`
///
/// [`trace_target`] already publishes the reflow's reach, and the engine asked
/// for that one by name. It is the right number for *"how far did the edit
/// run"* and the **wrong** number for the question the operator actually asked,
/// which was *"the entire line shifts to the right"*.
///
/// On his own sheet the defective commit reports no repositioned followers and
/// moves the line a long way right; on every document this repository can
/// author, the same request shape reports two followers and moves the line
/// nothing. A check bound to the follower count would therefore have to know
/// which document it was looking at to know which value was the bad one. A
/// check bound to displacement does not: **zero is correct everywhere**, on
/// every producer, for every request shape.
///
/// ⚠ **This is not a disclosure and must never become one.** It is a distance
/// in content-stream coordinates, and R8b reserves the status row for what the
/// operator can see. What he can see is the page; the line holding still *is*
/// the disclosure.
///
/// # The line, and the three fields that stop it lying
///
/// ```text
/// edit-text-left-edge page=0 run=17 committed=yes runs=412/412 before=72.000 after=72.000 moved=+0.000
/// ```
///
/// `moved=+0.000` is this check's PASS value, and three separate things could
/// produce it without the line having held still:
///
/// | field | what it rules out |
/// |---|---|
/// | `committed` | a refused edit, which leaves the page untouched and agrees with itself perfectly. Read from the edit epoch, not from the plan |
/// | `runs=n/m` | `n != m` means the decomposition changed under the index, so `before` and `after` may name different lines — see [`LineReading`] |
/// | `before`/`after` spelled `none` | nothing was measured, and `moved` is `none` rather than a difference taken against a missing operand |
pub fn trace_left_edge(
    page: usize,
    run: usize,
    committed: bool,
    before: Option<LineReading>,
    after: Option<LineReading>,
) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // `moved` is emitted rather than left to the reader's subtraction,
        // and only when both ends were measured. A harness that differenced
        // two fields itself would have to decide what `none - 812.4` means,
        // and the answer a parser reaches for is `0` — this check's PASS
        // value. An unmeasured edit would then read as a line that held still.
        let spell = |v: Option<LineReading>| {
            v.and_then(|r| r.left)
                .map_or_else(|| "none".to_owned(), |x| format!("{x:.3}"))
        };
        let count =
            |v: Option<LineReading>| v.map_or_else(|| "none".to_owned(), |r| r.runs.to_string());
        let moved = match (before.and_then(|r| r.left), after.and_then(|r| r.left)) {
            (Some(b), Some(a)) => format!("{:+.3}", a - b),
            _ => "none".to_owned(),
        };
        format!(
            "edit-text-left-edge page={page} run={run} committed={} runs={}/{} before={} after={} moved={moved}",
            if committed { "yes" } else { "no" },
            count(before),
            count(after),
            spell(before),
            spell(after)
        )
    });
}
