//! `form_preview_reading_choices_reach_the_form` — every Reading choice of
//! the Form preview window is offered, and an answer reaches the merge, the
//! layout, the views or the drawing it names.
//!
//! Driven off the desktop on `fixtures/xfa-choices.pdf`, which opens Form
//! preview by itself. Oracles, each on a new `form-preview` line:
//!
//! - the first draw is `choices=default excluded=0`, and every choice
//!   declares its `form-preview.choice.<slug>.1` answer (and
//!   `picture-rounding.2`, the one three-way choice);
//! - *A border with no fill: Is filled white* redraws with
//!   `choices=missing-fill:1` and a new pixel digest: the unfilled field
//!   over the grey shade is painted white;
//! - *The form is read for: Printing as well* redraws with `excluded=1`, a
//!   new digest, and a `form-preview-left-out` line naming
//!   `form1.body.screen_only`, the fixture's `relevant="-print"` draw;
//! - *A section laid out in rows: Moves whole* redraws with
//!   `tiled-split:1` in its choices: the layout setting reached the worker.

use super::form_preview::{Launched, launch, open_choices};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const FIXTURE: &str = "xfa-choices.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/xfa-choices.PROVENANCE.py`.";
const DRAWN: &str = "form-preview";
const LEFT_OUT: &str = "form-preview-left-out";
const SCREEN_ONLY: &str = "form1.body.screen_only";

/// Every choice's slug, as `dialogs::xfa_preview::choices::CHOICES` lists them.
const SLUGS: [&str; 24] = [
    "view",
    "view-match",
    "scope-search",
    "bare-ref",
    "data-attributes",
    "bind-items-repeat",
    "bind-items-missing",
    "bind-items-unbound",
    "bind-items-search",
    "choice-instances",
    "tiled-split",
    "table-cell-height",
    "three-d-stroke",
    "check-caption",
    "check-edge",
    "edge-placement",
    "missing-fill",
    "stipple",
    "font-weight",
    "caption-font",
    "image-placement",
    "image-overflow",
    "picture-rounding",
    "medium",
];

/// See the module documentation.
pub struct FormPreviewReadingChoicesReachTheForm;

impl Check for FormPreviewReadingChoicesReachTheForm {
    fn name(&self) -> &'static str {
        "form_preview_reading_choices_reach_the_form"
    }

    fn defect(&self) -> &'static str {
        "a Form preview reading choice is missing, or choosing it does not change the merge, \
         layout, views or drawing it names"
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
    let l = launch(ctx, report, FIXTURE, METHOD, "form_preview_choices", false)?;
    let trace = l.session.trace()?;
    let Some(first) = trace.last(DRAWN) else {
        return Ok(Some(format!(
            "★★★ no `{DRAWN}` line: the form never drew. Trace: {}.",
            l.session.trace_path().display()
        )));
    };
    report.note(first.raw.clone());
    if let Some(failure) = open_choices(&l)? {
        return Ok(Some(failure));
    }
    let trace = l.session.trace()?;
    if first.get("choices") != Some("default") || first.get("excluded") != Some("0") {
        return Ok(Some(format!(
            "★★★ `{}` must carry choices=default excluded=0: the form's own views keep \
             everything.",
            first.raw
        )));
    }
    let missing: Vec<String> = SLUGS
        .iter()
        .map(|s| format!("form-preview.choice.{s}.1"))
        .chain(std::iter::once(
            "form-preview.choice.picture-rounding.2".to_owned(),
        ))
        .filter(|r| declared_in(&trace, l.ui_rect, r).is_none())
        .collect();
    if !missing.is_empty() {
        return Ok(Some(format!(
            "★★★ these answers were never declared, so the window does not offer them: {}.",
            missing.join(", ")
        )));
    }
    let mut digest = first.get("digest").unwrap_or_default().to_owned();
    let white = choose(&l, "missing-fill")?;
    if let Some(f) = expect(white.as_ref(), "missing-fill:1", Some(&digest), "0") {
        return Ok(Some(f));
    }
    digest = white
        .and_then(|w| w.get("digest").map(str::to_owned))
        .unwrap_or_default();
    let print = choose(&l, "view")?;
    if let Some(f) = expect(print.as_ref(), "view:1", Some(&digest), "1") {
        return Ok(Some(f));
    }
    let named = l
        .session
        .trace()?
        .events(LEFT_OUT)
        .any(|e| e.get("path") == Some(SCREEN_ONLY));
    if !named {
        return Ok(Some(format!(
            "★★★ printing left a part out but no `{LEFT_OUT} path={SCREEN_ONLY}` line named it: \
             the left-out list does not say what the view removed."
        )));
    }
    let tiled = choose(&l, "tiled-split")?;
    Ok(expect(tiled.as_ref(), "tiled-split:1", None, "1"))
}

/// Click answer 1 of the choice `slug` and return the redraw it caused.
fn choose(l: &Launched, slug: &str) -> Result<Option<TraceLine>> {
    let trace = l.session.trace()?;
    let region = format!("form-preview.choice.{slug}.1");
    let Some((rect, vp)) = declared_in(&trace, l.ui_rect, &region) else {
        return Ok(None);
    };
    let before = trace.events(DRAWN).count();
    l.pointer
        .click_in(&l.session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    l.session.settle(60);
    Ok(l.session.trace()?.events(DRAWN).nth(before).cloned())
}

/// The redraw carries `choice` among its choices, `excluded`, and a digest
/// other than `old` when one is given.
fn expect(
    line: Option<&TraceLine>,
    choice: &str,
    old: Option<&str>,
    excluded: &str,
) -> Option<String> {
    let Some(line) = line else {
        return Some(format!(
            "★★★ choosing `{choice}` drew nothing new: the answer never reached the worker."
        ));
    };
    let carried = line
        .get("choices")
        .is_some_and(|c| c.split(',').any(|c| c == choice));
    if !carried || line.get("excluded") != Some(excluded) {
        return Some(format!(
            "★★★ `{}` must carry {choice} in its choices and excluded={excluded}.",
            line.raw
        ));
    }
    if old.is_some_and(|d| line.get("digest") == Some(d)) {
        return Some(format!(
            "★★★ `{}` after choosing `{choice}` drew the same pixels: the answer never reached \
             the drawing.",
            line.raw
        ));
    }
    None
}
