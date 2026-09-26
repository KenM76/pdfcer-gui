//! `field_delete_gate` — **a certified document does not offer a Delete for its
//! form fields, says so, and does not lose the sentence when Delete is
//! pressed.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/field_delete_gate.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// Edit mode, with the Properties panel put on screen.
///
/// `file.properties` is the command that mounts and activates the panel from
/// any arrangement, so the check does not have to know what dock layout the
/// machine it runs on happens to have persisted. `mode.edit` is load-bearing
/// rather than cosmetic — see the module header's last section.
const INVOKE: &str = "mode.edit,file.properties";
/// The certified fixture. See the module header and
/// `tools/gen-certified-fixture.py`.
///
/// **Relative to `CARGO_MANIFEST_DIR`, which is `tools/ui-verify`** — two
/// levels up, not three. See [`super::annot_delete_gate`]'s note on the same
/// constant: this file inherited the wrong depth from it, and both resolved to
/// a `D:/Dev/fixtures/` that does not exist, so both SKIPPED on every run while
/// telling the reader to run a generator that writes somewhere else.
const CERTIFIED: &str = "../../fixtures/certified-comments.pdf";
/// The same document with the certification removed.
const ORDINARY: &str = "../../fixtures/threaded-comments.pdf";
/// The line `canvas::forms` writes when a click selects a form field.
const SELECT_EVENT: &str = "form-field-selected";
/// The per-frame census `panels::properties::formfield` writes.
///
/// The `-gates` suffix is not decoration: `tools/gates/check-trace-names.py`
/// forbids a module's own summary line from sharing its first token with a
/// `vector_edit` funnel label. A harness reading a bare name would get the
/// funnel's line — `page`, `n`, `epoch`, `disclosures`, and none of the keys
/// read below. That confusion has produced a confident false negative on this
/// project three times.
const GATES_EVENT: &str = "form-field-gates";
/// The line `canvas::keys` writes when a Delete rung declines.
///
/// Shared with the annotation rung, which is why the `reason=` key is read
/// rather than the event alone: in Edit mode both rungs are reachable, and a
/// decline from the wrong one would say nothing about the gate under test.
const DECLINED_EVENT: &str = "canvas-delete-declined";
/// The **funnel's** own line for a widget delete that reached the engine.
///
/// Asserted **absent** in phase D. Its presence means the ladder let the action
/// through and the engine refused it — which is the pre-fix behaviour exactly,
/// and which no region assertion would catch, because the panel would still
/// have drawn its sentence on the frames before the press.
const FUNNEL_EVENT: &str = "delete-widget";
/// The **Delete field** button's region, published only when it is drawn.
const DELETE_REGION: &str = "properties.form_field.delete";
/// The refusal sentence's region, published only when the gate refuses.
///
/// Exactly one of this and [`DELETE_REGION`] is declared on any frame the
/// section runs, which is what makes each one's absence readable.
const REFUSED_REGION: &str = "properties.form_field.delete_refused";
/// The page's own region, so a failure can say whether a sheet was drawn.
const PAGE_REGION: &str = "page";

/// The signature widget's `/Rect` centre, in PDF user space on page 1.
///
/// Derived from `objs[11]` in `tools/gen-certified-fixture.py`
/// (`/Rect [60 60 300 120]`), and stated as a point rather than as a page
/// fraction for the reason [`super::annot_delete_gate`] gives about its own
/// operand: the target is **in the fixture**, so the aim has to be where the
/// fixture put it. Phase B asserts the click really selected `Certifier` by
/// name, so a click that missed reports as a miss rather than as a broken gate.
const WIDGET_CENTRE: DocPoint = DocPoint {
    page: 0,
    x: 180.0,
    y: 90.0,
};

/// The field the fixture pair names, asserted by `/T` rather than discovered.
///
/// The whole evidential value of the pair is that the two documents are
/// identical apart from one dictionary. A check that went looking for
/// *"a field"* could find a different one in each run and would report the
/// difference as a gate difference.
const FIELD_NAME: &str = "Certifier";

/// See the module documentation.
pub struct ACertifiedDocumentWithholdsFieldDelete;

impl Check for ACertifiedDocumentWithholdsFieldDelete {
    fn name(&self) -> &'static str {
        "field_delete_gate"
    }

    fn defect(&self) -> &'static str {
        "on a certified or encrypted document the Delete for a form field is drawn, enabled and \
         silently inert on three surfaces — the condition is a no-op with a field selected, the \
         canvas.field menu carries no gate at all, and the Delete key's field rung asks nothing \
         — and the press clears the selection anyway, blanking the Properties panel sentence \
         that was explaining the refusal"
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

/// One launch: open `fixture`, click the widget, and return the gate census's
/// `delete_refused` flag together with whether each region was declared.
///
/// Factored because phases A–C and phase E are the **same** sequence against
/// two files, and the whole value of the pair is that they were driven
/// identically. Two hand-written copies would eventually differ in a settle or
/// in an aim, and the difference would be reported as a difference between the
/// documents.
struct Run {
    session: Session,
    driver: Driver,
    /// `form-field-gates … delete_refused=` — `1` on the certified file, `0` on
    /// the ordinary one.
    refused: bool,
    /// Whether `properties.form_field.delete` is currently declared.
    delete_region: bool,
    /// Whether `properties.form_field.delete_refused` is currently declared.
    refused_region: bool,
}

/// Launch on `fixture`, select the signature widget, and read the gate.
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
    // NOT `ctx.pdf`: the oracle here is bound to a document whose
    // certification, `/AcroForm` and widget geometry are all known, so a
    // `--pdf` an operator passed would be measured against an expectation that
    // is not about it.
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

    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("field-delete-{label}.trace.txt")));
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
             A4 pages carrying one stroked rectangle each and a signature widget on page 1; \
             if this fails the document did not open."
        )));
    }

    // ---- select the signature widget -------------------------------------
    //
    // Asserted by FIELD NAME rather than by "something got selected". Page 1
    // also carries a `/Square` markup at `[120 560 320 700]`, and a click that
    // landed there would take the annotation branch — at which point the field
    // gate is deliberately not consulted and this check would report it open on
    // a certified file.
    let target = aim(ctx, &session, page_geometry(), WIDGET_CENTRE)?;
    driver.click_at(target)?;
    session.settle(12);

    let trace = session.trace()?;
    let selected = trace.last(SELECT_EVENT);
    let selected_field = selected.and_then(|l| l.get("field"));
    if selected_field != Some(FIELD_NAME) {
        return Ok(Err(format!(
            "the click at the signature widget's centre did not select `{FIELD_NAME}` on the \
             {label} run; the last `{SELECT_EVENT}` line said {:?}. The fixture puts a merged \
             /Sig field's sole widget at [60 60 300 120] on page 1. Either the canvas form \
             surface does not offer it as a target, the aim landed elsewhere, or the shell is \
             not in Edit mode — `canvas::forms` gives the SELECTION surface to `edit_content` \
             and the FILL surface to Read and Review.",
            selected.map(|l| l.raw.clone())
        )));
    }
    report.note(format!(
        "{label}: {SELECT_EVENT} {}",
        selected.map_or("", |l| l.raw.as_str())
    ));

    let Some(gates) = trace.last(GATES_EVENT) else {
        return Ok(Err(format!(
            "the Properties panel wrote no `{GATES_EVENT}` line on the {label} run, so the \
             form-field section never drew and every region assertion below would be an \
             absence with nothing behind it (rule 4). Either `file.properties` did not put \
             the panel on screen, or the section returned early — it has three early \
             returns: no selection, a selection naming a field the document no longer has, \
             and no /AcroForm."
        )));
    };
    let refused = gates.get_usize("delete_refused") == Some(1);
    report.note(format!("{label}: {GATES_EVENT} {}", gates.raw));

    Ok(Ok(Run {
        delete_region: declared(&trace, ui_rect, DELETE_REGION).is_some(),
        refused_region: declared(&trace, ui_rect, REFUSED_REGION).is_some(),
        refused,
        session,
        driver,
    }))
}

/// The fixtures' page size, which the generator writes as A4.
///
/// Stated rather than read from the file: the check is bound to fixtures the
/// repository generates itself, so a page size read back from them could only
/// confirm what the generator wrote — and a `--page-size` override would let a
/// caller aim this check at a document it is not about.
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
            "input is disabled (--no-input). This check clicks a form field on the page and \
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
            "the gate reported `delete_refused=0` on a document carrying an enforced \
             certification (/Perms /DocMDP, /P 2). Deleting a field is a STRUCTURAL change to \
             the form, which is what a certification signature exists to freeze — §12.8.2.2 \
             Table 257 permits filling such a form and forbids restructuring it — so \
             `EditSession::deletion_refusal` must answer Some. Either it is not being called \
             or its answer is being dropped."
                .to_owned(),
        ));
    }
    if certified.delete_region {
        return Ok(Some(format!(
            "`{DELETE_REGION}` was published on a document whose gate refuses, so the Delete \
             field button is drawn over a form nothing can restructure. R9: a permanently \
             refused capability renders nothing — greying is for the temporarily \
             unavailable and must explain itself on hover, and a certification signature is \
             neither temporary nor arguable."
        )));
    }
    if !certified.refused_region {
        return Ok(Some(format!(
            "the gate refused and no `{REFUSED_REGION}` was published, so the operator was \
             given a withheld control and no sentence. R9 permits absence in place of a \
             permanently refused control; it does not permit silence beside it, and a panel \
             that simply omits half its controls looks half-drawn."
        )));
    }

    // ---- D: the keystroke ------------------------------------------------
    //
    certified.driver.press(vk::DELETE)?;
    certified.session.settle(12);
    let trace = certified.session.trace()?;
    if let Some(funnel) = trace.last(FUNNEL_EVENT) {
        return Ok(Some(format!(
            "Delete reached the engine on a certified document: `{FUNNEL_EVENT} {}`. Rung 0 \
             of `canvas::keys`' ladder must decline before raising the action — the engine \
             refuses it either way, but the refusal lands in `vector_edit`'s `Err` arm, which \
             says nothing to the operator by its own recorded decision.",
            funnel.raw
        )));
    }
    match trace.last(DECLINED_EVENT) {
        Some(line) if line.get("reason") == Some("field-delete-refused") => {
            report.note(format!("certified: {DECLINED_EVENT} {}", line.raw));
        }
        Some(line) => {
            return Ok(Some(format!(
                "Delete declined for the wrong reason: `{}`. Rung 0 is the FORM FIELD rung and \
                 it is first of four; a decline naming another rung means the press was \
                 swallowed before it got there, so this run says nothing about the field gate.",
                line.raw
            )));
        }
        None => {
            return Ok(Some(format!(
                "Delete produced neither a `{FUNNEL_EVENT}` nor a `{DECLINED_EVENT}` line. \
                 The keystroke did not reach `canvas::keys` at all — check that the canvas \
                 had focus, that no dialog is in front, and that no text editor holds the \
                 key (D1: a focused `TextEdit` keeps Delete for itself)."
            )));
        }
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("");
    if declared(&trace, ui_rect, REFUSED_REGION).is_none() {
        return Ok(Some(format!(
            "after the press, `{REFUSED_REGION}` is no longer declared — the selection was \
             cleared by a delete that did not happen, and the sentence explaining why went \
             with it. That is the exact failure this check exists for: a silence that also \
             destroys its own explanation. `actions::forms::delete_widget` must clear \
             `doc.selected_field` only once the edit epoch has actually moved."
        )));
    }

    // ---- E: the ordinary twin -------------------------------------------
    //
    // Without this, a build whose gate refused unconditionally passes
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
             signed form, which is worse than the defect being fixed: the operator has no \
             gesture left that reports it."
                .to_owned(),
        ));
    }
    if !ordinary.delete_region {
        return Ok(Some(format!(
            "no `{DELETE_REGION}` on the uncertified twin, so the Delete field button was \
             withheld where it would have worked. The panel publishes that region only when \
             it draws the control, and the census on this run says the gate is open."
        )));
    }
    if ordinary.refused_region {
        return Ok(Some(format!(
            "`{REFUSED_REGION}` was published on the uncertified twin. The panel is \
             explaining a refusal the gate did not make, so the sentence and the control are \
             being derived from two different questions — which is precisely what \
             `formfield::refuses_delete` exists as one function to prevent."
        )));
    }
    drop(ordinary);
    Ok(None)
}
