//! `open_in_acrobat_hands_the_file_over` — with an Acrobat path configured,
//! *Open in Acrobat* on the File tab asks first; Cancel leaves the document
//! open, and the proceed button saves any unsaved edits, launches the
//! configured viewer on the document's own file, and then closes it.
//!
//! The configured viewer is a stub compiled at run time, so no real Acrobat
//! ever opens on the desktop the check runs on.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/open_in_acrobat.md`.

use std::path::{Path, PathBuf};

use super::dimdrive::{Fixture, launch_with, press, press_on_tab};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::Session;
use crate::report::CheckReport;

const FILE_TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.trailing.file.open_in_acrobat"; // ui-text-exempt: a trace region name, never displayed
const PROCEED: &str = "open-in-acrobat.proceed"; // ui-text-exempt: a trace region name, never displayed
const CANCEL: &str = "open-in-acrobat.cancel"; // ui-text-exempt: a trace region name, never displayed
/// The variable the stub reads to know where to record its arguments.
const RECORD_ENV: &str = "UIV_ACROBAT_STUB_RECORD"; // ui-text-exempt: an environment variable name
/// A GUI-subsystem program that writes its arguments to `RECORD_ENV` and
/// exits: no window, no console.
const STUB_SOURCE: &str = "#![windows_subsystem = \"windows\"]\n\
fn main() {\n\
    if let Ok(p) = std::env::var(\"UIV_ACROBAT_STUB_RECORD\") {\n\
        let a: Vec<String> = std::env::args().skip(1).collect();\n\
        let _ = std::fs::write(p, a.join(\"\\n\"));\n\
    }\n\
}\n";

const FIXTURE: Fixture = Fixture {
    file: super::dimdrive::FIXTURE,
    method: super::dimdrive::METHOD,
    page: super::dimdrive::PAGE,
};

/// Open in Acrobat asks, cancels cleanly, and hands the file over.
pub struct OpenInAcrobatHandsTheFileOver;

impl Check for OpenInAcrobatHandsTheFileOver {
    fn name(&self) -> &'static str {
        "open_in_acrobat_hands_the_file_over"
    }

    fn defect(&self) -> &'static str {
        "Open in Acrobat is missing from the File tab with a viewer configured, launches \
         without asking, closes the document on Cancel, hands over a file without its unsaved edits, or launches something other than the \
         configured viewer on the document's own file"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let exe = ctx.resolve_exe();
        let _restore = Restore(exe.as_deref());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Leave the profile's preferences as a first run left them, on every path.
struct Restore<'a>(Option<&'a Path>);

impl Drop for Restore<'_> {
    fn drop(&mut self) {
        if let Some(dir) = self.0.and_then(Path::parent) {
            let _ = crate::sandbox::reset_prefs(&dir.join("userdata"));
        }
    }
}

/// Compile the stub viewer into the scratch folder.
fn build_stub(ctx: &CheckContext) -> Result<PathBuf> {
    let src = ctx.out("acrobat-stub.rs");
    let exe = ctx.out("acrobat-stub.exe");
    std::fs::write(&src, STUB_SOURCE)
        .map_err(|e| Error::new(format!("writing {}: {e}", src.display())))?;
    let status = std::process::Command::new("rustc") // ui-text-exempt: an executable name
        .args(["-O", "-o"])
        .arg(&exe)
        .arg(&src)
        .status()
        .map_err(|e| Error::new(format!("rustc could not run to build the stub viewer: {e}")))?;
    if !status.success() || !exe.exists() {
        return Err(Error::new(format!(
            "rustc failed to build the stub viewer from {}",
            src.display()
        )));
    }
    Ok(exe)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let gui = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let stub = build_stub(ctx)?;
    let userdata = gui
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::write_prefs(&userdata, &format!("acrobat_path = {}\n", stub.display()))
        .map_err(|e| Error::new(format!("could not write preferences: {e}")))?;
    let record = ctx.out("acrobat-stub.args.txt");
    let env = [(RECORD_ENV, record.display().to_string())];

    if let Some(failure) = cancel(ctx, report, &env)? {
        return Ok(Some(failure));
    }
    if let Some(failure) = proceed(ctx, report, &env, &record, &CLOSE)? {
        return Ok(Some(failure));
    }
    proceed(ctx, report, &env, &record, &SAVE_FIRST)
}

/// Launch, press the File tab's item, require the question, press Cancel.
fn cancel(
    ctx: &CheckContext,
    report: &mut CheckReport,
    env: &[(&str, String)],
) -> Result<Option<String>> {
    let (session, pointer) = launch_with(ctx, report, &FIXTURE, "", "acrobat-cancel", env)?;
    let path = session.trace_path().display().to_string();
    press_on_tab(ctx, &session, &pointer, ITEM, FILE_TAB)?;
    session.settle(20);
    let trace = session.trace()?;
    let asked = trace.last("acrobat-asked").map(|l| l.raw.clone());
    report.note(format!("asked: {asked:?}"));
    if asked.is_none() || trace.last("acrobat-launched").is_some() {
        pointer.gone(&session)?;
        return Ok(Some(format!(
            "the File tab's Open in Acrobat traced {asked:?} and no question, or launched \
             without one. Trace: {path}."
        )));
    }
    let mark = trace.mark();
    press(ctx, &session, &pointer, CANCEL)?;
    session.settle(20);
    let trace = session.trace()?;
    let cancelled = trace.last_after("acrobat-cancelled", mark).is_some();
    let closed = trace.last_after("close", mark).map(|l| l.raw.clone());
    pointer.gone(&session)?;
    if !cancelled || closed.is_some() {
        return Ok(Some(format!(
            "Cancel traced acrobat-cancelled={cancelled} and closed {closed:?}; want the \
             cancel traced and the document left open. Trace: {path}."
        )));
    }
    Ok(None)
}

/// One proceed run: `invoke` at launch, the question must be `prompt`.
struct Run {
    stem: &'static str,
    invoke: &'static str,
    prompt: &'static str,
}

/// No edits: the question offers to close, and nothing is written.
const CLOSE: Run = Run {
    stem: "acrobat-proceed",
    invoke: "",
    prompt: "prompt=ConfirmClose",
};

/// A rotated page: the question offers to save first, and the file the
/// viewer is given carries the rotation.
const SAVE_FIRST: Run = Run {
    stem: "acrobat-save-first",
    invoke: "pages.rotate_right",
    prompt: "prompt=SaveFirst",
};

/// The question, when it names `run.prompt` and the configured stub;
/// otherwise an error, so proceed is never pressed on a real viewer.
fn asked_of_stub(ctx: &CheckContext, session: &Session, run: &Run) -> Result<usize> {
    let trace = session.trace()?;
    let asked = trace.last("acrobat-asked").map(|l| l.raw.clone());
    let stub = format!("{:?}", ctx.out("acrobat-stub.exe"));
    let ok = asked.as_deref().is_some_and(|a| {
        a.contains("source=Configured") && a.contains(&stub) && a.contains(run.prompt)
    });
    if !ok {
        return Err(Error::new(format!(
            "the question traced {asked:?}; want {} naming the configured stub {stub}. \
             Proceed was not pressed, so no real viewer was launched. Trace: {}.",
            run.prompt,
            session.trace_path().display()
        )));
    }
    Ok(trace.mark())
}

/// What the stub recorded, waiting up to five seconds for it to run.
fn given(record: &Path) -> String {
    for _ in 0..50 {
        if let Ok(text) = std::fs::read_to_string(record) {
            return text;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    String::new()
}

/// Launch, press the item, press the proceed button; require the stub to
/// have been given the document's file and the document to have closed.
fn proceed(
    ctx: &CheckContext,
    report: &mut CheckReport,
    env: &[(&str, String)],
    record: &Path,
    run: &Run,
) -> Result<Option<String>> {
    let _ = std::fs::remove_file(record);
    let (session, pointer) = launch_with(ctx, report, &FIXTURE, run.invoke, run.stem, env)?;
    let path = session.trace_path().display().to_string();
    session.settle(20);
    press_on_tab(ctx, &session, &pointer, ITEM, FILE_TAB)?;
    session.settle(20);
    let mark = match asked_of_stub(ctx, &session, run) {
        Ok(mark) => mark,
        Err(why) => {
            pointer.gone(&session)?;
            return Err(why);
        }
    };
    press(ctx, &session, &pointer, PROCEED)?;
    session.settle(40);
    let trace = session.trace()?;
    let launched = trace
        .last_after("acrobat-launched", mark)
        .map(|l| l.raw.clone());
    let closed = trace.last_after("close", mark).map(|l| l.raw.clone());
    pointer.gone(&session)?;
    report.note(format!(
        "{}: launched {launched:?}; closed {closed:?}",
        run.stem
    ));
    let doc = ctx.out(&format!("{}.pdf", run.stem));
    let given = given(record);
    report.note(format!("{}: the stub was given {given:?}", run.stem));
    let same = !given.is_empty()
        && std::fs::canonicalize(given.trim()).ok() == std::fs::canonicalize(&doc).ok();
    if launched.is_none() || closed.is_none() || !same {
        return Ok(Some(format!(
            "{}: proceed traced launched={launched:?} closed={closed:?}, and the configured \
             viewer was given {given:?}; want the launch, the close, and the viewer given {}. \
             Trace: {path}.",
            run.stem,
            doc.display()
        )));
    }
    Ok(saved(&doc, run))
}

/// After a save-first run, the file on disk must carry the rotation; after a
/// close run, it must not.
fn saved(doc: &Path, run: &Run) -> Option<String> {
    let bytes = std::fs::read(doc).unwrap_or_default();
    let rotated = bytes.windows(10).any(|w| w == b"/Rotate 90");
    let want = run.invoke == SAVE_FIRST.invoke;
    (rotated != want).then(|| {
        format!(
            "{}: the file handed over {} `/Rotate 90`; the save before the hand-over {}.",
            run.stem,
            if rotated { "carries" } else { "lacks" },
            if want {
                "did not happen"
            } else {
                "wrote an edit nobody made"
            }
        )
    })
}
