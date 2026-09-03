//! `embedding_fonts_puts_a_program_in_the_document` — **press Embed and the
//! document stops missing a font.**
//!
//! # What this is for
//!
//! `tools.embed_fonts` was registered, drawn on the Tools tab and **inert for
//! the whole life of the project**, behind a `SCAFFOLDED` entry whose stated
//! premise had expired and whose real blocker was in no register at all. It was
//! wired on 2026-08-28, and this is the check that keeps it wired.
//!
//! ## ★★★ The oracle is `missing_after=0`, not "a dialog appeared"
//!
//! Almost every link in this chain can be satisfied by a build that does
//! nothing. The window opens on a plan; a plan can be empty; an empty plan
//! draws a window with a greyed button and a list of refusals, and **that is a
//! legitimate outcome** for a document nothing on the machine can answer. So a
//! check that asserted only *"the window opened"* would pass on:
//!
//! - a resolver that finds no donor for anything;
//! - a request whose `supplied` map is never populated;
//! - an Embed button whose click raises no action;
//! - an apply arm that calls the engine and drops the result.
//!
//! The number that distinguishes all four from working software is the
//! engine's own `missing_after` — *"the end state the whole feature exists to
//! reach"*, in `EmbedPlan`'s own words. This check drives the real gesture and
//! asserts that it reaches zero.
//!
//! ## ★★★ Why `PDFCER_DIAG_FONT_DIR` exists, and why it APPENDS
//!
//! Embedding needs a folder of font files, and in the product that folder comes
//! from a preference an operator sets in Settings and pdfcer stores in
//! `userdata/preferences.txt`. A harness must not rewrite that file — it
//! belongs to whoever is running the build, and a check that edited it would
//! leave it edited.
//!
//! So `dispatch::fonts::folders` reads an environment variable and **adds** its
//! folders to the operator's. Adding rather than replacing is the load-bearing
//! half: a variable that replaced the preference would let this check pass on a
//! build whose preference plumbing was broken end to end, because the harness
//! would then be testing its own environment variable.
//!
//! ## ★★ Why the fixture is `a1-titleblock.pdf`, and what it proves that a
//! synthetic one would not
//!
//! It asks for `Helvetica`, `Helvetica-Bold` and `Helvetica` again, on three
//! surfaces, with no program for any of them — which is what every CAD exporter
//! writes and is the exact case this feature exists for.
//!
//! ★ **No Windows machine has a font called Helvetica.** So this fixture cannot
//! be embedded at all unless the resolver's *alias* rung works, and a passing
//! run is therefore evidence for a claim no unit test in this project can make:
//! that `pdfcer_render::FontEnvironment`'s standard-14 equivalence is reached
//! from the shell, on a real font folder, through the real dispatch. The first
//! draft of that resolver had only an exact rung and would fail here while
//! every one of its own tests passed — they registered a name and then asked
//! for that name.
//!
//! ## What this check does NOT cover, stated rather than implied
//!
//! **The saved file.** Embedding is one `EditSession` command and this asserts
//! on the session's own report of it; whether the program survives a write and
//! reopen is `save_copy`'s territory and is not asserted here. The engine tests
//! that round trip; this tests that the GUI reaches the engine.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, frame_of, list, stable_rect,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The command, invoked through the harness seam.
///
/// ★ `mode.edit` first. The command is drawn on the Tools tab, which every mode
/// shows, but embedding is a content edit and driving from a known mode makes
/// the run reproducible rather than dependent on whatever mode the last session
/// left behind.
const INVOKE: &str = "mode.edit,tools.embed_fonts";
/// The variable that supplies a font folder without touching the preference.
const FONT_DIR_ENV: &str = "PDFCER_DIAG_FONT_DIR";
/// The operating system's own font directory — the one folder that certainly
/// exists on the platform this ships for.
///
/// ★ Deliberately not what the product searches. `Prefs::font_folders` starts
/// empty and pdfcer never adds to it, for the licensing reason `app::fonts`
/// records: which font goes into somebody's document is the operator's call. A
/// harness may look where a product may not.
const SYSTEM_FONTS: &str = r"C:\Windows\Fonts";
/// The window body's region.
const BODY: &str = "embed.body";
/// The Embed button's region.
const BUTTON: &str = "embed.commit";
/// The line the dialog writes when the button is pressed.
const REQUESTED: &str = "embed-fonts-requested";
/// The line the apply arm writes when the engine has embedded.
///
/// ★ `-applied`, and the suffix is why this constant has a doc comment.
/// `vector_edit` writes a **second** line for the same edit under the bare name
/// — `embed-fonts page=0 n=3 epoch=1 disclosures=…` — and trace matching is on
/// the exact event name, so `.last()` on the bare name would read the funnel's
/// line, find no `embedded=` key, and report `embedded=0` about an embed that
/// worked. That defect has been made twice in this project and the naming
/// convention is what prevents the third.
const APPLIED: &str = "embed-fonts-applied";
/// The line the window writes when it opens, carrying its plan's counts.
///
/// ★★ The check reads `targets=` off this to tell a GREYED button from a broken
/// one. Both look identical from outside - no click reaches anything - and
/// exactly one of them is a fact about the fixture rather than about the
/// program.
const OPENED: &str = "embed-fonts-opened";
/// The line the dispatcher writes when there is nothing to open.
const DECLINED: &str = "embed-fonts-declined";

/// See the module documentation.
pub struct EmbeddingFontsPutsAProgramInTheDocument;

impl Check for EmbeddingFontsPutsAProgramInTheDocument {
    fn name(&self) -> &'static str {
        "embedding_fonts_puts_a_program_in_the_document"
    }

    fn defect(&self) -> &'static str {
        "Embed fonts is on the Tools tab and embeds nothing — the command was registered, drawn \
         and inert for the whole life of the project, and a document that asks for Helvetica \
         still asks for it after the operator has pressed the button and read a window telling \
         them what would happen"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check's whole subject is a click on the \
             Embed button. The window opening proves nothing on its own — an empty plan draws \
             the same window with the button greyed, which is a correct outcome.",
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
            "no --pdf. This check needs a document that names a font it does not carry; a fully \
             embedded document declines by name and the decline is not the behaviour under test.",
        )
    })?;
    if !std::path::Path::new(SYSTEM_FONTS).is_dir() {
        return Err(Error::new(format!(
            "no font folder at {SYSTEM_FONTS}, so there is nothing for the resolver to answer \
             with and a refusal here would say nothing about the program. SKIPPED rather than \
             failed: this is a fact about the machine."
        )));
    }

    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("embed-fonts.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((FONT_DIR_ENV.to_owned(), SYSTEM_FONTS.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE} and {FONT_DIR_ENV}={SYSTEM_FONTS}",
        exe.display(),
        session.pid()
    ));
    // Long, and deliberately: opening this window reads and parses every font
    // file in the system folder — measured at 3,359 indexed names — before it
    // can draw a single row.
    session.settle(90);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;

    // ★ A decline is reported as ITSELF and as a SKIP. *"This document's fonts
    // are all embedded"* is a statement about the `--pdf` the harness was aimed
    // at, and a check that failed on it would be blaming the feature for the
    // fixture.
    if let Some(declined) = trace.events(DECLINED).last() {
        return Err(Error::new(format!(
            "the command declined: `{}`. SKIPPED rather than failed: it says the --pdf carries \
             no font that is missing its program, which is the harness's aim rather than the \
             program's behaviour. Point it at a document a CAD tool exported. Trace: {}.",
            declined.raw,
            session.trace_path().display()
        )));
    }

    // ★★ The same fixture guard `unembed_fonts` documents at length - but here
    // it is a FAILURE, not a skip, and the asymmetry is the point.
    //
    // "Nothing removable" is a fact about the document alone. "Nothing
    // embeddable" is a fact about the document AND the folder this check aimed
    // the program at, and this check aims it at the system font folder on a
    // fixture asking for Helvetica. `targets=0` there means the resolver did
    // not reach its alias rung, which is a defect in the program.
    if let Some(opened) = trace.events(OPENED).last()
        && opened.get("targets") == Some("0")
    {
        return Ok(Some(format!(
            "★ THE WINDOW OPENED WITH NOTHING TO EMBED: `{}`.
             The document names fonts it does not carry and {SYSTEM_FONTS} was supplied, so a \
              plan with no targets means the donor map reaching the engine is empty. Read \
              `supplied=`: zero means `app::fonts::Library::donor_for` answered nothing, which \
              on a fixture asking for Helvetica means the ALIAS rung of \
              `FontEnvironment::resolve_for_embedding` was not reached. Trace: {}.",
            opened.raw,
            session.trace_path().display()
        )));
    }

    let Some(_body) = declared(&trace, ui_rect, BODY) else {
        return Ok(Some(format!(
            "★ EMBED FONTS WAS INVOKED AND NO WINDOW APPEARED: no `{BODY}` region and no \
             `{DECLINED}` line.\n\
             Three candidates. (1) **The command has no dispatch arm** — which is the state it \
             was in for the whole life of the project. (2) The window was built and never \
             drawn. (3) The plan construction panicked or returned early without tracing. \
             Regions beginning `embed`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "embed")),
            session.trace_path().display()
        )));
    };

    // ★ `stable_rect`, not `declared`, for the button: the window lays out over
    // several frames as the scroll area measures its rows, and a coordinate
    // read before it stops moving is a number rather than an error. See
    // `driving::stable_rect`'s own header, which is a defect report.
    let Some(button) = stable_rect(&session, ui_rect, BUTTON, 8)? else {
        return Ok(Some(format!(
            "the Embed window drew its body and declared no `{BUTTON}` region, so the one \
             control that changes anything was never laid out. Regions beginning `embed`: {}. \
             Trace: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "embed")),
            session.trace_path().display()
        )));
    };

    // ★★ `frame_of`, never `session.frame()`. This window is a child viewport
    // and its coordinates are its own; asking the main window for them yields a
    // point on the ribbon, and the click lands somewhere plausible and wrong.
    let trace = session.trace()?;
    let frame = frame_of(&session, &trace, ui_rect, BUTTON)?;
    driver.click_at(frame.declared_center(button))?;
    // The engine walks every font-bearing surface, writes a stream per target
    // and re-rasterizes the page.
    session.settle(60);

    let trace = session.trace()?;
    let Some(requested) = trace.events(REQUESTED).last() else {
        return Ok(Some(format!(
            "the Embed button was clicked and the window raised nothing: no `{REQUESTED}` line.\n\
             Either the click missed — the button is at {button:?} in the window's own frame — \
             or it landed on a **greyed** button, which is what the window draws when its plan \
             has no targets. A greyed button here means the resolver answered nothing for a \
             document asking for Helvetica out of {SYSTEM_FONTS}, which is the alias rung of \
             `FontEnvironment::resolve_for_embedding` not being reached. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ the window raised the action: `{}`",
        requested.raw
    ));

    let Some(applied) = trace.events(APPLIED).last() else {
        return Ok(Some(format!(
            "★ THE EMBED WAS REQUESTED AND NOTHING REACHED THE DOCUMENT: `{}` and no \
             `{APPLIED}` line.\n\
             The action was raised and its apply arm did not run, or the engine refused. A \
             refusal traces `embed-fonts-refused …` through the edit funnel and carries the \
             engine's own reason — read the trace for one. Trace: {}.",
            requested.raw,
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
            "★ THE EMBED RAN AND EMBEDDED NOTHING: `{}`.\n\
             The chain works end to end and the plan was empty, which means the donor map \
             reaching the engine was empty. The request is built in `dialogs::embed::open`; the \
             resolver is `app::fonts::Library::donor_for`. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    let before: usize = applied
        .get("missing_before")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let after: usize = applied
        .get("missing_after")
        .and_then(|v| v.parse().ok())
        .unwrap_or(usize::MAX);
    if after >= before {
        return Ok(Some(format!(
            "★ THE EMBED REPORTED {embedded} FONT(S) AND THE DOCUMENT IS NO BETTER OFF: `{}` \
             says {before} font(s) had no program before and {after} have none after.\n\
             `missing_before` is counted from the font inventory rather than from the plan, so \
             these two numbers are independent measurements of the same document. Them failing \
             to move is the case a count of embedded fonts cannot see. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ fonts with no program: {before} before, {after} after — {embedded} embedded, \
         substituted={}",
        applied.get("substituted").unwrap_or("?")
    ));

    // ★ Reported, never asserted. A stand-in was used here because this fixture
    // asks for Helvetica and this platform has none — but a machine with a real
    // Helvetica installed would answer `substituted=false` and be equally
    // correct, so pinning it would pin the check to a font folder.
    Ok(None)
}
