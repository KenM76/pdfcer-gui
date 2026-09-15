//! # `app::actions::textstyle` — changing how existing text looks
//!
//! > *"We should also have all the font tools available that Word does."*
//! > — `OPERATOR_REQUESTS.md` **O37**
//!
//! O37's inventory read the engine's text verbs — `add_text`, `edit_text`,
//! `delete_text_run` — and concluded, in a table with a column of crosses, that
//! pdfcer could choose how text looked when it was created and could not change
//! how existing text looked at all. Every cross in that column was wrong:
//! `EditSession::format_text` had shipped weeks earlier and had been extended
//! twice since. **An absence claim about a crate you do not build is a claim
//! about every route, and one verb is not every route.** This module exists
//! because somebody asked the engine instead of inferring from its index.
//!
//! ## What can actually be changed
//!
//! | control | verb | limit |
//! |---|---|---|
//! | **size** | `set_size` | none — the `Tf` operand changes and the line is relaid out |
//! | **colour** | `set_fill` | none. pdfcer stores the SPACE the operator chose (`rg`/`g`/`k`) instead of force-converting to DeviceRGB the way Acrobat does |
//! | **face** | `set_font` | the target must **already be a font resource on the page**; refused by name otherwise (`FF-C`) |
//! | **bold / italic** | `set_style` | walks the ladder below; one named refusal, on italic only |
//!
//! ## The style ladder belongs to the engine, and rung 2 is why that matters
//!
//! [`FormatRequest::set_style`] answers *"make this bold"* — as against
//! `set_synthetic`'s narrower *"thicken the strokes"* — by walking four rungs
//! inside the engine and reporting which one bound:
//!
//! | rung | what it binds | reported as |
//! |---|---|---|
//! | 1 | a real face already on the page that claims the style and passes the coverage gate — same family first, then any family | `StyleRung::RealFaceOnPage` |
//! | 2 | the **standard-14 sibling of the run's own family**, as a new `/Font` resource, nothing embedded | `StyleRung::StandardFourteenSibling` |
//! | 3 | a `--font-dir` donor | not built |
//! | 4 | synthesis — the stroke or the shear | `StyleRung::Synthetic` |
//!
//! Rung 2 is the rung that fires constantly and the one a shell-side walk
//! cannot reach. A CAD title block exported with `Helvetica` and nothing else
//! carries **no bold font resource at all**, so a gate that asks only *"is a
//! real bold face on this page?"* answers no — which is true of the page and
//! false of the world. `Helvetica-Bold` is one of the fourteen faces every
//! conforming reader is **required** to carry, needs no font file (ISO 32000-1
//! §9.6.2.2), and costs about sixty bytes to name. Thickening the strokes of
//! `Helvetica` instead is visible at 1:1 on a plotter and invisible to a test
//! that asserts the edit epoch moved.
//!
//! ⇒ So the walk is **not** re-implemented here. This module sets one verb,
//! passes the operator's posture straight through, and words what the engine
//! reports. A synthetic weight is the regular face thickened by stroking; a
//! synthetic slant is the upright face sheared. `R90` means neither is ever a
//! preference — only an explicit, per-use request — and the report says which
//! fired, in words passed through rather than re-written.
//!
//! ## Bold and Italic are never greyed
//!
//! The engine's instruction, and this shell obeys it: *"Do not grey out a bold
//! button. Offer it, and surface the disclosure when synthesis fires."*
//!
//! Greying would need this shell to predict a refusal that depends on a per-run
//! glyph-coverage test against a **candidate** face. `run_repertoire` answers a
//! narrower question — which characters the run's *current* font accepts — and
//! is cheap enough that `canvas::textedit::repertoire` calls it on every caret
//! landing. The question greying needs answered has only one instrument,
//! `preview_font_resources_for`, and the engine's own note on it says what that
//! costs: it walks every operation on the page, 129,758 objects on the
//! operator's benchmark sheet. Per frame, per button, that is not a greying
//! rule — it is a hang.
//!
//! ⚠ Both halves of that matter to whoever re-opens the question. *Nothing can
//! be measured here* is false; a verb measures exactly that. What keeps the
//! buttons live is the cost of the one instrument that measures the **right**
//! thing, not the absence of any instrument.
//!
//! ## Nothing here works around the engine
//!
//! A twenty-line shell-side search for a different bold resource would work,
//! and it would be this project second-guessing pdfcer's font selection — and
//! the workaround every other consumer would then have to write for
//! themselves. **A workaround is a boundary defect, and its side effects are
//! usually invisible from inside the workaround.** The honest move is to report
//! the gap and let the engine close it once, for everyone.
//!
//! The same rule decides which field a report is read from. Two `/Font`
//! resources can share one `/BaseFont`, so anything built from the **name**
//! reaches only one of the twins — possibly the twin that refuses. `selector`
//! is defined as the string that reaches the face pdfcer actually checked, and
//! it is therefore the field to use.
//!
//! ## How the engine's reports are read, and why each field is read that way
//!
//! * [`StyleLadder::same_family`] is `Option<bool>`, and `None` means *nothing
//!   was bound*. It must **not** be flattened into `Some(false)`: the engine
//!   says so at the field, and the difference is *"the same typeface, heavier"*
//!   against *"a different typeface, which you will see on a plot"*.
//!   `family_stem` is private and engine invariant `R74` forbids this shell
//!   re-deriving it, so this flag is the only honest source for that sentence.
//! * `PassedOver { base_font, reason, refusal }` is structured, and
//!   `Refusal::character` gives the offending character. Were it a
//!   `Vec<String>` of `"BaseFont (reason)"`, saying *"pdfcer tried
//!   `Times-Bold` and it has no `o`"* in this shell's voice would mean a
//!   locator for the engine's message format living in a GUI, breaking
//!   silently the first time a reason gained a parenthesis — so the sentence
//!   would not be written at all, and the operator would be told less because
//!   of a string format.
//! * `EditSession::preview_style_ladder` is what the hover hints run, and it
//!   previews **the ladder**, not the `R90` gate. The gate answers only *"no
//!   real face on this page claims that style and covers this run"*, which
//!   stopped being the same question as *"what will pressing Bold do?"* the
//!   moment rung 2 existed: the standard-14 sibling is by construction **not on
//!   the page**, so the gate cannot see it. A tooltip built on the gate
//!   promises thickening on exactly the pages where a real `Helvetica-Bold` is
//!   about to be bound, and the status line afterwards reports the real face —
//!   two instruments the operator can consult, disagreeing by construction, on
//!   the commonest page in the working set. `preview_style_ladder` runs the
//!   same planner `format_text` runs, walks the page's content once, stages
//!   nothing, and takes the [`FormatOptions`] the commit will use, so under
//!   `Refuse` it previews the refusal rather than predicting a synthesis that
//!   would never happen.
//!
//! [`FormatRequest::set_style`]: pdfcer_core::text_edit::FormatRequest::set_style
//! [`StyleLadder::same_family`]: pdfcer_core::text_edit::StyleLadder::same_family
//!
//! ## Why the runs are edited in descending order
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
//! adding a *combined* verb per case, which is how markup authoring got its
//! opacity in one entry rather than two. A restyle across a paragraph therefore
//! takes several `Ctrl+Z` presses to take back.
//!
//! Disclosed by the count rather than left to be discovered, and filed on the
//! request channel. **Not** worked around here: a shell-side coalesce would
//! work and would leave every other consumer with the same defect, which is the
//! boundary rule above stated from the other side.

use pdfcer_core::settings::StylePolicy;
use pdfcer_core::text_edit::{
    FormatError, FormatOptions, FormatReport, FormatRequest, NewFill, StyleLadder, StyleRung,
    StyleSynthesis,
};

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::text::status as t;
use crate::text::textedit::ReflowRefusal;

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
    /// Deliberately **not** named `Synthetic`, because whether it ends up
    /// synthetic is the engine's decision and not the operator's: the variant
    /// maps to `set_style` and the engine walks its four-rung ladder. The
    /// operator asked for bold; how bold is achieved on this page is a fact
    /// they are told afterwards.
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
            // `style`, **not** `synthetic`, and the one word is the whole
            // feature. `set_synthetic` means *"thicken the strokes"* and is
            // gated; `set_style` means *"make this bold"* and walks the
            // ladder. What the operator sees is a real `Helvetica-Bold`
            // instead of a stroked `Helvetica` on every page that carries no
            // bold resource, which is most CAD title blocks. Module header.
            //
            // ⚠ The two are **mutually exclusive per axis**: combining
            // `set_style` with `set_font`, or with an overlapping
            // `set_synthetic`, is `FormatError::Unsupported`. Nothing else in
            // this table sets either, and `Face` is its own variant, so one
            // press is one verb.
            Self::Weight { bold, italic } => req.style(StyleSynthesis::new(*bold, *italic)),
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

/// The request for one show operator, addressed **by pin alone**.
///
/// Built fresh per run per step; see the module header on why a batch of these
/// taken up front would be wrong.
///
/// # Why there is no `find` string
///
/// `FormatRequest::whole_operator(page, span)` is exactly
/// `new(page, "").pinned(span)`, and the named constructor is used because it
/// says what it means. The alternative is to pass the run's own text as
/// `find`, which requires slicing that text into per-operator pieces — **a
/// second locator living beside the engine's**, and the class of defect where
/// two locators agree on every fixture and disagree on a ligature. There is no
/// such slicing here and `pin::Operator` carries no `find`.
///
/// An empty `find` with **no** pin is still refused by name, which is the right
/// way round: a caller that forgot to pin gets a refusal rather than silent
/// whole-operator behaviour.
///
/// # What this deliberately closes off
///
/// Restyling **part** of a run. A shorter `find` restyles a shorter span, and
/// `whole_operator` shuts that door for this call site. `FormatRequest::new`
/// with a real `find` remains for the day a sweep's byte offsets can be trusted
/// across an extraction, which `TextSelection::runs` does not yet offer.
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
    restyle(doc, page, runs, change);
    resweep(doc, page);
}

/// **Keep the operator's sweep alive across the edit that just happened** —
/// `OPERATOR_REQUESTS.md` **O198**.
///
/// # The defect, in the operator's own gesture
///
/// Sweep three words. Press **Bold**: it applies. Press **Italic**: nothing
/// happens, and nothing says why. The wash has also gone from the page.
///
/// Bold is an edit, `super::apply::vector_edit` bumps `edit_epoch` on success,
/// and `TextSelection::live` is an equality test against that number. From the
/// frame after Bold the selection reports itself stale: it paints nothing and
/// `TextSelection::runs` hands back an empty list, so Italic's operand is empty
/// and `restyle` above declines with `NoRun`. Every pair of presses needed a
/// re-sweep in between, which is not a thing any editor in the class asks for.
///
/// # Why it is a re-resolution and not a re-stamp
///
/// `canvas::textsel`'s header §7 is emphatic that painting stored geometry
/// against a moved revision is the one thing rule 4 forbids outright, and it is
/// right: a restyle that changed a point size moves every glyph after it, so
/// the old quads would wash the wrong pixels and a subsequent restyle could act
/// on the wrong runs. So nothing is re-stamped. `textsel::reresolve` re-runs the
/// whole resolution from the operator's two positions against a fresh
/// extraction, and refuses unless the characters covered are identical. See its
/// docs for the argument; this function is only the wiring and the borrow
/// discipline.
///
/// # Called on every exit of `restyle`, including the refusals
///
/// A gesture that applied eleven runs and then stopped left the document
/// edited, so the selection is exactly as stale as a successful one. Wrapping
/// `restyle` rather than appending to its tail is what makes that true without
/// four call sites having to remember it — `restyle` has five early returns.
///
/// A no-op when nothing was swept, which is the whole clicked-object rung:
/// that operand comes from `app::textoperand`'s `Cache`, whose stamp includes
/// the epoch, so it re-resolves itself and needs nothing here.
///
/// # The cost, and why it is not a new one
///
/// One page extraction. `OpenDoc::page_text` is the shared cache and the edit
/// has just invalidated it, so this call pays for a rebuild — but the canvas
/// asks for the same extraction on its very next frame to hit-test the pointer,
/// so the work happens either way and this only moves it earlier by a frame.
fn resweep(doc: &mut OpenDoc, page: usize) {
    let Some(previous) = doc.text_selection.take() else {
        return;
    };
    // The `Ref` into the text cache and the `&mut doc.selection` write
    // cannot overlap, so the whole read is a block that yields a value. This is
    // the short-borrow discipline `app::cache::page_text`'s docs ask for, and
    // holding the `Ref` across the assignment below does not compile — which
    // is the borrow checker enforcing it rather than a convention doing so.
    let renewed = {
        let Some(text) = doc.page_text() else {
            return;
        };
        let Some(page_ref) = doc.pages.get(page) else {
            return;
        };
        let ctx = crate::canvas::textsel::PageContext {
            text: &text,
            page: page_ref,
            index: page,
            epoch: doc.edit_epoch,
        };
        crate::canvas::textsel::reresolve(&ctx, &previous)
    };
    let kept = renewed.is_some();
    doc.text_selection = renewed;
    crate::diag::trace(move || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("text-selection-resweep page={page} kept={kept}")
    });
}

/// The restyle itself. See [`apply`], which is this plus the re-sweep.
fn restyle(doc: &mut OpenDoc, page: usize, runs: &[usize], change: &StyleChange) {
    // **The operator's posture, sampled before the borrow.**
    //
    // `super::apply::vector_edit` takes `&mut doc`, so nothing inside the
    // closure can read `doc.settings`. Reading it here is not a convenience:
    // it is the same rule `OpenDoc::settings` itself documents — a setting
    // sampled once per gesture cannot change halfway through a multi-run
    // restyle and leave two runs decided by two different answers.
    let policy = doc.settings.style_policy;
    // **Derived-whitespace runs are skipped**, and skipping them is a
    // different act from tolerating a failed pin.
    //
    // A sweep across a title block covers many runs, and the ones the
    // extraction *derived* — the word spaces and line breaks between show
    // operators — carry no glyphs. No glyphs means no `GlyphProvenance`, which
    // means no pin, which means nothing to restyle. That is correct: there is
    // no show operator behind them.
    //
    // ⚠ The loop below treats an unpinnable run as a **stop**, because
    // half-applying and carrying on is worse than half-applying and saying so.
    // That is right for a run that has text in it and will not pin, and
    // catastrophic if it also catches the derived spaces: the **first** word
    // space then ends the gesture, and a swept label restyles one run out of
    // however many it had.
    //
    // ⇒ Filtering on `glyphs.is_empty()` rather than on "the pin failed" is
    // what keeps the two states apart. A glyphless run is *not text* and is
    // skipped silently; a run with glyphs that will not pin is a real refusal
    // and still stops. `page_text()` is the **shared** cache — no provenance,
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
        // A **fresh** read per run, and the unit inside it is the show
        // **operator**, not the run. `pin::operators_in_run`'s header carries
        // the argument: a run is closed on geometry and an operator is closed
        // on whatever its producer felt like, so a title-block cell can be one
        // run made of three `Tj`s. Pin the first and pass the run's whole text
        // as `find` and the engine correctly refuses, on a page where an
        // unpinned search for the same string succeeds instantly.
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
                // **The operator's posture, passed straight through** —
                // never pinned to a value here to provoke an informative
                // refusal.
                //
                // `set_style` walks all four rungs itself, so there is no
                // question left to ask by forcing one, and the posture goes
                // where it belongs: to the engine, as the operator set it.
                // Under `Auto` the ladder decides and discloses; under `Warn`
                // it does the same and [`ladder_note`] raises the synthetic
                // case to a sentence of its own; under `Refuse` the ladder's
                // **fourth** rung returns `SynthesisRefusedByPosture` — the
                // wide reading of *"never fake it"*, decided by the side that
                // knows what it tried.
                //
                // ⇒ One engine call per operator, and one only. A probe to ask
                // which face is available, plus a pre-check to predict the
                // posture refusal, plus the commit, is three reads of a page
                // whose answer can change between them.
                let options = FormatOptions::default().with_style_policy(policy);

                match session.format_text(&change.stamp(request(page, op.pin)), &options) {
                    Ok(report) => {
                        // The ladder's own sentence **first**, then the engine's
                        // disclosures. The order is the operator's reading
                        // order: what happened to their text, then the details
                        // pdfcer owes them about how.
                        if let Some(note) = ladder_note(&report, policy) {
                            notes.push(note);
                        }
                        notes.extend(report.disclosures);
                        Ok(notes.clone())
                    }
                    Err(error) => {
                        decline::record_text_style(refusal_of(&error));
                        outcome = Some(error);
                        Err(FormatError::NoOp)
                    }
                }
            });

            if let Some(error) = outcome {
                // A refusal that reaches the operator and not the trace is a
                // refusal nobody debugging can see. Without this line a
                // gesture that applied eleven runs and then stopped on purpose
                // leaves a trace holding eleven completed edits, no summary
                // and no decline — which reads to whoever is driving it as
                // *"Bold was pressed and nothing happened"*.
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
            // `applied` counts show **operators** and `runs` counts runs, and
            // they are different numbers: a trace reading `applied=19 of=14`
            // is nonsense because it puts two units under one comparison. A
            // count is only readable beside a total of the same thing, so both
            // names are spelled out.
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

/// The one sentence the style ladder earns, or `None` to let the engine speak.
///
/// # One sentence per rung, and never two about one outcome
///
/// [`StyleLadder::rung`] is the engine's own account of how it made the text
/// bold, and it is a closed question with one answer — so this is a **mapping,
/// not an accumulation**. Pushing a sentence per branch taken is how one
/// `Warn`-posture synthesis produces both *"pdfcer had to fake this"* and *"no
/// real face was available"* about a single event.
///
/// The `Warn` and default synthetic sentences are therefore **mutually
/// exclusive**, not additive. `Warn` means the operator asked to be told
/// prominently; they are told once, in the catalog's `Warn` words. Adding the
/// ordinary sentence underneath would be the same fact twice, and a disclosure
/// repeated is a disclosure skipped.
///
/// # What returns `None`, and why silence is the right answer there
///
/// `None` is not "nothing happened" — [`FormatReport::disclosures`] is appended
/// immediately after this in every case, and the engine's ladder disclosure is
/// already in it, naming the rung, the bound face and every face passed over.
/// `None` means **this shell has nothing to add that the engine did not say
/// better**, and there are three such cases:
///
/// * **A change that is not a style change.** `style_ladder` is `None` unless
///   `set_style` was set, so size, colour and face fall out here.
/// * **A rung this build does not know.** [`StyleRung`] is `#[non_exhaustive]`;
///   rung 3 (`--font-dir` donors) is not built and a fifth rung is possible.
///   An invented sentence for an outcome nobody here has seen is
///   worse than the engine's precise one — it would be untestable, and it would
///   assert something about a mechanism this file has never read. Traced so it
///   is visible to whoever wires it.
/// * **A MIXED outcome** — a real face bound for one axis while the other was
///   synthesised. The engine supports it per axis (an exceed over Acrobat) and
///   it is **unreachable from this shell**, because both controls send exactly
///   one axis: `dispatch::format` sends `{bold: true, italic: false}` and the
///   properties panel sends `{bold: false, italic: true}`. A sentence for a
///   state no gesture can produce is a sentence no test can falsify.
///
/// # Rung 1 is two sentences, keyed on `same_family`
///
/// **Nothing in this function reads `FormatReport::font_change`.** Naming both
/// `/BaseFont`s — *"was set in Calibri and is now set in Times-Bold"* — is
/// entitled and unguessed, and it is weaker prose than the fact the operator
/// needs, which is whether the letterforms changed. Engine invariant `R74`
/// forbids this shell re-deriving `family_stem` to work that out, so
/// [`StyleLadder::same_family`] is the only honest source, and the two
/// sentences are:
///
/// * `Some(true)` — the page carried the bold or italic form of this text's
///   own typeface. The letterforms are unchanged, and the sentence says so,
///   because that is the outcome the operator needs no warning about.
/// * `Some(false)` — the only real face that could show the run belonged to
///   another typeface. The letterforms will look different, not just heavier,
///   and on a plot that is visible where a status line is not.
///
/// `None` means **nothing was bound**, and it must not be flattened into
/// `Some(false)` — the engine says so at the field. On a rung that bound a face
/// it is an engine invariant breaking, so it is **traced, not guessed at**: a
/// third sentence for it would be a sentence about a state the engine says
/// cannot arise here, which is untestable by construction. The same reasoning,
/// and the same shape, as the `no-bound-face` arm below.
///
/// [`StyleLadder::same_family`]: pdfcer_core::text_edit::StyleLadder::same_family
///
/// [`StyleLadder`]: pdfcer_core::text_edit::StyleLadder
/// [`StyleLadder::rung`]: pdfcer_core::text_edit::StyleLadder::rung
/// [`StyleRung`]: pdfcer_core::text_edit::StyleRung
/// [`FormatReport::disclosures`]: pdfcer_core::text_edit::FormatReport::disclosures
fn ladder_note(report: &FormatReport, policy: StylePolicy) -> Option<String> {
    let ladder: &StyleLadder = report.style_ladder.as_ref()?;
    let bold = ladder.requested.bold();
    let italic = ladder.requested.italic();

    // A real rung that left one axis synthetic — unreachable from this shell.
    // Traced rather than guessed at; see the doc comment.
    let mixed = !ladder.synthesised.is_none() && !matches!(ladder.rung, StyleRung::Synthetic);

    match ladder.rung {
        _ if mixed => {
            trace_rung(ladder, "mixed");
            None
        }
        StyleRung::AlreadyStyled => Some(t::text_style_already_that_way(bold, italic)),
        StyleRung::RealFaceOnPage => match (ladder.bound.as_deref(), ladder.same_family) {
            (Some(to), Some(true)) => Some(t::text_style_used_sibling_face(bold, italic, to)),
            (Some(to), Some(false)) => Some(t::text_style_used_other_family(bold, italic, to)),
            // A rung that bound a face and reports no family verdict would be
            // an engine invariant breaking — `same_family` is `None` only when
            // `bound` is. Traced rather than asserted: a shell that panics on
            // the other side's invariant takes the operator's document down
            // over a sentence it could simply not have said.
            _ => {
                trace_rung(ladder, "no-family-verdict");
                None
            }
        },
        StyleRung::StandardFourteenSibling => match ladder.bound.as_deref() {
            Some(to) => Some(t::text_style_used_standard_face(bold, italic, to)),
            None => {
                trace_rung(ladder, "no-bound-face");
                None
            }
        },
        // Mutually exclusive with the `Warn` sentence, never both. See above.
        StyleRung::Synthetic => Some(if policy == StylePolicy::Warn {
            t::text_style_faked_warning().to_owned()
        } else {
            t::text_style_faked(bold, italic)
        }),
        _ => {
            trace_rung(ladder, "unknown-rung");
            None
        }
    }
}

/// Trace a ladder outcome this build has no sentence for.
///
/// Its own function so every `None` arm above costs one readable line, and so
/// the trace format is written once. `StyleRung` has a `Display` that spells the
/// rung in words (*"rung 2: the standard-14 sibling"*), which is what a future
/// session grepping a trace for an unhandled outcome needs to see.
fn trace_rung(ladder: &StyleLadder, why: &'static str) {
    let rung = ladder.rung.to_string();
    let bound = ladder.bound.as_deref().unwrap_or("-").to_owned();
    let requested = ladder.requested.axes();
    let synthesised = ladder.synthesised.axes();
    // Spelled by hand, **not** `{:?}` over the `Option<bool>`. A machine-read
    // field rendered through `Debug` is a standing defect: a driven check
    // keying on `family=false` matches `family=Some(false)` and `family=None`
    // alike once somebody widens the pattern, and then reports the opposite of
    // the truth while quoting the truth.
    let family = match ladder.same_family {
        Some(true) => "same",
        Some(false) => "other",
        None => "none-bound",
    };
    let passed = ladder.passed_over.len();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "text-style-ladder-unhandled why={why} rung={rung} bound={bound} requested={requested} synthesised={synthesised} family={family} passed={passed}"
        )
    });
}

/// Which operator-facing sentence a refusal earns.
///
/// The engine's own `Display` prose is deliberately **not** the sentence.
/// `check-ui-strings.sh` exclusion 3 says in as many words that an error type's
/// prose is not permission to route UI text through it; the prose goes to the
/// trace, where whoever is debugging wants it, and the catalog says the same
/// thing in the operator's terms with the remedy first.
///
/// Four named cases and a catch-all, chosen because they are the ones an
/// operator can *do something about*. Everything else — encryption, a no-op
/// request, a page index — is either impossible from this surface or is not
/// improved by being subdivided.
///
/// # The wildcard is the hazard, and every named arm here is hand-added
///
/// `FormatError` is `#[non_exhaustive]` and this `match` ends in `_`, so the
/// compiler cannot tell anyone that a new variant arrived — it lands in
/// `Other`, gets the generic sentence, and nothing anywhere goes red:
///
/// > *"any `match` of yours ending in `_` just gained a variant it will not
/// > distinguish, and the one it will not distinguish is the one you care
/// > about."*
///
/// [`FormatError::SynthesisRefusedByPosture`] is the case that proves it, and
/// the single most important refusal this surface can produce: it fires
/// **only** when the operator explicitly chose `StylePolicy::Refuse` and pdfcer
/// then walked every real rung and found nothing. Left to the wildcard, an
/// operator who ticked *"never fake it"* would press Bold, have their setting
/// honoured exactly as asked, and be told *"pdfcer could not change that text"*
/// — which reads as a malfunction rather than as their own instruction being
/// obeyed.
///
/// [`FormatError::SynthesisRefusedByPosture`]: pdfcer_core::text_edit::FormatError::SynthesisRefusedByPosture
fn refusal_of(error: &FormatError) -> t::TextStyleRefusal {
    match error {
        FormatError::TargetFontMissing(_) => t::TextStyleRefusal::FaceNotOnPage,
        FormatError::ShearUnsupported(_) => t::TextStyleRefusal::ItalicWouldMove,
        // The **only** arm that carries a payload out of the engine's error,
        // because it is the only refusal here an operator can act on without
        // guessing. `Refusal::remedy_faces` is the faces that *would* show
        // this run — the same list the engine's own message names in prose,
        // structured. Cloned rather than borrowed because the sentence
        // outlives the error: it is recorded in the decline slot and read by
        // the bar on later frames.
        //
        // ⚠ Do not reconstruct that list by splitting `Refusal::message` on a
        // clause, and do not substitute the public helper that looks like the
        // answer — the engine measured it as **wrong** on the very fixture
        // this refusal exists for. A structured field is the only source.
        FormatError::CoverageFailure(refusal) => {
            t::TextStyleRefusal::FaceLacksCharacters(refusal.remedy_faces.clone())
        }
        // The operator's own setting, reported back as their setting. This
        // variant carries `style`, `run_font`, `passed` and `flag`; none of them
        // reaches the sentence, because the remedy is *"turn the setting off"*
        // and naming the face pdfcer refused to fake would invite the operator
        // to go looking for a face that does not exist. The full detail is in
        // the `text-style-declined detail=` trace, where debugging wants it.
        FormatError::SynthesisRefusedByPosture { .. } => t::TextStyleRefusal::FakingDeclined,
        _ => t::TextStyleRefusal::Other,
    }
}

/// **Which sentence the engine's reflow refusal earns.**
///
/// # The same shape as [`refusal_of`] above, and for the same rule
///
/// The engine's error is a *diagnosis*; the operator needs a *next step*, and
/// the two are not the same list. `ReflowApplyError` carries many more variants
/// than there are things an operator can do — save and reopen, remove the
/// protection, accept that this text cannot be addressed, or stop — so this
/// maps rather than transcribes. Wording each engine variant separately would
/// put operator-facing decisions inside an error type written for a library.
///
/// # No wildcard, and that is the whole safety of the function
///
/// `ReflowApplyError` is `#[non_exhaustive]`. A `match` on it ending in `_`
/// therefore gains new variants **silently**, and the rule is stated in
/// `text::textedit::reflow_refusal`'s header in exactly this shape:
///
/// > *"any `match` of yours ending in `_` just gained a variant it will not
/// > distinguish, and the one it will not distinguish is the one you care
/// > about."*
///
/// A wildcard here would route a newly carved-out refusal into
/// [`ReflowRefusal::Other`] — the generic sentence, no cause, no remedy —
/// while the engine's whole reason for carving it out was that it had one.
///
/// ⇒ So everything except [`ReflowApplyError::Encrypted`] routes through
/// [`ReflowApplyError::decline`]. [`ReflowDecline`] is deliberately **not**
/// `#[non_exhaustive]` — the engine made the same written promise it made for
/// `RefusalKind` — so the `match` below is **compiler-proved complete**. A
/// future engine *refusal* joins an existing arm and keeps its correct
/// sentence; a future *decline* is a build failure here, which is the one place
/// it should be.
///
/// ⚠ Never narrow an operator sentence onto a cause the engine can no longer
/// produce. `Unsupported(String)` carries many distinct refusals under one
/// discriminant, only one of which ever had *save and reopen* as its remedy, so
/// it earns [`ReflowRefusal::EngineDeclined`] — which names no cause and
/// promises no remedy — rather than a specific sentence that would be wrong for
/// every other member of the set.
///
/// `Encrypted` is matched by variant, above the discriminant and on purpose.
/// `ReflowDecline::StructureForbids` covers *"encryption, **or** a save that
/// the edit gates refused"*, and those need different sentences — one says
/// *remove the protection*, the other cannot say anything so specific. Name the
/// narrower cause wherever the engine gives a narrower variant, and nowhere
/// else.
///
/// | decline | shell refusal | why |
/// |---|---|---|
/// | [`ReflowDecline::RetryAfterSaveAndReopen`] | [`ReflowRefusal::PageAlreadyEdited`] | ⚠ **unreachable at engine `025d703d`** — its only source error has no producer in the engine. The arm is mandatory (this `match` is compiler-proved complete over an exhaustive enum) and the sentence is retained; `check-unreachable-refusals` is what notices if it comes back |
/// | [`ReflowDecline::NotFound`] | [`ReflowRefusal::CannotTrace`] | *"glyphs cannot be traced back to their show operators"* is that sentence, verbatim |
/// | [`ReflowDecline::NotReflowable`] | [`ReflowRefusal::EngineDeclined`] | permanent for this document; the operator did nothing wrong and can do nothing |
/// | [`ReflowDecline::StructureForbids`] | [`ReflowRefusal::Other`] | reachable here only as a gate refusal, which this shell cannot describe more precisely than *"pdfcer could not, and nothing was changed"* |
///
/// Note what this shell refuses to do throughout: read
/// [`std::fmt::Display`]. The sentences are unchanged and are still
/// implementer-voiced. The engine's own doc says *"Match this, never `Display`
/// output"*, and that was this shell's position before the type existed.
fn reflow_refusal(error: &pdfcer_core::text_edit::ReflowApplyError) -> ReflowRefusal {
    use pdfcer_core::text_edit::ReflowApplyError as E;
    use pdfcer_core::text_edit::ReflowDecline as D;
    match error {
        // Named by variant because the engine names it by variant, and because
        // its remedy — remove the protection — is narrower than its decline.
        E::Encrypted => ReflowRefusal::Encrypted,
        // Named by **trigger**, not by variant, and the distinction is the
        // whole safety of this arm. `ReflowApplyError::Refused` is documented
        // as "the font-on-edit gate refused, by name (composite/CJK,
        // R-INV-4)" and `reflow_apply::refuse_if_composite` is its only
        // constructor today — but the payload is a general `encoding::Refusal`
        // carrying any of eight `RInvTrigger`s, and a match on the VARIANT
        // would silently start showing a composite-font sentence the first time
        // the engine refuses a reflow for some other encoding reason.
        //
        // So the guard reads the engine's own discriminant, and anything else
        // falls through to the funnel below and earns the honest general
        // sentence. This costs one comparison and removes an entire class of
        // "the sentence used to be true" defect. It is worth the arm because
        // the case is common: a drawing that sets its body text in a CIDFont
        // refuses every reflow, and `EngineDeclined`'s *"something about how
        // this page was drawn"* tells the operator nothing they can act on,
        // where *"the paragraph is set in a composite font"* does.
        E::Refused(refusal)
            if refusal.trigger == pdfcer_core::text_edit::RInvTrigger::Composite =>
        {
            ReflowRefusal::FontIsComposite
        }
        // **No wildcard.** See the header: a wildcard here swallows any
        // refusal the engine carves out of a broader one, on the day it ships.
        other => match other.decline() {
            // ⚠ Unreachable at engine `025d703d`, and mandatory anyway.
            // `ReflowDecline` is exhaustive on purpose, so this arm has
            // to exist; but `ReflowApplyError::PageEditedThisSession` is
            // the decline's sole source and the engine has no code that
            // constructs it, so nothing an operator does reaches this
            // line. Retained rather than `unreachable!()` — a panic here
            // would turn a future engine reinstating the guard into a
            // crash instead of a correct sentence, which is the wrong
            // direction for a refusal path.
            //
            // The chain is two links and **both** are watched, because a
            // marker on the error alone would miss an engine that gave
            // this decline a second, different producer:
            //
            // UNREACHABLE-FROM: pdfcer_core::text_edit::ReflowDecline::RetryAfterSaveAndReopen @ 025d703d
            //
            // (the other half sits on `ReflowRefusal::PageAlreadyEdited`'s
            // own definition, and names the error rather than the decline).
            D::RetryAfterSaveAndReopen => ReflowRefusal::PageAlreadyEdited,
            D::NotFound => ReflowRefusal::CannotTrace,
            D::NotReflowable => ReflowRefusal::EngineDeclined,
            D::StructureForbids => ReflowRefusal::Other,
        },
    }
}

#[cfg(test)]
mod tests;

/// **Re-wrap one paragraph to its own box.** `OPERATOR_REQUESTS.md` **O54**.
///
/// **The disclosures are the engine's, passed through verbatim.**
/// `ReflowApplyReport::disclosures` is already a `Vec<String>` written for an
/// operator, and it names the things this shell could not: how many lines the
/// paragraph had before and after, whether justification was applied, whether
/// the block now overflows the page. Re-wording them here would be a second
/// author for one fact — the rule this module's synthesis disclosure already
/// follows, and for the same reason.
///
/// # What reflow does to a multi-stream page
///
/// `reflow_block` re-emits the page's **first** `/Contents` object and its
/// commit sweep empties every other one. That is safe rather than lossy because
/// the plan is read from the **session's** graph and `ContentStream::from_page`
/// concatenates every `/Contents` entry, so a run appended this session is
/// inside the plan's source and is carried through verbatim. The collapse is
/// disclosed by the engine — *"multi-stream page: N additional /Contents
/// stream(s) were collapsed into the first"* — and that sentence reaches the
/// status line unchanged, which is the whole of what the operator is owed
/// about it.
///
/// ⇒ So reflow **accumulates with the other text verbs**, exactly as they
/// accumulate with each other. `super::text`'s header states that as a property
/// of the program and its test re-measures it against the pinned engine, which
/// is the only honest way to assert anything about a branch dependency.
///
/// # Nothing here forecasts a refusal
///
/// Every engine-side refusal is worded **after** the attempt, by
/// [`reflow_refusal`], from the engine's own discriminant. A shell-side
/// pre-flight would be a second implementation of the engine's predicate, in a
/// second crate, over the same `/Contents` list — two predicates over one
/// model, each self-consistent, and no test of either able to see them
/// disagree. The engine owns the question; the shell words the answer.
///
/// ⚠ The refusal a reader will actually meet on a CAD sheet is
/// [`ReflowRefusal::FontIsComposite`]: a paragraph set in a Type 0 / CIDFont is
/// refused by name, which is an engine feature not yet built rather than a
/// guard.
pub(super) fn reflow(doc: &mut OpenDoc, page: usize, block: usize) {
    // ⚠ **There is no pre-flight gate here, and adding one is a mistake.**
    //
    // The tempting one refuses reflow whenever the document has been edited, or
    // whenever *this* page carries a second non-empty `/Contents` stream. Both
    // are wrong, for different reasons, and both have to stay refused:
    //
    // * The wide form costs the operator reflow on every page touched this
    //   session, including the ones that were always safe. A guard whose reason
    //   nobody remembers reads as a limitation of the product.
    // * The narrow form is a **second implementation of the engine's own
    //   predicate**, in a second crate, over the same `/Contents` list — two
    //   predicates over one model, each self-consistent, with no test of either
    //   able to see them disagree.
    //
    // A structural test is also the wrong instrument for the question it looks
    // like it answers: a producer that splits page content across streams
    // (SOLIDWORKS does — one drawing's page 0 carries eight) is indistinguishable
    // from a page edited this session, so the predicate fires on the first frame
    // after `File > Open`.
    //
    // ⇒ What stands in its place is **nothing**. The call below routes the
    // engine's refusal through `record_reflow`, so the operator meets a specific
    // sentence rather than a blanket one, and the sentence stays the engine's.
    //
    // **The cropbox is supplied, and supplying it is the whole of the overflow
    // disclosure.**
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
        // A page index this document does not have. The reflow below will
        // refuse it by name; declining to guess a cropbox is what keeps the
        // refusal ABOUT the missing page rather than about a rectangle this
        // shell invented for it.
        None => request,
    };
    super::apply::vector_edit(doc, "reflow-block", page, 1, |session| {
        session
            .reflow_block(page, block, &request)
            // **The engine's own refusal, worded** — O127, defect 3.
            //
            // `funnel::vector_edit`'s error arm traces `detail={error}` and
            // shows `Declined::EditRefused`: *"That change was refused, and the
            // document is unchanged."* Nine words, no cause, no remedy — and
            // reflow has four engine-side causes with four different next
            // steps. An operator told only that something was refused has been
            // given the fact he already had.
            //
            // `inspect_err` rather than a `match` around the funnel, and the
            // ordering is what makes it work: `vector_edit` takes the decline
            // floor *before* running this closure, so the slot is empty when
            // this line writes to it — and `BeforeTheVerb::refused` only fills
            // the slot `if slot.is_none()`, so this specific sentence survives
            // and the generic one stands aside. That mechanism is `floor`'s
            // documented purpose (*"unless the verb already said something
            // better"*); this is the first verb to use it.
            .inspect_err(|error| {
                crate::app::status::decline::record_reflow(reflow_refusal(error));
            })
            .map(|report| {
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
                // The line count, added because the engine's own list does
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
