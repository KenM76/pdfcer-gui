//! `clicking_text_offers_its_colour` — **O89's object route, driven.**
//!
//! ⚠⚠⚠ **THIS CHECK HAS NOT BEEN RUN.** It was written on 2026-09-05 in a
//! session whose instructions forbade launching the GUI and forbade running
//! `ui-verify`, because the harness takes the whole desktop and the operator
//! may be at his keyboard. It compiles, it is registered, and **no line below
//! has been observed against a running binary.** Do not read a green suite as
//! evidence about this check until somebody has watched it fail against a build
//! with the object route removed — the falsification table at the foot of this
//! header says exactly how.
//!
//! # The defect
//!
//! `OPERATOR_REQUESTS.md` **O89**, in his words:
//!
//! > *"I don't see where I am able to edit the color of text, vectors, etc."*
//!
//! Text colour had shipped in two places and **both were gated on a swept
//! range**. Clicking a piece of text with the Select tool selects the *object*,
//! so both controls stayed greyed and the way to un-grey them — press `T`, arm
//! the Text tool, sweep across the words — is not guessable from anything on
//! screen. The capability was there; the route was not.
//!
//! `panels::properties::textobject` is the route: a **working colour control on
//! the clicked object**, whose operand is derived from the object's own
//! `BT`…`ET` byte span rather than from any guess at geometry.
//!
//! # ★★★ Why this check is not a subset of `font_group`
//!
//! `font_group`'s phase 1 asserts that the panel drew a **sentence** —
//! `properties.text.route` — telling the operator to press `T`. That sentence
//! still exists and this check asserts it too, in the same state, because it is
//! still the route to the four controls the object state does not offer.
//!
//! What is new, and what only this check covers, is that in that same state
//! there is now **a control that works**. A build that kept the sentence and
//! lost the control passes `font_group` completely and is the exact program the
//! operator complained about.
//!
//! # The oracle, in the order it is read
//!
//! | # | assertion | what its absence means |
//! |---|---|---|
//! | 0 | the click left **one text object** selected | the harness aimed wrong — **SKIP**, never fail |
//! | 1 | `properties.textobject` drew | the section is gone, or returned before drawing |
//! | 2 | `properties.textobject.swatch` **or** `properties.textobject.ink` drew | the colour row drew neither a control nor its refusal |
//! | 3 | `properties.text.route` drew | the sentence to the other four controls was lost with the move |
//! | 4 | clicking the swatch opened `properties.textobject.swatch.picker` | the control is drawn and inert — this project's founding defect |
//! | 5 | a pick then a close traced `text-style-applied` **and** `format-text` | the gesture decided to act and the action never reached the engine |
//!
//! ★★ **4 and 5 are two assertions and not one**, for the reason
//! `restyle_text`'s own header gives about its pair: a control that opens a
//! picker and never commits, and a control that never opens, are different
//! defects with different fixes, and one message covering both would name
//! neither.
//!
//! ★★★ **Step 2 is a disjunction on purpose, and it is not a weakened
//! assertion.** Which of the two draws is a fact about the *fixture*, not about
//! the program: text painted in CMYK or a spot colour must get the sentence and
//! no swatch, and text in RGB or Gray must get the swatch. Requiring the swatch
//! unconditionally would make this check FAIL on a correctly-behaving build over
//! a spot-inked drawing — which is precisely the class of false red
//! `RESUME.md` records four separate instances of. Which one appeared is
//! recorded as a note, and step 4 runs **only** when it was the swatch: there is
//! nothing to click when the program has correctly refused to draw a control.
//!
//! # ★ The aim
//!
//! Needs a `--doc-point` on a real run of text. `RESUME.md`'s aim table gives
//! `D:/Dev/pdfTests/SW41177/SW41177.pdf` at `0,1140,62` for the text family,
//! and that is this check's calibration point: a 5 pt title-block run at PDF
//! (1135.7, 58.4)–(1190.5, 63.4). Anything else and step 0 skips.
//!
//! # ★★★ THE FALSIFICATION TABLE — what to break, and what must go red
//!
//! Written here because the check has not been run and the next session has to
//! be able to prove it is not vacuous without re-deriving how.
//!
//! | plant | expected failure |
//! |---|---|
//! | `panels::properties::textobject::section` returns `false` immediately | step 1: no `properties.textobject` region |
//! | `classify` always returns `Colour::Ink { .. }` | step 2 records the ink sentence, step 4 is skipped — so **also** plant a text fixture in RGB, or the plant is invisible |
//! | the `Colour::Agreed`/`Mixed` arms draw a plain `ui.label` instead of the swatch | step 2 passes on the label's region? **No** — the label publishes no region, so step 2 fails. That is why the two states publish two names |
//! | `swatch::show` never opens the popup | step 4: no `.picker` region |
//! | `swatch::show` returns `None` unconditionally | step 5: no `text-style-applied` |
//! | the section raises no `Action::TextStyle` | step 5: `text-style-applied` present, `format-text` absent — the two-line oracle earning its keep |

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The only mode whose canvas selects page content, and the only one this
/// section is reachable in.
const MODE: &str = "edit";
/// The section's own region.
const SECTION: &str = "properties.textobject";
/// The colour control.
const SWATCH: &str = "properties.textobject.swatch";
/// The picker the control opens.
const PICKER: &str = "properties.textobject.swatch.picker";
/// The sentence drawn INSTEAD of a control over an ink pdfcer will not
/// overwrite.
const INK: &str = "properties.textobject.ink";
/// The sentence naming the route to the four controls the object state does not
/// offer.
///
/// ★ Spelled `properties.text.route`, which is where it lived before the
/// section moved. The surface did not move; only the module that draws it.
const ROUTE: &str = "properties.text.route";
/// The Properties pane's tab header, so the pane can be brought to the front.
///
/// ★★ Not optional. The dock draws only the ACTIVE tab's body, so a pane behind
/// another tab publishes **nothing** — indistinguishable, from here, from a
/// panel with nothing to say. `font_group`'s own note records the false bug
/// report that cost.
const PROPERTIES_TAB: &str = "dock.tab.file.properties";
/// The Properties panel's report of the object it is describing.
const PANEL_EVENT: &str = "properties-panel";
/// The canvas's report of a selection change; `sel=` is how many entries.
const CANVAS_SELECTION_EVENT: &str = "canvas-selection";
/// `summary::ObjectKind::Text` under `{:?}`.
const TEXT_KIND: &str = "Text";
/// The restyle module's summary line.
const STYLE_EVENT: &str = "text-style-applied";
/// The restyle module's refusal line.
const DECLINED_EVENT: &str = "text-style-declined";
/// The label `vector_edit` writes when the restyle reached the engine.
///
/// ★★ The second half of the two-line oracle. `text-style-applied` alone says a
/// module decided to act; this says the act landed in the document. A build
/// where the two disagree is exactly the shape `RESUME.md` records for
/// `import-form-data`, twice.
const APPLIED: &str = "format-text";

/// See the module documentation.
pub struct ClickingTextOffersItsColour;

impl Check for ClickingTextOffersItsColour {
    fn name(&self) -> &'static str {
        "clicking_text_offers_its_colour"
    }

    fn defect(&self) -> &'static str {
        "an operator who clicks a piece of text to change its colour is offered nothing — the \
         Properties panel names a sweep they have no way to guess at, and the colour control \
         they came for stays out of reach behind it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        report.note(
            "⚠ THIS CHECK HAS NEVER BEEN RUN — written 2026-09-05 in a session forbidden to \
             launch the GUI. Read its module header's falsification table before trusting a \
             green result from it.",
        );
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// What the click left selected, as the answers that decide whether to proceed.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Aim<'a> {
    /// Exactly one object, and it is text.
    OneTextObject,
    /// No page-content object at all.
    NothingSelected,
    /// One object, not text.
    NotText(&'a str),
    /// More than one (or a count that did not parse as one).
    NotAlone(usize),
}

/// Read [`Aim`] out of a settled trace.
///
/// ★ Split from the wording so the READ is testable without a running program,
/// which is `font_group::aim_verdict`'s rule and its reason: a guard against a
/// harness misreading its own oracle is worth nothing if the guard itself can
/// only be exercised by driving the mouse.
///
/// ★★ Order is *what* before *how many*. A click that lands on a path inside a
/// marquee of eleven is an aim problem twice over, and the kind is the half
/// that names the fixture coordinate to change.
fn aim_verdict(trace: &Trace) -> Aim<'_> {
    let Some(kind) = trace.last(PANEL_EVENT).and_then(|line| line.get("kind")) else {
        return Aim::NothingSelected;
    };
    if kind != TEXT_KIND {
        return Aim::NotText(kind);
    }
    // ★ Absent reads as ZERO, not as one. `canvas-selection` is written through
    // `trace_changed`, so no line at all means the selection never changed —
    // after a click, that is a click that selected nothing, and a
    // defaulted-to-one guard would wave it through.
    let selected = trace
        .last(CANVAS_SELECTION_EVENT)
        .and_then(|line| line.get_usize("sel"))
        .unwrap_or(0);
    if selected == 1 {
        Aim::OneTextObject
    } else {
        Aim::NotAlone(selected)
    }
}

/// Turn [`Aim`] into `Ok(())` or into a SKIP that names the harness's own aim.
///
/// ★★ SKIPPED, never failed. A `--doc-point` that is not on text is the
/// harness's aim, and a harness that reports its own aim as the program's
/// behaviour is worse than one that reports nothing — `RESUME.md` records that
/// costing a day on `font_group`, about a program that was working.
fn aimed_at_one_text_object(session: &Session, trace: &Trace, target: DocPoint) -> Result<()> {
    let aim = format!(
        "the --doc-point (page {}, {:.1}, {:.1})",
        target.page, target.x, target.y
    );
    let path = session.trace_path().display();
    match aim_verdict(trace) {
        Aim::OneTextObject => Ok(()),
        Aim::NothingSelected => Err(Error::new(format!(
            "{aim} selected no page-content object, so this check's subject — a piece of TEXT \
             selected as an OBJECT — does not exist in this run. No `{PANEL_EVENT}` line. \
             SKIPPED, not failed: this says where the harness aimed. `pdfcer extract-text \
             --json` gives the first glyph's x and y of every run; `RESUME.md`'s aim table \
             gives 0,1140,62 on SW41177.pdf. Trace: {path}."
        ))),
        Aim::NotText(kind) => Err(Error::new(format!(
            "{aim} selected a `{kind}`, not text — `{PANEL_EVENT} … kind={kind}`. \
             `panels::properties::textobject::section` is right to stay silent over a path or a \
             picture, so every oracle below would be asserting a sentence about text over a \
             selection that is not text. SKIPPED, not failed. Trace: {path}."
        ))),
        Aim::NotAlone(n) => Err(Error::new(format!(
            "{aim} left {n} object(s) selected, and this section is deliberately \
             single-selection: a colour control over eleven shapes and one label has no single \
             subject. SKIPPED, not failed — aim at a text run no other object overlaps. \
             Trace: {path}."
        ))),
    }
}

/// Poll until the restyle reports one way or the other.
///
/// ★ A bounded poll rather than a fixed sleep, for `restyle_text`'s reason: a
/// restyle re-resolves its pin from a fresh provenance extraction **per run**,
/// and this route's operand is a whole text object, which can be many runs. A
/// fixed sleep long enough for the worst case makes every run slow, and a short
/// one reads the trace mid-gesture and reports *"nothing happened"* about a
/// gesture that is still running.
fn wait_for_verdict(session: &Session) -> Result<u128> {
    const CEILING_MS: u128 = 30_000;
    let started = std::time::Instant::now();
    loop {
        session.settle(4);
        let trace = session.trace()?;
        if trace.last(STYLE_EVENT).is_some() || trace.last(DECLINED_EVENT).is_some() {
            return Ok(started.elapsed().as_millis());
        }
        if started.elapsed().as_millis() > CEILING_MS {
            return Ok(started.elapsed().as_millis());
        }
    }
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a page carrying real text."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming the LEFT END of a piece of \
             text's baseline — `RESUME.md`'s aim table gives 0,1140,62 on SW41177.pdf. A point \
             on blank paper selects no object and the check would report the route as missing.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a page object, clicks a swatch, \
             clicks inside a colour picker and dismisses it, and none of that can be simulated \
             from the trace.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("colour-clicked-text.trace.txt"));
    spec.pdf = Some(pdf.clone());
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
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // The pane must be in FRONT or it publishes nothing. See [`PROPERTIES_TAB`].
    if let Some(tab) = declared(&session.trace()?, ui_rect, PROPERTIES_TAB) {
        driver.click_at(session.frame()?.declared_center(tab))?;
    }
    session.settle(20);

    // =======================================================================
    // Click the text as an OBJECT — the state O89 is about, and the state an
    // operator is in when they go looking for the colour.
    // =======================================================================
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let frame = session.frame()?;
    // ★ Two points in and two up from the baseline origin, not at it.
    // `--doc-point` names the first glyph's origin — the bottom-left corner of
    // the ink — and on a five-point label a click exactly there can land in the
    // paper beside it.
    let on_text = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        target.page,
        target.x + 2.0,
        target.y + 2.0,
    ))?);
    driver.click_at(on_text)?;
    session.settle(24);

    let trace = session.trace()?;
    aimed_at_one_text_object(&session, &trace, target)?;
    report.note("the click selected exactly one object and it is a text object");

    // --- 1. the section drew at all ----------------------------------------
    if declared(&trace, ui_rect, SECTION).is_none() {
        let shot = ctx.out("colour_clicked_text.no-section.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ CLICKING A PIECE OF TEXT OFFERS NOTHING ABOUT ITS COLOUR: no `{SECTION}` \
             region.\n\
             This is O89 in his own words — *\"I don't see where I am able to edit the color of \
             text\"* — and the precondition above has already ruled out the two innocent \
             explanations: exactly one object is selected and it IS text by \
             `summary::object_kind`, the same call the section gates on. What is left. (1) \
             `panels::properties::textobject::section` returned before drawing — it returns \
             `false` for a live text SELECTION (deliberately: the swept-text editor owns that \
             state), so a stale sweep from an earlier gesture would suppress it. (2) The \
             expensive read failed: `pin::object_text` answers `None` when the object's runs \
             cannot be pinned, in which case the section still draws the heading and the route \
             sentence — so this absence is NOT that. (3) The region was drawn and not declared: \
             `diag::ui_rect_visible` withholds a rect less than 60 % inside its clip, which is \
             what a Properties pane taller than its dock slot produces. The screenshot beside \
             this report settles (3) by eye. Regions declared: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "properties.")),
            session.trace_path().display()
        )));
    }
    report.note("★ the Properties panel drew a section about the clicked text");

    // --- 2. a control, or the refusal that stands in for one ----------------
    let swatch = declared(&trace, ui_rect, SWATCH);
    let ink = declared(&trace, ui_rect, INK);
    if swatch.is_none() && ink.is_none() {
        let shot = ctx.out("colour_clicked_text.no-colour-row.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ THE SECTION DREW AND ITS COLOUR ROW DREW NEITHER A CONTROL NOR A REFUSAL: \
             neither `{SWATCH}` nor `{INK}`.\n\
             Those two are the section's ONLY honest outcomes and they are exhaustive by \
             construction — `classify` returns `Agreed`, `Mixed` or `Ink`, the first two draw \
             the swatch and the third draws the sentence. A frame with neither means the match \
             grew a fourth arm that draws nothing, which is the placeholder shape R9 forbids: \
             an operator sees a heading, a count, and a blank where the colour should be. \
             Regions declared: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "properties.textobject")),
            session.trace_path().display()
        )));
    }
    // ★★★ WHICH one drew is a fact about the FIXTURE, not the program. Recorded
    // rather than asserted — see the module header on why requiring the swatch
    // unconditionally would make this check red against a correct build over a
    // spot-inked drawing.
    if swatch.is_some() {
        report.note("★★ the colour row drew a working swatch for the clicked text");
    } else {
        report.note(
            "the colour row drew the named-ink refusal instead of a swatch — this fixture's \
             text is painted in CMYK or a spot colour, which is the correct behaviour and not a \
             defect. Steps 4 and 5 are SKIPPED: there is nothing to click.",
        );
    }

    // --- 3. the route to the other four controls survived -------------------
    if declared(&trace, ui_rect, ROUTE).is_none() {
        return Ok(Some(format!(
            "★ THE COLOUR CONTROL IS THERE AND THE ROUTE TO THE OTHER FOUR IS GONE: no \
             `{ROUTE}` region.\n\
             `crate::text::panels::properties::text_object_route` names the Text tool and its \
             chord, and it is the ONLY surface in the program that tells an operator how to \
             reach the face, size, bold and italic controls for text they clicked. It moved \
             module on 2026-09-05 — from `panels::properties::text::route` to \
             `panels::properties::textobject` — keeping its region SPELLING precisely so this \
             assertion and `font_group`'s would keep working. An absence here most likely means \
             the move dropped the `ui_rect_visible` call. Regions declared: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "properties.")),
            session.trace_path().display()
        )));
    }
    report.note("★ the route to the face, size and weight controls is still on screen");

    let Some(swatch_rect) = swatch else {
        // The fixture's text is a named ink and the program correctly refused a
        // control. Everything this check can assert about this build has been
        // asserted.
        return Ok(None);
    };

    // --- 4. the control is not inert ---------------------------------------
    driver.click_at(session.frame()?.declared_center(swatch_rect))?;
    session.settle(24);
    let trace = session.trace()?;
    let Some(picker) = declared(&trace, ui_rect, PICKER) else {
        let shot = ctx.out("colour_clicked_text.picker-did-not-open.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ THE COLOUR SWATCH IS DRAWN AND PRESSING IT DOES NOTHING: no `{PICKER}` region \
             after a real click on `{SWATCH}`.\n\
             A control that is drawn, pressed, and does nothing is the defect class this whole \
             project was founded on, and it is worse here than usual: the operator has already \
             failed once to find this control, and the thing they finally found is inert. \
             `panels::properties::swatch::show` opens `egui::Popup::menu` on the button's \
             response with an id it owns; the popup body publishes this region. Candidates: the \
             popup id collided with another control's (the fill and line swatches on the paint \
             section share this widget and must not share an `id_salt`), or the click landed \
             outside the swatch's rect. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note("★★ the swatch opened a colour picker");

    // --- 5. picking a colour and closing reaches the document ---------------
    //
    // ★ Aimed at the LOWER part of the picker, which is where
    // `color_picker_hsva_2d` puts its saturation/value square — the largest
    // target in the popup and the one whose whole area changes the colour. The
    // upper part carries the preview button and the hue strip; a click on the
    // preview would open a nested popup and change nothing.
    let picker_click = session.frame()?.declared_at(picker, 0.35, 0.80);
    driver.click_at(picker_click)?;
    session.settle(16);
    // ★★★ CLOSE the picker, because THE CLOSE IS THE COMMIT. The widget
    // deliberately does not act on `.changed()` — `egui`'s colour button marks
    // itself changed on every frame of a drag, so committing there would author
    // one undo entry per frame. The whole gesture becomes one action when the
    // popup closes, and a check that never closed it would report the feature
    // as dead.
    // `Esc` — 0x1B. `PopupCloseBehavior::CloseOnClickOutside` would also close
    // it, but a click outside is a click on whatever is behind, and in a
    // Properties panel that is another control.
    driver.press(0x1B)?;
    let waited = wait_for_verdict(&session)?;
    report.note(format!("waited {waited} ms for the restyle to report"));

    let trace = session.trace()?;
    if trace.last(STYLE_EVENT).is_none() {
        let shot = ctx.out("colour_clicked_text.no-restyle.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        let declined = trace
            .last(DECLINED_EVENT)
            .map_or_else(|| "none".to_owned(), |l| format!("{l:?}"));
        return Ok(Some(format!(
            "★ A COLOUR WAS PICKED AND NO RESTYLE WAS EVEN ATTEMPTED: no `{STYLE_EVENT}` line.\n\
             The section raises `Action::TextStyle` with the object's run range and \
             `StyleChange::Fill`. Candidates, in order of likelihood. (1) \
             `panels::properties::swatch::show` never committed: it answers `Some` only on the \
             frame the popup CLOSES and only if `dirty` was set, so a click inside the picker \
             that landed on inert space leaves `dirty` false and the gesture is correctly a \
             no-op — which would be the harness's aim, not the program. (2) The run range came \
             back empty, in which case `textstyle::apply` records `NoRun`. Any \
             `{DECLINED_EVENT}` line: {declined}. Trace: {}.",
            session.trace_path().display()
        )));
    }
    if trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "★ THE RESTYLE RAN AND NOTHING REACHED THE DOCUMENT: `{STYLE_EVENT}` is present and \
             `{APPLIED}` is not.\n\
             ★★ This is the two-line oracle earning its keep, and it is the reason this check \
             does not stop at the first line: `{STYLE_EVENT}` says the module decided to act, \
             and `{APPLIED}` is the label `app::actions::apply::vector_edit` writes when the \
             edit actually reached `EditSession`. A build where the first appears without the \
             second has a restyle that ran, reported success, and changed no file — which is \
             exactly the shape `RESUME.md` records for `import-form-data`, twice, and the \
             second time it was written by the session that had just written the note. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★★ the colour reached the document through the object route");
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One text object selected, as a settled trace says it.
    ///
    /// ★ Written with `concat!` rather than as a multi-line literal, and it is
    /// not style: an indented continuation line inside a `"…"` keeps its
    /// leading spaces, `Trace::parse` strips the prefix from the **start** of
    /// the line, and the fixture silently becomes a trace with one line in it.
    /// That cost a red on the first run of these tests.
    const ON_ONE_TEXT_OBJECT: &str = concat!(
        "pdfcer-diag properties-panel object=4 kind=Text notes=0\n",
        "pdfcer-diag canvas-selection page=0 sel=1 level=Object first=object:4\n"
    );
    /// One PATH selected — the 2026-08-28 aim that cost `font_group` a day.
    const ON_A_PATH: &str = concat!(
        "pdfcer-diag properties-panel object=832 kind=Path notes=0\n",
        "pdfcer-diag canvas-selection page=0 sel=1 level=Object first=object:832\n"
    );
    /// A text object inside a selection of eleven.
    const ON_ELEVEN: &str = concat!(
        "pdfcer-diag properties-panel object=4 kind=Text notes=0\n",
        "pdfcer-diag canvas-selection page=0 sel=11 level=Object first=object:4\n"
    );
    /// The panel described a text object and the canvas never reported a
    /// selection at all.
    const PANEL_ONLY: &str = "pdfcer-diag properties-panel object=4 kind=Text notes=0\n";

    /// ★★★ The aim read reaches **all four** of its answers from a trace.
    ///
    /// The guard that stops this check reporting its own aim as the program's
    /// behaviour, tested without a running program — which is the only way it
    /// can be tested at all in a session forbidden to drive the GUI. If these
    /// four collapsed to one, the SKIP messages would be interchangeable and a
    /// reader would be sent to the wrong place.
    #[test]
    fn the_aim_read_tells_its_four_answers_apart() {
        let text_one = Trace::parse(ON_ONE_TEXT_OBJECT, "pdfcer-diag");
        assert_eq!(aim_verdict(&text_one), Aim::OneTextObject);

        let path = Trace::parse(ON_A_PATH, "pdfcer-diag");
        assert_eq!(aim_verdict(&path), Aim::NotText("Path"));

        let many = Trace::parse(ON_ELEVEN, "pdfcer-diag");
        assert_eq!(aim_verdict(&many), Aim::NotAlone(11));

        assert_eq!(
            aim_verdict(&Trace::parse("", "pdfcer-diag")),
            Aim::NothingSelected
        );
    }

    /// ★★ **Silence about the selection count reads as ZERO, not as one.**
    ///
    /// `canvas-selection` is written through `trace_changed`, so a run with no
    /// line is a run where the selection never changed. A guard that defaulted
    /// to one would pass on a click that selected nothing, and every oracle
    /// below it would then be asserting sentences about a selection that does
    /// not exist.
    #[test]
    fn a_missing_selection_line_is_not_a_selection_of_one() {
        let only_panel = Trace::parse(PANEL_ONLY, "pdfcer-diag");
        assert_eq!(aim_verdict(&only_panel), Aim::NotAlone(0));
    }
}
