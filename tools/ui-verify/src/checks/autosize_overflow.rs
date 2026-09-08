//! `a_field_too_small_for_its_text_says_so` — the one auto-size outcome that
//! is not an answer, driven through the real window.
//!
//! # What this asserts, and why the size alone was never enough
//!
//! A PDF text field can ask the viewer to pick the point size (`/DA … 0 Tf`).
//! `pdfcer-core` picks one and reports **which constraint decided it**:
//! `AutoFitBound::Height`, `::Width`, or `::Floor`. The first two mean *it
//! fits*. The third does not — the engine's own comment at the branch that
//! returns it reads *"the one case where the returned size does NOT fit the
//! constraint that produced it"* — so the text is going to spill out of the
//! box, and pdfcer stopped shrinking only to keep it readable.
//!
//! Until 2026-09-07 this shell read the chosen **size** and threw the **bound**
//! away, so the operator got:
//!
//! > *"⚠ "FullName" asks for an automatic text size and pdfcer chose 4.0 pt.
//! > Another program filling this field may choose differently."*
//!
//! A sentence about interoperability, when the fact was that his text does not
//! fit. ⚠ Meanwhile `OPERATOR_REQUESTS.md` **O86** told him, under a ✅, that
//! *"pdfcer now tells you which way it decided … the box is too small for this
//! text, which will overflow"* — true of the engine and the CLI, and **false of
//! this shell for three days**.
//!
//! # ★★★ Why this check exists when two other tests already cover it
//!
//! Because neither of them is the operator.
//!
//! | test | proves | cannot see |
//! |---|---|---|
//! | `app::status::tests` | the three sentences differ and only one claims overflow | that anything ever *reaches* the overflow arm |
//! | `tests/autosize_floor_says_it_will_overflow.rs` | a real fill through `EditSession` reports `Floor`, and the sentence is built from it | that a **click and a keystroke** get there, or that the bar draws it |
//!
//! This project's standing lesson is that a chain can be green at every link
//! and broken end to end — *"eight green tests while the feature did 1 of 14"*.
//! So this check presses the keys.
//!
//! # The trace oracle, and why the application had to grow one
//!
//! `form-fill-text` carried `commands=` and `epoch=` and nothing else, so a
//! build that dropped the bound produced a **byte-identical** trace line to one
//! that honoured it. `panels::forms::edit` now appends `autosize=` and
//! `bound=`, for `place.rs`'s standing reason: *a trace line must carry the
//! number a wrong build would get wrong.*
//!
//! ★ `bound=` is spelled by this shell's own `bound_token`, never `{:?}`.
//! `Debug` is another crate's unstable rendering; a check keyed on it goes
//! quiet — or reports the opposite of the truth while quoting the truth — the
//! day upstream renames a variant.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The mode this drives. **Read**, deliberately.
///
/// ★★ Filling a form is not editing a document, and this shell has said so
/// since canvas filling landed: a field is fillable in Read mode because
/// filling in a reading stance is what forms are *for*. Driving it here rather
/// than in Edit also proves the disclosure reaches the one mode whose dock
/// does not mount the Forms panel — which is the whole argument for putting the
/// sentence on the status bar rather than in the panel alone.
const MODE: &str = "read";

/// `form-box page=… field=… widget=… kind=… rect=(x,y)+(w,h)` — the
/// application's own census of where it drew each widget.
const BOX_LINE: &str = "form-box";

/// `form-focus page=… field=… widget=…` — a click landed in a field.
const FOCUS_LINE: &str = "form-focus";

/// `form-fill-text commands=… epoch=… autosize=… bound=…`.
const FILL_LINE: &str = "form-fill-text";

/// The bound token that means *the text does not fit*.
const FLOOR: &str = "floor";

/// The status bar's fill-disclosure group, published by
/// `app::status::disclosure::disclosure_line` through `diag::ui_rect`.
///
/// ★★ Asserting this is what turns the check from *"the engine reported a
/// floor"* into *"the operator was told"*. The bound reaching the trace proves
/// the value survived the shell's own plumbing; it does **not** prove the bar
/// drew anything, and a build whose status row never called `fill_disclosure`
/// would satisfy every assertion above it.
///
/// ⚠ It proves the line was DRAWN, not which sentence it holds. The wording is
/// pinned by `app::status::tests` and by
/// `tests/autosize_floor_says_it_will_overflow.rs`; a pixel oracle for the
/// glyphs would be the only way to close that last inch and is not worth it
/// here, because the branch that chooses the sentence is a `match` on the very
/// value this check has already read out of the trace.
const BAR_REGION: &str = "status-group:fill-disclosure";

/// `Ctrl`, `A`, `Enter` as Windows virtual keys.
const VK_CONTROL: u16 = 0x11;
const VK_A: u16 = 0x41;
const VK_RETURN: u16 = 0x0D;

/// The value typed in.
///
/// Long enough that the **width** bound drives the size under the engine's 4 pt
/// legibility floor, and the length is **measured rather than reasoned**:
///
/// a first draft of 84 characters drove this fixture to `autosize=5.1
/// bound=width` — close, and not there. The chosen size scales inversely with
/// the text's width, so the 4.0 pt floor needs about `84 x 5.1 / 4.0 = 107`
/// characters; this is 132, which leaves room for a font-metric change without
/// leaving so much that the check stops resembling anything an operator types.
///
/// ★ That first run is why the SKIP arm below quotes `autosize=` as well as
/// `bound=`. *"It came out at 5.1"* names the next edit; *"it was not floor"*
/// does not.
///
/// ★ Letters and spaces only, and the constraint is the instrument rather than
/// the subject: `type_ascii` refuses punctuation because `-`, `.` and `/` are
/// `VK_OEM_*` codes whose meaning is keyboard-layout specific, so a check typing
/// them would pass here and type something else on another machine. A hyphen in
/// this name would have been a silent layout dependency for no gain.
///
/// ★ Plain ASCII on purpose. `type_ascii` sends key events, and a check that
/// also exercised the `WinAnsi` substitution path would be testing two
/// disclosures at once — and the other one has its own sentence, which would
/// then be concatenated onto this one and break the assertion for a reason that
/// has nothing to do with auto-size.
const TOO_LONG: &str = "Alexandra Christina Wetherby Fitzgerald of the Northern Districts \
                        Planning and Development Authority Second Floor East Wing";

/// See the module documentation.
pub struct AFieldTooSmallForItsTextSaysSo;

impl Check for AFieldTooSmallForItsTextSaysSo {
    fn name(&self) -> &'static str {
        "a_field_too_small_for_its_text_says_so"
    }

    fn defect(&self) -> &'static str {
        "typing into an auto-sized field whose box is too small for the text reports only the \
         point size pdfcer picked — so the operator is told a number that reads like a decision, \
         and finds out the text overflowed the box by looking at the printed sheet"
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

/// One `form-box` census line, parsed back into a canvas-space centre.
///
/// ★★ The application's numbers, not the fixture's — the same rule
/// `form_field`'s `placed_boxes` states: a check that computed the rect from
/// the PDF would be asserting that two independent derivations agree, and would
/// report a disagreement as a hit-test failure.
/// One widget, as the application says it drew it.
///
/// ⚠ `width` is carried rather than re-derived. The first draft of the SKIP
/// message below printed `cx * 2.0` as "the box the application drew N pt
/// wide", which is the CENTRE doubled and was wrong by the box's page offset —
/// it reported 330 pt for a 190 pt box. A diagnostic that invents a number is
/// how a correct check sends the next reader to look at the wrong thing.
struct PlacedBox {
    page: usize,
    field: String,
    /// Canvas-space centre — what the click aims at.
    centre: (f64, f64),
    /// Canvas-space width, for the SKIP message's arithmetic.
    width: f64,
}

fn first_box(trace: &Trace) -> Option<PlacedBox> {
    trace.events(BOX_LINE).find_map(|l| {
        let page: usize = l.get("page")?.parse().ok()?;
        let field = l.get("field")?.to_owned();
        let raw = l.get("rect")?;
        let (min, size) = raw.split_once(")+(")?;
        let (x, y) = min.trim_start_matches('(').split_once(',')?;
        let (w, h) = size.trim_end_matches(')').split_once(',')?;
        let x: f64 = x.trim().parse().ok()?;
        let y: f64 = y.trim().parse().ok()?;
        let w: f64 = w.trim().parse().ok()?;
        let h: f64 = h.trim().parse().ok()?;
        Some(PlacedBox {
            page,
            field,
            centre: (x + w / 2.0, y + h / 2.0),
            width: w,
        })
    })
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check needs a document with an AUTO-SIZED text field — one whose \
             /DA says `0 Tf`. Pass fixtures/autosize-field.pdf; its .PROVENANCE.py explains how \
             it was made and why the byte offsets survive.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a field and types into it. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("autosize_overflow.trace.txt"));
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
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(40);

    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- Where the application says the field is ---------------------------
    let trace = session.trace()?;
    let Some(placed) = first_box(&trace) else {
        return Ok(Some(format!(
            "★★ NO `{BOX_LINE}` LINE, so the application drew no fillable widget.\n\
             This fixture has exactly one, and Read mode is supposed to offer it — filling in a \
             reading stance is what forms are for. Either the widget census did not run, or \
             this document's field was classified unfillable. Check `canvas::forms::classify` \
             and `block_reason`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let (box_page, field) = (placed.page, placed.field.clone());
    let (cx, cy) = placed.centre;
    report.note(format!(
        "the application drew field {field:?} on page {box_page}, {:.1} pt wide, centred at \
         canvas ({cx:.1}, {cy:.1})",
        placed.width
    ));

    // The census is canvas space (y down from the page top); `doc_to_window`
    // takes PDF space (y up from the bottom). One flip, and it is the mapping's
    // own formula read backwards — the same conversion `form_field` documents.
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, box_page)?;
    let doc_y = page.height_pt - cy;
    let point = mapping.doc_to_window(DocPoint::new(box_page, cx, doc_y))?;
    let frame = session.frame()?;

    driver.click_at(frame.to_screen(point))?;
    session.settle(25);

    let trace = session.trace()?;
    let Some(focus) = trace.last(FOCUS_LINE) else {
        return Ok(Some(format!(
            "★★ THE CLICK ON THE FIELD PLACED NO CARET: no `{FOCUS_LINE}` line.\n\
             The application published where the box was and this check aimed at the centre of \
             it, so a miss here is a hit-test failure rather than an aim failure. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the click focused the field: `{}`", focus.raw));

    // --- Replace the value, and commit -------------------------------------
    //
    // ★ Select-all first. The fixture ships with `(Ada)` in the field, and
    // typing without clearing it would append — which still overflows, so the
    // check would PASS while proving something weaker than it claims.
    driver.press_chord(&[VK_CONTROL], VK_A)?;
    session.settle(8);
    driver.type_ascii(TOO_LONG)?;
    session.settle(20);

    // ★★★ The anchor goes HERE — after the typing, immediately before the
    // gesture that commits. `last()` over the whole capture would be satisfied
    // by any earlier fill, and this check's own setup does not perform one
    // today; that is a fact about today's setup, not a property to rely on.
    let mark = session.trace()?.mark();

    driver.press(VK_RETURN)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(fill) = trace.last_after(FILL_LINE, mark) else {
        return Ok(Some(format!(
            "★★ THE COMMIT WROTE NOTHING: no `{FILL_LINE}` line after the Enter.\n\
             The field was focused (proved above) and {} characters were typed, so this is \
             between the commit gesture and `panels::forms::edit::apply`. A `form-commit \
             outcome=unchanged` line here would mean the keystrokes never reached the field's \
             buffer. Trace: {}.",
            TOO_LONG.len(),
            session.trace_path().display()
        )));
    };
    report.note(format!("the commit applied: `{}`", fill.raw));

    // --- The assertion ------------------------------------------------------
    let Some(bound) = fill.get("bound") else {
        return Ok(Some(format!(
            "★★★ `{FILL_LINE}` CARRIES NO `bound=` FIELD: `{}`.\n\
             That field is this check's whole oracle and it is what makes a build that honours \
             the auto-size bound distinguishable from one that discards it — the two were \
             byte-identical traces before 2026-09-07. Restore it in \
             `panels::forms::edit::apply`. Trace: {}.",
            fill.raw,
            session.trace_path().display()
        )));
    };

    if bound != FLOOR {
        // ⚠ This is a fact about the FIXTURE or the engine's arithmetic, not
        // about the disclosure — so it is a SKIP with the value attached,
        // never a silent pass. The check has not seen its subject.
        return Err(Error::new(format!(
            "the fill reported `bound={bound}`, not `{FLOOR}`, so the text this check typed did \
             not overflow the field and the disclosure under test was never reached. SKIPPED \
             rather than passed. {} characters went into a box the application drew {:.1} pt \
             wide. If the engine's legibility floor or its padding moved, lengthen `TOO_LONG` \
             using the arithmetic on it — do NOT relax this. Line: `{}`. Trace: {}.",
            TOO_LONG.len(),
            placed.width,
            fill.raw,
            session.trace_path().display()
        )));
    }

    report.note(format!(
        "★★★ the engine reported bound={FLOOR} — the chosen size does NOT fit, and the text \
         will overflow: `{}`",
        fill.raw
    ));

    // --- And the operator was actually told ---------------------------------
    //
    // ★★★ The bound reaching the trace is the shell's plumbing working. This is
    // the part that is about the OPERATOR: the status bar drew its
    // fill-disclosure group, in Read mode, where the Forms panel is not
    // mounted — which is the whole reason the sentence lives on the bar rather
    // than in the panel alone.
    // ★★ `name=`, not `region=`. The first draft of this read `region=` and
    // FAILED against a build that was working perfectly — the bar had drawn the
    // line and the check was asking the wrong key. Recorded because that is the
    // commonest way a driven check invents a defect: `get()` on a missing key
    // returns `None`, which is indistinguishable from the value being absent.
    //
    // ★ Anchored past `mark` as well, so this cannot be satisfied by a
    // disclosure some earlier gesture left on the bar.
    let drawn = trace
        .events(ui_rect)
        .any(|l| l.lineno > mark && l.get("name") == Some(BAR_REGION));
    if !drawn {
        return Ok(Some(format!(
            "★★★ THE ENGINE REPORTED bound={FLOOR} AND THE STATUS BAR SAID NOTHING.\n\
             `{fill_raw}` proves the value reached this shell, so the disclosure was computed \
             and then not drawn — no `{BAR_REGION}` region in the whole capture.\n\
             Look at `app::status::disclosure::fill_disclosure`: it returns early when \
             `last_fill_disclosure(doc.edit_epoch)` is `None`, and that is keyed on the epoch \
             AFTER the fill, so an off-by-one there silences the sentence with everything else \
             still green. This is the exact failure mode the check was extended to catch. \
             Trace: {}.",
            session.trace_path().display(),
            fill_raw = fill.raw
        )));
    }
    report.note(format!(
        "★★ and the status bar drew it — `{BAR_REGION}` is in the capture, in Read mode, where \
         the Forms panel is not mounted"
    ));
    Ok(None)
}
