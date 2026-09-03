//! `annot_delete_gate` — **a certified document does not offer a Delete for its
//! comments, and says so.**
//!
//! The driven assertion for `EditSession::annotation_deletion_refusal`, which
//! `crate::panels::properties::annotdelete` consumes and which — until
//! 2026-08-29 — **nothing in `pdfcer-gui` called at all**.
//!
//! # ★★★ What was wrong, and why only driving can prove it fixed
//!
//! On a certified or encrypted drawing this shell drew three live Delete
//! controls — the Format tab's, the canvas object menu's, and the Delete key —
//! and every press of them reached `delete_annotation`, was refused, and landed
//! in `app::actions::apply::vector_edit`'s `Err` arm, which writes one line to
//! the trace and *says nothing to the operator*. Worse, `actions::annots::delete`
//! then cleared the selection **anyway**, because it clears after the funnel
//! rather than on success — so the press removed the panel sentence that would
//! have explained the refusal, had there been one.
//!
//! Three visible controls, silently inert, and a gesture that destroyed its own
//! explanation. That is the shape this project is named after.
//!
//! ⇒ Every unit test in the crate can assert the *rules*. None of them can
//! assert the **sequence**: a manifest `visible_when` resolved by `egui-shell`
//! against a condition set rebuilt per frame, a canvas hit test against a real
//! page, a real keystroke through `canvas::keys`' ladder, and a panel section
//! drawn into a dock slot. R1: a capability is not verified until the running
//! binary has been driven through it.
//!
//! # ★★★ The fixture pair, and why the check drives BOTH
//!
//! `fixtures/certified-comments.pdf` and `fixtures/threaded-comments.pdf` are
//! **one document differing in one dictionary** — the catalog's `/Perms`. Same
//! pages, same annotations, same object numbers, same geometry.
//! `tools/gen-certified-fixture.py` builds both and its header carries why.
//!
//! Driving only the certified one would satisfy a build whose gate refused
//! *unconditionally*, which is a worse defect than the one being fixed: a
//! control withheld where it would have worked leaves the operator no gesture
//! that reports it. So phase E re-launches on the ordinary twin and asserts the
//! control is **there**. Because the two files differ in exactly one
//! dictionary, any difference the harness sees between the two runs is caused
//! by that dictionary and by nothing else.
//!
//! # ★★ The absence assertions, and what makes them admissible
//!
//! Two of this check's five assertions are that something is **not** there —
//! `properties.annot_delete.collateral` on the certified run, and the funnel's
//! own `delete-annotation` line after the keystroke. `crate::checks`' rule 4
//! forbids treating an absence as evidence unless the thing that would have
//! produced it has been shown to be working.
//!
//! Both are admissible here, and for the same reason:
//! `panels::properties::annotdelete` writes its `annot-delete-gates` census
//! line on **every frame the section runs**, refused or not, collateral or not.
//! So the check first reads that line — which proves the section drew and the
//! gate was asked — and only then reads the regions. Without it, "no collateral
//! region" and "the Properties panel never opened" would be the same trace.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | launch on the **certified** fixture in Review with the Properties panel shown | the `page` region declared |
//! | B | click the centre of the square's `/Rect` | `annot-select` naming it |
//! | C | read the census and the regions | `annot-delete-gates … refused=1`, `properties.annot_delete.refused` declared, `…collateral` **not** |
//! | D | press Delete **until the trace shows it was heard** | `canvas-delete-declined … reason=annot-delete-refused`, and **neither** `delete-annotation` nor `delete-annotation-refused` |
//! | E | relaunch on the **ordinary** twin, click the same point | `annot-delete-gates … refused=0`, and `properties.annot_delete.refused` **not** declared |
//!
//! # ★ Why Review mode rather than Edit
//!
//! `canvas::keys`' annotation rung is gated on `caps.author_markup`, which is
//! true in Review and in Edit both — but in Review `caps.edit_content` is
//! **false**, so a build whose Delete fell through the annotation rung to the
//! *content* rung would raise nothing there and the check would pass on a
//! broken build. Review is the mode in which the annotation rung is the only
//! rung that can act, which makes phase D's assertion about that rung and not
//! about the ladder's shape.

use crate::checks::driving::{SHELL_DIAG_ENV, declared};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// Review mode, with the Properties panel put on screen.
///
/// `file.properties` is the command that mounts and activates the panel from
/// any arrangement — `app::PdfcerApp::show_panel` mounts it first if the
/// operator's saved layout no longer holds it — so the check does not have to
/// know what dock layout the machine it runs on happens to have persisted.
const INVOKE: &str = "mode.review,file.properties";
/// The certified fixture. See the module header and
/// `tools/gen-certified-fixture.py`.
///
/// ★★★ **Relative to `CARGO_MANIFEST_DIR`, which is `tools/ui-verify`** — so
/// two levels up, not three. Written as `../../../fixtures/…` when this check
/// was authored, which resolves to `D:/Dev/fixtures/…`: a directory that does
/// not exist and never will. The `pdf.exists()` guard below turned that into a
/// SKIP whose message told the reader to run the generator, and the generator
/// writes to `fixtures/` in *this* repository — so the check could not pass on
/// any build, and its reason for not passing pointed at a fix that could not
/// work. `reflow`, `text_edit` and `signature_save` all use `../../fixtures/`;
/// that is the convention, and this is the only file that departed from it.
const CERTIFIED: &str = "../../fixtures/certified-comments.pdf";
/// The same document with the certification removed.
const ORDINARY: &str = "../../fixtures/threaded-comments.pdf";
/// The line the canvas writes when a click selects an annotation.
const SELECT_EVENT: &str = "annot-select";
/// ★★★ The per-frame census `panels::properties::annotdelete` writes.
///
/// The verb suffix is not decoration: `tools/gates/check-trace-names.py`
/// forbids a module's own summary line from sharing its first token with a
/// `vector_edit` funnel label, and `delete-annotation` is such a label. A
/// harness asking `last("delete-annotation")` would get the funnel's line
/// instead — `page`, `n`, `epoch`, `disclosures`, and none of the keys read
/// below. That confusion has produced a confident false negative on this
/// project three times.
const GATES_EVENT: &str = "annot-delete-gates";
/// The line `canvas::keys` writes when the Delete rung declines.
const DECLINED_EVENT: &str = "canvas-delete-declined";
/// ★★★ The **funnel's** own line for a delete that reached the engine.
///
/// Asserted **absent** in phase D. Its presence would mean the gate let the
/// action through and the engine refused it — which is the pre-fix behaviour
/// exactly, and which no region assertion above would catch, because the panel
/// would still have drawn its sentence on the frames before the press.
const FUNNEL_EVENT: &str = "delete-annotation";
/// ★★★ The **funnel's refusal** line — the one the pre-fix build actually wrote.
///
/// `app::actions::apply::vector_edit` writes `<label>` on success and
/// `<label>-refused` on an `Err`, and it is the second that a Delete which
/// walked past the gate produces on a certified document: the ladder raised
/// `AnnotAction::Delete`, `EditSession::delete_annotation` refused it, and this
/// line went to the trace **and nothing went to the operator**.
///
/// ★★ Reading it is what turns phase D from an accusation into a diagnosis. On
/// 2026-08-29 this check reported *"the keystroke did not reach `canvas::keys`
/// at all — check that the canvas had focus"* while the trace carried this line
/// four rows above the region the same phase went on to read: the key had
/// arrived, been processed, and been silently refused. The failure was real and
/// severe — `canvas::keys::Keys::annot_delete_refused` was a constant `false`,
/// because `canvas::interact` filled it from `annotdelete::refuses_selected`,
/// which reads `doc.selection` at a point in the frame where the selection has
/// been moved off the document — and the check's own message pointed at focus.
///
/// ⇒ A check that reads only the line it hopes for can only say *"nothing
/// happened"*. Naming the line that means *"the wrong thing happened"* is what
/// lets it say which.
const FUNNEL_REFUSED_EVENT: &str = "delete-annotation-refused";
/// The refusal sentence's region, published only when a gate refuses.
const REFUSED_REGION: &str = "properties.annot_delete.refused";
/// The collateral sentence's region, published only when there is collateral.
const COLLATERAL_REGION: &str = "properties.annot_delete.collateral";
/// The page's own region, so a failure can say whether a sheet was drawn.
const PAGE_REGION: &str = "page";

/// The square's `/Rect` centre, in PDF user space on page 1.
///
/// ★ Derived from `SQUARE_RECT` in `tools/gen-certified-fixture.py`
/// (`[120 560 320 700]`), and stated as a point rather than as a page fraction
/// — unlike most checks in this suite, which place their own operand and can
/// therefore choose a fraction. Here the operand is **in the fixture**, so the
/// aim has to be where the fixture put it. `HANDOFF.md` §2's defect-8 rule
/// still applies from the other side: phase B asserts that the click actually
/// selected the square by object id, so a click that missed reports as a miss
/// rather than as a broken gate.
const SQUARE_CENTRE: DocPoint = DocPoint {
    page: 0,
    x: 220.0,
    y: 630.0,
};

/// See the module documentation.
pub struct ACertifiedDocumentWithholdsAnnotationDelete;

impl Check for ACertifiedDocumentWithholdsAnnotationDelete {
    fn name(&self) -> &'static str {
        "annot_delete_gate"
    }

    fn defect(&self) -> &'static str {
        "on a certified or encrypted document the Delete for a comment is drawn, enabled and \
         silently inert — every press is refused into the trace, nothing is said to the \
         operator, and the selection is cleared anyway, taking the explanation with it"
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

/// One launch: open `fixture`, click the square, and return the gate census's
/// `refused` flag together with whether each region was declared.
///
/// Factored because phases A–C and phase E are the **same** sequence against
/// two files, and the whole value of the pair is that they were driven
/// identically. Two hand-written copies would eventually differ in a settle or
/// in an aim, and the difference would be reported as a difference between the
/// documents.
struct Run {
    session: Session,
    driver: Driver,
    /// `annot-delete-gates … refused=` — `1` on the certified file, `0` on the
    /// ordinary one.
    refused: bool,
    /// Whether `properties.annot_delete.refused` is currently declared.
    refused_region: bool,
    /// Whether `properties.annot_delete.collateral` is currently declared.
    collateral_region: bool,
}

/// Launch on `fixture`, select the square, and read the gate.
fn open_and_select(
    ctx: &CheckContext,
    report: &mut CheckReport,
    fixture: &str,
    label: &str,
) -> Result<std::result::Result<Run, String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // ★ NOT `ctx.pdf`, and the reason is the same one `signature_save` gives:
    // the oracle here is bound to a document whose certification, annotation
    // geometry and reply threading are all known, so a `--pdf` an operator
    // passed would be measured against an expectation that is not about it.
    let pdf = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(fixture);
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the {label} fixture is missing at {}. Regenerate both: \
             python tools/gen-certified-fixture.py — no existing fixture carries an enforced \
             certification, and `signed-two-pages.pdf` is deliberately an approval signature.",
            pdf.display()
        )));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("annot-delete-{label}.trace.txt")));
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
        "launched {} on the {label} fixture as pid {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Ok(Err(format!(
            "the {label} run drew no page, so nothing below can be read. The fixture is two \
             A4 pages of one stroked rectangle each; if this fails the document did not open."
        )));
    }

    // ---- select the square ---------------------------------------------
    //
    // ★ The click is asserted by OBJECT ID rather than by "something got
    // selected". The fixture's page also carries a signature widget at
    // `[60 60 300 120]`, and a click that landed there would take the form
    // surface's branch and produce a `selected_field` — at which point the gate
    // is deliberately not consulted (the dispatcher's ladder puts a field
    // first), and this check would report the gate as open on a certified file.
    let target = aim(ctx, &session, page_geometry(), SQUARE_CENTRE)?;
    driver.click_at(target)?;
    session.settle(12);

    let trace = session.trace()?;
    let Some(selected) = trace.last(SELECT_EVENT) else {
        return Ok(Err(format!(
            "the click at the square's centre selected no annotation on the {label} run. \
             The fixture puts a /Square at [120 560 320 700] on page 1; either the canvas \
             hit test does not reach it or the aim landed elsewhere."
        )));
    };
    report.note(format!("{label}: {SELECT_EVENT} {}", selected.raw));

    let Some(gates) = trace.last(GATES_EVENT) else {
        return Ok(Err(format!(
            "the Properties panel wrote no `{GATES_EVENT}` line on the {label} run, so the \
             annotation-delete section never drew and every region assertion below would be \
             an absence with nothing behind it (rule 4). Either `file.properties` did not \
             put the panel on screen, or the section returned early."
        )));
    };
    let refused = gates.get_usize("refused") == Some(1);
    report.note(format!("{label}: {GATES_EVENT} {}", gates.raw));

    Ok(Ok(Run {
        refused_region: declared(&trace, ui_rect, REFUSED_REGION).is_some(),
        collateral_region: declared(&trace, ui_rect, COLLATERAL_REGION).is_some(),
        refused,
        session,
        driver,
    }))
}

/// The fixtures' page size, which both generators write as A4.
///
/// Stated rather than read from the file: the check is bound to fixtures it
/// generates itself, so a page size read back from them could only ever confirm
/// what the generator wrote — and a `--page-size` override would let a caller
/// aim this check at a document it is not about.
const fn page_geometry() -> PageGeometry {
    PageGeometry {
        width_pt: 595.0,
        height_pt: 842.0,
    }
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a comment on the page and \
             presses Delete. Both are real pointer and keyboard gestures.",
        ));
    }

    // ---- A, B, C: the certified file ------------------------------------
    let certified = match open_and_select(ctx, report, CERTIFIED, "certified")? {
        Ok(run) => run,
        Err(failure) => return Ok(Some(failure)),
    };
    if !certified.refused {
        return Ok(Some(
            "the gate reported `refused=0` on a document carrying an enforced certification \
             (/Perms /DocMDP, /P 2). §12.8.2.2 Table 254 puts annotation deletion on the \
             `P = 3` line, so `P = 2` must refuse — `annotation_deletion_refusal` is either \
             not being called or its answer is being dropped."
                .to_owned(),
        ));
    }
    if !certified.refused_region {
        return Ok(Some(format!(
            "the gate refused and no `{REFUSED_REGION}` was published, so the operator was \
             given a withheld control and no sentence. R9 permits absence in place of a \
             permanently-refused control; it does not permit silence beside it, and a panel \
             that simply omits half its controls looks half-drawn."
        )));
    }
    if certified.collateral_region {
        return Ok(Some(format!(
            "`{COLLATERAL_REGION}` was published on a refused document. The collateral \
             describes what a delete *would* take with it, and on this file there is no \
             delete to describe — `annotation_deletion_preview` raises the same refusals, so \
             a sentence here means a cached answer is being read as a fact."
        )));
    }

    // ---- D: the keystroke ------------------------------------------------
    //
    // ★★ The most valuable single assertion in the check, and the only one that
    // catches the pre-fix build directly. The regions above would all have been
    // right on a build whose *panel* asked the query and whose *ladder* did
    // not: the sentence would be drawn, and Delete would still raise the
    // action, be refused into the trace, and clear the selection — taking the
    // sentence away with it.
    // ★★★ **Pressed until the trace shows it was heard, not pressed once.**
    //
    // See `driving::press_until_traced`'s header for the rule and the day that
    // bought it. The three names are the complete list of what this key can
    // produce here, and giving all three is the contract: a list containing only
    // `DECLINED_EVENT` would turn a build that walks past the gate into *"the
    // key never arrived"*, which is the same false negative wearing the
    // opposite face.
    //
    // ★ Repeating the press is safe on THIS fixture by construction — every
    // delete on a certified document is refused, so a press that lands changes
    // nothing about the file — and the loop stops on the first one that lands
    // regardless. It would not be safe on the ordinary twin, which is why phase
    // E presses nothing.
    let heard = crate::checks::driving::press_until_traced(
        &certified.session,
        &certified.driver,
        vk::DELETE,
        &[DECLINED_EVENT, FUNNEL_EVENT, FUNNEL_REFUSED_EVENT],
    )?;
    if !heard {
        return Err(Error::new(format!(
            "Delete was pressed {} time(s) at the canvas and the application traced no \
             response to any of them — no `{DECLINED_EVENT}`, no `{FUNNEL_EVENT}`, no \
             `{FUNNEL_REFUSED_EVENT}`. **SKIPPED rather than failed**, on this suite's \
             standing rule: a check that cannot show its input was delivered has learned \
             nothing about the program, and a defect report is worse than no report. A bare \
             key has been measured on this machine arriving zero times in six attempts with a \
             dock panel raised, and this check raises one. Trace: {}.",
            crate::checks::driving::PRESS_TRIES,
            certified.session.trace_path().display()
        )));
    }
    let trace = certified.session.trace()?;
    // ★★★ The pre-fix build's signature, and the arm that names it.
    //
    // Checked BEFORE `FUNNEL_EVENT` because on a certified document it is the
    // one that actually appears: the delete reaches the engine and the engine
    // refuses, so the funnel takes its `Err` arm and writes the `-refused`
    // spelling. A phase D that read only the success spelling saw neither line
    // and fell through to *"the keystroke did not arrive"*.
    if let Some(refused) = trace.last(FUNNEL_REFUSED_EVENT) {
        return Ok(Some(format!(
            "★★★ DELETE WALKED PAST THE GATE AND WAS REFUSED BY THE ENGINE: \
             `{FUNNEL_REFUSED_EVENT} {}`.\n\
             The keystroke arrived and was processed — this is not a delivery problem — and \
             `canvas::keys`' annotation rung raised `AnnotAction::Delete` anyway, which means \
             `Keys::annot_delete_refused` was `false` on a document the Properties panel had \
             already traced `refused=1` for. Those two answers come from one function, so a \
             disagreement means the caller is asking it about the wrong thing: \
             `canvas::interact` takes the selection off the document for the length of the \
             frame (`std::mem::take(&mut doc.selection)`), so a gate reading `doc.selection` \
             there sees an empty one and answers *permitted* for every document. The refusal \
             then lands in `actions::apply::vector_edit`'s `Err` arm, which says nothing to \
             the operator, and `actions::annots::delete` clears the selection regardless — \
             taking the panel sentence that explained it away with it.",
            refused.raw
        )));
    }
    if let Some(funnel) = trace.last(FUNNEL_EVENT) {
        return Ok(Some(format!(
            "Delete reached the engine on a certified document: `{FUNNEL_EVENT} {}`. The \
             ladder in `canvas::keys` must decline before raising the action — the engine \
             refuses it either way, but the refusal lands in `vector_edit`'s `Err` arm, \
             which says nothing to the operator, and `actions::annots::delete` then clears \
             the selection regardless, removing the panel sentence that explained it.",
            funnel.raw
        )));
    }
    match trace.last(DECLINED_EVENT) {
        Some(line) if line.get("reason") == Some("annot-delete-refused") => {
            report.note(format!("certified: {DECLINED_EVENT} {}", line.raw));
        }
        Some(line) => {
            return Ok(Some(format!(
                "Delete declined for the wrong reason: `{}`. A rung above the annotation one \
                 swallowed the press, so this run says nothing about the gate.",
                line.raw
            )));
        }
        None => {
            // ★ Unreachable by construction, and written out anyway.
            //
            // `press_until_traced` returned `true`, which means one of the three
            // names appeared; the two funnel arms above returned; so this one is
            // the decline and cannot be `None`. It is spelled out rather than
            // `unreachable!()` because the three names are a list a future
            // change can add to, and the day one is added without a matching arm
            // this must say *"the trace moved and this check does not know
            // which line moved it"* rather than panic inside a sweep.
            //
            // ★★ What it must NOT say is what it used to say — *"the keystroke
            // did not reach `canvas::keys` at all"*. Delivery is settled above,
            // by the loop, and re-litigating it here is how a real defect was
            // reported as a focus problem.
            return Ok(Some(format!(
                "Delete was heard — `driving::press_until_traced` saw the event count move — \
                 and then produced neither a `{FUNNEL_EVENT}`, a `{FUNNEL_REFUSED_EVENT}` nor \
                 a `{DECLINED_EVENT}` line as the last of its kind. Delivery is not the \
                 question: some other line named in the evidence list moved, so the ladder's \
                 vocabulary has changed and this phase's three arms no longer cover it. Trace: \
                 {}.",
                certified.session.trace_path().display()
            )));
        }
    }
    if declared(
        &trace,
        ctx.profile.vocab.ui_rect_event.unwrap_or(""),
        REFUSED_REGION,
    )
    .is_none()
    {
        return Ok(Some(format!(
            "after the press, `{REFUSED_REGION}` is no longer declared — the selection was \
             cleared by a delete that did not happen, and the sentence explaining why went \
             with it. That is the exact failure this check exists for: a silence that also \
             destroys its own explanation."
        )));
    }

    // ---- E: the ordinary twin -------------------------------------------
    //
    // ★★★ Without this, a build whose gate refused unconditionally passes
    // everything above. The two fixtures differ in one dictionary, so a
    // difference here is caused by that dictionary and by nothing else.
    let ordinary = match open_and_select(ctx, report, ORDINARY, "ordinary")? {
        Ok(run) => run,
        Err(failure) => return Ok(Some(failure)),
    };
    if ordinary.refused {
        return Ok(Some(
            "the gate refused on the UNCERTIFIED twin, which differs from the certified \
             fixture only in the catalog's /Perms entry. An approval signature is not an \
             enforced certification — `forbids_structural_change` is `perms_enforced && \
             signatures > 0` — so a build that refuses here withholds Delete from every \
             signed document, which is worse than the defect being fixed: the operator has \
             no gesture left that reports it."
                .to_owned(),
        ));
    }
    if ordinary.refused_region {
        return Ok(Some(format!(
            "`{REFUSED_REGION}` was published on the uncertified twin. The panel is \
             explaining a refusal the gate did not make, so the sentence and the control are \
             being derived from two different questions — which is precisely what \
             `annotdelete::gate` exists as one function to prevent."
        )));
    }
    if !ordinary.collateral_region {
        return Ok(Some(format!(
            "no `{COLLATERAL_REGION}` on the uncertified twin. The fixture's square carries \
             a /Popup companion and one /IRT reply with no /RT — Table 170's default is `R` \
             — so `annotation_deletion_preview` must report `popup_removed` and \
             `replies_orphaned: 1`, and both belong on screen BEFORE the press. This delete \
             has no confirmation dialog, so there is no later moment to say it."
        )));
    }
    drop(ordinary);
    Ok(None)
}
