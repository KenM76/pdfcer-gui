//! `tabs_read_as_tabs` — the ribbon's and the dock's tabs are drawn as tabs,
//! not as buttons. See `docs/modules/ui-verify/checks/tabs_look.md`.

use crate::checks::driving::{self, MIN_PRESSED_DELTA, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::image::Rgb;
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

const OFFSCREEN: &str = "-4200,-4200,1400,900";

/// Height of the accent rule along a selected tab's top edge, in points.
const RULE_PTS: f32 = 2.0;
/// Inset from each side of the rule sample, past the tab's rounded corners.
const RULE_INSET_PTS: f32 = 8.0;

pub struct TabsReadAsTabs;

impl Check for TabsReadAsTabs {
    fn name(&self) -> &'static str {
        "tabs_read_as_tabs"
    }

    fn defect(&self) -> &'static str {
        "the selected ribbon tab and the selected panel tab are solid accent-filled plates, so \
         a row of tabs reads as a row of buttons (OPERATOR_REQUESTS.md O268)"
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

/// One tab's two samples: the strip along its top edge, and its body.
struct Sampled {
    name: String,
    rule: Rgb,
    body: Rgb,
}

impl Sampled {
    /// Selected, by this check's definition: its top edge is a different
    /// colour from its body. A solid plate has no such edge.
    fn ruled(&self) -> bool {
        driving::delta(self.rule, self.body) >= MIN_PRESSED_DELTA
    }
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
    let pdf = driving::repo_fixture("cropped-sheets.pdf", "")?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("tabs_look.trace.txt"));
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
    // Attached so the run never touches the operator's own pointer; this
    // check moves nothing, and a pointer that never arrives hovers nothing.
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("tabs_look.pointer.txt"))?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
    report.artifact(session.trace_path().to_path_buf());
    session.settle(30);

    let shot = ctx.out("tabs_look.png");
    // The window's own frame: an OS capture of an off-desktop window is
    // whatever is on screen at those coordinates.
    pointer.screenshot(&session, &shot)?;
    let image = crate::image::Image::load_png(&shot)?;
    report.artifact(shot);
    let frame = session.frame()?;
    let trace = session.trace()?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    for (family, prefix) in [("ribbon", "ribbon.tab."), ("dock", "dock.tab.")] {
        let names: Vec<String> = driving::declared_names(&trace, ui_rect, prefix)
            .into_iter()
            // `ribbon.tab.<id>` has no further dot; a dock panel id does.
            .filter(|n| family == "dock" || !n[prefix.len()..].contains('.'))
            .collect();
        if names.len() < 2 {
            return Err(Error::new(format!(
                "the {family} declared {} tab(s) ({}); a row of tabs needs two to compare.",
                names.len(),
                driving::list(&names)
            )));
        }
        let mut sampled = Vec::new();
        for name in &names {
            let Some(rect) = driving::declared(&trace, ui_rect, name) else {
                continue;
            };
            if rect.width() <= 2.0 * RULE_INSET_PTS || rect.height() < 8.0 {
                continue;
            }
            let rule = LRect::new(
                Pt {
                    x: rect.min.x + RULE_INSET_PTS,
                    y: rect.min.y,
                },
                Pt {
                    x: rect.max.x - RULE_INSET_PTS,
                    y: rect.min.y + RULE_PTS,
                },
            );
            let body = LRect::new(
                Pt {
                    x: rect.min.x + RULE_INSET_PTS,
                    y: rect.min.y + rect.height() * 0.35,
                },
                Pt {
                    x: rect.max.x - RULE_INSET_PTS,
                    y: rect.max.y - rect.height() * 0.2,
                },
            );
            let (Some(rule), Some(body)) = (
                driving::fill_of(&image, &frame, rule),
                driving::fill_of(&image, &frame, body),
            ) else {
                continue;
            };
            sampled.push(Sampled {
                name: name.clone(),
                rule,
                body,
            });
        }
        let ruled: Vec<&Sampled> = sampled.iter().filter(|s| s.ruled()).collect();
        report.note(format!(
            "{family}: {} tabs sampled, ruled: {}",
            sampled.len(),
            driving::list(&ruled.iter().map(|s| s.name.clone()).collect::<Vec<_>>())
        ));
        for s in &sampled {
            report.note(format!("  {} rule {:?} body {:?}", s.name, s.rule, s.body));
        }
        if ruled.is_empty() {
            return Ok(Some(format!(
                "★ no {family} tab has a top edge distinct from its body. A selected tab drawn \
                 as one solid plate is a button, not a tab."
            )));
        }
        if ruled.len() == sampled.len() {
            return Ok(Some(format!(
                "every {family} tab carries a distinct top edge, so the edge marks nothing."
            )));
        }
        if family == "ribbon" && ruled.len() != 1 {
            return Ok(Some(format!(
                "{} ribbon tabs are marked selected; exactly one is active.",
                ruled.len()
            )));
        }
    }
    Ok(None)
}
