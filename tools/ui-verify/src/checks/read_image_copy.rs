//! `read_mode_copies_a_picture_other_programs_can_paste` — **click a picture
//! while reading, press Ctrl+C, and it is on the Windows clipboard.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/read_image_copy.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode this is about, and the whole point of the row.
const MODE: &str = "read";
/// The selection line.
const SELECTION: &str = "selection-set"; // ui-text-exempt: a trace event name, never displayed
/// The `via=` this arm writes, which no other arm writes.
const VIA: &str = "read-image";
/// The line the copy writes.
const COPIED: &str = "clipboard-copy"; // ui-text-exempt: a trace event name, never displayed
/// The line the picture writes once the clipboard has taken it.
const PICTURE: &str = "clipboard-image"; // ui-text-exempt: a trace event name, never displayed
/// The line written when the picture could not be produced or published.
const DECLINED: &str = "clipboard-image-declined"; // ui-text-exempt: a trace event name, never displayed
/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed
/// The line that says which context menu a right-click resolved.
const MENU: &str = "canvas-menu"; // ui-text-exempt: a trace event name, never displayed
/// The context a reader's right-click on a picture must resolve to.
const READ_MENU: &str = "canvas.read-object"; // ui-text-exempt: a menu context id, never displayed
/// `C`, for the copy chord.
const VK_C: u16 = 0x43;

/// **The fixture: a page that is one image and nothing else.**
const FIXTURE: &str = "../../fixtures/synthetic-image-only.pdf";
/// Its page, as the file declares it.
const FIXTURE_PAGE: PageGeometry = PageGeometry {
    width_pt: 306.0,
    height_pt: 396.0,
};
/// Where to click — the middle of the sheet, which is the middle of the image.
const POINT: (f64, f64) = (153.0, 198.0);

pub struct ReadModeCopiesAPicture;

impl Check for ReadModeCopiesAPicture {
    fn name(&self) -> &'static str {
        "read_mode_copies_a_picture_other_programs_can_paste"
    }

    fn defect(&self) -> &'static str {
        "a picture cannot be selected while reading, and a copy puts a sentence on the Windows \
         clipboard rather than a picture — so pasting into Word or an email gives the words \
         \"3 objects copied\" instead of the drawing"
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
            "input is disabled (--no-input). This check is a click and a chord.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    // Its own fixture rather than `--pdf`: the subject is a picture, and the
    // sweep's drawing is vector geometry with no image on it at all.
    let pdf = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the fixture {FIXTURE} is missing, so there is no picture to click on."
        )));
    }
    let page = crate::fixture::page_geometry(&pdf).unwrap_or(FIXTURE_PAGE);

    let mut spec = LaunchSpec::new(&exe, ctx.out("read-image-copy.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} on a page that is one picture",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    let driver = Driver::new(session.window());

    // --- A: Read mode, and a click selects the picture ---------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }
    driver.click_at(aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, POINT.0, POINT.1),
    )?)?;
    session.settle(30);

    let trace = session.trace()?;
    if !trace
        .events(SELECTION)
        .any(|l| l.get("via") == Some(VIA) && l.get("object").is_some())
    {
        return Ok(Some(format!(
            "★★★ THE PICTURE COULD NOT BE SELECTED WHILE READING: no `{SELECTION} … via={VIA}` \
             line after clicking the middle of a page that is one image.\n\
             The click was consumed by the text sweep, which owns every press in Read — so the \
             image arm in `canvas::clicking` is missing, or it sits BELOW \
             `textsel::takes_the_press`, which in this mode is a rung that never runs. Trace: \
             {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ a click in Read selected the picture");

    // --- B: Ctrl+C copies it -----------------------------------------------
    driver.press_chord(&[crate::sys::vk::CONTROL], VK_C)?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(copied) = trace.last(COPIED) else {
        return Ok(Some(format!(
            "THE COPY DID NOT HAPPEN: `Ctrl+C` with the picture selected produced no `{COPIED}` \
             line. Copy is permitted in every mode by design — *copying is not authoring* — so \
             this says the chord did not reach `dispatch::clipboard` or the selection was gone \
             by the time it did. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★★ the copy happened: `{}`", copied.raw));

    // --- C: …and a PICTURE went on the clipboard ------------------------
    let Some(picture) = trace.last(PICTURE) else {
        let why = if trace.events(DECLINED).count() > 0 {
            "The application traced `clipboard-image-declined`, so it TRIED and could not: the \
             render refused, the clip was degenerate, or Windows would not give up the \
             clipboard (another process may hold it — re-run before believing this one)."
        } else {
            "Nothing tried at all, so `canvas::clipimage::publish` is not being called from the \
             copy path — the operator gets the marker sentence in Word, which is what this row \
             exists to replace."
        };
        return Ok(Some(format!(
            "★★★ NO PICTURE REACHED THE CLIPBOARD: no `{PICTURE}` line. {why} Trace: {}.",
            session.trace_path().display()
        )));
    };
    let (Some(w), Some(h)) = (picture.get_usize("w"), picture.get_usize("h")) else {
        return Ok(Some(format!(
            "`{PICTURE}` was traced without both dimensions, so this check cannot say what was \
             copied: `{}`. Trace: {}.",
            picture.raw,
            session.trace_path().display()
        )));
    };
    // The floor, not an exact size. The scale is chosen from the selection's
    // own extent (`canvas::clipimage`), so pinning the pixels would pin an
    // arithmetic this check has no business owning — but a picture SMALLER than
    // the selection is in points is one the operator cannot use, and that is
    // the property worth asserting.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a page dimension in points, compared against a pixel count" // ui-text-exempt: a lint justification, never displayed
    )]
    let floor = (page.width_pt.min(page.height_pt) as usize).min(w.min(h) + 1);
    if w.min(h) < floor {
        return Ok(Some(format!(
            "★★ THE PICTURE IS SMALLER THAN THE THING COPIED: {w}×{h} px for a selection on a \
             {:.0}×{:.0} pt page. `canvas::clipimage::MIN_SCALE` exists to stop exactly this — a \
             copy that pastes smaller than what is on screen cannot be scaled back up. Trace: \
             {}.",
            page.width_pt,
            page.height_pt,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ …and a {w}×{h} px picture went on the Windows clipboard with it"
    ));

    // --- D: the route somebody finds without being told ------------------
    driver.right_click_at(aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, POINT.0, POINT.1),
    )?)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(menu) = trace.last(MENU) else {
        return Ok(Some(format!(
            "★★ A RIGHT-CLICK IN READ OPENED NOTHING: no `{MENU}` line at all. Until \
             2026-09-01 `canvas::interact` asked `caps.edit_content` before asking WHICH \
             menu, so a right-click anywhere in Read was discarded — including on paper, \
             where the view menu is the correct answer and that file's own comment says \
             so. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if menu.get("context") != Some(READ_MENU) {
        return Ok(Some(format!(
            "★★ THE READER GOT THE WRONG MENU: `{}`, where `{READ_MENU}` was expected. \
             `canvas.object` is the editing menu — Delete, unshare, re-aim to the container \
             — every row of which this mode refuses, and R9 renders nothing rather than \
             greying a list a mode could not honour. `canvas.empty` would mean the \
             right-click resolved no object at all, so the hit test missed the picture the \
             click in step A found. Trace: {}.",
            menu.raw,
            session.trace_path().display()
        )));
    }
    report.note("★★ …and a right-click offers it, which is the route nobody has to be told about");
    Ok(None)
}
