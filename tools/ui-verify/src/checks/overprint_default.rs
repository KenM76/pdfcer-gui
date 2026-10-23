//! `overprint_default` — **Settings ▸ Display ▸ Use overprint preview decides
//! whether a document opens with the preview on, and the Settings window
//! writes the choice.** Each choice is planted and fixtures opened: one that
//! asks for overprint, the same file claiming PDF/X by output intent and by
//! XMP alone, the same file listing an overprinting state nothing invokes,
//! and one that asks for neither. *Automatic*'s answer arrives from a
//! thread, so the line read is the first with a known `asks`. Driven off the
//! desktop with the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/overprint_default.md`.

use super::dimdrive::{Fixture, launch_with, press, ui_rect_event};
use crate::checks::driving::{declared, declared_in, declared_names, list};
use crate::checks::scripteddrive::userdata;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::report::CheckReport;

const LETTER: PageGeometry = PageGeometry {
    width_pt: 612.0,
    height_pt: 792.0,
};
/// Asks for overprint (`/OP true`), claims no standard.
const ASKS: Fixture = Fixture {
    file: "print-shop.pdf",
    method: "Rebuild it with `python fixtures/print-shop.PROVENANCE.py`.",
    page: LETTER,
};
/// The same page, with a `GTS_PDFX` output intent.
const PDFX: Fixture = Fixture {
    file: "print-shop-pdfx.pdf",
    method: "Rebuild it with `python fixtures/print-shop-pdfx.PROVENANCE.py`.",
    page: LETTER,
};
/// The same page claiming PDF/X-4 in XMP only: no intent, no Info key.
const PDFX4: Fixture = Fixture {
    file: "print-shop-pdfx4.pdf",
    method: "Rebuild it with `python fixtures/print-shop-pdfx4.PROVENANCE.py`.",
    page: LETTER,
};
/// The same page listing `/GS0` (`/OP true`) without any `gs` invoking it.
const LISTED: Fixture = Fixture {
    file: "print-shop-listed.pdf",
    method: "Rebuild it with `python fixtures/print-shop-listed.PROVENANCE.py`.",
    page: LETTER,
};
/// Asks for no overprint and claims no standard.
const PLAIN: Fixture = Fixture {
    file: "checker-image.pdf",
    method: "Rebuild it with `python fixtures/checker-image.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 400.0,
    },
};
const KEY: &str = "overprint_preview_default"; // ui-text-exempt: a preferences-file key
const PREFS_FILE: &str = "preferences.txt";
const DECIDED: &str = "overprint-default"; // ui-text-exempt: a trace event name
/// Planted choice (`None` plants nothing), fixture, the choice the trace must
/// name, and whether the preview must open on. Every choice meets a fixture
/// where a wrong reading gives the other answer, so no constant passes.
const CASES: [(Option<&str>, &Fixture, &str, &str); 9] = [
    (None, &PLAIN, "always", "1"),
    (Some("never"), &ASKS, "never", "0"),
    (Some("never"), &PDFX, "never", "0"),
    (Some("pdfx"), &ASKS, "pdfx", "0"),
    (Some("pdfx"), &PDFX, "pdfx", "1"),
    (Some("pdfx"), &PDFX4, "pdfx", "1"),
    (Some("automatic"), &PLAIN, "automatic", "0"),
    (Some("automatic"), &ASKS, "automatic", "1"),
    (Some("automatic"), &LISTED, "automatic", "0"),
];
/// Settles allowed for *Automatic*'s walk to answer.
const WAITS: usize = 12;
/// The `asks` value of a line written before the walk answered.
const PENDING: &str = "?";
const OPEN_SETTINGS: &str = "file.settings";
const DIALOG: &str = "dialog:settings";
const DISPLAY_HEADING: &str = "settings.heading.display";
const PAGE_BODY: &str = "settings.page"; // ui-text-exempt: a trace region name
const NEVER: &str = "settings.display.overprint.never";
const MAX_SCROLLS: usize = 10;
const SAVE: &str = "dialog:settings.save";
const PREFS_SAVED: &str = "prefs-saved"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct OverprintDefaultDecidesTheOpen;

impl Check for OverprintDefaultDecidesTheOpen {
    fn name(&self) -> &'static str {
        "overprint_default"
    }

    fn defect(&self) -> &'static str {
        "a document opens with overprint preview on or off regardless of Settings ▸ Display ▸ \
         Use overprint preview, PDF/X or a page's request for overprint is not read, or the \
         Settings window does not write the choice"
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

fn plant(ctx: &CheckContext, choice: Option<&str>) -> Result<()> {
    let body = choice.map_or_else(String::new, |c| format!("{KEY} = {c}\n"));
    crate::sandbox::write_prefs(&userdata(ctx)?, &body)
        .map_err(|e| Error::new(format!("could not plant the preference: {e}")))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    for (i, (planted, fixture, choice, on)) in CASES.iter().enumerate() {
        plant(ctx, *planted)?;
        let stem = format!("overprint_default.{i}");
        let (session, pointer) = launch_with(ctx, report, fixture, "mode.read", &stem, &[])?;
        let mut line = None;
        for _ in 0..WAITS {
            line = session.trace()?.last(DECIDED).cloned();
            if line
                .as_ref()
                .is_some_and(|l| l.get("asks") != Some(PENDING))
            {
                break;
            }
            session.settle(10);
        }
        let _ = pointer.gone(&session);
        drop(session);
        let Some(line) = line else {
            return Ok(Some(format!(
                "{} opened under {planted:?} and no `{DECIDED}` line followed: the default \
                 was never applied.",
                fixture.file
            )));
        };
        report.note(format!("{planted:?} on {}: {}", fixture.file, line.raw));
        if line.get("asks") == Some(PENDING) {
            return Ok(Some(format!(
                "{} opened under {planted:?} and Automatic never answered: the last line \
                 is `{}`.",
                fixture.file, line.raw
            )));
        }
        if line.get("choice") != Some(*choice) || line.get("on") != Some(*on) {
            return Ok(Some(format!(
                "{} opened under {planted:?}: owed choice={choice} on={on}, the trace says `{}`.",
                fixture.file, line.raw
            )));
        }
    }
    plant(ctx, Some("automatic"))?;
    if let Some(failure) = choose_never(ctx, report)? {
        return Ok(Some(failure));
    }
    let written = written(&userdata(ctx)?)?;
    report.note(format!(
        "after choosing Never and Save the file reads {KEY} = {written:?}"
    ));
    if written.as_deref() != Some("never") {
        return Ok(Some(format!(
            "Never was chosen and saved, and the file says {KEY} = {written:?}."
        )));
    }
    Ok(None)
}

/// The choice line of the sandbox's preferences file, if any.
fn written(userdata: &std::path::Path) -> Result<Option<String>> {
    let path = userdata.join(PREFS_FILE);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
    Ok(text
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == KEY).then(|| v.trim().to_owned())
        }))
}

/// Open Settings ▸ Display, roll to Never, press it, Save. The window is
/// closed before this returns, so the file is final.
fn choose_never(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ui_rect_event(ctx)?;
    let (session, pointer) = launch_with(
        ctx,
        report,
        &PLAIN,
        OPEN_SETTINGS,
        "overprint_default.settings",
        &[],
    )?;
    let trace = session.trace()?;
    if declared_in(&trace, ui_rect, DIALOG).is_none() {
        return Err(Error::new(format!(
            "`{OPEN_SETTINGS}` was invoked and no `{DIALOG}` appeared."
        )));
    }
    if declared_in(&trace, ui_rect, DISPLAY_HEADING).is_none() {
        return Ok(Some(format!(
            "the Settings window declares no `{DISPLAY_HEADING}`. Headings: {}.",
            list(&declared_names(&trace, ui_rect, "settings.heading."))
        )));
    }
    press(ctx, &session, &pointer, DISPLAY_HEADING)?;
    for _ in 0..MAX_SCROLLS {
        let trace = session.trace()?;
        if declared_in(&trace, ui_rect, NEVER).is_some() {
            break;
        }
        let Some((body, vp)) = declared_in(&trace, ui_rect, PAGE_BODY) else {
            break;
        };
        pointer.wheel_in(&session, vp.as_deref(), WindowPoint::centre_of(body), -3.0)?;
        session.settle(10);
    }
    let trace = session.trace()?;
    if declared_in(&trace, ui_rect, NEVER).is_none() {
        return Ok(Some(format!(
            "the Display page declares no `{NEVER}`: the choice is not offered, or is out of \
             reach. Regions: {}.",
            list(&declared_names(&trace, ui_rect, "settings.display."))
        )));
    }
    press(ctx, &session, &pointer, NEVER)?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, SAVE).is_none() {
        return Ok(Some(format!("the Settings window declares no `{SAVE}`.")));
    }
    let saves = trace.events(PREFS_SAVED).count();
    press(ctx, &session, &pointer, SAVE)?;
    if session.trace()?.events(PREFS_SAVED).count() == saves {
        return Ok(Some(format!(
            "Save was pressed after choosing Never and no `{PREFS_SAVED}` line followed."
        )));
    }
    let _ = pointer.gone(&session);
    drop(session);
    Ok(None)
}
