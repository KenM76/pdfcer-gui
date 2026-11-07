//! `the_ribbon_says_what_its_controls_do` — the page-display words, the Find button,
//! the Selection tab and the collapsed groups' words, driven off the desktop.
//!
//! See `docs/modules/ui-verify/checks/ribbon_rulings.md` for each phase's oracle.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::invisible_text_scripted::press;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const WIDE: &str = "-4200,-4200,1400,900";
/// Window widths to try, widest first, until File collapses its Document group: the
/// ladder decides where that falls, and past it the group leaves for the overflow.
const NARROWING: [u32; 8] = [1800, 1600, 1500, 1400, 1300, 1200, 1100, 1000];
const PAGE_DISPLAY: [&str; 4] = [
    "ribbon.item.view.page_single",
    "ribbon.item.view.page_continuous",
    "ribbon.item.view.page_facing",
    "ribbon.item.view.page_facing_continuous",
];
/// An icon-only button is under this wide, in points; any word makes it wider.
const ICON_ONLY_PTS: f32 = 36.0;
const FORMAT_TAB: &str = "ribbon.tab.format";
const COLLAPSED: &str = "ribbon.group.file.document.collapsed";
/// Wider than "Document" and a chevron alone, and narrower than "Properties, Fonts".
const COLLAPSED_WITH_WORDS_PTS: f32 = 86.0;
/// The word on the page, at this many points, three times along the first line.
const NEEDLE: &str = "needle";
const TEXT_Y: f64 = 600.0;
const TEXT_FROM_X: f64 = 74.0;
const TEXT_TO_X: f64 = 240.0;
/// A label's words in the button font differ by more than this between the two
/// names, and an unchanged label differs by nothing.
const MIN_LABEL_DIFFERENCE_PTS: f32 = 8.0;

/// See the module documentation.
pub struct TheRibbonSaysWhatItsControlsDo;

impl Check for TheRibbonSaysWhatItsControlsDo {
    fn name(&self) -> &'static str {
        "the_ribbon_says_what_its_controls_do"
    }

    fn defect(&self) -> &'static str {
        "a page-display button is a bare picture, Find can only be run from the keyboard, a Format \
         tab with nothing to format keeps that name, or a collapsed group hides what it opens"
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

/// One Letter page with the needle three times on one line.
fn page_pdf() -> Vec<u8> {
    let content = format!("BT /F1 24 Tf 72 {TEXT_Y} Td ({NEEDLE} {NEEDLE} {NEEDLE}) Tj ET");
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 612 792] >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /Contents 4 0 R \
         /Resources << /Font << /F1 5 0 R >> >> >>"
            .to_owned(),
        format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
    ];
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for off in offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    tag: &str,
    viewport: &str,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out(&format!("ribbon_rulings_{tag}.pdf"));
    std::fs::write(&doc, page_pdf())
        .map_err(|e| Error::new(format!("writing the fixture: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("ribbon_rulings_{tag}.trace.txt")));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, viewport),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(
        &mut spec,
        ctx.out(&format!("ribbon_rulings_{tag}.pointer.txt")),
    )?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(90);
    Ok((session, pointer))
}

fn ui_rect_event(ctx: &CheckContext) -> Result<&'static str> {
    ctx.profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report, "wide", WIDE)?;
    for phase in [page_display, find_button, selection_tab] {
        if let Some(failure) = phase(ctx, report, &session, &pointer)? {
            return Ok(Some(failure));
        }
    }
    pointer.gone(&session)?;
    drop(session);
    for width in NARROWING {
        let viewport = format!("-4200,-4200,{width},700");
        let (session, pointer) = launch(ctx, report, "narrow", &viewport)?;
        let verdict = collapsed_words(ctx, report, &session, &pointer)?;
        pointer.gone(&session)?;
        drop(session);
        if let Some(verdict) = verdict {
            return Ok(verdict);
        }
    }
    Err(Error::new(format!(
        "File collapsed its Document group at none of {NARROWING:?} points, so its words were \
         not on trial."
    )))
}

/// Every page-display button carries a word.
fn page_display(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ui_rect_event(ctx)?;
    press(session, pointer, ui_rect, "ribbon.tab.view")?;
    session.settle(20);
    let png = ctx.out("ribbon_rulings_view.png");
    pointer.screenshot(session, &png)?;
    report.artifact(png);
    let trace = session.trace()?;
    for name in PAGE_DISPLAY {
        let Some(button) = declared(&trace, ui_rect, name) else {
            return Ok(Some(format!(
                "★★★ `{name}` is not on the View tab at 1,400 points. Items: {}.",
                list(&declared_names(&trace, ui_rect, "ribbon.item.view.page"))
            )));
        };
        if button.width() <= ICON_ONLY_PTS {
            return Ok(Some(format!(
                "★★★ `{name}` is {:.0} points wide, which is a picture with no word.",
                button.width()
            )));
        }
    }
    Ok(None)
}

/// The match the bar last reported, read from `readout=At { current: n, .. }`.
fn current_match(session: &Session) -> Result<Option<usize>> {
    let trace = session.trace()?;
    let Some(line) = trace.last("find-bar") else {
        return Ok(None);
    };
    let Some((_, tail)) = line.raw.split_once("current: ") else {
        return Ok(None);
    };
    let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
    Ok(digits.parse().ok())
}

/// The button searches as Enter does, steps on a second press, and hands the caret
/// back so that Enter walks on.
fn find_button(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ui_rect_event(ctx)?;
    pointer.key(session, None, "F", Some("ctrl"))?;
    session.settle(20);
    pointer.type_text(session, None, NEEDLE)?;
    session.settle(30);
    if let Some(n) = current_match(session)? {
        return Err(Error::new(format!(
            "typing alone reached match {n}, so the button's press would prove nothing."
        )));
    }
    press(session, pointer, ui_rect, "find-button")?;
    session.settle(40);
    let png = ctx.out("ribbon_rulings_find.png");
    pointer.screenshot(session, &png)?;
    report.artifact(png);
    let trace_path = session.trace_path().display();
    let first = current_match(session)?;
    if first != Some(1) {
        return Ok(Some(format!(
            "★★★ pressing Find on a typed word left the bar at match {first:?}, not 1: the button \
             did not run the search. Trace: {trace_path}."
        )));
    }
    press(session, pointer, ui_rect, "find-button")?;
    session.settle(30);
    let second = current_match(session)?;
    if second != Some(2) {
        return Ok(Some(format!(
            "★★★ a second press of Find left the bar at match {second:?}, not 2. Trace: {trace_path}."
        )));
    }
    pointer.key(session, None, "Enter", None)?;
    session.settle(30);
    let third = current_match(session)?;
    if third != Some(3) {
        return Ok(Some(format!(
            "★★★ Enter after pressing Find left the bar at match {third:?}, not 3: the caret did \
             not go back to the field. Trace: {trace_path}."
        )));
    }
    pointer.key(session, None, "Escape", None)?;
    session.settle(20);
    report.note("Find: the button reached match 1, then 2; Enter reached 3");
    Ok(None)
}

/// Sweep the fixture's line with the text tool, leaving a live text selection.
fn sweep_text(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    tag: &str,
) -> Result<()> {
    pointer.key(session, None, "T", None)?;
    session.settle(15);
    let doc = ctx.out(&format!("ribbon_rulings_{tag}.pdf"));
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let y = TEXT_Y + 6.0;
    let from: WindowPoint = mapping.doc_to_window(DocPoint::new(0, TEXT_FROM_X, y))?;
    let to: WindowPoint = mapping.doc_to_window(DocPoint::new(0, TEXT_TO_X, y))?;
    pointer.drag(session, from, to, 12)?;
    session.settle(30);
    Ok(())
}

/// The one tab reads Selection in Read and Format in Edit.
fn selection_tab(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ui_rect_event(ctx)?;
    pointer.key(session, None, "1", Some("ctrl"))?;
    session.settle(20);
    sweep_text(ctx, session, pointer, "wide")?;
    let trace = session.trace()?;
    let Some(read) = declared(&trace, ui_rect, FORMAT_TAB) else {
        return Ok(Some(format!(
            "★★★ text selected in Read raised no `{FORMAT_TAB}`. Tabs: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab.")),
            session.trace_path().display()
        )));
    };
    let png = ctx.out("ribbon_rulings_selection.png");
    pointer.screenshot(session, &png)?;
    report.artifact(png);
    pointer.key(session, None, "3", Some("ctrl"))?;
    session.settle(20);
    sweep_text(ctx, session, pointer, "wide")?;
    let trace = session.trace()?;
    let Some(edit) = declared(&trace, ui_rect, FORMAT_TAB) else {
        return Ok(Some(format!(
            "★★★ text selected in Edit left no `{FORMAT_TAB}`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "the tab is {:.1} points wide in Read and {:.1} in Edit",
        read.width(),
        edit.width()
    ));
    if read.width() < edit.width() + MIN_LABEL_DIFFERENCE_PTS {
        return Ok(Some(format!(
            "★★★ the tab is {:.1} points wide in Read and {:.1} in Edit, so it has one name in \
             both: with nothing to format it should read Selection, which is the longer word.",
            read.width(),
            edit.width()
        )));
    }
    Ok(None)
}

/// A collapsed group's button says what it opens. The outer `None` means this width
/// did not collapse it.
fn collapsed_words(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<Option<String>>> {
    let ui_rect = ui_rect_event(ctx)?;
    press(session, pointer, ui_rect, "ribbon.tab.file")?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(button) = declared(&trace, ui_rect, COLLAPSED) else {
        return Ok(None);
    };
    let png = ctx.out("ribbon_rulings_collapsed.png");
    pointer.screenshot(session, &png)?;
    report.artifact(png);
    report.note(format!(
        "collapsed Document is {:.1} points wide",
        button.width()
    ));
    if button.width() < COLLAPSED_WITH_WORDS_PTS {
        return Ok(Some(Some(format!(
            "★★★ the collapsed Document button is {:.1} points wide: its caption and chevron \
             alone, with no room for the words saying what it opens.",
            button.width()
        ))));
    }
    Ok(Some(None))
}
