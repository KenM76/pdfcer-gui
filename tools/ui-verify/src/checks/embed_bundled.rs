//! `embedding_works_with_no_font_folder_at_all` — **pdfcer's own fourteen faces
//! answer when nothing of the operator's can, AND only when he asks.**
//!
//! # ⚠ THE 2026-09-05 REWRITE HAS NEVER BEEN RUN
//!
//! This check passed in its earlier form. It was rewritten on 2026-09-05 with
//! the operator at his machine, so `ui-verify` could not be run and **the new
//! assertions have not fired in either direction.** The original oracle is
//! unchanged and is asserted here as the *ticked* position; what is new is the
//! unticked position beside it. Treat a red from either as unverified.
//!
//! # What this is for
//!
//! `OPERATOR_REQUESTS.md` **O47** asked the operator whether pdfcer should embed
//! the standard-14 faces it ships when none of their folders holds the font a
//! document names.
//!
//! ## ★★★ The answer changed shape on 2026-09-05, and so did this check
//!
//! The 2026-08-28 answer was *"always, disclosed loudly"* — no control, the
//! bundled rung simply on. It is now **the disclosed opt-in**: a checkbox in
//! the Embed window, **off** when the window opens, with the fonts it would
//! stand in for named beside it. The deciding reason is not the letterforms,
//! which were always disclosed; it is that pdfcer's fourteen faces are
//! BSD-3-Clause and embedding one puts that licence inside a file the operator
//! then distributes. `pdfcer`'s own CLI keeps `--use-bundled-fonts` off for
//! exactly that, in those words.
//!
//! ## ★★★ WHY THIS CHECK NOW DRIVES BOTH POSITIONS
//!
//! > *A check that the switch is off by default passes on a build that ignores
//! > the switch entirely.*
//!
//! That is the shape this project keeps meeting, and it is why *"assert the box
//! is unticked"* is not a check. Both positions are driven, in one run, in
//! order:
//!
//! | position | asserted |
//! |---|---|
//! | as the window opens | `own_fonts_on=false`, and `own_fonts_offered` is **greater than zero** — the offer was made and declined |
//! | after ticking the box | the commit reaches the engine with `substituted=true` |
//!
//! The first half alone passes on a build that has lost the bundled faces
//! altogether (nothing to offer, nothing offered, box off — all true). The
//! second half alone is the check as it stood before today, and passes on a
//! build with no switch at all. Neither is worth anything without the other,
//! and `own_fonts_offered=` exists on the trace line so that the first half can
//! be made at all.
//!
//! ## ★★★ Why it is a SEPARATE check and not a parameter of the other one
//!
//! Because it asserts the opposite premise. `embedding_fonts_puts_a_program_in_
//! the_document` supplies a real font folder and would pass identically with the
//! bundled rung ripped out — the folder answers first, every time, by design.
//! Only a run with **no folder at all** can distinguish *"pdfcer ships faces and
//! will use them"* from *"pdfcer ships faces and never reaches them"*.
//!
//! ★★ That is also why it is worth the extra process launch. Two checks over one
//! feature, differing in one environment variable, is the shape that catches a
//! rung being unreachable — which is the same failure the whole resolver had on
//! the day it was written, when only the exact rung worked and every test
//! registered a name and then asked for it.
//!
//! ## ★★ The oracle is `substituted=true`, and it is the point of the row
//!
//! The operator's *"yes"* came with a condition: **disclosed loudly**. A build
//! that embedded a bundled face and reported it as an ordinary match would
//! satisfy the letter of the request and lose the half he can act on — the
//! document goes out with pdfcer's stand-in in it, and nothing on the canvas says
//! which face went in.
//!
//! `substituted=` is `EmbedPlan::substitutes_any`, computed by the engine from
//! `FontMatch::is_substitute` on every target. It is `true` here **only if** the
//! shell reported the rung honestly on the way in — a shell that claimed `Exact`
//! for a bundled face would produce a green `missing_after=0` and a false
//! `substituted=false`, so this one field checks the disclosure and the
//! correctness together.
//!
//! ⇒ Reporting a bundled donor as `Exact` would also walk it past the engine's
//! **symbolic-font guard**, which turns on exactly that predicate. The
//! disclosure and the guard are the same flag, which is why understating it is a
//! correctness defect rather than a cosmetic one.
//!
//! ## ★★★ A DECLINE IS A SKIP, NOT A FAILURE — and reading it the other way
//! cost an afternoon
//!
//! The 2026-08-28 sweep ran this check twice. On `fixtures\a1-titleblock.pdf`
//! it **passed**, with `targets=3 … substituted=true` and no font folder
//! configured — the bundled rung firing exactly as O47 asked. On
//! `D:\Dev\pdfTests\SW41177\SW41177.pdf` it **failed**, on
//! `embed-fonts-declined folders=0 detail=nothing-to-open`, and that failure
//! was carried into the handoff as *"the strongest candidate for a real
//! defect"*. It was neither. SW41177 carries six fonts and **six
//! `/FontFile2` streams** — every face it names is already embedded, so
//! *"nothing to do"* is the correct and only honest answer.
//!
//! ⇒ The old message claimed a decline meant *"the bundled rung was not
//! reached"*. **That inference is not available**, and has not been since O47.
//! `DialogsState::open_embed_fonts` has exactly ONE decline sentence left —
//! `text::embed::nothing_missing` — because O47 falsified the other branch
//! (*"pdfcer has no font folders, so it cannot embed anything"*) and it was
//! deleted. `EmbedDialog::open` answers `None` only when `plan.targets` is
//! empty **and** every `plan.blocked` row is `AlreadyEmbedded`; a missing font
//! nothing can answer for is a `NoSourceFont` row, which is `shown`, which
//! opens the window. So a decline is a statement about the FIXTURE and
//! carries no information about the resolver at all.
//!
//! ★★★ **The oracle for a broken bundled rung is the `targets=0` branch
//! below, and it always was.** If `allow_bundled` stopped reaching
//! `resolve_for_embedding`, this run would still open a window — full of
//! `NoSourceFont` rows — and that branch would fail it by name. Turning the
//! decline into a skip therefore gives up nothing: it removes a false failure
//! and leaves every true one standing.
//!
//! ★★ The general lesson, and it is the expensive half: **a check whose
//! failure message names a suspect can teach a reader the wrong suspect.**
//! This one named `Library::scan_with(folders, true)` — a call that was
//! correct, tested (`app::fonts::a_bundled_face_answers_only_when_it_is_
//! allowed_to`) and proven in the very same sweep by the sibling run. Two runs
//! of one check disagreeing is a fact about their INPUTS first; check that
//! before believing either one's diagnosis.
//!
//! ## What this does NOT establish
//!
//! **Which face was substituted, or that it looks right.** pdfcer's standard-14
//! substitutes are the engine's to choose and its tests cover the choice. This
//! establishes that the shell reaches them, at the right moment, and says so.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, frame_of, list, stable_rect,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The command, invoked through the harness seam.
const INVOKE: &str = "mode.edit,tools.embed_fonts";
/// The variable this check deliberately does **not** set.
///
/// ★ Named as a constant it never uses, so a reader grepping for the font-dir
/// seam finds this file and its reason rather than concluding it was forgotten.
/// The whole point of this run is that no folder is configured.
#[allow(dead_code)]
const DELIBERATELY_UNSET: &str = "PDFCER_DIAG_FONT_DIR";
/// The window body's region.
const BODY: &str = "embed.body";
/// The Embed button's region.
const BUTTON: &str = "embed.commit";
/// The line the window writes when it opens, carrying its plan's counts.
const OPENED: &str = "embed-fonts-opened";
/// The line the apply arm writes when the engine has embedded.
const APPLIED: &str = "embed-fonts-applied";
/// The checkbox that offers pdfcer's own faces. Off when the window opens.
const OWN_FONTS_BOX: &str = "embed.use-own-fonts";
/// The line the checkbox writes when it is ticked or unticked.
const OWN_FONTS_TOGGLED: &str = "embed-fonts-own-fonts";
/// The line the window writes when the Embed button is pressed, carrying the
/// position of the switch that chose the request.
const REQUESTED: &str = "embed-fonts-requested";
/// The line the dispatcher writes when there is nothing to open.
///
/// ★★★ Since O47 this has exactly ONE meaning — *every font in this document
/// is already embedded* — so it aims the check, it does not fail it. See the
/// module header.
const DECLINED: &str = "embed-fonts-declined";

/// See the module documentation.
pub struct EmbeddingWorksWithNoFontFolderAtAll;

impl Check for EmbeddingWorksWithNoFontFolderAtAll {
    fn name(&self) -> &'static str {
        "embedding_works_with_no_font_folder_at_all"
    }

    fn defect(&self) -> &'static str {
        "with no font folder configured, Embed fonts either cannot reach pdfcer's own fourteen \
         standard faces at all — telling an operator to go and find a font pdfcer is already \
         carrying — or reaches them WITHOUT being asked, putting a substitute face and its \
         licence into a document he sends out on a press he thought was a no-op"
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
            "input is disabled (--no-input), and this check's subject is a click on the Embed \
             button.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check needs a document naming a STANDARD-14 font it does not carry \
             — Helvetica, Times or Courier — which is what every CAD exporter writes.",
        )
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("embed-bundled.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    // ★★★ No `PDFCER_DIAG_FONT_DIR`. That absence IS the check.
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with NO font folder configured — the point of this run",
        exe.display(),
        session.pid()
    ));
    session.settle(60);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    if let Some(declined) = trace.events(DECLINED).last() {
        return Err(Error::new(format!(
            "this fixture carries every font it names, so there was nothing to embed: `{}`. \
             Aim this check at a document that NAMES a standard-14 face it does not carry — \
             `fixtures\\a1-titleblock.pdf` is one. Trace: {}.",
            declined.raw,
            session.trace_path().display()
        )));
    }
    if declared(&trace, ui_rect, BODY).is_none() {
        return Ok(Some(format!(
            "EMBED FONTS WAS INVOKED AND NO WINDOW APPEARED: no `{BODY}` region and no \
             `{DECLINED}` line. Regions beginning `embed`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "embed")),
            session.trace_path().display()
        )));
    }
    let Some(opened) = trace.events(OPENED).last() else {
        return Ok(Some(format!(
            "the window drew and published no `{OPENED}` line, so this check cannot tell a \
             correctly greyed button from a broken one. That line is what makes the difference \
             readable; see `dialogs::embed::open`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    // ── POSITION 1: the offer was made, and it was DECLINED ───────────────────
    //
    // ★★★ `targets=0` HERE IS CORRECT SINCE 2026-09-05, and reading it as the
    // defect is the mistake this rewrite exists to avoid. With no folder
    // configured and the box unticked, the operator's own fonts answer for
    // nothing, so nothing will be embedded — which is the whole of the new
    // default and is exactly what he should see before he decides.
    //
    // What replaces that oracle is `own_fonts_offered`: the number of fonts
    // pdfcer holds a copy of and is OFFERING. Zero there is the bundled rung
    // being unreachable, which is the original defect, still asserted.
    if opened.get("own_fonts_on") != Some("false") {
        return Ok(Some(format!(
            "★★★ THE WINDOW OPENED WITH PDFCER'S OWN FACES ALREADY IN THE PLAN: `{}`.\n\
             The switch must be OFF when the window opens. Embedding one of pdfcer's fourteen \
             substitutes changes what the letters look like on the screen of whoever the \
             document is sent to, and carries a BSD-3-Clause attribution condition into a file \
             the operator distributes — `pdfcer`'s own CLI keeps `--use-bundled-fonts` off for \
             precisely that, and calls it the operator's decision to make. A press of Embed with \
             nothing ticked must not make it for him. See `EmbedDialog::open`. Trace: {}.",
            opened.raw,
            session.trace_path().display()
        )));
    }
    let offered: usize = opened
        .get("own_fonts_offered")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if offered == 0 {
        return Ok(Some(format!(
            "★★★ THE WINDOW OFFERED NONE OF PDFCER'S OWN FACES: `{}`.\n\
             With no folder configured, `own_fonts_offered` counts exactly what pdfcer can \
             supply from its OWN faces for the standard-14 names this document is missing — so \
             zero is the bundled rung not firing. `Library::donor_for` answered nothing for a \
             standard-14 name, which is `allow_bundled` not reaching `resolve_for_embedding`, \
             and the operator is being told to go and find a font pdfcer is holding. \
             ★ If it is zero because the box is drawn but the trace field was never added, that \
             is `EmbedDialog::open`'s trace line, not the resolver. Trace: {}.",
            opened.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the window offered {offered} of pdfcer's own face(s) with no folder set, and \
         defaulted to declining them: `{}`",
        opened.raw
    ));

    // ── POSITION 2: tick the box ──────────────────────────────────
    let Some(checkbox) = stable_rect(&session, ui_rect, OWN_FONTS_BOX, 8)? else {
        return Ok(Some(format!(
            "the window offered {offered} of pdfcer's own faces and drew NO CHECKBOX to accept \
             them: no `{OWN_FONTS_BOX}` region. An offer the operator cannot accept is worse \
             than no offer — it names a remedy and withholds it. Regions beginning `embed`: {}. \
             Trace: {}.",
            list(&declared_names(&trace, ui_rect, "embed")),
            session.trace_path().display()
        )));
    };
    let trace = session.trace()?;
    let frame = frame_of(&session, &trace, ui_rect, OWN_FONTS_BOX)?;
    driver.click_at(frame.declared_center(checkbox))?;
    session.settle(60);
    let trace = session.trace()?;
    if trace
        .events(OWN_FONTS_TOGGLED)
        .filter(|line| line.get("on") == Some("true"))
        .last()
        .is_none()
    {
        return Ok(Some(format!(
            "the checkbox was clicked and did not change: no `{OWN_FONTS_TOGGLED} on=true` line. \
             Its rect was declared and aimed at, so this is the click not reaching it. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ the operator ticked the box — pdfcer's own faces are now in the plan");

    let Some(button) = stable_rect(&session, ui_rect, BUTTON, 8)? else {
        return Ok(Some(format!(
            "the window drew its body and declared no `{BUTTON}` region. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let trace = session.trace()?;
    let frame = frame_of(&session, &trace, ui_rect, BUTTON)?;
    driver.click_at(frame.declared_center(button))?;
    session.settle(60);

    let trace = session.trace()?;
    match trace.events(REQUESTED).last() {
        Some(line) if line.get("own_fonts_on") == Some("true") => {
            report.note(format!(
                "★ the request sent was the one the box chose: `{}`",
                line.raw
            ));
        }
        Some(line) => {
            return Ok(Some(format!(
                "★★★ THE BOX WAS TICKED AND THE OTHER REQUEST WAS SENT: `{}`.\n\
                 `EmbedDialog::active` picks the plan the window is showing AND the request the \
                 button commits, so that the two cannot be chosen by different code. \
                 `own_fonts_on=false` here means the switch changes what is drawn and not what \
                 is done — a window showing one thing and doing another, which is the exact \
                 property `embed_preview` was designed to give this dialog for free. Trace: {}.",
                line.raw,
                session.trace_path().display()
            )));
        }
        None => {
            return Ok(Some(format!(
                "the Embed button was clicked and the window recorded no request: no \
                 `{REQUESTED}` line. Trace: {}.",
                session.trace_path().display()
            )));
        }
    }
    let Some(applied) = trace.events(APPLIED).last() else {
        return Ok(Some(format!(
            "the Embed button was clicked and nothing reached the document: no `{APPLIED}` \
             line. The window had targets, so this is the action or its apply arm rather than \
             the resolver. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the engine embedded: `{}`", applied.raw));

    // --- the oracle ---------------------------------------------------------
    let embedded: usize = applied
        .get("embedded")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if embedded == 0 {
        return Ok(Some(format!(
            "the embed ran and embedded nothing: `{}`. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    if applied.get("substituted") != Some("true") {
        return Ok(Some(format!(
            "★★★ {embedded} FONT(S) WERE EMBEDDED FROM PDFCER'S OWN FACES AND REPORTED AS NOT \
             SUBSTITUTED: `{}`.\n\
             With no folder configured every donor is a bundled face, so `substituted=false` \
             means the shell told the engine `FontMatch::Exact` for one. That is not a wording \
             defect. `is_substitute` is the predicate the engine's SYMBOLIC-FONT GUARD turns \
             on, so understating the rung disables that guard from the outside — and it loses \
             the disclosure the operator's *\"yes\"* to O47 was conditional on. See `rung()` and \
             the `Match::Bundled` arm in `dialogs::embed`. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ {embedded} font(s) embedded from pdfcer's own faces, with no folder configured, and \
         every one disclosed as a substitute — still missing afterwards: {}",
        applied.get("missing_after").unwrap_or("?")
    ));
    Ok(None)
}
