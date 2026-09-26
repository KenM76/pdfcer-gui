# `text::embed` — what the Embed-fonts window says before it changes
anything

The copy for [`crate::dialogs::embed`].

## This window exists to be READ, not to be filled in

⚠ **Corrected 2026-09-05.** This paragraph read *"It has no settings … there
is no useful way to make it configurable either"*, and that sentence was
used in `OPERATOR_REQUESTS.md` **O47** as the reason not to let the operator
decide whether pdfcer's own standard-14 faces may stand in for his. The
window now has exactly one control, and the reasoning that kept it out was
wrong rather than merely outdated — `dialogs::embed`'s header carries the
whole account.

What survives, and still governs every string in this file: `embed_fonts`
takes a request the shell has already resolved, so almost every word here is
a **report of what would happen** rather than a field to fill in, and the
window is a confirmation rather than a form. The one control is a *consent*,
not a configuration: it changes what the report says, and the report is
still what the operator is reading.

That shape is chosen because of what an embed is: it puts font **programs**
into a document permanently, changes its size, and can invalidate a PDF/A
claim. There is no honest way to offer that as a one-click ribbon verb.

## The three things it must say, in this order


**1. What will be embedded**, because that is the operator's answer.

**2. What will NOT be, and why, per font.** The engine's `EmbedBlocker` has
eight variants and they mean very different things — a Type 3 font cannot be
embedded at all, a font with no donor needs a folder, a composite needs a
different verb. Collapsing them to *"3 fonts could not be embedded"* would
throw away the only part an operator can act on.

**3. The PDF/A claim.** A document identifying as PDF/A that gains an
unembedded-to-embedded change is a document whose claim may no longer hold,
and `pdfcer-core` is explicit that it is **choosing** this disclosure rather
than matching Acrobat — parity there is an open research gap. A choice made
on purpose is one this shell repeats rather than quietly drops.

## Item notes

### `fn every_blocker_reads_as_a_sentence_a_person_can_act_on`

The failure this guards is a reason that leaves somebody trying things:
*"composite font"* is true and tells a person nothing about what to do
next. Each sentence below either points somewhere (a folder, Settings) or
closes the question (*"there is nothing to embed"*).

### `fn a_font_pdfcer_carries_is_offered_its_own_copy_and_one_it_does_not_is_not`

This is the assertion that catches the wording defect the switch
created. Before 2026-09-05 pdfcer's own faces answered
unconditionally, so a font pdfcer carries could never be reported as
*"pdfcer has nowhere to take it from"*; with the box unticked it can be,
and the old sentence would have told the operator pdfcer has no copy of
a font it is holding.

The two are asserted **against each other** rather than against
literal strings. A test pinning the exact sentence would fail every time
somebody improved the wording, which trains people to update the
expected string without reading it. What must never happen is the two
cases producing the same sentence, and that is what is checked.

### `fn the_offer_names_every_font_it_would_stand_in_for`

The property is that every face in the list appears in the sentence.
A count is what a hurried implementation produces and it is exactly what
makes the disclosure useless: an operator cannot decide whether he minds
a substitution without knowing which font is being substituted.

### `fn the_consequence_states_the_look_and_the_licence`

The letterform change is obvious enough that anybody writing this
sentence would include it. The licence condition is the reason
`pdfcer`'s own CLI keeps the equivalent switch off by default, it is the
half that binds the operator rather than his reader, and it is the half
a later edit tightening the wording would drop first.

### `fn no_claim_means_no_line`

The one that matters: this window opens on every drawing, and a line
saying *"this is not a PDF/A"* on all of them is noise that trains an
operator to stop reading the window.

### `fn the_disclosure_says_only_what_is_true`

The failure this guards is the fixed sentence: a line reading
*"0 still missing, 0 substituted"* on every ordinary embed is a line an
operator learns to skip, and the day one of those numbers is not zero it
is skipped too.

### `fn the_size_is_a_bound_and_never_a_prediction`

`bytes_added_uncompressed` is always larger than what lands on disk -
the writer deflates every program stream - so a sentence phrased as a
prediction would be wrong on every single embed. The word `most` is the
whole assertion.

### `fn every_rung_says_something_different_and_bundled_says_the_most`

`OPERATOR_REQUESTS.md` **O47** was answered *"yes"* — pdfcer may use its
own faces — and the condition attached to that answer was *disclosed
loudly*. The failure this guards is the quiet collapse: four rungs
rendering as two, so a document that went out with pdfcer's stand-in in
it reads on screen exactly like one carrying the operator's own Arial.

### `fn intro`

It states the **permanence** and the **size**, which are the two
consequences an operator cannot see from a list of font names. Embedding is
undoable in this session; it is not undoable in a file somebody has already
been sent.

### `fn embed_row`

It names the **source**, not just the face. Two files on a machine can
advertise one name and produce visibly different letters, and an operator
embedding into a drawing they will send out is entitled to know which one is
going in.

Three rungs, three sentences, and the collapse to two would be the
defect. `FontMatch`'s own doc calls them *"three materially different acts:
honouring a name the file already spells, applying a well-known family
equivalence, or falling back to a face pdfcer ships"* — and the third is the
one an operator would most want to know about and least expect, because
nothing they configured produced it.

### `fn blocked_row`

Every arm names **what would fix it**, or says plainly that nothing
will. A reason with no remedy and no closure is a sentence that leaves
somebody trying things.

`#[non_exhaustive]` on the engine's enum means a ninth blocker is
possible, and the catch-all says *"pdfcer would not embed it"* rather than
inventing a reason — the same posture `TextColor::Other` takes. A build
meeting a blocker it cannot name should say so, not guess.

### `fn own_fonts_offer`

**The offer: which fonts pdfcer could stand in for, by name.**

# A LIST, NEVER A COUNT

*"3 fonts would be substituted"* is a number an operator cannot act on.
*"Helvetica, Helvetica-Bold, Times-Roman"* is a sentence he can read and
answer — *"those are the title block, so no"*, or *"those are notes nobody
reads, so yes"*. The whole reason this control is safe to offer is that the
consequence is stated **before** the press, and a count does not state it.

The document's own spelling, subset tag and all, because that is the
string he saw in the Fonts panel and in whatever told him a font was
missing. Translating it to a tidier family name here would make the window
and the panel disagree about what the document contains.

### `fn own_fonts_consequence`

Both sentences are consequences he cannot see by looking at the drawing,
which is exactly the class rule 4 says an inference owes a report for.

1. **The letters change on somebody else's screen.** It is his drawing and
   his client's monitor, and a stand-in is a different face however good the
   metrics match — the page does not reflow, and every letterform differs.
2. **It is a licence he takes on, not just a look he accepts.** pdfcer's
   fourteen substitutes are BSD-3-Clause (`THIRD_PARTY_LICENSES.md`,
   *"Bundled Foxit substitute faces"*), and embedding one puts it inside a
   file he then sends out, carrying that licence's attribution condition
   with it. `pdfcer`'s own command line states this as the reason its
   equivalent switch is off by default: *"That is your decision to make, so
   pdfcer does not make it for you."*

⇒ The second is why this is a decision rather than a default. Written in
plain words, because "BSD-3-Clause attribution condition" is not a sentence
that helps anybody decide anything.

### `fn own_fonts_checkbox`

Phrased as what it does, not as what it is. *"Use pdfcer's own copies"*
answers *"what will happen if I tick this"*; a label like *"Bundled fonts"*
names an implementation detail and makes the operator work out the rest.

*"where none of yours match"* is in the label rather than only in the
prose above, because that clause is what makes the control safe: it is the
**last** rung, so ticking it can never displace a real font he owns. A label
without it reads as *"use substitutes instead of my fonts"*, which is not
what it does and is a reason to refuse it.

### `fn pdfa_line`

Returns `None` when there is no claim, because a document with no PDF/A
identification owes no sentence and a window that said *"this is not a
PDF/A"* to everybody would be noise on every ordinary drawing.

pdfcer is **choosing** this disclosure rather than matching Acrobat —
their own note says whether Acrobat warns about the same thing is an
unresolved gap in the parity research. A deliberate choice is one this shell
repeats rather than quietly dropping.

### `fn size_ceiling`

A **ceiling**, said as one. `bytes_added_uncompressed` is explicit that
the writer deflates every program stream and a face typically halves, so
this number is always larger than what lands on disk. Reporting it as a
prediction would make pdfcer wrong on every single embed; reporting it as a
bound makes it right on all of them, and it errs in the direction an
operator can absorb.

### `fn still_missing`

`missing_after` is *"the end state the whole feature exists to reach"*,
and the engine says so in those words. A window that reported only what it
embedded would read as success on a file a print service will still reject.
Returns `None` at zero — there is no sentence to write about a number that
has arrived.

### `fn still_missing_partly_unexplained`

Gated on `unexplained_missing`, and the engine's own docs demand exactly
this gate: under a named selection a font nobody asked about is neither a
target nor a refusal, so *"each one is listed below"* becomes a claim the
window cannot keep and an operator is sent looking for reasons that were
never printed. It is zero under `AllMissing` by construction — which is the
only selection this window sends today, and precisely why the wrong wording
would never have been caught here.

### `fn nothing_to_embed`

It points at the **evidence already on screen** rather than naming a
cause. The window is open precisely because there is a list, every row of
that list carries its own reason, and those reasons differ — one font needs
a folder, another is a Type 3 that never can be. A single hover sentence
that picked one of them would be wrong about the others; one that pointed
at the list is right about all of them and is two inches from the answer.

### `fn embedded_disclosure`

Three clauses and each is CONDITIONAL, which is the whole design. A
fixed sentence would either say nothing about the substitutions or say
*"0 substituted"* on every ordinary embed, and both train an operator to
stop reading the line.

- **What went in**, always.
- **What is still missing**, only when it is not zero — the number
  `missing_after` exists to drive to zero, and a report showing only what it
  embedded reads as success on a file a print service will still reject.
- **That a stand-in was used**, only when one was. This is Rule 4's
  surviving half in its purest form: substituting `Arial` for `Helvetica` is
  an inference the operator **cannot see** — the letters are metric
  compatible and the page looks right — so it is exactly the case that owes
  an off-canvas report.
