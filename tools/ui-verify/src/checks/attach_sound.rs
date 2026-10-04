//! `a_sound_attaches_as_an_icon` — Markup ▸ Attach sound, a click on the page,
//! the picker's WAV, the Microphone icon, the resample choice and Add put a
//! `/Sound` icon on the page carrying the converted recording.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/attach_sound.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared_in, declared_names, list, repo_fixture,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const FIXTURE: &str = "layer-assign.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/layer-assign.PROVENANCE.py`.";
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const SOUND_ENV: &str = "PDFCER_DIAG_SOUND_PATH"; // ui-text-exempt: an environment variable name
/// The recording's rate, Hz. Resampling must bring it to 22,050.
const RATE: u32 = 44_100;
/// Its length in stereo frames: a tenth of a second.
const FRAMES: u32 = 4_410;
/// An empty spot on the fixture's 800 x 600 page, clear of its box and annotation.
const SPOT: (f64, f64) = (150.0, 450.0);
const TAB: &str = "ribbon.tab.markup";
const ITEM: &str = "ribbon.item.markup.sound";
const MIC: &str = "text-annot.sound-icon.Mic";
const RESAMPLE: &str = "text-annot.sound-resample";
const ACCEPT: &str = "text-annot.accept";

/// See the module documentation.
pub struct ASoundAttachesAsAnIcon;

impl Check for ASoundAttachesAsAnIcon {
    fn name(&self) -> &'static str {
        "a_sound_attaches_as_an_icon"
    }

    fn defect(&self) -> &'static str {
        "Markup > Attach sound does not arm, a click on the page never asks for a recording or \
         never opens the window, the chosen icon or the resample choice is lost, or Add never \
         reaches add_sound_annotation"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report).and_then(|(session, pointer)| {
            let outcome = drive(ctx, &mut report, &session, &pointer);
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
}

macro_rules! step {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(why) => return Ok(Some(why)),
        }
    };
}

/// A 16-bit stereo PCM WAV at [`RATE`] of [`FRAMES`] frames.
fn wav() -> Vec<u8> {
    let data_len = FRAMES * 4;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 4).to_le_bytes());
    out.extend_from_slice(&4u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for frame in 0..FRAMES {
        let v = i16::try_from(frame % 1000).unwrap_or(0);
        out.extend_from_slice(&v.to_le_bytes());
        out.extend_from_slice(&(-v).to_le_bytes());
    }
    out
}

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let payload = ctx.out("attach-sound.payload.wav");
    std::fs::write(&payload, wav())
        .map_err(|e| Error::new(format!("cannot write {}: {e}", payload.display())))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("attach_sound.trace.txt"));
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
        .push((SOUND_ENV.to_owned(), payload.to_string_lossy().into_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("attach_sound.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(40);
    Ok((session, pointer))
}

/// Click a declared region, or the failure naming the regions under `family`.
fn press(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    region: &str,
    family: &str,
) -> Result<std::result::Result<(), String>> {
    let trace = session.trace()?;
    let Some((r, viewport)) = declared_in(&trace, ui_rect, region) else {
        return Ok(Err(format!(
            "no `{region}` region. Regions beginning `{family}`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, family)),
            session.trace_path().display()
        )));
    };
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(r))?;
    session.settle(20);
    Ok(Ok(()))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))?;
    click_mode_segment(session, pointer, ui_rect, "review")?;
    session.settle(20);
    step!(press(session, pointer, ui_rect, TAB, "ribbon.tab.")?);
    step!(press(
        session,
        pointer,
        ui_rect,
        ITEM,
        "ribbon.item.markup."
    )?);
    let trace = session.trace()?;
    if !trace
        .events("markup-tool")
        .any(|l| l.get("tool").is_some_and(|t| t.contains("Sound")))
    {
        return Ok(Some(format!(
            "Markup > Attach sound traced no `markup-tool tool=…Sound…` line, so it armed \
             nothing. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let pdf = repo_fixture(FIXTURE, METHOD)?;
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, SPOT.0, SPOT.1))?,
    )?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(open) = trace.events("sound-annot-open").next() else {
        return Ok(Some(format!(
            "the click on the page traced no `sound-annot-open`: the recording was never asked \
             for or the window never opened. Picker: {}.",
            trace
                .events("sound-picked")
                .last()
                .map_or("no `sound-picked` line", |l| l.raw.as_str())
        )));
    };
    report.note(open.raw.clone());
    step!(press(session, pointer, ui_rect, MIC, "text-annot.")?);
    step!(press(session, pointer, ui_rect, RESAMPLE, "text-annot.")?);
    step!(press(session, pointer, ui_rect, ACCEPT, "text-annot.")?);
    session.settle(20);
    placed(session, report)
}

/// The read line must carry the resampled rate, the source's two 16-bit
/// channels, one conversion and the Mic; the placed line a non-zero id.
fn placed(session: &Session, report: &mut CheckReport) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(read) = trace.events("sound-annot-read").next() else {
        return Ok(Some(format!(
            "Add traced no `sound-annot-read`: the window's accept never reached the place \
             action. Refused: {}.",
            trace
                .events("sound-annot-refused")
                .chain(trace.events("sound-annot-unreadable"))
                .next()
                .map_or("none", |l| l.raw.as_str())
        )));
    };
    report.note(read.raw.clone());
    let want = [
        ("rate", "22050"),
        ("channels", "2"),
        ("bits", "16"),
        ("conversions", "1"),
        ("icon", "Mic"),
    ];
    if want.iter().any(|(k, v)| read.get(k) != Some(*v)) {
        return Ok(Some(format!(
            "★★★ the recording read `{}`; it must carry rate=22050 channels=2 bits=16 \
             conversions=1 icon=Mic.",
            read.raw
        )));
    }
    let Some(line) = trace.events("sound-annot-placed").next() else {
        return Ok(Some(
            "★★★ the recording was read and no `sound-annot-placed` followed, so the engine \
             never authored the icon."
                .to_owned(),
        ));
    };
    report.note(line.raw.clone());
    Ok(line
        .get("id")
        .is_none_or(|id| id == "0")
        .then(|| format!("★★★ the placed line `{}` names no object.", line.raw)))
}
