# `panels::signatures` — three facts about each digital signature, reported
separately and never merged


## THE RULE THIS PANEL EXISTS TO KEEP

**This is the one place in the product where a wrong answer is worse than no
answer.** A panel that said *"trusted"* about a chain it had not really
validated would be the exact failure mode the engine's design prevents — and
the engine prevents it by refusing to produce one verdict at all:
`SignatureVerdict` carries `integrity`, `coverage` and `trust` as **three
facts that never collapse into one bool**.

So this panel draws three labelled lines per signature, always all three, in
that order. There is no badge, no tick, no colour that means "fine", and no
arithmetic anywhere in this file that combines two of the facts into a
third. The order is also the order of increasing uncertainty: integrity is
arithmetic, coverage is arithmetic, trust is a judgement about the world.

And [`pdfcer_core::signature::Trust::NotChecked`] renders **as itself**.
Not as a soft "no", not as a grey tick, not omitted. Hiding it would make
this panel indistinguishable, on screen, from one that had checked and found
nothing wrong — which is the inversion the whole feature exists to prevent.
[`crate::text::trust`] holds four separate sentences for the four reasons
trust can go unchecked, because *"you turned it off"* and *"your Acrobat
store is corrupt"* are opposite calls to action.

## What changed at the rewrite, and what deliberately did not

| | before | now |
|---|---|---|
| integrity | not computed; the panel said so in its first line | `verify_all_with_trust` |
| coverage | `byte_range_coverage`, per frame | unchanged — it is the same measurement, now printed beside the other two |
| trust | not computed | the operator's own Acrobat anchors, opt-in, off by default |
| the leading caveat | *"pdfcer does not check whether these signatures are valid — it cannot yet"* | replaced: that sentence became FALSE the moment this was wired |

That last row is this project's most expensive recorded failure shape —
a claim that was true when written and false within hours, with the prose
around it still true. The replacement
([`crate::text::trust::panel_intro`]) names the three facts instead of
denying one of them, so it cannot go stale the same way: it describes the
SHAPE of the report rather than the limits of a build.

## The file length, and the bytes, are read from DISK

`/ByteRange` is a claim about bytes, so it can only be checked against
bytes — which is why
[`pdfcer_core::signature::byte_range_coverage`] takes the length as a
parameter rather than reading it from the object graph: *"the object model
cannot check a claim about bytes against itself."* Verification needs the
same thing one step further: the real file, digested.

What is used is the file **on disk right now**, not a length captured when
the document was opened, and not the session's rendering of it. It answers
*"do these signatures hold over the file as it currently exists"*, which is
the question worth asking. An unsaved edit is not in the file and cannot be
covered by a signature; the panel says which state it measured rather than
leaving an operator to assume.

## Why verification is automatic, and what stops it being per-frame

The alternative considered was a *Check signatures* button. It was refused:
an operator who has opened a panel called Signatures has already asked, and
a button would leave the panel's resting state showing coverage numbers with
no integrity beside them — which is precisely the state this work exists to
end.

What makes that affordable is [`crate::trust::cached_report`], whose key
carries the file's identity, its length, its modification time and **both**
halves of the trust configuration. A cache that missed any of those would be
worse than none: the panel would show a verdict about a file, a setting or
an anchor set that is no longer in force, and it would look exactly like a
correct answer.

## This panel raises no actions, and cannot

Nothing about a signature is editable from anywhere in pdfcer — the engine
cannot sign, and this shell would have nothing to send it if it could. The
`actions` parameter is present because the dock calls every panel through
one signature; this one never pushes to it.

## Item notes

### `fn integrity_line`

`None` means the file could not be read at all, which is neither a pass nor
a failure and is reported as the coverage half already reports it: as
pdfcer's inability to look, not as a statement about the document.

### `fn coverage_line`

The two malformedness reports come FIRST, because they change what the
coverage numbers mean: a reader that rejects the array computes something
else, or nothing.

### `fn trust_line`

The four `NotChecked` sentences are chosen from the [`Anchors`] state
rather than from the verdict, and that is the whole design. The engine
reports `NotChecked` identically whether the operator opted out, has no
store, typed a wrong path, or has a corrupt store — it cannot know which,
because it was simply handed no anchors. This shell DOES know, because it is
the half that looked, and reporting all four as one sentence would be this
panel discarding the only thing it can contribute.

### `fn why_not_checked`

`None` — no report at all — is the never-saved document, and it takes the
opted-out sentence deliberately: there is no file to verify, the anchors were
never consulted, and inventing a fifth sentence for a case the coverage half
already reports would be two surfaces explaining one absence.

### `fn anchor_provenance`

Drawn only when there ARE anchors. The three no-anchor states are explained
per signature instead, on the trust line, because that is where an operator
is asking the question — and a store disclosure above a list of signatures
whose trust was never checked would be a header about nothing.

### `fn labelled`

A `horizontal_wrapped` rather than a `format!("{label} {said}")`, so the
three labels line up as a column and the sentences wrap under themselves.
The alignment is not decoration: three facts printed as three unlabelled
paragraphs is three facts a reader has to sort out, which is the first step
back towards reading them as one verdict.

### `fn field_token`

`none` rather than an empty string, matching every other token helper
here: an empty value after `=` is indistinguishable from a truncated line,
and a check cannot tell the two apart.

### `fn integrity_token`

Not operator copy and not in the catalog: a driven check matches on it,
and a check that matched translated prose would break the day the prose
improved. Same argument as `crate::trust`'s anchor token.

### `fn an_unreadable_file_is_not_reported_as_an_unsigned_one`

The first is a statement about the document. The second is a
statement about pdfcer's ability to look, and rendering it as the
first would be a claim about the operator's file made from an
inability to read it.

### `fn the_intro_promises_three_separate_facts`

This replaces `the_caveat_denies_validity_checking_explicitly`, which
asserted the OLD caveat's *"does not check … valid"* wording. That
wording became false the moment `verify_all_with_trust` was wired, and a
test pinning it would have kept a false sentence on screen while passing
— which is this project's most expensive recorded failure shape wearing
a green tick.

What is asserted instead is the property that does NOT expire: the panel
tells a reader, before they start, that there are three answers and that
pdfcer will not fold them into one.

### `fn every_unchecked_state_produces_a_sentence_that_says_not_checked`

The assertion that stands between an operator and a silent grey row.
Driven over the four anchor states plus the never-saved case, through
the same function the panel calls — so a branch added to
[`why_not_checked`] without a sentence fails here rather than rendering
an empty label.

### `fn the_reason_trust_was_not_checked_is_specific_to_the_state`

[`why_not_checked`] is a `match`, and a `match` with two arms returning
the same string compiles perfectly. This is what refuses that: the four
call for four different actions — turn the setting on, install Acrobat,
fix your typo, your store is corrupt — and telling somebody the wrong
one sends them to the wrong place.

### `fn the_trace_tokens_tell_the_verdicts_apart`

A driven check reads `integrity=` and `trust=` off the row line. Two
variants sharing a token would make a check that asserts *"the trust
verdict changed when the setting was turned on"* pass against a build
where it did not.
