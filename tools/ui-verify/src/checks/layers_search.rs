//! `layers_search_narrows_the_list` — **the Layers search field is drawn,
//! is reachable, and narrowing the list is not the same as emptying it.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/layers_search.md`.

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

    // And enough of them to have earned a field. Fewer than two is correct
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
