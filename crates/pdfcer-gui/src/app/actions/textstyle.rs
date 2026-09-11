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
//! ⚠ ~~So [`apply`] asks for synthesis first and, when the engine refuses
//! **because a real face is available**, retries with that face — which the
//! refusal names (`RealFaceAvailable { real_font, .. }`).~~ **That is how this
//! module worked until 2026-09-11; the walk now belongs to the engine.** See
//! *"the ladder is the engine's"* below. The OUTCOME the struck sentence
//! describes is unchanged and better: the operator presses one button and gets
//! the best weight the page can give — a genuine typeface where one exists, a
//! disclosed synthetic one where it does not.
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
//! ⚠ **SUPERSEDED — the paragraph above describes 2026-08-27 and was fixed on
//! 2026-08-28.** It is kept because it is what the engine request was argued
//! from, and because the diagnosis in it is the one the engine adopted.
//!
//! Filed as `request_gate_synthesis_names_a_face_that_cannot_cover_the_run.md`,
//! confirmed and reproduced by the engine the same day, and queued ahead of
//! their print-conformance work by their operator's instruction. The fix is the
//! one this project's diagnosis asked for: `gate_synthesis` treats a real face
//! as available only if `set_font` would accept it **for this run**.
//!
//! ⇒ **Shipped as `Pass 144.0` (`cfa2c44`) and consumed by this file the same
//! day.** ~~See the `RealFaceAvailable` arm below~~ — that arm was deleted on
//! 2026-09-11 with the hand-rolled retry; the fix now reaches this shell
//! through `set_style`, whose rung 1 inherits it (`find_styled_face` filters on
//! `accepted.is_ok()`, so a coverage-refusing same-family face is SKIPPED
//! rather than offered and then refused). So *"Bold is
//! unreachable there through either route"* has been false since 2026-08-28,
//! and the sentence before it was written in the future tense about work that
//! had already landed.
//!
//! ★ Found by the 2026-09-07 reply triage rather than by any gate: the file
//! that consumed the fix went on describing the defect 350 lines above the arm
//! that fixed it. **A module header is the part of a file least likely to be
//! re-read by whoever changes its body.**
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
//! ⚠ **The CONCLUSION above stands; the REASON given for it stopped being true
//! on 2026-09-09.** `EditSession::run_repertoire` — `Pass 280.0`, built in
//! answer to this project's own request — *is* a per-run glyph-coverage test,
//! it runs in one call, and `canvas::textedit::repertoire` calls it on every
//! caret landing. So *"a per-run glyph-coverage test it cannot run"* is a
//! sentence about a build this shell no longer is.
//!
//! ⇒ What it answers, though, is *which characters the run's **current** font
//! accepts* — not *would `set_font` accept **Calibri-Bold** for this run*, which
//! is the question greying these two buttons would need answered. That question
//! still has only one instrument, `preview_font_resources_for`, and the engine's
//! own rustdoc says what it costs: it *"answers by walking every operation in
//! the page's content stream"*, which on the operator's benchmark sheet is a
//! walk over 129,758 objects. Per frame, per button, that is not a greying rule
//! — it is a hang.
//!
//! ★ So the paragraph is corrected rather than deleted, because the two halves
//! fail differently and a later reader deciding whether to grey these buttons
//! needs to know which half he is up against. Left as written it would have
//! read as *nothing can be measured here*, and the first person to check would
//! have found a verb that measures exactly that and concluded the header was
//! simply out of date — which is how a correct conclusion loses its argument.
//! **A limitation sentence is a citation with an hours-long shelf life**, and
//! this file has now been corrected on that ground twice.
//!
//! ## ★★★ 2026-09-11 — the ladder is the ENGINE's, and rung 2 is the whole point
//!
//! Everything above describes a walk this module performed by hand: ask for
//! synthesis under a pinned `Refuse`, take the real face the refusal names, and
//! fake it when that face cannot cover the run. Three rungs, correct, and
//! **missing the one that matters most on the operator's own drawings.**
//!
//! [`FormatRequest::set_style`] (`Pass 179.0`, engine `71d13aa`, 2026-08-30,
//! shipped 2026-09-06) does the walk inside the engine, with four rungs:
//!
//! | rung | what it binds | reported as |
//! |---|---|---|
//! | 1 | a real face already on the page that CLAIMS the style and passes the coverage gate — same family first, then any family | `StyleRung::RealFaceOnPage` |
//! | 2 | the **standard-14 sibling of the run's own family**, as a new `/Font` resource, nothing embedded | `StyleRung::StandardFourteenSibling` |
//! | 3 | a `--font-dir` donor (`Pass 142.0`) | not built |
//! | 4 | synthesis — the stroke or the shear | `StyleRung::Synthetic` |
//!
//! ### ★★ Rung 2 is what this shell was missing, and it fires constantly
//!
//! A CAD title block exported with `Helvetica` and nothing else — pdfcer's own
//! `rotated-text.pdf`, and most of the operator's working set — carries **no
//! bold font resource at all**. `gate_synthesis`'s first branch reads *"No font
//! resources to search: nothing better exists, so the fallback is genuinely the
//! only option. Proceed."* That is true of the page and false of the world:
//! `Helvetica-Bold` is one of the fourteen faces every conforming reader is
//! **required** to carry, needs no font file (ISO 32000-1 §9.6.2.2), and costs
//! about sixty bytes to name.
//!
//! ⇒ So for five days after the engine shipped the rung, pressing Bold on the
//! commonest page in the operator's working set thickened the strokes of
//! `Helvetica` when it could have bound `Helvetica-Bold`. Nothing failed. No
//! gate was red. The difference is visible at 1:1 on a plotter and invisible in
//! a test that asserts the epoch moved.
//!
//! ### ★ What was deleted, and why each deletion is safe
//!
//! * **The `Refuse`-pinned probe.** It was how this shell ASKED which real face
//!   was available — *"the refusal is never shown to the operator, because the
//!   very next thing that happens is taking the offer it names."* The ladder
//!   answers by binding, so the question is gone and [`FormatOptions`] now
//!   carries the operator's real posture.
//! * **The `preview_style_resolution` pre-check.** It raised
//!   `TextStyleRefusal::FakingDeclined` because the engine's `set_synthetic`
//!   gate refuses only when a real face was passed over, and an operator who
//!   ticked *"never fake it"* meant something wider. `set_style`'s posture gate
//!   fires at **rung 4** — after both real rungs have been tried — and returns
//!   [`FormatError::SynthesisRefusedByPosture`]. Same variant, decided by the
//!   side that knows what it tried, and reached less often.
//! * **The entire `RealFaceAvailable` arm and its nested `Auto` fake-retry.**
//!   That error is returned only for the **explicit** `set_synthetic` verb under
//!   `Refuse`. This module never sets that verb again, so the arm was
//!   unreachable code with a plausible comment on it.
//!
//! ★★ **The `selector`-not-`real_font` finding is kept here rather than lost
//! with the arm that held it**, because it is about the engine's contract and
//! not about the deleted code: two `/Font` resources can share one `/BaseFont`,
//! a retry built from the NAME reaches only one of the twins — *"possibly the
//! twin that refuses"* — and `selector` is defined as the string that reaches
//! the face pdfcer actually checked. Filed by the engine in `Pass 144.0`'s reply
//! under "ACT ON THIS", and this shell kept using `real_font` for a day
//! afterwards. **A reply read is not a reply consumed**, which is also the
//! sentence that explains the five-day gap this section closes.
//!
//! ## ★★★ 2026-09-11, LATER THE SAME DAY — the five gaps consuming it opened
//! are all shut
//!
//! Adopting the ladder cost one sentence and one working instrument, and left
//! three smaller holes. All five were filed the same afternoon, one topic per
//! file; `Pass 295.0` (engine `2752b72`) answered all five in one Pass, and
//! **every workaround below is deleted rather than left dormant** — a
//! mechanism with no caller rots, and the next session cannot tell a live
//! workaround from a dead one.
//!
//! ### ★★ The sentence that was lost, and is back
//!
//! `text_style_used_other_family` said *"the letterforms will look different,
//! not just heavier or slanted"* — the one substitution an operator can SEE on
//! a plot. Its input was `RealFaceAvailable { same_family, .. }`, an error the
//! ladder never produces, and [`StyleLadder`] as first shipped carried no
//! `same_family` of its own. `family_stem` is private and engine invariant R74
//! forbids this shell re-deriving it, so for one afternoon [`ladder_note`]
//! named **both** `/BaseFont`s out of `FormatReport::font_change` — entitled,
//! unguessed, and weaker.
//!
//! [`StyleLadder::same_family`] is `Option<bool>`, `None` meaning *nothing was
//! bound* and explicitly **not** to be flattened into `Some(false)`. Rung 1 is
//! now two sentences keyed on it, and nothing in this module reads
//! `font_change` any more.
//!
//! ### ★★★ The hover hints predicted the wrong thing, and now predict the right
//! one
//!
//! `panels::properties::text`'s `bold_hint` / `italic_hint` were built from
//! `StyleOutlook`, which came from `preview_style_resolution` — **a preview of
//! the R90 GATE, not of the LADDER**. `StyleOutcome::WouldSynthesize` means
//! only *"no real face on this page claims that style and covers this run"*,
//! which stopped being the same question as *"what will pressing Bold do?"* the
//! moment rung 2 existed: the standard-14 sibling is by construction **not on
//! the page**, so the gate cannot see it. The tooltip promised thickening on
//! every page where a real `Helvetica-Bold` was about to be bound, and the
//! status line then reported the real face — two instruments disagreeing by
//! construction, on a CAD title block.
//!
//! `EditSession::preview_style_ladder` runs **the same planner `format_text`
//! runs**, walks the page's content once, stages nothing, and takes the
//! [`FormatOptions`] the commit will use — so under `Refuse` it previews the
//! refusal rather than predicting a synthesis that would never happen. The
//! hover now names the rung, and the softened sentences that stood in for it
//! are gone.
//!
//! ⇒ **The `preview_font_resources` join went with them.** Telling "a real face
//! will be used" from "a real face will be refused for this run" used to take a
//! string equality between two engine-issued selectors, evaluated against the
//! run's own font pre-flight, in an order that was load-bearing and
//! commented as such. The ladder answers directly, so the join, the ordering
//! constraint and `StyleOutlook::FaceCannotCover` are all deleted — the last
//! of those because **its subject is gone**: a face that claims the style and
//! cannot cover the run is no longer an outcome, it is an entry in
//! `passed_over` on the way to a rung that works.
//!
//! ### ★★ `passed_over` cost a FEATURE, not a workaround — and the feature is
//! written now
//!
//! It was `Vec<String>` of `"BaseFont (reason)"`, so saying *"pdfcer tried
//! `Times-Bold` and it has no `o`"* in this shell's own voice would have meant
//! splitting on `" ("` — a locator for the engine's message format, living in
//! a GUI, breaking silently the first time a reason gained a parenthesis. A
//! shell disciplined about not re-deriving engine facts keeps quiet instead, so
//! **the sentence was never written at all.**
//!
//! `PassedOver { base_font, reason, refusal }` is structured, and
//! `Refusal::character` gives the offending character. The sentence went to the
//! **hover**, not to the status line: `FormatReport::disclosures` already
//! carries the engine's own passed-over clause verbatim one line below
//! [`ladder_note`]'s, and a disclosure repeated is a disclosure skipped. Before
//! the press, nothing said it at all.
//!
//! ### ★ Two smaller ones
//!
//! * **[`FormatError::CoverageFailure`] was unconstructable by a consumer**
//!   (`E0639` — `Refusal` is `#[non_exhaustive]` with a private constructor),
//!   so the arm this module most needs to defend was the one its walk-every-
//!   variant test could not cover. `Refusal::new` is public; the case is in the
//!   test.
//! * **[`FormatError::SynthesisRefusedByPosture`]'s `Display` ran two words
//!   together** — `page facesHelvetica-Bold` — and read as *"X was used"* where
//!   it meant *"X was tried and rejected"*. The field carries a whole clause
//!   now (`thiserror`'s format string cannot branch), so it reads *"rung 1: page
//!   faces Times-Bold could not show the run"* or *"rung 1: no page face claims
//!   it"*. Trace-only here; `pdfcer-cli` surfaces it verbatim.
//!
//! [`FormatRequest::set_style`]: pdfcer_core::text_edit::FormatRequest::set_style
//! [`FormatError::SynthesisRefusedByPosture`]: pdfcer_core::text_edit::FormatError::SynthesisRefusedByPosture
//! [`FormatError::CoverageFailure`]: pdfcer_core::text_edit::FormatError::CoverageFailure
//! [`StyleLadder`]: pdfcer_core::text_edit::StyleLadder
//! [`StyleLadder::same_family`]: pdfcer_core::text_edit::StyleLadder::same_family
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
    /// ★ Deliberately **not** named `Synthetic`, because whether it ends up
    /// synthetic is the engine's decision and not the operator's. That was an
    /// aspiration when the name was chosen — the decision was really this
    /// module's, taken in a hand-rolled retry — and since 2026-09-11 it is
    /// literally true: the variant maps to `set_style`, and the engine walks
    /// its four-rung ladder. The operator asked for bold; how bold is achieved
    /// on this page is a fact they are told afterwards.
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
            // ★★★ `style`, NOT `synthetic` — changed 2026-09-11, and the one
            // word is the whole feature.
            //
            // `set_synthetic` means *"thicken the strokes"* and is gated;
            // `set_style` means *"make this bold"* and walks the ladder. The
            // difference the operator sees is a real `Helvetica-Bold` instead
            // of a stroked `Helvetica` on every page that carries no bold
            // resource, which is most CAD title blocks. Module header.
            //
            // ⚠ The two are MUTUALLY EXCLUSIVE per axis: combining `set_style`
            // with `set_font`, or with an overlapping `set_synthetic`, is
            // `FormatError::Unsupported`. Nothing else in this table sets
            // either, and `Face` is its own variant, so one press is one verb.
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
                // ★★★ THE OPERATOR'S POSTURE, PASSED STRAIGHT THROUGH — and until
                // 2026-09-11 it was pinned to `Refuse` here on purpose.
                //
                // The old comment, kept because the reasoning was sound and the
                // mechanism is what changed: *"the probe pins `Refuse`
                // unconditionally. The refusal is how this shell ASKS which real
                // face is available; it is never shown to the operator as a
                // refusal, because the very next thing that happens is taking
                // the offer it names."* Three rungs, hand-rolled, and one of
                // them — the standard-14 sibling — unreachable.
                //
                // ★★ `set_style` walks all four rungs itself, so there is no
                // question left to ask by provoking a refusal, and the posture
                // goes where it belongs: to the engine, as the operator set it.
                // Under `Auto` the ladder decides and discloses; under `Warn` it
                // does the same and [`ladder_note`] raises the synthetic case to
                // a sentence of its own; under `Refuse` the ladder's **fourth**
                // rung returns `SynthesisRefusedByPosture`, which is the wide
                // reading this shell used to have to construct for itself.
                //
                // ★ Module header for what was deleted and why each deletion is
                // safe. This is one call where there were up to three.
                let options = FormatOptions::default().with_style_policy(policy);

                match session.format_text(&change.stamp(request(page, op.pin)), &options) {
                    Ok(report) => {
                        // ★ The ladder's own sentence FIRST, then the engine's
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

/// The one sentence the style ladder earns, or `None` to let the engine speak.
///
/// # ★★★ One sentence per RUNG, and never two about one outcome
///
/// [`StyleLadder::rung`] is the engine's own account of how it made the text
/// bold, and it is a closed question with one answer — so this is a mapping,
/// not an accumulation. The hand-rolled dance it replaces pushed a sentence per
/// *branch it took*, which is how a `Warn`-posture synthesis could produce both
/// *"pdfcer had to fake this"* and *"no real face was available"* about the same
/// event.
///
/// ★★ The `Warn` and default synthetic sentences are therefore **mutually
/// exclusive**, not additive. `Warn` means the operator asked to be told
/// prominently; they are told once, in the catalog's `Warn` words. Adding the
/// ordinary sentence underneath would be the same fact twice, and a disclosure
/// repeated is a disclosure skipped.
///
/// # ★★ What returns `None`, and why silence is the right answer there
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
///   rung 3 (`--font-dir` donors, `Pass 142.0`) is not built yet and a fifth is
///   possible. An invented sentence for an outcome nobody here has seen is
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
/// # ★★★ Rung 1 is TWO sentences, keyed on `same_family`
///
/// Between 2026-08-30 and 2026-09-11 this arm read `FormatReport::font_change`
/// and named **both** `/BaseFont`s — *"was set in Calibri and is now set in
/// Times-Bold"* — because [`StyleLadder`] as first shipped carried no
/// `same_family` flag and engine invariant R74 forbids `pdfcer-gui`
/// re-deriving `family_stem` to work one out. That was a workaround: correct,
/// stated as a fact rather than a guess, and worse prose than the thing it
/// stood in for.
///
/// `Pass 295.0` shipped [`StyleLadder::same_family`] and the engine's reply
/// said in as many words that the workaround was the right call and can now be
/// dropped. It is dropped. **Nothing in this function reads `font_change` any
/// more**, and the two sentences are:
///
/// * `Some(true)` — the page carried the bold or italic form of this text's
///   own typeface. The letterforms are unchanged, and the sentence says so,
///   because that is the outcome the operator needs no warning about.
/// * `Some(false)` — the only real face that could show the run belonged to
///   another typeface. The letterforms will look different, not just heavier,
///   and on a plot that is visible where a status line is not.
///
/// ★★ `None` means **nothing was bound**, and it must not be flattened into
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
            // ★ A rung that bound a face and reports no family verdict would be
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
        // ★★ Mutually exclusive with the `Warn` sentence, never both. See above.
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
/// ★ Its own function so every `None` arm above costs one readable line, and so
/// the trace format is written once. `StyleRung` has a `Display` that spells the
/// rung in words (*"rung 2: the standard-14 sibling"*), which is what a future
/// session grepping a trace for an unhandled outcome needs to see.
fn trace_rung(ladder: &StyleLadder, why: &'static str) {
    let rung = ladder.rung.to_string();
    let bound = ladder.bound.as_deref().unwrap_or("-").to_owned();
    let requested = ladder.requested.axes();
    let synthesised = ladder.synthesised.axes();
    // ★★ Spelled by hand, NOT `{:?}` over the `Option<bool>`. A machine-read
    // field rendered through `Debug` is this project's standing defect: a
    // driven check keying on `family=false` would match `family=Some(false)`
    // and `family=None` alike once somebody widened the pattern, and would
    // then report the opposite of the truth while quoting the truth.
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
/// ★ The engine's own `Display` prose is deliberately **not** the sentence.
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
/// # ★★★ The wildcard is why the fourth case had to be added by hand
///
/// `FormatError` is `#[non_exhaustive]` and this `match` ends in `_`, so the
/// compiler cannot tell anyone that a new variant arrived — it lands in
/// `Other`, gets the generic sentence, and nothing anywhere goes red. The rule
/// is stated in `text::textedit::reflow_refusal`'s header and it is exactly
/// this shape: *"any `match` of yours ending in `_` just gained a variant it
/// will not distinguish, and the one it will not distinguish is the one you
/// care about."*
///
/// ★★ Wiring `set_style` on 2026-09-11 made
/// [`FormatError::SynthesisRefusedByPosture`] reachable for the first time, and
/// it is the single most important refusal this surface can produce: it fires
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
        // ★★★ The ONLY arm that carries a payload out of the engine's error,
        // and the reason is that this is the only refusal here an operator can
        // act on without guessing. `Refusal::remedy_faces` is the faces that
        // WOULD show this run — the same list the engine's own message names
        // in prose, structured (`Pass 296.1`, requested and shipped
        // 2026-09-11). Cloned rather than borrowed because the sentence
        // outlives the error: it is recorded in the decline slot and read by
        // the bar on later frames.
        //
        // ★ Before this, the shell showed nothing. Reaching the list meant
        // splitting `Refusal::message` on a clause, and the public helper that
        // looks like the answer is the naive one the engine measured as WRONG
        // on the very fixture this refusal exists for. Filed rather than
        // worked around; see the variant's docs.
        FormatError::CoverageFailure(refusal) => {
            t::TextStyleRefusal::FaceLacksCharacters(refusal.remedy_faces.clone())
        }
        // ★ The operator's own setting, reported back as their setting. This
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
/// # ★★★ The same shape as [`text_style_refusal`] above, and for the same rule
///
/// The engine's error is a *diagnosis*; the operator needs a *next step*, and
/// the two are not the same list. `ReflowApplyError` has ten variants and this
/// maps them onto the four things an operator can do about them — save and
/// reopen, remove the protection, accept that this text cannot be addressed, or
/// stop. Wording each engine variant separately would put ten operator-facing
/// decisions in an error type that was written for a library.
///
/// # ★★★ Switched onto `ReflowDecline` the same day, 2026-09-07
///
/// This function has been wrong twice in two days and the second correction is
/// the one worth reading, because **it removes the shape that caused both.**
///
/// **Correction 1 (earlier today).** It read
/// `E::Unsupported(_) => ReflowRefusal::PageSetChanged`, which told the
/// operator *"pages have been added, removed or reordered since. Save this file
/// and open it again."* `Pass 257.0` had deleted both page-set refusals the day
/// before, so that named a cause the engine could no longer produce — for all
/// **ten** sentences `Unsupported(String)` then carried, exactly one of which
/// (`Pass 251.0`'s data-loss guard) actually had *save and reopen* as its
/// remedy. It was replaced with [`ReflowRefusal::EngineDeclined`], which names
/// no cause and promises no remedy, and the discriminant was **filed**.
///
/// **Correction 2 (hours later).** The engine shipped it — `ReflowDecline`,
/// `ReflowApplyError::decline()`, `is_recoverable()`, and a named
/// `PageEditedThisSession` carved out of `Unsupported` with its sentence
/// byte-identical.
///
/// ⚠⚠⚠ **And the reply flagged a trap in this very function, which is why it
/// is worth quoting:**
///
/// > *"any `match` of yours ending in `_` just gained a variant it will not
/// > distinguish, and the one it will not distinguish is the one you care
/// > about."*
///
/// Exactly right. `ReflowApplyError` is `#[non_exhaustive]`, so adding
/// `PageEditedThisSession` produced **no compile error here** — the new variant
/// would have fallen into `_ => ReflowRefusal::Other` and the one recoverable
/// refusal in the whole set would have lost its remedy for the second time in
/// one day, silently. A capability arriving is not a capability landing, and a
/// wildcard is how the difference stays invisible.
///
/// ## So the wildcard is gone, and that is the actual repair
///
/// Everything except [`ReflowApplyError::Encrypted`] now routes through
/// [`ReflowApplyError::decline`]. [`ReflowDecline`] is deliberately **not**
/// `#[non_exhaustive]` — the engine made the same written promise it made for
/// `RefusalKind` — so the `match` below is **compiler-proved complete**. A
/// future engine refusal joins an existing arm and keeps its correct sentence;
/// a future *decline* is a build failure here, which is the one place it should
/// be. This shell can no longer silently mis-word a refusal it has not met.
///
/// ★ `Encrypted` is matched by variant, above the discriminant and on purpose.
/// `ReflowDecline::StructureForbids` covers *"encryption, **or** a save that the
/// edit gates refused"*, and those need different sentences — one says *remove
/// the protection*, the other cannot say anything so specific. Naming the
/// narrower cause where the engine gives a narrower variant is the whole
/// lesson of correction 1, applied in the other direction.
///
/// | decline | shell refusal | why |
/// |---|---|---|
/// | [`ReflowDecline::RetryAfterSaveAndReopen`] | [`ReflowRefusal::PageAlreadyEdited`] | the one refusal the operator can act on |
/// | [`ReflowDecline::NotFound`] | [`ReflowRefusal::CannotTrace`] | *"glyphs cannot be traced back to their show operators"* is that sentence, verbatim |
/// | [`ReflowDecline::NotReflowable`] | [`ReflowRefusal::EngineDeclined`] | permanent for this document; the operator did nothing wrong and can do nothing |
/// | [`ReflowDecline::StructureForbids`] | [`ReflowRefusal::Other`] | reachable here only as a gate refusal, which this shell cannot describe more precisely than *"pdfcer could not, and nothing was changed"* |
///
/// ★★ Note what this shell still refuses to do: read
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
        // ★★★ NO WILDCARD. See the header: the wildcard that used to be here
        // would have swallowed `PageEditedThisSession` on the day it shipped.
        other => match other.decline() {
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
    // ★★★ **THE OVER-BROAD FORECAST IS GONE — deleted 2026-09-05 when the engine
    // closed the defect it existed to hide from.**
    //
    // Until today this refused reflow whenever `doc.edit_epoch != 0`, i.e. after
    // **any** edit to **anything** in the document. The comment that stood here
    // said so, called itself over-broad, and defended it — correctly at the
    // time:
    //
    // > `EditSession::add_text` appends a NEW content stream to the page's
    // > `/Contents` and never touches the first one, so it does not trip the
    // > engine's guard. `reflow_block` then plans from the BASE document …
    // > and **empties every other `/Contents` entry**. The added text is in one
    // > of those entries. ⇒ A reflow permitted after an add-text would silently
    // > delete text the operator can see on the page.
    //
    // That was true, it was filed, and the engine has answered it. `Pass 251.0`
    // (`edit.rs:9599`) refuses a page carrying a non-empty appended stream **by
    // name**, in a sentence naming the remedy — *"text was added to this page
    // this session (in a new content stream); reflow is planned against the base
    // content and would drop the added run, so save and reopen before reflowing
    // this page"*. Their reply says it in as many words: *"that is the option
    // you offered in §7, so you can drop your over-broad forecast gate."*
    //
    // ⇒ **A workaround kept past its cause rots, and this one was not inert
    // while it rotted.** It cost the operator reflow on every page he had
    // touched this session, including the ones that were always safe — and a
    // guard whose reason nobody remembers reads as a limitation of the product.
    //
    // ★★ Deleted rather than narrowed. Narrowing it — asking here whether *this*
    // page carries a non-empty appended stream — would be a **second
    // implementation of the engine's own predicate**, in a second crate, over
    // the same `/Contents` list. That is the shape that made an invisible
    // annotation clickable earlier today: two predicates over one model, each
    // self-consistent, and no test of either able to see them disagree. The
    // engine owns the question; the shell words the answer.
    //
    // ⚠ What replaces it is **nothing** — deliberately. The call below already
    // routes the engine's refusal through `record_reflow`, so the operator now
    // meets the specific sentence instead of the blanket one, and gets it only
    // when it applies.
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
        session
            .reflow_block(page, block, &request)
            // ★★★ **The engine's own refusal, worded** — O127, defect 3.
            //
            // `funnel::vector_edit`'s error arm traces `detail={error}` and
            // shows `Declined::EditRefused`: *"That change was refused, and the
            // document is unchanged."* Nine words, no cause, no remedy — and
            // reflow has four engine-side causes with four different next
            // steps. An operator told only that something was refused has been
            // given the fact he already had.
            //
            // ★★ `inspect_err` rather than a `match` around the funnel, and the
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
