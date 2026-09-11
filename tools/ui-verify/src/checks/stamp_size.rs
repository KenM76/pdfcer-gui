//! `stamp_size_reaches_the_engine` — the Size chooser in the stamp dialog is
//! pressed for real, and the number the operator picked is asserted to arrive
//! at the engine call.
//!
//! # The report this closes
//!
//! The operator, 2026-09-09:
//!
//! > *"I have to draw the size before it gets applied … still can't adjust the
//! > size of a stamp on the canvas, or by entering a different size in the
//! > properties box."*
//!
//! Engine `Pass 287.0` answered the authoring half of that on 2026-09-10, and
//! this shell wired it the same day: `dialogs::textannot::sizes` draws a **Size**
//! combo under the stamp gallery, offering *Fit the box I drew* and nine stated
//! point sizes, and the value travels
//! `TextAnnotDialog::stamp_size` → `Action::CommitTextAnnot::stamp_size` →
//! `app::actions::textannot::Placement` → `canvas::textannot::spec` →
//! `StampStyle::with_font_size`.
//!
//! ⚠ **It shipped in `v0.5.0-dev.20260910.1` undriven**, because the release was
//! asked for immediately. `OPERATOR_REQUESTS.md` O168 records that as a debt in
//! those words rather than folding it into a green report. This file is the
//! payment.
//!
//! # ★★★ Why unit tests could not have closed it, in this project's own words
//!
//! *"Unit tests cannot see the chain in front of the verb"* — the standing
//! lesson from the day eight green tests sat in front of a feature that did 1
//! of its 14 steps. The chain here is four hops long and every hop has a
//! passing test:
//!
//! | hop | its test | what that test cannot see |
//! |---|---|---|
//! | combo writes `self.stamp_size` | `a_fresh_dialog_is_empty_and_defaulted` | whether the combo is reachable with a pointer at all |
//! | dialog puts it on the action | `the_chosen_icon_reaches_the_commit_action` | it sets the field directly; no widget is pressed |
//! | action carries it to `spec` | the `Placement` tests | they construct a `Placement` by hand |
//! | `spec` builds the style | `canvas::textannot::tests` | it calls `spec` with the value already chosen |
//!
//! Every one of those passes on a build whose combo is drawn **underneath the
//! Accept button**, or clipped off the bottom of a window that did not grow, or
//! whose popup opens off-screen. `each_kinds_window_is_as_tall_as_its_body_needs`
//! asserts the window grew — it does not assert the control inside it can be
//! hit. This project has shipped a panel that was unreachable in a real build
//! with every gate green.
//!
//! # ★★ The oracle, and why the application grew a trace line for it
//!
//! `autosize_overflow` states the rule: *a trace line must carry the number a
//! wrong build would get wrong.* Before 2026-09-10 every line this route emits
//! — `text-annot-note`, `text-annot-page-rotate`, `add-text-annot` — was
//! **byte-identical** between a build that carried the chosen size and one that
//! dropped it. `canvas::textannot::spec`'s stamp arm now emits:
//!
//! ```text
//! pdfcer-diag stamp-style size=24 fit=grow rect_w=220.0 rect_h=220.0
//! ```
//!
//! `size=` is `StampSize::trace_token`'s output — the bare number for a
//! stated size, the literal word `derived` when the operator let the drawn box
//! decide. Its contract is asserted by
//! `canvas::textannot::tests::every_trace_token_is_parseable_and_distinct`,
//! because a token that drifts makes this check fail to **match** rather than
//! fail to build, and a check that cannot find `size=24` reports *"the
//! operator's choice did not reach the engine"* — a defect report about the
//! application, written by a defect in the harness.
//!
//! # ★★★ The falsification, which is the part that makes this a test
//!
//! **The size pressed is deliberately not the default.**
//!
//! `DEFAULT_STAMP_SIZE` is `StampSize::FitTheBox`, which traces `derived`. If
//! this check pressed *Fit the box I drew* it would pass on a build that
//! ignores the combo entirely, ignores the whole `stamp_size` field, and hard
//! codes the old behaviour — which is precisely the silent decline
//! `canvas::textannot`'s header warns about, since `font_size: None` compiles
//! and means exactly that. So it presses **24 pt** and asserts `size=24`, a
//! value no build can produce without having read the operator's choice.
//!
//! ⚠ And it asserts the default **first**, before pressing anything, for the
//! other direction: a build that opened the chooser on 12 pt would already have
//! shrunk every stamp on the operator's drawings, and would still pass a check
//! that only looked at the end state after an explicit selection.
//!
//! # ★★★ Falsified, 2026-09-10 — which assertions were made to fail, and which
//! was not
//!
//! *"A check that cannot fail is not evidence."* This one passed on its first
//! run, which is worth exactly nothing until the passing has been shown to be
//! contingent. Each defect below was planted in a real release build, the build
//! was driven, and the file was restored **from a kept copy** rather than
//! through git.
//!
//! | # | planted defect | one line | outcome |
//! |---|---|---|---|
//! | 1 | `dialogs::textannot` puts `DEFAULT_STAMP_SIZE` on the action instead of `self.stamp_size` — the dialog→action hop drops the pick | the exact silent decline the feature was built against | **FAIL**, at step 7, reporting `size=derived` and naming the hop |
//! | 2 | `TextAnnotDialog::open` starts the chooser at `StampSize::Points(12)` — the engine's flat default adopted | every stamp on his drawings shrinks, nobody presses anything | **FAIL**, at step 5, before any control is touched |
//!
//! ★★ **Defect 1 is the one that proves the check is not merely watching
//! itself.** The chooser still reported `24` under it — step 6 was green — and
//! only the commit disagreed. A check that had asserted the control's own state
//! and stopped there would have passed on a build that dropped the operator's
//! choice on the floor one hop later.
//!
//! ⚠ **Step 6 itself was NOT falsified**, and that is stated rather than left
//! for someone to assume from the table. Planting it means making an egui
//! `selectable_value` write a field without repainting its own combo, which is
//! not a defect this shell can express in one line — it would be a bug in the
//! framework. It is asserted because it is the operator's only confirmation,
//! not because it has been shown to be able to fail.
//!
//! # What this check does NOT claim
//!
//! It does not assert what the engine draws. `size=24` proves the shell handed
//! `StampStyle { font_size: Some(24.0), fit: GrowToText }` to
//! `add_text_annotation_with`; whether `pdfcer-core` then paints 24 pt text and
//! widens the `/Rect` is asserted by the engine's own tests. Stated because a
//! green line here must not be read as a claim about `pdfcer-core`'s renderer.
//!

use crate::checks::driving::{
    SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, frame_of, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The side, in PDF points, of the rectangle dragged for the stamp.
///
/// Matches `text_annot`'s box for the same reason it chose it: unambiguously a
/// drag rather than a click the gesture machine might round to one, and small
/// enough to stay on the sheet from any `--doc-point` that is itself on it.
const BOX_PT: f64 = 220.0;

/// The size the check asks for, as the operator would read it.
///
/// ★ **24, and the choice of number is load-bearing.** It must not be the
/// default (`derived`) or the check passes on a build that ignores the chooser;
/// it must not be `12` either, because that is `StampStyle::default()`'s flat
/// size — a build that threw the operator's choice away and adopted the
/// engine's default would trace `size=12` and look like a pass. 24 is
/// reachable only by having read what was pressed.
const WANTED_PT: u32 = 24;

/// The region the entry for [`WANTED_PT`] declares while the popup is open.
///
/// Keyed on the same token the trace carries, by construction in
/// `dialogs::textannot::sizes` — press `…stamp-size.24`, then look for
/// `size=24`. One vocabulary at both ends, so a rename cannot leave the check
/// pressing a control that exists and matching a field that no longer describes
/// it.
const WANTED_REGION: &str = "text-annot.stamp-size.24";

/// See the module documentation.
pub struct StampSizeReachesTheEngine;

impl Check for StampSizeReachesTheEngine {
    fn name(&self) -> &'static str {
        "stamp_size_reaches_the_engine"
    }

    fn defect(&self) -> &'static str {
        "the stamp dialog's Size chooser cannot be reached with a pointer, its popup opens \
         nowhere the operator can press, or the size he picks is dropped between the combo and \
         the engine call — leaving every stamp at whatever the build happens to default to"
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
             canvas, opens a combo box and picks an entry from its popup. Reported as SKIPPED \
             rather than passed, because a check that cannot press the control it is named \
             after has measured nothing.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. This check needs somewhere on the page to draw the stamp, and a \
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("stamp_size.trace.txt"));
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

    // --- 1: Review mode and the Markup tab ---------------------------------
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

    // --- 2: arm the stamp ---------------------------------------------------
    let trace = session.trace()?;
    let item = declared(&trace, ui_rect, "ribbon.item.markup.stamp").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.item.markup.stamp` region on the Markup tab. Items declared: {}.",
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
        return Ok(Some(
            "clicking Markup > Stamp traced no `markup-tool tool=TextAnnot(..)` line, so the \
             control armed nothing."
                .to_owned(),
        ));
    }

    // --- 3: drag the box and open the dialog --------------------------------
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    let frame = session.frame()?;
    let from = frame.to_screen(mapping.doc_to_window(target)?);
    let to = frame.to_screen(mapping.doc_to_window(DocPoint {
        page: target.page,
        x: target.x + BOX_PT,
        y: target.y + BOX_PT,
    })?);
    driver.drag(from, to)?;
    session.settle(20);

    let trace = session.trace()?;
    if trace.events("text-annot-open").next().is_none() {
        return Ok(Some(
            "the drag completed and no `text-annot-open` line was traced, so the release did \
             not open the stamp dialog."
                .to_owned(),
        ));
    }

    // --- 4: the chooser must EXIST and be reachable -------------------------
    //
    // ★★ A separate failure from "the size did not travel", and it is worth its
    // own message. A combo drawn below the window's bottom edge, or under the
    // Accept button, produces a check that presses nothing and an operator who
    // cannot use the feature — and the unit test asserting the window grew is
    // green in both cases.
    //
    // ★★★ **This step does NOT scroll, and that is the assertion — 2026-09-11.**
    //
    // The application publishes this region through `diag::ui_rect_visible`, so
    // it is declared only on the frames where it is actually on the screen. A
    // check that scrolled the body until it appeared would therefore be green
    // on a dialog whose Size chooser opens below its own fold — which is
    // precisely the state the full sweep of 2026-09-11 found and the state
    // `STAMP_EXTRA_PTS` was corrected to 190 pt to fix. Scrolling here would
    // have converted an operator-visible defect into a harness step.
    //
    // ⇒ The absence of a scroll is load-bearing. If this ever fails with "was
    // never drawn" on a build where the chooser plainly exists, the answer is
    // in `dialogs::textannot`'s window height, not in this file.
    let Some(combo) = declared(&trace, ui_rect, "text-annot.stamp-size") else {
        return Ok(Some(format!(
            "the stamp dialog is open and declares no `text-annot.stamp-size` region, so the \
             Size chooser is not on the screen when the dialog opens. ★ The region is \
             published through `diag::ui_rect_visible`, which stays silent for a rectangle \
             clipped out of its scroll area — so the likely cause is a window too short for \
             its own body (`dialogs::textannot::STAMP_EXTRA_PTS` plus `custom_extra_pts`), \
             not a missing control. The operator's version of this is a chooser he has to \
             scroll to find under a gallery that fills the window. Regions the dialog \
             declared: {}.",
            list(&declared_names(&trace, ui_rect, "text-annot."))
        )));
    };

    // ★★ **Declared is not the same as WHOLLY on screen**, and the difference is
    // 40 % of the control.
    //
    // `diag::ui_rect_visible` publishes the FULL rectangle once
    // `VISIBLE_FRACTION` (0.6) of it survives the clip. That threshold is right
    // for the application — a control 80 % shown is a control the operator can
    // use — but it leaves a chooser whose bottom third is under the pinned
    // footer declaring an aimable centre. The centre would still be pressable
    // here, so this check would pass; the operator would be looking at a combo
    // sliced by the Add button.
    //
    // So the containment is asserted separately against `dialog:text-annot`,
    // which is the window's whole content rectangle. It is a stricter question
    // than the one the application answers, asked by the surface whose job is
    // to ask stricter questions.
    if let Some(body) = declared(&trace, ui_rect, "dialog:text-annot") {
        if !body.contains_rect(combo) {
            return Ok(Some(format!(
                "the Size chooser is declared at {combo:?}, which is not wholly inside the \
                 dialog's content rectangle {body:?}. ★ `diag::ui_rect_visible` publishes a \
                 region once 60 % of it survives the clip, so a control sliced by the pinned \
                 Add/Cancel row still declares a pressable centre — this check passes and the \
                 operator sees a cut-off combo. The window is short of its own body by about \
                 {} pt; see `dialogs::textannot::STAMP_EXTRA_PTS`.",
                (combo.max.y - body.max.y).ceil().max(0.0)
            )));
        }
        report.note("the Size chooser is wholly inside the dialog, not merely declared");
    }
    report.note("the stamp dialog drew a Size chooser");

    // --- 5: ★★★ the default, ASSERTED before anything is pressed ------------
    //
    // The other direction of the same feature, and the one that would cost the
    // operator silently. `StampStyle::default()` is a flat 12 pt; a typical
    // 60 pt-high stamp box derived about 25 pt before engine `Pass 287.0`. A
    // build that opened this chooser on 12 pt would shrink every stamp on his
    // drawings **without anybody pressing anything**, and a check that only
    // looked at the end state after an explicit selection would never see it.
    //
    // ★★ This is an assertion and not a note. A note describing the default
    // would be read by whoever greps this file as "the default is covered",
    // which is the shape of claim this suite exists to stop being made without
    // a measurement behind it.
    let Some(opened_on) = chooser_reads(&session) else {
        return Ok(Some(
            "the stamp dialog drew a Size chooser and traced no `stamp-size-chooser selected=` line, so the control's own state cannot be observed. That line is emitted by `dialogs::textannot::sizes` through `diag::trace_changed` on every frame the chooser is drawn; its absence means either the chooser is not being drawn on the frame its region was declared, or the diagnostic channel is off for this process."
                .to_owned(),
        ));
    };
    if opened_on != "derived" {
        return Ok(Some(format!(
            "a fresh stamp dialog opens its Size chooser on `{opened_on}`, not on `derived`. ★ `DEFAULT_STAMP_SIZE` is `StampSize::FitTheBox` deliberately: before engine `Pass 287.0` a stamp's label size was worked out from the box the operator drew, and a 60 pt-high box came out around 25 pt. `derived` keeps that. `12` is `StampStyle::default()`, and adopting it would make every stamp on his drawings noticeably smaller from one release to the next — as a side effect of a fix he asked for, with no control anywhere to undo it."
        )));
    }
    report.note("a fresh chooser opens on `derived`, so existing habits are unchanged");

    // --- 6: open the popup and press 24 pt ----------------------------------
    driver.click_at(
        frame_of(&session, &trace, ui_rect, "text-annot.stamp-size")?.declared_center(combo),
    )?;
    session.settle(14);

    // ⚠ Read the frame AFTER the press. The entries are declared only while the
    // popup is open — a region that stops being declared emits `ui-rect-gone` —
    // so a cached capture from before the click is a fossil, which is the exact
    // shape of three wrong defect reports this project has already filed.
    let trace = session.trace()?;
    let Some(entry) = declared(&trace, ui_rect, WANTED_REGION) else {
        let offered = declared_names(&trace, ui_rect, "text-annot.stamp-size.");
        if offered.is_empty() {
            return Ok(Some(
                "pressing the Size chooser declared no entry regions at all, so the popup did \
                 not open — or opened in a layer that draws nothing the operator can press. \
                 The control is visible and inert, which reads to an operator as the feature \
                 not existing."
                    .to_owned(),
            ));
        }
        return Ok(Some(format!(
            "the popup opened and does not offer `{WANTED_REGION}`. Entries declared: {}.",
            list(&offered)
        )));
    };
    driver.click_at(frame_of(&session, &trace, ui_rect, WANTED_REGION)?.declared_center(entry))?;
    session.settle(14);

    // ★★ The control must now REPORT its own state, and this is a **separate
    // defect** from the size not travelling. A combo that stores the pick and
    // goes on displaying the previous one leaves the operator unable to tell an
    // accepted choice from an ignored click; the document comes out right for a
    // reason he could not have predicted, which is not the same thing as the
    // feature working.
    let now = chooser_reads(&session);
    if now.as_deref() != Some(&*wanted_token()) {
        return Ok(Some(format!(
            "`{WANTED_REGION}` was pressed and the chooser reports {now:?}, not `{}`. The popup entry is reachable and the click did not change the control's own state — whatever the commit below carries, the operator has no confirmation that pdfcer heard him.",
            wanted_token()
        )));
    }
    report.note(format!(
        "the chooser reports `{}` after the press",
        wanted_token()
    ));

    // --- 7: accept, and assert the NUMBER reached the engine call -----------
    let trace = session.trace()?;
    let Some(accept) = declared(&trace, ui_rect, "text-annot.accept") else {
        return Ok(Some(
            "the dialog declared no `text-annot.accept` region.".to_owned(),
        ));
    };
    driver.click_at(
        frame_of(&session, &trace, ui_rect, "text-annot.accept")?.declared_center(accept),
    )?;
    session.settle(24);

    let trace = session.trace()?;
    let styles: Vec<String> = trace
        .events("stamp-style")
        .filter_map(|l| l.get("size").map(std::string::ToString::to_string))
        .collect();
    let Some(got) = styles.last() else {
        return Ok(Some(
            "Accept was pressed and no `stamp-style` line was traced, so `canvas::textannot::\
             spec` never built a stamp. Either the commit did not reach it or it declined — \
             look for `text-annot-declined` in the trace."
                .to_owned(),
        ));
    };
    let wanted = WANTED_PT.to_string();
    if got != &wanted {
        return Ok(Some(format!(
            "{WANTED_PT} pt was picked from the Size chooser and the engine call carried \
             `size={got}`. The operator's choice was dropped somewhere between the combo and \
             `canvas::textannot::spec`. ★ `size=derived` means the build kept the pre-Pass-287 \
             behaviour and never read the field — the silent decline `font_size: None` \
             compiles into; `size=12` means it adopted `StampStyle::default()` instead, which \
             is the change that would shrink every stamp on the operator's drawings. Every \
             `stamp-style` line traced: {}.",
            list(&styles)
        )));
    }
    report.note(format!(
        "the engine call carried size={got}, which no build can produce without having read \
         what was pressed"
    ));
    Ok(None)
}

/// The size the check asks for, as the trace and the region name both spell it.
///
/// One function so the two cannot drift from each other or from [`WANTED_PT`];
/// [`WANTED_REGION`] is asserted against it by
/// [`the_region_and_the_token_agree`].
fn wanted_token() -> String {
    WANTED_PT.to_string()
}

/// What the Size chooser currently reports about **itself**.
///
/// # ★★★ Why this reads a purpose-built line and not the `ui-rect` line
///
/// The first draft of this check read the selection out of `ui-rect`, which
/// carries a **name** and a **rect** and no text whatsoever. It would have
/// returned `None` on every build that has ever existed, and the check would
/// have gone on to print a note saying the chooser *"still reads None after the
/// press"* — narrating an absence it had never measured, in a tone that reads
/// as a finding.
///
/// ⚠ That is this project's standing lesson about **unevidenced excuses**: a
/// check that explains a gap it did not measure turns an open question into a
/// closed one, and nobody looks again. The application grew
/// `stamp-size-chooser selected=` for this, so the absence is closed rather
/// than described.
///
/// Returns `None` only when the line is genuinely not there, and every caller
/// treats that as a **failure to observe** — reported as such — never as
/// *"reads nothing"*.
fn chooser_reads(session: &Session) -> Option<String> {
    let trace = session.trace().ok()?;
    trace
        .events("stamp-size-chooser")
        .filter_map(|l| l.get("selected").map(std::string::ToString::to_string))
        .last()
}

#[cfg(test)]
mod tests {
    use super::{WANTED_PT, WANTED_REGION, wanted_token};

    /// ★★ **The region this check presses and the token it matches are the same
    /// word**, and nothing else in the repository enforces it.
    ///
    /// `dialogs::textannot::sizes` builds the region name by interpolating
    /// `StampSize::trace_token()`, so the application's two ends agree by
    /// construction. This constant is the harness's hand-written copy of that
    /// name, and a hand-written copy of a generated string is exactly where the
    /// two drift — with the failure landing as *"the popup does not offer
    /// `…stamp-size.24`"*, a defect report about the application written by a
    /// stale constant here.
    #[test]
    fn the_region_and_the_token_agree() {
        assert_eq!(
            WANTED_REGION,
            format!("text-annot.stamp-size.{}", wanted_token()),
            "the region pressed must be the one the application declares for {WANTED_PT} pt"
        );
    }

    /// ★★★ **The size pressed is not a value any indifferent build produces.**
    ///
    /// The whole check turns on this. `derived` is what a build that ignores
    /// the chooser emits — `font_size: None` compiles and means *"work it out
    /// from the box"* — and `12` is what a build that adopted
    /// `StampStyle::default()` emits. A check pressing either would be green on
    /// the broken build it was written to catch.
    #[test]
    fn the_asserted_size_is_no_builds_default() {
        assert_ne!(
            wanted_token(),
            "derived",
            "that is the pre-Pass-287 default"
        );
        assert_ne!(wanted_token(), "12", "that is StampStyle::default()'s size");
    }
}
