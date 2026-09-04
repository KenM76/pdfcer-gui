//! `font_folders_lands_on_the_fonts_setting` — **Tools ▸ Font folders opens
//! the window AT the folders, with the checkbox reachable.**
//!
//! # What this is for
//!
//! `OPERATOR_REQUESTS.md` **O50**. He asked for a font-folder setting that had
//! shipped the day before, and the interesting half of that is not that he
//! missed a row:
//!
//! > the route built for exactly his question — a command called *Font folders*
//! > — dropped him at the top of ten **collapsed** headings and left the finding
//! > to him.
//!
//! ⇒ **A route that exists because of one setting must land on that setting.**
//! Opening the right window is not the same as answering the question the
//! command's own name asks.
//!
//! ## ★★★ Why this check is possible at all, and what it replaces
//!
//! The Fonts group is inside a `ScrollArea`, below the fold, inside a
//! `CollapsingHeader` that is **closed by default**. A control in that state
//! publishes no `ui_rect_visible` region — deliberately, because
//! `settings_headings_legible` once measured three headings that were laid out
//! and clipped and reported the drawing behind the dialog as illegible text.
//!
//! An earlier session tried to drive the Settings scroll to reach a group,
//! fixed four separate causes, still failed, and reverted. **This check does not
//! scroll.** It invokes the command whose job is to land there, and asserts that
//! the landing happened — which is both a smaller mechanism and a truer test,
//! because scrolling is not what an operator does either: they press the thing
//! named after what they want.
//!
//! ## ★★ The oracle is a VISIBLE region, and the distinction is the point
//!
//! `settings.fonts.use_os` is published through `ui_rect_visible`, which needs
//! 60 % of the control inside the clip rect. So the assertion *"this region was
//! declared"* means the checkbox is **on screen and pressable**, not merely
//! constructed. A build that forced the group open and did not scroll to it
//! would lay the checkbox out below the fold and publish nothing — and would
//! pass any check that asked whether the group existed.
//!
//! ## What this does NOT cover
//!
//! **Ticking it.** The click and its effect on an embed are a second gesture and
//! a second document; `embedding_works_with_no_font_folder_at_all` already
//! drives the *resolver* half from the other end. What is asserted here is the
//! half O50 is actually about: that an operator looking for this can find it.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The command under test — **not** `file.settings`.
///
/// ★ The two open one window and ask different questions, which is why they
/// stopped sharing a route. `file.settings` is *"show me the settings"* and
/// lands at the top, correctly; this one is *"where do font folders live"*.
const INVOKE: &str = "tools.font_folders";
/// The Fonts group's own region.
const GROUP: &str = "settings.fonts";
/// The checkbox O50 asked for, published only when it is genuinely on screen.
const CHECKBOX: &str = "settings.fonts.use_os";
/// The Add-folder button, so a failure can distinguish "the group did not open"
/// from "the group opened and the checkbox is missing".
const ADD: &str = "settings.fonts.add";

/// See the module documentation.
pub struct FontFoldersLandsOnTheFontsSetting;

impl Check for FontFoldersLandsOnTheFontsSetting {
    fn name(&self) -> &'static str {
        "font_folders_lands_on_the_fonts_setting"
    }

    fn defect(&self) -> &'static str {
        "Tools ▸ Font folders opens the Settings window at the top of ten collapsed headings and \
         leaves the operator to find the one the command is named after — which is how he came \
         to ask for a font-folder setting that had shipped the day before"
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
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    // ★ No `--pdf`, and no input. Settings is one of the few windows that must
    // work with nothing open — the folders are a preference, not a property of
    // a document — and driving it on an empty shell is what proves that. It also
    // makes this one of the few checks that runs under `--no-input`: the
    // landing is the subject, and a landing needs no pointer.
    let mut spec = LaunchSpec::new(&exe, ctx.out("os-fonts-setting.trace.txt"));
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
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE} and no document open",
        exe.display(),
        session.pid()
    ));
    // The Settings window is its own OS viewport and the scroll lands over
    // several frames — the forced-open group changes the content height, which
    // is what the scroll is then solved against.
    session.settle(50);

    let trace = session.trace()?;
    if declared(&trace, ui_rect, GROUP).is_none() {
        return Ok(Some(format!(
            "★ TOOLS ▸ FONT FOLDERS OPENED NO SETTINGS WINDOW, or opened one without a Fonts \
             group: no `{GROUP}` region.\n\
             The command shares `file.settings`' dispatch arm as of 2026-08-28; before that it \
             raised `Action::Command(\"file.settings\")` from `dispatch::routes`. If neither \
             fires, the id has no claimant at all. Regions beginning `settings`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "settings")),
            session.trace_path().display()
        )));
    }

    let Some(rect) = declared(&trace, ui_rect, CHECKBOX) else {
        // ★★ The two-way diagnosis, and it is why `ADD` is read at all.
        //
        // The Add button sits ABOVE the checkbox in the same group. If it
        // published and the checkbox did not, the group opened and the window
        // did not scroll far enough — the body is taller than the view. If
        // neither published, the group never opened, which is the
        // `CollapsingHeader::open(Some(true))` half rather than the
        // `scroll_to_me` half. Two causes, one symptom, and the failure names
        // which.
        let add_seen = declared(&trace, ui_rect, ADD).is_some();
        let (cause, place) = if add_seen {
            (
                "the group OPENED and the window did not scroll far enough to bring the \
                 checkbox into view",
                "`widgets::group_focused`'s `scroll_to_me`, and the group's own height",
            )
        } else {
            (
                "the group never OPENED, so nothing inside it was laid out where it could be \
                 seen",
                "`Draft::focus` reaching `show`, and `CollapsingHeader::open(Some(true))`",
            )
        };
        return Ok(Some(format!(
            "★★★ THE CHECKBOX O50 ASKED FOR IS NOT REACHABLE: the Fonts group is on screen and \
             no `{CHECKBOX}` region was published, which means {cause}.\n\
             It is published through `ui_rect_visible`, so an absence means the control is not \
             60 % inside the clip — laid out, and not pressable. Look at {place}. Regions \
             beginning `settings.fonts`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "settings.fonts")),
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★★★ Tools ▸ Font folders landed on the Fonts group and the checkbox is on screen at \
         {rect:?} — an operator asking where font folders live is shown them"
    ));

    // ★ The Add button too, reported rather than asserted: it is above the
    // checkbox, so its presence is implied by the assertion that passed. It is
    // noted because a run where only ONE of them is visible is a group whose
    // height has grown past the view, and the next person to add a row here
    // will want the measurement rather than a rediscovery.
    if declared(&trace, ui_rect, ADD).is_some() {
        report.note("the Add-folder button is on screen with it");
    } else {
        report.note(
            "★ the Add-folder button is NOT on screen, though the checkbox is — the group has \
             grown taller than the view and the landing now shows its bottom half",
        );
    }
    Ok(None)
}
