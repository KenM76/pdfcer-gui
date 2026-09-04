//! # `dialogs::print::verdicts` — what the operator has actually LOOKED at
//!
//! ## The contradiction this module exists to remove
//!
//! Operator request O113 made the preview's clip hatch **ink-aware**: on a 1:1
//! CAD sheet whose overhang is empty paper, nothing is hatched and the caption
//! says *"This sheet hangs over the printable area, but nothing is printed
//! there — the overhang is blank."*
//!
//! [`Job::clipped`] stayed **geometric** — it counts sheets whose page box
//! exceeds the printable rectangle, a plan-time test taken with no raster in
//! hand — so the commit button went on reading *"Print — 1 sheet will be
//! clipped"* over a picture showing nothing lost.
//!
//! Both sentences were true. They read as contradicting each other, and the
//! button is the louder surface.
//!
//! ## ★★★ The wording was NOT weakened, because that is how the next defect
//! ## gets built
//!
//! The obvious fix — soften the button to *"1 sheet may be clipped"* — takes a
//! statement that is exactly true and makes it vaguer so that it stops
//! disagreeing with a better one. The disagreement is then hidden rather than
//! resolved, and the surface that was *right* is the one that got worse.
//!
//! The operator's ruling, 2026-09-04:
//!
//! > *"Remember the blank/not-blank verdict for each sheet as the operator
//! > steps through the preview, and label the button with the geometric count
//! > minus the sheets known blank, with the ones not yet looked at still
//! > counted. Every claim stays true and nothing has to render the whole
//! > job."*
//!
//! **Make the COUNT better, not the sentence vaguer.** That is what this
//! module does, and the wording then follows from what the count can support
//! rather than from what reads comfortably.
//!
//! ## The arithmetic, and what it is a bound ON
//!
//! Every clipped sheet is in exactly one of three states:
//!
//! | state | what is known | in the count? |
//! |---|---|---|
//! | **known blank** | the preview rendered it and the ink test found nothing in the overhang | **no** — subtracted |
//! | **known inked** | the preview rendered it and there is ink out in the band | yes |
//! | **unexamined** | nothing has looked at it, or it would not render | yes |
//!
//! ```text
//! displayed = geometric − known_blank = known_inked + unexamined
//! ```
//!
//! ### ★★ The displayed number is a CEILING, not a floor
//!
//! The request that authorised this work called it *"a floor rather than a
//! total."* That is the wrong way round, and the direction decides the
//! wording, so it is worth doing the inequality rather than adopting the
//! phrase.
//!
//! Let `T` be the number of sheets that really will lose ink. Every known-blank
//! sheet is definitely not in `T`; every known-inked sheet definitely is; each
//! unexamined sheet may or may not be. So
//!
//! ```text
//! known_inked  ≤  T  ≤  known_inked + unexamined  =  displayed
//! ```
//!
//! The number on the button is the **largest** `T` could be. `known_inked` is
//! the floor and it is not the number displayed — displaying it would be the
//! genuinely dangerous mistake, because it would under-report loss on exactly
//! the sheets nobody has looked at.
//!
//! A ceiling can only be said with a hedge, and *that hedge is a correction
//! rather than a weakening*: it is added at the same moment the number stops
//! being a count of anything measured. See [`ClipClaim`], where each of the
//! four states carries the strongest sentence that state can support.
//!
//! ## ★★★ The key: a cached verdict is a CLAIM ABOUT PIXELS, and it must not
//! ## outlive them
//!
//! `preview::PreviewKey`'s doc comment states the discipline this module
//! inherits: a cache key is a claim that *"if these are equal, re-rendering
//! would produce the same image."* A verdict cached under a weaker key is a
//! verdict about a page that has changed — and it would be **confidently**
//! wrong, which is worse than no cache at all, because the whole point of the
//! entry is to remove a warning.
//!
//! So the key here is not merely as strong as the texture's; it is strictly
//! stronger, and it is built so that it *cannot* be weaker:
//!
//! - [`Context`] holds the job-wide half — the annotation scope, the whole
//!   `Settings`, and the printable rectangle. Its [`Context::preview_key`] is
//!   the **only** construction site of a `PreviewKey` in the crate, so the
//!   texture the preview shows is keyed on a value *derived from* this
//!   context. The texture cache therefore cannot be keyed on anything this one
//!   is not.
//! - Each entry additionally pins the sheet's own geometry — the `Placement`
//!   and the page's size in pdf dimensions — because the verdict depends on
//!   *where the band falls*, and the same pixels under a different placement
//!   give a different answer. `PreviewKey` deliberately omits the placement
//!   (it does not change one pixel of the raster); a verdict that omitted it
//!   would survive a switch from Fit to 100 % and answer for the wrong band.
//!
//! ### Why an over-strong key is safe and an under-strong one is not
//!
//! The failure modes are asymmetric, and the key is chosen toward the safe
//! one deliberately:
//!
//! - **Too strong** ⇒ a live verdict is dropped ⇒ the sheet counts as
//!   unexamined ⇒ the number goes *up* ⇒ a warning the operator did not need
//!   is shown once more, and one more look at the preview removes it.
//! - **Too weak** ⇒ a stale verdict is trusted ⇒ the number goes *down* ⇒ a
//!   warning is **removed** on evidence about a different page. That is a
//!   false claim on the one surface in this application with no undo behind
//!   it.
//!
//! Everything ambiguous therefore resolves to "unexamined". That is also why
//! [`Overhang::Unknown`] — *"the page would not render, so we could not
//! look"* — is counted as unexamined rather than as knowledge: not being able
//! to look is not a finding.
//!
//! ## What is NOT keyed, and the hole that leaves
//!
//! The document's own edit state. `PreviewKey` does not carry an edit
//! generation, so an unsaved edit does not invalidate the preview texture
//! either — the preview renders `session.view()` but is keyed on the settings.
//! That hole is inherited rather than introduced: this cache is exactly as
//! stale as the pixels it describes and never staler, which is the strongest
//! property available without changing the texture key. Closing it belongs in
//! `PreviewKey`, in the commit that gives this crate an edit generation to key
//! on, and both halves must move together.
//!
//! ## Cost
//!
//! One `BTreeMap` entry per previewed page — a `Placement`, two `f64` and a
//! one-byte enum, bounded by the document's page count — and one `Settings`
//! clone per frame for the context. `claim` is a linear pass over the plan
//! list of `bool` tests and map lookups, which is the same order as
//! [`Job::clipped`] itself. **Nothing here renders anything**, which is the
//! constraint the whole design is built to satisfy: an ink-aware count that
//! rasterised the job would make opening the dialog cost more than the print.

use std::collections::BTreeMap;

use super::preview::{Overhang, PreviewKey};
use super::spooler::{Job, PagePlan, Placement};
use crate::text::print as t;

/// The job-wide half of a verdict's key, and the one place a
/// [`PreviewKey`] is built.
///
/// # Why these three fields and no others
///
/// A sheet's overhang verdict is a function of exactly two things: **the
/// pixels** (what the page renders to) and **the band** (which part of the
/// page falls outside the printable rectangle).
///
/// - `scope` and `settings` decide the pixels. They are `PreviewKey`'s own two
///   non-page fields, and they are held here whole for the reason that type
///   gives at length: naming the five rendering settings individually would be
///   a second statement of which settings affect a render, and the failure mode
///   of the two disagreeing is a preview that silently never updates.
/// - `printable_pt` decides the band, together with the per-sheet placement.
///   The *sheet* rectangle and the unprintable offset do **not** appear,
///   because they only move the whole diagram: the band, expressed as a
///   fraction of the page — which is the coordinate system
///   [`super::ink::InkMask`] speaks — depends on the printable extent, the
///   placement and the page size, and on nothing else. The preview's zoom, pan
///   and fit drop out for the same reason, which is why panning does not throw
///   a verdict away.
///
/// ★ It is `PartialEq` rather than `Eq` because `Settings` carries a `String`
/// and `printable_pt` carries `f64`s. Comparing device dimensions with `==` is
/// exact here on purpose: these numbers are copied out of the driver's own
/// report, not computed, so two reads of an unchanged device produce the same
/// bits. A device that reported a different rectangle *should* void every
/// verdict.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Context {
    /// Which annotation classes are painted — `PreviewKey`'s field.
    scope: pdfcer_render::AnnotationScope,
    /// The operator's configuration, whole — `PreviewKey`'s field.
    settings: pdfcer_core::settings::Settings,
    /// The printable area in ce dimensions, from the planned device geometry.
    printable_pt: (f64, f64),
}

impl Context {
    /// Snapshot the frame's rendering inputs and printable rectangle.
    ///
    /// Built **once per frame** in [`super::PrintDialog::show`] and passed
    /// down, rather than rebuilt at each of the three sites that need it: the
    /// `Settings` clone is the only allocation in this module's whole hot path
    /// and there is no reason to pay for it three times.
    pub(super) fn new(
        scope: pdfcer_render::AnnotationScope,
        settings: &pdfcer_core::settings::Settings,
        printable_pt: (f64, f64),
    ) -> Self {
        Self {
            scope,
            settings: settings.clone(),
            printable_pt,
        }
    }

    /// The preview texture's cache key for `page`.
    ///
    /// # ★★★ This is the enforcement, not a convenience
    ///
    /// `super::preview::texture_for` obtains its key from **here**. That is
    /// what makes "the verdict is keyed on at least what the pixels are keyed
    /// on" a structural fact rather than a promise: the texture's key is
    /// derived from this context, so a context that still matches implies a
    /// `PreviewKey` that still matches, for every page.
    ///
    /// Building the key in two places instead — one for the texture and one
    /// for the verdict — is the exact shape of drift this project has been
    /// caught by repeatedly: two readings of one rule, kept level by memory.
    pub(super) fn preview_key(&self, page: usize) -> PreviewKey {
        PreviewKey::new(page, self.scope, &self.settings)
    }
}

/// The per-sheet half of a verdict's key: **where the band falls**.
///
/// Not the plan's position in the send order. A job may print the same
/// document page twice (uncollated copies) or in a reversed or filtered
/// sequence, and the verdict is a fact about *this page under this placement*,
/// not about a position in a list. Two plans naming the same page carry
/// identical placements by construction — `super::spooler::plan` computes one
/// placement per page — so the second one inherits the first's verdict
/// legitimately: it is derived, not invented.
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
    ///
    /// ★ One function, called by both the write path and the read path, so
    /// the two cannot build the identity differently. A remembered verdict
    /// that could never be found again would look exactly like a preview that
    /// was never opened — a silent, permanent over-count with nothing to say
    /// why.
    ///
    /// ★★ `page_sizes` is indexed by `plan.index`, the **document** page, and
    /// never by a position in the plan list. That is the same defect
    /// `super::preview::paint` carries a comment about, and it would be
    /// re-introduced here by using the loop counter.
    fn of(plan: &PagePlan, page_sizes: &[(f64, f64)]) -> Option<Self> {
        Some(Self {
            placement: plan.placement,
            page_pt: *page_sizes.get(plan.index)?,
        })
    }
}

/// What the preview has found, for the sheets it has been shown.
///
/// Lives on [`super::PrintDialog`], so it is forgotten when the dialog closes.
/// That is the right lifetime: the verdicts describe one job's placements
/// against one device, and neither survives the dialog.
#[derive(Debug, Default)]
pub(super) struct Verdicts {
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
    ///
    /// Called from `super::preview::paint`, at the single point where the ink
    /// question was actually asked — the same computation the hatch is drawn
    /// from. **Nothing recomputes the verdict a second way**, which is the
    /// property `super::preview::lost_regions` was made pure to guarantee and
    /// which this cache would otherwise quietly undo.
    ///
    /// A change of context throws the whole map away first. Merging instead —
    /// keeping entries whose sheet identity happens to still match — would be
    /// keeping verdicts recorded from a *different raster*, which is precisely
    /// the staleness the context exists to catch.
    pub(super) fn remember(
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
    ///
    /// Three ways to get `None`, and they are deliberately indistinguishable
    /// to the caller because they mean the same thing — *no claim can be made
    /// about this sheet*:
    ///
    /// 1. the context has moved on (different settings, scope or device);
    /// 2. the plan names a page that is no longer there;
    /// 3. the sheet was never previewed, or was previewed under a different
    ///    placement or page size.
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
    ///
    /// One pass over the plan list. Every clipped sheet is sorted into one of
    /// three buckets and [`ClipClaim::from_counts`] turns the three totals
    /// into a claim — split that way so the arithmetic is testable without a
    /// `Job`, a device or a document.
    ///
    /// ★ Only [`Overhang::BlankBand`] and [`Overhang::Losing`] are treated as
    /// knowledge. `Unknown` means the page would not render and the whole band
    /// was hatched as the honest fallback — *"we could not look"*, which must
    /// not be allowed to look like *"we looked and it was fine"*. `Fits` on a
    /// sheet whose placement reports a clip is the degenerate case
    /// `super::preview::lost_regions` describes, where the arithmetic produced
    /// no positive band; it is not evidence about ink either. Both count as
    /// unexamined, which keeps the sheet in the number.
    pub(super) fn claim(
        &self,
        context: &Context,
        job: &Job,
        page_sizes: &[(f64, f64)],
    ) -> ClipClaim {
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
///
/// Four states, four sentences, each the strongest thing its state can support.
/// The type exists so that the number and the wording are decided **together**,
/// once: a count that is a measurement and a count that is a ceiling cannot
/// share a sentence, and choosing the sentence at the two call sites
/// separately is how the button and the caption come to disagree — which is
/// the entire defect this module was written for, one level up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ClipClaim {
    /// Nothing to disclose. Either no sheet's page box exceeds the printable
    /// area, or every one that does has been examined and found blank — the
    /// operator's 1:1 CAD drawing, after one look at the preview.
    None,
    /// **Nothing has been subtracted**, so the plain geometric fact still
    /// stands exactly: this many sheets have a page box exceeding the printable
    /// rectangle. Said in the words it has always been said in.
    ///
    /// ★★★ THE RULING ON PRINTING WITHOUT EVER OPENING THE PREVIEW, and it is
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
    ///
    /// The count is in the button's own label rather than beside it, which is
    /// this dialog's standing choice: *"the difference between a warning the
    /// operator has to have read and one they can have looked past — it is on
    /// the control their hand is already on."*
    pub(super) fn commit_label(self) -> Option<String> {
        match self {
            Self::None => Option::None,
            Self::Geometric(n) => Some(t::commit_with_clipping(n)),
            Self::Measured(n) => Some(t::commit_losing_content(n)),
            Self::AtMost(n) => Some(t::commit_may_lose_content(n)),
        }
    }

    /// The job-wide sentence under the preview, or `None` when there is
    /// nothing to say.
    ///
    /// ★ `Geometric` and `Measured` share one sentence, and that is not
    /// laziness. [`t::clip_summary`] has always read *"N of these T sheets
    /// will lose content outside the printable area"* — a content claim. Under
    /// `Measured` that claim is now **verified**; under `Geometric` it is the
    /// unchanged shipped wording, for the reason [`Self::Geometric`] gives.
    /// Only the ceiling needs a sentence of its own, because only the ceiling
    /// is a number that was never measured.
    pub(super) fn summary(self, total: usize) -> Option<String> {
        match self {
            Self::None => Option::None,
            Self::Geometric(n) | Self::Measured(n) => Some(t::clip_summary(n, total)),
            Self::AtMost(n) => Some(t::clip_summary_at_most(n, total)),
        }
    }

    /// One word for the diagnostic trace.
    ///
    /// ★ This is the ONLY headless evidence of which claim a frame made, and
    /// it is needed for the same reason `overhang=` is: a button reading
    /// *"Print — 2 sheets will lose content"* and one reading *"Print — up to
    /// 2 sheets may lose content"* differ by a state nothing else exposes, and
    /// a capture cannot tell a correct subtraction from a cache that silently
    /// never matched. The word says which.
    pub(super) const fn trace_word(self) -> &'static str {
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
    pub(super) const fn count(self) -> usize {
        match self {
            Self::None => 0,
            Self::Geometric(n) | Self::Measured(n) | Self::AtMost(n) => n,
        }
    }
}

#[cfg(test)]
#[path = "verdicts_tests.rs"]
mod tests;
