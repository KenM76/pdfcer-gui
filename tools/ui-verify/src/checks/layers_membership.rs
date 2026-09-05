//! `selecting_an_object_names_its_layer` — **clicking a page object says which
//! optional-content group it is on, and clicking one on no layer says that
//! instead.**
//!
//! # ⚠ THIS CHECK HAS NOT BEEN RUN
//!
//! Written 2026-09-05 and **not executed**. `ui-verify` drives the real cursor
//! and keyboard and takes the whole desktop; the operator may be at his
//! machine, and the session that wrote this was instructed not to launch the
//! GUI. It is registered, it compiles, and it has never seen a running binary.
//!
//! **Do not read a green suite as evidence for this row until it has been
//! driven once.** Say so in whatever report cites it. A check nobody has run
//! is a specification, not a verification, and this project's founding rule is
//! that the two are not the same thing.
//!
//! # The claim
//!
//! `OPERATOR_REQUESTS.md` O126, in his words: *"selecting an object highlights
//! that layer"*.
//!
//! Until `pdfcer-core` v0.38.0 only an **annotation** could be answered for —
//! `vector::decompose` counted `/OC` sections and discarded the group id.
//! `Pass 250.0` put `oc: Option<ObjId>` on `PathObject`, `TextObject` and
//! `ImageObject`, and this check is the driven half of consuming it.
//!
//! # ★★★ Why the fixture is pinned, and it is the vacuous-pass argument
//!
//! This check **ignores `--pdf`** and says so in its notes when one was
//! supplied. Its subject is a *relation between two things*, and a document
//! cannot exercise it unless it carries both:
//!
//! | the fixture must have | or the check |
//! |---|---|
//! | **at least two** optional-content groups | passes under a build that highlights a constant |
//! | **at least one object on no layer at all** | passes under a build that highlights whatever the first layer is |
//! | the two reachable at **different points** | cannot separate "the answer follows the selection" from "there is one answer" |
//!
//! ★★ The middle row is the one that would have been missed. A fixture whose
//! every object is on the same layer makes *"the highlight follows the
//! selection"* true of a build that ignores the selection entirely, and the
//! check would go green while measuring nothing. That is this project's
//! standing failure shape — *ask what the check SAMPLED before asking what is
//! broken* — and the answer here is: two points, two different answers, one of
//! which is an established absence.
//!
//! `layers/painted-layers.pdf` in the engine's read-only synthetic corpus
//! carries exactly that, in fourteen objects of hand-written syntax:
//!
//! ```text
//! /OC /L1 BDC  0 0 0 rg 60 60 120 120 re f  EMC        <- "Visible Box"  (obj 4)
//! /OC /L2 BDC  0 0 0 rg 400 60 120 120 re f            <- "Hidden Box"   (obj 5, /OFF)
//!   /OC /L4 BDC 0 0 0 rg 400 220 120 120 re f EMC      <- "Nested Inner" (obj 7)
//! EMC
//! /OC /L3 BDC  0 0 300 792 re W n EMC                  <- "Clip Only"    (obj 6, /OFF)
//! 0.5 g 0 600 612 60 re f                              <- ★ ON NO LAYER
//! ```
//!
//! # The two clicks, and why those two
//!
//! | # | point (PDF user space) | what is there | the assertion |
//! |---|---|---|---|
//! | 1 | `120, 120` | the *Visible Box* square, 60→180 in both axes | the answer is `group` and names **Visible Box**, and **exactly one** panel row carries the highlight |
//! | 2 | `150, 630` | the grey bar, drawn outside every `BDC` | the answer is `no-layer`, and **no** row carries the highlight |
//!
//! ★ Neither point is on a layer the document turns **off**. `/OFF` names L2
//! and L3, and both are avoided deliberately: an object the renderer does not
//! draw is still in the object model, so a click there selects something
//! invisible and a failure report about it would need a paragraph before it
//! could be read. The subject here is the membership relation, not the
//! visibility one.
//!
//! # ★★ The two oracles, and why one would not do
//!
//! | oracle | what it proves | what it cannot see |
//! |---|---|---|
//! | `layer-membership … answer= name=` (status bar) | the shell *computed* the right answer, with no panel open | whether anything is drawn |
//! | `layer-row … name= highlighted=` (Layers panel) | **which row is lit, and that only one is** | nothing, if the panel is closed |
//!
//! The first is the **canvas route** — the answer reachable by clicking, with
//! no panel open, because *the canvas is the primary surface, never a panel*.
//! The second is the operator's literal word, *"highlights"*. A check reading
//! only the first would pass against a build that computed the answer and drew
//! nothing; one reading only the second would pass against a build whose only
//! route to the answer is a panel the operator has to know to open.
//!
//! # ★★★ Rule 4 is asserted, not assumed
//!
//! Both oracles are **off-canvas** by construction: a status-bar line and a
//! panel row. There is deliberately no assertion about the canvas here,
//! because there is deliberately nothing on the canvas to assert about — no
//! badge, tint, dashed outline or provisional layer is drawn over the selected
//! content to express its membership. If a future change adds one, this check
//! will not catch it, and that is worth saying out loud rather than implying
//! coverage the file does not have.
//!
//! # What this check does NOT cover, stated rather than implied
//!
//! * **The form-leaf path.** `for_leaf`'s repair of the engine's D1 partial
//!   (a leaf inheriting the `/OC` its `Do` was painted under) is held by unit
//!   tests only. It needs a fixture with a form XObject invoked from inside a
//!   `BDC /OC` section, and no such file exists in either corpus.
//! * **The multi-object fold.** `Membership::join` is unit-tested exhaustively
//!   for commutativity and associativity; no driven marquee exercises it.
//! * **The search-narrowed row.** The sentence shown when the highlighted
//!   layer has been filtered out of the list needs typing into the search
//!   field, which `layers_search`'s own header explains this harness has no
//!   seam for.

use std::collections::BTreeMap;

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint, WindowFrame};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The fixture, under the engine's read-only synthetic corpus. See the module
/// docs for why it is pinned and what `--pdf` cannot substitute for.
const FIXTURE: &str = "layers/painted-layers.pdf";

/// The fixture's page, in PDF points. Stated rather than read: the whole file
/// is hand-written syntax, and a page size that changed would change what
/// every constant below means.
const PAGE: PageGeometry = PageGeometry {
    width_pt: 612.0,
    height_pt: 792.0,
};

/// The centre of the square painted inside `/OC /L1` — 60→180 in both axes.
const ON_A_LAYERED_OBJECT: (f64, f64) = (120.0, 120.0);

/// The layer that square is on, exactly as the document's `/Name` spells it.
const EXPECTED_LAYER: &str = "Visible Box";

/// A point on the grey bar (`0 600 612 60 re f`), which is painted after every
/// `EMC` and is therefore on **no** optional-content group.
///
/// ★ `x = 150` rather than the bar's centre at 306: the fixture leaves a
/// `0 0 300 792 re W n` clip in force with no `Q` to restore it, so the right
/// two thirds of the bar may or may not be painted depending on how a renderer
/// treats a clip inside a switched-off group. The object's page bbox spans the
/// full width either way, but aiming where the two readings agree keeps a
/// failure here about the layer relation rather than about clipping.
const ON_AN_UNLAYERED_OBJECT: (f64, f64) = (150.0, 630.0);

/// The commands that put the shell in a state where this can be measured, one
/// per frame in this order.
///
/// ★ `mode.edit` first and it is not optional: Read mode refuses a canvas
/// click on content by design (`DEFECTS.md` D6), and a check that skipped this
/// step once reported the mode gate as a selection defect.
const INVOKE: &str = "mode.edit,view.panel_layers";

/// The status bar's line — the canvas route's oracle.
const MEMBERSHIP_EVENT: &str = "layer-membership";

/// The Layers panel's per-row line — the highlight's oracle.
const ROW_EVENT: &str = "layer-row";

/// The canvas's own selection line, used to establish that the click landed
/// before anything is concluded from the absence of an answer.
const SELECTION_EVENT: &str = "canvas-selection";

/// See the module documentation.
pub struct SelectingAnObjectNamesItsLayer;

impl Check for SelectingAnObjectNamesItsLayer {
    fn name(&self) -> &'static str {
        "selecting_an_object_names_its_layer"
    }

    fn defect(&self) -> &'static str {
        "selecting a page object says nothing about which layer it is on — the operator asked \
         for 'selecting an object highlights that layer', and a shell that answers only for \
         annotations leaves every path, every text run and every image on a layered drawing \
         unanswerable, silently and identically to a mark that is on no layer at all"
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

/// Run the sequence.
///
/// The three-way return is the SKIP/FAIL/PASS rule made structural: `Err` is a
/// precondition that was absent (SKIP), `Ok(Some(_))` is an assertion that did
/// not hold (FAIL), `Ok(None)` is a pass. An author who reaches for `?` gets a
/// SKIP, which is the safe default — the unsafe default would be a pass.
#[allow(clippy::too_many_lines)] // one linear sequence; splitting it would hide the order
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;

    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check cannot be performed without \
             clicking. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    let fixture = engine_fixture(FIXTURE).ok_or_else(|| {
        Error::new(format!(
            "the engine's layers fixture is not at D:/Dev/pdfcer/fixtures/synthetic/{FIXTURE}. \
             This check pins it and ignores --pdf: it needs two named layers AND an object on \
             no layer at all, at two different points. On a document without all three, a pass \
             would be a pass about nothing — see the module docs."
        ))
    })?;
    if let Some(supplied) = ctx.pdf.as_ref() {
        // ★ Said out loud. A sweep that silently ignored a flag is
        // indistinguishable from one that honoured it, and this project has
        // twice spent a session on a check that had thrown its fixture away
        // without saying so.
        report.note(format!(
            "· --pdf {} was supplied and is IGNORED; this check pins {}",
            supplied.display(),
            FIXTURE
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its mode segments are and this check has no fallback route out of Read mode.",
            ctx.profile.name
        ))
    })?;

    // --- launch ------------------------------------------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("layers-membership.trace.txt"));
    spec.pdf = Some(fixture.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    // The shell's own channel too: `click_mode_segment` reads `egui-shell`'s
    // trace, and without this the fallback mode click looks like a miss.
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} on {}",
        exe.display(),
        session.pid(),
        fixture.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process and \
             this check has no oracle. Captured stderr is at {}.",
            vocab.start_event,
            session.trace_path().display()
        )));
    }

    // -----------------------------------------------------------------
    // A. THE PRECONDITIONS, established rather than assumed.
    //
    // Both of the things this check reads are drawn conditionally, and a
    // check whose "nothing happened" branch is a pass is a check that has
    // stopped running and will not say so.
    // -----------------------------------------------------------------
    let rows = row_states(&trace);
    if rows.len() < 2 {
        return Err(Error::new(format!(
            "the Layers panel published {} row(s) after `{INVOKE}`, and this check needs at \
             least two — a document with one layer cannot distinguish 'the highlight follows \
             the selection' from 'there is one answer'. Either the panel is not on screen, or \
             the fixture at {} is not the one this check was written against. ERROR, not a \
             pass, deliberately.",
            rows.len(),
            fixture.display()
        )));
    }
    report.note(format!(
        "· the Layers panel drew {} rows: {}",
        rows.len(),
        rows.keys().cloned().collect::<Vec<_>>().join(", ")
    ));
    if !rows.contains_key(EXPECTED_LAYER) {
        return Err(Error::new(format!(
            "the panel has no row called {EXPECTED_LAYER:?}, so the fixture is not the one this \
             check's coordinates were derived from and every assertion below would be about a \
             different document."
        )));
    }

    // ★ Nothing is selected yet, so nothing may be highlighted. This is the
    // baseline that makes the later assertions mean something: without it, a
    // build that highlights the first row unconditionally would satisfy step B
    // and the check would report a constant as a relation.
    if let Some(lit) = highlighted_row(&rows) {
        return Ok(Some(format!(
            "before any click, with nothing selected, the row {lit:?} is already highlighted. \
             The highlight is supposed to be a statement about the selection, so a lit row on \
             an empty selection is a constant wearing a relation's clothes — and it would make \
             every assertion below pass without measuring anything. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note("★ baseline: nothing selected, no row highlighted");

    // --- make sure we are in Edit, whatever the invoke queue managed --------
    let driver = Driver::new(session.window());
    if let Err(why) = click_mode_segment(&session, &driver, ui_rect, "edit") {
        // Not fatal: `PDFCER_DIAG_INVOKE` already asked for `mode.edit`, and
        // this is the belt to that braces. Worth a note rather than a skip,
        // because a failure here with a working invoke queue is a harness
        // fact and not a product one.
        report.note(format!(
            "· the Edit mode segment could not be clicked ({why}); relying on \
             PDFCER_DIAG_INVOKE's `mode.edit`"
        ));
    }
    session.settle(12);

    // --- aim, in document space -------------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, PAGE, 0)?;
    report.note(format!(
        "canvas rect {:?} at zoom {:.3}",
        mapping.image_rect, mapping.zoom
    ));
    let frame = session.frame()?;

    // -----------------------------------------------------------------
    // B. A CLICK ON A LAYERED OBJECT NAMES ITS LAYER.
    // -----------------------------------------------------------------
    let at = aim(&mapping, &frame, ON_A_LAYERED_OBJECT)?;
    report.note(format!(
        "clicking the {EXPECTED_LAYER:?} square (page 0, {:.1}, {:.1}) -> screen ({}, {})",
        ON_A_LAYERED_OBJECT.0,
        ON_A_LAYERED_OBJECT.1,
        at.x(),
        at.y()
    ));
    driver.click_at(at)?;
    session.settle(16);
    let after = session.trace()?;

    // The click must have landed before anything is concluded from the
    // answer. "selected nothing" and "the answer is wrong" are different
    // diagnoses and this is what separates them.
    let selected = after
        .last(SELECTION_EVENT)
        .and_then(|l| l.get("first"))
        .unwrap_or("none")
        .to_owned();
    report.note(format!(
        "· after the click: {SELECTION_EVENT} first={selected}"
    ));
    if selected == "none" {
        return Err(Error::new(format!(
            "the click on the {EXPECTED_LAYER:?} square selected nothing. The square is \
             120x120 pt at (60,60)-(180,180) on a 612x792 page, so this is not a near-miss: \
             either the click did not reach the canvas, or the shell is still in Read mode \
             where a content click is refused by design. Read the `canvas rect` note above. \
             Reported as SKIPPED rather than failed because the subject of this check — which \
             layer a selection is on — was never reached. Trace: {}",
            session.trace_path().display()
        )));
    }

    let Some(line) = after.last(MEMBERSHIP_EVENT) else {
        return Ok(Some(format!(
            "an object is selected ({selected}) and the shell published no `{MEMBERSHIP_EVENT}` \
             line at all. That line is written whenever a selection exists on a document that \
             declares optional content, so its absence is the defect this check is named for: \
             selecting an object says nothing about which layer it is on. Trace: {}",
            session.trace_path().display()
        )));
    };
    let answer = line.get("answer").unwrap_or("").to_owned();
    let named = line.get("name").unwrap_or("").to_owned();
    report.note(format!(
        "· {MEMBERSHIP_EVENT} answer={answer} name={named:?}"
    ));
    if answer != "group" || named != EXPECTED_LAYER {
        return Ok(Some(format!(
            "the click landed on the square painted inside `/OC /L1`, whose group is named \
             {EXPECTED_LAYER:?}, and the shell answered `answer={answer} name={named:?}`. \
             `answer=no-layer` here would be the wrong-positive this feature's whole \
             three-valued design exists to forbid; a different name would be a wrong \
             highlight, which the operator's own bar rates worse than no highlight at all. \
             Trace: {}",
            session.trace_path().display()
        )));
    }

    // ★★ And the panel actually lit that row — the operator's literal word was
    // "highlights". Exactly one, because a build that lit every row would
    // satisfy "the right row is lit" and say nothing.
    let rows = row_states(&after);
    let lit: Vec<&String> = rows
        .iter()
        .filter(|(_, on)| **on)
        .map(|(name, _)| name)
        .collect();
    report.note(format!("· rows highlighted: {lit:?}"));
    if lit.len() != 1 || lit[0] != EXPECTED_LAYER {
        return Ok(Some(format!(
            "the shell computed the right answer ({EXPECTED_LAYER:?}) and the Layers panel \
             highlighted {lit:?}. Exactly one row must carry the plate and it must be that \
             one: no row is the feature not arriving on screen, and more than one is a plate \
             that has stopped being a statement about the selection. Trace: {}",
            session.trace_path().display()
        )));
    }

    // -----------------------------------------------------------------
    // C. …AND A CLICK ON AN OBJECT THAT IS ON NO LAYER SAYS SO INSTEAD.
    //
    // ★★★ This is the half that cannot be faked. Everything in B passes
    // against a build that ignores the selection and always answers with the
    // first layer; nothing here does.
    // -----------------------------------------------------------------
    let at = aim(&mapping, &frame, ON_AN_UNLAYERED_OBJECT)?;
    report.note(format!(
        "clicking the unlayered grey bar (page 0, {:.1}, {:.1}) -> screen ({}, {})",
        ON_AN_UNLAYERED_OBJECT.0,
        ON_AN_UNLAYERED_OBJECT.1,
        at.x(),
        at.y()
    ));
    driver.click_at(at)?;
    session.settle(16);
    let after = session.trace()?;

    let selected = after
        .last(SELECTION_EVENT)
        .and_then(|l| l.get("first"))
        .unwrap_or("none")
        .to_owned();
    report.note(format!(
        "· after the second click: {SELECTION_EVENT} first={selected}"
    ));
    if selected == "none" {
        return Err(Error::new(format!(
            "the click on the grey bar selected nothing. It is a 612x60 pt filled rectangle at \
             y 600..660 and the aim point is at its vertical centre, so this is a harness \
             miss rather than a product answer — and treating it as `no-layer` would be a \
             pass earned by a click that never happened. Trace: {}",
            session.trace_path().display()
        )));
    }

    let Some(line) = after.last(MEMBERSHIP_EVENT) else {
        return Ok(Some(format!(
            "the second click selected {selected} and produced no `{MEMBERSHIP_EVENT}` line. \
             The first click produced one, so this is the answer going silent on the object \
             that is on no layer — which is exactly the state the operator cannot distinguish \
             from a broken feature. Trace: {}",
            session.trace_path().display()
        )));
    };
    let answer = line.get("answer").unwrap_or("").to_owned();
    report.note(format!("· {MEMBERSHIP_EVENT} answer={answer}"));
    if answer != "no-layer" {
        return Ok(Some(format!(
            "the grey bar is painted after every `EMC` in this fixture's content stream, so it \
             is on no optional-content group, and the shell answered `answer={answer}`. \
             `answer=group` here means the answer does not follow the selection — the shape a \
             build that highlights a constant produces, and the reason this check uses two \
             points rather than one. `answer=unknown` means the shell would not commit, which \
             is honest but wrong here: `pdfcer-core` can answer for this object. Trace: {}",
            session.trace_path().display()
        )));
    }

    let rows = row_states(&after);
    if let Some(lit) = highlighted_row(&rows) {
        return Ok(Some(format!(
            "the selection is on no layer and the Layers panel still highlights {lit:?}. A \
             stale plate is worse than no plate: it asserts a membership the shell has just \
             established does not exist. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note("★ nothing highlighted for an object on no layer — the plate is not stale");

    Ok(None)
}

/// The final highlight state of every layer row, keyed by name.
///
/// # ★ Why the LAST line per name rather than a count
///
/// `panels::layers` traces one `layer-row` per drawn row **per frame**, so a
/// run of a few seconds leaves hundreds of lines and any count is a count of
/// repaints. What this check asks is *"what is the state now?"*, and the
/// answer is the most recent line for each row — the same reasoning
/// `form_selection::last_first` gives for reading the last selection line
/// instead of counting new ones.
fn row_states(trace: &crate::trace::Trace) -> BTreeMap<String, bool> {
    let mut out = BTreeMap::new();
    for line in trace.events(ROW_EVENT) {
        let Some(name) = line.get("name") else {
            continue;
        };
        // ★ `unwrap_or(false)` is wrong here and `continue` is right: a build
        // that never publishes `highlighted=` would read as "no row is
        // highlighted", which is this check's pass condition in section C. A
        // missing field must not be able to manufacture an assertion.
        let Some(on) = line.get("highlighted") else {
            continue;
        };
        out.insert(name.to_owned(), on == "true");
    }
    out
}

/// The one highlighted row, if any — for the assertions that require none.
fn highlighted_row(rows: &BTreeMap<String, bool>) -> Option<String> {
    rows.iter()
        .find(|(_, on)| **on)
        .map(|(name, _)| name.clone())
}

/// Resolve a fixture under the engine repository's synthetic corpus.
///
/// ★ The path is derived, not configured. `D:\Dev\pdfcer` is READ-ONLY to this
/// project and its corpus is the only place this shape exists, so the check
/// reads from it and writes nowhere near it. `None` rather than a panic turns
/// a missing corpus into a SKIP with a reason instead of a crash mid-suite.
fn engine_fixture(rel: &str) -> Option<std::path::PathBuf> {
    let path = std::path::Path::new("D:/Dev/pdfcer/fixtures/synthetic").join(rel);
    path.is_file().then_some(path)
}

/// A page-space point, through the mapping and the window frame, to a desktop
/// point.
///
/// Its own function so the two call sites cannot hop differently — the class of
/// error `crate::coords` exists to prevent, and the one a literal screen
/// coordinate always is.
fn aim(mapping: &CanvasMapping, frame: &WindowFrame, point: (f64, f64)) -> Result<ScreenPoint> {
    let window = mapping.doc_to_window(DocPoint {
        page: 0,
        x: point.0,
        y: point.1,
    })?;
    Ok(frame.to_screen(window))
}
