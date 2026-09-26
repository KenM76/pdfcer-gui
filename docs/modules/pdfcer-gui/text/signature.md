# `text::signature` — what this shell says about a digital signature before
and after it writes

The copy for [`crate::dialogs::signature`] and for the two status-bar notes
[`crate::app::save`] records once a file has been written.

## THE RULE THAT GOVERNS EVERY STRING IN THIS FILE

**Nothing here may be invented.** This is claim-bearing copy about a
security property of a legal artifact, and the engine that computes the
verdict has already written down, at length, exactly which claims are
supportable and which are not. Every sentence below is a translation of a
distinction `pdfcer-core`'s `signature` module draws, into operator English,
**without softening it and without strengthening it**. Where a sentence
could be read as saying more than the engine says, it has been cut back
until it cannot.

Read `D:\Dev\pdfcer\crates\pdfcer-core\src\signature.rs`'s module
documentation before changing a word of this file. Its own header opens
with the instruction, and the reason is not ceremony: the module records
that *"the folk model is wrong in three separate places"*, so recall is a
worse source here than in any other area of the catalog.

## The two stages, which is the distinction the whole file is built around

ISO 32000-1 §12.8.2.2.2 splits signature validation in two:

| Stage | What it proves | Applies to |
|---|---|---|
| **1 — byte-range digest** | the bytes the signature covers are unchanged since signing | every signature with a `/ByteRange` |
| **2 — permitted-changes analysis** | later revisions stayed inside the author's allowance | only signatures carrying a transform method |

§12.8.1 NOTE 1 promises that an **incremental** update preserves the signed
byte range. That is a **stage-1** fact and it says nothing at all about
stage 2. The engine states the consequence in as many words:

> Reporting stage-1 success as "the signature is still valid" is the
> specific error this section exists to prevent.

Hence [`preserved_note`], which is the only string in this file that
describes a save whose signatures survived it, spends its second half
saying what it does **not** mean. That is not hedging. It is the difference
between a disclosure and a reassurance, and the engine's variant
documentation forbids the second in terms: *"A front end that renders this
as a reassurance is committing precisely the error §12.8.2.2.2's two-stage
split exists to prevent. Pair it with the uncertainty, or say nothing."*

## Why there are TWO wordings for one verdict

`SignatureImpact::Invalidated` is reached on two different footings, and
`SignatureImpact::documentation_basis` exists — in the engine's own words —
*"because the two deserve different operator-facing wording and a front end
cannot tell them apart from the variant alone"*:

| Footing | When | The strings |
|---|---|---|
| `ImpactBasis::SpecSourced` | the document carries a **certification** signature, whose `/DocMDP` transform records a **closed** list of permitted changes (Table 254: *"other changes shall invalidate the signature"*) | [`headline_certified`], [`basis_certified`], [`proceed_certified`] |
| `ImpactBasis::ConservativeReport` | the document carries only **approval** signatures, for which ISO 32000-1 defines stage 1 and **only** stage 1 — so a conforming validator reading the standard alone would call the document still valid | [`headline_approval`], [`basis_approval`], [`proceed_approval`] |

The second row is the engine's headline negative result, and it is the
reason the second wording is careful in a way the first is not. pdfcer
reports `Invalidated` there anyway — *"a product decision under rule 4
(fuzzy-never-sneaky), not a spec citation"* — on the asymmetry that
**over-reporting is a reviewable hint an operator can dismiss, and
under-reporting is pdfcer making a silent claim about the integrity of a
legal artifact.** So the copy must report the verdict *and* say whose
verdict it is. [`basis_approval`] does both in one sentence.

## Three things no string in this file is allowed to say

1. **That any other reader agrees.** The widely-repeated claim that Acrobat
   and the PAdES family report such a document as *"signed, but altered
   since signing"* is, in the engine's words, *"empirical tool behaviour,
   explicitly not sourced"*, and *"must not be cited from here"*. So no
   sentence below predicts what will happen in another application. That is
   a real temptation — it is the most useful thing an operator could be
   told — and it is unsourced, which under the claim-bearing-copy rule
   settles it.
2. **That a signature is valid, or verified, or checked.** **The
   REASON changed on 2026-09-05 and the RULE did not, which is the only
   reason this entry is still here.** It used to rest on the engine's own
   opening line, *"This module verifies nothing"* — and that stopped being
   true at `pdfcer-core` v0.38.0 (`b01964f`), where
   `signature::verify_all_with_trust` does compute the digest and does walk
   the chain. What survives is a **scope** rule rather than a capability
   one: the checking is reported by [`crate::panels::signatures`], as three
   separate labelled facts, and **nothing in this file may borrow that
   verdict**, because every string here is drawn by a save dialog that has
   looked at byte ranges and nothing else. [`verifies_nothing`] is the
   footnote that says so, and since 2026-09-05 it also names the surface
   that has the other answer instead of implying there is none.

   ⇒ *This is the seventh recurrence of the project's most expensive
   pattern.* The prohibition was right; its stated justification was a
   dated citation of another crate, and it expired silently. A rule that
   outlives its reason keeps passing its own tests.
3. **"Author signature", or any resolution of it.** Table 234's seed value
   `MDP /P 0` defines *"an author signature"* to mean an ordinary approval
   signature, while §12.8.2.2.1 uses *"the author of a document"* to mean
   the certifier — two incompatible uses of one word in one clause family.
   The engine uses neither and says so; neither does this file. The words
   used here are **certification signature** (the engine's own term, from
   §12.8.1) and, for everything else, plainly *signed*.

## Why the counts are spelled out rather than `(s)`

[`crate::text::compact::signature_line`] writes *"{count} digital
signature(s)"*, and this file deliberately does not follow it. That form is
readable as a template rather than as a sentence, and this copy is read at
the moment an operator is deciding whether to accept an irreversible
change — which is the worst possible moment for a surface to look
unfinished. Each string below branches on the count and reads as English in
both directions. The cost is one `if` per sentence; the divergence from the
neighbouring module is recorded here so it reads as a choice.

## Voice

The catalog's standing conventions apply — sentence case, full sentences
with punctuation for prose, name the thing and what the operator can do.
One addition specific to this area: **no exclamation, no capitals, no
"warning".** The one sentence in this shell that shouts —
`text::compact::signature_line`'s *"CANNOT keep them"* — earns it by
describing a loss that cannot be repaired by any later act. A save that
invalidates a signature is not in that class: the file the operator started
from is still on disk (a copy) or still contains its earlier revision (an
in-place incremental save), so the situation is recoverable and the copy
should not imply otherwise.

## Item notes

### `fn the_preserved_note_pairs_the_fact_with_the_uncertainty`

The one assertion in this file that is about a *prohibition* rather
than about content, and it is the engine's own: a front end that
renders `ByteRangePreserved` as "the signature is still valid" commits
the specific error §12.8.2.2.2's two-stage split exists to prevent.

Asserted as the presence of the negation rather than as the absence of
the phrase, because the absence is the weaker property: a rewrite that
dropped the qualifying half would still contain no such phrase and
would still be a reassurance by implication. What makes the sentence
safe is that it *names the open question*, so that is what is checked.

### `fn the_two_footings_are_worded_differently`

`SignatureImpact::documentation_basis` exists *"because the two deserve
different operator-facing wording and a front end cannot tell them
apart from the variant alone"*. A build that called the same function
for both would satisfy every other test in this file, so the divergence
is asserted directly.

The second half is the substantive one: the word *invalidate* may
appear as an assertion about the future only where Table 254 supports
it. `basis_approval` names the verdict too, but attributes it — hence
the check is on the **headline**, which is the sentence read first and
often alone.

### `fn the_approval_footing_attributes_the_verdict_to_pdfcer`

The engine's headline negative result, guarded. A sentence that
reported `Invalidated` for an approval signature without saying that
the standard is silent would be pdfcer citing a clause that does not
exist — and it would read as more authoritative than the certified
case, which is exactly backwards.

### `fn nothing_here_claims_what_another_reader_will_say`

The engine names the claim and forbids citing it: that Acrobat and the
PAdES family report such a document as *"signed, but altered since
signing"* is empirical tool behaviour, explicitly not sourced in the
spec RAG. It is the single most tempting sentence to add here, because
it is the most useful thing an operator could be told — which is why it
is guarded by a test rather than by a paragraph.

### `fn no_string_claims_a_signature_is_valid_or_was_verified`

`pdfcer-core`'s signature module opens *"This module verifies nothing."*
So the affirmative forms are forbidden outright and the negative ones
are the point — which is why this test hunts **phrases** rather than
the word *valid*. The word itself is unavoidable: *invalidate* and
*invalidated* are the verdict's own vocabulary and contain it, and a
substring check would fail on the two sentences the file most needs.

The forbidden list is therefore the set of ways a sentence could assert
the thing pdfcer has not established, plus *verified* and *we checked*,
which claim an act that never happened.

### `fn the_counts_read_as_sentences`

The `(s)` form this file deliberately does not use would pass a test
that only checked the count appeared, which is why the assertion is
that the two forms **differ** and that neither carries the template
marker.

### `fn the_in_place_sentence_names_the_file`

The one string here that takes an operand, and the operand is the whole
point: *"this writes over your file"* is a different statement from
*"this writes over `D:\jobs\4471\Sheet 1.pdf`"* to an operator with
four documents open, and the tab strip makes that the ordinary case.

### `fn window_title`

Deliberately **neutral about the verdict**, and it is the one string here
that does not branch on the basis. A title is read first and out of
context — it is also what the taskbar entry shows — so it states the fact
that is true on both footings and leaves the verdict to the body, where the
sentence that qualifies it is one line away rather than a window away.

*"This document is signed"* is also the fact an operator is most likely not
to know. A drawing that arrived by email carries no visible mark of it in
this shell's canvas, and the Signatures panel is a dock tab they may never
have opened.

### `fn headline_certified`

It asserts the outcome flatly, because here pdfcer can. §12.8.1 makes a
signature a certification signature when its `/Reference` array holds a
signature-reference dictionary whose `/TransformMethod` is `/DocMDP`, and
Table 254's permitted-change lists are **closed** — *"other changes shall
invalidate the signature"* is a `shall`, with no minor-change tolerance.
The engine works pdfcer's operations against that table and concludes that
none of them is on the permitted list at any `/P` value. So *"will"* is the
standard's own modality and not this catalog's confidence.

### `fn headline_approval`

It states the **change**, not the verdict, and that asymmetry with
[`headline_certified`] is the whole point of having two headlines.

The verdict here is pdfcer's cautious one rather than the standard's: for an
approval signature with no `/Reference`, ISO 32000-1 defines validation in
exactly one sentence — *"A signature shall be validated by recomputing the
digest and comparing it with the one stored in the signature"* — which is
stage 1 and only stage 1, and the engine's RAG confirms the **absence** of
any clause saying a post-signing revision invalidates such a signature.

A headline reading *"Saving will invalidate…"* would therefore put a claim
pdfcer cannot source in the largest type in the window, which is exactly
where a reader stops. So the headline says the part that is
incontrovertible — the document is being changed after it was signed — and
[`basis_approval`], one line below, gives pdfcer's verdict together with the
fact that it is pdfcer's.

### `fn basis_certified`

The `ImpactBasis::SpecSourced` sentence: a statement of fact, phrased as
one. Every clause in it is lifted from the engine's module documentation —
the certifier's list, its closedness, and the finding that no pdfcer
operation is on it.

It says *"the person who certified it"* rather than *"the author"*. See
the header's third prohibition: the standard uses "author" for both parties
in adjacent clauses, and this shell must not silently pick one.

### `fn basis_approval`

**The most carefully worded string in this catalog**, and the one that
most repays reading the engine's module documentation before editing.

It has to do three incompatible-looking things at once:

1. **Report pdfcer's verdict**, which is `Invalidated`. Not reporting it
   would be the under-reporting the engine names as the worse error.
2. **Not attribute that verdict to the standard**, because the standard is
   silent. The engine is explicit that this arm rests on *"a product
   decision under rule 4 (fuzzy-never-sneaky), not a spec citation"*.
3. **Not predict another reader's behaviour**, which is the unsourced claim
   the engine forbids citing.

The sentence that satisfies all three is the engine's own asymmetry stated
plainly: pdfcer would rather over-report a reviewable hint than make a
silent claim about a legal artifact. An operator reading it learns both
what pdfcer thinks and how much weight to give it, which is more than either
half alone would tell them.

### `fn target_in_place`

Named because the two save paths differ in the one way an operator cares
about at this moment: whether the file they already have survives. This one
writes over it.

### `fn target_copy`

Lifted deliberately close to [`crate::text::compact::signature_line`]'s
closing sentence — *"Your original file keeps its signatures."* — because
that sentence already ships, is already true of a command that always
writes a new file, and two different phrasings of one guarantee is how two
surfaces come to be read as promising two different things.

### `fn proceed_certified`

It **names the destructive act**, which is `crate::dialogs::unsaved`'s
standing rule for this crate: *"a destructive button says the destructive
thing, so that an operator who reads only the buttons — which is most
operators, most of the time — cannot get it wrong."*

It is safe to name it here and not on the other footing, and that
difference is the rule this catalog runs on: **the button may assert
exactly as much as the evidence does.** Table 254 supports *"invalidate"*
as a statement of fact; nothing supports it for a plain approval signature.

### `fn proceed_approval`

*"Save anyway"* rather than *"Save and invalidate the signature"*,
deliberately, and see [`proceed_certified`] for the rule. The word *anyway*
carries the whole of what pdfcer can honestly put on a button here: there is
something to weigh, the operator has read it, and they are proceeding. It
does not assert an outcome pdfcer cannot source.

It is also the conventional label for exactly this gesture, which matters:
the operator's standing instruction is to use the conventional interaction
rather than invent one, and a proceed-past-a-warning button reading
anything else would be a novelty in a dialog whose whole job is to be
instantly legible.

### `fn cancel_button`

*"Cancel"*, matching `crate::text::unsaved::cancel_button` and every
other confirmation in this crate. The non-destructive answer is the one an
operator presses reflexively to make a surprise go away, and it must wear
the label that reflex expects.

### `fn verifies_nothing`

The single most important sentence in this file, and it is in the
smallest type — because its job is not to be read in this window but to be
available when an operator wonders what pdfcer actually knows.

The engine's module documentation opens with it: **"This module verifies
nothing.** It computes no digest, parses no PKCS#7 blob, and validates no
certificate chain." Everything this shell says about signatures is
arithmetic over where bytes are, plus a reading of a permissions
dictionary. Without this sentence, an operator who sees pdfcer speak
confidently about a signature here will reasonably conclude that pdfcer's
*silence* elsewhere means it looked and found nothing wrong — and it never
looked at all.


The sentence used to read *"pdfcer does not check any signature's
certificate or its cryptography."* That was a claim about the **build**,
and it became false the day `signature::verify_all_with_trust` was wired
(`crate::trust::examine`, consumed by `crate::panels::signatures`, which
draws integrity, coverage and trust as three separate labelled lines).
Engine `pdfcer-core` v0.38.0 at `b01964f`; measured against that lock, not
recalled.

What is still true is the narrower thing this window needed all along:
**this window** has not checked anything — it is arithmetic over byte
ranges — and the Signatures panel is where the checking is reported. So
the sentence is now scoped to its own surface and points at the one that
carries the other answer, which is what it should have said when it was
written. ⇒ *A sentence about what the program cannot do is a dated
citation; where it can name the surface it is true of, it should.*

### `fn preserved_note`

⚠️ **This string is the reason `SignatureImpact::ByteRangePreserved` has a
surface at all, and it must never become a reassurance.** The engine's
variant documentation:

> ⚠️ **This is not "the signature is still valid."** Stage 2 — whether the
> changes are ones the signer permitted — is a separate question this
> variant makes no claim about. A front end that renders this as a
> reassurance is committing precisely the error §12.8.2.2.2's two-stage
> split exists to prevent. **Pair it with the uncertainty, or say nothing.**

This shell pairs it. The sentence is built in exactly that order — the
stage-1 fact, then the word *but*, then the stage-2 question named as
unanswered — so that a reader who stops halfway has read the fact and not
yet reached a conclusion, rather than the reverse.

## Why *pair it* was chosen over *say nothing*, which was permitted

Both are allowed and the choice is this shell's. Three reasons, in order of
weight:

1. **This shell has already made the operator a promise adjacent to it.**
   `file.save_copy`'s shipped tooltip says the edits *"are appended as an
   update so the previous version stays intact inside the file"*, and the
   Save-a-compacted-copy window says in as many words that a rewrite
   **cannot** keep signatures while the original file does. An operator who
   has read those two surfaces has been handed exactly the premises from
   which the folk conclusion — *appended, therefore my signature is fine* —
   follows. Silence here leaves that inference standing, and it is wrong.
2. **Rule 4.** The operator cannot see this anywhere else: no mark appears
   on the canvas, and the Signatures panel reports what the document
   carries rather than what a save did to it.
3. **It is cheap and it is rare.** It costs one line on a row that already
   exists, only for a document that actually carries a signature — which is
   the same conditional-rarity argument `crate::text::compact` uses to
   justify showing its own signature sentence only when it applies.

### `fn invalidated_note`

It exists for a path where it is the **only** disclosure, and that is
why it repeats what the window said rather than assuming the window was
seen. `crate::app::lifecycle::resume_after_unsaved` writes a copy from
inside an already-answered question, and this shell does not stack a second
modal on that gesture — see `crate::dialogs::signature`'s header for the
argument. On that route this note is the whole of what the operator is
told, so it has to stand alone.

It says *"pdfcer reports"* rather than *"this invalidated"*, and it says
so on **both** footings. A post-hoc receipt is read quickly and out of
context; splitting it in two the way the window's copy is split would put
the more careful wording on the less careful reading. Attributing the
verdict to pdfcer is true when the basis is a spec citation (pdfcer is
reporting what the standard says) and necessary when it is not, so one
sentence serves both without over-claiming on either.
