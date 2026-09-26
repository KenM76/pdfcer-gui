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

### `fn panel_intro`

That sentence had to go the moment `verify_all_with_trust` was wired,
and this project's most expensive recorded failure is exactly a claim like
it going stale while the prose around it stayed true. What replaces it is
**not** a reassurance: it names the three facts, in the order the rows print
them, so a reader knows before they start that there are three answers and
that one of them may be *not checked*.

### `fn integrity_label`

A shared prefix rather than three sentences that each happen to mention
integrity: the three facts are read as a column, and a column with a ragged
left edge is one an operator scans instead of reads.

### `fn integrity_verified`

It names the algorithms rather than saying "yes", and that is
[`pdfcer_core::signature::Integrity::Verified`]'s own instruction: the two
fields are carried *"so a shell can disclose a SHA-1 signature as
verified-with-a-weak-digest rather than hide it"*. A shell that printed
"verified" alone would be discarding the one field that distinguishes a
modern signature from one nobody should rely on.

### `fn integrity_weak_digest`

The engine reports SHA-1 in its `notes` and does not downgrade the verdict,
which is right: the signature genuinely verifies. This shell repeats the
fact where the verdict is read, because *"verified"* and *"verified with a
digest that has been collision-broken since 2017"* are different things to
act on.

### `fn integrity_digest_mismatch`

The one string in this file that states a loss. Worded as the engine words
it — the digest does not match, so the bytes changed — and deliberately not
as *"the signature is invalid"*, because that phrase folds integrity, trust
and coverage into one word and is the exact collapse this feature refuses.

### `fn integrity_signature_invalid`

A genuinely different fault from a digest mismatch and the engine keeps them
apart, so this does too: the document's covered bytes are what was signed,
and the signature, the certificate or the signed attributes were tampered
with instead.

### `fn integrity_unverifiable`

`reason` is passed through **unedited**. The engine promises this case is
*"never reported as either of the other three"*, and it names each cause
precisely — an unimplemented subfilter, `adbe.x509.rsa_sha1`, RFC 3161,
P-521, Brainpool, a malformed CMS, a hole that does not fit the range. A
shell that paraphrased would produce a second, vaguer vocabulary for faults
the engine already names exactly.

### `fn trusted`

**The single most dangerous string in this application**, and the reason
it is long. It states four things in one sentence because separating any of
them would leave the good news standing alone:

1. the chain reached a trusted anchor, **by verified signatures**;
2. **which** anchor, by subject — an operator who does not recognise the
   name has learned something a tick could not tell them;
3. its provenance (`AATL`/`EUTL`/`ADBE`), because those are three different
   programmes with three different admission bars;
4. **what was not checked** — revocation always, and validity dates when the
   signature carried no signing-time clock.

Point 4 is not a hedge. `PathChecks::revocation_checked` is `false` on every
verdict this build can produce, and a certificate that was revoked the day
after it was issued chains exactly as well as one that was not.

### `fn untrusted`

*"Valid but untrusted"* is a real and common state — a self-signed
certificate, a corporate CA nobody added to Acrobat — and this sentence says
so, because an operator who reads "untrusted" beside an intact signature will
otherwise conclude the document was tampered with. The engine's own reason
is carried through unedited.

### `fn signer_unknown`

The engine keeps this apart from `Untrusted` and so does this. *"pdfcer could
not read the certificate"* and *"pdfcer read the certificate and does not
trust it"* are opposite findings, and only the second says anything about the
signer.

### `fn not_checked_prefix`

Its own function, and every caller of the four `not_checked_*` sentences
goes through [`not_checked`], so the words *"Not checked"* cannot be dropped
from one branch by a well-meaning edit that shortened it.

### `fn not_checked`

The `why` half is supplied by one of the four functions below. They are kept
separate from the prefix so that a test can assert **every** one of them
starts with the same three words — see this module's tests.

### `fn not_checked_opted_out`

Names the remedy and where it is, because this is the one of the four states
the operator can fix in five seconds and will otherwise assume is a missing
feature.

### `fn not_checked_configured_missing`

Not the same sentence as [`not_checked_no_store`], and the separation is
the point: this person did not fail to have a store, they made a typo, and
telling them their machine has no trust list would send them looking in
entirely the wrong place.

### `fn store_line`

See this module's header for why the date is not separable from the count.
`modified` is already formatted by [`crate::trust::modified_date`]; a `None`
says the filesystem would not give a date, which is itself worth printing
because a store whose age is unknown is not a store known to be current.

### `fn store_undecodable`

Only drawn when non-zero. Surfaced rather than swallowed because an operator
whose signer happens to be one of the refused entries would otherwise see an
inexplicable *"does not chain"* with nothing to look at.

### `fn at_own_risk`

Translated from `pdfcer_core::settings::AcrobatTrustStore`'s own type
documentation and from the CLI's identical warning, deliberately: two front
ends wording one legal limitation differently is worse than either wording,
and this is the sentence a person would quote back at us.

### `fn use_store_silence`

**This one is not a spec silence and the sentence says so**, exactly as
`quad_point_order`'s does. ISO 32000-1 is perfectly clear that validation
has a trust leg; what it does not do — and cannot — is tell a program which
certificates a particular person trusts. That is a fact about the operator,
not about the format, and there is no public machine-readable bundle of the
lists that matter.

### `fn use_store_on_label`

The label spells *"at your own risk"* because the engine's own persisted
token does — `acrobat_trust_store = at_own_risk` — and an operator who opens
`settings.txt` must find the same words they clicked.

### `fn resolved_found`

Reported as of the last time pdfcer looked, which is every frame this
group is drawn — a `stat`, not a read. That differs from
`crate::text::acrobat::resolved_note`, which cannot update as you type
because resolving an Acrobat spawns processes. Locating a file does not, so
this line **is** live and the field's mistakes are visible where they are
made.

### `fn inspect_button`

**This control is drawn only when a store was actually found**, which is
R9: an unavailable capability renders nothing, and greying is reserved for
something that is *temporarily* unavailable. A person with no Acrobat store
is not one press away from having one. The path field above stays visible in
that case, because it is the remedy — and R9's rule cuts both ways: an
absent capability whose remedy is also absent is a dead end.
