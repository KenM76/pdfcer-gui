//! `delete_key_after_canvas_click` — the regression test for **D1**.
//!
//! # The defect
//!
//! Reported by the operator as *"I can't even click on an object and delete it
//! by hitting the delete key."* It is real, it is not a discoverability
//! problem, and the selection half works perfectly — which is what makes it so
//! confusing to use.
//!
//! The causal chain, from `DEFECTS.md` D1, verified against source:
//!
//! 1. **Click-select works with no gating.** The canvas hit-tests and assigns
//!    the selection. The object visibly selects.
//! 2. **The canvas grabs egui keyboard focus on every click**
//!    (`main.rs:16891`): `if image_response.clicked() { image_response
//!    .request_focus(); }`. Deliberate and reasonable — the canvas was meant to
//!    be a real Tab stop rather than an inert image. Because the widget is
//!    recreated every frame its id stays live, so the focus never lapses.
//! 3. **The keyboard guard tests the wrong thing** (`main.rs:13777`):
//!    `let typing = ctx.egui_wants_keyboard_input();`. In egui 0.35 that is
//!    **not** "a text field is focused" — verified in the vendored source at
//!    `context.rs:2884`, it is `self.memory(|m| m.focused().is_some())`, i.e.
//!    *any* focused widget, the canvas included. The doc comment directly above
//!    it promises the opposite. This is an egui API footgun, not a careless
//!    read.
//! 4. **So the binding is never installed.** `if (!tool_active ||
//!    canvas_delete_target) && !typing` — the first half is satisfied, `typing`
//!    is `true` from step 3, and the branch never runs.
//! 5. **The deletion logic downstream is correct and simply unreachable.**
//!
//! The same guard also kills `PageDown`/`PageUp`, `Home`/`End` and `[`/`]`
//! from the same click, so keyboard page navigation dies with it.
//!
//! # Why the existing test could not catch it
//!
//! `collect_keyboard_actions` has exactly one test, and it builds a bare
//! `egui::Context::default()` with **no widgets**. Therefore `memory.focused()`
//! is `None`, therefore `typing` is always `false`, therefore the single
//! property that breaks in the real application is *structurally absent from
//! the only harness that exercises the function*. Object deletion is covered at
//! the `Action` level and never through the key.
//!
//! The regression is self-declared in its own commit message: *"a focused text
//! field keeps its unmodified keys — analysis-confirmed, NOT empirically
//! verified."*
//!
//! # How this check detects it
//!
//! By doing what the operator did, through the operating system:
//!
//! 1. convert a **document point** to a screen point (never a literal screen
//!    coordinate — see [`crate::coords`]);
//! 2. click it with the real cursor, so egui's focus machinery runs exactly as
//!    it does for a person — the reason [`crate::input`] rejects in-process
//!    injection for this check specifically;
//! 3. **assert the selection is non-empty** — the precondition, without which
//!    the next step's failure would be a mystery rather than a defect;
//! 4. press `Delete` with a real keystroke;
//! 5. **assert the object count dropped.**
//!
//! Against the current old binary, steps 1–3 succeed and step 5 does not: no
//! deletion is traced, because the key was suppressed at step 4. That is a
//! FAIL, and it is the acceptance evidence for this harness.
//!
//! # Two oracles for step 5, and the check says which one it used
//!
//! **The page object count**, when the binary reports one (`objects n=…`).
//! Read after the click, read again after the key, compared. This is the
//! preferred oracle for one reason: it measures *the property the check is
//! about* — did the object leave the page — rather than the verb that was
//! meant to change it. It is also indifferent to a deletion implemented by
//! some future command nobody remembered to add a trace call to.
//!
//! **The `delete-objects` event**, when there is no count. Weaker, in the
//! specific way described below, and used only as the fallback.
//!
//! Every failure string from this check begins by naming which of the two it
//! used. That costs one clause and removes the question a reader would
//! otherwise have to answer by reading this file: *is "no deletion traced" a
//! statement about the page, or about a trace call site?*
//!
//! # The one subtle judgement in this file
//!
//! The fallback oracle's evidence is **the absence of a `delete-objects` trace
//! line**. Absence is normally weak
//! evidence and this crate refuses it elsewhere. It is admissible here, and
//! only here, for three stated reasons:
//!
//! * the event is **in the binary's vocabulary** — the code path exists and
//!   traces unconditionally when it runs, so its absence means the path was
//!   not taken, not that the binary cannot report;
//! * the harness has already **established the precondition** in step 3, so it
//!   is known to have got as far as a live selection;
//! * the harness **pressed the key itself**, so it is known that the key event
//!   was delivered to the foreground window.
//!
//! Remove any one of those and this becomes a SKIP. That is why step 3 is an
//! assertion and not a convenience, and why a hit test that finds nothing
//! produces a SKIP rather than a FAIL: the harness cannot then distinguish "the
//! application is broken" from "the harness aimed at empty page", and filing
//! the former when it is the latter is precisely the retracted-false-defect
//! outcome `crate::coords` documents.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::CanvasMapping;
use crate::error::Result;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// See the module documentation.
pub struct DeleteKeyAfterCanvasClick;

impl Check for DeleteKeyAfterCanvasClick {
    fn name(&self) -> &'static str {
        "delete_key_after_canvas_click"
    }

    fn defect(&self) -> &'static str {
        "D1 — the canvas takes egui focus on click, and the keyboard guard tests \
         'any widget focused' rather than 'a text field focused', so Delete is \
         permanently suppressed from the first canvas click onward"
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
/// The three-way return is the SKIP/FAIL/PASS rule made structural:
/// `Err` is a precondition that was absent (SKIP), `Ok(Some(_))` is an
/// assertion that did not hold (FAIL), `Ok(None)` is a pass. A check author
/// who reaches for `?` gets a SKIP, which is the safe default — the unsafe
/// default would be a pass.
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;

    // --- preconditions: can the harness begin at all? ----------------------
    let exe = ctx.resolve_exe().ok_or_else(|| {
        crate::error::Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        crate::error::Error::new(
            "no fixture document. Pass --pdf: this check has to open a drawing that contains \
             at least one selectable object.",
        )
    })?;
    let target = ctx.target.ok_or_else(|| {
        crate::error::Error::new(
            "no target point. Pass --doc-point PAGE,X,Y naming a point in PDF user space \
             (origin bottom-left) where the fixture has a selectable object. The harness \
             deliberately has no default: guessing a point would produce a click on empty \
             page, which is symptom-identical to a broken hit test.",
        )
    })?;
    if !ctx.allow_input {
        return Err(crate::error::Error::new(
            "input is disabled (--no-input), and this check cannot be performed without \
             clicking and typing. Reported as SKIPPED rather than passed: a check that did \
             not run has learned nothing.",
        ));
    }

    let page = match ctx.page_size {
        Some((w, h)) => crate::coords::PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            crate::error::Error::new(format!(
                "cannot read a page size from {}. The harness needs the page height to flip \
                 PDF y (up) into window y (down). Pass --page-size WxH. It refuses to guess: \
                 a wrong page height mirrors every click about the page centre, which lands \
                 on the page and hit-tests something plausible.",
                pdf.display()
            ))
        })?,
    };
    report.note(format!(
        "fixture {} — page 1 is {:.0}x{:.0} pt",
        pdf.display(),
        page.width_pt,
        page.height_pt
    ));

    // --- launch ------------------------------------------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("delete_key.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    // ★ The SHELL's diagnostic channel too, added 2026-08-17 with the Edit-mode
    // step below. `egui-shell` traces ribbon and mode events under its own
    // prefix and its own switch, separate from the application's — so a check
    // that sets only the application's env sees no `ribbon-mode-selected` line
    // and cannot tell "the click missed" from "the channel is off". That is
    // exactly the ambiguity `click_mode_segment`'s failure text refuses to
    // resolve, and it is resolved here instead, at the launch that causes it.
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} with {}={} and {}={}",
        exe.display(),
        session.pid(),
        ctx.profile.diag_env.0,
        ctx.profile.diag_env.1,
        SHELL_DIAG_ENV.0,
        SHELL_DIAG_ENV.1
    ));
    report.artifact(session.trace_path().to_path_buf());
    // The measured window geometry, noted before anything depends on it. When
    // a driven click appears to do nothing, this line is the first thing to
    // read: a client area of implausible size or at an implausible origin
    // means the harness found the wrong window, and every coordinate below is
    // then wrong in a way that looks like the application ignoring input.
    if let Ok(f) = session.frame() {
        report.note(format!(
            "window client area {}x{} px at desktop ({}, {}), DPI scale {:.2}",
            f.client_size.0, f.client_size.1, f.client_origin.0, f.client_origin.1, f.scale
        ));
    }
    // Generous: opening and rastering a CAD drawing is the slow part, and a
    // check that measured before the page was drawn would click into a canvas
    // whose rect is still the placeholder.
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(vocab.start_event) {
        return Err(crate::error::Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process. Without the trace this check has no oracle. Captured stderr is at {}.",
            vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    for reject in trace.rejected_steps() {
        report.note(format!(
            "the application REJECTED a script step: {}",
            reject.raw
        ));
    }

    // --- step 0: PUT THE APPLICATION IN A MODE THAT CAN SELECT -------------
    //
    // ★ Added 2026-08-17, and its absence had turned this check into a
    // reporter of a defect that does not exist.
    //
    // This check was written before Read mode existed. The shell's default
    // mode is `read` — `mode-changed from=None to=read remembered=false` on a
    // fresh profile — and a Read-mode canvas click is CORRECTLY refused by
    // `app::modes::capability`, which is `DEFECTS.md` D6's fix working
    // exactly as designed.
    //
    // So the check clicked page content in a mode that must not select,
    // observed no selection, and reported *"Selection is not taking the hit
    // test's result"* — naming the selection subsystem for a refusal the
    // gate had made on purpose. Six doc-points spread across a dense CAD
    // sheet all reported `hit 0 object(s)`, which is what finally gave it
    // away: a hit test that misses everywhere is not a hit test, it is a
    // gate.
    //
    // ★ Two checks in this suite were asserting OPPOSITE things about the
    // same gesture and only one of them established the mode.
    // `read_mode_refuses_canvas_edits` clicks the Review segment, then the
    // Read segment, and asserts the click selects nothing. This one asserts
    // the click selects something. Both are right; the difference is
    // entirely the mode, and the one that did not say so was reading a
    // property of whatever `layout.ron` happened to remember.
    //
    // That is the deeper finding and it generalises past this check: **a
    // driven check that does not establish the state it needs is measuring
    // the previous run.** The persisted-mode case is especially quiet,
    // because `read_mode_hides_the_chrome` deliberately *ends in Read mode*
    // (it says so — the only way out is a chord and this machine cannot
    // inject one), so a full-suite run leaves a profile that biases the next
    // one.
    //
    // Edit rather than Review: Review offers markup on content but this
    // check is about selecting page CONTENT and deleting it, which is
    // authoring, and `app::modes` gives content selection to Edit.
    let driver = Driver::new(session.window());
    // `Err` here is a SKIP, which is the right verdict: a profile that cannot
    // say where its mode segments are leaves this check unable to establish
    // its own precondition, and a check that cannot establish its precondition
    // must not report on the property beyond it.
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        crate::error::Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot \
             state where its mode segments are and this check cannot leave Read mode — \
             where a canvas click on content is refused by design.",
            ctx.profile.name
        ))
    })?;
    click_mode_segment(&session, &driver, ui_rect, "edit")?;
    report.note(
        "clicked the Edit mode segment first — the shell's default mode is Read, where a \
         canvas click on content is refused BY DESIGN (DEFECTS.md D6). Without this step \
         the check reports the mode gate as a selection defect, which is what it did \
         before 2026-08-17",
    );
    let trace = session.trace()?;

    // --- the layout probe --------------------------------------------------
    //
    // Some builds only trace their canvas layout when something happens, so a
    // freshly opened document reports no canvas rect and the harness has
    // nothing to aim against. One click at the client-area centre provokes the
    // report. It is not an assertion and it does not care what it hits; the
    // real document-space click follows and replaces whatever it selected.
    // See `WindowFrame::layout_probe_point` for the full argument, the
    // assumption it rests on, and the one-line change to the application that
    // makes it unnecessary.
    let driver = Driver::new(session.window());
    let trace = if trace.last(vocab.canvas_event).is_some() {
        trace
    } else {
        report.note(format!(
            "no `{}` event yet — this build appears to trace its canvas layout only on \
             pointer events, so the harness is sending one layout-probe click at the \
             client-area centre to provoke the report",
            vocab.canvas_event
        ));
        let probe = session.frame()?.layout_probe_point();
        driver.click_at(probe)?;
        session.settle(10);
        session.trace()?
    };

    // --- aim, in document space -------------------------------------------
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    report.note(format!(
        "canvas rect {:?} at zoom {:.3}",
        mapping.image_rect, mapping.zoom
    ));
    let window_point = mapping.doc_to_window(target)?;
    let frame = session.frame()?;
    let screen_point = frame.to_screen(window_point);
    report.note(format!(
        "document point (page {}, {:.1}, {:.1}) → window ({:.1}, {:.1}) → screen ({}, {}) \
         at DPI scale {:.2}",
        target.page,
        target.x,
        target.y,
        window_point.x(),
        window_point.y(),
        screen_point.x(),
        screen_point.y(),
        frame.scale
    ));

    // How many click events the layout probe already produced. Everything from
    // here on must read only the events the REAL click adds — reading `last`
    // without this would silently report the probe's result as the check's
    // result on any run where the real click produced no event at all, which
    // is the most misleading possible outcome.
    let clicks_before = trace.events(vocab.click_event).count();

    // --- step 1: click it, through the OS ----------------------------------
    driver.click_at(screen_point)?;
    session.settle(12);
    let after_click = session.trace()?;

    // --- step 2: the PRECONDITION assertion --------------------------------
    //
    // Two different negative outcomes, and they are NOT the same verdict.
    let click_line = after_click.events(vocab.click_event).nth(clicks_before);
    let hits = click_line.and_then(|l| l.get_usize(vocab.click_hits_field));
    let selected = click_line
        .and_then(|l| l.get_usize(vocab.click_selection_field))
        .or_else(|| {
            after_click
                .last(vocab.canvas_event)
                .and_then(|l| l.get_usize(vocab.canvas_selection_field))
        });

    match (hits, selected) {
        (None, None) => {
            // The honest SKIP, and the wording matters more than usual.
            //
            // The new application does NOT emit `sel=` on its canvas line, and
            // that is a decision rather than an oversight: there is no
            // selection subsystem at S2, and a binary that emitted `sel=0`
            // would satisfy the `(_, Some(0))` arm below — turning this into a
            // FAIL that reads "selection is not taking the hit test's result"
            // about code nobody has written. An absent field is the correct
            // way to say "no answer"; a zero is a wrong answer.
            //
            // So the reason says what was NOT found and stops. It does not
            // assert that the application lacks a selection subsystem (this
            // check cannot see the difference between "not implemented" and
            // "implemented and silent"), and it does not assert that anything
            // is broken.
            return Err(crate::error::Error::new(format!(
                "the click produced no `{}` event and no `{}=` field on `{}`, so the harness \
                 has no way to tell whether anything was selected — and without that \
                 precondition, a Delete that removes nothing is a mystery rather than a \
                 defect. Two readings, and this check cannot distinguish them: the binary has \
                 no selection subsystem yet and is correctly saying nothing about one, or it \
                 has one that does not report. Note that a build reporting `{}=0` instead of \
                 staying silent would be read as `the selection refused the hit`, which would \
                 be a FAIL against whichever of those two is the truth. Trace: {}",
                vocab.click_event,
                vocab.canvas_selection_field,
                vocab.canvas_event,
                vocab.canvas_selection_field,
                session.trace_path().display()
            )));
        }
        (Some(0), _) => {
            // The retracted-false-defect case, refused on purpose. A hit test
            // that finds nothing means EITHER the point is not on an object OR
            // hit testing is broken, and the harness genuinely cannot tell
            // those apart from here.
            return Err(crate::error::Error::new(format!(
                "the hit test found nothing at document point ({:.1}, {:.1}) on page {}. \
                 That is either a --doc-point that is not on an object, or a broken hit \
                 test, and this harness cannot distinguish them — so it declines to file \
                 either. Check the point against the fixture, then re-run. (This refusal \
                 exists because a stale coordinate is symptom-identical to a broken \
                 coordinate conversion, and that confusion has already produced one \
                 filed-then-retracted defect in this codebase.)",
                target.x, target.y, target.page
            )));
        }
        (_, Some(0)) => {
            // The hit test DID find something and the selection did not take
            // it. Unambiguous, and a genuine assertion failure.
            return Ok(Some(format!(
                "the click hit {} object(s) at document point ({:.1}, {:.1}) and the \
                 selection is still empty. Selection is not taking the hit test's result.",
                hits.unwrap_or(0),
                target.x,
                target.y
            )));
        }
        _ => {}
    }
    let selected_n = selected.unwrap_or(0);
    report.note(format!(
        "precondition holds: the click hit {} object(s) and the selection is {} object(s)",
        hits.map_or_else(|| "?".to_owned(), |h| h.to_string()),
        selected_n
    ));

    // --- choose the oracle -------------------------------------------------
    //
    // Two are available in principle and they are not equally good.
    //
    // The page object count measures **the property the check is about**: the
    // question is "did Delete remove the object", and a count answers it
    // directly, before and after, with no inference. It is also robust to a
    // deletion performed by some future verb nobody remembered to trace — it
    // measures the page, not the code path.
    //
    // The `delete-objects` event measures **the verb that was meant to change
    // the property**. Its absence is the weaker evidence this module's header
    // admits exactly once, under three conditions, and only because the old
    // binary has nothing better.
    //
    // So: prefer the count, fall back to the event, and — this is the part
    // that costs nothing and saves an argument — say in the failure text which
    // one was used. A reader who knows the verdict came from a count does not
    // have to wonder whether a trace call site was simply missing.
    let before_count = vocab.object_count(&after_click);
    if let Some(n) = before_count {
        report.note(format!(
            "oracle: the page object count, which is {n} before the key. This measures the \
             property the check is about rather than the verb meant to change it."
        ));
    } else {
        report.note(format!(
            "oracle: the `{}` event. This binary reports no page object count{}, so the \
             evidence is the presence or absence of the deletion event — weaker, and \
             admissible only because the preconditions above have been established.",
            vocab.delete_event,
            match vocab.object_count_event {
                Some(e) => format!(" (no `{e}` line in the trace)"),
                None => " (no such event in its vocabulary)".to_owned(),
            }
        ));
    }
    let deletes_before = after_click.events(vocab.delete_event).count();

    // --- step 3: press Delete, through the OS ------------------------------
    driver.press(vk::DELETE)?;
    session.settle(12);
    let after_delete = session.trace()?;

    // --- step 4: the POSTCONDITION assertion -------------------------------
    if let (Some(before), Some(event)) = (before_count, vocab.object_count_event) {
        let Some(after) = vocab.object_count(&after_delete) else {
            // Not a SKIP. The binary answered this question a moment ago and
            // has stopped answering it, which is a change of behaviour across
            // the keystroke rather than a missing capability — and the
            // application's contract says a page it cannot count produces
            // `{event}-unavailable`, so this is worth reading as a finding.
            return Ok(Some(format!(
                "ORACLE: the page object count. The binary reported `{event}` before the key \
                 ({before} objects) and no longer reports a readable one after it, so the \
                 count could not be compared. Look for an `{event}-unavailable` line in the \
                 trace: the application emits that instead of a count when a page will not \
                 decompose, and a page that decomposed before the key and not after it is \
                 itself the news."
            )));
        };
        report.note(format!("page object count after Delete: {after}"));
        if after >= before {
            return Ok(Some(format!(
                "ORACLE: the page object count — measured before and after the keystroke, \
                 which is the property this check is about rather than the verb meant to \
                 change it. Delete removed nothing: the count is {after}, and it was {before} \
                 before the key. {}",
                D1_DIAGNOSIS
            )));
        }
        report.note(format!(
            "the page object count fell from {before} to {after} across the keystroke"
        ));
        return Ok(None);
    }

    // No object-count event. Fall back to the deletion event — admissible as
    // an absence for the three reasons in the module documentation, all of
    // which have now been established above.
    let deletes_after: Vec<&crate::trace::TraceLine> =
        after_delete.events(vocab.delete_event).collect();
    if deletes_after.len() <= deletes_before {
        return Ok(Some(format!(
            "ORACLE: the absence of a `{}` event — this binary reports no page object count, \
             which would have been the better evidence. Delete removed nothing. The harness \
             clicked a document point, the hit test found {} object(s), the selection became \
             {selected_n}, and the harness then pressed Delete as a real keystroke into the \
             foreground window — and the application traced no `{}` event at all. The \
             deletion path exists in this binary and traces unconditionally when it runs, so \
             its silence means the path was never entered. {}",
            vocab.delete_event,
            hits.map_or_else(|| "?".to_owned(), |h| h.to_string()),
            vocab.delete_event,
            D1_DIAGNOSIS
        )));
    }

    let deleted: usize = deletes_after
        .iter()
        .skip(deletes_before)
        .filter_map(|l| l.get_usize(vocab.delete_count_field))
        .sum();
    report.note(format!(
        "traced deletions after the key: {deleted} object(s)"
    ));
    if deleted == 0 {
        return Ok(Some(format!(
            "ORACLE: the `{}` event. The application traced one, reporting zero objects \
             removed. {}",
            vocab.delete_event, D1_DIAGNOSIS
        )));
    }
    Ok(None)
}

/// The sentence appended to every D1 failure.
///
/// A failure report that only says "Delete did nothing" sends the reader to
/// the deletion code, which is correct and has been correct throughout. Naming
/// the actual guard saves that trip — and the fix really is one line.
// typing-guard-exempt: PROSE, not a call. This names the predicate inside a
// multi-line failure message, and the gate cannot see that a continuation
// line belongs to a string rather than to code - a limitation worth having
// over the alternative, which is a gate that tries to parse Rust in bash and
// gets it subtly wrong in the direction of letting a real call through.
const D1_DIAGNOSIS: &str = "This is the D1 signature. Look at the keyboard guard, not at the \
     deletion code: the canvas requests egui focus on click, and a guard written as \
     `ctx.egui_wants_keyboard_input()` means 'any widget has focus', not 'a text field has \
     focus'. `ctx.text_edit_focused()` is the predicate that was intended. Check \
     PageDown/PageUp and Home/End too — they are suppressed by the same guard from the same \
     click.";
