//! # `canvas::textedit::report` — what an edit report is worth telling anyone
//!
//! ## The seam
//!
//! Split out of [`super`] on 2026-08-20 under R2. The subject is narrow and
//! real: `pdfcer_core::text_edit::EditReport` comes back from every commit
//! carrying eleven fields, and **almost none of them belong on a status row**.
//! Deciding which go to the operator, which go to the diagnostic channel and
//! which go nowhere is a judgement that recurs every time the engine adds a
//! field — most recently `form_object`, `form_invocations` and `form_pages` in
//! `Pass 119.0`, and `followers_repositioned` becoming load-bearing in
//! `Pass 121.1` a few hours later.
//!
//! ## The rule this module applies, stated once
//!
//! | goes to | when |
//! |---|---|
//! | **the status row**, verbatim from `report.disclosures` | the engine wrote a sentence for an operator to read. pdfcer owns that wording; re-phrasing it here would be a second account of one fact, free to drift |
//! | **the diagnostic channel** | the fact is a number about a content stream. An operator cannot act on *"1,676 followers were repositioned"*; a driven check can, and a regression then names itself |
//! | **nowhere** | it restates something already visible on the page |
//!
//! ★ The middle row is the one that earns this module. R8b rule 4 says a
//! disclosure must be in terms of what the operator can see — so a number about
//! operator counts is not a disclosure, it is *evidence*, and evidence belongs
//! where a check can read it.

/// **The forms THIS PAGE invokes directly** — the set that decides whether the
/// shared-content disclosure may name a remedy.
///
/// # ★★★ Why this type exists, and why the remedy is conditional
///
/// This module's header said, correctly and for eight days, that the operator's
/// half of the shared-content report *"is already handled and is deliberately
/// not re-worded"*: `pdfcer-core` writes the `"SHARED CONTENT: …"` sentence into
/// `EditReport::disclosures` and the `edit_text` arm carries it to the status
/// row verbatim.
///
/// That is still true, and **nothing below re-words it.** What changed on
/// 2026-08-28 is that a *remedy* came into existence:
/// `EditSession::unshare_form`, surfaced as `format.unshare_form`. The engine
/// cannot name it — `pdfcer-core` has never heard of this shell's commands — so
/// the sentence that names it has to be the shell's, appended, exactly as
/// `crate::text::textedit::pinned_tail_disclosure` already is in the same arm.
///
/// # ★★★ The nesting case, which is the whole reason this is a TYPE and not a
/// # boolean
///
/// The remedy is **not always available**, and offering it where it does not
/// work would be worse than staying silent — it would send an operator through
/// undo, a command, and a re-edit to arrive at the identical fan-out.
///
/// `format.unshare_form` always hands the engine the **outermost** enclosing
/// form, because that is the only operand `unshare_form` accepts (a nested one
/// is refused by name). Now consider a page that invokes form `A`, where `A`
/// invokes form `B`, and the edited text lives in `B`:
///
/// | | before | after unsharing `A` |
/// |---|---|---|
/// | this page | `A` → `B` | `A'` → **`B`** |
/// | the other 35 | `A` → `B` | `A` → **`B`** |
///
/// `A` is privatised and `B` — the stream holding the edited glyphs — is
/// **still shared by both copies**. The command succeeds, discloses that it
/// succeeded, and the operator's next edit fans out exactly as before. There is
/// no refusal to catch, because nothing was refused.
///
/// ⇒ So the remedy is offered **only when the edit went into a form the page
/// invokes directly**, which is the case where unsharing that form is the
/// stream holding the text. That is what this set answers, and it is derived
/// from `FormLeaf::containment[0]` — *"the chain of enclosing form XObjects,
/// outermost first"* — which is the same element
/// `panels::objects::provider::ObjectModelProvider::containing_form_object`
/// hands the verb. One derivation: the sentence cannot promise a form the
/// command would not act on.
///
/// # Why it is gathered BEFORE the edit rather than after
///
/// `super::super::super::app::actions::apply::vector_edit` takes `&mut OpenDoc`
/// and the decomposition is reached through a `Ref` into a `RefCell` cache, so
/// the borrow must be released before the edit begins. Gathering a `BTreeSet`
/// of `u32` first costs one walk of a list the frame already built, and the
/// page's own `/XObject` names cannot change during the edit that is about to
/// happen — a text edit rewrites a content stream, not a resource dictionary.
pub struct PageLevelForms(std::collections::BTreeSet<u32>);

impl PageLevelForms {
    /// Gather the set from the current page's decomposition.
    ///
    /// **Empty when the page has not decomposed**, and that is the safe
    /// answer rather than a degraded one: an empty set offers no remedy, and
    /// silence is the correct output when this shell cannot confirm that the
    /// remedy would work. A fallback that guessed *"probably page-level"* would
    /// be advice given without evidence, on the one surface where wrong advice
    /// costs an operator two commands and a re-edit.
    #[must_use]
    pub fn of(doc: &crate::app::state::OpenDoc) -> Self {
        Self(doc.page_objects().map_or_else(Default::default, |objects| {
            objects
                .page_objects()
                .leaves
                .iter()
                .filter_map(|leaf| leaf.containment.first())
                .map(|id| id.num)
                .collect()
        }))
    }

    /// The remedy sentence, if this edit's fan-out is one the shell can offer
    /// to undo.
    ///
    /// Two conditions, both necessary:
    ///
    /// 1. **`form_invocations > 1`** — the engine's own `InvocationSet::is_shared`
    ///    predicate, spelled here as the comparison it is (`sites.len() > 1`),
    ///    so the remedy appears on exactly the edits the `SHARED CONTENT`
    ///    sentence appears on and never beside a report that did not warn. A
    ///    remedy for a problem the operator was not told they had is noise.
    /// 2. **the edited form is page-level** — see the type's docs for the
    ///    nesting case this excludes and why excluding it is the point.
    #[must_use]
    pub fn remedy_for(&self, report: &pdfcer_core::text_edit::EditReport) -> Option<String> {
        (report.form_invocations > 1
            && report
                .form_object
                .is_some_and(|form| self.0.contains(&form)))
        .then(crate::text::unshare::shared_content_remedy)
    }
}

/// **Which content stream the commit rewrote, and how many places paint it.**
///
/// # ★★★ Shared content, and why this is worth a named function
///
/// `Pass 119.0` made form-XObject text editable, and a form XObject may
/// legally be painted **from several pages and several times on one page** —
/// ISO 32000-1 §8.10.1 states that as the *purpose* of the feature, and names
/// a CAD system's standard component as the illustration, which is this
/// operator's title block exactly.
///
/// **No clause in either edition binds a form to a page.** That is a confirmed
/// permanent negative result in pdfcer's spec corpus (`FX-N1`), argued three
/// independent ways. So editing text inside a shared form changes **every place
/// it appears**, and there is nothing pdfcer can do about that: there is exactly
/// one stream holding those glyphs. The engine's words when it shipped this:
///
/// > *"A shell that ignores `form_invocations` is a shell that changes six
/// > drawing sheets while showing one."*
///
/// # The operator's half is already handled, and is deliberately not re-worded
///
/// `pdfcer-core` puts a `"SHARED CONTENT: …"` sentence into
/// `EditReport::disclosures`, worded for direct display, and the `edit_text`
/// apply arm has always carried that list to the status row. Re-wording it here
/// would be a second account of one fact, free to drift from the engine's.
///
/// ★ It is **absent** on the ordinary single-paint case, by the engine's
/// design and this project's own rule: a warning that fires every time is one
/// nobody reads, and this one is meant to be startling. That is also why there
/// is no badge, tint or flag drawn into the page — R8b rule 4 as narrowed by
/// pdfcer's decision 059. The disclosure is off-canvas or it is nowhere.
///
/// # What this adds is the machine-readable half
///
/// A driven check cannot assert on prose, and these three numbers are exactly
/// what a wrong build gets wrong:
///
/// | field | what a wrong build reports |
/// |---|---|
/// | `form=` | `none` when the edit was meant for a form — the target collapsed to the page stream |
/// | `invocations=` | `1` for a shared form, i.e. the fan-out was not asked for |
/// | `pages=` | a count that disagrees with `invocations` on a form painted twice on one page |
///
/// The first is the regression that matters most on this operator's documents,
/// because `EditTarget::Auto` offers a pinned span to the page's own stream
/// first — and on the benchmark sheet that stream holds 3,007 single-character
/// show operators, so a stray match there is a dense field of near-misses
/// rather than a theoretical collision. See [`plan`], which names the target
/// from the same provenance record it takes the pin from.
///
/// # What is NOT built, said so it is a decision
///
/// There is no **pre-commit** warning: an operator whose caret lands in a
/// shared form is not told before they type. The engine publishes
/// `text_edit::forms::invocation_map`, which answers the fan-out for every form
/// in one document walk, so it is buildable — but one walk per click on text is
/// not affordable uncached, and a cache keyed on the document rather than the
/// page is a piece of work rather than a line. Recorded in
/// `OPERATOR_REQUESTS.md` rather than left implied.
pub fn trace_target(page: usize, run: usize, report: &pdfcer_core::text_edit::EditReport) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "edit-text-target page={page} run={run} form={} invocations={} pages={} \
             followers={} disposition={:?}",
            report
                .form_object
                .map_or_else(|| "none".to_owned(), |o| o.to_string()),
            report.form_invocations,
            report.form_pages.len(),
            // ★★ THE REFLOW'S REACH, on the channel because the engine asked
            // for it by name and because of what it caught.
            //
            // `Pass 121.1`, 2026-08-20. A four-character edit on the operator's
            // own drawing reported `followers_repositioned=1676` and moved
            // 34,059 pixels across the whole sheet; after the fix the same edit
            // moved 42 pixels inside one label. The cause was that reflow shifts
            // "the rest of the line" until a `Td`/`TD`/`T*` boundary — and **a
            // CAD stream positions everything with `Tm` and never emits `Td`**,
            // so there was no boundary and the scan ran to the end of the
            // stream.
            //
            // The engine's request, verbatim: *"if you show one number from an
            // edit report beyond the disclosures, make it that one."* On
            // absolutely-placed content it should be `0`; a large number means
            // the edited "line" ran further than the line.
            //
            // ★ It is on the trace and NOT on the status row, and that is a
            // decision. The operator cannot act on "1,676 followers were
            // repositioned" — it is a number about a content stream, and rule 4
            // says a disclosure must be in terms of what he can see. What he
            // CAN see is the page, and this shell's answer to "did the edit move
            // more than it should" is the render diff a driven check measures.
            // The number is here so that a check has a cheap oracle and a
            // regression names itself, rather than being found by him again.
            report.followers_repositioned,
            report.disposition,
        )
    });
}
