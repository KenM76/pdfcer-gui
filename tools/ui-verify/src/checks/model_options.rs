//! `checks::model_options` — **Insert 3D model's choices reach the file, and
//! the viewer's part list shows and hides parts**
//!
//! Both drive the window off the desktop through the scripted pointer.
//!
//! *Insert options.* Places the engine corpus's `assembly.prc` with "when the
//! page opens" and CAD lights picked in the Insert 3D model window. The
//! window's `insert-model-placed` line, the action's `model-insert-requested`
//! line and the engine's opening view (`model-insert-lit scheme=Some(Cad)
//! framed=true views=1`) must all carry them, and the 3D models list must read
//! them back (`models-row views=1 activation=PageOpen`): a choice the window
//! shows but the action drops is the defect.
//!
//! *Parts.* Opens `named-tree.prc`, whose tree stores Pin hidden and Nut
//! suppressed, in the viewer. Every part in it is the same square in the same
//! place, so a part's presence is read from an otherwise empty picture:
//! clearing Bolt, Bracket and Plate must draw nothing; ticking Pin must draw
//! it; clearing it again must draw the empty picture's exact bytes (the paint
//! is lent and given back); ticking it and clicking its name must highlight
//! it, a different picture.

use crate::checks::driving::declared_in;
use crate::checks::model_inferences::open_viewer;
use crate::checks::model_view_window::{MODEL, launch_with_model_file, press_insert_with, ui_rect};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// The "when the page opens" choice (`dialogs::insert_model::REGION_PAGE_OPEN`).
const PAGE_OPEN: &str = "insert-model.activation.page-open";
/// The CAD-lights choice (`dialogs::insert_model::REGION_LIGHTING`).
const LIGHTING: &str = "insert-model.lighting";
/// Assembly (0) > Bolt (1), Bracket (2), Plate (3), Pin (4, stored hidden),
/// Nut (5, stored suppressed).
const NAMED_TREE: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/named-tree.prc";
const SHOW_PIN: &str = "model3d.part.show.4";
const NAME_PIN: &str = "model3d.part.name.4";
const STATE: &str = "model-view-parts-state";
const RENDERED: &str = "model-view-rendered";

/// See the module documentation.
pub struct InsertModelChoicesReachTheFile;

impl Check for InsertModelChoicesReachTheFile {
    fn name(&self) -> &'static str {
        "insert_3d_model_choices_reach_the_file"
    }

    fn defect(&self) -> &'static str {
        "the Insert 3D model window offers when the model starts and how it is lit, but the \
         placed model starts on a click and carries no lighting"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model_file(ctx, &mut report, "model-options", MODEL)
            .and_then(|(session, pointer)| insert(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn insert(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    pointer.gone(session)?;
    session.settle(10);
    if !press_insert_with(ctx, session, pointer, &[PAGE_OPEN, LIGHTING])? {
        return Ok(Some("no 3D model button on the Edit tab.".to_owned()));
    }
    session.settle(30);
    let trace = session.trace()?;
    let last = |event: &str| trace.events(event).last().map(|l| l.raw.clone());
    let placed = last("insert-model-placed");
    let requested = last("model-insert-requested");
    let lit = last("model-insert-lit");
    report.note(format!("placed: {placed:?}"));
    report.note(format!("requested: {requested:?}"));
    report.note(format!("lit: {lit:?}"));
    let Some(placed) = trace.events("insert-model-placed").last() else {
        return Ok(Some(
            "the Insert 3D model window traced no `insert-model-placed`: Place was not reached."
                .to_owned(),
        ));
    };
    if placed.get("activation") != Some("PageOpen") || placed.get("lighting") != Some("Some(Cad)") {
        return Ok(Some(format!(
            "★★★ the window placed activation={:?} lighting={:?}; the check picked PageOpen and \
             CAD lights.",
            placed.get("activation"),
            placed.get("lighting")
        )));
    }
    let Some(requested) = trace.events("model-insert-requested").last() else {
        return Ok(Some("Place traced no `model-insert-requested`.".to_owned()));
    };
    if requested.get("activation") != Some("PageOpen") || requested.get("lighting") != Some("Cad") {
        return Ok(Some(format!(
            "★★★ the action asked the engine for activation={:?} lighting={:?}: the window's \
             choices were dropped on the way.",
            requested.get("activation"),
            requested.get("lighting")
        )));
    }
    let Some(lit) = trace.events("model-insert-lit").last() else {
        return Ok(Some(
            "★★★ no `model-insert-lit` line: the placed model carries no lit opening view."
                .to_owned(),
        ));
    };
    let wanted = [("scheme", "Some(Cad)"), ("framed", "true"), ("views", "1")];
    if wanted.iter().any(|(k, v)| lit.get(k) != Some(v)) {
        return Ok(Some(format!(
            "★★★ the opening view is `{}`; a framed view lit by CAD lights, alone, was asked for.",
            lit.raw
        )));
    }
    listed(session, report)
}

/// The 3D models list must read back what was written: one view, and the
/// page-open start.
fn listed(session: &Session, report: &mut CheckReport) -> Result<Option<String>> {
    session.settle(20);
    let trace = session.trace()?;
    let Some(row) = trace.events("models-row").last() else {
        return Ok(Some(
            "the Attachments panel, opened at launch, drew no 3D models row after Place."
                .to_owned(),
        ));
    };
    report.note(format!("listed: {}", row.raw));
    if row.get("activation") != Some("PageOpen") || row.get("views") != Some("1") {
        return Ok(Some(format!(
            "★★★ the 3D models list reads `{}` back; the model was placed to start when the page \
             opens, with one view.",
            row.raw
        )));
    }
    Ok(None)
}

/// See the module documentation.
pub struct TheViewerShowsAndHidesParts;

impl Check for TheViewerShowsAndHidesParts {
    fn name(&self) -> &'static str {
        "the_3d_viewer_shows_and_hides_parts"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's part list cannot show a part the file hides, hide one it shows, or \
         highlight one"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model_file(ctx, &mut report, "model-parts", NAMED_TREE)
            .and_then(|(session, pointer)| parts(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The last `model-view-rendered` line's `covered` and `hash`.
fn picture(session: &Session) -> Result<Option<(usize, String)>> {
    Ok(session.trace()?.events(RENDERED).last().and_then(|l| {
        let covered = l.get("covered")?.parse().ok()?;
        Some((covered, l.get("hash")?.to_owned()))
    }))
}

/// Click `region`, drawn in the viewer, and return the parts line it traced.
fn press(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some((rect, vp)) = declared_in(&trace, ui_rect(ctx)?, region) else {
        return Ok(None);
    };
    let before = trace.events(STATE).count();
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    let trace = session.trace()?;
    Ok(trace
        .events(STATE)
        .nth(before)
        .map(|l| l.raw.clone())
        .or_else(|| Some(String::new())))
}

fn parts(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(failure) = open_viewer(ctx, session, pointer)? {
        return Ok(Some(failure));
    }
    let Some((covered, _)) = picture(session)? else {
        return Ok(Some("the viewer rendered nothing.".to_owned()));
    };
    report.note(format!("stored state covers {covered} pixels"));
    if covered == 0 {
        return Ok(Some(
            "the stored state drew nothing; Bolt, Bracket and Plate are shown.".to_owned(),
        ));
    }
    let mut pictures = Vec::new();
    for (region, wanted) in STEPS {
        let Some(line) = press(ctx, session, pointer, region)? else {
            return Ok(Some(format!("the part list drew no `{region}`.")));
        };
        report.note(format!("{region}: {line}"));
        if !line.contains(wanted) {
            return Ok(Some(format!(
                "★★★ pressing `{region}` traced `{line}`; it must carry `{}`.",
                wanted.trim()
            )));
        }
        let Some(drawn) = picture(session)? else {
            return Ok(Some("the viewer stopped rendering.".to_owned()));
        };
        report.note(format!("  covered={} hash={}", drawn.0, drawn.1));
        pictures.push(drawn);
    }
    judge(&pictures)
}

/// Bolt, Bracket and Plate hidden; Pin shown, hidden again, then shown and
/// highlighted.
const STEPS: [(&str, &str); 7] = [
    ("model3d.part.show.1", "hidden=1|4|5 "),
    ("model3d.part.show.2", "hidden=1|2|4|5 "),
    ("model3d.part.show.3", "hidden=1|2|3|4|5 "),
    (SHOW_PIN, "hidden=1|2|3|5 "),
    (SHOW_PIN, "hidden=1|2|3|4|5 "),
    (SHOW_PIN, "hidden=1|2|3|5 "),
    (NAME_PIN, "highlight=4 "),
];

fn judge(p: &[(usize, String)]) -> Result<Option<String>> {
    let failure = if p[2].0 != 0 {
        "★★★ with every shown part's box cleared the picture still covers pixels: hiding draws nothing different."
    } else if p[3].0 == 0 {
        "★★★ ticking Pin, which the file stores hidden, drew nothing: a stored-hidden part cannot be shown."
    } else if p[4] != p[2] {
        "★★★ clearing Pin again did not draw the empty picture: the paint lent for a render was not given back."
    } else if p[6].1 == p[5].1 {
        "★★★ highlighting Pin drew the same picture: the highlight is not painted."
    } else {
        return Ok(None);
    };
    Ok(Some(failure.to_owned()))
}
