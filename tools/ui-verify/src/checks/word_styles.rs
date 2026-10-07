//! Two checks of the word-processor character and paragraph controls inside
//! a text edit (`OPERATOR_REQUESTS.md` O271, O273), driven off the desktop
//! through the scripted pointer:
//!
//! - `ctrl_b_bolds_the_word_at_the_caret` — a caret inside a word, Ctrl+B, and
//!   exactly that word is restyled and stays selected, with the ribbon's Bold
//!   drawn pressed.
//! - `the_ribbon_aligns_the_paragraph_at_the_caret` — a caret in a paragraph,
//!   the Format tab's Align Right, and that paragraph is reflowed right-aligned.
//!
//! Both on `fixtures/paragraph.pdf`: six lines of Helvetica 12 pt from
//! (72, 700), 16 pt apart.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_in, declared_names, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const INVOKE: &str = "mode.edit,edit.text";
const FIXTURE: &str = "paragraph.pdf";
/// Inside `drawing`, the second word of the first line.
pub(crate) const IN_WORD: (f64, f64) = (112.0, 703.0);
/// Inside the third line.
const IN_PARAGRAPH: (f64, f64) = (120.0, 671.0);
const CARET: &str = "text-edit-caret"; // ui-text-exempt: a trace event name, never displayed
const SPAN_APPLIED: &str = "text-span-style-applied"; // ui-text-exempt: a trace event name, never displayed
const DECLINED: &str = "text-style-declined"; // ui-text-exempt: a trace event name, never displayed
const SELECTED: &str = "ribbon-item-selected"; // ui-text-exempt: a trace event name, never displayed
const ALIGNED: &str = "text-align-applied"; // ui-text-exempt: a trace event name, never displayed
const REFLOWED: &str = "reflow-block-applied"; // ui-text-exempt: a trace event name, never displayed
const FORMAT_TAB: &str = "ribbon.tab.format"; // ui-text-exempt: a trace region name, never displayed
const ALIGN_RIGHT: &str = "ribbon.item.format.align_right"; // ui-text-exempt: a trace region name, never displayed
const FONT_COLLAPSED: &str = "ribbon.group.format.font.collapsed"; // ui-text-exempt: a trace region name, never displayed

/// See the module documentation.
pub struct CtrlBBoldsTheWordAtTheCaret;

impl Check for CtrlBBoldsTheWordAtTheCaret {
    fn name(&self) -> &'static str {
        "ctrl_b_bolds_the_word_at_the_caret"
    }

    fn defect(&self) -> &'static str {
        "Ctrl+B inside a text edit did nothing, or restyled the whole line rather than the \
         word the caret was in"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        finish(self.name(), self.defect(), ctx, drive_bold)
    }
}

/// See the module documentation.
pub struct TheRibbonAlignsTheParagraphAtTheCaret;

impl Check for TheRibbonAlignsTheParagraphAtTheCaret {
    fn name(&self) -> &'static str {
        "the_ribbon_aligns_the_paragraph_at_the_caret"
    }

    fn defect(&self) -> &'static str {
        "paragraph alignment was reachable only by reflowing, and the ribbon had no control \
         for it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        finish(self.name(), self.defect(), ctx, drive_align)
    }
}

type Drive = fn(&CheckContext, &mut CheckReport) -> Result<std::result::Result<String, String>>;

fn finish(
    name: &'static str,
    defect: &'static str,
    ctx: &CheckContext,
    drive: Drive,
) -> CheckReport {
    let mut report = CheckReport::new(name, defect);
    match drive(ctx, &mut report) {
        Ok(Ok(note)) => {
            report.note(&note);
            report.pass()
        }
        Ok(Err(failure)) => report.fail(failure),
        Err(why) => report.from_error(&why),
    }
}

/// The launched program, its pointer, and the page mapping.
pub(crate) struct Driven {
    pub(crate) session: Session,
    pub(crate) pointer: ScriptedPointer,
    mapping: CanvasMapping,
    ui_rect: &'static str,
}

/// Launch on a copy of `paragraph.pdf` with the Text tool armed in Edit, off
/// the desktop, with a scripted pointer.
pub(crate) fn launch(ctx: &CheckContext, report: &mut CheckReport, stem: &str) -> Result<Driven> {
    launch_on(ctx, report, stem, FIXTURE).map(|(driven, _)| driven)
}

/// [`launch`] on a copy of `fixtures/<fixture>`; also answers the copy's path.
pub(crate) fn launch_on(
    ctx: &CheckContext,
    report: &mut CheckReport,
    stem: &str,
    fixture: &str,
) -> Result<(Driven, std::path::PathBuf)> {
    launch_invoking(ctx, report, stem, fixture, INVOKE)
}

/// [`launch_on`], running the commands `invoke` names (comma-separated ids)
/// at startup instead of arming the Text tool.
pub(crate) fn launch_invoking(
    ctx: &CheckContext,
    report: &mut CheckReport,
    stem: &str,
    fixture: &str,
    invoke: &str,
) -> Result<(Driven, std::path::PathBuf)> {
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
    let source = crate::fixture::workspace_root()
        .join("fixtures")
        .join(fixture);
    let doc = ctx.out(&format!("{stem}.pdf"));
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {fixture}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.pdf = Some(doc.clone());
    for (k, v) in [
        ctx.profile.diag_env,
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", invoke),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new(format!("could not read a page size from {fixture}.")))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let driven = Driven {
        session,
        pointer,
        mapping,
        ui_rect,
    };
    Ok((driven, doc))
}

impl Driven {
    /// Click at a page point and require a caret.
    pub(crate) fn caret_at(&self, at: (f64, f64)) -> Result<std::result::Result<(), String>> {
        let before = self.session.trace()?.events(CARET).count();
        self.pointer.click(
            &self.session,
            self.mapping.doc_to_window(DocPoint::new(0, at.0, at.1))?,
        )?;
        self.session.settle(20);
        if self.session.trace()?.events(CARET).count() == before {
            return Ok(Err(format!(
                "a click at ({}, {}) on the paragraph opened no caret: no new `{CARET}` line. \
                 Trace: {}.",
                at.0,
                at.1,
                self.session.trace_path().display()
            )));
        }
        Ok(Ok(()))
    }

    /// Whether the last frame declared `region`.
    pub(crate) fn declares(&self, region: &str) -> Result<bool> {
        Ok(declared(&self.session.trace()?, self.ui_rect, region).is_some())
    }

    /// Click a declared region.
    pub(crate) fn click(&self, region: &str) -> Result<()> {
        let trace = self.session.trace()?;
        let (rect, viewport) = declared_in(&trace, self.ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, self.ui_rect, prefix))
            ))
        })?;
        self.pointer.click_in(
            &self.session,
            viewport.as_deref(),
            WindowPoint::centre_of(rect),
        )?;
        self.session.settle(15);
        Ok(())
    }

    pub(crate) fn path(&self) -> String {
        self.session.trace_path().display().to_string()
    }
}

/// Caret inside `drawing`, Ctrl+B; the word alone is restyled and Bold is
/// drawn pressed.
fn drive_bold(
    ctx: &CheckContext,
    report: &mut CheckReport,
) -> Result<std::result::Result<String, String>> {
    let d = launch(ctx, report, "word-styles-bold")?;
    if let Err(why) = d.caret_at(IN_WORD)? {
        return Ok(Err(why));
    }
    // The contextual tab is shown, not raised: raise it so Bold is drawn and
    // can report its pressed state.
    d.click(FORMAT_TAB)?;
    d.pointer.key(&d.session, None, "B", Some("ctrl"))?;
    d.session.settle(40);
    let trace = d.session.trace()?;
    if let Some(line) = trace.events(DECLINED).last() {
        return Ok(Err(format!(
            "★★★★ Ctrl+B in `drawing` was declined: `{}`. Trace: {}.",
            line.raw,
            d.path()
        )));
    }
    let Some(applied) = trace.events(SPAN_APPLIED).last() else {
        return Ok(Err(format!(
            "★★★★ Ctrl+B in `drawing` restyled nothing and declined nothing: no `{SPAN_APPLIED}` \
             line. Trace: {}.",
            d.path()
        )));
    };
    report.note(&applied.raw);
    let pieces_ok = applied.get("applied").is_some_and(|n| n != "0");
    let kept = applied.get("reselected") == Some("true");
    // The pressed state is the shell's event, under the shell's prefix.
    let pressed = shell_trace(&d.session)?
        .events(SELECTED)
        .filter(|l| l.get("id") == Some("format.bold"))
        .last()
        .is_some_and(|l| l.get("selected") == Some("1"));
    d.pointer.gone(&d.session)?;
    if !pieces_ok || !kept {
        return Ok(Err(format!(
            "★★★ the restyle `{}` applied nothing or lost the word's selection. Trace: {}.",
            applied.raw,
            d.path()
        )));
    }
    if !pressed {
        return Ok(Err(format!(
            "★★ the word was made bold but the ribbon's Bold is not drawn pressed: no \
             `{SELECTED} id=format.bold selected=1`. Trace: {}.",
            d.path()
        )));
    }
    Ok(Ok(
        "Ctrl+B made the word at the caret bold, kept it selected, and pressed Bold".to_owned(),
    ))
}

/// Caret in the paragraph, Format ▸ Align Right; one paragraph is reflowed.
fn drive_align(
    ctx: &CheckContext,
    report: &mut CheckReport,
) -> Result<std::result::Result<String, String>> {
    let d = launch(ctx, report, "word-styles-align")?;
    if let Err(why) = d.caret_at(IN_PARAGRAPH)? {
        return Ok(Err(why));
    }
    d.click(FORMAT_TAB)?;
    let trace = d.session.trace()?;
    if declared(&trace, d.ui_rect, ALIGN_RIGHT).is_none()
        && declared(&trace, d.ui_rect, FONT_COLLAPSED).is_some()
    {
        d.click(FONT_COLLAPSED)?;
    }
    d.click(ALIGN_RIGHT)?;
    d.session.settle(40);
    let trace = d.session.trace()?;
    d.pointer.gone(&d.session)?;
    let Some(aligned) = trace.events(ALIGNED).last() else {
        let declined = trace
            .events("reflow-declined")
            .last()
            .map(|l| l.raw.clone());
        return Ok(Err(format!(
            "★★★★ Align Right with the caret in the paragraph aligned nothing: no `{ALIGNED}` \
             line (last reflow decline: {declined:?}). Trace: {}.",
            d.path()
        )));
    };
    report.note(&aligned.raw);
    let reflowed = trace
        .events(REFLOWED)
        .last()
        .is_some_and(|l| l.raw.contains("alignment=Some(Right)"));
    if aligned.get("blocks") != Some("1/1") || !reflowed {
        return Ok(Err(format!(
            "★★★ `{}` did not reflow exactly the caret's paragraph right-aligned. Trace: {}.",
            aligned.raw,
            d.path()
        )));
    }
    Ok(Ok(
        "Align Right on the ribbon reflowed the caret's paragraph right-aligned".to_owned(),
    ))
}
