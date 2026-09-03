//! Shared machinery for the two pixel checks: *where to look*, and *is what is
//! drawn there readable*.
//!
//! Both [`super::ribbon_captions`] and [`super::settings_headings`] ask the
//! same question of a different surface: *given this image and these named
//! regions, is every one of them legible?* The question is worth answering in
//! one place because the two ways of getting it wrong are subtle and would
//! otherwise be got wrong twice.
//!
//! ## The two failure modes, and why they are reported separately
//!
//! **Illegible** — something is drawn and its contrast against its background
//! is below the threshold. This is D2.
//!
//! **Absent** — the region is uniform: nothing is drawn there at all. The
//! 2026-08-08 screenshot audit found two ribbon groups rendering with **no
//! caption**, and a contrast-only check would report those as low contrast and
//! send whoever read it looking at the theme. They are different defects with
//! different fixes, so they get different sentences.
//!
//! There is now a third, which only became reachable once the application
//! started declaring its own regions: **off-surface** — the application says a
//! region is at a rect that is not inside the window it was captured from.
//! That is not a colour problem and not a missing caption; it is the control
//! being laid out outside its pane, which `PROJECT_PLAN.md` §4.2 prerequisite
//! 2 names as one of the two recorded cases that justified having a pixel
//! oracle at all.
//!
//! ## Where a region comes from — two sources, and the order is the point
//!
//! [`resolve_set`] consults them in this order:
//!
//! 1. **The application's own `ui-rect` declarations, from this run's trace.**
//! 2. **A calibrated [`RegionSet`] of fractions** in [`crate::profile`].
//!
//! ### Why the trace wins
//!
//! A rect the application measured on the frame it reported it for **cannot
//! go stale**, because there is no interval between the measurement and the
//! claim. A fraction written into the harness is a claim about a layout that
//! held when somebody looked at it, and it becomes wrong the first time a
//! panel is resized, the ribbon collapses to an icon rail, or a workspace is
//! switched — all three of which are on `MODES_AND_PANELS.md`'s roadmap.
//!
//! And it goes wrong in the worst available way: **silently, by measuring the
//! wrong pixels**. The assertion still runs, still samples thousands of
//! pixels, and still prints a contrast ratio. Nothing about the output says it
//! is now describing the wrong widget. That is the hazard `PROJECT_PLAN.md`
//! §4.2 prerequisite 1 names, and it is why the ordering is not a matter of
//! taste: if both sources are available they are both *plausible*, and only
//! one of them is *dated to this frame*.
//!
//! ### Why the fraction source is kept anyway
//!
//! Not every surface reports. `evidence/crop_settings.png` is a screenshot
//! taken in 2026-08 of a binary that has no `ui-rect` vocabulary at all, and
//! the D2 falsification run asserts against it — that run is the harness's own
//! acceptance evidence and it cannot ask a PNG to declare its regions. So the
//! fraction source stays, as the fallback, guarded by [`Calibration`].
//!
//! ## The calibration guard
//!
//! [`resolve_set`] refuses to apply a *fraction* region set to a surface it
//! was not calibrated for. Fractions measured against a 1860×1035 evidence
//! crop describe *that crop*; applied to a live 2560×1440 window they would
//! sample whatever happens to lie at those fractions and produce a real
//! measurement of the wrong thing. A number like that is worse than no number,
//! because it looks like evidence.
//!
//! Note that the guard has nothing to say about the trace source, and does not
//! need to: a traced rect carries its own calibration, in the sense that the
//! surface it describes is the window it was measured in, which is the window
//! that was then captured. That is the same argument as above, stated as an
//! absence of machinery.

use std::path::Path;

use crate::geom::{FracRect, PixRect};
use crate::image::Image;
use crate::pixels::{self, ContrastReport};
use crate::profile::{Calibration, Profile, RegionSet};
use crate::report::CheckReport;

/// Where a plan's rectangles came from. Printed, because a reader judging a
/// contrast number needs to know whether it was aimed by the application or by
/// a number somebody wrote down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionSource {
    /// The application declared them this run, via its `ui-rect` event.
    Trace,
    /// Calibrated fractions from the profile.
    ProfileFraction,
}

impl RegionSource {
    /// One word for the report.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Trace => "the application's own ui-rect declarations, this run",
            Self::ProfileFraction => "calibrated fractions in the harness profile",
        }
    }
}

/// How one region's area is expressed, before it is resolved against a
/// specific image.
///
/// Two variants because the two sources are resolved against different things
/// and at different times: a fraction needs the image's dimensions, which are
/// only known once the PNG is loaded, whereas a traced rect was converted to
/// capture pixels at the moment the window was measured. Collapsing them into
/// one would mean either resolving fractions too early or carrying a
/// [`crate::coords::WindowFrame`] into the offline path that has no window.
#[derive(Clone, Copy, Debug)]
pub enum RegionArea {
    /// Fractions of the measured surface.
    Fraction(FracRect),
    /// Device pixels within the capture, already converted from the logical
    /// rect the application reported.
    Pixels(PixRect),
}

/// One region a check is about to measure.
#[derive(Clone, Debug)]
pub struct PlannedRegion {
    /// What is expected to be drawn here, as the application named it or as
    /// the profile named it. Used verbatim in the report.
    pub name: String,
    /// Where.
    pub area: RegionArea,
}

/// Everything [`assess`] needs: what to measure, and where it came from.
#[derive(Clone, Debug)]
pub struct RegionPlan {
    /// The set name the check asked for.
    pub set_name: String,
    /// Which source supplied the rectangles.
    pub source: RegionSource,
    /// How they were obtained, in a sentence. Printed, so a reader can judge
    /// how much to trust a number derived from them.
    pub provenance: String,
    /// The regions, in report order.
    pub regions: Vec<PlannedRegion>,
}

/// What the application declared this run, as a check sees it.
///
/// Built by the check, because only the check knows which of the declared
/// names are the ones it is about. It carries the unmatched names too, and
/// that is not padding: it is what lets a SKIP reason distinguish
///
/// * "the application declared nothing at all" — the trace channel is not
///   working, or the diagnostic switch did not reach the process;
/// * "the application declared five regions and none of them is a ribbon
///   caption" — the trace channel is fine and the *ribbon* is what is missing.
///
/// Those two send a reader to different files, which is the entire reason this
/// crate is fussy about reason strings.
#[derive(Clone, Debug)]
pub struct TraceRegions {
    /// The declared regions this check is about, already in capture pixels.
    pub matched: Vec<PlannedRegion>,
    /// Every name the application declared, matched or not, in trace order.
    pub declared: Vec<String>,
    /// How the check describes what it was looking for, as a noun phrase that
    /// completes "the application declared no …". Written by the check, so the
    /// SKIP reason names the check's own convention rather than a generic one.
    pub convention: &'static str,
}

impl TraceRegions {
    /// A one-line summary of what the application said, for a report note.
    #[must_use]
    pub fn summary(&self) -> String {
        if self.declared.is_empty() {
            return "the application declared no ui-rect regions at all".to_owned();
        }
        format!(
            "the application declared {} ui-rect region(s): {}",
            self.declared.len(),
            self.declared.join(", ")
        )
    }
}

/// Decide what to measure, and say so — or explain why nothing can be.
///
/// `image_source` is the file being asserted against in offline mode, or
/// `None` in live mode. `trace` is what the application declared this run, or
/// `None` when no trace was consulted at all (offline mode, or a check that
/// does not launch the binary).
///
/// Precedence is documented at length in the module docs: **trace first**,
/// fractions second. The short version is that a rect measured on the frame it
/// was reported for cannot go stale, and a fraction is wrong the first time a
/// panel resizes — silently, by measuring the wrong pixels while still
/// printing a plausible number.
///
/// # Errors
///
/// Returns `Err(reason)` when the check should SKIP. The reason is assembled
/// from *what was actually consulted*, never from a template: a reason that
/// says the trace declared no regions when no trace was read is not merely
/// imprecise, it sends the reader to the wrong file — which is worse than
/// giving no reason at all, because they will believe it.
pub fn resolve_set(
    profile: &'static Profile,
    set_name: &str,
    image_source: Option<&Path>,
    trace: Option<&TraceRegions>,
) -> Result<RegionPlan, String> {
    // --- source 1: the application said so, this run ------------------------
    if let Some(t) = trace
        && !t.matched.is_empty()
    {
        return Ok(RegionPlan {
            set_name: set_name.to_owned(),
            source: RegionSource::Trace,
            provenance: format!(
                "declared by the application itself as `ui-rect` events on the frames they were \
                 laid out for, out of {} region(s) it declared in total",
                t.declared.len()
            ),
            regions: t.matched.clone(),
        });
    }

    // --- source 2: a calibrated fraction set in the profile -----------------
    let Some(set) = profile.region_set(set_name) else {
        return Err(no_region_source(profile, set_name, trace));
    };

    match (set.calibrated_for, image_source) {
        (Calibration::LiveWindow, _) => Ok(plan_from_set(set)),
        (Calibration::Image(expected), Some(actual)) => {
            let matches = Path::new(expected)
                .file_name()
                .is_some_and(|e| actual.file_name().is_some_and(|a| a == e));
            if matches {
                Ok(plan_from_set(set))
            } else {
                Err(format!(
                    "the `{set_name}` region set is calibrated for {expected} and the run was \
                     pointed at {}. Applying it anyway would sample whatever lies at those \
                     fractions and report a real measurement of the wrong thing.",
                    actual.display()
                ))
            }
        }
        (Calibration::Image(expected), None) => Err(format!(
            "the `{set_name}` region set is calibrated for the image {expected}, not for a \
             live window, so it cannot be used to assert against a running application. \
             Either run with --image {expected}, or have the application declare these \
             regions with `ui-rect` events, which needs no calibration at all."
        )),
    }
}

/// The SKIP reason for "no source could supply a region", assembled from what
/// was actually consulted.
///
/// Three distinct situations, three distinct sentences, because they have
/// three different fixes and the reader is about to go and do one of them.
fn no_region_source(
    profile: &'static Profile,
    set_name: &str,
    trace: Option<&TraceRegions>,
) -> String {
    let profile_clause = format!(
        "the `{}` profile carries no `{set_name}` region set",
        profile.name
    );
    match trace {
        // No trace was read. Say exactly that, and do NOT claim anything about
        // what the application declared — this is the case the old wording got
        // wrong, and it got it wrong in the direction that blames the
        // application for a silence nobody listened for.
        None => format!(
            "{profile_clause}, and this run consulted no live trace, so there is no source for \
             the regions and the check does not know where to look. Either point it at a \
             running binary so the application can declare its own `ui-rect` regions, or \
             calibrate a region set for the surface being asserted against."
        ),
        // A trace was read and the application said nothing at all. That is
        // about the trace channel, not about any one subsystem.
        Some(t) if t.declared.is_empty() => format!(
            "the application declared no `ui-rect` regions at all this run, and {profile_clause}. \
             An application that declares nothing has either not reached the frame that declares \
             its regions, or is not tracing at all — check that the diagnostic switch reached the \
             process before looking at any one widget."
        ),
        // The channel works. Something more specific is missing, and naming it
        // is the whole value of this branch.
        Some(t) => format!(
            "the application declared no {} — it declared {} other region(s) ({}), so the \
             `ui-rect` channel is working and it is the regions themselves that do not exist \
             yet. {profile_clause}, deliberately: a fraction written into the harness would be \
             a guess at a surface nobody has measured, and it would keep measuring after the \
             surface moved. This check starts asserting on its own, with no harness change, on \
             the first run where those regions are declared.",
            t.convention,
            t.declared.len(),
            t.declared.join(", ")
        ),
    }
}

/// Turn a static fraction set into a plan.
fn plan_from_set(set: &'static RegionSet) -> RegionPlan {
    RegionPlan {
        set_name: set.name.to_owned(),
        source: RegionSource::ProfileFraction,
        provenance: set.provenance.to_owned(),
        regions: set
            .regions
            .iter()
            .map(|r| PlannedRegion {
                name: r.name.to_owned(),
                area: RegionArea::Fraction(r.area),
            })
            .collect(),
    }
}

/// One region's measurement.
///
/// `contrast` is `None` for a region that could not be sampled at all — an
/// off-surface region. Deliberately an `Option` rather than a contrast of
/// 1.0 over zero pixels: the two are the same number and completely different
/// findings, and this crate's whole thesis is that a measurement of nothing
/// must not be presentable as a measurement.
struct Measurement {
    name: String,
    contrast: Option<ContrastReport>,
    uniform: bool,
}

/// Measure every region in `plan` against `image`, recording a note per region.
///
/// Returns `None` if every region is legible, or `Some(reason)` naming every
/// region that is not — all of them, not the first, because a reader fixing a
/// theme wants the whole list.
pub fn assess(
    image: &Image,
    plan: &RegionPlan,
    threshold: f64,
    report: &mut CheckReport,
) -> Option<String> {
    report.note(format!(
        "region set `{}` — {} region(s); source: {}",
        plan.set_name,
        plan.regions.len(),
        plan.source.label()
    ));
    report.note(format!("provenance: {}", plan.provenance));
    report.note(format!(
        "surface {}x{} px; threshold {threshold:.1}:1 (WCAG 2.1 AA)",
        image.width(),
        image.height()
    ));

    let mut measurements = Vec::new();
    for region in &plan.regions {
        let px = match region.area {
            RegionArea::Fraction(f) => f.resolve(image.width(), image.height()),
            RegionArea::Pixels(p) => clip(p, image.width(), image.height()),
        };
        if px.area() == 0 {
            report.note(format!("  {:<40} [OFF-SURFACE] {px:?}", region.name));
            measurements.push(Measurement {
                name: region.name.clone(),
                contrast: None,
                uniform: false,
            });
            continue;
        }
        let contrast = pixels::contrast_at(image, px);
        let uniform = pixels::region_not_uniform(image, px).is_uniform();
        report.note(format!(
            "  {:<40} {}{}",
            region.name,
            contrast.summary(),
            if uniform { "  [UNIFORM]" } else { "" }
        ));
        measurements.push(Measurement {
            name: region.name.clone(),
            contrast: Some(contrast),
            uniform,
        });
    }

    let off_surface: Vec<&str> = measurements
        .iter()
        .filter(|m| m.contrast.is_none())
        .map(|m| m.name.as_str())
        .collect();
    let absent: Vec<&str> = measurements
        .iter()
        .filter(|m| m.contrast.is_some() && m.uniform)
        .map(|m| m.name.as_str())
        .collect();
    let illegible: Vec<String> = measurements
        .iter()
        .filter(|m| !m.uniform)
        .filter_map(|m| m.contrast.as_ref().map(|c| (m.name.as_str(), c)))
        .filter(|(_, c)| !c.meets(threshold))
        .map(|(name, c)| format!("{name} at {:.2}:1", c.ratio))
        .collect();

    if off_surface.is_empty() && absent.is_empty() && illegible.is_empty() {
        return None;
    }

    let mut reason = String::new();
    if !off_surface.is_empty() {
        reason.push_str(&format!(
            "{} region(s) are OFF-SURFACE — the application declared them at rects that are not \
             inside the window that was captured: {}. Nothing was measured there because there \
             was nothing to measure; this is a layout defect, not a colour one. ",
            off_surface.len(),
            off_surface.join(", ")
        ));
    }
    if !absent.is_empty() {
        reason.push_str(&format!(
            "{} region(s) are UNIFORM — nothing is drawn there at all: {}. That is a missing \
             caption, not a colour problem; do not go looking at the theme. ",
            absent.len(),
            absent.join(", ")
        ));
    }
    if !illegible.is_empty() {
        reason.push_str(&format!(
            "{} region(s) are below the {threshold:.1}:1 floor: {}. Something is drawn and it \
             is not readable — check the widget's foreground against the fill it actually \
             paints with, which is not necessarily the one the palette assigns.",
            illegible.len(),
            illegible.join(", ")
        ));
    }
    Some(reason)
}

/// Intersect a pixel rect with the surface, returning a zero-area rect when
/// they do not overlap at all.
///
/// A traced rect is already clamped to the client area by
/// [`crate::coords::WindowFrame::logical_to_capture_pixels`]; this repeats the
/// clamp against the *image* because the two can differ by a pixel when a
/// window is resized between the measurement and the capture, and because
/// `--image` mode can hand a plan a surface of an entirely different size.
fn clip(r: PixRect, width: u32, height: u32) -> PixRect {
    let x0 = r.x.min(width);
    let y0 = r.y.min(height);
    let x1 = r.x.saturating_add(r.w).min(width);
    let y1 = r.y.saturating_add(r.h).min(height);
    PixRect::new(x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{PDFCER_GUI, PDFCER_LEGACY};

    fn traced(name: &str, area: PixRect) -> PlannedRegion {
        PlannedRegion {
            name: name.to_owned(),
            area: RegionArea::Pixels(area),
        }
    }

    /// The precedence rule, as a test so it cannot be reordered by accident.
    /// The legacy profile HAS a `settings_headings` fraction set; a trace that
    /// declared the same regions must win, because the fractions describe a
    /// dated crop and the trace describes this frame.
    #[test]
    fn a_traced_region_outranks_a_calibrated_fraction() {
        let t = TraceRegions {
            matched: vec![traced("Appearance", PixRect::new(10, 10, 50, 12))],
            declared: vec!["Appearance".to_owned()],
            convention: "settings heading regions",
        };
        let plan = resolve_set(
            &PDFCER_LEGACY,
            "settings_headings",
            Some(Path::new("evidence/crop_settings.png")),
            Some(&t),
        )
        .expect("the trace supplies the regions");
        assert_eq!(plan.source, RegionSource::Trace);
        assert_eq!(plan.regions.len(), 1);
    }

    /// …and with no trace, the fraction set is still used. The fallback is not
    /// decoration: the D2 falsification run depends on it.
    #[test]
    fn the_fraction_set_remains_the_fallback_for_a_surface_that_cannot_report() {
        let plan = resolve_set(
            &PDFCER_LEGACY,
            "settings_headings",
            Some(Path::new("evidence/crop_settings.png")),
            None,
        )
        .expect("the calibrated set applies to its own evidence file");
        assert_eq!(plan.source, RegionSource::ProfileFraction);
        assert_eq!(plan.regions.len(), 7);
    }

    /// An empty match list must NOT be treated as a source. Falling through to
    /// the fractions is what keeps the offline path working.
    #[test]
    fn a_trace_that_matched_nothing_falls_through_rather_than_planning_nothing() {
        let t = TraceRegions {
            matched: Vec::new(),
            declared: vec!["page".to_owned()],
            convention: "settings heading regions",
        };
        let plan = resolve_set(
            &PDFCER_LEGACY,
            "settings_headings",
            Some(Path::new("evidence/crop_settings.png")),
            Some(&t),
        )
        .expect("falls back to the calibrated fractions");
        assert_eq!(plan.source, RegionSource::ProfileFraction);
    }

    /// The stale-reason regression. When a trace WAS read and the application
    /// declared regions, the reason must say what it declared and must not
    /// claim it declared none.
    #[test]
    fn the_skip_reason_names_the_missing_subsystem_not_the_trace_channel() {
        let t = TraceRegions {
            matched: Vec::new(),
            declared: vec![
                "canvas-viewport".to_owned(),
                "central-panel".to_owned(),
                "page".to_owned(),
            ],
            convention: "ribbon group caption regions",
        };
        let reason = resolve_set(&PDFCER_GUI, "ribbon_group_captions", None, Some(&t))
            .expect_err("there is no ribbon yet");
        assert!(
            reason.contains("declared no ribbon group caption regions"),
            "{reason}"
        );
        assert!(
            reason.contains("central-panel"),
            "the reason must show what WAS declared: {reason}"
        );
        assert!(
            !reason.contains("no `ui-rect` regions at all"),
            "the application declared three of them; claiming otherwise sends the reader to \
             diag.rs, which is finished: {reason}"
        );
    }

    /// The other half of the same regression: when nothing was consulted, the
    /// reason must not assert anything about the application either.
    #[test]
    fn a_run_with_no_trace_does_not_claim_the_application_was_silent() {
        let reason = resolve_set(&PDFCER_GUI, "ribbon_group_captions", None, None)
            .expect_err("no source at all");
        assert!(reason.contains("consulted no live trace"), "{reason}");
        assert!(
            !reason.contains("declared no"),
            "nothing listened, so nothing may be claimed about what was said: {reason}"
        );
    }

    #[test]
    fn a_silent_application_is_reported_as_a_trace_channel_problem() {
        let t = TraceRegions {
            matched: Vec::new(),
            declared: Vec::new(),
            convention: "ribbon group caption regions",
        };
        let reason = resolve_set(&PDFCER_GUI, "ribbon_group_captions", None, Some(&t))
            .expect_err("no source at all");
        assert!(reason.contains("no `ui-rect` regions at all"), "{reason}");
        assert!(reason.contains("diagnostic switch"), "{reason}");
    }

    #[test]
    fn clipping_keeps_an_overlapping_region_and_empties_a_disjoint_one() {
        assert_eq!(
            clip(PixRect::new(90, 90, 40, 40), 100, 100),
            PixRect::new(90, 90, 10, 10)
        );
        assert_eq!(clip(PixRect::new(200, 0, 10, 10), 100, 100).area(), 0);
    }
}
