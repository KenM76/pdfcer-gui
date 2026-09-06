//! # `text::editrefusal` — **why a text edit the operator committed did not
//! happen**, in his words
//!
//! `OPERATOR_REQUESTS.md` **O140**, **O141** and **O142**. One enum, one
//! classification function, one sentence per cause — and the whole of what this
//! shell says when `EditSession::edit_text` comes back `Err`.
//!
//! ## Why it is its own file
//!
//! **R2.** [`super::textedit`] crossed 1,500 lines on 2026-09-06, when O142's
//! ambiguity refusal and `Pass 256.1`'s ambiguous-character refusal arrived
//! together. The seam was already drawn and already labelled — that file carried
//! a banner reading *"Why an edit the operator committed did not happen — O140"*
//! — and the two subjects either side of it are genuinely different questions:
//!
//! * above it, **what an edit COSTS**: the pinned-tail disclosure, the
//!   multi-run note, the reflow refusals. Sentences about an edit that
//!   happened, or about a re-wrap that was declined before any verb ran.
//! * here, **why a commit was REFUSED**: the engine answered `Err`, and this is
//!   the joining of its coarse `RefusalKind` with the facts only the shell
//!   holds.
//!
//! ★ Every item is re-exported from [`super::textedit`], so no call site moved
//! and nothing outside this pair needs to know the split happened.
//!
//! ## ★★★ The rule that governs every arm in here
//!
//! **The engine's category wins wherever it has one, and a shell-side fact is
//! allowed to sharpen exactly the bucket where the engine's answer is true and
//! unusable at the same time.** Three shell-side facts are consulted, each in
//! one bucket only:
//!
//! | fact | bucket | what it separates |
//! |---|---|---|
//! | `character` ([`RefusedCharacter`]) | `UnsupportedFont` | a font that cannot spell the letter, from one that spells it two ways |
//! | `occurrences` | `NotFound` | a page holding the words twice, from a run written one glyph at a time |
//! | `one_operator` | `NotFound` | a run written in pieces, from a page that moved under the caret |
//!
//! ⚠ **Nothing here greps a `Display` string or keys on a trigger id to
//! reconstruct a category.** `RefusedCharacter` reads one *field* of a
//! structured refusal, on the same licence the character itself is read on, and
//! its constructor's doc comment carries that argument in full. Building a
//! second copy of the engine's taxonomy is what `RefusalKind`'s own header
//! exists to forbid, and the cost of getting it wrong is named there: telling
//! the operator the WRONG reason, which is strictly worse than the silence it
//! replaced.

// ===========================================================================
// ★★★ Why an edit the operator committed did not happen — O140
// ===========================================================================

/// ★★★ **Why a committed text edit was refused** — `OPERATOR_REQUESTS.md`
/// **O140**, and the sentence that replaces `edit_declined_by_engine`'s nine
/// cause-free words for this one verb.
///
/// The operator, 2026-09-05, on his own quote:
///
/// > *"on page 2 there is a spelling mistake — clien instead of client. if I
/// > try to edit the edit is not accepted. **the lines I added below `price)`
/// > are editable, but everything else that existed when I got the pdf is
/// > not.**"*
///
/// He met a caret that took his keystrokes, a Ctrl+Enter that did nothing, and
/// a status line reading *"That change was refused, and the document is
/// unchanged."* — which is true, and tells him nothing he could act on. His own
/// second sentence is a better diagnosis than anything the program gave him.
///
/// # ★★★ Where the categories come from, and why not from here
///
/// `crate::text::status::edit_declined_by_engine`'s documentation says, in as
/// many words, that it *"is written to be **deleted**… The day `EditError`
/// gains a `kind()`, this becomes four sentences that name the four buckets."*
///
/// **That day arrived and nothing here had noticed.** `pdfcer-core` shipped
/// `text_edit::RefusalKind` — four variants, deliberately **not**
/// `#[non_exhaustive]` so a front end may match it exhaustively and have the
/// compiler prove the sentences are complete — at revision `b1033ab`, in direct
/// answer to this project's 2026-09-04 request. It sat unconsumed because both
/// gates that watch the engine are keyed on `EditSession`'s **verbs**: a new
/// *type* is invisible to `check-verb-coverage.sh` and to
/// `check-engine-backlog.sh` alike.
///
/// ⇒ Four of the five variants below are `RefusalKind`'s, one-for-one. Nothing
/// here re-derives the engine's reasoning, matches on its error variants or
/// greps its `Display` prose — the three things that entry forbids. The mapping
/// lives beside the errors, in their crate, and moves with them.
///
/// # ★★★ The fifth variant, and why the shell is entitled to it
///
/// [`Self::SplitAcrossPieces`] is **not** a `RefusalKind`. It is a fact this
/// shell measured before it asked, and it is the one his document actually
/// hits.
///
/// `canvas::textedit::plan` calls `pin::spans_one_operator` and traces the
/// answer as `edit-text-pin … one_operator=…`. On his file that is **false**:
/// the producer emitted **one show operator per glyph** — the page-2 content
/// stream is a run of `(\x00\x17) Tj  16.07 0 Td  (\x00\x11) Tj  8.03 0 Td …`,
/// one two-byte code and its own `Td` per letter — so no single operator holds
/// more than one character. The shell therefore sends the reconstructed `find`
/// rather than the whole-operator form (see `plan`'s own note: the
/// whole-operator form on a split run would replace one fragment's text with
/// the whole replacement and leave the others painting their old glyphs —
/// *visible corruption reported as success*), and a 36-character `find` cannot
/// match inside a one-character operator. The engine answers
/// `EditError::NoMatch`, whose `RefusalKind` is `RefusalKind::NotFound`.
///
/// ★★ **`NotFound` is the honest engine answer and the wrong operator
/// sentence.** *"pdfcer couldn't find what the edit named"* is
/// indistinguishable from *"your search string is wrong"* — and he did not type
/// a search string; he clicked a word he can see. The shell knows the piece of
/// information that turns that answer into a true one, and no other layer does:
/// **it placed the caret on that run itself, from its own extraction, and it
/// knows the run is split.** So the join is made here, at the only place both
/// facts exist.
///
/// # ★★★ What this variant is NOT, and the forecast that was falsified
///
/// The feature request filed against the engine on the same day names the cause
/// as `Identity-H` — *"the document's original faces are `Type0/CIDFontType2`
/// `Identity-H`, `verdict=blocked-identity`… the only character information is
/// a one-way `/ToUnicode`"* — and asks for that map to be inverted.
///
/// **Measured, and it is not the cause.** `Pass 29.0` made composite runs
/// editable whenever their `/ToUnicode` inverts, and on his own file, with the
/// engine's own CLI:
///
/// ```text
/// pdfcer edit-text --page 2 --find "n" --replace "t"
///   → base_font=AAAAAA+Arimo-Bold  …  OK
/// ```
///
/// — an `Identity-H`, `blocked-identity`, embedded `CIDFontType2` face, edited
/// end to end, read back out of the saved file. A **one-character** find
/// matches inside a one-character operator; a five-character one cannot. The
/// font was never the obstacle.
///
/// ⇒ **So this shell must not forecast on the font.** A guard reading
/// *"`Identity-H` ⇒ refuse"* would withhold the caret on text pdfcer can edit,
/// on every document a modern producer makes, silently — which is
/// `Refusal::InsideForm`'s episode repeated with a different predicate. The
/// falsifying fixture is `pdfcer-core`'s own
/// `fixtures/synthetic/text/composite-editable.pdf`: identical `list-fonts`
/// verdict to every face in his document, and
/// `an_invertible_composite_run_is_editable_end_to_end` asserts it edits.
///
/// # ★★ Why the caret is still OFFERED, against R9's first reading
///
/// R9 says a control that fails on press is worse than one that is not there,
/// and the obvious application is *"do not put a caret on a split run"*. It is
/// the wrong application here, for a reason that is a property of the caret
/// rather than of this refusal: **the caret anchors more than the replace.**
/// `edit.reflow_block` resolves its operand from `Anchor::Run`, and
/// `format_text` restyles through the same pin without ever needing a `find` —
/// both work on a split run. Withdrawing the anchor because one of the three
/// verbs it feeds is certain to refuse would take reflow and restyling away
/// from every run in this document to save one wasted keystroke in it.
///
/// ⇒ The caret stays; the **silence** goes. That is the whole of O140.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditRefusal {
    /// ★★★ **The line was written one piece at a time and the change spans
    /// more than one piece** — the operator's own document, and the case
    /// `RefusalKind` cannot name because it is not the engine's to see.
    ///
    /// Raised only when the engine answered `RefusalKind::NotFound` *and*
    /// `pin::spans_one_operator` had already said `false`. Both halves are
    /// required: a split run whose refusal is a font refusal gets the font
    /// sentence, because that is what actually stopped it.
    SplitAcrossPieces,
    /// ★★★ **The words being corrected appear more than once on the page, and
    /// pdfcer cannot tell which one the operator meant** — `OPERATOR_REQUESTS.md`
    /// **O142**, and the refusal that exists so that a *wrong* edit cannot.
    ///
    /// # Why this state exists at all
    ///
    /// A run the producer wrote one glyph per show operator can only be reached
    /// by `find` — `Pass 256.0`'s cross-operator matcher — and *"a pinned
    /// request never spans"*, so reaching it means sending the request **without
    /// the provenance pin**. The pin is the only thing `EditRequest` carries
    /// that can choose between two identical strings on one page: there is no
    /// occurrence index on the request, and `pinned_span` is the whole of its
    /// disambiguation.
    ///
    /// ⇒ So on a page where the text occurs twice the shell has a choice between
    /// **refusing** and **guessing**, and it refuses.
    /// [`crate::canvas::textedit::plan`] keeps the pin in that case, which makes
    /// the request unmatchable on purpose, and this variant is what turns the
    /// resulting `NotFound` into the true sentence instead of the one about the
    /// producer's glyph-per-operator writing — which is also true, and is not
    /// what stopped it.
    ///
    /// ★★ **This is the variant that must never be softened into an edit.** The
    /// document this shell was reported against is a signed quotation. Editing
    /// the wrong occurrence of a word on one of those is not a bug the operator
    /// reports — it is a bug he finds later, in a document he has already sent.
    /// The count that produces it deliberately over-counts rather than under-,
    /// and [`crate::canvas::textedit::page_occurrences`] carries that argument.
    ///
    /// ★ The payload is the count, because *"twice"* and *"nine times"* send the
    /// operator to different remedies: the first is a page he can look at, the
    /// second is a page where he wants Find rather than a caret.
    AmbiguousOnThePage(usize),
    /// ★★★ **The character IS in this font — twice — and pdfcer will not pick
    /// which glyph he meant.** `Pass 256.1`, consumed 2026-09-06.
    ///
    /// The composite twin of [`Self::FontLacksTheCharacter`], and the two arrive
    /// from the engine as the **same** `RefusalKind::UnsupportedFont`. What
    /// separates them is `Refusal::trigger`: `TargetAbsent` for a character the
    /// font does not carry, `Ambiguous` for one two CIDs map to.
    ///
    /// # Why it needs its own sentence when the remedy is identical
    ///
    /// Because the *fact* is opposite. `FontLacksTheCharacter` tells him the
    /// font *"was built with only the letters your page already prints"*, and on
    /// an ambiguous character that is a confident falsehood — the letter is on
    /// his page, in that font, twice. He would go looking for something missing
    /// that is in front of him.
    ///
    /// ★ Before `Pass 256.1` the engine refused this **per font** (`R-INV-4`,
    /// *"cannot be inverted"*), so the whole run was uneditable and no character
    /// was named. Now every character produced by exactly one code edits
    /// normally and only the colliding one refuses — so this variant describes a
    /// document that mostly works, and the sentence says so.
    FontHasTwoGlyphsFor(char),
    /// `RefusalKind::UnsupportedFont` — the code↔glyph relation is
    /// unrecoverable, or a substitute could not cover the text.
    ///
    /// ★★ Since O141 this is the arm for the refusals that name **no**
    /// character: R-INV-2 (a symbolic font whose encoding lives inside a
    /// program `pdfcer-core` does not parse), R-INV-3 and R-INV-4. Nothing
    /// about the run's *content* stopped those, so no other face would help and
    /// no offer is made. See [`Self::FontLacksTheCharacter`] for its twin.
    UnsupportedFont,
    /// ★★★ **The font carries the letters the page prints and not the one the
    /// operator just typed** — `OPERATOR_REQUESTS.md` **O141**, 2026-09-05.
    ///
    /// > *"if the character isn't available in a pdf are we able to change to a
    /// > different font?"*
    ///
    /// # Why this is a SIXTH sentence and not a rewording of the fifth
    ///
    /// The engine collapses both into `RefusalKind::UnsupportedFont` —
    /// `EditError::Refused(_)` maps there wholesale — and from the shell's chair
    /// they are two different situations with two different answers:
    ///
    /// | what happened | what the operator can do |
    /// |---|---|
    /// | [`Self::UnsupportedFont`] — R-INV-2/3/4, the font's code↔glyph relation is unrecoverable | nothing this build offers |
    /// | this — R-INV-1/6/7/8, the relation is fine and this **character** is not in it | **change the face**, which pdfcer can already do |
    ///
    /// Wording them the same would tell an operator who could fix their document
    /// in two clicks that pdfcer cannot edit their text — a confident wrong
    /// reason, which `RefusalKind`'s own header calls strictly worse than the
    /// silence it replaced.
    ///
    /// # ★★ The discriminant is the engine's, and it is a DATUM rather than a
    /// second taxonomy
    ///
    /// `crate::app::status::decline::record_edit_text_refusal` reads
    /// `Refusal::character` off `EditError::Refused`. That is not this crate
    /// re-deriving the engine's reasoning — the category still comes from
    /// `RefusalKind` and nothing here inspects a trigger id or greps a `Display`
    /// string. It reads **one field the coarse kind structurally cannot carry**,
    /// on the identical licence `one_operator` has: the sentence needs a fact
    /// the category does not hold.
    ///
    /// # ★★★ The character IS in this sentence, and it was not on the day the
    /// variant shipped
    ///
    /// It shipped carrying no payload, and the reason written here was a
    /// constraint of the surface rather than a judgement about the copy:
    /// *"`Declined::line` returns `&'static str`, so the status bar's `⊗` slot
    /// cannot interpolate a runtime character."* That was true, and it made the
    /// one decline in this catalog with a specific subject read exactly like the
    /// five that have none — which is the shape an operator learns to skip.
    ///
    /// [`Self::line`] returns a [`Cow`] now, and the whole cost of that was one
    /// arm of `Declined::line`: every other sentence in the shell is still
    /// borrowed static prose and allocates nothing. The character rides on the
    /// variant as a `char`, so [`EditRefusal`] and `Declined` both stay `Copy`.
    ///
    /// ★ The panel is still where the **route** is —
    /// `panels::properties::refusedchar` holds the chooser, rule 4's disclosure
    /// and the retype — because `disclosure_line` truncates the bar's slot to
    /// 45 % and hangs the rest on hover, and a route that ends in a hover is a
    /// route the operator does not find. What changed is that the bar now names
    /// the obstacle *specifically* before pointing at the panel.
    FontLacksTheCharacter(char),
    /// `RefusalKind::StructureFrozen` — encryption, an enforced certification
    /// signature, or a suppressed object set.
    DocumentProtected,
    /// `RefusalKind::NotFound` on a run the shell had measured as a **single**
    /// operator, so the split explanation is unavailable and the honest reading
    /// is that the page moved under the caret.
    TextMovedAway,
    /// `RefusalKind::Other` — an invalid parameter, an unbuilt combination, a
    /// parse or save failure. The engine's own words are on the trace.
    Unstated,
}

/// **Which of the two character-level font refusals the engine raised** —
/// `Pass 256.1`, consumed 2026-09-06.
///
/// A shell-side spelling of one field of `pdfcer_core::text_edit::Refusal`, and
/// deliberately not a copy of the engine's `RInvTrigger`: it carries the **two
/// cases that produce different sentences here**, and collapses the six that do
/// not. Widening it is how a second taxonomy starts, so widen it only when a
/// seventh trigger earns a seventh sentence.
///
/// `app::status::decline::textedit::refused_char_kind` is the one place that
/// builds it, and its doc comment carries the argument for why reading
/// `Refusal::trigger` there is a datum rather than a re-derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusedCharacter {
    /// R-INV-1/6/7/8 — the font does not carry this character.
    NotInTheFont(char),
    /// R-INV-5 on a composite font — the font carries it **twice**, two CIDs map
    /// to it, and pdfcer does not pick glyphs.
    TwoGlyphsFor(char),
}

impl RefusedCharacter {
    /// The character itself, whichever case this is.
    #[must_use]
    pub const fn character(self) -> char {
        match self {
            Self::NotInTheFont(c) | Self::TwoGlyphsFor(c) => c,
        }
    }
}

impl EditRefusal {
    /// **Classify one refused text edit**, from the engine's coarse kind and
    /// the one thing the shell knows that the engine cannot.
    ///
    /// # ★★★ The order of the arms is the design
    ///
    /// The engine's category wins wherever it has one, and the shell's fact is
    /// consulted **only** inside `NotFound` — the single bucket where the
    /// engine's answer is true and unusable at the same time.
    ///
    /// That ordering is what keeps this from being a second taxonomy. A split
    /// run whose edit was stopped by a font gets
    /// [`Self::UnsupportedFont`], not [`Self::SplitAcrossPieces`], because the
    /// font is what actually stopped it and the split is merely also true. The
    /// failure mode of the opposite ordering is precisely the one
    /// `RefusalKind`'s own header warns about: *telling the operator the wrong
    /// reason, which is strictly worse than the silence it replaced.*
    ///
    /// # ★★ `one_operator` is a measurement, and `true` is its "not measured"
    ///
    /// See [`crate::canvas::textedit::Plan::one_operator`]. It is `true` when
    /// the plan could not read provenance at all, so an unmeasured run falls to
    /// [`Self::TextMovedAway`] — a sentence that is honest about a page that
    /// moved and never claims a structure this shell did not observe.
    /// # ★★★ `character` is the second fact the category cannot hold
    ///
    /// `OPERATOR_REQUESTS.md` O141. `Some` when the engine's refusal carried a
    /// `Refusal::character` — i.e. the inverse-encoding gate stopped on **one
    /// scalar it has no code for** (R-INV-1, 6, 7, 8) rather than on the font's
    /// whole code↔glyph relation being unreadable (R-INV-2, 3, 4). Both arrive
    /// as `RefusalKind::UnsupportedFont`, and only the first has a remedy.
    ///
    /// ★★ It arrives as the character itself rather than as a `bool` since
    /// 2026-09-05, and the widening is the whole of what let the status bar name
    /// it. The datum was already being read at the call site — the recorder
    /// needs it for the panel either way — and passing the answer instead of a
    /// predicate over it removes the state where two surfaces agree that *a*
    /// character was refused and only one of them can say which.
    ///
    /// ★ It is consulted **only** inside `UnsupportedFont`, on the same rule the
    /// `NotFound` split follows: the engine's category wins wherever it has one,
    /// and a shell-side fact is allowed to sharpen exactly the bucket where the
    /// engine's answer is true and unusable at the same time.
    /// # ★★★ `occurrences` is the THIRD fact the category cannot hold
    ///
    /// `OPERATOR_REQUESTS.md` O142. `Some(n)` when the plan had to reach the run
    /// by `find` — the producer wrote it one glyph per show operator, so the
    /// provenance pin would have confined the match to one character — and `n`
    /// is how many times that text occurs on the page. `None` when the pin was
    /// exact and no string was being matched at all.
    ///
    /// It is consulted **only** inside `NotFound`, and **before**
    /// `one_operator`, and that ordering is the design rather than an
    /// implementation detail. Both facts are true at once on the page that
    /// raises this: the run *is* split across pieces, and the text *does* occur
    /// twice. Only the second is what stopped the edit —
    /// [`crate::canvas::textedit::plan`] kept the pin **deliberately**, to make
    /// the request unmatchable rather than let the engine pick an occurrence —
    /// so reporting the split would be this shell explaining its own refusal
    /// with somebody else's reason.
    ///
    /// ⇒ That is the same rule the arms below already follow, applied one level
    /// deeper: *the thing that actually stopped it wins, and the things that are
    /// merely also true stand aside.*
    #[must_use]
    pub const fn of(
        kind: pdfcer_core::text_edit::RefusalKind,
        one_operator: bool,
        character: Option<RefusedCharacter>,
        occurrences: Option<usize>,
    ) -> Self {
        use pdfcer_core::text_edit::RefusalKind as K;
        // ★ Matched as a PAIR rather than with an `if let` guard, so the arms
        // are exhaustive over both facts by construction and the compiler proves
        // the split rather than a reader checking it. The `_` in the three arms
        // below the font pair is not laziness: the character is meaningful only
        // where the font stopped the edit, and reading it anywhere else would be
        // the shell inventing a remedy for a refusal that has none.
        match (kind, character) {
            (K::UnsupportedFont, Some(RefusedCharacter::NotInTheFont(c))) => {
                Self::FontLacksTheCharacter(c)
            }
            // ★★★ Pass 256.1. Same engine category, same remedy, opposite FACT
            // — the character is in the font twice rather than not at all — so
            // it is a different sentence. See `RefusedCharacter`.
            (K::UnsupportedFont, Some(RefusedCharacter::TwoGlyphsFor(c))) => {
                Self::FontHasTwoGlyphsFor(c)
            }
            (K::UnsupportedFont, None) => Self::UnsupportedFont,
            (K::StructureFrozen, _) => Self::DocumentProtected,
            // ★★★ O142. Ahead of the split arm on purpose — see the
            // `occurrences` section above. `Some(n)` with `n != 1` is the shell
            // having KEPT the pin so that this refusal would happen; anything
            // else falls through to the arms that were here before.
            (K::NotFound, _) => match occurrences {
                Some(n) if n != 1 => Self::AmbiguousOnThePage(n),
                _ if !one_operator => Self::SplitAcrossPieces,
                _ => Self::TextMovedAway,
            },
            (K::Other, _) => Self::Unstated,
        }
    }

    /// **The category's stable name, for the trace** — not `{:?}`, and the
    /// difference cost a driven run on 2026-09-05.
    ///
    /// # Why this exists rather than a derive
    ///
    /// `app::status::decline::textedit` publishes `said=` beside `kind=` so a
    /// build that read the engine's category correctly and then chose the wrong
    /// sentence goes red, and `tools/ui-verify`'s
    /// `a_refused_character_offers_a_face_that_can_type_it` and
    /// `a_refused_typo_fix_says_why_it_was_refused` both match on that field.
    ///
    /// It was `{why:?}`. The moment [`Self::FontLacksTheCharacter`] gained its
    /// payload the field became `FontLacksTheCharacter('q')` and both checks
    /// went red against a build that was working perfectly — the same failure,
    /// in the same file, that the trace's own comment had already recorded once:
    /// *"`{:?}` on a domain type makes the trace's vocabulary a consequence of a
    /// Rust derive, so it changes silently when the type does."* The note did
    /// not prevent the second occurrence; this function does, because a variant
    /// added without a name is a compile error here.
    ///
    /// ★ The payload is deliberately **not** in the name. The character already
    /// has its own `character=` field on the same line, and one datum spelled
    /// two ways in one trace line is how a reader and a check come to disagree
    /// about which is authoritative.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SplitAcrossPieces => "SplitAcrossPieces",
            Self::AmbiguousOnThePage(_) => "AmbiguousOnThePage",
            Self::UnsupportedFont => "UnsupportedFont",
            Self::FontLacksTheCharacter(_) => "FontLacksTheCharacter",
            Self::FontHasTwoGlyphsFor(_) => "FontHasTwoGlyphsFor",
            Self::DocumentProtected => "DocumentProtected",
            Self::TextMovedAway => "TextMovedAway",
            Self::Unstated => "Unstated",
        }
    }

    /// The sentence for this cause.
    ///
    /// One function over the enum rather than one per variant, for
    /// [`refusal`]'s and [`ReflowRefusal::line`]'s reason: a variant added
    /// without a sentence is a compile error rather than a commit that refuses
    /// silently.
    ///
    /// # ★★ Every sentence is FRONT-LOADED, and that is a layout fact
    ///
    /// `app::status::disclosure::disclosure_line` draws the decline with
    /// `.truncate()` and hangs the whole text on hover. So the first clause is
    /// what most operators read, and it must carry the claim that matters:
    /// **pdfcer cannot**, not *you did something wrong*. The cause, the
    /// contrast and the remedy follow it in that order.
    ///
    /// # ★ Two of them name his contrast explicitly
    ///
    /// He noticed it before the program told him: *"the lines I added below
    /// `price)` are editable, but everything else that existed when I got the
    /// pdf is not."* A sentence that explains the refusal and ignores the
    /// contrast reads as evasive, because he has already worked out that the
    /// two cases differ and is waiting to hear why.
    ///
    /// # ★★★ Why this returns a [`Cow`] and every other catalog function does not
    ///
    /// Because exactly one of these sentences has a **subject the operator can
    /// see** — the character they just typed — and until 2026-09-05 it could not
    /// say it. The five sentences around it are about a property of the document
    /// (its font, its protection, how its producer wrote the line) and are
    /// complete as fixed prose; this one was the general statement of a specific
    /// event, and it read like every other decline in the bar as a result.
    ///
    /// [`Cow::Borrowed`] is what five of the six arms return, so the change
    /// costs no allocation on any frame that is not reporting this one refusal —
    /// and the bar redraws every frame, which is why that mattered enough to
    /// state. The alternative, a `String` return, would have allocated the same
    /// five static sentences over and over for the life of the process.
    #[must_use]
    pub fn line(self) -> std::borrow::Cow<'static, str> {
        use std::borrow::Cow;
        // ★ Bound through a `&'static str` so that only the ONE arm which
        // interpolates carries any machinery: it returns, and every other arm
        // stays the plain catalog entry it was. Wrapping all six in
        // `Cow::Borrowed(..)` would have put five words in front of every
        // sentence in the file for the sake of one of them.
        let fixed: &'static str = match self {
            // ★★★ The remedy clause is the one this project checked before
            // writing. `pdfcer-core` can edit a piece: `--find "n" --replace
            // "t"` succeeded on this very run. What it cannot do is address ONE
            // piece from a caret — the shell has no gesture that names a single
            // glyph operator, and `EditRequest` carries one pin per request. So
            // there is no sequence of clicks that gets him his `t` today, and
            // the sentence says exactly that rather than inventing one.
            //
            // ⚠ It deliberately does NOT say "delete it and retype it". That
            // was checked too: `add_text` writes the engine's bundled
            // Helvetica, so retyping a line of AbrilFatface or Arimo replaces
            // his typography with a substitute at a position he would have to
            // find by eye — and there is no verb here that deletes a text run
            // in the first place. Offering it would be a workaround that does
            // not exist.
            Self::SplitAcrossPieces => {
                "pdfcer cannot change these words. The program that made this file wrote the line \
                 one letter at a time, and pdfcer rewrites a whole piece of text at once — so \
                 there is no piece here that holds the word you are correcting. Text you added \
                 with pdfcer is written a line at a time, which is why those lines do edit. Your \
                 document is unchanged; this limit is on the list to fix."
            }
            Self::UnsupportedFont => {
                "pdfcer cannot write new letters into this text. Its font records what each shape \
                 looks like but not which letter it is, so pdfcer cannot spell a letter that is \
                 not already there. Text you added with pdfcer uses a font it can spell in, which \
                 is why those lines do edit. Your document is unchanged."
            }
            // ★★★ O141. The one decline in this catalog whose whole purpose is
            // to hand the operator on to a control, so the last clause names
            // that control and the panel it is in.
            //
            // ⚠ It deliberately does NOT name the character. `Declined::line`
            // is `&'static str` and `disclosure_line` truncates the slot to
            // 45 % of the bar; a sentence that promised to name a character and
            // then could not would be worse than one that sends the operator
            // to the surface that does. `panels::properties::refusedchar` names
            // it, offers the faces, and carries rule 4's disclosure.
            //
            // ★★★ The one sentence in this catalog built from a runtime value,
            // so it leaves through a `return` rather than through `fixed`. The
            // character is quoted with the SAME `'c'` spelling the Properties
            // block and the classification trace use, so an operator reading the
            // bar and the panel sees one character described one way — and a
            // driven check comparing the two surfaces compares like with like.
            // `app::status::decline::textedit` learned the cost of two spellings
            // on 2026-09-05, when a debug-formatted tuple in the trace made a
            // correct build report itself broken.
            Self::FontLacksTheCharacter(c) => {
                return std::borrow::Cow::Owned(font_lacks_the_character(c));
            }
            // ★★★ O142. The second sentence in this catalog built from a runtime
            // value, and it leaves through a `return` for the same reason the
            // first one does. The count is the whole point: *"more than once"*
            // leaves the operator hunting, *"3 times"* tells him what to look
            // for.
            Self::AmbiguousOnThePage(n) => {
                return std::borrow::Cow::Owned(ambiguous_on_the_page(n));
            }
            // ★★ Pass 256.1's own words for the remedy: "this letter has two
            // glyphs in this font; pick another font for it".
            Self::FontHasTwoGlyphsFor(c) => {
                return std::borrow::Cow::Owned(font_has_two_glyphs_for(c));
            }
            Self::DocumentProtected => {
                "This document's protection does not allow its text to be changed, so pdfcer left \
                 it alone. If you have the password, remove the protection first with Protect > \
                 Remove security, then edit."
            }
            Self::TextMovedAway => {
                "pdfcer could not find the text this edit named — the page has moved on since the \
                 cursor was placed there. Click in the words again and make the change a second \
                 time. Your document is unchanged."
            }
            // ★ The one variant with no cause to name, and it is deliberately
            // the SAME sentence the whole verb used to show. Where the engine
            // says only "other", inventing a cause here would be the "second
            // copy of their taxonomy that drifts and then tells the operator
            // the WRONG reason" that `RefusalKind` exists to prevent.
            Self::Unstated => crate::text::status::edit_declined_by_engine(),
        };
        Cow::Borrowed(fixed)
    }
}

/// ★★★ **"pdfcer cannot type a `q` into this text"** — the status bar's `⊗`
/// sentence for `OPERATOR_REQUESTS.md` **O141**, with the character in it.
///
/// The operator, 2026-09-05: *"if the character isn't available in a pdf are we
/// able to change to a different font?"*
///
/// # Why the character is worth the one allocation this catalog makes
///
/// Because the generic form of this sentence is what he meets first, and it is
/// indistinguishable from the five other declines he might have earned. *"pdfcer
/// cannot type that character"* asks him to remember what he typed and to
/// believe pdfcer knows; *"pdfcer cannot type a `q`"* is a report he can check
/// against the keyboard in front of him. The engine handed the character over
/// (`Refusal::character`) and two surfaces already had it; only the bar could
/// not say it, and only because of a return type.
///
/// # ★★ The clauses, in the order the bar truncates them
///
/// `app::status::disclosure::disclosure_line` draws at most 45 % of the bar and
/// hangs the rest on hover, so the order is not style:
///
/// 1. **what pdfcer cannot do, with the character** — the claim, which must
///    survive truncation;
/// 2. **why**, in his words rather than the format's: the font was built with
///    only the letters the page already prints. O141's framing is that he should
///    not have to learn the word *subset* to understand his own file;
/// 3. **his document is unchanged** — the reassurance every decline in this
///    catalog carries, because the operator's first question after a refusal is
///    whether something happened anyway;
/// 4. **where the route is.** The bar cannot hold a chooser; Properties can, and
///    the sentence says so by name.
///
/// ⚠ It deliberately does not say *"choose another font"* on its own. That is
/// the answer, and an answer with no control beside it is O141 filed all over
/// again — *"that last clause is the answer to your question, and it is buried
/// in an error message."*
#[must_use]
pub fn font_lacks_the_character(character: char) -> String {
    format!(
        "pdfcer cannot type '{character}' into this text. The font here was built with only the \
         letters your page already prints, and pdfcer cannot add one to a font that is already \
         inside the file. Your document is unchanged — open Properties, which names the \
         character and offers the faces that can type it."
    )
}

/// ★★★ **"this font draws 'ﬁ' two different ways"** — the status bar's `⊗`
/// sentence for the composite ambiguous-inverse refusal (`Pass 256.1`).
///
/// # What is true here, and it is the opposite of the sentence beside it
///
/// [`font_lacks_the_character`] tells the operator the font *"was built with
/// only the letters your page already prints"*. That is exactly wrong for this
/// refusal: the character **is** on his page, in this font, drawn by two
/// different glyphs — and pdfcer declines to choose between them, because
/// choosing would silently swap one shape for another that happens to spell the
/// same letter.
///
/// ★ The engine used to refuse the whole font for this (`R-INV-4`), so the
/// sentence's second clause is now load-bearing in a way it could not have been
/// before `Pass 256.1`: **the rest of the text still edits**. Saying only that
/// something was refused would make him abandon a document where every other
/// character is fine.
///
/// ★★ The remedy is the same control as [`font_lacks_the_character`]'s and the
/// sentence ends by naming it, because the face offer really is raised for this
/// case too — `app::status::decline::textedit` calls
/// `panels::properties::refusedchar::record` on any refusal that named a
/// character, and this one names one.
#[must_use]
pub fn font_has_two_glyphs_for(character: char) -> String {
    format!(
        "pdfcer cannot type '{character}' into this text, because this font draws that letter \
         two different ways and pdfcer will not choose one for you — it would change the shape \
         of the letter without telling you. The rest of this text still edits normally. Your \
         document is unchanged — open Properties, which offers the faces that spell it only one \
         way."
    )
}

/// ★★★ **"the same words appear 3 times on this page"** — the status bar's `⊗`
/// sentence for `OPERATOR_REQUESTS.md` **O142**, with the count in it.
///
/// # What it must convey, and the order is deliberate
///
/// 1. **pdfcer will not guess.** The first clause carries the claim, because
///    `app::status::disclosure::disclosure_line` truncates the slot and hangs
///    the rest on hover — most operators read only the first clause.
/// 2. **His document is untouched.** This refusal happens on a page he may have
///    already signed and sent, and *"could not"* without *"nothing changed"*
///    reads as *"something half-happened"*.
/// 3. **What to do instead.** Which is where the sentence has to be careful.
///
/// # ★★ Why it does NOT tell him to use Find and Replace
///
/// That was the obvious remedy and it was checked before being written, on this
/// project's standing rule that a sentence naming a remedy is a claim about the
/// build. Find and Replace would rewrite **every** occurrence, which is the
/// opposite of what he is asking for — he wants one of them corrected. Sending
/// him there would trade a refusal for a wrong edit he performed himself.
///
/// So it names the thing that actually narrows the ambiguity: editing a **longer
/// stretch** of the line, which is unique where the short one is not. That works
/// today, needs no new capability, and is the same gesture he already made.
///
/// ★ It says *"the same words"* rather than quoting them. The words in question
/// can be a whole line — thirty-six characters on the page he reported — and the
/// bar's slot is 45 % of its width; a sentence that began by quoting them would
/// truncate before it reached the part that tells him anything.
#[must_use]
pub fn ambiguous_on_the_page(count: usize) -> String {
    format!(
        "pdfcer will not change these words, because the same words appear {count} times on this \
         page and it cannot tell which one you mean. Nothing in your document has changed. Click \
         in the line again and include more of it in your change — a longer stretch appears only \
         once, and pdfcer can then correct exactly the one you are looking at."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    /// **Every variant**, so a sixth added without a sentence, without an
    /// obstacle in its first clause, or promising a remedy this build does not
    /// have, goes red rather than shipping.
    ///
    /// ★ A hand-written list inside a completeness sweep is a shape this
    /// project has been bitten by three times, and it is tolerable here for one
    /// reason: `EditRefusal::line`'s `match` is exhaustive, so a new variant is
    /// a **compile error** in the catalog before it can be a gap in this list.
    /// The list is a convenience over a closed set, not the closure itself.
    const EVERY: [EditRefusal; 8] = [
        EditRefusal::SplitAcrossPieces,
        // ★ O142. `3` rather than `2` on purpose: the sentence interpolates the
        // count, and `2` is the one value a build could hard-code and still
        // satisfy a sweep that only checked the number was present.
        EditRefusal::AmbiguousOnThePage(3),
        EditRefusal::UnsupportedFont,
        // ★ The character is arbitrary here on purpose: this list exists to
        // sweep the sentences, and `'q'` is the operator's own example.
        EditRefusal::FontLacksTheCharacter('q'),
        // ★ Its Pass 256.1 twin — same character, opposite fact.
        EditRefusal::FontHasTwoGlyphsFor('q'),
        EditRefusal::DocumentProtected,
        EditRefusal::TextMovedAway,
        EditRefusal::Unstated,
    ];

    /// **No sentence opens with the operator.** His report is *"the edit is not
    /// accepted"*, and a sentence beginning with what he did reads as a
    /// correction of him rather than an admission by the program.
    ///
    /// ★ Checked on the first clause because that is the part the status bar
    /// actually shows — `disclosure_line` truncates and hangs the rest on
    /// hover.
    #[test]
    fn no_edit_refusal_opens_by_naming_what_the_operator_did() {
        for why in EVERY {
            let first = why.line().split('.').next().unwrap_or_default().to_owned();
            assert!(
                !first.starts_with("You ") && !first.starts_with("Your "),
                "a decline that opens with the operator reads as a correction of them: {first:?}"
            );
        }
    }

    /// ★★★ **Every sentence this shell WORDED names the obstacle in its first
    /// clause** — and the exception proves the rule rather than weakening it.
    ///
    /// The first version of this test asserted it over all five variants and
    /// went red on [`EditRefusal::Unstated`], whose line is
    /// *"That change was refused, and the document is unchanged."* — the
    /// pre-O140 sentence, deliberately reused byte for byte.
    ///
    /// ⇒ **The test was asserting more than the design promises.** `Unstated`
    /// is the arm where the engine says only *other*, and its whole point is
    /// that this shell has nothing to add; naming an obstacle there would be
    /// the invented cause `RefusalKind` exists to prevent. So the obligation
    /// belongs to the four categorised sentences, and is stated over exactly
    /// those — with the fifth pinned separately, by
    /// [`Self::the_uncategorised_case_keeps_the_sentence_it_always_had`], to
    /// the string it is supposed to be.
    ///
    /// [`Self::the_uncategorised_case_keeps_the_sentence_it_always_had`]: the_uncategorised_case_keeps_the_sentence_it_always_had
    #[test]
    fn every_categorised_refusal_names_the_obstacle_in_its_first_clause() {
        for why in [
            EditRefusal::SplitAcrossPieces,
            EditRefusal::UnsupportedFont,
            EditRefusal::FontLacksTheCharacter('q'),
            EditRefusal::DocumentProtected,
            EditRefusal::TextMovedAway,
        ] {
            let first = why.line().split('.').next().unwrap_or_default().to_owned();
            assert!(
                first.contains("pdfcer") || first.contains("document's protection"),
                "the visible clause must name the program or the document as the obstacle: \
                 {first:?}"
            );
        }
    }

    /// ★★★ **Every mapping from the engine's four buckets, exhaustively.**
    ///
    /// The compiler already proves `EditRefusal::of` handles every
    /// `RefusalKind` — that is what `RefusalKind` not being `#[non_exhaustive]`
    /// buys, and it is why the engine committed to it. What the compiler cannot
    /// prove is that the **`NotFound` split is wired the right way round**, and
    /// getting it backwards is the failure mode that matters most here: the
    /// operator would be told his page had moved when his line is written one
    /// letter at a time, or the reverse.
    #[test]
    fn the_engines_four_buckets_map_to_six_sentences_and_two_of_them_split() {
        use pdfcer_core::text_edit::RefusalKind as K;
        // The two that ignore both shell-side facts, asserted every way so a
        // stray condition added to either goes red.
        for one in [true, false] {
            for named in [Some(RefusedCharacter::NotInTheFont('q')), None] {
                // ★ Swept over `occurrences` as well since O142: three
                // shell-side facts now reach this function and these two
                // buckets must ignore all of them. A condition on the newest one
                // is the easiest to add by accident, because it is the only one
                // whose absence carries a meaning (`None` = the pin was exact).
                for seen in [None, Some(1), Some(2)] {
                    assert_eq!(
                        EditRefusal::of(K::StructureFrozen, one, named, seen),
                        EditRefusal::DocumentProtected
                    );
                    assert_eq!(
                        EditRefusal::of(K::Other, one, named, seen),
                        EditRefusal::Unstated
                    );
                }
            }
        }
        // ★ And the two that do not. `one_operator == false` means the run is
        // written in several pieces, which is the operator's own document.
        assert_eq!(
            EditRefusal::of(K::NotFound, false, None, None),
            EditRefusal::SplitAcrossPieces,
            "a split run's NotFound is the split, not a page that moved"
        );
        assert_eq!(
            EditRefusal::of(K::NotFound, true, None, None),
            EditRefusal::TextMovedAway,
            "a single-operator run's NotFound has no split to blame, and claiming one \
             would be a structure this shell did not observe"
        );
        // ★★★ O142, AND THE PRECEDENCE IS THE ASSERTION. On the page that raises
        // this, both facts are true at once: the run is split across pieces AND
        // the text occurs twice. Only the second is what stopped the edit —
        // `canvas::textedit::plan` kept the pin deliberately, to make the
        // request unmatchable rather than let the engine choose an occurrence —
        // so a build reporting the split would be explaining its own refusal
        // with somebody else's reason.
        assert_eq!(
            EditRefusal::of(K::NotFound, false, None, Some(2)),
            EditRefusal::AmbiguousOnThePage(2),
            "★ ambiguity OUTRANKS the split, and this is the ordering assertion"
        );
        // ★★ And `Some(1)` must NOT take that arm. One occurrence is the case
        // where the pin was dropped and the edit was expected to land, so a
        // `NotFound` there is the honest split refusal. A build that keyed on
        // "`occurrences` is `Some`" rather than on the count would tell every
        // operator of every per-glyph run on every page that his words appear
        // once and pdfcer cannot tell which one he means.
        assert_eq!(
            EditRefusal::of(K::NotFound, false, None, Some(1)),
            EditRefusal::SplitAcrossPieces,
            "★ a UNIQUE run that still refused is the split, not an ambiguity"
        );
        assert_eq!(
            EditRefusal::of(K::NotFound, true, None, Some(2)),
            EditRefusal::AmbiguousOnThePage(2),
            "and ambiguity does not depend on the split: it is a fact about the PAGE"
        );
    }

    /// ★★★ **The font bucket splits on whether the engine named a character**
    /// — `OPERATOR_REQUESTS.md` O141, and it is the mapping a wrong build gets
    /// backwards.
    ///
    /// `EditError::Refused(_)` maps to `RefusalKind::UnsupportedFont`
    /// **wholesale**, so R-INV-1 (*"this font has no glyph for '€'"* — a
    /// character the subset does not carry, and a face swap fixes it) and
    /// R-INV-2 (*"its code↔glyph relation lives inside the embedded program,
    /// which pdfcer-core does not parse"* — and no face swap helps, because the
    /// run cannot be re-encoded at all) arrive as the same category.
    ///
    /// Getting it round the wrong way costs in both directions: an operator two
    /// clicks from their `€` is told pdfcer cannot edit the text, or an operator
    /// with an unreadable font is sent to a chooser that will refuse every row.
    /// `Refusal::character` is `Some` for exactly the first family and `None`
    /// for the second, which is why the datum is read rather than the trigger id.
    ///
    /// ★ Asserted across `one_operator` as well, because the run's provenance
    /// has nothing to do with the font's repertoire and a condition that crept
    /// in would make the sentence depend on how the producer emitted the line.
    #[test]
    fn a_font_refusal_that_names_a_character_is_the_one_with_a_way_out() {
        use pdfcer_core::text_edit::RefusalKind as K;
        for one in [true, false] {
            // ★ `Some(2)` deliberately, not `None`: a font refusal on a page
            // whose text is ambiguous is still a FONT refusal. The occurrence
            // count is consulted only inside `NotFound`, and a condition that
            // leaked out of that bucket would send an operator whose font cannot
            // spell his character to a sentence about duplicated words.
            assert_eq!(
                EditRefusal::of(
                    K::UnsupportedFont,
                    one,
                    Some(RefusedCharacter::NotInTheFont('q')),
                    Some(2)
                ),
                EditRefusal::FontLacksTheCharacter('q'),
                "a refusal naming one character is a repertoire fact, and changing the face \
                 is the remedy pdfcer already has"
            );
            // ★★★ Pass 256.1. The SAME engine category and the SAME character,
            // separated only by `Refusal::trigger` — and they must not collapse
            // into one sentence, because the facts are opposite: one says the
            // letter is absent from the font, the other says it is in there
            // twice. An operator told the wrong one goes looking for a missing
            // letter that is on his page in front of him.
            assert_eq!(
                EditRefusal::of(
                    K::UnsupportedFont,
                    one,
                    Some(RefusedCharacter::TwoGlyphsFor('q')),
                    Some(2)
                ),
                EditRefusal::FontHasTwoGlyphsFor('q'),
                "an AMBIGUOUS inverse is not a missing character: the font draws it two ways \
                 and pdfcer declines to pick a glyph"
            );
            assert_eq!(
                EditRefusal::of(K::UnsupportedFont, one, None, Some(2)),
                EditRefusal::UnsupportedFont,
                "a refusal naming NO character is about the font's whole code-to-glyph \
                 relation, and no other face makes this run re-encodable"
            );
        }
    }

    /// ★★ **The sentence that has a way out says where the way out is**, and it
    /// is the only decline in this catalog that hands the operator to a control.
    ///
    /// Without the last clause O141 is answered with a better diagnosis and no
    /// route — which is exactly the state O141 was filed about: *"That last
    /// clause is the answer to your question, and it is buried in an error
    /// message."*
    #[test]
    fn the_missing_character_sentence_names_the_surface_that_answers_it() {
        let s = EditRefusal::FontLacksTheCharacter('q').line();
        assert!(
            s.contains("Properties"),
            "a decline with a remedy must name where the remedy is: {s:?}"
        );
        assert!(
            s.contains("faces that can type it"),
            "and it must say what the operator will find there, or the route reads as a \
             suggestion to go and look: {s:?}"
        );
        assert!(
            s.contains("unchanged"),
            "the first question after a refused edit is always whether it took: {s:?}"
        );
        // ★ And it must NOT claim to name the character, because it cannot:
        // `Declined::line` is `&'static str`. A sentence promising a character
        // it does not carry is the shape of falsehood this catalog's tests are
        // for.
        assert!(
            !s.contains('“') && !s.contains('”'),
            "the quoted character belongs to the panel, which can interpolate one: {s:?}"
        );
    }

    /// ★★★ **The two font-shaped refusals answer his contrast.**
    ///
    /// *"the lines I added below `price)` are editable, but everything else
    /// that existed when I got the pdf is not."* He had the diagnosis before
    /// the program did. A sentence that explains the refusal and says nothing
    /// about why his own lines behave differently leaves the one question he
    /// actually asked unanswered.
    #[test]
    fn the_two_content_refusals_explain_why_his_own_added_lines_edit() {
        for why in [EditRefusal::SplitAcrossPieces, EditRefusal::UnsupportedFont] {
            let s = why.line();
            assert!(
                s.contains("added with pdfcer"),
                "his contrast is unaddressed: {s:?}"
            );
            assert!(s.contains("do edit"), "and it must say which way: {s:?}");
        }
    }

    /// **No sentence promises a workaround.**
    ///
    /// ⚠ Verified before it was written, not assumed: there is no verb in this
    /// shell that deletes a text run, and `add_text` writes the engine's
    /// bundled Helvetica — so *"delete it and retype it"* would cost the
    /// operator his typography and his position, and is not offered. A remedy
    /// named in a decline is a promise, and a promise that does not resolve is
    /// worse than the silence it replaced.
    #[test]
    fn no_edit_refusal_offers_a_remedy_this_build_does_not_have() {
        for why in EVERY {
            let s = why.line().to_ascii_lowercase();
            assert!(
                !s.contains("retype") && !s.contains("delete it"),
                "neither deletion nor retyping is available, so neither may be suggested: {s:?}"
            );
        }
    }

    /// ★★ **The split-run sentence says the document is intact and does not
    /// pretend there is something to do.**
    ///
    /// The failure this ends is not *"I was not told why"* — it is *"I do not
    /// know whether it took"*. `edit_declined_by_engine`'s own documentation
    /// carries the argument; this variant inherits the obligation.
    #[test]
    fn the_split_run_sentence_says_the_document_is_unchanged() {
        let s = EditRefusal::SplitAcrossPieces.line();
        assert!(s.contains("unchanged"), "{s:?}");
        assert!(
            s.contains("one letter at a time"),
            "the cause has to be in his terms, not in show operators: {s:?}"
        );
    }

    /// **`Unstated` is the old sentence, unchanged.** Where the engine says
    /// only *other*, this shell has nothing to add — and adding something would
    /// be the invented cause `RefusalKind` exists to prevent.
    #[test]
    fn the_uncategorised_case_keeps_the_sentence_it_always_had() {
        assert_eq!(
            EditRefusal::Unstated.line(),
            crate::text::status::edit_declined_by_engine()
        );
    }
}
