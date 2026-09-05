//! `copy_as_vector_places_the_measured_order` — the Clipboard group's copy-OUT
//! is reachable, the press places something, and **the SVG is first**.
//!
//! # The gap this closes — `OPERATOR_REQUESTS.md` **O120**
//!
//! The operator, 2026-09-03: *"I'd like to be able to copy and paste anything
//! to other software - like copy and paste vector graphics into word or
//! inkscape for example if possible."*
//!
//! O120's own Status line sets the bar and it is the engine's:
//!
//! > *"they get ticked when the GUI half is **driven**, not when it compiles."*
//!
//! ⚠⚠ **THIS CHECK HAS NOT BEEN RUN.** It was written on 2026-09-04 in a
//! session that was instructed not to launch the GUI — another track owned the
//! desktop, and `ui-verify` was deliberately not run. It is committed unrun,
//! with that stated here rather than implied by an absent result: **a check
//! nobody has executed is a check whose own correctness is unmeasured**, and
//! the first person to run it should expect to fix it rather than to read a
//! verdict from it. `export_image_emf` carries the same warning for the same
//! reason and on the same day.
//!
//! # ⚠ This check REPLACES the operator's clipboard, and it must
//!
//! It clears the clipboard, presses a button whose whole job is to write to the
//! clipboard, and reads back what landed. There is no version of driving this
//! feature that leaves the machine's clipboard alone, and pretending otherwise
//! would mean not checking the thing the operator asked for. Said here so it is
//! a known cost of running `ui-verify` rather than a surprise.
//!
//! ★ It is also why **no unit test anywhere in this project touches the real
//! clipboard**: `crate::clipboard` and `crates/native-clipboard` assert on the
//! bytes that *would* be placed, so a `cargo test` run cannot destroy anything.
//! The destructive act is confined to a harness the operator starts on purpose.
//!
//! # ★★★ Why the ORDER is the assertion, and availability is not enough
//!
//! A pasting application "typically retrieves … the first format it
//! recognizes". So the design of this feature *is* an order, measured by the
//! engine against a real Word paste through combridge: `image/svg+xml`, then
//! `CF_ENHMETAFILE`, then `PNG`, then `CF_DIBV5`.
//!
//! ⇒ A check that asked *"is `image/svg+xml` on the clipboard?"* would **pass
//! on the build that fails in Word** — the one that placed the raster formats
//! first. Word takes the first thing it recognises, stores it as a picture, and
//! nothing anywhere says so. `sys::clipboard_formats` walks
//! `EnumClipboardFormats`, which enumerates in placement order, precisely so
//! this check can assert on the *prefix* rather than on the *set*.
//!
//! ★★ **Windows synthesises formats, and they come after.** A placed
//! `CF_DIBV5` makes `CF_DIB` and `CF_BITMAP` appear too. That is why this
//! asserts a prefix and ignores the tail: a build that placed all four
//! correctly will show more than four entries, and demanding exactly four would
//! fail a correct build on a Windows behaviour nobody controls.
//!
//! # The five links this needs driving for
//!
//! 1. **the control exists on the Clipboard group.** `shell::manifest::edit`
//!    lists five members in a unit test whatever the ribbon draws; whether a
//!    fifth icon reached the screen, inside the band's height and not pushed
//!    into the overflow, is a question about a laid-out frame.
//! 2. **pressing it reaches the handler.** `shell::commands::reach` proves an
//!    arm exists by reading the source; it cannot prove the click lands on it.
//! 3. **the renderers run against the live session** — the `DocumentView` with
//!    its overlay and staging buffer, so a copy carries unsaved edits.
//! 4. **the Win32 transaction actually places anything.** Nothing in a unit
//!    test can reach `SetClipboardData`; `crates/native-clipboard`'s `unsafe`
//!    is verified by construction and by review, and **this is the only thing
//!    in the project that observes its effect.**
//! 5. **the order survives the transaction.** The order is decided in
//!    `crate::clipboard::ORDER` and asserted there — but a staging bug, a
//!    `HashMap` somewhere, or a future "tidy" that sorts the entries would
//!    reorder them between the assertion and the clipboard.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, declared_or_in_overflow, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode this runs in.
///
/// **Edit**, and not by preference: the shipped manifest shows the Edit tab in
/// the Edit mode alone, so the control is *absent* in Read and Review rather
/// than refusing. `app::modes::capability::offers_command` makes that a
/// property of the tab rather than a list, so there is nothing to exempt and
/// nothing that could drift.
const MODE: &str = "edit";
/// The ribbon control — `shell::manifest::edit`'s Clipboard group.
const CONTROL: &str = "ribbon.item.edit.copy_as_vector";
/// The Edit tab.
const TAB: &str = "ribbon.tab.edit";
/// The trace the handler emits when the placement succeeded.
const PLACED: &str = "clipboard-copy-out";
/// The trace it emits when it refused.
const REFUSED: &str = "clipboard-copy-out-refused";

/// The four format names in placement order, as `crate::clipboard::ClipFormat`
/// spells them.
///
/// ★ Written out here rather than imported, deliberately. `ui-verify` drives
/// the built binary through the operating system and must not link the crate
/// under test — an expected value taken from the code being checked is a
/// tautology, and this list is the *engine's measurement*, which is the
/// independent source. If these two ever disagree, one of them is wrong and the
/// check is the only thing that would say so.
const EXPECTED: [&str; 4] = ["image/svg+xml", "CF_ENHMETAFILE", "PNG", "CF_DIBV5"];

/// `CF_ENHMETAFILE`, which Windows has no *name* for — predefined formats are
/// matched by id.
const CF_ENHMETAFILE: u32 = 14;
/// `CF_DIBV5`, likewise.
const CF_DIBV5: u32 = 17;

/// See the module documentation.
pub struct CopyAsVectorPlacesTheMeasuredOrder;

impl Check for CopyAsVectorPlacesTheMeasuredOrder {
    fn name(&self) -> &'static str {
        "copy_as_vector_places_the_measured_order"
    }

    fn defect(&self) -> &'static str {
        "Edit > Clipboard > Copy as vector is missing, or the press places nothing, or it \
         places the raster formats ahead of the vector ones — so a paste into Word arrives as \
         a flat picture that cannot be scaled, recoloured or ungrouped, which is \
         indistinguishable to the operator from the feature not existing"
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

/// Whether a clipboard entry is the format `want` names.
///
/// Registered formats are matched by **name**, because their numeric ids are
/// assigned at run time and differ between boots. Predefined ones are matched
/// by **id**, because Windows gives them no name at all — see
/// `sys::clipboard_formats`.
fn is_format(entry: &(u32, String), want: &str) -> bool {
    match want {
        "CF_ENHMETAFILE" => entry.0 == CF_ENHMETAFILE,
        "CF_DIBV5" => entry.0 == CF_DIBV5,
        name => entry.1 == name,
    }
}

/// A clipboard listing, rendered for a failure message.
fn describe(formats: &[(u32, String)]) -> String {
    formats
        .iter()
        .map(|(id, name)| {
            if name.is_empty() {
                format!("#{id}")
            } else {
                format!("{name} (#{id})")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, a ribbon tab \
             and a ribbon control, and it REPLACES the clipboard. Reported as SKIPPED rather \
             than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    // ★★★ CLEARED FIRST, and this is the precondition without which the whole
    // check is theatre.
    //
    // `sys::clear_clipboard`'s own documentation carries the argument, learned
    // from defect O18: a check that presses a button which does NOTHING and
    // then finds the right formats left by an earlier run passes while the
    // application is broken. Clearing is what turns the read below into a
    // statement about *this* run.
    if !crate::sys::clear_clipboard() {
        return Err(Error::new(
            "could not clear the clipboard before the run, so anything read afterwards could \
             have been placed by an earlier run or by another program. Reported as an error \
             rather than a failure: the application has not been asked to do anything yet.",
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("copy_as_vector.trace.txt"));
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
    session.settle(40);
    let driver = Driver::new(session.window());

    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- 1: the Edit tab ---------------------------------------------------
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "no `{TAB}` region in {MODE}. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);

    // --- 2: ★★ the CONTROL EXISTS ------------------------------------------
    //
    // Link 1 of the module header, and the one that catches the commonest way
    // this feature could be absent without any test noticing: the command is
    // registered, the manifest names it, and the band's fifth icon does not fit
    // — so it silently lives only in the overflow, or nowhere.
    // `declared_or_in_overflow` opens the overflow if it has to, which is the
    // honest bar: reachable, not necessarily on the band.
    let Some(item) = declared_or_in_overflow(&session, &driver, ui_rect, CONTROL)? else {
        return Ok(Some(format!(
            "the Edit tab declares no `{CONTROL}`, on the band or in the overflow, so there \
             is no way to reach the copy-out at all. Items declared: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.edit."
            ))
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    // A page render, an SVG recording, an EMF recording and a PNG encode, then
    // the transaction. Longer than a button press deserves and shorter than a
    // CAD sheet may take; a failure here reads as "no trace line", which the
    // message below distinguishes from a refusal.
    session.settle(60);

    // --- 3: ★ the handler RAN, and said which operand it took --------------
    let trace = session.trace()?;
    if let Some(refused) = trace.last(REFUSED) {
        return Ok(Some(format!(
            "`{CONTROL}` was clicked and the copy-out refused: `{}`. A refusal is the correct \
             behaviour for a page whose vector form cannot be made — but on an ordinary \
             fixture it means a writer failed, and the operator gets a sentence where they \
             asked for a copy.",
            refused.raw
        )));
    }
    let Some(placed) = trace.last(PLACED) else {
        let unimplemented = trace
            .events("command-unimplemented")
            .any(|l| l.get("id") == Some("edit.copy_as_vector"));
        return Ok(Some(if unimplemented {
            "`edit.copy_as_vector` was clicked and traced `command-unimplemented` — the \
             control is drawn and the arm is still scaffolded."
                .to_owned()
        } else {
            format!(
                "`{CONTROL}` was clicked and neither `{PLACED}` nor `{REFUSED}` followed. The \
                 click reached no handler, or the handler returned before saying anything, \
                 which is the one outcome the operator cannot distinguish from a button that \
                 does not work."
            )
        }));
    };
    report.note(format!("placed: `{}`", placed.raw));

    // --- 4: ★★★ the CLIPBOARD holds the measured PREFIX ---------------------
    //
    // Links 4 and 5, and the only observation of `native-clipboard`'s `unsafe`
    // anywhere in this project.
    let Some(formats) = crate::sys::clipboard_formats() else {
        return Err(Error::new(
            "the copy-out reported success and the clipboard could not be opened to read it \
             back. Reported as an error rather than a failure: another process holding the \
             clipboard is a flake, and calling it a defect would teach the wrong lesson.",
        ));
    };
    if formats.is_empty() {
        return Ok(Some(format!(
            "★ the shell traced `{}` and the clipboard is EMPTY. The transaction reported \
             formats it did not place — which is the worst failure available here, because \
             the status row tells the operator the copy worked and every paste they try \
             afterwards will offer them nothing.",
            placed.raw
        )));
    }
    report.note(format!("clipboard holds: {}", describe(&formats)));

    if formats.len() < EXPECTED.len() {
        return Ok(Some(format!(
            "★★ the clipboard holds {} format(s) and the measured order needs {}: {}. Placed: \
             {}. A partial placement is not a smaller success — if the two vector entries are \
             not both there, Word's paste is a flat picture.",
            formats.len(),
            EXPECTED.len(),
            EXPECTED.join(", "),
            describe(&formats)
        )));
    }

    for (position, want) in EXPECTED.iter().enumerate() {
        let entry = &formats[position];
        if !is_format(entry, want) {
            let found_elsewhere = formats.iter().any(|e| is_format(e, want));
            return Ok(Some(format!(
                "★★★ position {} of the clipboard is `{}` and the measured order wants `{want}`. \
                 That format is on the clipboard somewhere else: {found_elsewhere}. \n\n\
                 The ORDER is the design: a pasting application takes the first format it \
                 recognises, so a raster ahead of a vector makes Word's paste a flat picture \
                 that looks correct at 100% and cannot be scaled, recoloured or ungrouped. \
                 Nothing in Word says so and the operator finds out days later.\n\n\
                 Wanted prefix: {}\nGot: {}",
                position + 1,
                if entry.1.is_empty() {
                    format!("#{}", entry.0)
                } else {
                    entry.1.clone()
                },
                EXPECTED.join(", "),
                describe(&formats)
            )));
        }
    }

    // ★ The tail is NOT asserted. Windows synthesises `CF_DIB` and `CF_BITMAP`
    // from the `CF_DIBV5` this placed, plus `CF_LOCALE` and others depending on
    // the machine — so a correct build shows more than four entries and
    // demanding exactly four would fail it on a behaviour nobody controls.
    report.note(format!(
        "the measured prefix is intact: {} (then {} synthesised or extra entr{})",
        EXPECTED.join(", "),
        formats.len() - EXPECTED.len(),
        if formats.len() - EXPECTED.len() == 1 {
            "y"
        } else {
            "ies"
        }
    ));

    // --- 5: ★ the trace and the clipboard AGREE on the count ---------------
    //
    // The cross-check that makes this more than a smoke test, and the same
    // shape `export_image_emf` uses on its byte count: two values that are
    // supposed to describe one thing are exactly the pair a refactor separates.
    if let Some(reported) = placed.get("formats") {
        let named: Vec<&str> = reported.split(',').filter(|s| !s.is_empty()).collect();
        if named.len() != EXPECTED.len() {
            return Ok(Some(format!(
                "the shell reported placing {} format(s) — `{reported}` — and the measured \
                 order is {}. The disclosure describes a different copy from the one on the \
                 clipboard.",
                named.len(),
                EXPECTED.len()
            )));
        }
        for (position, want) in EXPECTED.iter().enumerate() {
            if named[position] != *want {
                return Ok(Some(format!(
                    "the shell reported `{reported}`, whose position {} is `{}` and not \
                     `{want}`. The clipboard is right and the trace is wrong, or the trace is \
                     built from a different list than the placement — either way the one \
                     record of what happened does not describe it.",
                    position + 1,
                    named[position]
                )));
            }
        }
    }
    Ok(None)
}
