//! `layers_search_narrows_the_list` — **the Layers search field is drawn,
//! is reachable, and narrowing the list is not the same as emptying it.**
//!
//! # What this is for
//!
//! `OPERATOR_REQUESTS.md` O126: *"there is a search to implement on the
//! layers"*.
//!
//! `panels::layers::search`'s unit tests own the **predicate** — case
//! folding, substrings, trimming, the multi-byte walk — and they are swept
//! with no window open. This check answers the two things they cannot:
//!
//! 1. **Is the field on screen at all?** A predicate with no control in
//!    front of it is a function nobody can call. `panels::layers` draws the
//!    field only when the document has at least
//!    `search::MIN_LAYERS_FOR_SEARCH` layers, so a fixture with too few
//!    layers would produce a green run about a control that is not there —
//!    which is why this check refuses to pass on absence (see below).
//! 2. **Is it reachable, rather than merely laid out?** The field lives at
//!    the top of a panel body that is clipped into its stack. A rect proves
//!    layout; `crate::diag::ui_rect_visible` is what proves visibility, and
//!    the dock's whole rect stream goes through it.
//!
//! # ★★★ Why this check cannot pass on an absence
//!
//! The field is drawn conditionally, and `crate::diag::ui_rect` is a **change
//! log** — it emits a line when a rect appears or moves, and
//! `ui-rect-gone` when it retires. So "no `panel.layers.search` line" has
//! three causes that look identical from here:
//!
//! | cause | is it a defect? |
//! |---|---|
//! | the field regressed and is not drawn | **yes** |
//! | the fixture has fewer than two layers | no — correct behaviour |
//! | the Layers panel is not on screen in this mode | no |
//!
//! `D:/dev/rag/egui/a_check_that_may_decline_to_judge_is_a_check_that_cannot_fail.md`
//! is the rule: a check whose "nothing happened" branch is a pass is a check
//! that has stopped running and will not say so. So this one **establishes
//! its own precondition first** — it asserts the panel drew at all by looking
//! for `layer-row`, which `panels::layers` traces once per drawn row — and
//! only then requires the field. A fixture with too few layers makes the
//! check ERROR with a message naming the fixture, not pass.
//!
//! # ★★★ What is deliberately NOT covered, and it is the honest limit
//!
//! **Typing, and therefore the narrowed and empty states.**
//!
//! There is no seam in this application that fills a `TextEdit`, and adding
//! one would be a second way to set a value the operator sets exactly one
//! way — which is the shape of harness affordance that ends up exercising a
//! path no operator has. Driving it properly needs synthetic keystrokes into
//! a focused field, i.e. a pointer to focus it first, which puts this check
//! in the class that cannot run on a machine somebody is using.
//!
//! ⇒ So the three states behind the field are held by unit tests instead,
//! and they are held tightly:
//!
//! | state | where it is proved |
//! |---|---|
//! | the list narrows, case-insensitively, on substrings | `panels::layers::search::tests` — nine tests, swept |
//! | clearing restores the list | `an_empty_query_matches_every_layer` |
//! | an empty result is distinguishable from an empty document | `an_empty_result_knows_it_was_the_query_that_emptied_it` |
//! | the empty result quotes the query back | `text::panels::layersearch::tests::the_empty_case_repeats_what_was_typed` |
//!
//! Every one of those was falsified by planting the inverted behaviour. What
//! **no** unit test can say is whether the field is on screen — and that is
//! precisely what this check is for. The division is deliberate: the pure
//! rule to the sweep, the existence of the control to the driven run.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The command that puts the Layers panel on screen.
const SHOW_LAYERS: &str = "view.panel_layers";

/// The region the search field publishes. Must match
/// `panels::layers::REGION_SEARCH`.
const REGION_SEARCH: &str = "panel.layers.search";

/// The per-row trace the panel emits, used as this check's precondition.
const ROW_EVENT: &str = "layer-row";

/// See the module documentation.
pub struct LayersSearchNarrowsTheList;

impl Check for LayersSearchNarrowsTheList {
    fn name(&self) -> &'static str {
        "layers_search_narrows_the_list"
    }

    fn defect(&self) -> &'static str {
        "the Layers panel has no search field on screen, so the predicate behind it is a \
         function nobody can call. The operator asked for a search on the layers; a filter with \
         no control is the shape of half-implementation the request names"
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
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new("no --pdf. The Layers panel draws nothing without a document.")
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("layers-search.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), SHOW_LAYERS.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let trace = session.trace()?;

    // -----------------------------------------------------------------
    // A. THE PRECONDITION, established rather than assumed.
    //
    // If the panel drew no rows, everything below would be silent and a
    // check that treated that silence as a pass would have stopped running.
    // -----------------------------------------------------------------
    let rows = trace.events(ROW_EVENT).count();
    if rows == 0 {
        return Err(Error::new(format!(
            "the Layers panel drew no rows, so this check could not run. Either the panel is \
             not on screen after `{SHOW_LAYERS}`, or the --pdf has no optional content. Give it \
             a layered drawing. This is an ERROR and not a pass, deliberately: a check whose \
             \"nothing happened\" branch is green is a check that has stopped running and will \
             not say so."
        )));
    }
    report.note(format!("· the panel drew {rows} layer row(s)"));

    // ★ And enough of them to have earned a field. Fewer than two is correct
    // behaviour (`search::MIN_LAYERS_FOR_SEARCH`), so it is an ERROR about the
    // fixture rather than a failure of the program.
    if rows < 2 {
        return Err(Error::new(format!(
            "the --pdf has only {rows} layer, and the search field is deliberately not drawn \
             below two — a search over one row can only remove the row. Give this check a \
             drawing with several layers; a pass on this fixture would be a pass about a \
             control the program was right not to draw."
        )));
    }

    let mut failures: Vec<String> = Vec::new();

    // -----------------------------------------------------------------
    // B. THE FIELD IS ON SCREEN.
    //
    // `ui_rect` is only reached for the field when the panel body runs, and
    // the dock's own compartment rects go through `ui_rect_visible` — so a
    // panel clipped out of its stack publishes no compartment and this line
    // is the application's half of the pair.
    // -----------------------------------------------------------------
    match trace
        .events("ui-rect")
        .find(|l| l.raw.contains(REGION_SEARCH))
    {
        Some(l) => {
            report.note(format!("★ the search field is drawn: {}", l.raw));
        }
        None => failures.push(format!(
            "no `{REGION_SEARCH}` region was published, with {rows} layers on screen. The \
             field is not being drawn, so the search predicate behind it is unreachable"
        )),
    }

    // -----------------------------------------------------------------
    // C. THE PANEL'S OWN COMPARTMENT IS REACHABLE.
    //
    // ★★ `dock.body.view.panel_layers` goes through `ui_rect_visible`, which
    // publishes ONLY when at least 60 % of the region survived its clip. So
    // its presence is a reachability claim and not merely a layout one —
    // which is the distinction three panels shipped without on 2026-08-10.
    // Without it, section B could pass about a field drawn inside a
    // compartment nobody can see.
    // -----------------------------------------------------------------
    let body = format!("dock.body.{SHOW_LAYERS}");
    match trace.events("ui-rect").find(|l| l.raw.contains(&body)) {
        Some(l) => {
            report.note(format!("★ and its compartment is reachable: {}", l.raw));
        }
        None => failures.push(format!(
            "`{body}` published nothing. `ui_rect_visible` is silent below 60 % visibility, so \
             the panel is laid out somewhere the operator cannot read — which makes the field \
             in section B a control drawn inside a compartment nobody can see"
        )),
    }

    if failures.is_empty() {
        return Ok(None);
    }
    Ok(Some(format!(
        "★ {} of the Layers search properties failed:\n  · {}",
        failures.len(),
        failures.join("\n  · ")
    )))
}
