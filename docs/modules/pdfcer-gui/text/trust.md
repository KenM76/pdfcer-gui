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
