//! `signature_trust_is_reported_as_its_own_fact` — **the Signatures panel
//! reports three facts and never merges them, and `not checked` says so.**
//!
//! # ★★★ WRITTEN 2026-09-05 AND **NOT RUN**
//!
//! Said here, in the module's own words, rather than left for an absent result
//! to imply. The operator may be at his keyboard, and this harness drives a
//! real cursor and a real keyboard over the whole desktop — a run while he is
//! working is a run whose verdict is noise that looks like a finding.
//!
//! `export_text` and `copy_as_vector` carry the same disclosure for the same
//! reason. **An unrun check is not a passing check**, and this project's
//! standing rule is that no UI change is done until it has been verified by
//! driving the running binary. This one has not been.
//!
//! # What it detects, and why nothing else can
//!
//! `pdfcer-core`'s `SignatureVerdict` carries **three facts that never collapse
//! into one bool**: whether the signed bytes are intact, what the signature
//! covers, and whether the signer chains to a trusted anchor. The design exists
//! because a reader that says *"valid ✓"* about a chain it did not validate is
//! the one failure worse than saying nothing.
//!
//! `crate::panels::signatures`' unit tests pin the *sentences* and the
//! *mapping* — that four unchecked states produce four explanations, that a
//! `Trusted` line discloses what it did not check. What no unit test can reach
//! is the six links between a signed file on disk and three lines on screen:
//!
//! 1. the panel is reachable at all, from the View tab, in a real layout;
//! 2. its dock tab is selected, so its body actually executes — a docked pane
//!    that is not in front publishes nothing, which is indistinguishable from a
//!    panel with nothing to say (`RESUME.md` records that exact misdiagnosis);
//! 3. `crate::trust::cached_report` reads the file **from disk** and gets a
//!    verdict list back, rather than failing silently and leaving the panel
//!    showing coverage alone;
//! 4. the row loop pairs coverage `i` with verdict `i`;
//! 5. every row publishes an `integrity=` token AND a `trust=` token — the two
//!    facts that did not exist before this work;
//! 6. with the setting at its shipped default, `trust=not-checked` is what
//!    comes out, rather than nothing.
//!
//! Link 6 is the assertion the whole feature stands on. A build that dropped
//! the trust line entirely when there were no anchors would pass every other
//! assertion here and would be the exact defect this feature was written to
//! prevent: on screen, "we did not look" and "we looked and it was fine" would
//! be the same picture.
//!
//! # ★★ The oracle is the TRACE TOKEN, not the sentence
//!
//! `signature-row … integrity=<token> trust=<token>` carries diagnostic words —
//! `verified`, `not-checked`, `untrusted` — that are deliberately **not** in
//! the copy catalog. A check that matched operator prose would go red the day
//! somebody improved a sentence, which trains people to re-baseline checks; and
//! a check re-baselined is a check that has stopped measuring.
//!
//! The **rectangle** is asserted separately, through `panel:signatures`,
//! because a trace line proves the code ran and says nothing about whether an
//! operator can see it. This project has recorded that distinction more than
//! once.
//!
//! # ★★★ What this deliberately does NOT assert
//!
//! **That anything is ever `trusted`.** It cannot, and pretending otherwise
//! would be the worst check in the suite. A `Trusted` verdict needs a signature
//! whose signer chains to a real AATL or EUTL anchor, which needs (a) somebody's
//! real certificate committed to this repository and (b) the operator's own
//! Acrobat trust store present on whatever machine runs the suite. The first is
//! a certificate that expires; the second makes the verdict a report about the
//! machine rather than about the code.
//!
//! So the positive trust path is **unverified by this harness**, and saying so
//! is the whole of the honesty available here. What covers it instead:
//!
//! * `pdfcer-core`'s own `trust_chain` tests, against synthetic chains it
//!   builds and signs itself;
//! * `pdfcer trust-store-list` at the command line, which the engine ran
//!   against this operator's real 1,780-anchor store;
//! * the Settings group's *Show what is in it* button, which is the operator's
//!   own route to the same reading and reports the count and the store's date.
//!
//! ⇒ **If one thing in this feature is to be driven next, it is the Settings
//! group on the operator's own machine**, where a store really exists: turn the
//! setting on, press *Show what is in it*, and read the counts back. That needs
//! his machine and his file, which is why it is named here rather than written
//! as a check that would SKIP everywhere else.
//!
//! # The fixture
//!
//! `fixtures/signed-two-pages.pdf`, **pinned**, and `--pdf` is ignored. The
//! check's subject is *"what does the panel say about a signature"*, and on a
//! document with no signature fields the panel correctly draws one sentence and
//! publishes no rows — so an arbitrary fixture would make this unable to fail.
//! It says so in its notes when a `--pdf` was supplied and thrown away, because
//! a run that silently ignored a flag is indistinguishable from one that
//! honoured it.

use std::path::PathBuf;

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, declared_or_in_overflow, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The fixture this check pins. See the module header.
const FIXTURE: &str = "signed-two-pages.pdf";
/// The ribbon control that shows the panel.
const PANEL_ITEM: &str = "ribbon.item.view.panel_signatures";
/// The panel's dock tab — the evidence it is OPEN, independent of its body.
const PANEL_TAB: &str = "dock.tab.view.panel_signatures";
/// The panel body's own region — the evidence it is ON SCREEN.
const PANEL_BODY: &str = "panel:signatures";
/// The per-signature trace line the panel writes.
const ROW: &str = "signature-row";
/// The whole-report line `crate::trust::cached_report` writes.
const REPORT: &str = "trust-report";

/// See the module documentation.
pub struct SignatureTrustIsReportedAsItsOwnFact;

impl Check for SignatureTrustIsReportedAsItsOwnFact {
    fn name(&self) -> &'static str {
        "signature_trust_is_reported_as_its_own_fact"
    }

    fn defect(&self) -> &'static str {
        "the Signatures panel folds integrity, coverage and trust into one verdict, or drops the \
         trust line when no anchors are available — so a document whose signer pdfcer never \
         looked at is indistinguishable, on screen, from one it checked and approved"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, a ribbon control \
             and a dock tab. Reported as SKIPPED rather than passed: a check that did not run \
             has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;
    let pdf = repo_fixture(ctx)?;
    if ctx.pdf.is_some() {
        report.note(
            "★ a --pdf was supplied and IGNORED: this check pins its own fixture, because on a \
             document with no signature fields the panel draws one sentence and publishes no \
             rows — which would make it unable to fail",
        );
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("trust-store.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // Read is where the application starts; say so anyway, or this stops
    // testing Read the day the default moves.
    click_mode_segment(&session, &driver, ui_rect, "read")?;

    // ★ Only if it is not already showing. The ribbon control is a TOGGLE, and
    // pressing it over an open panel closes the thing under test — which is how
    // a sibling check produced a SKIP on one run and a FAIL on the next from
    // the same build.
    if declared(&session.trace()?, ui_rect, PANEL_TAB).is_none() {
        open_signatures(&session, &driver, ui_rect)?;
    }
    // ★★ …and then SELECT it. A dock tab is declared whether or not it is the
    // one in front, and the dock draws only the selected tab's body. A panel
    // behind another tab publishes nothing, which reads exactly like a panel
    // with nothing to say — `RESUME.md` records that misdiagnosis costing a
    // session.
    if let Some(tab) = declared(&session.trace()?, ui_rect, PANEL_TAB) {
        driver.click_at(session.frame()?.declared_center(tab))?;
        session.settle(25);
    }

    let trace = session.trace()?;
    if declared(&trace, ui_rect, PANEL_TAB).is_none() {
        return Err(Error::new(format!(
            "the Signatures panel is not showing — no `{PANEL_TAB}` region — so this run cannot \
             tell 'the panel reports nothing' from 'the panel never opened'. SKIPPED, not \
             passed. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "dock.tab"))
        )));
    }
    // ★★★ The panel's BODY, not only its tab. The tab proves the dock knows
    // about the panel; the body region proves the panel's own code ran and laid
    // itself out somewhere visible. Two different claims, and only the second
    // is what an operator experiences.
    let Some(body) = declared(&trace, ui_rect, PANEL_BODY) else {
        return Ok(Some(format!(
            "★ THE SIGNATURES PANEL'S TAB IS SELECTED AND ITS BODY DREW NOTHING: no \
             `{PANEL_BODY}` region. Either the panel bailed out before publishing — the \
             `std::fs::metadata` guard at the top, which is the 'pdfcer could not read this \
             file' path — or the region was renamed and this check is now aimed at nothing. \
             Regions beginning `panel:`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "panel:")),
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "the Signatures panel is open and on screen at {body:?} on {FIXTURE}"
    ));

    // ---------------------------------------------------------------------
    // The report ran at all
    // ---------------------------------------------------------------------
    // ★ Read as parsed EVENTS rather than by substring. `Trace::events` matches
    // the event NAME, so a `report.note` that happened to quote the word
    // `trust-report` cannot be mistaken for the application having emitted one
    // — which is the class of false green this harness has recorded twice.
    let report_line = trace.first(REPORT);
    if report_line.is_none() {
        return Ok(Some(format!(
            "★★★ NO `{REPORT}` LINE: the panel drew and `crate::trust::cached_report` never \
             produced a verdict, so the three-facts report was not computed at all. The likely \
             causes, in order: the document reports `Origin::Created` so `stored_under()` gave \
             `None` and there was nothing to verify; the file could not be read from disk; or \
             the call was removed. Coverage alone would still have rendered, which is exactly \
             why this is asserted separately. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "the trust report ran: {}",
        report_line.map(|l| l.raw.trim()).unwrap_or_default()
    ));

    // ---------------------------------------------------------------------
    // Every row carries BOTH new facts
    // ---------------------------------------------------------------------
    let rows: Vec<_> = trace.events(ROW).collect();
    if rows.is_empty() {
        return Ok(Some(format!(
            "★ NO `{ROW}` LINE, on a fixture chosen because it carries a signature. Either \
             `byte_range_coverage` found no signature FIELD (the fixture has been replaced), or \
             the row loop no longer traces. This check cannot judge a panel that drew no rows. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    for row in &rows {
        if row.get("integrity").is_none() {
            return Ok(Some(format!(
                "★★★ A SIGNATURE ROW REPORTS NO INTEGRITY FACT: {}\n\
                 The panel is back to reporting coverage alone — which is what it did before \
                 `signature::verify_all_with_trust` was wired, and means an operator is being \
                 shown byte counts about a signature whose bytes may have been altered. Trace: \
                 {}.",
                row.raw,
                session.trace_path().display()
            )));
        }
        if row.get("trust").is_none() {
            return Ok(Some(format!(
                "★★★ A SIGNATURE ROW REPORTS NO TRUST FACT: {}\n\
                 This is the defect the whole feature exists to prevent. A row with no trust \
                 line looks, on screen, exactly like a row whose signer pdfcer checked and \
                 approved. `NotChecked` must render AS ITSELF. Trace: {}.",
                row.raw,
                session.trace_path().display()
            )));
        }
    }
    report.note(format!(
        "{} signature row(s), every one carrying its own integrity and trust facts",
        rows.len()
    ));

    // ---------------------------------------------------------------------
    // ★★★ The default is `not-checked`, and it is SAID
    // ---------------------------------------------------------------------
    // `Settings::acrobat_trust_store` ships as `Off`, so on any machine and in
    // any checkout this run must produce `trust=not-checked`. That makes the
    // assertion deterministic — unlike anything about a real anchor store,
    // which is why the positive path is not asserted here at all.
    //
    // ⚠ If the operator has turned the setting ON in his own `settings.txt`,
    // this reads `trusted`/`untrusted`/`signer-unknown` instead. That is not a
    // failure of the build, so it is reported as a SKIP-shaped note plus a
    // relaxed assertion: what must hold in every case is that the token is one
    // of the four the panel knows, and never absent or `unknown-variant`.
    const KNOWN: &[&str] = &["not-checked", "trusted", "untrusted", "signer-unknown"];
    let mut saw_not_checked = false;
    for row in &rows {
        let token = row.get("trust").unwrap_or_default();
        if token == "unknown-variant" {
            return Ok(Some(format!(
                "★★ A TRUST VERDICT THIS BUILD DOES NOT RECOGNISE: {}\n\
                 `pdfcer_core::signature::Trust` is `#[non_exhaustive]` and has grown a variant \
                 the panel has no sentence for. It renders through the catch-all, which says \
                 'not checked' — honest, and wrong about a verdict the engine actually reached. \
                 Read the engine's `signature_verify` and give the new variant its own line.",
                row.raw
            )));
        }
        if !KNOWN.contains(&token) {
            return Ok(Some(format!(
                "★ A TRUST TOKEN THIS CHECK DOES NOT RECOGNISE ({token:?}) in: {}\n\
                 The tokens are `crate::panels::signatures::trust_token`'s and are deliberately \
                 not operator copy. Either a token was renamed — update this check WITH the \
                 rename — or the trace format moved.",
                row.raw
            )));
        }
        saw_not_checked |= token == "not-checked";
    }
    if saw_not_checked {
        report.note(
            "★★★ trust reads `not-checked` at the shipped default, which is the property this \
             feature stands on: pdfcer says out loud that it did not look, rather than leaving \
             a silence an operator reads as approval",
        );
    } else {
        report.note(
            "★ NO row read `not-checked`, which on a default profile would be a defect and here \
             is most likely the operator having turned `acrobat_trust_store = at_own_risk` on in \
             his own settings.txt. The four-token assertion above still held. To assert the \
             default, run against a profile with a clean settings store.",
        );
    }

    // ---------------------------------------------------------------------
    // What this run did NOT establish, stated in the report rather than left
    // to an absent line.
    // ---------------------------------------------------------------------
    report.note(
        "NOT verified by this check: that any signature ever reads `trusted`. That needs a \
         signer chaining to a real AATL/EUTL anchor, which needs somebody's real certificate in \
         this repository and the operator's own Acrobat store on the machine running the suite \
         — the first expires, the second makes the verdict a report about the machine. See this \
         module's header for what covers it instead.",
    );
    Ok(None)
}

/// Show the Signatures panel from the View tab.
fn open_signatures(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.view").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.view` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let item = declared_or_in_overflow(session, driver, ui_rect, PANEL_ITEM)?.ok_or_else(|| {
        Error::new(format!(
            "no `{PANEL_ITEM}` region on the View tab or in its overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                "ribbon.item.view."
            ))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(20);
    Ok(())
}

/// **Resolve this check's pinned fixture, refusing to guess.**
///
/// Refused rather than SKIPped when it is missing, for `protect`'s stated
/// reason: a SKIP reads as *"this build does not have the feature"*, and a
/// missing fixture is a fact about the checkout.
fn repo_fixture(ctx: &CheckContext) -> Result<PathBuf> {
    // *** From this crate's own manifest directory, NOT from `ctx.source_root`
    // *** -- corrected 2026-09-05, the first time this check was ever run.
    //
    // `--source-root` defaults to `crates`, because its job is the STALENESS
    // comparison: which tree's mtimes decide whether the binary is older than
    // its sources. It is not a repository root and never was. So
    // `root.join("fixtures")` resolved to `crates/fixtures/...`, which does not
    // exist, and this check reported:
    //
    // ```text
    // [SKIP] -> the fixture crates\fixtures\<name>.pdf is missing
    // ```
    //
    // => It would have SKIPPED FOR EVER WHILE LOOKING HEALTHY, which is the
    // precise failure this check's own header warns about for a fixture that
    // cannot exercise the feature. This is the same trap one level out: not a
    // fixture too weak to fail, but a fixture never found at all -- and a suite
    // reporting SKIP is reporting *nothing*, which is why this harness exits 3
    // rather than 0 on an incomplete run.
    //
    // `CARGO_MANIFEST_DIR` is `tools/ui-verify`, so two parents up is the
    // workspace root. Resolved at COMPILE TIME, so it cannot be got wrong by an
    // invocation -- the property `--source-root` lacked. This is the pattern
    // `checks::comment_popup` already used, and that check ran green.
    let _ = ctx;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join(FIXTURE);
    if !path.is_file() {
        return Err(Error::new(format!(
            "the fixture {} is missing. This check needs a document that carries a signature \
             FIELD with a /ByteRange; on anything else the panel correctly draws one sentence \
             and publishes no rows, and this check would be unable to fail.",
            path.display()
        )));
    }
    Ok(path)
}
