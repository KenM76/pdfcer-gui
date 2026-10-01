//! `system_theme_follows_the_host` — the Match the system theme (O268) takes
//! its light or dark mode and its accent from Windows. See
//! `docs/modules/ui-verify/checks/system_theme.md`.

use std::path::Path;

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::image::Rgb;
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

const OFFSCREEN: &str = "-4200,-4200,1400,900";

/// The selected mode segment, which is painted in the accent.
const READ_SEGMENT: &str = "ribbon.mode.read";
/// The application's central area, for the window's lightness.
const CENTRAL_PANEL: &str = "central-panel";

/// The Quiet and Dark presets' own accents: a fill nearer one of these than
/// the host's accent means the host's accent was not taken.
const QUIET_ACCENT: Rgb = Rgb::new(0x17, 0x5C, 0xC4);
const DARK_ACCENT: Rgb = Rgb::new(0x4C, 0x9A, 0xFF);

/// Mean channel value dividing a light window from a dark one.
const LIGHT_FLOOR: u16 = 128;

pub struct SystemThemeFollowsTheHost;

impl Check for SystemThemeFollowsTheHost {
    fn name(&self) -> &'static str {
        "system_theme_follows_the_host"
    }

    fn defect(&self) -> &'static str {
        "the Match the system theme ignores the host's light/dark mode or its accent colour \
         (OPERATOR_REQUESTS.md O268)"
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

/// One `REG_DWORD` under `HKCU\<key>`, through `reg.exe` so the harness needs
/// no registry binding of its own.
fn reg_dword(key: &str, value: &str) -> Option<u32> {
    let out = std::process::Command::new("reg")
        .args(["query", &format!(r"HKCU\{key}"), "/v", value])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let hex = text
        .lines()
        .find(|l| l.contains("REG_DWORD"))?
        .split_whitespace()
        .last()?;
    u32::from_str_radix(hex.trim_start_matches("0x"), 16).ok()
}

fn write_theme(exe: &Path) -> Result<()> {
    let dir = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    // The theme is an engine setting (`settings.txt`), not a shell preference.
    std::fs::create_dir_all(&dir)
        .and_then(|()| std::fs::write(dir.join("settings.txt"), "theme = system\n"))
        .map_err(|e| Error::new(format!("could not write settings: {e}")))
}

/// `rect` less a 4-point margin, clear of borders and rounded corners.
fn inset(rect: LRect) -> LRect {
    LRect::new(
        Pt::new(rect.min.x + 4.0, rect.min.y + 4.0),
        Pt::new(rect.max.x - 4.0, rect.max.y - 4.0),
    )
}

fn mean(c: Rgb) -> u16 {
    (u16::from(c.r) + u16::from(c.g) + u16::from(c.b)) / 3
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let accent = reg_dword(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\Accent",
        "AccentColorMenu",
    )
    .ok_or_else(|| Error::new("the host publishes no accent colour (AccentColorMenu)."))?;
    let [r, g, b, _] = accent.to_le_bytes();
    let host_accent = Rgb::new(r, g, b);
    let host_light = reg_dword(
        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
        "AppsUseLightTheme",
    )
    .is_none_or(|v| v != 0);
    let base = if host_light {
        QUIET_ACCENT
    } else {
        DARK_ACCENT
    };
    if driving::delta(host_accent, base) < 2 * driving::MIN_PRESSED_DELTA {
        return Err(Error::new(format!(
            "the host's accent {host_accent:?} is too close to the preset's own {base:?} to \
             tell which was taken."
        )));
    }
    report.note(format!(
        "host: accent {host_accent:?}, {} mode",
        if host_light { "light" } else { "dark" }
    ));

    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    write_theme(&exe)?;
    let pdf = driving::repo_fixture("cropped-sheets.pdf", "")?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("system_theme.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("system_theme.pointer.txt"))?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
    report.artifact(session.trace_path().to_path_buf());
    session.settle(30);

    let shot = ctx.out("system_theme.png");
    pointer.screenshot(&session, &shot)?;
    let image = crate::image::Image::load_png(&shot)?;
    report.artifact(shot);
    let frame = session.frame()?;
    let trace = session.trace()?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    let segment = driving::declared(&trace, ui_rect, READ_SEGMENT)
        .ok_or_else(|| Error::new(format!("no `{READ_SEGMENT}` was declared.")))?;
    let central = driving::declared(&trace, ui_rect, CENTRAL_PANEL)
        .ok_or_else(|| Error::new(format!("no `{CENTRAL_PANEL}` was declared.")))?;
    let fill = driving::fill_of(&image, &frame, inset(segment))
        .ok_or_else(|| Error::new("the Read segment sampled no pixels."))?;
    let ground = driving::fill_of(&image, &frame, inset(central))
        .ok_or_else(|| Error::new("the central panel sampled no pixels."))?;
    report.note(format!("Read segment {fill:?}, central panel {ground:?}"));

    let window_light = mean(ground) >= LIGHT_FLOOR;
    if window_light != host_light {
        return Ok(Some(format!(
            "★ the host is in {} mode and the window drew {} (central panel {ground:?}).",
            if host_light { "light" } else { "dark" },
            if window_light { "light" } else { "dark" }
        )));
    }
    if driving::delta(fill, host_accent) >= driving::delta(fill, base) {
        return Ok(Some(format!(
            "★ the selected mode segment is {fill:?}: nearer the preset's own accent {base:?} \
             than the host's {host_accent:?}."
        )));
    }
    Ok(None)
}
