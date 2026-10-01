//! `export_dxf_writes_the_version_and_scale_chosen` — the DXF version and the
//! drawing scale an operator picks in the export window must be the ones in
//! the file it writes.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/export_dxf_options.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

/// Off the desktop, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1400,900";

/// Pinned: page 0 carries a stroked rectangle, so the file has extents to
/// measure the scale by.
const FIXTURE: &str = "fixtures/cropped-sheets.pdf";

/// `PDFCER_DIAG_SAVE_PATH` — the seam that answers the save picker.
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";

/// The File tab, and the command that opens the window.
const FILE_TAB: &str = "file";
const COMMAND: &str = "ribbon.item.file.export_dxf";

/// The window's controls.
const VERSION_COMBO: &str = "export-dxf.version";
const VERSION_ITEM_PREFIX: &str = "export-dxf.version.item.";
const SCALE_FIELD: &str = "export-dxf.scale";
const EXPORT: &str = "export-dxf.export";

/// `export-dxf-open … version=…` and `export-dxf-requested … scale=… version=…`.
const OPENED_EVENT: &str = "export-dxf-open";
const REQUESTED_EVENT: &str = "export-dxf-requested";
/// The apply arm's success line.
const WROTE_EVENT: &str = "export-dxf";

/// One export: which drop-down entry to pick, the `$ACADVER` it must write,
/// and the real-world number to type into the scale ratio.
struct Round {
    item: usize,
    acadver: &'static str,
    real: &'static str,
}

/// R12 then R2004, in the drop-down's oldest-first order. Two different
/// numbers typed, so a field that ignores typing writes the same scale twice.
const ROUNDS: [Round; 2] = [
    Round {
        item: 0,
        acadver: "AC1009",
        real: "37",
    },
    Round {
        item: 2,
        acadver: "AC1018",
        real: "53",
    },
];

/// Relative tolerance on the extents ratio: the header prints six decimals.
const RATIO_TOLERANCE: f64 = 1e-4;

/// What one export left behind.
struct Written {
    acadver: String,
    extmax_x: f64,
    scale: f64,
    units: String,
}

/// See the module documentation.
pub struct ExportDxfWritesTheVersionAndScaleChosen;

impl Check for ExportDxfWritesTheVersionAndScaleChosen {
    fn name(&self) -> &'static str {
        "export_dxf_writes_the_version_and_scale_chosen"
    }

    fn defect(&self) -> &'static str {
        "the DXF export window offers a version and a drawing scale and the file it writes \
         ignores them — an R12 file asked for and a 2004 file delivered, or a 1:50 drawing \
         exported at paper scale — which a CAD operator discovers only when the geometry \
         arrives fifty times too small or the old reader refuses the file"
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

/// The value on the line after `name`'s group code, in a DXF header.
fn header_value<'a>(dxf: &'a str, name: &str) -> Option<&'a str> {
    let mut lines = dxf.lines().map(str::trim);
    lines.find(|l| *l == name)?;
    lines.next()?;
    lines.next()
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
    let pdf = ctx.source_root.clone().unwrap_or_default().join(FIXTURE);
    let pdf = if pdf.exists() {
        pdf
    } else {
        std::path::PathBuf::from(FIXTURE)
    };
    if !pdf.exists() {
        return Err(Error::new(format!("the fixture {FIXTURE} is not on disk.")));
    }

    let target = ctx.out("export_dxf_options.dxf");
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "cannot clear {} before the run, so an earlier run's file could be read as this \
             one's.",
            target.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("export_dxf_options.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), target.display().to_string()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("export_dxf_options.pointer.txt"))?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} on {} as pid {}",
        exe.display(),
        pdf.display(),
        session.pid()
    ));
    session.settle(30);
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    let mut written: Vec<Written> = Vec::new();
    for (n, round) in ROUNDS.iter().enumerate() {
        let opened_before = session.trace()?.events(OPENED_EVENT).count();
        crate::checks::ocr::click_tab(&session, &pointer, ui_rect, FILE_TAB)?;
        let Some(item) = driving::declared_or_in_overflow(&session, &pointer, ui_rect, COMMAND)?
        else {
            return Err(Error::new(format!(
                "the File tab declares no `{COMMAND}`, on the band or in the overflow."
            )));
        };
        crate::input::Click::click_rect(&pointer, &session, item)?;
        session.settle(20);
        let trace = session.trace()?;
        let opens: Vec<_> = trace.events(OPENED_EVENT).collect();
        if opens.len() <= opened_before {
            return Ok(Some(format!(
                "round {n}: `{COMMAND}` was clicked and the window traced no `{OPENED_EVENT}`."
            )));
        }
        let open_version = opens.last().and_then(|l| l.get("version")).unwrap_or("?");
        report.note(format!(
            "round {n}: the window opened on version {open_version}"
        ));
        // The window remembers the last export's choices, so the second
        // opening must already show the first round's version.
        if let Some(previous) = written.last()
            && open_version != previous.acadver
        {
            return Ok(Some(format!(
                "the first export wrote {} and the window reopened on {open_version}. The \
                 version is not remembered between exports.",
                previous.acadver
            )));
        }

        // -- the version ------------------------------------------------------
        let entry = format!("{VERSION_ITEM_PREFIX}{}", round.item);
        let mut listed = false;
        for _ in 0..3 {
            click_region(&session, &pointer, ui_rect, VERSION_COMBO)?;
            session.settle(10);
            if driving::declared(&session.trace()?, ui_rect, &entry).is_some() {
                listed = true;
                break;
            }
        }
        if !listed {
            return Err(Error::new(format!(
                "the version list did not open, or opened without `{entry}`. Entries declared: \
                 {}.",
                driving::list(&driving::declared_names(
                    &session.trace()?,
                    ui_rect,
                    VERSION_ITEM_PREFIX
                ))
            )));
        }
        click_region(&session, &pointer, ui_rect, &entry)?;
        session.settle(10);

        // -- the scale: replace the real-world number --------------------------
        let viewport = click_region(&session, &pointer, ui_rect, SCALE_FIELD)?;
        session.settle(6);
        pointer.key(&session, viewport.as_deref(), "A", Some("ctrl"))?;
        pointer.type_text(&session, viewport.as_deref(), round.real)?;
        pointer.key(&session, viewport.as_deref(), "Enter", None)?;
        session.settle(10);

        // -- export --------------------------------------------------------------
        let wrote_before = session.trace()?.events(WROTE_EVENT).count();
        click_region(&session, &pointer, ui_rect, EXPORT)?;
        session.settle(30);
        let trace = session.trace()?;
        if trace.events(WROTE_EVENT).count() <= wrote_before {
            return Ok(Some(format!(
                "round {n}: Export was pressed and no `{WROTE_EVENT}` line followed. failed={}",
                trace
                    .last("export-dxf-failed")
                    .map_or("none".to_owned(), |l| l.raw.clone())
            )));
        }
        let requested = trace.last(REQUESTED_EVENT).ok_or_else(|| {
            Error::new(format!(
                "round {n}: the export ran and traced no `{REQUESTED_EVENT}`."
            ))
        })?;
        report.note(format!("round {n}: `{}`", requested.raw));
        let scale = requested
            .get("scale")
            .and_then(|v| v.parse::<f64>().ok())
            .ok_or_else(|| Error::new(format!("no readable scale in `{}`", requested.raw)))?;
        let units = requested.get("units").unwrap_or("?").to_owned();

        let dxf = std::fs::read_to_string(&target).map_err(|e| {
            Error::new(format!(
                "round {n}: the shell traced a write and {} cannot be read: {e}",
                target.display()
            ))
        })?;
        let acadver = header_value(&dxf, "$ACADVER").unwrap_or("none").to_owned();
        let extmax_x = header_value(&dxf, "$EXTMAX")
            .and_then(|v| v.parse::<f64>().ok())
            .ok_or_else(|| {
                Error::new(format!(
                    "round {n}: the file carries no readable `$EXTMAX`, so the scale cannot \
                     be measured."
                ))
            })?;
        report.note(format!(
            "round {n}: file says {acadver}, $EXTMAX x = {extmax_x}"
        ));

        if acadver != round.acadver {
            return Ok(Some(format!(
                "★ round {n}: drop-down entry {} was picked, which is {}, and the file says \
                 {acadver}. The trace said version={}.",
                round.item,
                round.acadver,
                requested.get("version").unwrap_or("?")
            )));
        }
        if requested.get("version") != Some(round.acadver) {
            return Ok(Some(format!(
                "round {n}: the file is {acadver} and the shell traced version={}. The \
                 disclosure and the file disagree.",
                requested.get("version").unwrap_or("?")
            )));
        }
        written.push(Written {
            acadver,
            extmax_x,
            scale,
            units,
        });
    }

    // -- the scale reached the file ---------------------------------------------
    let (a, b) = (&written[0], &written[1]);
    if a.units != b.units {
        return Err(Error::new(format!(
            "the units changed between exports ({} then {}), so the extents cannot be compared \
             on scale alone.",
            a.units, b.units
        )));
    }
    let typed = ROUNDS[1].real.parse::<f64>().unwrap_or(f64::NAN)
        / ROUNDS[0].real.parse::<f64>().unwrap_or(f64::NAN);
    let traced = b.scale / a.scale;
    if (traced - typed).abs() > RATIO_TOLERANCE * typed {
        return Ok(Some(format!(
            "★ {} then {} was typed into the scale ratio, and the exports carried scale {} then \
             {} — a ratio of {traced:.4}, not {typed:.4}. The typed number did not become the \
             export's scale.",
            ROUNDS[0].real, ROUNDS[1].real, a.scale, b.scale
        )));
    }
    let measured = b.extmax_x / a.extmax_x;
    if (measured - traced).abs() > RATIO_TOLERANCE * traced {
        return Ok(Some(format!(
            "★ the shell exported at scale {} then {} (×{traced:.4}) and the file's extents \
             went from {} to {} (×{measured:.4}). The scale was reported and not applied.",
            a.scale, b.scale, a.extmax_x, b.extmax_x
        )));
    }
    report.note(format!(
        "typed ×{typed:.4}, traced ×{traced:.4}, extents ×{measured:.4}"
    ));
    Ok(None)
}

/// Click a region the application declared, in whichever viewport it was
/// declared in. Returns that viewport, for keys sent afterwards.
fn click_region(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    name: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let (rect, viewport) = driving::declared_in(&trace, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{name}` region. Regions beginning `export-dxf.`: {}.",
            driving::list(&driving::declared_names(&trace, ui_rect, "export-dxf."))
        ))
    })?;
    if !rect.is_substantial() {
        return Err(Error::new(format!(
            "`{name}` was declared at {rect:?}, which has no usable area to click."
        )));
    }
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    Ok(viewport)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_header_value_is_two_lines_after_its_name() {
        let dxf = "  0\nSECTION\n  9\n$ACADVER\n  1\nAC1018\n  9\n$EXTMAX\n 10\n12.5\n 20\n3.0\n";
        assert_eq!(header_value(dxf, "$ACADVER"), Some("AC1018"));
        assert_eq!(header_value(dxf, "$EXTMAX"), Some("12.5"));
        assert_eq!(header_value(dxf, "$INSUNITS"), None);
    }
}
