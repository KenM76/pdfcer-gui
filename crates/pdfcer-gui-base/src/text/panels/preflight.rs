//! # `text::panels::preflight` — every string the Preflight panel shows
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/panels/preflight.md`.

/// The plain name of a built-in profile, by its engine name. An unknown name
/// is shown as the engine spells it.
#[must_use]
pub fn profile_name(name: &str) -> String {
    let plain = match name {
        "digital-printing" => "Digital printing",
        "pdfa-1b" => "PDF/A-1b — archive",
        "pdfa-2b" => "PDF/A-2b — archive",
        "pdfa-3b" => "PDF/A-3b — archive",
        "pdfua-1" => "PDF/UA-1 — accessible",
        "pdfua-2" => "PDF/UA-2 — accessible",
        "pdfx-1a-2001" => "PDF/X-1a:2001 — press, CMYK only",
        "pdfx-1a-2003" => "PDF/X-1a:2003 — press, CMYK only",
        "pdfx-3-2002" => "PDF/X-3:2002 — press, colour managed",
        "pdfx-3-2003" => "PDF/X-3:2003 — press, colour managed",
        "pdfx-4" => "PDF/X-4 — press, transparency kept",
        other => return other.to_owned(),
    };
    plain.to_owned()
}

/// The label in front of the profile picker.
#[must_use]
pub fn profile_label() -> &'static str {
    "Profile"
}

/// The Run button.
#[must_use]
pub fn run() -> &'static str {
    "Run"
}

/// The Run button once a result is out of date.
#[must_use]
pub fn run_again() -> &'static str {
    "Run again"
}

/// Hover on Run.
#[must_use]
pub fn run_hover() -> &'static str {
    "Check the document as it stands now, unsaved changes included. You can keep working while it runs."
}

/// The Cancel button while a run is going.
#[must_use]
pub fn cancel() -> &'static str {
    "Cancel"
}

/// The progress line.
#[must_use]
pub fn progress(done: usize, of: usize) -> String {
    format!("Checking page {done} of {of}…")
}

/// Shown when a run ended because the operator cancelled it.
#[must_use]
pub fn cancelled(checked: usize, of: usize) -> String {
    format!(
        "Cancelled after {checked} of {of} pages. The findings below cover only those pages; this is not a pass."
    )
}

/// Shown above results the document has moved on from.
#[must_use]
pub fn stale() -> &'static str {
    "The document has changed since this check. These findings may no longer be true, and fixes are off until you run it again."
}

/// Shown when the run's worker went away without an answer.
#[must_use]
pub fn worker_lost() -> &'static str {
    "The check stopped without a result. Run it again."
}

/// The headline of a report with no findings at all.
#[must_use]
pub fn clean(pages: usize) -> String {
    format!("Clean: no findings on {pages} page(s).")
}

/// The headline of a report with findings.
#[must_use]
pub fn summary(violations: usize, people: usize, incomplete: usize, other: usize) -> String {
    let mut parts = Vec::new();
    if violations > 0 {
        parts.push(format!("{violations} violation(s)"));
    }
    if people > 0 {
        parts.push(format!("{people} need a person"));
    }
    if incomplete > 0 {
        parts.push(format!("{incomplete} incomplete"));
    }
    if other > 0 {
        parts.push(format!("{other} other"));
    }
    format!("Not clean: {}.", parts.join(", "))
}

/// Findings past the cap, which the engine counted but did not keep.
#[must_use]
pub fn dropped(n: usize) -> String {
    format!("{n} more finding(s) were found and not listed: the list is capped.")
}

/// The kind label of a violation.
#[must_use]
pub fn kind_violation() -> &'static str {
    "Violation"
}

/// The kind label of a finding a person must judge.
#[must_use]
pub fn kind_needs_person() -> &'static str {
    "Needs a person"
}

/// The kind label of a check that could not finish.
#[must_use]
pub fn kind_incomplete() -> &'static str {
    "Incomplete — not checked"
}

/// A rule heading: its id, severity and how many findings it has.
#[must_use]
pub fn rule_heading(rule: &str, severity: &str, n: usize) -> String {
    format!("{rule} ({severity}) — {n}")
}

/// Where a finding is, for its row.
#[must_use]
pub fn at_page(page: usize) -> String {
    format!("p. {page}")
}

/// A finding that names no page.
#[must_use]
pub fn at_document() -> &'static str {
    "document"
}

/// A measured value against its limit.
#[must_use]
pub fn measured(value: f64, limit: f64, unit: &str) -> String {
    format!("{value:.1} {unit} (limit {limit:.1})")
}

/// A finding row: its kind, where it is, the engine's message, and the
/// measurement when there is one.
pub fn row(kind: &str, at: &str, message: &str, measured: Option<&str>) -> String {
    match measured {
        Some(m) => format!("{kind} · {at} · {message} · {m}"),
        None => format!("{kind} · {at} · {message}"),
    }
}

/// Hover on a finding row.
#[must_use]
pub fn finding_hover() -> &'static str {
    "Go to this finding. An object finding selects the object, so Properties and Delete act on it."
}

/// The Copy report button.
#[must_use]
pub fn copy_report() -> &'static str {
    "Copy report"
}

/// Hover on Copy report.
#[must_use]
pub fn copy_report_hover() -> &'static str {
    "Copy the whole report as plain text."
}

/// The status line after Copy report.
#[must_use]
pub fn copied() -> &'static str {
    "Preflight report copied."
}

/// The per-rule fix button.
#[must_use]
pub fn fix() -> &'static str {
    "Fix"
}

/// Hover on a per-rule fix.
#[must_use]
pub fn fix_hover() -> &'static str {
    "Fix this rule's findings, as one step you can undo. Run the check again afterwards to see what is left."
}

/// The Fix all button.
#[must_use]
pub fn fix_all() -> &'static str {
    "Fix all"
}

/// Hover on Fix all.
#[must_use]
pub fn fix_all_hover() -> &'static str {
    "Fix every rule that has a fix, as one step you can undo. Rules with no fix are listed as not fixed."
}

/// Why the fixes are off in a mode that cannot change content.
#[must_use]
pub fn fix_needs_edit() -> &'static str {
    "Fixes change the document. Switch to Edit mode to use them."
}

/// Why the fixes are off on an out-of-date result.
#[must_use]
pub fn fix_needs_fresh() -> &'static str {
    "The document has changed since this check. Run it again first."
}

/// The heading over the last fix's report.
#[must_use]
pub fn fixed_heading() -> &'static str {
    "Last fix"
}

/// Rows of one rule past the drawn limit.
#[must_use]
pub fn more_rows(n: usize) -> String {
    format!("…and {n} more. Copy report lists every one.")
}

/// A standard's verdict, at the top of the results.
#[must_use]
pub fn standard_verdict(claim: &str, verdict: Option<&str>) -> String {
    let word = match verdict {
        Some("conforms") => "conforms",
        Some("fails") => "does not conform",
        Some(_) => "cannot tell",
        None => "no overall verdict",
    };
    format!("{claim}: {word}")
}

/// Why the verdict is what it is.
#[must_use]
pub fn standard_reasons(
    failed: usize,
    warned: usize,
    person: usize,
    not_checked: Option<usize>,
) -> String {
    let mut parts = vec![
        format!("{failed} requirement(s) failed"),
        format!("{warned} warned"),
        format!("{person} need a person to decide"),
    ];
    if let Some(n) = not_checked {
        parts.push(format!("{n} not checked"));
    }
    format!("{}.", parts.join(", "))
}

/// Why PDF/UA has no verdict.
#[must_use]
pub const fn standard_ua_no_verdict() -> &'static str {
    "The checker does not decide PDF/UA conformance as a whole: it reports what failed, and some requirements need a person to judge. A file with nothing failed may still not conform."
}

/// Why a verdict of "cannot tell".
#[must_use]
pub const fn standard_unconfirmed() -> &'static str {
    "Nothing failed, but something warned, needs a person, or was not checked, so conformance is not confirmed."
}

/// The Convert to PDF/X button.
#[must_use]
pub const fn convert_pdfx() -> &'static str {
    "Convert to PDF/X…"
}

/// What Convert to PDF/X does.
#[must_use]
pub const fn convert_pdfx_hover() -> &'static str {
    "Open Convert to PDF/X: flatten, convert colours and set the output profile so the file meets the standard."
}
