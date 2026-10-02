//! `title_bars_follow_the_theme` — every window's title bar turns dark with
//! the Dark preset and light again with Quiet, the main window and an open
//! dialog alike. See `docs/modules/ui-verify/checks/caption_theme.md`.

use std::path::{Path, PathBuf};

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

const OFFSCREEN: &str = "-4200,-4200,1400,900";

const FILE_TAB: &str = "file";
const SETTINGS: &str = "ribbon.item.file.settings";
const APPEARANCE: &str = "settings.heading.appearance";
const DARK: &str = "settings.theme.dark";
const QUIET: &str = "settings.theme.quiet";

/// `caption-theme dark=… windows=…`, one line per re-application.
const CAPTION_EVENT: &str = "caption-theme";

/// See the module documentation.
pub struct TitleBarsFollowTheTheme;

impl Check for TitleBarsFollowTheTheme {
    fn name(&self) -> &'static str {
        "title_bars_follow_the_theme"
    }

    fn defect(&self) -> &'static str {
        "the window title bars stay in the host's colours whatever the theme, so a dark \
         window carries a white caption (OPERATOR_REQUESTS.md O281)"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The scratch binary's `settings.txt`, put back as it was when dropped.
struct SettingsGuard {
    path: PathBuf,
    before: Option<Vec<u8>>,
}

impl SettingsGuard {
    fn quiet(exe: &Path) -> Result<Self> {
        let dir = exe
            .parent()
            .ok_or_else(|| Error::new("the binary has no parent directory"))?
            .join("userdata");
        let path = dir.join("settings.txt");
        let before = std::fs::read(&path).ok();
        std::fs::create_dir_all(&dir)
            .and_then(|()| std::fs::write(&path, "theme = quiet\n"))
            .map_err(|e| Error::new(format!("could not write settings: {e}")))?;
        Ok(Self { path, before })
    }
}

impl Drop for SettingsGuard {
    fn drop(&mut self) {
        let _ = match &self.before {
            Some(bytes) => std::fs::write(&self.path, bytes),
            None => std::fs::remove_file(&self.path),
        };
    }
}

fn click_region(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    name: &str,
) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = driving::declared_in(&trace, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{name}` region. Regions beginning `settings.`: {}.",
            driving::list(&driving::declared_names(&trace, ui_rect, "settings."))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(12);
    Ok(())
}

/// Each visible window of the session's process with its caption mode.
fn captions(session: &Session) -> Vec<(String, Option<bool>)> {
    crate::sys::windows_for_pid(session.pid())
        .into_iter()
        .map(|w| (crate::sys::describe_window(w), crate::sys::dark_caption(w)))
        .collect()
}

/// `None` when every window in `seen` (at least `min` of them) is `dark`.
fn judge(stage: &str, seen: &[(String, Option<bool>)], dark: bool, min: usize) -> Option<String> {
    let mode = if dark { "dark" } else { "light" };
    if seen.len() < min {
        return Some(format!(
            "{stage}: {} window(s) found, {min} expected.",
            seen.len()
        ));
    }
    let wrong: Vec<String> = seen
        .iter()
        .filter(|(_, d)| *d != Some(dark))
        .map(|(name, d)| format!("{name} = {d:?}"))
        .collect();
    (!wrong.is_empty()).then(|| {
        format!(
            "★ {stage}: the theme is {mode} and these title bars are not: {}.",
            wrong.join("; ")
        )
    })
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let _settings = SettingsGuard::quiet(&exe)?;
    let pdf = driving::repo_fixture("cropped-sheets.pdf", "")?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("caption_theme.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("caption_theme.pointer.txt"))?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
    report.artifact(session.trace_path().to_path_buf());
    session.settle(30);
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    let start = captions(&session);
    report.note(format!("at start (Quiet): {start:?}"));
    if let Some(failure) = judge("at start, Quiet", &start, false, 1) {
        return Ok(Some(failure));
    }

    crate::checks::ocr::click_tab(&session, &pointer, ui_rect, FILE_TAB)?;
    let item = driving::declared_or_in_overflow(&session, &pointer, ui_rect, SETTINGS)?
        .ok_or_else(|| Error::new(format!("the File tab declares no `{SETTINGS}`.")))?;
    crate::input::Click::click_rect(&pointer, &session, item)?;
    session.settle(20);
    click_region(&session, &pointer, ui_rect, APPEARANCE)?;

    for (radio, dark) in [(DARK, true), (QUIET, false)] {
        let before = session.trace()?.events(CAPTION_EVENT).count();
        click_region(&session, &pointer, ui_rect, radio)?;
        session.settle(12);
        let trace = session.trace()?;
        let lines: Vec<_> = trace.events(CAPTION_EVENT).skip(before).collect();
        let want = if dark { "true" } else { "false" };
        if !lines.iter().any(|l| l.get("dark") == Some(want)) {
            return Ok(Some(format!(
                "`{radio}` was clicked and no `{CAPTION_EVENT} dark={want}` line followed."
            )));
        }
        let seen = captions(&session);
        report.note(format!("after {radio}: {seen:?}"));
        if let Some(failure) = judge(radio, &seen, dark, 2) {
            return Ok(Some(failure));
        }
    }
    Ok(None)
}
