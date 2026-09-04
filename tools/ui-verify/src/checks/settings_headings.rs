//! `settings_headings_legible` — the regression test for **D2**.
//!
//! # The defect
//!
//! Every collapsible section heading in the Settings dialog — *Appearance,
//! Theme, Colour, Images and transparency, Copying and extracting text, Pages
//! and printing, Saving files* — renders near-white on light grey. So do the
//! dock tab labels. At 1× they are simply not readable.
//!
//! ## Cause
//!
//! `theme.rs:434-450` loops over all five widget states setting
//! `corner_radius`, `bg_stroke` and `fg_stroke`, and then:
//!
//! ```text
//! v.widgets.inactive.weak_bg_fill = p.panel;
//! v.widgets.hovered.weak_bg_fill  = p.surface;
//! v.widgets.active.weak_bg_fill   = p.accent;
//! v.widgets.active.fg_stroke      = Stroke::new(1.0, p.label_backdrop);
//! ```
//!
//! `label_backdrop` is `rgba(250,250,250,220)`. Pairing it with the accent is
//! *correct* — light text on an accent fill. But only `weak_bg_fill` is
//! assigned the accent. **`widgets.active.bg_fill` is never set at all.** So
//! widgets that paint their background with `bg_fill` rather than
//! `weak_bg_fill` — `egui_tiles` tab buttons, `CollapsingHeader` headers — get
//! the near-white foreground on a light background.
//!
//! # Why CI did not catch it
//!
//! Two tests sit directly adjacent and neither covers it:
//!
//! * `text_contrasts_with_its_background_in_every_preset` checks `text`
//!   against `surface`/`panel` and `text_muted` against `surface`. It never
//!   tests `label_backdrop`.
//! * `label_plates_stay_page_facing_not_chrome_facing` **asserts that
//!   `label_backdrop` stays light** — correct for its stated purpose, because
//!   labels also sit over the white page — without checking what is actually
//!   behind it in chrome.
//!
//! Both test the *palette*. The defect is in the *pairing*, and the pairing
//! only exists once something is drawn. There is one oracle for that, and it
//! is the rendered screenshot.
//!
//! # How this check detects it
//!
//! It measures the WCAG contrast ratio of each heading's rendered region
//! against its own background, using the population algorithm in
//! [`crate::pixels`] rather than a min/max that a single stray pixel could
//! fake. The threshold is WCAG 2.1 AA for large text, 3:1 — a published
//! standard rather than a matter of taste, which is what stops a failing check
//! becoming an argument about whether the grey is nice.
//!
//! The defect measures around **1.1:1**.
//!
//! # Two modes, and why the offline one exists
//!
//! * **Live** — drive the application to its Settings dialog and capture. This
//!   is the mode the new application will use. Half of what it needs now
//!   exists: `diag::ui_rect` is in the new binary and the dialog's headings
//!   would be located by declaring themselves, with no fractions for the
//!   harness to hard-code and no calibration to go stale when the dialog is
//!   resized. The other half does not: **there is no Settings dialog yet**,
//!   and no scripted step that would open one.
//! * **Offline (`--image`)** — assert against a screenshot somebody already
//!   captured. This exists for falsification: `evidence/crop_settings.png` is
//!   the dated artefact `DEFECTS.md` D2 cites, and running this check against
//!   it is how the harness demonstrates it detects the real defect rather than
//!   merely claiming to.
//!
//! So the live mode SKIPs against both binaries, for a reason that is about
//! **modal state** rather than about the trace: the new application has no
//! Settings dialog, and the old one has a dialog with no scripted way in.
//! Pointing this check at the old GUI's own captured evidence reports FAIL.
//! Both are honest, and the second is the acceptance criterion.
//!
//! Note that this is exactly why this check does not launch anything, while
//! [`super::ribbon_captions`] does. A ribbon is chrome — it is on screen as
//! soon as the window is, so its trace answers the question. A dialog is not,
//! so launching would confirm only that a dialog nobody opened declared no
//! regions, and would put a window on the operator's desktop to do it.

use crate::checks::legibility;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::image::Image;
use crate::report::CheckReport;

/// See the module documentation.
pub struct SettingsHeadingsLegible;

/// The region set this check asks a profile for.
const SET: &str = "settings_headings";

/// The prefix the application publishes each collapsible heading's rect under.
///
/// Matched **literally**, so it is part of the contract with
/// `crate::dialogs::settings::REGION_HEADING_PREFIX`. That constant's own doc
/// comment states the other half of the bargain: the key is deliberately not
/// derived from the caption, because a caption is operator copy that may be
/// reworded or translated, and a check aimed at a region named after it would
/// silently stop finding its subject and report *a heading that is not there*
/// rather than *a heading that is illegible*. Those are different verdicts and
/// only one of them is true.
const HEADING_PREFIX: &str = "settings.heading.";

/// How this check describes what it looked for, when it found nothing.
///
/// Completes the sentence "the application declared no …", so it names this
/// check's own convention rather than a generic one — a reader who gets this
/// SKIP should know exactly which string to grep the application for.
const CONVENTION: &str = "settings dialog heading regions (`settings.heading.<group>`)";

impl Check for SettingsHeadingsLegible {
    fn name(&self) -> &'static str {
        "settings_headings_legible"
    }

    fn defect(&self) -> &'static str {
        "D2 — the theme assigns widgets.active.fg_stroke a near-white but never \
         assigns widgets.active.bg_fill, so CollapsingHeader headings and dock tab \
         labels render near-white on light grey"
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

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // Offline mode first: it needs no binary, so a run that has an image and
    // no application still produces a verdict. A PNG cannot declare its own
    // regions, so no trace is consulted — passed as an explicit `None` so the
    // SKIP reason cannot imply anything about a trace nobody read.
    if let Some(path) = &ctx.image {
        report.note(format!("asserting against the image {}", path.display()));
        let plan = legibility::resolve_set(ctx.profile, SET, Some(path), None)
            .map_err(crate::error::Error::new)?;
        let image = Image::load_png(path)?;
        return Ok(legibility::assess(
            &image,
            &plan,
            ctx.contrast_threshold,
            report,
        ));
    }

    // ★★ LIVE MODE — built 2026-08-17, after this check had SKIPPED for the
    // whole life of the project.
    //
    // The reason it skipped was correct when written and had gone stale twice
    // over: *"the new application has no Settings dialog at S2"* — it has had
    // one since 2026-08-17 — and *"neither known binary accepts a scripted way
    // in"*, which stopped being the blocker the moment the dialog landed with
    // a ribbon control that can be clicked.
    //
    // That is worth a sentence rather than a quiet deletion, because a SKIP
    // whose reason has expired is the most comfortable kind of untested code:
    // it reports honestly, it is not a failure, and nothing ever revisits it.
    // This one guarded **D2**, which is the defect that justified building the
    // harness — headings rendering at about 1.1:1 against a 3:1 floor.
    let exe = ctx.resolve_exe().ok_or_else(|| {
        crate::error::Error::new(format!(
            "no binary to drive and no --image to assert against. Pass --exe, or build the profile's default at {}, or point the check at a captured screenshot.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(crate::error::Error::new(
            "input is disabled (--no-input). This check has to click the Settings control open before there is a heading to measure. Reported as SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        crate::error::Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say where its headings are.",
            ctx.profile.name
        ))
    })?;

    let mut spec = crate::launch::LaunchSpec::new(&exe, ctx.out("settings_headings.trace.txt"));
    spec.pdf = ctx.pdf.clone();
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env.push((
        crate::checks::driving::SHELL_DIAG_ENV.0.to_owned(),
        crate::checks::driving::SHELL_DIAG_ENV.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = crate::launch::Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let driver = crate::input::Driver::new(session.window());
    open_settings(&session, &driver, ui_rect, report)?;

    // The headings only exist once the dialog is up, so the plan is resolved
    // from the trace taken AFTER it opened — not from the launch trace, which
    // is the ordering mistake that would report "no headings declared" about a
    // dialog that had not been asked to appear yet.
    let trace = session.trace()?;
    // ★★ THE DIALOG'S OWN WINDOW. Settings became a real OS window on
    // 2026-08-21, and a capture of the application shows the page where the
    // dialog used to be — while the contrast sampler goes on sampling and
    // reports a confident 1.51:1 about a piece of the drawing. **A measurement
    // of the wrong surface is indistinguishable from a measurement of a broken
    // one**, which is the worst failure a check of this kind can have.
    let frame = crate::checks::driving::frame_of(&session, &trace, ui_rect, "dialog:settings")?;
    let png = ctx.out("settings_headings.png");
    let image = crate::capture::frame_to_png(&session, &frame, &png)?;
    report.artifact(png);

    // The regions this check is about: every `settings.heading.<key>` the
    // dialog declared. One per collapsible header, so each is measured against
    // ITS OWN background — D2 was a foreground/background pairing, and a
    // pairing only exists once something is drawn.
    let declared_regions = ctx.profile.vocab.declared_regions(&trace);
    // ★★★ A REGION BELOW THE FOLD IS NOT A REGION THIS CHECK CAN MEASURE, and
    // leaving that out produced a confident 1.53:1 about a heading that renders
    // perfectly well.
    //
    // The application declares a heading it has laid out, and a `ScrollArea`
    // lays out what is past its bottom edge as readily as what is above it. The
    // rect is therefore honest and the pixels at it are not the heading: they
    // are whatever is under the window, sampled through a rectangle that has
    // been clamped to the capture's edge.
    //
    // **A measurement of the wrong surface is indistinguishable from a
    // measurement of a broken one.** That is the same sentence the capture
    // fix above carries, and this is its second instance in one afternoon: the
    // first was the wrong WINDOW, this is the wrong part of the right one.
    // ★ Tested on the RAW rect, NOT through `logical_to_capture_pixels`, and
    // the first version of this filter made exactly that mistake and changed
    // nothing. That conversion **clamps to the client area** — which is right
    // for aiming a sampler and fatal for asking whether something is inside it,
    // because a rect past the bottom edge comes back sitting exactly ON the
    // bottom edge and passes every containment test written against it.
    let (client_w, client_h) = frame.client_size;
    let logical_w = client_w as f32 / frame.scale;
    let logical_h = client_h as f32 / frame.scale;
    let visible = |r: crate::geom::LRect| -> bool {
        r.min.x >= 0.0
            && r.min.y >= 0.0
            && r.max.x <= logical_w
            && r.max.y <= logical_h
            && r.max.x > r.min.x
            && r.max.y > r.min.y
    };
    let trace_regions = legibility::TraceRegions {
        matched: declared_regions
            .iter()
            .filter(|r| r.name.starts_with(HEADING_PREFIX))
            .filter(|r| visible(r.rect))
            .map(|r| legibility::PlannedRegion {
                name: r.name.clone(),
                area: legibility::RegionArea::Pixels(frame.logical_to_capture_pixels(r.rect)),
            })
            .collect(),
        declared: declared_regions.iter().map(|r| r.name.clone()).collect(),
        convention: CONVENTION,
    };
    // ★ A NAMED LIMIT, reported rather than left for a reader to infer from a
    // small number.
    //
    // The application only declares a heading it can actually draw (see
    // `diag::ui_rect_visible`), so this measures the headings **currently in
    // view** and not the whole window. Scrolling is not driven. Without this
    // note a reader sees "2 region(s)" beside a dialog they know has eight
    // groups and has to work out whether six are missing or six are below the
    // fold — and the first of those readings is alarming and wrong.
    //
    // Left as a limit rather than fixed because fixing it means driving the
    // scroll area and re-capturing per scroll position, which is a real piece
    // of work; and because the value here is mostly in the FIRST heading. D2
    // was a theme-wide pairing — `widgets.active.fg_stroke` against a fill the
    // palette never assigned — so it would show on every heading at once, not
    // on the seventh.
    //
    // ★★★ **ATTEMPTED AND REVERTED on 2026-08-28, and the attempt is recorded
    // because the estimate above turned out to be exactly right.**
    //
    // `driving::scroll_to` had just been written for the form-field pane, so
    // "a real piece of work" looked like it had become an import. It had not.
    // Four causes were found and fixed on the way and the dialog still did not
    // move:
    //
    // 1. the body's rect is `ui.max_rect()` published **before** the scroll
    //    area, so its centre is not reliably inside the scrollable region —
    //    the same defect fixed in `properties.form_field` the same evening;
    // 2. anchoring on the last visible heading instead did not help;
    // 3. the dialog is a **child viewport**, so `session.frame()` maps its
    //    rects through the wrong window — `frame_of` is required, and using it
    //    still did not make the wheel land;
    // 4. the `ui-rect` trace is a **change log**, so a dialog that has finished
    //    laying out stops emitting its body rect, and re-reading the anchor
    //    each iteration found `None` on the second pass and stopped the loop
    //    after one wheel event.
    //
    // Fixing all four produced a sweep that reached **fewer** headings than
    // doing nothing. Whatever the fifth reason is, it is not known, and a sweep
    // that reports a subset in the confident voice of a whole is worse than a
    // stated limit.
    //
    // ⇒ Reverted to this note. ★ The one thing that survives is
    // `driving::scroll_to`, which is now shared and does work on dock panels —
    // three callers in `form_field` rely on it.
    //
    // **What is still unmeasured, and it is not a legibility gap:** five of the
    // seven groups have never been observed by anything, so *"does this group
    // draw at all"* has no instrument. That was found when a new group was
    // added below the fold on 2026-08-28 and nothing could say whether it
    // existed. It is a coverage gap wearing a legibility gap's clothes and it
    // wants its own check, not a bigger version of this one.
    report.note(format!(
        "measures the {} heading(s) currently IN VIEW; the dialog scrolls and this check does not drive the scroll, so headings below the fold are not measured. D2 was a theme-wide foreground/background pairing and would show on the first heading as readily as the last",
        trace_regions.matched.len()
    ));

    let plan = legibility::resolve_set(ctx.profile, SET, None, Some(&trace_regions))
        .map_err(crate::error::Error::new)?;
    Ok(legibility::assess(
        &image,
        &plan,
        ctx.contrast_threshold,
        report,
    ))
}

/// **Get the Settings dialog on screen**, by whichever route this window
/// offers.
///
/// # ★★★ THREE routes, and this file knew about two until 2026-08-27
///
/// `file.settings` is the first item of the *pdfcer* group, which is the LAST
/// group on the File tab. At the shipped 1100 pt window width that group does
/// not fit, so the item has no rect of its own — and there are **two different
/// things** the ribbon may have done with it:
///
/// | | what the trace shows | how to reach the item |
/// |---|---|---|
/// | on the band | `ribbon.item.file.settings` | click it |
/// | folded into the overflow | `ribbon.overflow` | open the overflow, then click |
/// | **collapsed as a group** | `ribbon.group.file.pdfcer.collapsed` | click the group button, then click |
///
/// The third arrived with the O31 ribbon work, which gave the band a middle
/// rung: when it runs short of width a whole group folds into one captioned
/// button whose items live in its popup. This function hand-rolled the first
/// two and therefore **could not run at all** afterwards — every invocation
/// SKIPped with *"neither `ribbon.item.file.settings` nor `ribbon.overflow` was
/// declared"*, which was true, and not the whole truth: the trace it printed
/// contained `ribbon.group.file.pdfcer.collapsed` five lines down.
///
/// ★ **It SKIPped rather than FAILed, and that is the only reason this was
/// cheap.** A check that had claimed the Settings control was missing would
/// have sent somebody looking for a defect in a ribbon that was behaving
/// exactly as designed — the false-failure-believed pattern this suite has paid
/// for twice. The honest SKIP cost nothing but the coverage.
///
/// # The fix is to stop hand-rolling it
///
/// `driving::declared_or_in_overflow` already knows all three places and tries
/// them in the right order — direct, then each collapsed group (non-destructive:
/// a popup can be opened and closed without moving the band), then the overflow
/// (which scrolls, and so must be last). It was written for exactly this and
/// `export_dxf` already uses it.
///
/// **A rule stated twice is a rule that drifts**, and this is what the drift
/// looks like: the shared helper gained a third case, this copy did not, and
/// nothing failed — the check simply stopped being able to begin. There is now
/// one statement of "where can a ribbon command be".
fn open_settings(
    session: &crate::launch::Session,
    driver: &crate::input::Driver,
    ui_rect: &str,
    report: &mut CheckReport,
) -> crate::error::Result<()> {
    use crate::checks::driving::{declared, declared_names, declared_or_in_overflow, list};

    const ITEM: &str = "ribbon.item.file.settings";

    let item = declared_or_in_overflow(session, driver, ui_rect, ITEM)?.ok_or_else(|| {
        crate::error::Error::new(format!(
            "`{ITEM}` was not on the band, in any collapsed group, or in the overflow, so there is no route to the Settings dialog on this window. Ribbon regions declared: {}.",
            list(&declared_names(&session.trace().unwrap_or_default(), ui_rect, "ribbon."))
        ))
    })?;
    report.note(
        "reached the Settings control through `declared_or_in_overflow`, which tries the band, then each collapsed group's popup, then the overflow. At the harness's 1100 pt window it is usually the second of those: the pdfcer group is the last on the File tab and collapses first.",
    );
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(24);

    // ★ Assert the dialog actually appeared before measuring anything.
    //
    // Without this the contrast pass would run against a window with no
    // dialog, find no heading regions, and report whatever `resolve_set` makes
    // of an empty set — a verdict about nothing, dressed as a verdict about
    // D2. The dialog declares its own body rect precisely so a harness can ask
    // this question.
    let trace = session.trace()?;
    if declared(&trace, ui_rect, "dialog:settings").is_none() {
        return Err(crate::error::Error::new(format!(
            "the Settings control was clicked and no `dialog:settings` region appeared, so the dialog did not open and there is nothing to measure. Regions declared beginning `dialog`: {}.",
            list(&declared_names(&trace, ui_rect, "dialog"))
        )));
    }
    report.note("the Settings dialog is open and declared its own body rect");
    Ok(())
}
