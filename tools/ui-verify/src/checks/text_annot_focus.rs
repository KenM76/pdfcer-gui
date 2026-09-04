//! `text_annot_focus` — **the dialog takes the keyboard without being clicked.**
//!
//! The operator, 2026-08-18:
//!
//! > *"adding text does bring up a window and a prompt, but it doesn't type
//! > anything in the box when I type and nothing gets added."*
//!
//! # ★★ Why this exists when `text_annot_places_and_authors` already types
//!
//! Because that check **clicks the text field before typing into it**, and an
//! operator has no reason to. A dialog that opens with a caption asking for
//! words is asking to be typed into; nobody clicks a field that is already
//! showing a caret. So the existing check sets up the very state it is meant to
//! be testing, and passes on a build where the dialog never takes focus at all.
//!
//! That is the third time in this project a green assertion has pointed in the
//! right direction and measured the wrong thing, and it is worth naming the
//! shape rather than the instance: **a driven check that arranges the
//! precondition it is asserting about is not a driven check.** If a step in the
//! script is one the operator would not perform, it belongs in the assertion,
//! not in the setup.
//!
//! # The defect it was written against
//!
//! `dialogs::textannot::field` latched on having **asked** for focus:
//!
//! ```ignore
//! if !self.focused_once {
//!     response.request_focus();
//!     self.focused_once = true;
//! }
//! ```
//!
//! Asking and holding are different facts. The dialog's first draw is the frame
//! *after* the gesture that opened it — the canvas raises `BeginTextAnnot` and
//! the action queue drains at frame end — so the pointer release that finished
//! the drag is still being resolved around the request, and egui keeps the
//! earlier of two requests made in one pass. The one attempt was spent on the
//! frame most likely to lose it, and nothing ever asked again.
//!
//! # The oracle, and why it is indirect on purpose
//!
//! Nothing traces the draft's length, and adding a trace for it would be adding
//! a seam to observe a thing the operator observes directly. So this asks the
//! question the way they do: **Accept is greyed while the field is empty**
//! (`kind.uses_gallery() || !text.trim().is_empty()` — `dialogs::textannot`),
//! so clicking Accept after typing either authors an annotation or does
//! nothing, and those two outcomes are exactly "the keystrokes landed" and
//! "they did not".
//!
//! Read the failure message before believing a green: an authored annotation
//! proves the field had focus, which is the whole claim.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | Review mode, Markup tab, arm **Text box** | `markup-tool tool=TextAnnot(..)` |
//! | B | drag a box on the page | `text-annot-open`, `dialog:text-annot` declared |
//! | C | **type immediately — no click on the field** | nothing traced; the draft is not observable |
//! | D | click Accept | `add-text-annot`, one more than before |

use crate::checks::driving::{
    SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, frame_of, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The placed box, in PDF points. `text_annot`'s constant and its reasoning.
const BOX_PT: f64 = 220.0;

/// See the module documentation.
pub struct TextAnnotTakesTheKeyboardUnclicked;

impl Check for TextAnnotTakesTheKeyboardUnclicked {
    fn name(&self) -> &'static str {
        "text_annot_takes_the_keyboard_unclicked"
    }

    fn defect(&self) -> &'static str {
        "the text-annotation dialog opens and does not hold the keyboard, so an operator who \
         types straight into it — which is what the window is asking them to do — gets \
         nothing, and Accept stays greyed"
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
            "input is disabled (--no-input). This check clicks a ribbon control, drags on the \
             canvas and types. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. This check needs somewhere on the page to draw the box, and a \
             guessed one can land off the sheet — which is symptom-identical to a drag that \
             never registered.",
        )
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("text_annot_focus.trace.txt"));
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

    // --- A: Review mode, Markup tab, arm the text box ----------------------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, "review")?;
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.markup").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.markup` region after switching to Review. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some("markup"))
    {
        return Err(Error::new(
            "the click on the Markup tab produced no tab-selected line, so nothing below \
             would mean anything.",
        ));
    }

    let trace = session.trace()?;
    let item = declared(&trace, ui_rect, "ribbon.item.markup.text_box").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.item.markup.text_box` region on the Markup tab. Items declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.item.markup."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(14);
    let trace = session.trace()?;
    if !trace
        .events("markup-tool")
        .any(|l| l.get("tool").is_some_and(|t| t.contains("TextAnnot")))
    {
        return Err(Error::new(
            "Markup > Text box armed nothing, so the drag below would draw no box and the \
             dialog under test would never open.",
        ));
    }
    report.note("Markup > Text box armed the text-annotation tool");

    let before = annot_count(&session);

    // --- B: capture the paper, then drag the box ---------------------------
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    let frame = session.frame()?;
    let corner = mapping.doc_to_window(target)?;
    let far = mapping.doc_to_window(DocPoint {
        page: target.page,
        x: target.x + BOX_PT,
        y: target.y + BOX_PT,
    })?;
    // The box in the window's own logical coordinates, WITH its border.
    //
    // ★ A first cut inset past the border, reasoning that `spec` draws one
    // unconditionally so an empty box would pass a test that sampled the edge.
    // That was the wrong trade and it produced a false FAIL: at fit-page zoom on
    // a C-size sheet the whole annotation is 65 device pixels and its text is
    // three, so the inset removed the border AND most of the glyphs, and the
    // check reported "authored and not visible" about a build that draws it
    // correctly. A harness that calls a working feature broken is worse than one
    // that says nothing, because it sends the next reader to the wrong file.
    //
    // The empty-box case it was guarding against is already covered one step
    // earlier, and better: Accept is greyed while the field is empty, so an
    // annotation existing at all proves the keystrokes landed. This assertion
    // answers the remaining question — *did anything reach the page* — and a
    // border is a perfectly good answer to it.
    let box_rect = LRect::new(
        Pt::new(corner.x().min(far.x()), corner.y().min(far.y())),
        Pt::new(corner.x().max(far.x()), corner.y().max(far.y())),
    );

    let before_path = ctx.out("text_annot_focus.before.png");
    let before_shot = crate::capture::window_to_png(&session, &before_path)?;
    report.artifact(before_path);

    let from = frame.to_screen(corner);
    let to = frame.to_screen(far);
    driver.drag(from, to)?;
    session.settle(20);

    let trace = session.trace()?;
    if trace.events("text-annot-open").next().is_none() {
        return Err(Error::new(
            "the drag opened no dialog, so there is nothing to type into. That is \
             `text_annot_places_and_authors`' territory, not this check's — fix it there.",
        ));
    }
    let Some(_field) = declared(&trace, ui_rect, "text-annot.text") else {
        return Err(Error::new(
            "the dialog declared no `text-annot.text` region, so it is not the text-bearing \
             kind and this check has nothing to say.",
        ));
    };
    report.note("the drag opened the dialog and it declared its text field");

    // --- C: ★★ TYPE. NO CLICK ON THE FIELD. --------------------------------
    //
    // The one line that separates this check from the one beside it. The
    // pointer stays exactly where the drag left it — over the page, outside the
    // window — and the keys go to whatever holds focus. If the dialog seated
    // its caret, that is the field. If it only *asked* and lost, it is not.
    //
    // Two keys already in `sys::vk`. WHAT is typed does not matter: Accept is
    // gated on the field being non-empty and nothing else.
    for key in [vk::F, vk::DIGIT_2] {
        driver.press(key)?;
        session.settle(8);
    }
    report.note("typed two characters WITHOUT clicking the field, the way an operator does");

    // --- D: Accept, and let the greying answer the question ----------------
    let trace = session.trace()?;
    let Some(accept) = declared(&trace, ui_rect, "text-annot.accept") else {
        return Err(Error::new(
            "the dialog declared no `text-annot.accept` region to press.",
        ));
    };
    driver.click_at(
        frame_of(&session, &trace, ui_rect, "text-annot.accept")?.declared_center(accept),
    )?;
    session.settle(24);

    let after = annot_count(&session);
    if after <= before {
        return Ok(Some(format!(
            "Accept was pressed after typing and the page still carries {after} \
             annotation(s). Accept is greyed while the field is empty, so this says the \
             KEYSTROKES NEVER REACHED THE FIELD — the dialog opened without holding the \
             keyboard. `text_annot_places_and_authors` does not see this because it clicks \
             the field first, which an operator has no reason to do. Look at \
             `dialogs::textannot::field`: a focus latch that records having ASKED rather \
             than `response.has_focus()` spends its one attempt on the frame after the \
             gesture, which is the frame the pointer release is still being resolved on."
        )));
    }
    report.note(format!(
        "the unclicked dialog took the keystrokes: Accept authored, {before} -> {after} \
         annotation(s)"
    ));

    // --- E: ★★ AND IT IS ON THE PAGE ---------------------------------------
    //
    // `add-text-annot` says the funnel ran. It does not say anything is
    // VISIBLE, and the operator's report — *"nothing gets added"* — is about
    // what they can see. An annotation authored without a usable appearance
    // renders as nothing at all, and every trace assertion above passes on
    // exactly that build. `D:/dev/rag/egui` states the rule this obeys:
    // **a rendering defect has one oracle, and it is a rendered pixel.**
    //
    // The pointer is parked away from the box first, for `markup_rectangle`'s
    // reason: a capture taken with the cursor over the region would measure a
    // hover as well as an annotation.
    driver.move_to(frame.to_screen(mapping.doc_to_window(DocPoint {
        page: target.page,
        x: target.x + BOX_PT * 2.0,
        y: target.y,
    })?))?;
    session.settle(10);
    let after_path = ctx.out("text_annot_focus.after.png");
    let after_shot = crate::capture::window_to_png(&session, &after_path)?;
    report.artifact(after_path);

    let px = frame.logical_to_capture_pixels(box_rect);
    if px.area() == 0 {
        return Err(Error::new(
            "the placed box resolves to no pixels of the captured client area, so there is \
             nothing to look at. The --doc-point is probably off the visible page.",
        ));
    }
    let changed = changed_pixels(&before_shot, &after_shot, px);
    let total = px.area();
    let ratio = f64::from(changed) / f64::from(total);
    if ratio < MIN_CHANGED_RATIO {
        return Ok(Some(format!(
            "the annotation was AUTHORED and is not VISIBLE. `add-text-annot` traced and the \
             count went {before} -> {after}, but only {changed} of {total} pixels \
             ({:.2}%) inside the placed box differ from the same region before the drag — \
             below the {:.0}% floor. The operator's words were *\"nothing gets added\"*. \
             Every trace assertion in this check and in \
             `text_annot_places_and_authors` passes on this build, because they measure \
             whether the engine was CALLED. Look for a missing or empty appearance stream, \
             an ink colour equal to the paper, or a page texture that was not invalidated.",
            ratio * 100.0,
            MIN_CHANGED_RATIO * 100.0
        )));
    }
    report.note(format!(
        "and it is on the page: {changed} of {total} pixels ({:.1}%) inside the box changed",
        ratio * 100.0
    ));
    Ok(None)
}

/// The share of sampled pixels that must differ for the annotation to count as
/// drawn.
///
/// Two characters of 12 pt text in a 220 pt box cover well under a percent of
/// it, so this floor is deliberately tiny — it separates *nothing at all* from
/// *something*, and is not a measurement of how much was drawn. Anti-aliasing
/// and the canvas's own re-render jitter are what it has to clear.
const MIN_CHANGED_RATIO: f64 = 0.001;

/// Count pixels of `region` that differ between two captures.
///
/// A plain per-channel threshold rather than a perceptual difference: the
/// question is *"did anything appear here"*, and text on paper is a large
/// contrast wherever it lands. The threshold exists only to ignore the
/// one-or-two-level noise a re-render produces on identical content.
fn changed_pixels(
    before: &crate::image::Image,
    after: &crate::image::Image,
    region: PixRect,
) -> u32 {
    let mut n = 0;
    for y in region.y..region.y.saturating_add(region.h) {
        for x in region.x..region.x.saturating_add(region.w) {
            let (Some(a), Some(b)) = (before.pixel(x, y), after.pixel(x, y)) else {
                continue;
            };
            let d = u16::from(a.r.abs_diff(b.r))
                .max(u16::from(a.g.abs_diff(b.g)))
                .max(u16::from(a.b.abs_diff(b.b)));
            if d > 12 {
                n += 1;
            }
        }
    }
    n
}

/// How many text annotations this session has authored. `text_annot`'s oracle,
/// and its reasoning: a difference between two reads taken the same way, so a
/// build that traces nothing produces equal counts and FAILS rather than passes.
fn annot_count(session: &Session) -> usize {
    session
        .trace()
        .map(|t| t.events("add-text-annot").count())
        .unwrap_or(0)
}
