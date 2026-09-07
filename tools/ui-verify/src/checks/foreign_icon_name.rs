//! `a_foreign_icon_name_reaches_the_panel` — **a sticky note whose icon name
//! pdfcer does not model shows that name, and is not silently renamed.**
//!
//! # What this is for
//!
//! ISO 32000-1 §12.5.6.4 gives seven icon names for a `/Text` annotation and
//! says they are *"a standard set, not a closed one"* — a producer's own name
//! is **conforming**. Until `pdfcer-core` `Pass 253.5` the engine's reader ran
//! `/Name` through `StickyIcon::from_name` and `.unwrap_or(Note)`, so a note
//! carrying `/Sparkle` read back as `Note` and a restyle of its **colour
//! alone** wrote `/Name /Note` into the operator's file. This shell filed it
//! (`request_set_text_annot_style_rewrites_a_foreign_icon_name.md`) and worked
//! around it by reading the raw dictionary beside the spec.
//!
//! The engine now carries the bytes (`StickyIcon::Other`), the workaround is
//! deleted, and this check is what says the whole chain arrived.
//!
//! ## ★★★ Why the in-process tests are not enough
//!
//! `tests/engine_overlay_skew.rs`'s
//! `a_foreign_icon_name_survives_a_colour_only_restyle` proves the **engine**
//! keeps the name through a colour change, end to end, on this fixture. It
//! cannot prove any of what an operator meets:
//!
//! | # | link | its own test |
//! |---|---|---|
//! | 1 | a click on the note selects it as an annotation | `selection::annot` — the geometry, given candidates |
//! | 2 | the Properties panel resolves it to `Reach::TextAnnot` | `markup::tests` — the routing, given a spec |
//! | 3 | **the icon subsection actually draws** | **nothing** — it is behind a dock tab, a mode and a scroll |
//! | 4 | **the chooser shows the FILE's name and not a variant label** | **nothing** |
//! | 5 | the foreign-icon note is shown beside it | **nothing** |
//!
//! **Link 3 is the one that would ship as silence.** In Review the right dock
//! opens on Comments, and a dock draws only its active tab — so a panel that is
//! perfect and a panel that is broken produce the same empty trace. This check
//! brings the tab forward before it reads anything, which is a lesson this
//! project has paid for three times in one afternoon.
//!
//! ## ★★ The oracle is a trace line, and it has to be
//!
//! A build that flattened `/Sparkle` to `Note` and one that carried it draw
//! **the same rectangle, in the same place, with the same controls**, differing
//! only in the words inside a combo box. No published region can see that, and
//! a pixel comparison of rendered text at this size is not an assertion anybody
//! should build on. `textannot-rows` carries `icon=` as the file spells it, so
//! the check reads the name and not the layout.
//!
//! ## ★ The fixture is PLANTED, and a weaker one would make this vacuous
//!
//! `fixtures/foreign-icon-name.pdf` is `comment-note.pdf` with one `/Comment`
//! rewritten to `/Sparkle` — the same length, so every byte offset in the file
//! is preserved. A fixture that merely *omitted* `/Name` would not defeat the
//! old behaviour at all: Table 172's default is `Note`, so the absent case and
//! the flattened case produce the identical value. The name has to be present,
//! conforming, and outside the seven.
//!
//! ⚠ It carries **two** sticky notes — an ordinary `/Note` and the `/Sparkle` —
//! which is deliberate and is why this check aims at a rect it derives from the
//! trace rather than at "the first annotation". The in-process test's first
//! draft took the first `/Text` on the page and reported, confidently and
//! wrongly, that the engine had flattened a name.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Review, where markup lives and where the Comments tab is in front.
const INVOKE: &str = "mode.review";
/// The line the properties panel writes saying what its icon subsection shows.
const ROWS_EVENT: &str = "textannot-rows";
/// The line the canvas writes when a click selects an annotation.
const SELECT_EVENT: &str = "annot-select";
/// The right dock's Properties tab — clicked before anything is read.
const PROPERTIES_TAB_REGION: &str = "dock.tab.file.properties";
/// The page's own region, so a failure can say whether a sheet was drawn.
const PAGE_REGION: &str = "page";
/// The icon name planted in the fixture.
const PLANTED: &str = "Sparkle";

/// See the module documentation.
pub struct AForeignIconNameReachesThePanel;

impl Check for AForeignIconNameReachesThePanel {
    fn name(&self) -> &'static str {
        "a_foreign_icon_name_reaches_the_panel"
    }

    fn defect(&self) -> &'static str {
        "a sticky note whose icon name is one pdfcer does not model is shown in the properties \
         panel as one of the seven it does — so the chooser asserts a value the file does not \
         carry, and a change to the note's COLOUR ALONE rewrites its icon name in the operator's \
         file, silently. Or the panel shows the name correctly and omits the sentence saying \
         pdfcer draws its own symbol for it, which leaves an operator comparing this window with \
         Acrobat's with no explanation for the difference"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a sticky note and a dock tab; both \
             are real pointer gestures.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;

    // ★★★ **This check PINS its own fixture and ignores `--pdf`.**
    //
    // The defect is a property of a document carrying a `/Name` outside
    // §12.5.6.4's seven, and no drawing the operator would pass on the command
    // line has one. A check that took `--pdf` here would run against a file
    // with nothing to find and report a clean pass — which is the "green check
    // over an empty scan" this project has already been bitten by. It is
    // declared rather than silent: the note below says the argument was
    // ignored and why.
    let pdf = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/foreign-icon-name.pdf");
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the pinned fixture is missing at {}. It is `comment-note.pdf` with one `/Comment` \
             icon name rewritten to `/{PLANTED}` — the same length, so every byte offset is \
             preserved. This check cannot be run against another document: no ordinary drawing \
             carries an unmodelled icon name.",
            pdf.display()
        )));
    }
    if ctx.pdf.is_some() {
        report.note(
            "--pdf was supplied and is deliberately ignored; this check pins its own fixture",
        );
    }

    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("foreign-icon.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} on the pinned fixture",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen. \
             Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- 1: bring the Properties tab forward -------------------------------
    //
    // Before the click, so the panel is already the visible tab when the
    // selection lands and its first frame is the one that draws the rows.
    let trace = session.trace()?;
    let Some(tab) = declared(&trace, ui_rect, PROPERTIES_TAB_REGION) else {
        return Ok(Some(format!(
            "the right dock declared no `{PROPERTIES_TAB_REGION}`, so the Properties panel could \
             not be brought forward and nothing below this line can be measured. Tabs declared: \
             {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "dock.tab.")),
            session.trace_path().display()
        )));
    };
    driver.click_at(session.frame()?.declared_at(tab, 0.5, 0.5))?;
    session.settle(20);
    report.note("★ the Properties tab was brought forward");

    // --- 2: click the sticky note ------------------------------------------
    //
    // ★ Aimed at the note's own `/Rect`, which the fixture fixes at
    // `[104 604 124 624]` in PDF user space. Expressed through the harness's
    // document-point mapping rather than as a screen coordinate, so the aim
    // survives the window size, the zoom and the dock widths.
    let page = match ctx.page_size {
        Some((w, h)) => crate::coords::PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf)
            .ok_or_else(|| Error::new("could not read the pinned fixture's page size."))?,
    };
    let at = crate::checks::text_selection::aim(
        ctx,
        &session,
        page,
        crate::coords::DocPoint::new(0, 114.0, 614.0),
    )?;
    driver.click_at(at)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(selected) = trace.events(SELECT_EVENT).last() else {
        return Ok(Some(format!(
            "THE STICKY NOTE COULD NOT BE SELECTED: a click at its `/Rect` centre produced no \
             `{SELECT_EVENT}` line. Selecting an annotation is the step before the one under \
             test and has worked since 2026-08-18, so this says the click missed — check the \
             fixture still puts a `/Text` at [104 604 124 624]. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the note was selected: `{}`", selected.raw));

    // --- 3: ★★★ WHAT DOES THE PANEL SAY THE ICON IS? -----------------------
    let trace = session.trace()?;
    let Some(rows) = trace.events(ROWS_EVENT).last() else {
        return Ok(Some(format!(
            "★★★ THE ICON SUBSECTION DID NOT DRAW. No `{ROWS_EVENT}` line followed the \
             selection, and that line is written unconditionally at the top of \
             `panels::properties::markup::textannot::rows`.\n\
             Three things produce this and they are in different files:\n\
             1. the Properties panel is not the visible dock tab — but this check clicked it \
             forward two steps ago, and said so above;\n\
             2. `Current::reach` did not answer `Reach::TextAnnot`, so the parent drew the \
             MARKUP rows instead. `annot_author::text_spec_from_dict` refusing the annotation \
             is the usual cause;\n\
             3. the section is scrolled out of the panel's viewport — but `ui_rect_visible` \
             would still have let the trace line through, since it is not gated on visibility.\n\
             Regions beginning `properties.`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "properties.")),
            session.trace_path().display()
        )));
    };
    report.note(format!("★★ the icon subsection drew: `{}`", rows.raw));

    let icon = rows.get("icon").unwrap_or("");
    if icon != PLANTED {
        return Ok(Some(format!(
            "★★★ THE PANEL SHOWS THE ICON AS `{icon}` AND THE FILE SAYS `/{PLANTED}`.\n\
             \n\
             `Note` is the historic failure and the one this check was written for: \
             `annot_author::text_spec_from_dict` used to run `/Name` through \
             `StickyIcon::from_name` and `.unwrap_or(StickyIcon::Note)`, so a conforming \
             producer name was flattened on the way in — and `set_text_annot_style` re-baked \
             from that spec, so changing the note's COLOUR ALONE wrote `/Name /Note` into the \
             operator's file with nothing on screen saying so.\n\
             \n\
             If that is what this says, the engine pin has gone BACKWARDS past `Pass 253.5`, or \
             this shell has stopped reading `StickyIcon::Other`. Check `Cargo.lock`, then \
             `Reading::of` — and note that `tests/engine_overlay_skew.rs`'s \
             `a_foreign_icon_name_survives_a_colour_only_restyle` asserts the engine half in \
             process, so if that is green and this is red the break is in this shell.\n\
             \n\
             `none` means the panel decided the face has no icon at all, which for a `/Text` is \
             a routing failure rather than a reading one. Line: `{}`. Trace: {}.",
            rows.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the chooser shows the file's own name: icon={icon}"
    ));

    // --- 4: and it is disclosed ---------------------------------------------
    //
    // ★★ Asserted separately from step 3, and that is the point. A build that
    // carried the name and forgot the sentence passes everything above it,
    // looks completely correct, and leaves an operator comparing this window
    // with the program that made the note with no explanation for why the two
    // symbols differ. Nothing is lost any more — so this is no longer a
    // warning, and it is still owed.
    if rows.get("foreign") != Some("1") {
        return Ok(Some(format!(
            "★★ THE NAME IS CARRIED AND THE PANEL DOES NOT SAY IT IS AN ICON PDFCER DRAWS ITS \
             OWN WAY: `{}`.\n\
             `foreign=0` means `Reading::foreign_icon` answered false for a `StickyIcon::Other`, \
             which is a contradiction inside one struct — the field is derived from the same \
             value `icon=` was printed from. `Reading::of`'s `matches!(icon, \
             StickyIcon::Other(_))` is the one line. Trace: {}.",
            rows.raw,
            session.trace_path().display()
        )));
    }
    report.note("★★ and it is disclosed: pdfcer draws its own symbol for it");

    Ok(None)
}
