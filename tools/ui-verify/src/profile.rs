// old-name-exempt-file: this file DEFINES the falsification profile for the
// pre-rename GUI, so it must spell that build's repository, binary,
// diagnostic environment variables and trace prefix the way THAT build
// spells them. Every one of those names lives outside this repository and
// did not rename with us.
//
// ** The grep is not being relaxed, it is being REPLACED by a better
// instrument. Two tests at the bottom of this file cover what the grep
// covered here and more:
//
//   * `legacy_profile_names_the_pre_rename_gui` -- the four external names
//     must carry the old stem and must NOT carry the new one;
//   * `current_profile_names_only_the_new_project` -- this build's own
//     profile must carry the new stem and no bare old one.
//
// A grep can only ask one of those two questions and gets the other one
// backwards, because "pdfcer" CONTAINS "pdfce".
//! What the harness knows about a particular target binary.
//!
//! ## Why the vocabulary is data and not hard-coded
//!
//! Two binaries matter to this project: the one being built here, and the one
//! at `D:\Dev\pdfcer\target\release\pdfcer-gui.exe` that it replaces. The
//! harness must be pointable at both, because "these checks FAIL against the
//! old binary and PASS against the new one" is the acceptance criterion for
//! the *harness itself* (`PROJECT_PLAN.md` §4, stage S1). A check suite that
//! has never been observed to fail is not evidence of anything — the same
//! argument that put a self-test in `tools/gates/check-ui-strings.sh`.
//!
//! Those two binaries do not share a trace vocabulary. The old one reports a
//! selection count on its `canvas` line as `sel=`, and a deletion as
//! `delete-objects n=`. The new one will report whatever it decides to report.
//! A parser with either spelling baked in could only ever be aimed at one of
//! them, so the spelling lives here, in a [`Vocabulary`], and the checks ask
//! for concepts.
//!
//! ## Region sets and the calibration rule
//!
//! A pixel check needs to know **where** to look. There are three sources, in
//! priority order, and the third is what stops a false pass:
//!
//! 1. **The application says so**, by tracing a `ui-rect` event naming a
//!    region and giving its rect. This is the right answer, and as of S2 the
//!    new application implements it: it survives every layout change, because
//!    the rect is measured on the frame it is reported for. Consulted
//!    **first** — see [`crate::checks::legibility::resolve_set`] for the
//!    precedence and the argument for that order.
//! 2. **A calibrated [`RegionSet`]** in this file — fractions of a surface,
//!    with the surface it was calibrated against recorded alongside.
//! 3. **Neither**, in which case the check reports SKIPPED naming what is
//!    missing. It does not guess, and it does not pass.
//!
//! [`Calibration`] is the guard on source 2. A region set measured against a
//! dated 1860×1035 evidence crop describes *that image*, and applying it to a
//! live 2560×1440 window would sample whatever happens to be at those
//! fractions — producing a contrast number that is a real measurement of the
//! wrong thing. Numbers like that are worse than no numbers, so the check
//! refuses the mismatch and says which surface the set was calibrated for.
//!
//! ## Where the trace is *interpreted*, and why it is here rather than in
//! [`crate::trace`]
//!
//! [`crate::trace`] is deliberately meaning-free: it turns bytes into
//! `key=value` pairs and stops. It does not know that `objects` carries a
//! count or that `ui-rect` declares a region, because those facts are
//! properties of a *particular binary's* vocabulary and the parser must be
//! aimable at more than one.
//!
//! So the two readers a check needs — [`Vocabulary::declared_regions`] and
//! [`Vocabulary::object_count`] — live on [`Vocabulary`], which is exactly the
//! type that knows the spellings. They are thin: both go through
//! [`crate::trace::TraceLine`]'s accessors, so there is still only one piece
//! of code in the crate that knows what `[[x0 y0] - [x1 y1]]` means.

use crate::geom::{FracRect, LRect};
use crate::trace::Trace;

/// The trace field names a check needs, by concept.
///
/// Every field is the *name the target binary uses*, not a name this crate
/// invents. See the module docs.
#[derive(Clone, Copy, Debug)]
pub struct Vocabulary {
    /// The unconditional first line — how the harness tells "the diagnostic
    /// switch never reached the process" from "the process had nothing to
    /// say".
    pub start_event: &'static str,
    /// The event carrying the canvas's laid-out geometry.
    pub canvas_event: &'static str,
    /// Field on it holding the canvas image rect.
    pub canvas_rect_field: &'static str,
    /// Field on it holding the view magnification.
    pub canvas_zoom_field: &'static str,
    /// Field on it holding the scroll offset, if the profile has established
    /// that the rect does not already account for it. See
    /// [`crate::coords`]'s "assumed, and NOT verified" section — including the
    /// experiment that would settle it. `None` means "treated as zero", which
    /// is correct for an unscrolled view either way.
    pub canvas_scroll_field: Option<&'static str>,
    /// Field on it holding the current selection size.
    pub canvas_selection_field: &'static str,
    /// The event emitted when a click is resolved against the page.
    pub click_event: &'static str,
    /// Field on it holding how many objects the hit test found.
    pub click_hits_field: &'static str,
    /// Field on it holding the resulting selection size.
    pub click_selection_field: &'static str,
    /// The event emitted when objects are deleted.
    pub delete_event: &'static str,
    /// Field on it holding how many were deleted.
    pub delete_count_field: &'static str,
    /// An event reporting the page's total object count, if the binary has
    /// one. Strictly better evidence than the delete event, because it
    /// measures the property the check is actually about rather than the verb
    /// that was meant to change it.
    pub object_count_event: Option<&'static str>,
    /// Field on that event holding the count.
    pub object_count_field: &'static str,
    /// An event by which the application declares a named UI rectangle, if it
    /// has one — region source 1 in the module docs.
    pub ui_rect_event: Option<&'static str>,
    /// Field on it holding the region name.
    pub ui_rect_name_field: &'static str,
    /// Field on it holding the rectangle.
    pub ui_rect_rect_field: &'static str,
}

impl Vocabulary {
    /// The vocabulary of the binary this project is **building**.
    ///
    /// Every name here was read out of `crates/pdfcer-gui/src`, not guessed:
    ///
    /// * `canvas rect= zoom= page= pages= off=` — `canvas/mod.rs`, traced
    ///   through the de-duplicating gate so there is one line per document
    ///   open and one more per layout change (`PROJECT_PLAN.md` §4.3
    ///   requirement 1, landed at S2 — which is why the layout-probe click in
    ///   [`crate::checks::delete_key`] is no longer needed against this
    ///   binary, only against the old one).
    /// * `ui-rect name= rect=` — `diag.rs::ui_rect` (§4.3 requirement 2).
    /// * `objects n= page= paths= text= images= forms=` —
    ///   `app/state.rs::trace_object_count` (§4.3 requirement 3).
    /// * `start` — emitted unconditionally, so an empty trace can be told
    ///   from a trace the diagnostic switch never reached.
    ///
    /// ## Two fields that name events this binary does not emit yet
    ///
    /// `click_event` and `delete_event` keep the old binary's spellings. That
    /// is not an oversight and it is not a claim that this binary emits them:
    /// a vocabulary entry is the name a check will *look for*, and looking for
    /// a name that is absent is how a check discovers a subsystem has not been
    /// built and reports SKIP. Nothing is gained by blanking them — a `None`
    /// there would produce the same SKIP with a vaguer reason.
    ///
    /// Likewise `canvas_selection_field`. This binary deliberately does **not**
    /// emit `sel=` on its `canvas` line, because there is no selection
    /// subsystem at S2 and `sel=0` would be a false statement about the
    /// document that turns an honest SKIP into a FAIL blaming code nobody has
    /// written. [`crate::checks::delete_key`] handles the absence explicitly;
    /// see the `(None, None)` arm there.
    #[must_use]
    pub const fn pdfcer_gui() -> Self {
        Self {
            start_event: "start",
            canvas_event: "canvas",
            canvas_rect_field: "rect",
            canvas_zoom_field: "zoom",
            canvas_scroll_field: None,
            canvas_selection_field: "sel",
            click_event: "vector-click",
            click_hits_field: "hits",
            click_selection_field: "newsel",
            delete_event: "delete-objects",
            delete_count_field: "n",
            object_count_event: Some("objects"),
            object_count_field: "n",
            ui_rect_event: Some("ui-rect"),
            ui_rect_name_field: "name",
            ui_rect_rect_field: "rect",
        }
    }

    /// The vocabulary of the OLD binary at `D:\Dev\pdfcer`.
    ///
    /// Every name was read out of `D:\Dev\pdfce\crates\pdfce-gui\src` at the
    /// 2026-08-12 release build:
    ///
    /// * `canvas … rect= zoom= sel=` — `main.rs:16866`, traced only on pointer
    ///   events
    /// * `vector-click … hits= newsel=` — `main.rs:22163`
    /// * `delete-objects n=` — `main.rs:5293`
    /// * `start` — `main.rs:624`, emitted unconditionally
    ///
    /// `object_count_event` and `ui_rect_event` are `None` because this binary
    /// has neither, and **that is load-bearing rather than incidental**. It is
    /// what keeps the D1 reproduction honest: with no object count to read,
    /// [`crate::checks::delete_key`] falls back to the weaker
    /// absence-of-`delete-objects` oracle, which is the oracle that produced
    /// the recorded FAIL. If this profile were given the new binary's
    /// vocabulary, the check would look for a count that is never emitted and
    /// the fallback would still fire — but the *reason string* would then name
    /// an event this binary cannot produce, and a skip or failure reason that
    /// misidentifies the blocked component sends the reader to the wrong file.
    #[must_use]
    pub const fn pdfcer_legacy() -> Self {
        Self {
            start_event: "start",
            canvas_event: "canvas",
            canvas_rect_field: "rect",
            canvas_zoom_field: "zoom",
            canvas_scroll_field: None,
            canvas_selection_field: "sel",
            click_event: "vector-click",
            click_hits_field: "hits",
            click_selection_field: "newsel",
            delete_event: "delete-objects",
            delete_count_field: "n",
            object_count_event: None,
            object_count_field: "n",
            ui_rect_event: None,
            ui_rect_name_field: "name",
            ui_rect_rect_field: "rect",
        }
    }

    /// Every named region the application declared in this trace.
    ///
    /// ## The shape of the evidence
    ///
    /// ```text
    /// pdfcer-diag ui-rect name=central-panel   rect=[[8.0 8.0] - [1092.0 792.0]]
    /// pdfcer-diag ui-rect name=page            rect=[[16.0 22.8] - [1084.0 777.2]]
    /// pdfcer-diag ui-rect name=canvas-viewport rect=[[8.0 8.0] - [1092.0 792.0]]
    /// ```
    ///
    /// The rects are in **window logical points**, relative to the client
    /// area's top-left — the same space the `canvas` event's `rect=` is in,
    /// and the same space [`crate::coords::WindowFrame`] converts out of. They
    /// are *not* device pixels and they are *not* desktop coordinates;
    /// [`crate::coords::WindowFrame::logical_to_capture_pixels`] is the one
    /// conversion that turns them into something a screenshot can be sampled
    /// with.
    ///
    /// ## Last declaration wins
    ///
    /// The application re-declares a region whenever it moves or resizes, and
    /// suppresses the frames in between. So a name may appear many times in
    /// one trace and only the final one describes the window as it now
    /// stands. Returning the last is the same rule [`Trace::last`] applies to
    /// every other "what is the current value of X?" question.
    ///
    /// ## A missing or unparsable `rect=` is dropped, not zeroed
    ///
    /// A `ui-rect` line the parser cannot read is a harness-side parse bug or
    /// a vocabulary drift, and either way the honest thing to do is to not
    /// pretend a region was declared. It is never a zero-area region: a
    /// zero-area region would be measured, would sample nothing, and would be
    /// reported as an invisible caption — a false FAIL manufactured out of a
    /// parse failure.
    ///
    /// Returns the regions sorted by name, so a report reads the same way
    /// twice and a diff between two runs is a diff about the application.
    #[must_use]
    pub fn declared_regions(&self, trace: &Trace) -> Vec<DeclaredRegion> {
        let Some(event) = self.ui_rect_event else {
            return Vec::new();
        };
        let mut by_name: std::collections::BTreeMap<String, DeclaredRegion> =
            std::collections::BTreeMap::new();
        for line in trace.events(event) {
            let (Some(name), Some(rect)) = (
                line.get(self.ui_rect_name_field),
                line.get_rect(self.ui_rect_rect_field),
            ) else {
                continue;
            };
            let name = name.trim().to_owned();
            if name.is_empty() {
                continue;
            }
            by_name.insert(
                name.clone(),
                DeclaredRegion {
                    name,
                    rect,
                    lineno: line.lineno,
                },
            );
        }
        by_name.into_values().collect()
    }

    /// The page object count the application last reported, if it reports one.
    ///
    /// `None` covers three situations that the caller must **not** collapse
    /// into "zero objects":
    ///
    /// 1. this binary has no object-count event in its vocabulary;
    /// 2. it has one and has not emitted it yet;
    /// 3. it emitted `objects-unavailable page=… reason=…` instead, because
    ///    the page's content streams would not decode.
    ///
    /// The third is why the application's contract is that failure is a
    /// *different event* rather than the success event with a missing field:
    /// an `objects` line is a claim that the count was measured, so a check
    /// comparing before against after can trust it. A missing `n=` on an
    /// `objects` line is therefore a harness-side parse bug, and reading it as
    /// zero would turn a parse bug into "the page is empty" — a confident,
    /// wrong statement about the document.
    #[must_use]
    pub fn object_count(&self, trace: &Trace) -> Option<usize> {
        let event = self.object_count_event?;
        trace
            .last(event)
            .and_then(|l| l.get_usize(self.object_count_field))
    }
}

/// One named region the application declared, as it declared it.
///
/// The rect is in **window logical points** — see
/// [`Vocabulary::declared_regions`].
#[derive(Clone, Debug, PartialEq)]
pub struct DeclaredRegion {
    /// The name the application chose. Matched literally by checks, so it is
    /// part of the contract between the two.
    pub name: String,
    /// Where it was, on the frame it was reported for.
    pub rect: LRect,
    /// Which line of the capture said so, for a report that wants to point at
    /// the evidence rather than paraphrase it.
    pub lineno: usize,
}

/// What surface a [`RegionSet`]'s fractions were measured against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Calibration {
    /// A specific image file, named relative to the repository root.
    ///
    /// A set with this calibration may only be used in `--image` mode, against
    /// that image. See the module docs for why a mismatch is refused.
    Image(&'static str),
    /// The live application window, at any size — the fractions are stable
    /// because the layout is proportional.
    ///
    /// A claim, and a strong one. Only use it for a set that has been checked
    /// at more than one window size.
    LiveWindow,
}

/// One named rectangle to measure.
#[derive(Clone, Copy, Debug)]
pub struct NamedRegion {
    /// What is expected to be drawn here — used verbatim in the report, so
    /// write it as the operator would recognise it.
    pub name: &'static str,
    /// Where, as a fraction of the calibrated surface.
    pub area: FracRect,
}

/// A group of regions with a shared calibration.
#[derive(Clone, Copy, Debug)]
pub struct RegionSet {
    /// Which check asks for this set.
    pub name: &'static str,
    /// What the fractions were measured against.
    pub calibrated_for: Calibration,
    /// How they were measured, in a sentence. Printed when the set is used, so
    /// a reader can judge how much to trust a number derived from it.
    pub provenance: &'static str,
    /// The regions.
    pub regions: &'static [NamedRegion],
}

/// A target binary and everything the harness knows about it.
#[derive(Clone, Copy, Debug)]
pub struct Profile {
    /// Selected with `--profile`.
    pub name: &'static str,
    /// One line, printed by `--list`.
    pub description: &'static str,
    /// Where the binary usually is, if `--exe` is not given.
    pub default_exe: &'static str,
    /// The environment variable that turns the trace on, and its value.
    pub diag_env: (&'static str, &'static str),
    /// The marker at the head of every diagnostic line.
    pub trace_prefix: &'static str,
    /// An environment variable that places the window, if the binary honours
    /// one. Not required: the harness measures the window wherever it lands.
    pub viewport_env: Option<&'static str>,
    /// Trace field names.
    pub vocab: Vocabulary,
    /// Calibrated region sets.
    pub region_sets: &'static [RegionSet],
}

impl Profile {
    /// The region set a check asks for, by name.
    #[must_use]
    pub fn region_set(&self, name: &str) -> Option<&'static RegionSet> {
        self.region_sets.iter().find(|s| s.name == name)
    }
}

/// Every profile the harness knows.
#[must_use]
pub fn all() -> &'static [Profile] {
    &[PDFCER_GUI, PDFCER_LEGACY]
}

/// Look a profile up by name.
#[must_use]
pub fn by_name(name: &str) -> Option<&'static Profile> {
    all().iter().find(|p| p.name == name)
}

/// The application this project is building. The default target.
///
/// It exists as of S2 and it speaks all three of the dialects
/// `PROJECT_PLAN.md` §4.3 asked it for: an unconditional `canvas` line, a
/// `ui-rect` line per named region, and an `objects` count. What it does
/// **not** have yet is a ribbon, a selection subsystem or a Settings dialog —
/// so the checks that need those still report SKIPPED, and the reason each
/// gives now names the missing *subsystem* rather than the missing trace
/// channel. That difference matters: a reason that blamed the trace channel
/// would send a reader to `diag.rs`, which is finished.
pub const PDFCER_GUI: Profile = Profile {
    name: "pdfcer-gui",
    description: "the application this project is building (crates/pdfcer-gui)",
    default_exe: "target/release/pdfcer-gui.exe",
    diag_env: ("PDFCER_DIAG", "1"),
    trace_prefix: "pdfcer-diag",
    viewport_env: Some("PDFCER_DIAG_VIEWPORT"),
    vocab: Vocabulary::pdfcer_gui(),
    // No region sets, deliberately, and now permanently rather than
    // provisionally. The right source for this binary is region source 1 —
    // the application tracing its own `ui-rect` events, which as of S2 it
    // does — because a rect it measures on the frame it reports is correct
    // under every layout change, and a fraction written here is correct until
    // the first panel is resized. Adding fractions now would create exactly
    // the stale-coordinate hazard `crate::coords` exists to prevent, and it
    // would do so *while a correct source is available*, which is worse: the
    // fraction would be consulted only when the trace source found nothing,
    // i.e. precisely when the harness has no idea what the window looks like.
    region_sets: &[],
};

/// The GUI this project replaces, at `D:\Dev\pdfce` — the
/// **pre-rename** repository, which is where it still is and where it stays.
///
/// **Read-only, always.** The harness launches it and photographs it; nothing
/// in this crate writes anywhere near it.
///
/// Its purpose here is falsification. A check suite is only evidence if it has
/// been seen to fail on a known-defective build, and this profile is how that
/// is demonstrated.
///
/// # ★★★ EVERY NAME IN THIS PROFILE IS AN OLD NAME, DELIBERATELY
///
/// This is the one place in the crate where the old stem is *correct*, and it
/// went wrong on 2026-09-03 in exactly the way this project has a memory for:
/// **a rename can blind an instrument silently.** The project-wide sweep
/// rewrote all four of the fields below, and every one of them names something
/// **outside this repository** that did not rename with us:
///
/// | field | swept to | actually |
/// |---|---|---|
/// | `default_exe` | `\pdfcer\…\pdfcer-gui.exe` | `\pdfce\…\pdfce-gui.exe` |
/// | `diag_env` | `PDFCER_DIAG` | `PDFCE_DIAG` |
/// | `trace_prefix` | `pdfcer-diag` | `pdfce-diag` |
/// | `viewport_env` | `PDFCER_DIAG_VIEWPORT` | `PDFCE_DIAG_VIEWPORT` |
///
/// The swept `default_exe` is worse than merely wrong: the engine's
/// `Pass 247.0` **stripped the in-repo GUI crate** from the new repository, so
/// that path can never exist — the falsification profile was pointing at a
/// binary nothing will ever build. The three trace names would each have failed
/// *quietly*, which is worse still: an environment variable the old binary does
/// not read simply leaves diagnostics off, and a trace prefix that does not
/// match reads an EMPTY trace — indistinguishable from a build that emitted
/// nothing.
///
/// Guarded by `legacy_profile_names_the_pre_rename_gui`, which is the
/// mechanism; this comment is only the reason.
pub const PDFCER_LEGACY: Profile = Profile {
    name: "pdfcer-legacy",
    // old-name-exempt: the old GUI's own repository, which did not rename.
    description: "the OLD GUI at D:\\Dev\\pdfce — the known-defective build the checks must fail against",
    // old-name-exempt: the pre-rename binary, in the pre-rename repository.
    default_exe: r"D:\Dev\pdfce\target\release\pdfce-gui.exe",
    // old-name-exempt: the variable the OLD binary reads. Renaming it turns
    // its diagnostics off silently, which reads as a build that says nothing.
    diag_env: ("PDFCE_DIAG", "1"),
    // old-name-exempt: the old GUI's `diag.rs:746` prints this exact prefix.
    trace_prefix: "pdfce-diag",
    // old-name-exempt: the old GUI's `main.rs:601` reads this exact spelling.
    viewport_env: Some("PDFCE_DIAG_VIEWPORT"),
    vocab: Vocabulary::pdfcer_legacy(),
    region_sets: &[SETTINGS_HEADINGS_LEGACY],
};

/// The Settings dialog's section headings in the old GUI — D2's subjects.
///
/// # Provenance, stated in full because a calibrated number is only as good as
/// its provenance
///
/// Measured against `evidence/crop_settings.png` (1860×1035), the dated
/// artefact `DEFECTS.md` D2 cites as its evidence. That file is a crop of the
/// old GUI's Settings dialog captured on 2026-08-12; the seven headings listed
/// below are the seven `DEFECTS.md` names in the order they appear down the
/// dialog.
///
/// The calibration is [`Calibration::Image`], so these fractions can only be
/// used against that image. They are **not** valid against a live window of
/// the old GUI: the crop is a sub-rectangle of the dialog, not the client
/// area, so the same fractions would sample the wrong part of a live capture.
/// Driving the live old binary to its Settings dialog needs its own
/// calibration pass, and until somebody does that pass the live-mode check
/// SKIPs saying so — which is the honest report, and is not the same as a pass.
const SETTINGS_HEADINGS_LEGACY: RegionSet = RegionSet {
    name: "settings_headings",
    calibrated_for: Calibration::Image("evidence/crop_settings.png"),
    provenance: "measured off evidence/crop_settings.png (1860x1035), the dated artefact DEFECTS.md D2 cites",
    regions: &[
        NamedRegion {
            name: "Appearance",
            area: FracRect::new(0.030, 0.000, 0.160, 0.042),
        },
        NamedRegion {
            name: "Theme",
            area: FracRect::new(0.033, 0.066, 0.105, 0.108),
        },
        NamedRegion {
            name: "Colour",
            area: FracRect::new(0.033, 0.594, 0.105, 0.638),
        },
        NamedRegion {
            name: "Images and transparency",
            area: FracRect::new(0.033, 0.681, 0.272, 0.725),
        },
        NamedRegion {
            name: "Copying and extracting text",
            area: FracRect::new(0.033, 0.768, 0.298, 0.812),
        },
        NamedRegion {
            name: "Pages and printing",
            area: FracRect::new(0.033, 0.855, 0.215, 0.899),
        },
        NamedRegion {
            name: "Saving files",
            area: FracRect::new(0.033, 0.941, 0.145, 0.985),
        },
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_profile_is_findable_by_its_own_name() {
        for p in all() {
            assert!(by_name(p.name).is_some(), "{} is not findable", p.name);
        }
    }

    #[test]
    fn the_legacy_settings_region_set_is_image_calibrated() {
        let set = PDFCER_LEGACY
            .region_set("settings_headings")
            .expect("the legacy profile carries a settings region set");
        assert!(
            matches!(set.calibrated_for, Calibration::Image(_)),
            "these fractions describe a crop, not a live client area; claiming otherwise \
             would sample the wrong pixels and report a real measurement of the wrong thing"
        );
        assert_eq!(set.regions.len(), 7, "DEFECTS.md D2 names seven headings");
    }

    #[test]
    fn the_new_profile_declares_no_stale_fractions() {
        assert!(
            PDFCER_GUI.region_sets.is_empty(),
            "the new application traces its own ui-rect regions; hard-coded fractions \
             would be stale the first time a panel is resized"
        );
    }

    /// The two vocabularies must NOT be the same, and the difference is the
    /// whole point of having two: the old binary cannot emit what it does not
    /// implement, and a harness that asked it for an object count would name
    /// an event it cannot produce in its own failure text.
    #[test]
    fn the_old_binary_is_not_credited_with_the_new_ones_trace_channels() {
        assert_eq!(PDFCER_LEGACY.vocab.object_count_event, None);
        assert_eq!(PDFCER_LEGACY.vocab.ui_rect_event, None);
        assert_eq!(PDFCER_GUI.vocab.object_count_event, Some("objects"));
        assert_eq!(PDFCER_GUI.vocab.ui_rect_event, Some("ui-rect"));
    }

    /// The exact lines the S2 application emits, verbatim from a real capture.
    #[test]
    fn ui_rect_lines_are_read_as_named_regions() {
        let trace = Trace::parse(
            "pdfcer-diag ui-rect name=central-panel rect=[[8.0 8.0] - [1092.0 792.0]]\n\
             pdfcer-diag ui-rect name=page            rect=[[16.0 22.8] - [1084.0 777.2]]\n\
             pdfcer-diag ui-rect name=canvas-viewport rect=[[8.0 8.0] - [1092.0 792.0]]",
            "pdfcer-diag",
        );
        let regions = Vocabulary::pdfcer_gui().declared_regions(&trace);
        let names: Vec<&str> = regions.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["canvas-viewport", "central-panel", "page"]);
        let page = regions.iter().find(|r| r.name == "page").unwrap();
        assert!((page.rect.width() - 1068.0).abs() < 0.01);
        assert!((page.rect.height() - 754.4).abs() < 0.01);
    }

    /// A region moves; the harness must measure where it is NOW.
    #[test]
    fn the_last_declaration_of_a_region_wins() {
        let trace = Trace::parse(
            "pdfcer-diag ui-rect name=page rect=[[0.0 0.0] - [10.0 10.0]]\n\
             pdfcer-diag ui-rect name=page rect=[[5.0 5.0] - [40.0 40.0]]",
            "pdfcer-diag",
        );
        let regions = Vocabulary::pdfcer_gui().declared_regions(&trace);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].rect.width(), 35.0);
    }

    /// A `ui-rect` the parser cannot read is dropped rather than admitted as a
    /// zero-area region. A zero-area region would be measured, would sample
    /// nothing, and would be reported as an invisible caption — a false FAIL
    /// manufactured out of a parse failure.
    #[test]
    fn an_unparsable_ui_rect_is_dropped_not_zeroed() {
        let trace = Trace::parse(
            "pdfcer-diag ui-rect name=broken rect=nonsense\n\
             pdfcer-diag ui-rect rect=[[0.0 0.0] - [4.0 4.0]]",
            "pdfcer-diag",
        );
        assert!(
            Vocabulary::pdfcer_gui().declared_regions(&trace).is_empty(),
            "neither line declares a readable named region"
        );
    }

    /// A binary with no `ui-rect` in its vocabulary declares nothing, however
    /// many `ui-rect` lines happen to be in the capture.
    #[test]
    fn a_binary_without_the_event_declares_no_regions() {
        let trace = Trace::parse(
            "pdfcer-diag ui-rect name=page rect=[[0.0 0.0] - [10.0 10.0]]",
            "pdfcer-diag",
        );
        assert!(
            Vocabulary::pdfcer_legacy()
                .declared_regions(&trace)
                .is_empty()
        );
    }

    #[test]
    fn the_object_count_is_read_from_the_last_objects_line() {
        let trace = Trace::parse(
            "pdfcer-diag objects n=28 page=0 paths=13 text=15 images=0 forms=0\n\
             pdfcer-diag objects n=27 page=0 paths=12 text=15 images=0 forms=0",
            "pdfcer-diag",
        );
        assert_eq!(Vocabulary::pdfcer_gui().object_count(&trace), Some(27));
    }

    /// Failure is a *different event*, and it must not read as a count.
    /// `objects-unavailable` means "the count could not be measured", which is
    /// not "the page has no objects" — and a check that read it as zero would
    /// report a deletion that never happened.
    #[test]
    fn an_unavailable_page_reports_no_count_rather_than_zero() {
        let trace = Trace::parse(
            "pdfcer-diag objects-unavailable page=0 reason=decompose-failed detail=eof",
            "pdfcer-diag",
        );
        assert_eq!(Vocabulary::pdfcer_gui().object_count(&trace), None);
    }

    #[test]
    fn a_binary_without_an_object_count_event_reports_none() {
        let trace = Trace::parse("pdfcer-diag objects n=28 page=0", "pdfcer-diag");
        assert_eq!(Vocabulary::pdfcer_legacy().object_count(&trace), None);
    }
    /// The falsification profile must keep naming the PRE-RENAME GUI.
    ///
    /// # Why this test exists
    ///
    /// On 2026-09-03 the project-wide `pdfce` -> `pdfcer` sweep rewrote all
    /// four external names in [`PDFCER_LEGACY`]. Nothing went red. The old
    /// GUI does not live in this repository and did not rename, so:
    ///
    /// * the exe path came to name a binary in the ENGINE repository, whose
    ///   `Pass 247.0` had just deleted the only GUI crate it ever had --
    ///   a path that can never exist;
    /// * the diagnostic environment variable came to name one the old binary
    ///   does not read, which leaves its tracing OFF;
    /// * the trace prefix came to name one the old binary never prints, which
    ///   parses to an EMPTY trace.
    ///
    /// Each of the last two is silent. An empty trace and a build that said
    /// nothing are the same bytes, so the falsification suite would have
    /// reported "the old build does not exhibit the defect" -- the exact
    /// inversion the suite exists to prevent -- with every gate green.
    ///
    /// This asserts the shape rather than the spelling: the four fields must
    /// carry the old stem and must NOT carry the new one. It is deliberately
    /// a test and not a comment, because a comment is what was there.
    #[test]
    fn legacy_profile_names_the_pre_rename_gui() {
        // Built rather than written, so this file carries no literal that a
        // future sweep could helpfully "correct".
        let old_stem = "pdfce";
        let new_stem = "pdfcer";

        // `contains(old_stem)` is true of the new stem as well -- "pdfcer"
        // CONTAINS "pdfce" -- so the honest question is whether the new stem
        // appears at all. That asymmetry is the whole reason the rename needed
        // a gate in the first place.
        for (field, value) in [
            ("default_exe", PDFCER_LEGACY.default_exe),
            ("diag_env", PDFCER_LEGACY.diag_env.0),
            ("trace_prefix", PDFCER_LEGACY.trace_prefix),
            (
                "viewport_env",
                PDFCER_LEGACY
                    .viewport_env
                    .expect("legacy profile declares a viewport env var"),
            ),
        ] {
            let lower = value.to_ascii_lowercase();
            assert!(
                lower.contains(old_stem),
                "PDFCER_LEGACY.{field} = {value:?} does not name the old GUI at all"
            );
            assert!(
                !lower.contains(new_stem),
                "PDFCER_LEGACY.{field} = {value:?} was swept to the NEW name. \
                 The old GUI is in another repository and did not rename; \
                 see this constant's doc comment for what each wrong name \
                 breaks, and note that three of the four break SILENTLY."
            );
        }

        // And the exe must be in the OLD repository, not the engine's new one.
        // Spelled as a path fragment because the failure that happened was a
        // correct-looking path in the wrong tree.
        let exe = PDFCER_LEGACY.default_exe.to_ascii_lowercase();
        assert!(
            exe.contains("dev\\pdfce\\target"),
            "PDFCER_LEGACY.default_exe = {:?} is not under the pre-rename repository",
            PDFCER_LEGACY.default_exe
        );
    }
    /// This build's own profile must name THIS project, not the old one.
    ///
    /// The other half of the file-level exemption at the top of this file.
    /// `legacy_profile_names_the_pre_rename_gui` asserts the falsification
    /// profile still points at the pre-rename build; this asserts that the
    /// exemption did not become a place where a genuine rename miss could
    /// hide.
    ///
    /// It is the strictly harder direction, and it is the one a grep cannot
    /// ask: the new stem CONTAINS the old one, so "does this line mention
    /// pdfce" is true of every correct line as well as every stale one. A
    /// test can compare the exact bytes.
    #[test]
    fn current_profile_names_only_the_new_project() {
        let new_stem = "pdfcer";

        for (field, value) in [
            ("name", PDFCER_GUI.name),
            ("diag_env", PDFCER_GUI.diag_env.0),
            ("trace_prefix", PDFCER_GUI.trace_prefix),
            (
                "viewport_env",
                PDFCER_GUI
                    .viewport_env
                    .expect("this build declares a viewport env var"),
            ),
        ] {
            let lower = value.to_ascii_lowercase();
            assert!(
                lower.contains(new_stem),
                "PDFCER_GUI.{field} = {value:?} does not name this project"
            );
            // Every occurrence of the old stem must be the PREFIX of a new
            // one -- i.e. followed by an `r`. That is the same lookahead
            // `tools/gates/check-old-name-absent.sh` uses, and it is the only
            // honest form of the question.
            let bytes = lower.as_bytes();
            for (i, _) in lower.match_indices("pdfce") {
                let next = bytes.get(i + 5).copied();
                assert_eq!(
                    next,
                    Some(b'r'),
                    "PDFCER_GUI.{field} = {value:?} carries the OLD project name at byte {i}"
                );
            }
        }
    }
}
