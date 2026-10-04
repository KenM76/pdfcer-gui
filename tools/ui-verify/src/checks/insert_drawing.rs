//! `insert_image_places_a_drawing` — an SVG and an EMF chosen through Edit ▸
//! Insert image reach the page as vector artwork, and what the import could
//! not carry is said before Insert and after it. Driven through the scripted
//! pointer on a window off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/insert_drawing.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::Click;
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

/// Off the desktop, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const IMAGE_PATH_ENV: &str = "PDFCER_DIAG_IMAGE_PATH"; // ui-text-exempt: an environment variable name
const MODE: &str = "edit"; // ui-text-exempt: a ribbon mode id
const TAB: &str = "edit"; // ui-text-exempt: a ribbon tab id
const COMMAND: &str = "ribbon.item.edit.insert_image"; // ui-text-exempt: a trace region name
const INSERT: &str = "insert-image.insert"; // ui-text-exempt: a trace region name
const IMPORTED: &str = "image-imported"; // ui-text-exempt: a trace event name
const REQUESTED: &str = "insert-image-requested"; // ui-text-exempt: a trace event name
const SELECTED: &str = "selection-set"; // ui-text-exempt: a trace event name

/// One file to place: its fixture (`None` for a PNG this harness encodes),
/// its kind token, a word its import notes must carry (`None` when the import
/// is exact), and a phrase the placement's disclosures must carry.
struct Case {
    fixture: Option<&'static str>,
    kind: &'static str,
    note: Option<&'static str>,
    says: &'static str,
}

const CASES: [Case; 3] = [
    Case {
        fixture: Some("vector-art.svg"),
        kind: "svg",
        note: Some("text"), // ui-text-exempt: matched against the engine's note
        says: "placed as vector artwork", // ui-text-exempt: matched against a disclosure
    },
    Case {
        fixture: Some("vector-art.emf"),
        kind: "emf",
        note: None,
        says: "placed as vector artwork", // ui-text-exempt: matched against a disclosure
    },
    // The raster path through the same window, which the drawing work
    // reshaped: the fit choice and the resolution preview are raster-only.
    Case {
        fixture: None,
        kind: "image",
        note: None,
        says: " dpi", // ui-text-exempt: matched against a disclosure
    },
];

/// The file the picker seam names for `case`.
fn picked(ctx: &CheckContext, case: &Case) -> Result<std::path::PathBuf> {
    if let Some(name) = case.fixture {
        return driving::repo_fixture(name, "See fixtures/vector-art.PROVENANCE.md.");
    }
    let path = ctx.out("insert_drawing.raster.png");
    let pixels: Vec<u8> = (0..16u8).flat_map(|i| [i * 16, 0, 255 - i * 16]).collect();
    let png = crate::png::encode_rgb(4, 4, &pixels)
        .ok_or_else(|| Error::new("the harness's PNG encoder refused a 4 x 4 picture."))?;
    std::fs::write(&path, png)
        .map_err(|e| Error::new(format!("cannot write {}: {e}", path.display())))?;
    Ok(path)
}

/// See the module documentation.
pub struct InsertImagePlacesADrawing;

impl Check for InsertImagePlacesADrawing {
    fn name(&self) -> &'static str {
        "insert_image_places_a_drawing"
    }

    fn defect(&self) -> &'static str {
        "Insert image refuses an SVG or EMF, rasterises it, places it without saying it is \
         vector artwork, or drops the import's note about what the drawing lost"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        for case in &CASES {
            match assess(ctx, &mut report, case) {
                Ok(Some(failure)) => return report.fail(failure),
                Ok(None) => {}
                Err(why) => return report.from_error(&why),
            }
        }
        report.pass()
    }
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    case: &Case,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let viewport_env = ctx
        .profile
        .viewport_env
        .ok_or_else(|| Error::new("the profile has no viewport variable."))?;
    let pdf = driving::repo_fixture(
        "pure-k-square.pdf",
        "A one-page vector fixture every writer accepts, so --pdf is ignored.",
    )?;
    let drawing = picked(ctx, case)?;
    let stem = format!("insert_drawing.{}", case.kind);
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((IMAGE_PATH_ENV.to_owned(), drawing.display().to_string()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched with {} as pid {}",
        drawing.display(),
        session.pid()
    ));
    session.settle(30);
    Ok((session, pointer))
}

/// Open Insert image with the drawing and press Insert.
fn drive(session: &Session, pointer: &ScriptedPointer, ui_rect: &str) -> Result<()> {
    driving::click_mode_segment(session, pointer, ui_rect, MODE)?;
    crate::checks::ocr::click_tab(session, pointer, ui_rect, TAB)?;
    let Some(item) = driving::declared_or_in_overflow(session, pointer, ui_rect, COMMAND)? else {
        return Err(Error::new(format!(
            "the Edit tab declares no `{COMMAND}`, on the band or in a collapsed group."
        )));
    };
    pointer.click_rect(session, item)?;
    session.settle(40);
    // The dialog is its own OS window; its rects are relative to it.
    let Some((insert, viewport)) = driving::declared_in(&session.trace()?, ui_rect, INSERT) else {
        return Err(Error::new(format!(
            "the command was clicked and no `{INSERT}` button was declared. Trace: {}.",
            session.trace_path().display()
        )));
    };
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(insert))?;
    session.settle(60);
    pointer.gone(session).map(|_| ())
}

fn assess(ctx: &CheckContext, report: &mut CheckReport, case: &Case) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report, case)?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    let kind = case.kind;
    if let Some(line) = session.trace()?.last(IMPORTED)
        && line.get("kind") != Some(kind)
    {
        return Ok(Some(format!(
            "★ the picker's file imported as `{}`.",
            line.raw
        )));
    }
    drive(&session, &pointer, ui_rect)?;
    let trace = session.trace()?;
    let Some(imported) = trace.last(IMPORTED) else {
        return Ok(Some(format!(
            "★ `{}` was picked and no `{IMPORTED}` line was traced, so the importer refused it.",
            case.kind
        )));
    };
    report.note(format!("`{}`", imported.raw));
    if imported.get("kind") != Some(kind) {
        return Ok(Some(format!(
            "★ `{}` imported as `{}`; it should be `kind={kind}`.",
            case.kind, imported.raw
        )));
    }
    let notes = imported.get("notes").unwrap_or("none");
    if let Some(word) = case.note
        && !notes.contains(word)
    {
        return Ok(Some(format!(
            "★★ the import of `{}` should note what it did not carry (`{word}`); it traced \
             `notes={notes}`.",
            case.kind
        )));
    }
    let requested = trace.last(REQUESTED);
    if requested.and_then(|l| l.get("kind")) != Some(kind) {
        return Ok(Some(format!(
            "★ Insert was pressed and no `{REQUESTED} kind={kind}` followed. Trace: {}.",
            session.trace_path().display()
        )));
    }
    // A raster promises a resolution; a drawing has none to promise.
    let dpi = requested.and_then(|l| l.get("dpi")).unwrap_or("absent");
    if (kind == "image") != dpi.parse::<f64>().is_ok() {
        return Ok(Some(format!(
            "★★ the `{kind}` request traced `dpi={dpi}`: a picture must preview a resolution and \
             a drawing must not."
        )));
    }
    judge_placement(report, &trace, case)
}

/// The engine placed it, its disclosures carry the case's phrase and repeat
/// the import's note, and the placed object was selected.
fn judge_placement(
    report: &mut CheckReport,
    trace: &crate::trace::Trace,
    case: &Case,
) -> Result<Option<String>> {
    let label = format!("add-{}", case.kind);
    if let Some(refused) = trace.last(&format!("{label}-refused")) {
        return Ok(Some(format!(
            "★★ the engine refused the drawing: `{}`.",
            refused.raw
        )));
    }
    let Some(applied) = trace.last(&label) else {
        return Ok(Some(format!(
            "★★ the window asked for the drawing and no `{label}` line followed, so the \
             action reached no placement verb."
        )));
    };
    if !applied.raw.contains(case.says) {
        return Ok(Some(format!(
            "★★★ the placement's disclosures should carry `{}`: `{}`.",
            case.says, applied.raw
        )));
    }
    if let Some(word) = case.note
        && !applied.raw.contains(word)
    {
        return Ok(Some(format!(
            "★★★ the placement drops the import's note (`{word}`): `{}`.",
            applied.raw
        )));
    }
    report.note(format!("`{}`", applied.raw));
    if !trace
        .events(SELECTED)
        .any(|l| l.lineno > applied.lineno && l.get("via") == Some("placed"))
    {
        return Ok(Some(format!(
            "the drawing was placed and not selected (no `{SELECTED} via=placed` after `{label}`), \
             so the operator's first press on it draws a marquee instead of resizing it."
        )));
    }
    Ok(None)
}
