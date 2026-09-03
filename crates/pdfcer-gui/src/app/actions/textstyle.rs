//! # `app::actions::textstyle` — changing how EXISTING text looks
//!
//! ## The operator's ask, and the table that answered it wrong
//!
//! > *"We should also have all the font tools available that Word does."*
//! > — 2026-08-25, `OPERATOR_REQUESTS.md` O37
//!
//! O37's inventory read the engine's text verbs — `add_text`, `edit_text`,
//! `delete_text_run` — and concluded, in a table with fourteen rows and a
//! column of crosses, that **pdfcer could choose how text looked when it was
//! created and could not change how existing text looked at all.**
//!
//! Every cross in that column was wrong. `EditSession::format_text` shipped as
//! `Pass 14.2`, was extended through `Pass 19.x`, and became retargetable into
//! form XObjects as `Pass 119.2` — five days *before* the request that said it
//! did not exist. It had reached this project only as a paragraph inside a note
//! about something else, which is the engine's own recorded defect (`R220`):
//! *"a verb whose only description is something somebody said once, in
//! passing, while writing about something else."*
//!
//! ★ Worth keeping, because half the lesson is ours: **an absence claim about a
//! crate you do not build is a claim about every route, and one verb is not
//! every route.** This module exists because somebody eventually asked instead
//! of inferring.
//!
//! ## What can actually be changed, measured rather than described
//!
//! | control | verb | limit |
//! |---|---|---|
//! | **size** | `set_size` | none — the `Tf` operand changes and the line is relaid out |
//! | **colour** | `set_fill` | none. pdfcer stores the SPACE the operator chose (`rg`/`g`/`k`) instead of force-converting to DeviceRGB the way Acrobat does |
//! | **face** | `set_font` | the target must **already be a font resource on the page**; refused by name otherwise (`FF-C`) |
//! | **bold / italic** | `set_synthetic`, *or* `set_font` — see below | one named refusal, on italic only |
//!
//! ## ★★★ Why Bold is never greyed, and why one press can take either verb
//!
//! `set_font` selects a real face and refuses when the page carries none.
//! `gate_synthesis` is its **exact complement**: it refuses synthesis when a
//! real face *is* available, and its own first branch reads *"No font resources
//! to search: nothing better exists, so the fallback is genuinely the only
//! option. Proceed."*
//!
//! ⇒ ~~Between the two verbs **every page is covered**, and there is no page on
//! which bold is unreachable.~~ **Retracted — see below.** The engine's
//! instruction stands and is unchanged: *"Do not grey out a bold button. Offer
//! it, and surface the disclosure when synthesis fires."*
//!
//! So [`apply`] asks for synthesis first and, when the engine refuses **because
//! a real face is available**, retries with that face — which the refusal names
//! (`RealFaceAvailable { real_font, .. }`). The operator presses one button and
//! gets the best weight the page can give: a genuine typeface where one exists,
//! a disclosed synthetic one where it does not.
//!
//! ★ That retry is the one place this module reacts to an error variant rather
//! than reporting it, and it is not cleverness papering over a refusal: the
//! refusal's own prose *names the remedy and asks for it to be applied*. Not
//! taking it would mean showing the operator a sentence telling them to do
//! something the program could have done.
//!
//! A synthetic weight is the regular face thickened by stroking; a synthetic
//! slant is the upright face sheared. `R90` means neither is ever a preference
//! — only an explicit, per-use request — and the report says which fired, in
//! words passed straight through rather than re-written.
//!
//!
//! ## ★★★ RETRACTED 2026-08-27 — "every page is covered" was never true
//!
//! The paragraph above is kept because the *behaviour* it describes is
//! correct and shipped, and struck at its one load-bearing claim, which the
//! engine has since withdrawn in writing:
//!
//! > ~~"Between the two verbs every page is covered. … There is no page on
//! > which bold is unreachable."~~
//!
//! On pdfcer's own `textedit/format_family.pdf`, `gate_synthesis` prefers a
//! real face by **family** — it names `Times-Bold` for a run set in `Times`
//! — and `Times-Bold` on that page does not map `o`, so `set_font` refuses it
//! by name. `Calibri-Bold`, on the same page, covers the run and would have
//! worked. Neither verb reaches it: synthesis is gated off because a "real
//! face is available", and the real face it names cannot show the text. **Bold
//! is unreachable there through either route.**
//!
//! Filed as `request_gate_synthesis_names_a_face_that_cannot_cover_the_run.md`,
//! confirmed and reproduced by the engine the same day, and queued ahead of
//! their print-conformance work by their operator's instruction. The fix is
//! the one this project's diagnosis asked for: `gate_synthesis` will treat a
//! real face as available only if `set_font` would accept it *for this run*.
//!
//! ★ **Nothing here works around it**, and the twenty-line shell-side search
//! for a different bold resource was considered and refused. It would work,
//! and it would be this project second-guessing pdfcer's font selection —
//! decision 058's exact case, and the workaround every other consumer would
//! then have to write for themselves.
//!
//! ★★ The two buttons are **still never greyed**, and the retraction does not
//! change that. Greying them would need this shell to predict a refusal that
//! depends on a per-run glyph-coverage test it cannot run without doing the
//! engine's work; the honest behaviour on a page like that one is to try, and
//! to show the engine's own named refusal in the status bar. That is what
//! happens.
//!
//! ## ★★★ Why the runs are edited in DESCENDING order
//!
//! The load-bearing decision in the file, and invisible until it is wrong.
//!
//! `format_text` rewrites one **show operator**. A sweep can cover several, so
//! a restyle is several calls. Each call rewrites the content stream, so every
//! pin taken before it is stale afterwards — which is why
//! [`crate::canvas::textedit::pin::resolve`] is re-run between steps instead of
//! the pins being batched up front.
//!
//! Re-resolving fixes the *spans*. It does not fix the **indices**: synthetic
//! italic brackets its run with two absolute `Tm` operators, and a `Tm` can
//! split a run, so an edit at index *k* may renumber everything after it.
//!
//! Descending order makes that harmless by construction. Editing run *k* can
//! only insert operators at or after *k*'s position in the buffer, so runs
//! `0..k` keep both their bytes and their ordinals. Working downwards, every run
//! still to be done is always *before* the one just done, and its index is still
//! the index it was measured at.
//!
//! Ascending order would work for four of the five controls and fail for italic,
//! on multi-run selections only, by restyling the wrong text — the shape of
//! defect that ships because the case that breaks it is the one nobody tries.
//!
//! ## What is NOT here, and is filed rather than hidden
//!
//! **One gesture is N undo entries** when the selection covers N runs.
//! `EditSession` has no grouping verb; the engine solves multi-verb undo by
//! adding a *combined* verb per case, which is how `Pass 81.1` gave markup
//! authoring an opacity in one entry rather than two. A restyle across a
//! paragraph therefore takes several `Ctrl+Z` presses to take back.
//!
//! Disclosed by the count rather than left to be discovered, and filed on the
//! request channel. **Not** worked around here: a shell-side coalesce would
//! work and would leave every other consumer with the same defect, which is
//! decision 058's whole argument and is quoted in the engine's own docs about
//! the last time it happened.

use pdfcer_core::settings::StylePolicy;
use pdfcer_core::text_edit::{
    FormatError, FormatOptions, FormatRequest, NewFill, StyleOutcome, StyleSynthesis,
};

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::text::status as t;

/// One property of a text run, and the value the operator chose for it.
///
/// One variant per control, because **one control press is one undo entry**. A
/// struct carrying five `Option`s would let the panel batch a size and a colour
/// into a single request — which the engine supports — and would make `Ctrl+Z`
/// after two separate presses take back a state the operator never saw. The
/// panel commits on `drag_stopped` / `lost_focus` for the same reason.
#[derive(Debug, Clone, PartialEq)]
pub enum StyleChange {
    /// A new size in points, changing the `Tf` operand.
    Size(f64),
    /// A new fill colour, stored in the space the operator chose.
    Fill(NewFill),
    /// A new face, named by `/Resources /Font` key or by `/BaseFont`.
    ///
    /// A `String` rather than a `FontSelector` because [`super::action::Action`]
    /// derives `PartialEq` and `FontSelector` is `#[non_exhaustive]`. The
    /// conversion is one line at the point of use.
    Face(String),
    /// Weight and slant, as two independent flags.
    ///
    /// ★ Deliberately **not** named `Synthetic`, because whether it ends up
    /// synthetic is the engine's decision and not the operator's: see the
    /// module header on the two-verb retry. The operator asked for bold; how
    /// bold is achieved on this page is a fact they are told afterwards.
    Weight {
        /// Bold wanted.
        bold: bool,
        /// Italic wanted.
        italic: bool,
    },
}

impl StyleChange {
    /// Stamp this change onto a request that is already pinned.
    ///
    /// Its own function so the mapping from *a control the operator pressed* to
    /// *a field on `FormatRequest`* is one readable table, and so a sixth
    /// control cannot be added without appearing in it.
    fn stamp(&self, req: FormatRequest) -> FormatRequest {
        match self {
            Self::Size(points) => req.size(*points),
            Self::Fill(fill) => req.fill(fill.clone()),
            Self::Face(selector) => req.font(pdfcer_core::text_edit::FontSelector::new(selector)),
            Self::Weight { bold, italic } => req.synthetic(StyleSynthesis::new(*bold, *italic)),
        }
    }

    /// The trace word for this change, for `PDFCER_DIAG`.
    const fn label(&self) -> &'static str {
        match self {
            Self::Size(_) => "size",
            Self::Fill(_) => "fill",
            Self::Face(_) => "face",
            Self::Weight { .. } => "weight",
        }
    }
}

/// A pinned request for one run, ready to hand to the engine.
///
/// Built fresh per run per step; see the module header on why a batch of these
/// taken up front would be wrong.
/// The request for one show operator, addressed by pin alone.
///
/// # ★★★ It carried a `find` until 2026-08-27, and the reason it no longer does
///
/// The comment this replaces is worth keeping, because it was **correct when it
/// was written** and is the whole argument for the affordance that replaced it:
///
/// > `find` is the RUN'S OWN TEXT, and it is required. The obvious shape — pin
/// > the operator and leave `find` empty, because the pin already says which
/// > operator — does not work, and finding that out is what the first driven
/// > test of this module was for. `match_run` refuses an empty `find` by name.
///
/// It was filed as one request, deliberately on its own — *"`find: ""` on a
/// pinned request should mean the whole operator"* — and `Pass 145.0` shipped
/// it the same day. `FormatRequest::whole_operator(page, span)` is exactly
/// `new(page, "").pinned(span)`, and the named constructor is used here because
/// it says what it means.
///
/// ★★ What this **deletes** is the more important half: the run's text had to
/// be sliced into per-operator pieces to build those `find`s, and that slicing
/// was a second locator living beside the engine's. It is gone —
/// `pin::Operator` no longer carries a `find` or the byte cursor that extended
/// it — which removes the whole class of defect where two locators agree on
/// every fixture and disagree on a ligature.
///
/// ★ An empty `find` with **no** pin is still refused by name. A caller that
/// forgot to pin gets a refusal rather than silent whole-operator behaviour,
/// which is the right way round.
///
/// ## The one thing that did NOT become free
///
/// Restyling **part** of a run. The old shape's `find` was a door to it — a
/// shorter `find` restyles a shorter span — and `whole_operator` deliberately
/// closes that door for this call site. Nothing is lost: `FormatRequest::new`
/// with a real `find` is still there for the day a sweep's byte offsets can be
/// trusted across an extraction, which `TextSelection::runs` still does not
/// offer.
fn request(page: usize, pinned: crate::canvas::textedit::pin::Pinned) -> FormatRequest {
    FormatRequest::whole_operator(page, pinned.span).target(pinned.target)
}

/// Restyle every run the selection covers.
///
/// # The ordering is done here, not asked of the caller
///
/// `runs` arrives in whatever order the caller measured it; this sorts,
/// deduplicates and reverses. A caller that had to remember to pass them
/// backwards is a caller that will one day forget, and the failure would be
/// silent and rare — see the module header.
///
/// # What a refusal does
///
/// **Stops.** A restyle that half-applies and carries on is worse than one that
/// half-applies and says so: the operator sees some of their text change, has no
/// way to tell how much, and the undo stack holds an unknown number of entries.
pub(super) fn apply(doc: &mut OpenDoc, page: usize, runs: &[usize], change: &StyleChange) {
    // ★★★ THE OPERATOR'S POSTURE, SNAPSHOT BEFORE THE BORROW.
    //
    // `super::apply::vector_edit` takes `&mut doc`, so nothing inside the
    // closure can read `doc.settings`. Reading it here is not a convenience:
    // it is the same rule `OpenDoc::settings` itself documents — a setting
    // sampled once per gesture cannot change halfway through a multi-run
    // restyle and leave two runs decided by two different answers.
    let policy = doc.settings.style_policy;
    // ★★★ DERIVED-WHITESPACE RUNS ARE SKIPPED, and the first driven run of this
    // module is why.
    //
    // A sweep across 266 characters of a title block covers many runs, and the
    // ones the extraction *derived* — the word spaces and line breaks between
    // show operators — carry no glyphs. No glyphs means no `GlyphProvenance`,
    // which means no pin, which means nothing to restyle. That is correct and
    // expected: there is no show operator behind them.
    //
    // The loop below used to treat an unpinnable run as a **stop**, on the
    // argument that half-applying and carrying on is worse than half-applying
    // and saying so. That argument is still right for a run that has text in it
    // and would not pin; it was catastrophically wrong here, because the FIRST
    // derived space ends the gesture. Driven, the operator swept a whole label
    // and got `applied=1`.
    //
    // Filtering on `glyphs.is_empty()` rather than on "the pin failed" keeps the
    // two states apart, which is the whole point: a glyphless run is *not text*
    // and is skipped silently; a run with glyphs that will not pin is a real
    // refusal and still stops. `page_text()` is the SHARED cache — no provenance,
    // no second extraction, no cost.
    let glyphless: std::collections::BTreeSet<usize> =
        doc.page_text()
            .map_or_else(std::collections::BTreeSet::new, |text| {
                text.runs
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| r.glyphs.is_empty())
                    .map(|(i, _)| i)
                    .collect()
            });
    let mut ordered: Vec<usize> = runs
        .iter()
        .copied()
        .filter(|r| !glyphless.contains(r))
        .collect();
    ordered.sort_unstable();
    ordered.dedup();
    ordered.reverse();

    let total = ordered.len();
    if total == 0 {
        decline::record_text_style(t::TextStyleRefusal::NoRun);
        return;
    }

    // Accumulated across the whole gesture and surfaced on the LAST successful
    // step, because `super::apply::vector_edit` records the disclosure slot per
    // call and the last write wins. Collecting and emitting once is what stops
    // a three-run restyle showing only the third run's sentence.
    let mut carried: Vec<String> = Vec::new();
    let mut applied = 0_usize;

    for (position, run) in ordered.into_iter().enumerate() {
        let last = position + 1 == total;
        // ★★★ A FRESH read per run, and the unit inside it is the show
        // OPERATOR, not the run. `pin::operators_in_run`'s header carries the
        // argument and the driven run that produced it: a run is closed on
        // geometry and an operator is closed on whatever its producer felt
        // like, so a title-block cell can be one run made of three `Tj`s. Pin
        // the first and pass the run's text as `find` and the engine correctly
        // refuses, on a page where an UNpinned search for the same string
        // succeeds instantly.
        //
        // Descending here too, for the same reason the runs are: an edit only
        // moves bytes at or after itself.
        let mut ops = crate::canvas::textedit::pin::operators(doc, page, run);
        if ops.is_empty() {
            stop(doc, applied, t::TextStyleRefusal::Unpinnable);
            return;
        }
        ops.reverse();
        let final_op = ops.len();
        for (which, op) in ops.into_iter().enumerate() {
            let mut outcome: Option<FormatError> = None;
            let mut notes: Vec<String> = Vec::new();
            super::apply::vector_edit(doc, "format-text", page, 1, |session| {
                // ★★★ THE OPERATOR'S POSTURE MUST NOT REACH THIS CALL, AND THE
                // REASON IS THAT THE PROBE IS A QUESTION, NOT AN ACTION.
                //
                // `Pass 179.0` (engine `71d13aa`, 2026-08-30) made the
                // synthesis gate posture-dependent. Under its new default —
                // `StylePolicy::Auto` — asking for synthetic bold on a page
                // that carries a real bold face no longer refuses: it fakes the
                // weight and reports the face it passed over.
                //
                // ★★ That silently removed this shell's Bold button. The retry
                // below is triggered BY the refusal, so when the refusal
                // stopped arriving the retry stopped happening, and pressing
                // Bold on a page carrying `Calibri-Bold` began thickening
                // `Calibri` instead of using it. Nothing failed anywhere; one
                // test caught it only because it asserts the face BY NAME.
                //
                // ⇒ So the probe pins `Refuse` unconditionally. The refusal is
                // how this shell ASKS which real face is available; it is never
                // shown to the operator as a refusal, because the very next
                // thing that happens is taking the offer it names. The engine
                // says this in as many words: *"a shell must read the posture
                // to know what pressing the button does."* This shell's answer
                // is that the button always means "make this bold", so it reads
                // the posture for a different question — see `policy` below.
                let probe = FormatOptions::default().with_style_policy(StylePolicy::Refuse);

                // ★★ Under `Refuse` a fake is declined even when NO real face
                // was available, and the gate cannot say so.
                //
                // `gate_synthesis` refuses only when a real face exists — a run
                // whose page carries no bold at all sails through it and gets
                // thickened. That is the engine's contract and it is right for
                // the engine. It is not what an operator who chose "never fake
                // it" asked for, so the read-only preview is asked instead.
                //
                // ★ Only under `Refuse`, so the overwhelmingly common path pays
                // nothing for a posture nobody selected.
                if policy == StylePolicy::Refuse
                    && let StyleChange::Weight { bold, italic } = change
                {
                    let want = StyleSynthesis::new(*bold, *italic);
                    // An empty `find` WITH a pin is addressed by the pin alone —
                    // the same addressing `request` uses. An empty find with no
                    // pin is refused by name, which is why the pin is passed.
                    let would_fake = session
                        .preview_style_resolution(page, "", Some(op.pin.span), want)
                        .is_ok_and(|res| {
                            matches!(res.combined, Some(StyleOutcome::WouldSynthesize))
                        });
                    if would_fake {
                        decline::record_text_style(t::TextStyleRefusal::FakingDeclined);
                        outcome = Some(FormatError::NoOp);
                        return Err(FormatError::NoOp);
                    }
                }

                match session.format_text(&change.stamp(request(page, op.pin)), &probe) {
                    Ok(report) => {
                        // ★ Nothing real was on offer, so whatever happened here
                        // is the honest best available. Under `Warn` the fact
                        // that it was FAKED is raised from a quiet disclosure to
                        // a sentence of its own — the engine's own reading of
                        // that posture, applied to the one thing this shell can
                        // observe about it.
                        if policy == StylePolicy::Warn && !report.synthesis.is_none() {
                            notes.push(t::text_style_faked_warning().to_owned());
                        }
                        notes.extend(report.disclosures);
                        Ok(notes.clone())
                    }
                    // ★★ The two-verb retry. The engine refused synthesis
                    // *because a real face is available* and named it; taking
                    // that offer is what makes one Bold button work on every
                    // page. See the module header — the alternative is showing
                    // the operator a sentence telling them to do a thing the
                    // program could have done.
                    // ★★★ `selector`, NOT `real_font`, and the engine had to
                    // tell this project so.
                    //
                    // They are the same string on almost every page and differ
                    // exactly where two `/Font` resources share one
                    // `/BaseFont`. A retry built from the NAME reaches only one
                    // of the twins - *"possibly the twin that refuses"* - and
                    // `selector` is defined as the string that reaches the face
                    // pdfcer actually checked.
                    //
                    // ★★ The failure it prevents is silent and would have read
                    // as a font bug: the operator presses Bold, the engine
                    // names a face it has verified can show the run, the shell
                    // asks for that face BY NAME, and lands on a different
                    // resource that refuses. Every sentence in the chain is
                    // true and the button does nothing.
                    //
                    // => Filed by the engine in `Pass 144.0`'s reply, under
                    // "ACT ON THIS", and this shell kept using `real_font` for
                    // a day. A reply read is not a reply consumed.
                    Err(FormatError::RealFaceAvailable {
                        selector,
                        real_font,
                        style,
                        same_family,
                        ..
                    }) => {
                        let retry = request(page, op.pin)
                            .font(pdfcer_core::text_edit::FontSelector::new(&selector));
                        match session.format_text(&retry, &probe) {
                            Ok(report) => {
                                // ★★ `same_family` gets its own sentence. The
                                // engine says outright that a fallback to
                                // another family is *"a bigger change than a
                                // weight swap"*, and it is the one an operator
                                // will SEE - the letterforms change, not just
                                // their weight. Reporting it as an ordinary
                                // real-face substitution would be true and
                                // would bury the part they can notice.
                                notes.push(if same_family {
                                    t::text_style_used_real_face(style, &real_font)
                                } else {
                                    t::text_style_used_other_family(style, &real_font)
                                });
                                notes.extend(report.disclosures);
                                Ok(notes.clone())
                            }
                            // ★★★ THE THIRD RUNG, AND IT IS NEW.
                            //
                            // On `textedit/format_family.pdf` the gate names
                            // `Times-Bold` — family-matching the run — and
                            // `Times-Bold` remaps `o` to a bullet, so it cannot
                            // cover "hello world" while `Calibri-Bold` on the
                            // same page can. "There is a real bold face, use
                            // it" is useless advice when using it is what just
                            // failed.
                            //
                            // ★★ Until 2026-08-30 this shell STOPPED here, and
                            // the operator got a refusal for a request pdfcer
                            // could have satisfied badly-but-visibly. That was
                            // defensible while the engine itself refused; it is
                            // not defensible now that the engine's own default
                            // posture is "decide and apply" and the operator's
                            // ruling behind it was **"shouldn't have to
                            // intervene"**.
                            //
                            // ⇒ So: fake it, and SAY which real face was tried
                            // and could not show this text. Under `Refuse` the
                            // operator has said they would rather be told, and
                            // the refusal stands.
                            Err(error) => {
                                if policy == StylePolicy::Refuse {
                                    decline::record_text_style(refusal_of(&error));
                                    outcome = Some(error);
                                    return Err(FormatError::NoOp);
                                }
                                // `Auto`, explicitly, whatever the operator
                                // chose: the shell has ALREADY established that
                                // no usable real face exists, so the gate has
                                // nothing left to refuse in favour of, and
                                // pinning the posture here keeps the second call
                                // from re-asking a question already answered.
                                let fake =
                                    FormatOptions::default().with_style_policy(StylePolicy::Auto);
                                let fresh = change.stamp(request(page, op.pin));
                                match session.format_text(&fresh, &fake) {
                                    Ok(report) => {
                                        notes.push(t::text_style_faked_instead(&real_font));
                                        notes.extend(report.disclosures);
                                        Ok(notes.clone())
                                    }
                                    // The RETRY's refusal is reported, not this
                                    // one: it names the face that was tried,
                                    // which is the half the operator can act on.
                                    Err(_) => {
                                        decline::record_text_style(refusal_of(&error));
                                        outcome = Some(error);
                                        Err(FormatError::NoOp)
                                    }
                                }
                            }
                        }
                    }
                    Err(error) => {
                        decline::record_text_style(refusal_of(&error));
                        outcome = Some(error);
                        Err(FormatError::NoOp)
                    }
                }
            });

            if let Some(error) = outcome {
                // ★★ A refusal that reaches the operator and not the trace is a
                // refusal nobody debugging can see. This branch returned
                // silently for one build, and a driven run then polled twenty
                // seconds over a trace holding eleven completed edits, neither
                // a summary nor a decline, and reported "Bold was pressed and
                // nothing happened" about a gesture that had done eleven things
                // and stopped on purpose.
                if applied > 0 {
                    decline::record_text_style(t::TextStyleRefusal::PartOnly);
                    emit_carried(doc, page, applied, total, &carried);
                }
                let detail = error.to_string();
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!(
                        "text-style-declined page={page} run={run} applied={applied} runs={total} detail={detail}"
                    )
                });
                return;
            }
            applied += 1;
            carried.extend(notes);

            if last && which + 1 == final_op {
                emit_carried(doc, page, applied, total, &carried);
            }
        }
    }

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            // ★ `applied` counts show OPERATORS and `runs` counts runs, and they
            // are different numbers — the first driven pass printed
            // `applied=19 of=14`, which reads as nonsense because it was two
            // units under one comparison. A count is only readable beside a
            // total of the same thing.
            "text-style-applied page={page} change={} applied={applied} runs={total}",
            change.label()
        )
    });
}

/// Re-record the whole gesture's disclosures, on the last step.
///
/// A second `vector_edit` would be a second undo entry, so this writes the
/// disclosure slot directly. That is the one place in this module that reaches
/// past the funnel, and it is sound because it changes **no document**: the
/// epoch it stamps is the one the final edit already bumped.
fn emit_carried(doc: &OpenDoc, page: usize, applied: usize, total: usize, carried: &[String]) {
    let mut notes: Vec<String> = carried.to_vec();
    if total > 1 {
        notes.push(t::text_style_multi(applied));
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("text-style-disclosed page={page} n={}", notes.len())
    });
    super::disclosure::record_edit_disclosure(Some(super::disclosure::EditDisclosure {
        epoch: doc.edit_epoch,
        notes,
    }));
}

/// Record that the gesture stopped, and how far it got.
fn stop(doc: &mut OpenDoc, applied: usize, why: t::TextStyleRefusal) {
    let _ = doc;
    decline::record_text_style(why);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("text-style-declined applied={applied} detail=unpinnable")
    });
}

/// Which operator-facing sentence a refusal earns.
///
/// ★ The engine's own `Display` prose is deliberately **not** the sentence.
/// `check-ui-strings.sh` exclusion 3 says in as many words that an error type's
/// prose is not permission to route UI text through it; the prose goes to the
/// trace, where whoever is debugging wants it, and the catalog says the same
/// thing in the operator's terms with the remedy first.
///
/// Three named cases and a catch-all, chosen because they are the three an
/// operator can *do something about*. Everything else — encryption, a no-op
/// request, a page index — is either impossible from this surface or is not
/// improved by being subdivided.
fn refusal_of(error: &FormatError) -> t::TextStyleRefusal {
    match error {
        FormatError::TargetFontMissing(_) => t::TextStyleRefusal::FaceNotOnPage,
        FormatError::ShearUnsupported(_) => t::TextStyleRefusal::ItalicWouldMove,
        FormatError::CoverageFailure(_) => t::TextStyleRefusal::FaceLacksCharacters,
        _ => t::TextStyleRefusal::Other,
    }
}

#[cfg(test)]
mod tests;

/// **Re-wrap one paragraph to its own box.** `OPERATOR_REQUESTS.md` **O54**.
///
/// ★★★ The disclosures are the ENGINE'S, passed through verbatim.
/// `ReflowApplyReport::disclosures` is already a `Vec<String>` written for an
/// operator, and it names the things this shell could not: how many lines the
/// paragraph had before and after, whether justification was applied, whether
/// the block now overflows the page. Re-wording them here would be a second
/// author for one fact — the rule `textstyle`'s synthesis disclosure already
/// follows, and for the same reason.
///
/// ★★ **The refusal is the interesting half.** Unlike every other verb in this
/// module, `reflow_block` is planned against the base document and refuses a
/// page this session has already edited. That is not a defect and it is not
/// rare: one typed character makes it fire. The remedy is specific — save and
/// reopen — and `vector_edit`'s error arm traces but does not word a refusal, so
/// this one is worded here, before the funnel, on the one condition the shell
/// can see without asking.
pub(super) fn reflow(doc: &mut OpenDoc, page: usize, block: usize) {
    // ★★★ Asked BEFORE the attempt, so the operator is told the remedy rather
    // than shown a silence. `edit_epoch` is non-zero exactly when this session
    // has changed the document, which is the shell-side shadow of the engine's
    // own condition — it is broader (an edit to ANOTHER page also trips it) and
    // deliberately so: a sentence that says *"save and reopen"* one page too
    // eagerly costs a save, and one that says nothing costs an operator who
    // thinks the feature is broken.
    //
    // ⇒ The engine's own refusal remains the backstop and is traced by the
    // funnel. This is the wording, not the gate.
    if doc.edit_epoch != 0 {
        super::record_note(
            doc.edit_epoch,
            crate::text::textedit::reflow_after_edit().to_owned(),
        );
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("reflow-declined page={page} block={block} reason=session-has-edits")
        });
        return;
    }
    // ★★★ **The cropbox is supplied, and supplying it is the whole of the
    // overflow disclosure.**
    //
    // `ReflowRequest::page_cropbox` defaults to `None`, which means *"do not
    // check whether the re-wrapped block runs off the page"* — and a shell that
    // takes that default has silently declined a rule-4 disclosure it was
    // offered for free. Re-wrapping can only ever ADD lines when it narrows a
    // block, and lines are added downward, so *"your paragraph now ends below
    // the bottom of the sheet"* is a real outcome the operator cannot see from
    // a canvas that has scrolled.
    //
    // ⇒ It is the same argument the engine makes for its own disclosures and
    // the same shape as every other one this shell forwards: render normally,
    // report separately. The report arrives in `report.disclosures` and goes to
    // the status line verbatim.
    let request = pdfcer_core::text_edit::ReflowRequest::new();
    let request = match doc.pages.get(page) {
        Some(page_ref) => request.with_page_cropbox(page_ref.crop_box),
        // ★ A page index this document does not have. The reflow below will
        // refuse it by name; declining to guess a cropbox is what keeps the
        // refusal ABOUT the missing page rather than about a rectangle this
        // shell invented for it.
        None => request,
    };
    super::apply::vector_edit(doc, "reflow-block", page, 1, |session| {
        session.reflow_block(page, block, &request).map(|report| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    // `-applied`, per the convention `forms::import_data`
                    // records: the funnel writes its own bare-named line for
                    // the same edit and `.last()` would read that one.
                    "reflow-block-applied page={page} block={block} lines={}->{} \
                         justified={} height_delta={:.2}",
                    report.lines_before,
                    report.lines_after,
                    report.justified_lines,
                    report.height_delta
                )
            });
            let mut notes = report.disclosures;
            // ★★ The line count, added because the engine's own list does
            // not always carry one and it is the fact the operator can
            // check by looking. A reflow that changed nothing is a correct
            // outcome — the paragraph already fitted — and reads as a
            // failure without a sentence.
            if report.lines_before == report.lines_after && notes.is_empty() {
                notes.push(crate::text::textedit::reflow_unchanged().to_owned());
            }
            notes
        })
    });
}
