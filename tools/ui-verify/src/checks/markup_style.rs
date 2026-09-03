//! `markup_style_group_is_drawn` — the ribbon group that was a caption over
//! nothing.
//!
//! # The defect
//!
//! `RIBBON_IA.md` §5.5 specifies a **Style** group on the Markup tab —
//! *"Colour · Line width · Fill · Opacity"* — and the manifest has declared it
//! since S2, as one `Item::custom("colour_swatch")`.
//!
//! **No renderer ever matched that kind.** `egui-shell`'s custom-item extension
//! point works by the application supplying a closure that looks at
//! `item.kind`; the application's closure looked only for the Recent menu and
//! returned `None` for everything else. So the shell reserved the item's space,
//! the application declined to draw it, and the Style group rendered as a
//! **caption over an empty band** for the whole of v0.1.0.
//!
//! Meanwhile the pen was two hard-coded constants — red, 2 pt — so every mark
//! this shell has ever authored is the same colour and the same width, with no
//! way to change either. §5.5 predicted the operator's report in advance:
//!
//! > The `Style` group sets defaults for the next markup. … Both must exist;
//! > today only the first does, **which is why a placed markup feels final**.
//!
//! # ★ Why no test could see it, and this one can
//!
//! This is a **third** shape of the invisible-wiring failure this harness
//! exists for, and it is worth naming beside the other two:
//!
//! | shape | example | what was green |
//! |---|---|---|
//! | a command with no dispatch arm | `file.settings` | the registry, the manifest, the reachability check |
//! | a linked crate with a refusing adapter | `pdfcer-print` | the adapter's own tests, which asserted the refusal |
//! | **a declared item with no renderer** | `colour_swatch` | everything — the manifest test asserts the item is *declared* |
//!
//! The third is the quietest. `shell::manifest::mod`'s own test asserts
//! `assert_eq!(style, vec![Item::custom(COLOUR_SWATCH)])` and passes, correctly,
//! for a build in which the item draws nothing: it is a claim about the
//! manifest, and the manifest was right. The reachability check cannot help
//! either — a `Custom` item carries no command id, which is the whole point of
//! it, so it is invisible to every check built on `command_references()`.
//!
//! What is left is asking the running program whether it drew anything, which
//! is what `diag::ui_rect` is for.
//!
//! # What this measures
//!
//! Three regions, published by `canvas::markup::swatch`:
//!
//! ```text
//! markup.style.ink           the pen swatch
//! markup.style.highlighter   the highlighter swatch
//! markup.style.width         the width control
//! ```
//!
//! All three must be **declared and substantial**. Substantial matters as much
//! as declared: a control laid out with no usable area is the redaction panel's
//! apply button shipped below the bottom of its own pane, which this project
//! has already had once.
//!
//! Then it **changes the width** and reads the `markup-pen` trace line back, so
//! the pass is not merely "three rectangles exist" but "a control was driven
//! and the pen moved".
//!
//! # What it does NOT do, and what that costs
//!
//! **It does not open the colour picker.** `egui`'s
//! `color_edit_button_srgba` opens a popup whose internals publish no regions,
//! so a harness cannot aim at a hue in it — and clicking blind inside a popup
//! is how a check starts passing for the wrong reason.
//!
//! So the colour half is verified one step short: the swatches are on screen
//! and substantial, and the *pen* is proved mutable through the width control,
//! which shares the same trace line and the same state. What is not
//! machine-verified is that dragging in the picker lands on the annotation —
//! and the unit test `pen::tests::a_colour_round_trips_through_the_swatch`
//! covers the conversion either side of it. That gap is named here rather than
//! papered over: the honest description of this check is *"the Style group is
//! drawn and its state is live"*, not *"colour picking works"*.

use crate::checks::driving::{
    ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, list, shell_trace,
};
use crate::checks::{Check, CheckContext, CheckReport, driving};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The tab the group lives on, and the region that activates it.
const TAB_ID: &str = "markup";
const TAB: &str = "ribbon.tab.markup";

/// The three regions `canvas::markup::swatch` publishes, spelled as literals
/// because that is the contract between the two crates.
const INK: &str = "markup.style.ink";
const HIGHLIGHTER: &str = "markup.style.highlighter";
const WIDTH: &str = "markup.style.width";

/// The trace line one change to the pen emits.
const PEN_EVENT: &str = "markup-pen";

pub struct MarkupStyleGroupIsDrawn;

impl Check for MarkupStyleGroupIsDrawn {
    fn name(&self) -> &'static str {
        "markup_style_group_is_drawn"
    }

    fn defect(&self) -> &'static str {
        "Markup ▸ Style declares a colour_swatch item that no renderer draws, so the group is a \
         caption over an empty band and the pen is two hard-coded constants"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. The Markup tab is not in Read's tab list, so this check has to switch to \
             a mode that carries it — and the mode selector is only meaningful with a document.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is three clicks and a drag. Reported \
             as SKIPPED rather than passed — a check that did not run has learned nothing.",
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("markup_style.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    // Maximised for the same reason `settings_theme` is: the Style group is not
    // the first on its tab, and a group past the fold moves into the ribbon's
    // overflow where its items publish no rects at all.
    session.maximize();
    session.settle(30);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process.",
            ctx.profile.vocab.start_event
        )));
    }

    let driver = Driver::new(session.window());
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    // --- A. Review mode, then the Markup tab -------------------------------
    //
    // pdfcer opens in **Read**, whose tab list is `["file", "view"]`, so Markup
    // does not exist in the mode this process starts in. Review rather than
    // Edit because Review is the stance in which markup is placed — and
    // `manifest::format`'s header makes the point that a reviewer who cannot
    // restyle a cloud they just drew has been given half a tool, which is this
    // check's subject from the other end.
    driving::click_mode_segment(&session, &driver, ui_rect, "review")?;

    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region after switching to Review. Tabs \
             declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(12);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(TAB_ID))
    {
        return Err(Error::new(format!(
            "the click on `{TAB}` produced no `{TAB_EVENT} tab={TAB_ID}` line."
        )));
    }

    // --- B. all three controls, declared and substantial --------------------
    let trace = session.trace()?;
    let mut missing = Vec::new();
    let mut flat = Vec::new();
    let mut rects = Vec::new();
    for name in [INK, HIGHLIGHTER, WIDTH] {
        match declared(&trace, ui_rect, name) {
            None => missing.push(name),
            Some(rect) if !rect.is_substantial() => flat.push(format!("{name} at {rect:?}")),
            Some(rect) => rects.push((name, rect)),
        }
    }
    if !missing.is_empty() {
        return Ok(Some(format!(
            "the Markup tab is active and its controls publish their rects, but {} of the Style \
             group's three is absent: {}. That is the defect — the manifest declares the item and \
             no renderer draws it, so the group is a caption over an empty band. Controls \
             declared on this tab: {}.",
            missing.len(),
            missing.join(", "),
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        )));
    }
    if !flat.is_empty() {
        return Ok(Some(format!(
            "the Style group's controls are declared and have no usable area: {}. They are laid \
             out and not on screen.",
            flat.join(", ")
        )));
    }
    report.note(format!(
        "all three Style controls are drawn: {}",
        rects
            .iter()
            .map(|(n, _)| (*n).to_owned())
            .collect::<Vec<_>>()
            .join(", ")
    ));

    // --- C. drive one of them, and read the pen back ------------------------
    //
    // ★ The width, not a swatch. `color_edit_button_srgba` opens a popup whose
    // internals publish no regions, so a harness cannot aim at a hue inside it
    // and clicking blind there is how a check passes for the wrong reason. The
    // width control shares the pen and the trace line, so driving it proves the
    // same property: the group's state is LIVE, not merely drawn.
    let before = shell_pen_lines(&session)?;
    let width_rect = rects
        .iter()
        .find(|(n, _)| *n == WIDTH)
        .map(|(_, r)| *r)
        .expect("the width rect was collected above");
    // ★ Both endpoints come from the control's OWN declared rect, and no
    // screen coordinate is written here.
    //
    // Rule 2 of this crate's check-writing rules: *only ever write a
    // `DocPoint` or a `FracRect` literal, never a screen coordinate.* A
    // `DragValue` is neither a document position nor a fraction of the canvas,
    // so the rule's spirit applies through a third route — the drag runs
    // between the left and right quarters of the rectangle the application
    // itself published, so it cannot go stale when the ribbon relayouts and it
    // cannot land outside the control.
    //
    // Left-to-right increases the value, which matters: dragging the other way
    // from a default of 2 pt would clamp at the pen's 0.25 floor after a short
    // distance, and a clamped value that stops moving is indistinguishable from
    // a control that does nothing.
    let frame = session.frame()?;
    let quarter = (width_rect.max.x - width_rect.min.x) / 4.0;
    let from = frame.declared_center(crate::geom::LRect {
        min: egui_point(width_rect.min.x + quarter, width_rect.min.y),
        max: egui_point(width_rect.min.x + quarter, width_rect.max.y),
    });
    let to = frame.declared_center(crate::geom::LRect {
        min: egui_point(width_rect.max.x - quarter, width_rect.min.y),
        max: egui_point(width_rect.max.x - quarter, width_rect.max.y),
    });
    driver.drag(from, to)?;
    session.settle(12);

    let after = shell_pen_lines(&session)?;
    if after <= before {
        return Ok(Some(format!(
            "the three Style controls are drawn, and dragging the width produced no new \
             `{PEN_EVENT}` line ({before} before, {after} after). The group renders and its \
             state is inert, which is the same outcome as not rendering it — the operator moves \
             a control and the next mark is authored at the old width."
        )));
    }

    let trace = session.trace()?;
    let last = trace
        .events(PEN_EVENT)
        .last()
        .and_then(|l| l.get("width_pts").map(str::to_owned))
        .unwrap_or_default();
    report.note(format!(
        "the width control is live: {} `{PEN_EVENT}` line(s), last width_pts={last}",
        after
    ));
    Ok(None)
}

/// A logical point, for building a sub-rect of a declared one.
///
/// Exists only so the drag endpoints above can be expressed as *parts of a
/// rectangle the application published* rather than as coordinates this file
/// invented — see the comment at the call site.
fn egui_point(x: f32, y: f32) -> crate::geom::Pt {
    crate::geom::Pt { x, y }
}

/// How many `markup-pen` lines the application has written so far.
///
/// Counted rather than compared by value, for the reason
/// `checks::delete_key`'s header sets out about absences: a count that goes up
/// is positive evidence that a control acted, whereas an unchanged *value*
/// could mean the control did nothing or that it was dragged back to where it
/// started.
fn shell_pen_lines(session: &Session) -> Result<usize> {
    Ok(session.trace()?.events(PEN_EVENT).count())
}
