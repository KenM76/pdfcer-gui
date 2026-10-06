//! `a_new_radio_buttons_mark_is_chosen` and `a_new_check_boxs_mark_is_chosen`
//! — the new-field window's Mark picker authors a radio button, or a check
//! box, drawn with a star, read back from the file.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/new_radio_mark.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, declared_names, list, repo_fixture};
use crate::checks::properties_pane;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const FIXTURE: &str = "layer-assign.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/layer-assign.PROVENANCE.py`.";
const OFFSCREEN: &str = "-4200,-4200,1400,980";
/// The field kind one check authors.
struct Kind {
    /// Edit mode, the Properties panel, then the kind's tool.
    invoke: &'static str,
    /// The name `formdraft::next_free` gives the first field of the kind on a
    /// page with no fields; the selection seam waits for it to exist.
    name: &'static str,
    /// Prefix of the trace and pointer artefacts.
    stem: &'static str,
}

const RADIO: Kind = Kind {
    invoke: "mode.edit,file.properties,edit.form_radio_button",
    name: "Group1",
    stem: "new_radio_mark",
};

/// The default name holds a space; the trace reader splits only at ` key=`,
/// so `name=Check Box1` reads whole.
const CHECK_BOX: Kind = Kind {
    invoke: "mode.edit,file.properties,edit.form_check_box",
    name: "Check Box1",
    stem: "new_check_box_mark",
};
/// A drag in an empty part of the fixture's 800 x 600 page.
const SWEEP: ((f64, f64), (f64, f64)) = ((560.0, 480.0), (590.0, 450.0));
const COMBO: &str = "dialog.form_field.mark";
/// Star is entry 2 of `CHECK_STYLES`.
const STAR_ENTRY: &str = "dialog.form_field.mark.2";
const ACCEPT: &str = "dialog.form_field.accept";
const OPENED: &str = "form-field-open";
const SHOWN: &str = "widget-mark-shown";

/// See the module documentation.
pub struct ANewRadioButtonsMarkIsChosen;

impl Check for ANewRadioButtonsMarkIsChosen {
    fn name(&self) -> &'static str {
        "a_new_radio_buttons_mark_is_chosen"
    }

    fn defect(&self) -> &'static str {
        "the new-field window offers a radio button no mark, or the pick never reaches \
         add_radio_button, so every new radio button is drawn as a dot"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(ctx, CheckReport::new(self.name(), self.defect()), &RADIO)
    }
}

/// See the module documentation.
pub struct ANewCheckBoxsMarkIsChosen;

impl Check for ANewCheckBoxsMarkIsChosen {
    fn name(&self) -> &'static str {
        "a_new_check_boxs_mark_is_chosen"
    }

    fn defect(&self) -> &'static str {
        "the new-field window's Mark pick never reaches add_check_box, so every new check \
         box is drawn with the default tick"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(
            ctx,
            CheckReport::new(self.name(), self.defect()),
            &CHECK_BOX,
        )
    }
}

fn run(ctx: &CheckContext, mut report: CheckReport, kind: &Kind) -> CheckReport {
    let driven = launch(ctx, &mut report, kind).and_then(|(session, pointer)| {
        let outcome = drive(ctx, &mut report, &session, &pointer, kind.name);
        let parked = pointer.gone(&session);
        match outcome? {
            Some(failure) => Ok(Some(failure)),
            None => parked.map(|_| None),
        }
    });
    match driven {
        Ok(Some(failure)) => report.fail(failure),
        Ok(None) => report.pass(),
        Err(why) => report.from_error(&why),
    }
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    kind: &Kind,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{}.trace.txt", kind.stem)));
    spec.pdf = Some(repo_fixture(FIXTURE, METHOD)?);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), kind.invoke.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_SELECT_FIELD".to_owned(), kind.name.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer =
        ScriptedPointer::attach(&mut spec, ctx.out(&format!("{}.pointer.txt", kind.stem)))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(45);
    Ok((session, pointer))
}

/// Click a declared region, or the failure naming the regions under `family`.
fn press(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    region: &str,
    family: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some((r, viewport)) = declared_in(&trace, ui_rect, region) else {
        return Ok(Some(format!(
            "no `{region}` region. Regions beginning `{family}`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, family)),
            session.trace_path().display()
        )));
    };
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(r))?;
    session.settle(20);
    Ok(None)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    name: &str,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))?;
    let trace = session.trace()?;
    let page = crate::fixture::page_geometry(&repo_fixture(FIXTURE, METHOD)?)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    let at = |p: (f64, f64)| mapping.doc_to_window(DocPoint::new(0, p.0, p.1));
    pointer.drag(session, at(SWEEP.0)?, at(SWEEP.1)?, 8)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(open) = trace.events(OPENED).last() else {
        return Ok(Some(format!(
            "the drag traced no `{OPENED}`: the field tool never opened the new-field window. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(open.raw.clone());
    if open.get("name") != Some(name) {
        return Ok(Some(format!(
            "the window proposed a name other than `{name}`, which the selection seam waits \
             for: `{}`.",
            open.raw
        )));
    }
    for (region, family) in [
        (COMBO, "dialog.form_field"),
        (STAR_ENTRY, "dialog.form_field.mark"),
        (ACCEPT, "dialog.form_field"),
    ] {
        if let Some(failure) = press(session, pointer, ui_rect, region, family)? {
            return Ok(Some(failure));
        }
    }
    session.settle(40);
    // `H` is ZapfDingbats' star, read back from the new widget's `/MK /CA` by
    // the Properties panel once the seam has selected the authored field.
    properties_pane::reads(
        session,
        report,
        (SHOWN, name),
        "after Add",
        &[("mark", "H")],
    )
}
