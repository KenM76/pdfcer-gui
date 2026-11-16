//! `extract_keeps_the_ce_dimension_groups` — a page extracted through
//! Pages ▸ Extract… keeps its ce dimensions in their group, with the group's
//! scale and number format, and they open as ce dimensions in the new file.
//!
//! Drives `dimension-scaled.pdf` (one page, one linear ce dimension in a
//! group calibrated to 5 mm/pt) off the desktop with the scripted pointer:
//! the Dimension groups tab switches the group to feet and inches in
//! sixteenths, then Pages ▸ Extract… writes the page. The written file is read
//! by the engine's own reader, `pdfcer dimension-list` (the harness does not
//! link `pdfcer-core`), and must hold one ce dimension in a ft-in group
//! scaled 0.016404 ft/pt whose value shows the fraction the source showed. The
//! `extract` trace line must report `ce_dims_carried=1 ce_dims_left=0
//! ce_sidecar_newer=0`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/extract_ce_dimensions.md`.

use std::path::{Path, PathBuf};

use crate::checks::driving::{SHELL_DIAG_ENV, declared, repo_fixture};
use crate::checks::group_unit_converts::press;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,1000";
const INVOKE: &str = "mode.review,measure.manage_groups";
const FIXTURE: &str = "dimension-scaled.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/dimension-scaled.PROVENANCE.py`.";
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
/// The engine's command line, which reads the written file's sidecar.
const CLI_ENV: &str = "PDFCER_EXE"; // ui-text-exempt: an environment variable name
const CLI_DEFAULT: &str = "D:/Dev/pdfcer/target/release/pdfcer.exe";
const ROW: &str = "dimension-groups.row.0"; // ui-text-exempt: a trace region name, never displayed
const COMBO: &str = "dimension-groups.unit.combo"; // ui-text-exempt: a trace region name, never displayed
const FEET_INCHES: &str = "dimension-groups.unit.option.ft-in"; // ui-text-exempt: a trace region name, never displayed
const DENOMINATOR: &str = "precision.denominator"; // ui-text-exempt: a trace region name, never displayed
const SIXTEENTHS: &str = "precision.denominator.16"; // ui-text-exempt: a trace region name, never displayed
const TAB: &str = "ribbon.tab.pages"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.pages.extract"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.pages.organise.collapsed"; // ui-text-exempt: a trace region name, never displayed
const EXTRACT: &str = "extract-pages.extract"; // ui-text-exempt: a trace region name, never displayed
const SHOWN: &str = "dimension-member-shown"; // ui-text-exempt: a trace event name, never displayed
const WROTE: &str = "extract"; // ui-text-exempt: a trace event name, never displayed
/// 1000 mm over 200 pt, in feet per point.
const FT_PER_PT: f64 = 5.0 / 304.8;

/// See the module documentation.
pub struct ExtractKeepsTheCeDimensionGroups;

impl Check for ExtractKeepsTheCeDimensionGroups {
    fn name(&self) -> &'static str {
        "extract_keeps_the_ce_dimension_groups"
    }

    fn defect(&self) -> &'static str {
        "a page extracted to a new file loses its ce dimension groups: the copy's ce \
         dimensions fall back to the default group's scale and format, or open as plain \
         lines"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let target = ctx.out("extract-ce-dimensions-copy.pdf");
        let _ = std::fs::remove_file(&target);
        let driven = launch(ctx, &mut report, &target).and_then(|(session, pointer)| {
            let outcome = steps(ctx, &session, &pointer);
            let parked = pointer.gone(&session);
            match outcome? {
                Ok(shown) => parked.map(|_| Ok((shown, session))),
                Err(failure) => Ok(Err(failure)),
            }
        });
        let verdict = match driven {
            Ok(Ok((shown, session))) => judge(&mut report, &session, &target, &shown),
            Ok(Err(failure)) => Ok(Some(failure)),
            Err(why) => Err(why),
        };
        report.artifact(target);
        match verdict {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    target: &Path,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out("extract-ce-dimensions.pdf");
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("extract_ce_dimensions.trace.txt"));
    spec.pdf = Some(doc);
    let saved = target.display().to_string();
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
        (SAVE_PATH_ENV, saved.as_str()),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("extract_ce_dimensions.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// Set the group to sixteenths of an inch, then extract the page. `Ok(Ok)`
/// carries the member text the group showed; `Ok(Err)` is a failure sentence.
fn steps(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<std::result::Result<String, String>> {
    for region in [ROW, COMBO, FEET_INCHES, DENOMINATOR] {
        press(ctx, session, pointer, region)?;
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, SIXTEENTHS)?;
    let Some(shown) = session
        .trace()?
        .last_after(SHOWN, mark)
        .and_then(|l| l.get("text").map(str::to_owned))
    else {
        return Ok(Err(format!(
            "sixteenths was chosen and no `{SHOWN}` line followed, so the source never had a \
             non-default format to carry. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    press(ctx, session, pointer, TAB)?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, ITEM).is_none() && declared(&trace, ui_rect, COLLAPSED).is_some() {
        press(ctx, session, pointer, COLLAPSED)?;
    }
    press(ctx, session, pointer, ITEM)?;
    session.settle(20);
    if declared(&session.trace()?, ui_rect, EXTRACT).is_none() {
        return Ok(Err(format!(
            "Extract… opened no window with an Extract button. Trace: {}.",
            session.trace_path().display()
        )));
    }
    press(ctx, session, pointer, EXTRACT)?;
    session.settle(25);
    Ok(Ok(shown))
}

/// The written file first, so a build whose extraction drops the sidecar
/// fails on what the operator loses rather than on a missing trace field.
fn judge(
    report: &mut CheckReport,
    session: &Session,
    target: &Path,
    shown: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last(WROTE) else {
        return Ok(Some(format!(
            "Extract was pressed and no `{WROTE}` line followed. Failed: {}. Trace: {}.",
            trace
                .last("extract-failed")
                .map_or("none".to_owned(), |l| l.raw.clone()),
            session.trace_path().display()
        )));
    };
    report.note(format!("`{}`", line.raw));
    let listing = list_dimensions(target)?;
    report.note(format!("dimension-list: {}", listing.replace('\n', " / ")));
    if let Some(failure) = judge_listing(&listing, shown) {
        return Ok(Some(failure));
    }
    let expect = [
        ("ce_dims_carried", "1"),
        ("ce_dims_left", "0"),
        ("ce_sidecar_newer", "0"),
    ];
    Ok(expect
        .iter()
        .find(|(k, v)| line.get(k) != Some(v))
        .map(|(key, want)| {
            format!(
                "the copy is right and the receipt is not: `{key}` should be {want}; the \
                 extraction traced `{}`.",
                line.raw
            )
        }))
}

/// `pdfcer dimension-list <target>`'s standard output.
fn list_dimensions(target: &Path) -> Result<String> {
    let cli = std::env::var_os(CLI_ENV).map_or_else(|| PathBuf::from(CLI_DEFAULT), PathBuf::from);
    if !cli.is_file() {
        return Err(Error::new(format!(
            "the engine's command line is not at {}; set {CLI_ENV} to read the copy.",
            cli.display()
        )));
    }
    let out = std::process::Command::new(&cli)
        .arg("dimension-list")
        .arg(target)
        .output()
        .map_err(|e| Error::new(format!("running {}: {e}", cli.display())))?;
    if !out.status.success() {
        return Err(Error::new(format!(
            "dimension-list could not read {}: {}",
            target.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        ))
        .fatal());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The failure sentence for a listing that does not hold the carried group
/// and its one member, or `None`.
fn judge_listing(listing: &str, shown: &str) -> Option<String> {
    if !listing
        .lines()
        .next()
        .is_some_and(|h| h.contains("dimensions=1"))
    {
        return Some(format!(
            "the copy holds no ce dimension record, so its ce dimension opens as a plain line \
             and its group is gone: `{}`.",
            listing.lines().next().unwrap_or("")
        ));
    }
    let Some(group) = listing.lines().find(|l| {
        l.trim_start().starts_with("group ") && l.contains("unit=ft-in") && l.contains("members=1")
    }) else {
        return Some(format!(
            "no ft-in group with one member in the copy: the group's format was not carried. \
             Listing: {listing}"
        ));
    };
    let scale = group
        .split_whitespace()
        .find_map(|t| t.strip_prefix("scale="))
        .and_then(|s| s.parse::<f64>().ok());
    if !scale.is_some_and(|s| (s - FT_PER_PT).abs() < FT_PER_PT * 1e-6) {
        return Some(format!(
            "the copy's group should be scaled {FT_PER_PT} ft/pt; it lists `{}`.",
            group.trim()
        ));
    }
    let id = group.split_whitespace().nth(1).unwrap_or("");
    let Some(member) = listing.lines().find(|l| l.trim_start().starts_with("dim ")) else {
        return Some(format!("the listing names no ce dimension: {listing}"));
    };
    if !member.contains(&format!("group={id} ")) {
        return Some(format!(
            "the copy's ce dimension is not in its group {id}: `{}`.",
            member.trim()
        ));
    }
    let fraction = fraction_of(shown);
    if fraction.is_none_or(|f| !member.contains(&format!(" {f}"))) {
        return Some(format!(
            "the source showed `{shown}` and the copy's ce dimension lists `{}`: the group's \
             fraction setting was not carried.",
            member.trim()
        ));
    }
    None
}

/// The `n/d` token of a feet-and-inches reading, `3'-3 6/16"` → `6/16`.
fn fraction_of(text: &str) -> Option<&str> {
    text.split(|c: char| !(c.is_ascii_digit() || c == '/'))
        .find(|t| t.contains('/') && !t.starts_with('/') && !t.ends_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARRIED: &str = "dimension-list x.pdf groups=1 dimensions=1\n  \
        group 0 \"Default\" unit=ft-in scale=0.016404199475065617 ft/pt visible=true members=1\n  \
        dim 0 group=0 kind=linear value=\"3'-3 6/16\"\" offset=30 text_along=0 overrides=0\n";

    #[test]
    fn a_carried_group_passes() {
        assert_eq!(judge_listing(CARRIED, "3'-3 6/16\""), None);
    }

    #[test]
    fn a_copy_without_the_sidecar_fails() {
        let bare = "dimension-list x.pdf groups=0 dimensions=0\n";
        assert!(judge_listing(bare, "3'-3 6/16\"").is_some());
    }

    #[test]
    fn a_default_group_or_a_lost_fraction_fails() {
        let mm = CARRIED.replace("unit=ft-in", "unit=mm");
        assert!(judge_listing(&mm, "3'-3 6/16\"").is_some());
        assert!(judge_listing(CARRIED, "3'-3 3/8\"").is_some());
        let unscaled = CARRIED.replace("scale=0.016404199475065617", "scale=1");
        assert!(judge_listing(&unscaled, "3'-3 6/16\"").is_some());
    }

    #[test]
    fn the_fraction_is_read_from_the_reading() {
        assert_eq!(fraction_of("3'-3 6/16\""), Some("6/16"));
        assert_eq!(fraction_of("3'-3\""), None);
    }
}
