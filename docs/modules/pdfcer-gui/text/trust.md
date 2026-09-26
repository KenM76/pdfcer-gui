# `text::trust` — every word this shell says about whether a signature can
be trusted

The catalog area for [`crate::panels::signatures`] and
[`crate::dialogs::settings::signatures`]. Its subject is the one place in
the product where **a wrong answer is worse than no answer**, so it carries
rules the rest of the catalog does not.

## THE FOUR RULES THAT GOVERN EVERY STRING IN THIS FILE

### 1. Nothing here may be invented

`crate::text::signature`'s header states this for the *save-time* half of
the subject and it binds this file identically: this is claim-bearing copy
about a security property of a legal artifact, and the engine that computes
the verdict has already written down at length which claims are supportable.
Every sentence below is a translation of a distinction `pdfcer-core`'s
`signature_verify` and `trust_chain` modules draw, **without softening it and
without strengthening it**.

Read `D:\Dev\pdfcer\crates\pdfcer-core\src\trust_chain.rs`'s module
documentation before changing a word. Its "Security posture" section is what
[`trusted`] is a translation of, clause for clause.

### 2. The three facts never collapse

`SignatureVerdict` carries `integrity`, `coverage` and `trust`, and the
engine's own design note is that they *never collapse into one bool*. So
there is no `verdict()` function in this file, no composite sentence, and no
badge. Three labelled lines, always all three, in that order — which is also
the order of increasing uncertainty: integrity is arithmetic, coverage is
arithmetic, trust is a judgement about the world.

### 3. `NotChecked` renders as itself

Never as a soft "no", never as a grey tick, never omitted. [`not_checked`]
and its three siblings all begin with the words *"Not checked"* before they
explain which of the four situations applies. A surface that hid the
unchecked case would be indistinguishable, on screen, from one that had
checked and found nothing wrong.

And the four explanations are four sentences rather than one. *"You have
this turned off"*, *"this machine has no Acrobat trust list"*, *"you pointed
at a file that is not there"* and *"the list is there and pdfcer could not
read it"* are four different calls to action, and only one of them is
*"nothing is wrong"*.

### 4. A `Trusted` verdict carries what it did NOT check, in the same sentence

Not in a tooltip, not in a footnote at the bottom of the panel. `Pass 10.5`
checks chain linkage, RFC 5280 CA/key-usage constraints and — only when a
signing-time clock exists — certificate validity dates. It does **not** check
revocation, and cannot: CRL and OCSP need the network `pdfcer-core` never
touches. The engine's own note on every `Trusted` verdict says so, and
[`trusted`] says it where the operator is reading the good news, because a
qualification separated from its claim is one nobody reads.

## Why the store's DATE is in the same sentence as its size

`ENGINE_BACKLOG.md`'s own argument for the whole feature:

> an anchor set that silently went stale is worse than one that was never
> imported.


## Voice

The catalog's standing conventions, plus one borrowed from
[`crate::text::signature`]: **no exclamation, no capitals, no "warning"**.
The one place this file raises its voice is [`integrity_digest_mismatch`],
which describes bytes that were altered after signing — a fact about the
operator's document that nothing they do now can undo, which is the same
class `text::compact::signature_line` earns its shout in.

## Item notes

### `fn every_unchecked_trust_sentence_says_not_checked`

The property this whole feature stands on, asserted over the sentence
rather than trusted to the layout. All four branches go through
[`not_checked`], so the words cannot be dropped from one of them — and
this test would catch it if the funnel were bypassed, because it checks
each explanation through that funnel.

⚠ **When this fails, the fix is the sentence, not this test.** A softer
wording — "trust unavailable", "no trust information" — is precisely the
drift it exists to refuse: those read as *nothing is wrong*, and the
whole point is that pdfcer has not looked.

### `fn the_four_reasons_trust_was_not_checked_are_four_sentences`

Not a tautology: the cheap implementation of this feature has one
"trust was not checked" string and four call sites, and it would pass
every other test in this file. The four situations call for four
different actions — turn the setting on, install Acrobat, fix your typo,
your store is corrupt — and collapsing any two of them tells somebody to
do the wrong thing.

### `fn a_trusted_verdict_never_claims_more_than_the_engine_checked`

The engine attaches that disclosure to every `Trusted` note it produces
and the whole design rests on the shell not dropping it. This is the
assertion that stops a future edit shortening the sentence to *"trusted
— chains to X"*, which is what every other PDF reader says and is the
one thing pdfcer must not say without the qualification.

### `fn untrusted_separates_itself_from_integrity`

The single likeliest misreading on this surface: an operator sees
`Signer: does NOT chain…` beside an intact signature and concludes the
document was altered. The sentence has to separate the two claims
itself, because it is read alone.

### `fn the_store_is_never_described_without_its_age`

`ENGINE_BACKLOG.md`: *"an anchor set that silently went stale is worse
than one that was never imported."* There is deliberately no accessor
that yields the count without the date, and this asserts the one that
exists carries both — including the honest sentence for a store whose
date could not be read.

### `fn the_three_facts_are_labelled_apart`

Cheap, and it is what stops a tidy-up merging two of the columns. The
engine's design note is that the three never collapse into one; a shared
label is the first step of collapsing them.
