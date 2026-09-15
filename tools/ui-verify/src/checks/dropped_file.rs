//! `a_dropped_image_reaches_the_placement_window` — **drag-and-drop**, driven
//! through the one seam that can carry it.
//!
//! # What this is for
//!
//! The operator, 2026-08-19: *"also can't drag and drop a jpg file onto a new
//! pdf, and the insert image button doesn't insert it either."*
//!
//! Half of that was false and half was worse than reported.
//!
//! - **The button works.** `insert_image_places_a_picture` drives it end to end
//!   and passes, including on a real JPEG once this harness was taught to feed
//!   it one instead of a PNG it encoded itself.
//! - **The drop did nothing at all.** Nothing in the shell or in `egui-shell`
//!   read `dropped_files`. A file dragged onto the window was ignored, silently,
//!   with no cursor feedback on the way in.
//!
//! ★★ And the second made the first *look* broken. Both were tried in the same
//! minute; only one of them told the operator anything, so the reasonable
//! conclusion from the chair was that pictures do not work.
//!
//! # ★★ Why this check needs an environment seam where others need none
//!
//! Because a drop **cannot be synthesised by moving a mouse**. It originates in
//! Explorer and is delivered by the window manager as an OLE drag-drop
//! transaction; this harness drives a cursor and a keyboard and has no way to
//! begin one. Without `PDFCER_DIAG_DROP_PATH`, drag-and-drop would be the single
//! feature in this shell that R1 cannot reach — implemented, unit-tested, and
//! never once exercised in a running window, which is precisely the state R1
//! exists to forbid.
//!
//! The seam is honest about what it does and does not prove:
//!
//! | proved here | **not** proved here |
//! |---|---|
//! | the classification, the routing, the import, the placement window opening, the disclosure | that Windows delivers the drop to this window at all |
//!
//! That second column is real and is stated rather than glossed. What closes it
//! is the operator dragging a file onto the build, which is how the gap was
//! found in the first place.
//!
//! # The oracle
//!
//! `dropped source=env path=…` — the shell saw a drop — **plus** the placement
//! window's own region. Two lines, because a build that read the drop and then
//! routed it nowhere writes the first and not the second, and that is exactly
//! the shape of the defect being fixed: a file that arrives and is ignored.

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The seam that simulates one drop.
const DROP_PATH_ENV: &str = "PDFCER_DIAG_DROP_PATH";
/// `dropped n=… first=…` or `dropped source=env path=…`.
const DROPPED_EVENT: &str = "dropped";
/// The placement window's own region.
const PLACEMENT_REGION: &str = "dialog:insert-image";
/// The fixture's size, in pixels.
const FIXTURE_W: u32 = 48;
/// See [`FIXTURE_W`].
const FIXTURE_H: u32 = 24;

/// See the module documentation.
pub struct ADroppedImageReachesThePlacementWindow;

impl Check for ADroppedImageReachesThePlacementWindow {
    fn name(&self) -> &'static str {
        "a_dropped_image_reaches_the_placement_window"
    }

    fn defect(&self) -> &'static str {
        "a file dragged onto the window is ignored, silently and with no feedback — which teaches \
         the operator that this program does not accept drops, a conclusion they will not revisit \
         about a program that opens documents for a living"
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
        Error::new("no --pdf. A dropped image needs a page to go on; pass a document.")
    })?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    // The fixture: a PNG this harness encodes, or the file named by
    // `PDFCER_UIV_IMAGE` — the same seam `insert_image` grew on the same day and
    // for the same reason. A drop of a **JPEG** is what the operator reported,
    // so pointing this at one is the run that answers his sentence.
    let fixture = match std::env::var_os("PDFCER_UIV_IMAGE") {
        Some(v) => std::path::PathBuf::from(v),
        None => {
            let path = ctx.out("dropped_fixture.png");
            let px = vec![90u8; (FIXTURE_W * FIXTURE_H * 3) as usize];
            let png = crate::png::encode_rgb(FIXTURE_W, FIXTURE_H, &px)
                .ok_or_else(|| Error::new("the harness's own PNG encoder refused its fixture"))?;
            std::fs::write(&path, &png)
                .map_err(|e| Error::new(format!("cannot write {}: {e}", path.display())))?;
            path
        }
    };
    if !fixture.exists() {
        return Err(Error::new(format!(
            "the fixture image at {} does not exist.",
            fixture.display()
        )));
    }
    report.note(format!("dropping {}", fixture.display()));

    let mut spec = LaunchSpec::new(&exe, ctx.out("dropped_file.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((DROP_PATH_ENV.to_owned(), fixture.display().to_string()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    // No clicks at all. The whole gesture is the drop, which the seam performs
    // on the first frame; everything after it is the application's own doing.
    session.settle(50);

    let trace = session.trace()?;
    let Some(seen) = trace.last(DROPPED_EVENT) else {
        return Ok(Some(format!(
            "★★ THE SHELL NEVER SAW THE DROP. `{DROP_PATH_ENV}` was set and no `{DROPPED_EVENT}` \
             line followed, so `app::dropped::take` either is not being called from the frame or \
             returned before its seam. It must run at the TOP of `eframe::App::ui`, before \
             anything is drawn — `egui` reports drops on the Context rather than on a widget, so \
             there is nowhere else correct to read them. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the shell saw the drop: `{}`", seen.raw));

    if driving::declared(&trace, ui_rect, PLACEMENT_REGION).is_none() {
        return Ok(Some(format!(
            "★ THE DROP WAS SEEN AND NO PLACEMENT WINDOW OPENED. That is the defect this check \
             exists for, one step further in: the file arrived, was classified, and went nowhere. \
             Look at `app::dropped::take`'s `Image` arm and at `frame.rs`'s call site, which must \
             hand the returned path to `dispatch::images::insert_path` — the half of the picker \
             path that was split out on 2026-08-19 precisely so a drop could reuse it. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★ the dropped image opened the SAME placement window the ribbon command opens");
    Ok(None)
}
