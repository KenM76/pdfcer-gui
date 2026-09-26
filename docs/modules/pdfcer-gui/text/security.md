# `pdfcer-gui/text/security`

## Item notes

### `fn password_prompt`

It names the FILE, because an operator who has opened four drawings and
walked away needs to know which one is asking. The prompt appears in answer
to their own Open, but not always in the same minute as it.

### `fn password_cancel`

*Cancel*, not *Close*: it abandons an attempt the operator started, and
the tab stays in the document list showing why it did not open. Nothing is
lost by pressing it.

### `fn password_rejected`

It says **which attempt** this was, and that is not decoration. Without
it, a second wrong password produces a dialog identical to the first, and an
operator who did not see the field clear cannot tell whether their press
registered at all — the same ambiguity the measure tools' running count
exists to remove.

It does **not** say how many attempts remain, because there is no limit:
pdfcer is reading a local file and rate-limiting the operator's own guesses
at their own document would be theatre.

### `fn password_needs_normalisation`

# Why this is not "wrong password", and why the engine made it a
separate error

`DocError::PasswordRequiresNormalisation` exists, in `pdfcer-core`'s own
words, *"so that failure does not masquerade as `PasswordRequired`'s 'you
typed it wrong', which would send the operator to re-check a password that
was correct."*

The mechanism: `/R` 5 specifies **SASLprep** (RFC 4013) over the password
before hashing, and pdfcer does not implement it — no stringprep dependency
was taken for a read-only increment. For an all-ASCII password SASLprep is
the identity, so this can only ever arise from a password containing
something else.

⇒ So the sentence tells the operator the true thing: **the password may be
perfectly correct**, and pdfcer cannot prove it either way. Sending them to
re-type it would be this program wasting their afternoon on its own
limitation.

### `fn not_encrypted`

Stated rather than left blank. *"This document is not encrypted"* is a
fact an operator checking a file wants confirmed; an empty panel is
indistinguishable from a panel that failed to load.

### `fn cipher_line`

The revision number (`/R`) is deliberately absent. It is the number that
matters to an implementer and means nothing to the person holding the
drawing; what they can act on is *how strong is this* and *is it modern*.

### `fn auth_line`

# The engine asked for this by name

From its 2026-09-03 reply: *"`AuthKind` tells you which one opened the file
— surface that, because `remove_encryption` will refuse a
user-authenticated session and the operator should see WHY before pressing
it."*

So this is not a curiosity. It is the precondition of a control that does
not exist yet, shown before that control arrives, so the day it does the
refusal is already explained.

[`AuthKind::EmptyUser`] is the case an operator never sees happen: the
document declares a user password of nothing, every conforming reader tries
it silently, and the file opens with no prompt. It is worth naming, because
*"this file is encrypted"* and *"you needed a password"* are then two
different facts and the operator has only observed one of them.

### `fn permissions_are_advisory`

Supplied by `pdfcer-core` on 2026-09-03 with the instruction *"take this one
verbatim; it is the sentence the CLI will print too."* Do not re-word it,
do not shorten it for a narrow column, and do not soften *"a request, not a
lock"* — that clause is the whole of what an operator needs to know and it
is the one a marketing instinct would file off.

### `fn permission_state`

Three states, not two. `Permissions::granted` returns `Option<bool>`, and
`None` means *the bit does not apply to this document's encryption
revision* — which is not "refused". Rendering it as refused would tell the
operator their document forbids something it has no opinion about.

### `fn perms_disagree`

# Reported, never acted on, and the engine is emphatic about why

`/Perms` holds an **encrypted** copy of `/P`. The plaintext copy sits in the
`/Encrypt` dictionary where anyone can edit it, with no integrity protection
anywhere else in clause 7.6. So a disagreement means somebody changed the
stated permissions after encryption — and **no clause says what to do about
it**.

`pdfcer-core` reports it and prefers neither value, keeping `/P` because that
is what the file declares and what every other viewer shows. Its own doc
comment names the rule: *"Silently substituting the decrypted copy would be
pdfcer deciding, on an inference, what the operator is told — the exact shape
project rule 4"*.

⇒ So this sentence states the disagreement and stops. It does not say the
document was tampered with, because pdfcer does not know that.

### `fn perms_not_applicable`

Said, and said as ordinary. `pdfcer-core`: *"`NotApplicable` for every
`/R` ≤ 4 document, where the entry does not exist. That is the ordinary
answer, not a failed check, and a front end must not render it as one."*

### `fn signature_not_verified`

Our draft said pdfcer *"cannot tell you the document is unaltered"*. The
engine asked for wording that **will not have to be unwritten** when its
integrity check ships — because *unaltered* is exactly what that check will
answer. So: *"does not yet check"*, and the clause that will change is
separated from the clause about trust, which will not.

**THAT DAY CAME — 2026-09-05 — and this sentence was false for it.**
The paragraph above said *"when `signature::verify` lands, the first two
clauses change"*. It landed: `pdfcer-core` v0.38.0 (`b01964f`) carries
`signature::verify_all_with_trust`, `crate::trust::examine` calls it, and
`crate::panels::signatures` draws integrity, coverage and trust as three
separate labelled lines. The clause *"It does not yet check the signature
itself"* was untrue from that moment.

⚠ **And nothing went red, because this function has ZERO call sites.** The
panel was built against [`crate::text::trust`] and [`crate::text::panels`]
instead, which orphaned this whole signature group — `signatures_heading`,
`not_signed`, `signature_count` and `coverage_line` are likewise
unreferenced. A dead string cannot mislead an operator, but it can and did
mislead a reader auditing what this build claims, which is what
`FEATURES.md` is re-measured against.

The wording below is **not guessed**: it is scoped down to the two facts
this catalog's own surviving callers deal in, and everything about
verdicts is deferred to [`crate::text::trust`], which is the catalog the
engine's reply was actually spent on. ⇒ *An absence claim is a claim about
every route; when the absence ends, grep for the sentence, not just for
the caller.*

### `fn coverage_line`

`covers_to_eof` is a real answer and it is the useful half of the two:
content appended after a signature is the ordinary way a signed document
stops meaning what it said, and it needs no cryptography to detect.

### `fn cannot_author`

# Why this exists rather than eight greyed buttons

R9: an unavailable capability renders **nothing**, and greying is reserved
for *temporarily* unavailable. A row of dead *Encrypt*, *Set permissions*,
*Remove encryption* and *Sign* controls would be eight promises this build
cannot keep, and an operator would spend their time discovering that one at
a time.

But *nothing at all* is the other failure. An operator opening a tab
called Security and finding only readouts will reasonably conclude the
feature is half-built and stop looking — which is the discoverability defect
that produced the Tool panel, arriving from the opposite direction. So the
tab states the boundary once, in one sentence, as a fact about this build.

**CORRECTED 2026-09-05 — two of its three clauses were false, and it
too has ZERO call sites.** *"Encryption first, signing later"* was the
right prediction and it came true on 2026-09-04: `file.encrypt` and
`file.permissions` are registered, dispatched through
`crate::app::dispatch::security`, drawn on **File ▸ Security**, and
`crate::protect` calls `set_encryption`, `set_permissions` and
`remove_encryption`. So pdfcer can add a password, remove one, and change
these permissions. Only *sign a document* is still true.

The tab this was written for was superseded by that window and by
[`crate::text::protect`], which is where the live wording lives — so this
sentence sat false and unreferenced for a day. It is scoped to the one
clause that survives, and it now names where the rest went.

**CORRECTED AGAIN 2026-09-06, and the last surviving clause is gone
too.** It read *"It cannot sign a document; that is still being built in the
engine."* Both halves were false by then: `pdfcer_core::sign` shipped on
2026-09-05 — 101 public items, written in answer to this shell's own
request — and `file.sign` is now registered, dispatched and drawn on the
same File > Security band as its two neighbours.

⇒ **This is the THIRD correction to one sentence, and it has had ZERO call
sites throughout.** That is the finding worth keeping: a string nothing
draws cannot be caught by looking at the screen, cannot be caught by a
driven check, and is corrected only when somebody happens to grep past it.
It has now been wrong about encryption, wrong about permissions, and wrong
about signing, in that order, each time by outliving a capability's arrival.

⚠ The function is kept rather than deleted for the reason its own header
gives — the tab it belongs to states a boundary, and a boundary that is
merely absent reads as a half-built feature — but what it now states is the
**shape** of the boundary rather than a list of missing verbs, because a
list of missing verbs is a dated citation and this one has now expired three
times. `crate::panels::signatures` is named because that is the honest limit:
pdfcer authors a signature and reports what it can check about one, and
whether a recipient trusts it is not pdfcer's to say.
