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
