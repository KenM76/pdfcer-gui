//! `embedding_works_with_no_font_folder_at_all` — **pdfcer's own fourteen faces
//! answer when nothing of the operator's can, AND only when he asks.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/embed_bundled.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, frame_of, list, stable_rect,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The command, invoked through the harness seam.
const INVOKE: &str = "mode.edit,tools.embed_fonts";
/// The variable this check deliberately does **not** set.
#[allow(dead_code)]
const DELIBERATELY_UNSET: &str = "PDFCER_DIAG_FONT_DIR";
/// The window body's region.
const BODY: &str = "embed.body";
/// The Embed button's region.
const BUTTON: &str = "embed.commit";
/// The line the window writes when it opens, carrying its plan's counts.
const OPENED: &str = "embed-fonts-opened";
/// The line the apply arm writes when the engine has embedded.
const APPLIED: &str = "embed-fonts-applied";
/// The checkbox that offers pdfcer's own faces. Off when the window opens.
const OWN_FONTS_BOX: &str = "embed.use-own-fonts";
/// The line the checkbox writes when it is ticked or unticked.
const OWN_FONTS_TOGGLED: &str = "embed-fonts-own-fonts";
/// The line the window writes when the Embed button is pressed, carrying the
/// position of the switch that chose the request.
const REQUESTED: &str = "embed-fonts-requested";
/// The line the dispatcher writes when there is nothing to open.
const DECLINED: &str = "embed-fonts-declined";

/// See the module documentation.
pub struct EmbeddingWorksWithNoFontFolderAtAll;

impl Check for EmbeddingWorksWithNoFontFolderAtAll {
    fn name(&self) -> &'static str {
        "embedding_works_with_no_font_folder_at_all"
    }

    fn defect(&self) -> &'static str {
        "with no font folder configured, Embed fonts either cannot reach pdfcer's own fourteen \
         standard faces at all — telling an operator to go and find a font pdfcer is already \
         carrying — or reaches them WITHOUT being asked, putting a substitute face and its \
         licence into a document he sends out on a press he thought was a no-op"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check's subject is a click on the Embed \
             button.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check needs a document naming a STANDARD-14 font it does not carry \
             — Helvetica, Times or Courier — which is what every CAD exporter writes.",
        )
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("embed-bundled.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    // No `PDFCER_DIAG_FONT_DIR`. That absence IS the check.
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with NO font folder configured — the point of this run",
        exe.display(),
        session.pid()
    ));
    session.settle(60);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    if let Some(declined) = trace.events(DECLINED).last() {
        return Err(Error::new(format!(
            "this fixture carries every font it names, so there was nothing to embed: `{}`. \
             Aim this check at a document that NAMES a standard-14 face it does not carry — \
             `fixtures\\a1-titleblock.pdf` is one. Trace: {}.",
            declined.raw,
            session.trace_path().display()
        )));
    }
    if declared(&trace, ui_rect, BODY).is_none() {
        return Ok(Some(format!(
            "EMBED FONTS WAS INVOKED AND NO WINDOW APPEARED: no `{BODY}` region and no \
             `{DECLINED}` line. Regions beginning `embed`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "embed")),
            session.trace_path().display()
        )));
    }
    let Some(opened) = trace.events(OPENED).last() else {
        return Ok(Some(format!(
            "the window drew and published no `{OPENED}` line, so this check cannot tell a \
             correctly greyed button from a broken one. That line is what makes the difference \
             readable; see `dialogs::embed::open`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    // ── POSITION 1: the offer was made, and it was DECLINED ───────────────────
    //
    //
    // What replaces that oracle is `own_fonts_offered`: the number of fonts
    // pdfcer holds a copy of and is OFFERING. Zero there is the bundled rung
    // being unreachable, which is the original defect, still asserted.
    if opened.get("own_fonts_on") != Some("false") {
        return Ok(Some(format!(
            "★★★ THE WINDOW OPENED WITH PDFCER'S OWN FACES ALREADY IN THE PLAN: `{}`.\n\
             The switch must be OFF when the window opens. Embedding one of pdfcer's fourteen \
             substitutes changes what the letters look like on the screen of whoever the \
             document is sent to, and carries a BSD-3-Clause attribution condition into a file \
             the operator distributes — `pdfcer`'s own CLI keeps `--use-bundled-fonts` off for \
             precisely that, and calls it the operator's decision to make. A press of Embed with \
             nothing ticked must not make it for him. See `EmbedDialog::open`. Trace: {}.",
            opened.raw,
            session.trace_path().display()
        )));
    }
    let offered: usize = opened
        .get("own_fonts_offered")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if offered == 0 {
        return Ok(Some(format!(
            "★★★ THE WINDOW OFFERED NONE OF PDFCER'S OWN FACES: `{}`.\n\
             With no folder configured, `own_fonts_offered` counts exactly what pdfcer can \
             supply from its OWN faces for the standard-14 names this document is missing — so \
             zero is the bundled rung not firing. `Library::donor_for` answered nothing for a \
             standard-14 name, which is `allow_bundled` not reaching `resolve_for_embedding`, \
             and the operator is being told to go and find a font pdfcer is holding. \
             ★ If it is zero because the box is drawn but the trace field was never added, that \
             is `EmbedDialog::open`'s trace line, not the resolver. Trace: {}.",
            opened.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the window offered {offered} of pdfcer's own face(s) with no folder set, and \
         defaulted to declining them: `{}`",
        opened.raw
    ));

    // ── POSITION 2: tick the box ──────────────────────────────────
    let Some(checkbox) = stable_rect(&session, ui_rect, OWN_FONTS_BOX, 8)? else {
        return Ok(Some(format!(
            "the window offered {offered} of pdfcer's own faces and drew NO CHECKBOX to accept \
             them: no `{OWN_FONTS_BOX}` region. An offer the operator cannot accept is worse \
             than no offer — it names a remedy and withholds it. Regions beginning `embed`: {}. \
             Trace: {}.",
            list(&declared_names(&trace, ui_rect, "embed")),
            session.trace_path().display()
        )));
    };
    let trace = session.trace()?;
    let frame = frame_of(&session, &trace, ui_rect, OWN_FONTS_BOX)?;
    driver.click_at(frame.declared_center(checkbox))?;
    session.settle(60);
    let trace = session.trace()?;
    if trace
        .events(OWN_FONTS_TOGGLED)
        .filter(|line| line.get("on") == Some("true"))
        .last()
        .is_none()
    {
        return Ok(Some(format!(
            "the checkbox was clicked and did not change: no `{OWN_FONTS_TOGGLED} on=true` line. \
             Its rect was declared and aimed at, so this is the click not reaching it. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ the operator ticked the box — pdfcer's own faces are now in the plan");

    let Some(button) = stable_rect(&session, ui_rect, BUTTON, 8)? else {
        return Ok(Some(format!(
            "the window drew its body and declared no `{BUTTON}` region. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let trace = session.trace()?;
    let frame = frame_of(&session, &trace, ui_rect, BUTTON)?;
    driver.click_at(frame.declared_center(button))?;
    session.settle(60);

    let trace = session.trace()?;
    match trace.events(REQUESTED).last() {
        Some(line) if line.get("own_fonts_on") == Some("true") => {
            report.note(format!(
                "★ the request sent was the one the box chose: `{}`",
                line.raw
            ));
        }
        Some(line) => {
            return Ok(Some(format!(
                "★★★ THE BOX WAS TICKED AND THE OTHER REQUEST WAS SENT: `{}`.\n\
                 `EmbedDialog::active` picks the plan the window is showing AND the request the \
                 button commits, so that the two cannot be chosen by different code. \
                 `own_fonts_on=false` here means the switch changes what is drawn and not what \
                 is done — a window showing one thing and doing another, which is the exact \
                 property `embed_preview` was designed to give this dialog for free. Trace: {}.",
                line.raw,
                session.trace_path().display()
            )));
        }
        None => {
            return Ok(Some(format!(
                "the Embed button was clicked and the window recorded no request: no \
                 `{REQUESTED}` line. Trace: {}.",
                session.trace_path().display()
            )));
        }
    }
    let Some(applied) = trace.events(APPLIED).last() else {
        return Ok(Some(format!(
            "the Embed button was clicked and nothing reached the document: no `{APPLIED}` \
             line. The window had targets, so this is the action or its apply arm rather than \
             the resolver. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the engine embedded: `{}`", applied.raw));

    // --- the oracle ---------------------------------------------------------
    let embedded: usize = applied
        .get("embedded")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if embedded == 0 {
        return Ok(Some(format!(
            "the embed ran and embedded nothing: `{}`. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    if applied.get("substituted") != Some("true") {
        return Ok(Some(format!(
            "★★★ {embedded} FONT(S) WERE EMBEDDED FROM PDFCER'S OWN FACES AND REPORTED AS NOT \
             SUBSTITUTED: `{}`.\n\
             With no folder configured every donor is a bundled face, so `substituted=false` \
             means the shell told the engine `FontMatch::Exact` for one. That is not a wording \
             defect. `is_substitute` is the predicate the engine's SYMBOLIC-FONT GUARD turns \
             on, so understating the rung disables that guard from the outside — and it loses \
             the disclosure the operator's *\"yes\"* to O47 was conditional on. See `rung()` and \
             the `Match::Bundled` arm in `dialogs::embed`. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ {embedded} font(s) embedded from pdfcer's own faces, with no folder configured, and \
         every one disclosed as a substitute — still missing afterwards: {}",
        applied.get("missing_after").unwrap_or("?")
    ));
    Ok(None)
}
