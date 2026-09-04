//! PASS, FAIL, SKIPPED — and why the third one has to exist.
//!
//! ## The rule
//!
//! > **A missing precondition SKIPs. A missing postcondition FAILs.**
//!
//! A check that could not run has learned nothing, and "learned nothing" must
//! never be printed as green. That is the same defect as the gate documented in
//! `PROJECT_PLAN.md` §4.1 — a string checker that scanned three files out of
//! forty and reported `clean`, because finding nothing looks exactly like
//! finding no violations.
//!
//! It is a live hazard here rather than a theoretical one, because **the
//! application these checks target is still being built**. Checks in this
//! crate report SKIPPED for weeks at a time — as of S2 the application runs
//! and traces, and two of the three surfaces under test (the ribbon, the
//! Settings dialog) have not been written. If SKIPPED rendered as a pass, the
//! suite would be green for its entire construction period and would go on
//! being green after the first check silently broke.
//!
//! Which is why a SKIP reason is held to the same standard as a FAIL reason,
//! and audited when the application changes. A reason that names the wrong
//! blocked component — "the application traces no `ui-rect` regions", said of
//! a binary that traces three of them on every frame — sends its reader to a
//! finished module to look for a defect that is not there. That is strictly
//! worse than no reason at all, because they will believe it before they
//! disbelieve it.
//!
//! ## Where the line falls, concretely
//!
//! | Situation | Verdict | Why |
//! |---|---|---|
//! | no binary at the given path | SKIP | the harness never began |
//! | no window appeared | SKIP | ditto |
//! | the trace has no canvas rect | SKIP | nothing to aim at; the click was never made |
//! | the click was made and selected nothing | **FAIL** | the harness did its job and the application did not do its own |
//! | Delete was pressed and no deletion was traced | **FAIL** | see [`crate::checks::delete_key`] — this is the D1 verdict |
//! | the region set is calibrated for a different surface | SKIP | measuring the wrong pixels is worse than measuring none |
//! | the caption's contrast is 1.1:1 | **FAIL** | this is the D2 verdict |
//!
//! The interesting row is the fifth, and it is the row that makes this suite
//! evidence rather than decoration. The old binary is perfectly capable of
//! emitting its deletion event — the code path exists and the event is in its
//! vocabulary — it simply never gets there, because the key is suppressed. So
//! the absence of that event, *after* the harness has established that the
//! click landed and something was selected, is a postcondition failure and not
//! a missing feature.

use std::path::PathBuf;

/// One check's verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// It ran and the assertion held.
    Pass,
    /// It ran and the assertion did not hold. The string is the reason, in a
    /// form a reader can act on.
    Fail(String),
    /// A precondition was absent. The string names the missing thing.
    Skip(String),
}

impl Outcome {
    /// Four characters, for a column.
    #[must_use]
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail(_) => "FAIL",
            Self::Skip(_) => "SKIP",
        }
    }

    /// The reason, if there is one.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Pass => None,
            Self::Fail(r) | Self::Skip(r) => Some(r),
        }
    }
}

/// What one check produced.
#[derive(Clone, Debug)]
pub struct CheckReport {
    /// The check's name, as `--check` accepts it.
    pub name: &'static str,
    /// Which defect it detects, in one line.
    pub defect: &'static str,
    /// The verdict.
    pub outcome: Outcome,
    /// Observations made along the way, in order.
    ///
    /// Printed for every outcome, not only failures. On a PASS these are what
    /// show the check actually did something — a pass with no notes is a pass
    /// that should be read sceptically, which is the whole thesis of this
    /// crate applied to itself.
    pub notes: Vec<String>,
    /// Files written: screenshots, trace captures.
    pub artifacts: Vec<PathBuf>,
}

impl CheckReport {
    /// A report in progress.
    #[must_use]
    pub fn new(name: &'static str, defect: &'static str) -> Self {
        Self {
            name,
            defect,
            outcome: Outcome::Skip("the check did not reach a verdict".to_owned()),
            notes: Vec::new(),
            artifacts: Vec::new(),
        }
    }

    /// Record an observation.
    pub fn note(&mut self, note: impl Into<String>) -> &mut Self {
        self.notes.push(note.into());
        self
    }

    /// Record a written file.
    pub fn artifact(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.artifacts.push(path.into());
        self
    }

    /// Finish as a pass.
    #[must_use]
    pub fn pass(mut self) -> Self {
        self.outcome = Outcome::Pass;
        self
    }

    /// Finish as a failure.
    #[must_use]
    pub fn fail(mut self, reason: impl Into<String>) -> Self {
        self.outcome = Outcome::Fail(reason.into());
        self
    }

    /// Finish as a skip.
    #[must_use]
    pub fn skip(mut self, reason: impl Into<String>) -> Self {
        self.outcome = Outcome::Skip(reason.into());
        self
    }

    /// **Finish from an `Error`, letting the error decide skip or fail.**
    ///
    /// The one line every check's `run` uses for its `Err` arm, so that
    /// classification lives in one place instead of in 152 identical match
    /// arms.
    ///
    /// # ★★★ Why this is not just `skip`
    ///
    /// Because for two years it was, and on 2026-09-03 that let a **crashing
    /// build report PASS**. `dialogs_open_in_their_own_window` drives
    /// `pdfcer ▸ Keyboard shortcuts`, which aborted the process on open; the
    /// `viewport-inner` line the check greps for is written before the panic,
    /// so the evidence existed and the check was satisfied.
    ///
    /// The guard that catches it is in `Session::trace`, and it has to be able
    /// to produce a **red** result. Routing it through `skip` would have made a
    /// crashed program report as "did not run" — and this project's own record
    /// is that *a SKIP is not red, so a check can stop running unnoticed*.
    ///
    /// So the `Error` carries the distinction ([`crate::error::Error::fatal`])
    /// and this method reads it. Everything that was a precondition failure
    /// still skips; only what the harness positively observed the subject doing
    /// fails.
    #[must_use]
    pub fn from_error(self, err: &crate::error::Error) -> Self {
        if err.is_fatal() {
            self.fail(err.message().to_owned())
        } else {
            self.skip(err.message().to_owned())
        }
    }

    /// Print the full report for one check.
    pub fn print(&self) {
        println!("[{}] {}", self.outcome.tag(), self.name);
        println!("       detects: {}", self.defect);
        for note in &self.notes {
            println!("       · {note}");
        }
        if let Some(reason) = self.outcome.reason() {
            for (i, line) in wrap(reason, 76).into_iter().enumerate() {
                if i == 0 {
                    println!("       → {line}");
                } else {
                    println!("         {line}");
                }
            }
        }
        for a in &self.artifacts {
            println!("       artifact: {}", a.display());
        }
        println!();
    }
}

/// The whole run.
#[derive(Clone, Debug, Default)]
pub struct RunReport {
    /// One entry per check, in the order they ran.
    pub checks: Vec<CheckReport>,
}

impl RunReport {
    /// Print every check and a summary.
    pub fn print(&self) {
        for c in &self.checks {
            c.print();
        }
        let passed = self.count(|o| matches!(o, Outcome::Pass));
        let failed = self.count(|o| matches!(o, Outcome::Fail(_)));
        let skipped = self.count(|o| matches!(o, Outcome::Skip(_)));

        println!("------------------------------------------------------------------------");
        println!("  {passed} passed, {failed} failed, {skipped} skipped");
        println!();
        if failed > 0 {
            println!(
                "RESULT: FAIL — a check drove the application and the assertion did not hold."
            );
        } else if passed == 0 {
            println!("RESULT: NOTHING VERIFIED — no check reached a verdict.");
            println!();
            println!("  This is NOT a pass. Every check above was unable to begin, and the");
            println!("  reasons are printed with each one. A suite that cannot run has told");
            println!("  you nothing about the application, and 'told you nothing' rendered");
            println!("  as green is the exact failure this harness exists to remove.");
        } else if skipped > 0 {
            println!("RESULT: INCOMPLETE — {passed} verified, {skipped} could not run.");
            println!();
            println!("  Read each SKIP reason and decide whether it is expected. While the");
            println!("  application is under construction, most of them will be.");
        } else {
            println!("RESULT: PASS — every check drove the application and every assertion held.");
        }
    }

    /// The process exit code.
    ///
    /// * `0` — everything that ran passed, and at least one thing ran.
    /// * `1` — something failed.
    /// * `3` — nothing failed, but something did not run. Non-zero on purpose:
    ///   CI must not go green on a suite that did not execute. The distinct
    ///   code lets a caller tell "incomplete" from "broken" without parsing
    ///   the text.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        if self.count(|o| matches!(o, Outcome::Fail(_))) > 0 {
            return 1;
        }
        if self.count(|o| matches!(o, Outcome::Skip(_))) > 0 || self.checks.is_empty() {
            return 3;
        }
        0
    }

    fn count(&self, f: impl Fn(&Outcome) -> bool) -> usize {
        self.checks.iter().filter(|c| f(&c.outcome)).count()
    }
}

/// Wrap a reason to a width, so a long explanation stays readable in a
/// terminal instead of becoming one unbroken line.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(outcome: Outcome) -> CheckReport {
        let mut r = CheckReport::new("x", "y");
        r.outcome = outcome;
        r
    }

    /// The rule this whole module exists for.
    #[test]
    fn a_suite_that_only_skipped_does_not_exit_zero() {
        let run = RunReport {
            checks: vec![report(Outcome::Skip("no binary".into()))],
        };
        assert_eq!(run.exit_code(), 3);
    }

    #[test]
    fn an_empty_suite_does_not_exit_zero_either() {
        assert_eq!(RunReport::default().exit_code(), 3);
    }

    #[test]
    fn a_failure_outranks_a_skip() {
        let run = RunReport {
            checks: vec![
                report(Outcome::Skip("no binary".into())),
                report(Outcome::Fail("delete did nothing".into())),
            ],
        };
        assert_eq!(run.exit_code(), 1);
    }

    #[test]
    fn all_passing_exits_zero() {
        let run = RunReport {
            checks: vec![report(Outcome::Pass), report(Outcome::Pass)],
        };
        assert_eq!(run.exit_code(), 0);
    }

    #[test]
    fn a_mixed_pass_and_skip_run_is_incomplete_not_a_pass() {
        let run = RunReport {
            checks: vec![report(Outcome::Pass), report(Outcome::Skip("x".into()))],
        };
        assert_eq!(run.exit_code(), 3);
    }
}
