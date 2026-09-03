//! `qat_controls_are_icon_only` — the regression test for the icon painter
//! that existed, was tested, and was never handed to the ribbon.
//!
//! # The defect
//!
//! The quick-access toolbar draws Open, Save a copy, Undo and Redo. Every one
//! of those commands names an icon and carries a tooltip, the icon set was
//! salvaged and landed with 47 glyphs and 52 tests (**72 glyphs** as of
//! 2026-08-14, when the 41 commands that still rendered as bare words were
//! swept — 30 wired, 11 refused on the record), and
//! `crate::icons::paint_ribbon_icon` was written, documented and asserted to
//! satisfy the shell's seam.
//!
//! And the QAT rendered **text buttons**, because nobody wrote
//! `.with_icon_painter(&mut icons)` at the one call site that matters.
//!
//! The trace said so plainly, and in exactly the terms this check asserts on:
//!
//! ```text
//! ui-rect name=ribbon.qat.file.open rect=[[8.0 5.0] - [81.1 23.0]]     <- 73 pt: a label
//! ui-rect name=ribbon.qat.file.open rect=[[8.0 5.0] - [32.0 23.0]]     <- 24 pt: a glyph
//! ```
//!
//! # ★ Why no unit test could have caught it, which is why this file exists
//!
//! `egui_shell::ribbon::qat`'s `shows_label` decides icon-only from **three**
//! conditions: the command names an icon, it has a tooltip to serve as that
//! icon's accessible name, and *the application supplied a painter*. The third
//! clause was itself added after an earlier build produced a row of blank grey
//! boxes.
//!
//! Both outcomes are legal, and **both render correctly**. A ribbon of text
//! buttons is not an error state — it is what a consumer with no icon set is
//! supposed to get. The difference is not a condition a type can forbid or an
//! assertion can trip: it is *whether a caller remembered to pass an
//! argument*.
//!
//! So the crate's own tests were all true and all silent. `icons`' 52 tests
//! proved the painter draws. `egui-shell`'s proved the seam accepts it. The
//! one fact nobody could state in either crate is that `pdfcer-gui` hands the
//! former to the latter — because that is a property of a call site, and the
//! only place a call site's *effect* is observable is a running window.
//!
//! This is the project's founding rule with a fourth instance behind it, and
//! the first three are recorded in `DEFECTS.md` and `README.md`: verification
//! means driving the binary, because a green suite is evidence about the code
//! that was written and not about the code that was not.
//!
//! # What this asserts, and why it is a shape rather than a pixel
//!
//! **Every `ribbon.qat.*` region is approximately square.**
//!
//! An icon-only control is its glyph plus symmetric padding, so its width and
//! height are within a small factor of each other. A control that fell back to
//! a label is its glyph *or* its text plus padding, and the text is a word —
//! `Save a copy…` measured 107 pt against an 18 pt height, a ratio of nearly
//! six.
//!
//! Deliberately **not** an assertion on an exact width. A width is a function
//! of the theme's icon metric, the padding, and `pixels_per_point`, none of
//! which this check owns; pinning one would produce a check that fails when a
//! designer changes a spacing constant, which is how a check stops being run.
//! The ratio is invariant under all three.
//!
//! Deliberately **not** a pixel assertion either. Reading the glyph's pixels
//! would test that the *icon set* draws, which `icons`' own tests already do
//! properly and offline. What was missing was never "does a glyph render" — it
//! was "did anything ask for one", and the reserved rectangle answers that
//! without a screenshot, without raising a window, and therefore without
//! taking the operator's focus.
//!
//! # Why the QAT and not the whole ribbon
//!
//! Band controls legitimately show a label *beside* an icon — that is what a
//! ribbon button is — so a wide band control says nothing. The QAT is the one
//! surface in the shell whose controls are icon-**only** by design, which
//! makes it the only place the missing painter is expressible as a shape.
//!
//! One consequence worth stating: this check passes on a build with **no icon
//! keys at all**, because `shows_label`'s first condition then fails for a
//! different reason and the control is honestly a label. That is correct. This
//! check's subject is the painter, and a build with no icons has nothing to
//! paint. `crate::checks::ribbon_captions` covers the ribbon's legibility
//! either way.

use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};

/// How far from square a control may be and still count as icon-only.
///
/// An icon-only control is a square glyph box plus symmetric padding, so its
/// ratio is 1.0 before padding and drifts *below* 1.0 as vertical padding is
/// added — measured at 24 × 18 pt, i.e. 1.33, on the build that fixed the
/// defect.
///
/// 2.0 is therefore loose by design, and the looseness is the point: the
/// failing case was 4.1 (`Open…`, 73 × 18) and 5.9 (`Save a copy…`,
/// 107 × 18). Anything between 2 and 4 would be a control this check has no
/// theory about, and a threshold set just above the passing measurement would
/// fail the first time somebody adds a point of padding.
const MAX_ASPECT: f32 = 2.0;

/// The prefix `egui-shell` publishes QAT control rects under.
const QAT_PREFIX: &str = "ribbon.qat.";

pub struct QatControlsAreIconOnly;

impl Check for QatControlsAreIconOnly {
    fn name(&self) -> &'static str {
        "qat_controls_are_icon_only"
    }

    fn defect(&self) -> &'static str {
        "the quick-access toolbar falls back to text labels because no icon painter was supplied"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // No offline mode, and that is a property of the subject rather than an
    // omission. The other legibility checks can assert against a captured PNG
    // because a colour is in the pixels; a *reserved rectangle* is not. A
    // screenshot of a text-button QAT and one of an icon QAT differ in their
    // pixels, but recovering "which control is this, and what width did the
    // layout give it" from those pixels means re-implementing the layout —
    // whereas the application already states it, once, in the trace.
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}. This check \
             has no --image mode: its subject is a rectangle the application declares, not a \
             colour a capture preserves.",
            ctx.profile.default_exe
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("qat_icons.trace.txt"));
    spec.pdf = ctx.pdf.clone();
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    // The QAT is chrome: it is laid out on the first frame and does not wait
    // for a document. A short settle only so the trace is flushed.
    session.settle(12);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process, and this check has no way to learn what width any control was given. \
             Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }

    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its controls are and this check cannot read them.",
            ctx.profile.name
        ))
    })?;

    // LAST wins per name. A control is re-declared every frame, and an early
    // frame can carry a rect from before the layout settled — the find bar's
    // one-frame misplacement was exactly that shape, and taking the first
    // occurrence would have made this check assert on it.
    let mut controls: Vec<(String, crate::geom::LRect)> = Vec::new();
    for line in trace.events(ui_rect) {
        let Some(name) = line.get("name") else {
            continue;
        };
        if !name.starts_with(QAT_PREFIX) {
            continue;
        }
        let Some(rect) = line.get_rect("rect") else {
            continue;
        };
        match controls.iter_mut().find(|(n, _)| n == name) {
            Some(slot) => slot.1 = rect,
            None => controls.push((name.to_owned(), rect)),
        }
    }

    if controls.is_empty() {
        // A SKIP, emphatically not a pass. Finding no controls looks exactly
        // like finding no wide controls, and PROJECT_PLAN.md §4.1 records a
        // gate that printed "clean" while checking a handful of files.
        return Err(Error::new(format!(
            "the application declared no `{QAT_PREFIX}*` regions, so there was nothing to \
             measure. Either this build has no quick-access toolbar, or it does not publish \
             its control rects through `{ui_rect}`. Reported as SKIPPED rather than passed: a \
             check that measured nothing has learned nothing."
        )));
    }

    report.note(format!(
        "{} quick-access control(s) declared their rects",
        controls.len()
    ));

    let mut wide = Vec::new();
    for (name, rect) in &controls {
        let (w, h) = (rect.width(), rect.height());
        // A zero-height control cannot be divided by, and it is a real state:
        // a control clipped out of a narrow window. Reported as its own fact
        // rather than folded into the aspect test, because "it has no height"
        // and "it is too wide" have different causes.
        if h <= 0.0 {
            wide.push(format!("{name} has no height ({w:.1} x {h:.1} pt)"));
            continue;
        }
        let aspect = w / h;
        report.note(format!("{name}: {w:.1} x {h:.1} pt, aspect {aspect:.2}"));
        if aspect > MAX_ASPECT {
            wide.push(format!("{name} is {w:.1} x {h:.1} pt (aspect {aspect:.2})"));
        }
    }

    if wide.is_empty() {
        return Ok(None);
    }
    Ok(Some(format!(
        "{} of {} quick-access control(s) are wider than {MAX_ASPECT:.1}x their height, which \
         is the shape of a TEXT LABEL rather than an icon: {}. \
         `egui_shell::ribbon::qat`'s `shows_label` draws a control icon-only only when the \
         command names an icon, it has a tooltip, AND the application supplied a painter — so \
         the usual cause is a missing `Ribbon::with_icon_painter(...)` at the ribbon's call \
         site, not a missing icon.",
        wide.len(),
        controls.len(),
        wide.join("; ")
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The threshold separates the two states that were actually measured,
    /// with room on both sides.
    ///
    /// These are not invented numbers: 73 x 18 and 107 x 18 are the widths the
    /// defective build published for `file.open` and `file.save_copy`, and
    /// 24 x 18 is what the fixed build publishes. Pinning them here means a
    /// future change to `MAX_ASPECT` has to be made against the evidence
    /// rather than against a guess about what "roughly square" means.
    #[test]
    fn the_threshold_separates_the_measured_states() {
        let icon = 24.0_f32 / 18.0;
        let label_open = 73.1_f32 / 18.0;
        let label_save = 107.4_f32 / 18.0;
        assert!(icon < MAX_ASPECT, "the fixed build must pass: {icon:.2}");
        assert!(
            label_open > MAX_ASPECT,
            "the defective build must fail: {label_open:.2}"
        );
        assert!(
            label_save > MAX_ASPECT,
            "the defective build must fail: {label_save:.2}"
        );
        // Room on both sides, so neither a padding tweak nor a slightly
        // shorter label flips the verdict.
        assert!(
            MAX_ASPECT - icon > 0.5 && label_open - MAX_ASPECT > 0.5,
            "the threshold sits too close to a measured value"
        );
    }

    /// The prefix is the one `egui-shell` actually publishes.
    ///
    /// Owned by another crate, so it is asserted rather than assumed — the
    /// same reason `ribbon_captions` pins its caption spelling.
    #[test]
    fn the_prefix_matches_the_shells_own_spelling() {
        assert!("ribbon.qat.file.open".starts_with(QAT_PREFIX));
        assert!("ribbon.qat.edit.undo".starts_with(QAT_PREFIX));
        // …and does not swallow a band control, which may legitimately be
        // wide because a ribbon button shows a label beside its icon.
        assert!(!"ribbon.group.view.zoom".starts_with(QAT_PREFIX));
        assert!(!"ribbon.tab.file".starts_with(QAT_PREFIX));
    }
}
