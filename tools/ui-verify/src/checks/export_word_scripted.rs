//! `export_word_without_the_mouse` — File ▸ Export ▸ Word document… writes
//! `fixtures/ruled-table.pdf` as a `.docx` holding its table as a Word table,
//! found by its rules because the file is untagged.
//! `export_word_follows_the_tags` — the same export of
//! `fixtures/tagged-report.pdf`, whose one table has no rules, takes its
//! heading and table from the file's tags.
//! `export_word_honours_its_choices` — the window's page range, page-break,
//! tables and structure choices each reach the writer. All run on a window
//! placed off the desktop and driven through the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/export_word_scripted.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The line a successful export writes.
const WROTE: &str = "export-word"; // ui-text-exempt: a trace event name, never displayed
const TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.export_word"; // ui-text-exempt: a trace region name, never displayed
/// The Export group when the band is too narrow to show it open.
const COLLAPSED: &str = "ribbon.group.file.export.collapsed"; // ui-text-exempt: a trace region name, never displayed
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// The main part's name, stored uncompressed in the zip's directory.
const MAIN_PART: &[u8] = b"word/document.xml";
/// The window's Export button.
const EXPORT: &str = "export-word.export"; // ui-text-exempt: a trace region name, never displayed

/// One gesture in the Export-to-Word window before Export is pressed.
#[derive(Clone, Copy)]
enum Choice<'a> {
    Click(&'a str),
    /// Click the field `into`, then type `text` there.
    Type {
        into: &'a str,
        text: &'a str,
    },
}

pub struct ExportWordWithoutTheMouse;

impl Check for ExportWordWithoutTheMouse {
    fn name(&self) -> &'static str {
        "export_word_without_the_mouse"
    }

    fn defect(&self) -> &'static str {
        "File ▸ Export ▸ Word document… does not write a Word package, or writes the \
         document's table as paragraphs instead of a table"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let expect = [
            ("tables", "1"),
            ("structure", "layout"),
            ("structure_fallback", "no-tree"),
        ];
        match drive(
            ctx,
            &mut report,
            "ruled-table.pdf",
            "export-word",
            &[],
            &expect,
        ) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

pub struct ExportWordFollowsTheTags;

impl Check for ExportWordFollowsTheTags {
    fn name(&self) -> &'static str {
        "export_word_follows_the_tags"
    }

    fn defect(&self) -> &'static str {
        "File ▸ Export ▸ Word document… on a tagged PDF ignores its tags and guesses headings and tables from the page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        // The table has no rules, so only the tags can yield it.
        let expect = [
            ("structure", "tree"),
            ("structure_fallback", "none"),
            ("tables", "1"),
            ("headings", "1"),
        ];
        match drive(
            ctx,
            &mut report,
            "tagged-report.pdf",
            "export-word-tagged",
            &[],
            &expect,
        ) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

pub struct ExportWordHonoursItsChoices;

impl Check for ExportWordHonoursItsChoices {
    fn name(&self) -> &'static str {
        "export_word_honours_its_choices"
    }

    fn defect(&self) -> &'static str {
        "Export to Word writes every page whatever range is typed, or breaks pages \
         when told not to, or writes tables when told to write them as text, or \
         follows the tags when told to judge the page layout"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        // Each run's expectation differs from that file's default export:
        // four pages and breaks, one ruled table, the tags followed.
        let runs: [Run; 3] = [
            (
                "four-pages.pdf",
                "export-word-range",
                &[
                    Choice::Type {
                        into: "export-word.pages.range",
                        text: "2-3",
                    },
                    Choice::Click("export-word.page_breaks"),
                ],
                &[("pages", "2"), ("page_breaks", "0")],
            ),
            (
                "ruled-table.pdf",
                "export-word-no-tables",
                &[Choice::Click("export-word.tables")],
                &[("tables", "0"), ("table_option", "0")],
            ),
            (
                "tagged-report.pdf",
                "export-word-layout",
                &[Choice::Click("export-word.structure.layout")],
                &[("structure", "layout"), ("structure_fallback", "disabled")],
            ),
        ];
        let mut failures = Vec::new();
        for (fixture, stem, choices, expect) in runs {
            match drive(ctx, &mut report, fixture, stem, choices, expect) {
                Ok(Some(failure)) => failures.push(failure),
                Ok(None) => {}
                Err(why) => failures.push(why.to_string()),
            }
        }
        if failures.is_empty() {
            report.pass()
        } else {
            report.fail(failures.join(" | "))
        }
    }
}

/// One run: the fixture, the artifact stem, the choices made, the fields required.
type Run = (
    &'static str,
    &'static str,
    &'static [Choice<'static>],
    &'static [(&'static str, &'static str)],
);

/// Exports `fixture` to Word after making `choices` in the window, and
/// requires each `expect` field on the `export-word` line. `stem` names this
/// run's artifacts.
fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    fixture: &str,
    stem: &str,
    choices: &[Choice],
    expect: &[(&str, &str)],
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
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join(fixture);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the fixture is not at {}; it is committed, so this is a broken checkout.",
            pdf.display()
        )));
    }
    let target = ctx.out(&format!("{stem}.docx"));
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "cannot clear {} before the export.",
            target.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}-scripted.trace.txt")));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), target.display().to_string()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer =
        ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}-scripted.pointer.txt")))?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    let fresh = |region: &str| -> Result<Option<crate::geom::LRect>> {
        Ok(declared(&session.trace()?, ui_rect, region))
    };
    let click = |region: &str| -> Result<()> {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, ui_rect, prefix))
            ))
        })?;
        pointer.click_in(&session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(15);
        Ok(())
    };

    click(TAB)?;
    if fresh(ITEM)?.is_none() && fresh(COLLAPSED)?.is_some() {
        click(COLLAPSED)?;
    }
    click(ITEM)?;
    session.settle(20);
    if fresh(EXPORT)?.is_none() {
        return Ok(Some(format!(
            "Word document… opened no window with an Export button. Trace: {}.",
            session.trace_path().display()
        )));
    }
    for choice in choices {
        match *choice {
            Choice::Click(region) => click(region)?,
            Choice::Type { into, text } => {
                click(into)?;
                let trace = session.trace()?;
                let viewport = declared_in(&trace, ui_rect, into).and_then(|(_, vp)| vp);
                pointer.type_text(&session, viewport.as_deref(), text)?;
                session.settle(10);
            }
        }
    }
    click(EXPORT)?;
    session.settle(20);
    pointer.gone(&session)?;

    let trace = session.trace()?;
    let Some(line) = trace.last(WROTE) else {
        return Ok(Some(format!(
            "the item was clicked and no `{WROTE}` line followed. Refused: {}. Failed: {}. \
             Trace: {}.",
            trace
                .last("export-word-refused")
                .map_or("none".to_owned(), |l| l.raw.clone()),
            trace
                .last("export-word-failed")
                .map_or("none".to_owned(), |l| l.raw.clone()),
            session.trace_path().display()
        )));
    };
    if let Some((key, want)) = expect.iter().find(|(k, v)| line.get(k) != Some(v)) {
        return Ok(Some(format!(
            "{fixture} should export with `{key}={want}`; the Word export traced `{}`.",
            line.raw
        )));
    }
    let bytes = std::fs::read(&target).map_err(|e| {
        Error::new(format!(
            "the export traced success and {} cannot be read: {e}",
            target.display()
        ))
    })?;
    if !bytes.starts_with(b"PK\x03\x04") {
        return Ok(Some(format!(
            "{} is not a zip package, and a .docx is one.",
            target.display()
        )));
    }
    if !bytes.windows(MAIN_PART.len()).any(|w| w == MAIN_PART) {
        return Ok(Some(format!(
            "{} holds no `word/document.xml`, so Word cannot open it.",
            target.display()
        )));
    }
    report.artifact(target);
    report.note(format!("`{}`", line.raw));
    Ok(None)
}
