//! # `printverdicts` — what the operator has actually LOOKED at
//!
//! ## The contradiction this module exists to remove
//!
//! Operator request O113 made the preview's clip hatch **ink-aware**: on a 1:1
//! CAD sheet whose overhang is empty paper, nothing is hatched and the caption
//! says *"This sheet hangs over the printable area, but nothing is printed
//! there — the overhang is blank."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/printverdicts.md`.

use std::collections::BTreeMap;

use crate::printpreviewkey::{Overhang, PreviewKey};
use crate::printspooler::{Job, PagePlan, Placement};
use crate::text::print as t;

/// The job-wide half of a verdict's key, and the one place a
/// [`PreviewKey`] is built.
#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    /// Which annotation classes are painted — `PreviewKey`'s field.
    scope: pdfcer_render::AnnotationScope,
    /// The operator's configuration, whole — `PreviewKey`'s field.
    settings: pdfcer_core::settings::Settings,
    /// The printable area in ce dimensions, from the planned device geometry.
    printable_pt: (f64, f64),
    /// The fixed line width on paper, points (O233); `None` when off.
    lines_pt: Option<f64>,
}

impl Context {
    /// Snapshot the frame's rendering inputs and printable rectangle.
    pub fn new(
        scope: pdfcer_render::AnnotationScope,
        settings: &pdfcer_core::settings::Settings,
        printable_pt: (f64, f64),
        lines_pt: Option<f64>,
    ) -> Self {
        Self {
            scope,
            settings: settings.clone(),
            printable_pt,
            lines_pt,
        }
    }

    /// The preview texture's cache key for `page`.
    pub fn preview_key(&self, page: usize, placement: f64) -> PreviewKey {
        PreviewKey::new(
            page,
            self.scope,
            &self.settings,
            self.lines_pt.map(|pt| (pt, placement)),
        )
    }
}

/// The per-sheet half of a verdict's key: **where the band falls**.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Sheet {
    /// Scale and offset within the printable area.
    placement: Placement,
    /// The page's own size in pdf dimensions.
    ///
    /// Present because the band is computed as a *fraction of the page*: the
    /// same placement over a page of a different size is a different band. A
    /// document edit that resizes a page therefore drops the verdict — into
    /// "unexamined", which is the safe direction.
    page_pt: (f64, f64),
}

impl Sheet {
    /// The identity of the sheet `plan` describes, or `None` when the plan
    /// names a page the document no longer has.
    fn of(plan: &PagePlan, page_sizes: &[(f64, f64)]) -> Option<Self> {
        Some(Self {
            placement: plan.placement,
            page_pt: *page_sizes.get(plan.index)?,
        })
    }
}

/// What the preview has found, for the sheets it has been shown.
#[derive(Debug, Default)]
pub struct Verdicts {
    /// The context every entry below was recorded under.
    ///
    /// One field for the whole map rather than a copy in every entry: the
    /// context is job-wide, so when it changes **every** verdict is void at
    /// once, and holding it once makes that a single comparison and a single
    /// `clear` instead of a per-entry test that could be forgotten.
    context: Option<Context>,
    /// Document page index → the sheet identity it was recorded for, and what
    /// the ink test found in its overhang.
    ///
    /// A `BTreeMap` rather than a `HashMap` for a small, boring reason: it
    /// makes iteration order deterministic, so a test over the map reads the
    /// same on every run. Nothing here iterates it in anger.
    seen: BTreeMap<usize, (Sheet, Overhang)>,
}

impl Verdicts {
    /// Record what the preview just found in the overhang of one sheet.
    pub fn remember(
        &mut self,
        context: &Context,
        plan: &PagePlan,
        page_sizes: &[(f64, f64)],
        overhang: Overhang,
    ) {
        let Some(sheet) = Sheet::of(plan, page_sizes) else {
            // The plan names a page the document does not have. There is no
            // sheet to be a fact about; the preview is already drawing the
            // honest empty picture for it.
            return;
        };
        if self.context.as_ref() != Some(context) {
            self.seen.clear();
            self.context = Some(context.clone());
        }
        self.seen.insert(plan.index, (sheet, overhang));
    }

    /// What is known about `plan`'s overhang **right now**, or `None` when
    /// nothing is.
    fn verdict(
        &self,
        context: &Context,
        plan: &PagePlan,
        page_sizes: &[(f64, f64)],
    ) -> Option<Overhang> {
        if self.context.as_ref() != Some(context) {
            return None;
        }
        let sheet = Sheet::of(plan, page_sizes)?;
        let &(recorded, overhang) = self.seen.get(&plan.index)?;
        (recorded == sheet).then_some(overhang)
    }

    /// **The number on the button, and what may honestly be said about it.**
    pub fn claim(&self, context: &Context, job: &Job, page_sizes: &[(f64, f64)]) -> ClipClaim {
        let mut geometric = 0usize;
        let mut known_blank = 0usize;
        let mut unresolved = 0usize;
        for plan in &job.plans {
            // The cheap geometric gate, kept as the gate. A sheet that fits
            // cannot lose anything and is not in any of the three buckets.
            if !plan.placement.clipped {
                continue;
            }
            geometric += 1;
            match self.verdict(context, plan, page_sizes) {
                Some(Overhang::BlankBand) => known_blank += 1,
                Some(Overhang::Losing) => {}
                Some(Overhang::Unknown | Overhang::Fits) | None => unresolved += 1,
            }
        }
        ClipClaim::from_counts(geometric, known_blank, unresolved)
    }
}

/// **How many sheets will lose content, and how well that is known.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipClaim {
    /// Nothing to disclose. Either no sheet's page box exceeds the printable
    /// area, or every one that does has been examined and found blank — the
    /// operator's 1:1 CAD drawing, after one look at the preview.
    None,
    /// **Nothing has been subtracted**, so the plain geometric fact still
    /// stands exactly: this many sheets have a page box exceeding the printable
    /// rectangle. Said in the words it has always been said in.
    ///
    /// THE RULING ON PRINTING WITHOUT EVER OPENING THE PREVIEW, and it is
    /// a decision rather than an accident of the arithmetic.
    ///
    /// The preview column is drawn whenever the dialog is open, so "never
    /// previewed" means the operator has not stepped to the offending sheet —
    /// on a multi-sheet job, the common case. Every clipped sheet is then
    /// unexamined, `known_blank` is zero, and this variant carries the
    /// unchanged geometric count with the unchanged wording.
    ///
    /// **That is deliberate, and the alternative was refused.** Making the
    /// button hedge — *"up to N sheets may lose content"* — in a state where
    /// nothing has been looked at would replace an exactly true statement with
    /// a bounded one for no gain, and would soften the disclosure that carries
    /// pdfcer's whole divergence from Acrobat (which clips silently) in
    /// exactly the state where there is no evidence to soften it with. Where
    /// knowledge exists the count and the sentence both improve; where none
    /// exists, nothing changes — **byte for byte, this is the behaviour that
    /// shipped before O113**, which is the honest degradation.
    Geometric(usize),
    /// **Every** clipped sheet has been examined and this many carry ink in
    /// the overhang. An exact, measured content claim — the only state in
    /// which the stronger sentence *"will lose content"* is earned.
    Measured(usize),
    /// Some sheets are known blank and some have not been looked at. The
    /// number is `known_inked + unexamined`, which is a **ceiling** on what
    /// will actually be lost, so the sentence hedges — see this module's
    /// header for the inequality.
    AtMost(usize),
}

impl ClipClaim {
    /// Turn the three bucket totals into a claim.
    ///
    /// # The order of the arms IS the argument
    ///
    /// ```text
    /// count = geometric − known_blank
    /// ```
    ///
    /// 1. `count == 0` — every clipped sheet was looked at and none loses
    ///    anything, or nothing is clipped at all. Say nothing.
    /// 2. `unresolved == 0` — every clipped sheet has a definite verdict, so
    ///    `count` is exactly the number that will lose ink. **Measured**, and
    ///    it is tested before the geometric arm on purpose: when all of them
    ///    are inked the two counts coincide, and reporting the weaker
    ///    geometric sentence there would throw away knowledge that was
    ///    actually obtained.
    /// 3. `known_blank == 0` — nothing has been subtracted, so `count` still
    ///    equals the geometric count and the geometric sentence is still
    ///    exactly true. **Geometric**; see that variant for the ruling on the
    ///    never-previewed case.
    /// 4. otherwise — a correction was made and unexamined sheets remain.
    ///    `count` is a ceiling. **AtMost**.
    ///
    /// Every arm is exactly true of the state that reaches it. That is the
    /// property to preserve if this is ever edited: not brevity, not
    /// reassurance — truth per state.
    const fn from_counts(geometric: usize, known_blank: usize, unresolved: usize) -> Self {
        // `saturating_sub` states the invariant rather than relying on it:
        // `known_blank` is counted inside the `clipped` branch of the same
        // loop, so it can never exceed `geometric` — and an underflow here
        // would produce `usize::MAX` sheets on the one button in this
        // application with no undo behind it.
        let count = geometric.saturating_sub(known_blank);
        if count == 0 {
            Self::None
        } else if unresolved == 0 {
            Self::Measured(count)
        } else if known_blank == 0 {
            Self::Geometric(count)
        } else {
            Self::AtMost(count)
        }
    }

    /// The commit button's label, or `None` for the plain **Print**.
    pub fn commit_label(self) -> Option<String> {
        match self {
            Self::None => Option::None,
            Self::Geometric(n) => Some(t::commit_with_clipping(n)),
            Self::Measured(n) => Some(t::commit_losing_content(n)),
            Self::AtMost(n) => Some(t::commit_may_lose_content(n)),
        }
    }

    /// The job-wide sentence under the preview, or `None` when there is
    /// nothing to say.
    pub fn summary(self, total: usize) -> Option<String> {
        match self {
            Self::None => Option::None,
            Self::Geometric(n) | Self::Measured(n) => Some(t::clip_summary(n, total)),
            Self::AtMost(n) => Some(t::clip_summary_at_most(n, total)),
        }
    }

    /// One word for the diagnostic trace.
    pub const fn trace_word(self) -> &'static str {
        match self {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Self::None => "none",
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Self::Geometric(_) => "geometric",
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Self::Measured(_) => "measured",
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Self::AtMost(_) => "at-most",
        }
    }

    /// The number the claim carries, for the trace.
    pub const fn count(self) -> usize {
        match self {
            Self::None => 0,
            Self::Geometric(n) | Self::Measured(n) | Self::AtMost(n) => n,
        }
    }
}

#[cfg(test)]
#[path = "printverdicts_tests.rs"]
mod tests;
