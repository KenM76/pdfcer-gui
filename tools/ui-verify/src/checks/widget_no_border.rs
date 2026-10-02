//! `checks::widget_no_border` — **every kind of form field can lose its
//! border from the Properties panel**
//!
//! For each field of `all-field-kinds.pdf` (text, check box, radio, both
//! drop-downs' kind, list, push button, signature) the seam selects it, the
//! scripted pointer opens the Border combo and presses *No border*, and the
//! panel's re-read of the document must say `border=none width=0.00`. The
//! border colour must be gone too, except on a check box or radio, whose mark
//! is drawn in it. Every field but the push button must also come back
//! `redrawn=yes`: an unsigned signature field and another program's check box
//! or radio are redrawn without the frame. A push button another program drew
//! is kept as it is, which the engine does not cover and the status line says.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused; tall so the border row needs little scrolling.
const OFFSCREEN: &str = "-4200,-4200,1200,1350";
/// Edit mode, then the Properties panel.
const INVOKE: &str = "mode.edit,file.properties";
const FIXTURE: &str = "all-field-kinds.pdf";
const METHOD: &str = "One widget of every field kind pdfcer reads, each with /BS /W 1 and \
                      /MK /BC black — see `fixtures/all-field-kinds.PROVENANCE.py`.";
const PANEL_BODY: &str = "dock.body.file.properties";
const COMBO: &str = "properties.widget_edit.border";
const NONE_ENTRY: &str = "properties.widget_edit.border.none";
const SHOWN: &str = "widget-border-shown";
const APPLIED: &str = "edit-widget-applied";
const MAX_SCROLL: usize = 12;

/// Each field, whether its border colour is the mark's and so stays, and
/// whether its appearance must be redrawn.
const FIELDS: [(&str, bool, bool); 7] = [
    ("TextOne", false, true),
    ("CheckOne", true, true),
    ("RadioGroup", true, true),
    ("ComboOne", false, true),
    ("ListOne", false, true),
    ("PushOne", false, false),
    ("SigOne", false, true),
];

/// See the module documentation.
pub struct EveryFieldKindCanLoseItsBorder;

impl Check for EveryFieldKindCanLoseItsBorder {
    fn name(&self) -> &'static str {
        "every_field_kind_can_lose_its_border"
    }

    fn defect(&self) -> &'static str {
        "the Properties panel's Border control has no No border choice, so a signature field, a \
         button or a check box with a /MK /BC frame keeps it for ever"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut failures = Vec::new();
        for (field, keeps_colour, redrawn) in FIELDS {
            match drive(ctx, &mut report, field, (keeps_colour, redrawn)) {
                Ok(None) => {}
                Ok(Some(failure)) => failures.push(failure),
                Err(why) => return report.from_error(&why),
            }
        }
        if failures.is_empty() {
            report.pass()
        } else {
            report.fail(failures.join("\n"))
        }
    }
}

/// Launch on `field`, press *No border*, and judge the panel's re-read.
fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    field: &str,
    expect: (bool, bool),
) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    // Driven on a copy, so a stray save never touches the repository's fixture.
    let doc = ctx.out(&format!("no-border-{field}.pdf"));
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("no-border-{field}.trace.txt")));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
        ("PDFCER_DIAG_SELECT_FIELD", field),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(
        &mut spec,
        ctx.out(&format!("no-border-{field}.pointer.txt")),
    )?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);

    let mut combo = declared_in(&session.trace()?, ui_rect, COMBO);
    let mut turns = 0;
    while combo.is_none() && turns < MAX_SCROLL {
        let Some((panel, vp)) = declared_in(&session.trace()?, ui_rect, PANEL_BODY) else {
            pointer.gone(&session)?;
            return Err(Error::new(format!(
                "{field}: no `{PANEL_BODY}` region, so the Properties panel is not open to scroll."
            )));
        };
        pointer.wheel_in(&session, vp.as_deref(), WindowPoint::centre_of(panel), -3.0)?;
        session.settle(10);
        turns += 1;
        combo = declared_in(&session.trace()?, ui_rect, COMBO);
    }
    let Some((combo, vp)) = combo else {
        pointer.gone(&session)?;
        return Ok(Some(format!(
            "{field}: no visible `{COMBO}` after {MAX_SCROLL} wheel turns — the field's Border \
             control is not reachable. Trace: {}",
            session.trace_path().display()
        )));
    };
    pointer.click_in(&session, vp.as_deref(), WindowPoint::centre_of(combo))?;
    session.settle(15);
    let Some((entry, evp)) = declared_in(&session.trace()?, ui_rect, NONE_ENTRY) else {
        pointer.gone(&session)?;
        return Ok(Some(format!(
            "★★★ {field}: the open Border combo offers no `{NONE_ENTRY}` entry, so this field's \
             border cannot be turned off. Trace: {}",
            session.trace_path().display()
        )));
    };
    pointer.click_in(&session, evp.as_deref(), WindowPoint::centre_of(entry))?;
    session.settle(30);
    pointer.gone(&session)?;
    let trace = session.trace()?;
    drop(session);
    judge(report, &trace, field, expect)
}

/// The edit applied to `field`, the panel's re-read shows no border and the
/// expected border colour, and the appearance was redrawn where it must be.
fn judge(
    report: &mut CheckReport,
    trace: &crate::trace::Trace,
    field: &str,
    (keeps_colour, redrawn): (bool, bool),
) -> Result<Option<String>> {
    let Some(applied) = trace
        .events(APPLIED)
        .filter(|l| l.get("field") == Some(field))
        .last()
    else {
        return Ok(Some(format!(
            "★★★ {field}: pressing No border applied no edit (no `{APPLIED} field={field}`)."
        )));
    };
    let shown = trace
        .events(SHOWN)
        .filter(|l| l.get("field") == Some(field))
        .last();
    let Some(shown) = shown else {
        return Ok(Some(format!(
            "{field}: the panel traced no `{SHOWN}` line."
        )));
    };
    report.note(format!("{field}: {} | {}", shown.raw, applied.raw));
    let colour = if keeps_colour { "present" } else { "absent" };
    if shown.get("border") != Some("none")
        || shown.get("width") != Some("0.00")
        || shown.get("border_colour") != Some(colour)
    {
        return Ok(Some(format!(
            "★★★ {field}: after No border the panel re-reads `{}`; it must be border=none \
             width=0.00 border_colour={colour}.",
            shown.raw
        )));
    }
    let want = if redrawn { "yes" } else { "no" };
    if applied.get("redrawn") != Some(want) {
        return Ok(Some(format!(
            "★★★ {field}: the edit reports `{}`; it must carry redrawn={want}, or the old frame \
             still shows.",
            applied.raw
        )));
    }
    Ok(None)
}
