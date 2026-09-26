# `text::sign` — every operator-facing string on the control that puts the
operator's own signature into a document

The write side of a subject whose read side already has two modules:
[`crate::text::security`] reports what a document says about its
protection, [`crate::text::trust`] reports what pdfcer could and could not
check about a signature that already exists, and this one describes
something pdfcer is **about to do with a private key**.

## THE STANDARD THIS MODULE IS HELD TO, AND IT IS NOT THE USUAL ONE

[`crate::text::trust`]'s subject is a **verdict**, and its failure mode is
claiming more than the engine checked. This module's subject is an **act**,
and its failure mode is different and worse: a sentence here can persuade an
operator to attach their legal identity to a document. So two rules bind
every string below, and both are narrower than "be accurate".

1. **Nothing here calls a signature valid, trusted, secure or verified.**
   Not once, not as a summary, not in a tooltip. Authoring a signature and
   the signature being *trusted by a recipient* are different facts settled
   by different parties, and this surface only ever performs the first.
   [`crate::panels::signatures`] is the only place in pdfcer that reports
   the second, it reports three facts that never collapse into one, and a
   cheerful word here would undo that whole design before the panel is
   opened. `sign.svg`'s own note carries the same constraint for the glyph.
2. **Every sentence about what will be written names what will be
   written.** `/Reason`, `/Location` and the signing time are the
   operator's words, copied verbatim into a legal artifact; the copy says
   so rather than describing them as "details".

## What is deliberately NOT offered, and it is a string's absence

**There is no *Name* field**, and its absence is a decision rather than an
omission — recorded here because an absence cannot be read out of the code
that does not contain it.

`SignRequest::name` writes `/Name`, and the engine's own note says `None`
*"omits the key and a verifier falls back to the certificate subject (Table
252 says it should anyway)"*. A free-text name beside a certificate is a
**second, unverifiable claim about who signed**: nothing stops it saying
something the certificate does not, and a reader that trusts `/Name` over
the subject would show a name nobody vouched for. The certificate is the
name. So the window shows the subject it read out of the operator's own
`.pfx`, and offers no way to write a different one.

## `/ContactInfo` is not offered either, for a smaller reason

It is legitimate and harmless — a phone number for a verifier who wants to
reach the signer. It is left out because three free-text boxes on a form
whose two important controls are the certificate and the destination is
three boxes an operator scrolls past, and because nobody has asked for it.
Adding it is one field and one string; that is the right size for a request,
and the wrong size for a guess.

## Item notes

### `const APPEARANCE_INDENT`

One constant because [`written_details`] writes it and [`appearance_shown`]
matches it, and two spellings of an indent would drift without a symptom:
the sentence would still read correctly and the counter would silently
report that none of it reached the operator.

### `fn file_sign`

# The label is *Sign…* and not *Digitally sign…*

Because there is no other kind of signing in pdfcer, and a qualifier that
distinguishes nothing is a longer button. The tooltip carries the words a
person searching for the feature will have in mind — *certificate*,
*digital ID* — so the control is findable without the label carrying them.

# The tooltip names the two refusals in the same sentence

R9's *explained* branch. Whether **this** document is encrypted, or is
carrying a redaction the operator armed ten minutes ago, is not known when
the ribbon is built, so the control cannot be absent — and finding out by
pressing is the failure this project has paid for more than once. The hover
says what will happen before the press.

# …and it says the signature goes in a new file by default

Because the alternative reading — that pressing this changes the document
on screen — is the reading every other verb on the Edit tab has taught, and
this one is the operator's legal artifact.

### `fn intro`

It states the **shape** of what is about to happen rather than the
limits of this build — [`crate::panels::signatures`]' header records why
that distinction matters: a sentence naming a limit was true when written
and false within hours, and the prose around it stayed true. A sentence
describing the mechanism cannot go stale the same way.

### `fn refusal_redaction_pending`

Named first among the refusals and worded as one step rather than as a
wall, because it *is* one step: the operator armed the removal, and Edit ▸
Redact holds both the button that finishes it and the button that calls it
off.

### `fn refusal_encrypted`

It names the engine's reason rather than stopping at "it is encrypted",
because the two suggest opposite next moves: an operator told only that the
document is protected will look for a permission to change, and the actual
remedy is to sign first and protect afterwards.

### `fn refusal_certification_forbids`

The permission number is in the sentence because it is in the document
and an operator taking this to whoever certified the file needs to be able
to quote it.

### `fn refusal_line`

One pure function rather than a `match` at each of the two call sites —
the window, which draws it instead of a form, and
[`crate::app::actions::sign`], which reaches it when the document changed
between the window opening and the press. Two spellings of one mapping is
two chances for a refusal to be worded differently depending on when it was
noticed.

A new [`crate::sign::Refusal`] variant is a compile error here rather than a
silent fall-through to a catch-all.

### `fn already_signed`

Not a refusal — a PDF may hold many — and shown anyway, because an
operator adding a second signature to a document they thought was unsigned
has learned something about the file they were handed.

### `fn passphrase_note`

This is a **promise about behaviour**, and it is the one sentence in
this module that a reader is entitled to check the code against. It is true
because of `crate::secret::Secret` (a type whose value cannot be formatted)
and `crate::sign`'s §5 (no trace line carries the passphrase, its length, or
the certificate's path). If either of those changes, this sentence must be
the thing that changes with it.

### `fn open_certificate`

A separate press rather than opening the file as soon as both boxes have
something in them. Two reasons, and the second decides it: a passphrase is
typed one character at a time, so an eager load would attempt — and fail —
on every keystroke, and some PKCS#12 containers use a key-derivation
function expensive enough for that to be felt. More importantly, **it puts
the identity on screen before the signing control is reachable at all**: the
operator sees whose certificate they are about to use, from the file itself,
rather than trusting that they picked the right one.

### `fn identity_friendly_name`

Shown as *"stored as"* rather than as a name, because it is the label
whoever exported the file typed into their own certificate manager. It is
useful for recognising the right file and is not a claim about anything.

### `fn identity_integrity`

A PKCS#12 file may carry no `macData` at all, in which case the passphrase
opened the key and nothing verified that the file is the one that was
exported. The engine reports `mac: None` for exactly that case, and this is
the one fact on this window an operator could act on that they would not
otherwise be told: a container whose integrity was never checked is one that
could have been altered between export and here.

It is stated in both directions rather than only in the bad one, because a
line that appears only when something is wrong is a line nobody learns to
look for.

### `fn identity_unrelated`

Disclosed rather than silently discarded. An operator whose file holds
four certificates and whose signature embeds two should be told which
happened, because the usual cause is a container exported with a whole
address book in it and the second usual cause is a chain that does not
actually chain.

### `fn identity_refused`

The engine's own message is printed **verbatim** and is not re-worded.
`Pkcs12Error` distinguishes a wrong passphrase from a scheme pdfcer does not
implement, from a container with no private key, from a key algorithm it
cannot sign with — four different next moves — and every one of its variants
is already a sentence written to be read. Softening them into "the
certificate could not be opened" is how an operator comes to spend an
afternoon retyping a passphrase that was right.

### `fn reason_hint`

An example rather than an instruction, and a bland one on purpose: a
placeholder reading *"I approve this document"* is a suggestion, and a
suggested reason on a legal artifact is pdfcer putting words in somebody's
mouth. Leave-it-blank has to be an equally comfortable answer.

### `fn name_comes_from_the_certificate`

See this module's header for the full argument. It is on the window and not
only in the source, because an operator looking for a Name box needs to know
the box is missing on purpose.

### `fn signing_time`

The engine reads no clock — its `SignRequest::signing_time` doc says a
GUI *"passes the time it showed the operator"* — so this string is not a
report of what was written, it is the **source** of it. The moment on screen
and the moment in the file are the same value.

### `fn clock_unusable`

The one failure `crate::app::clock::pdf_date_utc` can have. PAdES requires
`/M` and pdfcer will not invent one, so this is a refusal rather than a
signature with no time on it.

### `fn placement_note`

R8b Rule 4 in its sharpest form: the box is **applied content**, it renders
exactly as the saved file will render, and there is nothing provisional about
it. An operator who found something on their drawing afterwards that nobody
had described would read it as a defect, and they would be right to.


**What this string said until the pin moved:** *"The box is an empty frame:
pdfcer does not yet draw your name or the date inside it."* That was true of
`pdfcer-core` at `f9bc7c8` (v0.41.0), where a visible signature's appearance
was, in the engine's own words, *"a thin frame only — no text"*.

**It is false at `d6b998f` (v0.42.0), the revision `Cargo.lock` now pins.**
`Pass 10.14` (`187fa09`) composes the signer's CN, the date, and the reason
and location when given, in Helvetica, shrunk to fit from 10 pt to 4 pt, and
refuses a rectangle too small for them by name
(`SignApplyError::AppearanceOverflow`) **before anything is staged** rather
than clipping — because a signature box whose text is silently cut is a
signature box that misstates who signed.

⇒ **The falsehood was an UNDER-promise, and that is why it needed an
alarm rather than a test.** An operator told the box would be empty, who then
finds his own name in it, has been pleasantly surprised; he files nothing.
No screen, no unit test and no gate could have gone red. What caught it was
that the old string's doc comment carried the engine commit, the measurement
(`git merge-base --is-ancestor`, not a changelog) and the instruction *"when
the pin moves past `f9bc7c8`, re-read this string first"*. **A claim about
the engine is a dated citation; write its expiry beside it.**

The old wording is quoted above in full rather than deleted, so a future
improvement cannot reinstate it out of git history believing it to be a
simplification.

# What did NOT change, and why the recommendation survives

The default is still *draw nothing*, and the argument for it is now a
different argument rather than a weakened one. It used to be *"the box would
be empty and read as a defect"*. It is now: **a reader shows the signature in
its own panel whether the box is there or not**, so the box adds no
information and does add content the operator did not draw to a sheet he is
about to send out. An operator who wants the stamp gets a stamp with his name
in it, and the sentence below now tells him that truthfully.

⚠ [`placement_where`]'s 180 × 60 and `crate::sign::default_rect`'s clamp were
re-examined with this: the clamp shrinks the box only on a page smaller than
252 × 132 pt, and `AppearanceOverflow` is the engine's refusal on exactly
that case — by name, before staging, so a small page produces a sentence
rather than a clipped signature. Nothing to change; recorded because the
question was asked.

### `fn placement_existing`

**Worded from the operator's situation, not from the format.** He does
not think *"there is an empty `/FT /Sig` field in the AcroForm"*; he thinks
*"they sent it back with a box on it for me to sign in"*. The label names the
situation, and `count` is in it because the number is the one thing that
tells him whether the box he was told about was found.

### `fn placement_field_note`

**R9's *absent* branch needs a sentence, and this is it.** The engine
refuses `--visible`/`--page` alongside a field name by name — *"the existing
field already has a rectangle"* — so this shell makes the combination
unrepresentable and the controls simply go. A control that vanishes without
explanation is indistinguishable from one that broke; a greyed control with
no hover is the thing O77's sweep found seven of.

### `fn field_row`

The page number is 1-based and is omitted rather than guessed when the
document does not say. A widget's `/P` is optional in the standard, so
*"page 1"* on a field that names no page would be this shell inventing a
fact about the operator's document.

### `fn field_invisible`

Said because the operator would otherwise sign, look at the drawing, see
nothing, and conclude it had failed. The author chose this; §12.7.4.5 makes a
zero-area rectangle the standard way to place an invisible signature.

### `fn field_locks`

Table 233. Signing a field that carries a `/Lock` makes the engine write a
`/FieldMDP` reference copying the lock's Action and Fields (§12.8.2.4) —
which genuinely freezes other fields in the document. That is a consequence
the operator has to consent to, and consent given after the file is written
is not consent.

The sentence says **who decided**. The freeze is not pdfcer being
cautious; it is an instruction the person who prepared the document wrote
into it, and an operator who reads it as pdfcer's own behaviour will go
looking for a setting to turn off.

### `fn field_constrained`

**Stated as a possibility, not a verdict, and that is deliberate.** This
shell reads only whether `/SV` is present; the engine evaluates it in full at
signing time. Saying *"this will be refused"* would be a second, worse answer
to a question with one authoritative answer, and saying nothing would let the
refusal arrive as a surprise. So: a warning that a refusal is possible and
whose it would be.

### `fn no_existing_fields`

The option is drawn and disabled with this beneath it rather than hidden,
which is the opposite of this window's usual rule and is right here for one
reason: the operator was **told by the sender** that there is a box. *"The
option is missing"* and *"the box the sender promised is not in this file"*
are the same picture and completely different facts, and only the second is
true.

### `fn placement_where`

The numbers are in the sentence because the box is content in the
operator's file and *"near the bottom right"* is not something anybody can
check against the result.

### `fn kind_heading`

A phrase rather than a caption, on this window's standing rule: `.strong()`
resolves to the accent-filled widget colour and draws pale text on a pale
panel (`DEFECTS.md` D11), so the hierarchy is carried by wording and layout.

### `fn kind_certify`

The label avoids the word *certify* as its only cue and says what the act
means — signing **as the author** — because "certify" reads to most people as
a stronger synonym for "sign" rather than as the specific `/DocMDP` act it
is. The word is kept in the sentence beneath so the operator can match it to
what a reader will show him.

### `fn mdp_level`

**The engine's `MdpPermission` is the input, and the plain wording is
this shell's.** `MdpPermission::meaning` renders Table 254's own words — *"no
changes"*, *"form fill-in and signing"* — which are exact and are a
standard's phrasing, not a person's. What an operator needs is what happens
to *his* signature, so each line says that; the standard's own word is not
repeated, because two renderings of one fact on one screen is how a surface
starts disagreeing with itself.

### `fn certify_unavailable`

R9's *explained* branch applied to an option rather than a window: both of
the engine's certification refusals are states of the document, knowable when
the window opens, so the option is **absent with this sentence** rather than
offered and then refused. The document can still be signed, and the sentence
says so — otherwise an operator reading a refusal on this window will read it
as a refusal of the window.

### `fn confirm_disabled_no_certificate`

One function returning the FIRST outstanding thing rather than a list,
because a hover is read in one glance and because the conditions are met in
this order anyway. `crate::text::protect::confirm_disabled` is the same
shape for the same reason.

### `struct Written`

A struct rather than eight parameters for two reasons. The sentence takes
four strings in a row, and a caller that transposed two of them would
compose something entirely plausible and entirely wrong. And the engine's
`SignReport` is `#[non_exhaustive]`, so it cannot be built outside the
engine crate: this is the shape a test in this crate *can* construct, which
is what keeps the composing pure and asserted headlessly.

⚠ What no test here can see is the mapping *into* this struct. See
`ui-verify`'s `signing`, whose `sign-disclosed` line is that link's only
oracle.

### `fn written_details`

The engine's `SignReport` exists so a front end can state what it wrote
rather than assume it. This is that statement, and it names the field, whose
certificate was used and its serial — the two things a recipient will quote
back when they ask *"is this really you?"*

# What each part is, and why it is on this screen

* **`reused`** — whether the signature went into a box that was already
  there. Two outcomes that produce identical byte counts and can produce
  identical field names: *"it signed the sender's box"* and *"it made a new
  box beside it"*. Only the report can tell them apart, so it is said.
* **`lock`** — `SignReport::field_lock`, the `/FieldMDP` that was written
  because the field carried a `/Lock`. Disclosed here **as well as** before
  the press, because this is the sentence that says it *happened* rather
  than that it *would*.
* **`certification`** — the `/DocMDP` level, with Table 254's own meaning
  beside the number, so the operator can read what he just permitted.
* **`notes`** — `SignReport::notes`: seed-value constraints the form author
  RECOMMENDED and this signature does not meet. These are the ones that
  did **not** refuse. Silence about them would be exactly the *"quiet
  divergence"* the engine's own strictness exists to prevent, arriving one
  layer up.
* **`appearance`** — `SignReport::appearance_lines`, the text a visible
  signature's box shows. The engine composes it from the certificate's
  subject, the time of signing and whatever reason and location were typed,
  so it is content the operator never wrote and **cannot look at**: the
  document still open is the unsigned one — see [`open_document_unchanged`].
  Empty for an invisible signature, which is a placement he chose and which
  therefore owes no sentence.

### `fn appearance_shown`

Counted against the SENTENCE, never against the slice. Reading the slice's
own length twice would be satisfied by a caller that handed
[`written_details`] an empty one, which is the defect no test in this
process can see: `SignReport` is `#[non_exhaustive]`, so the mapping from
the engine's report into [`Written`] cannot be exercised here at all.

The indented form is what is matched, because it is the shape the
appearance block writes and nothing else in the sentence produces it. A
bare `contains` would score a line that merely happens to be a substring of
the subject.

⇒ `ui-verify`'s `signing` reads this through `sign-disclosed`, and that
driven run is the only oracle the mapping has.

### `fn open_document_unchanged`

`crate::sign`'s §3: the session still holds the placeholder, not the
signature, and the engine's own instruction to a GUI is to reload. An
operator who pressed `Ctrl+S` after this without being told would append a
second revision onto a base that is no longer the file on disk.

### `fn engine_refused`

The engine's message verbatim, for [`identity_refused`]'s reason:
`SignApplyError` has a distinct, already-written sentence per variant, and
two of them (the encrypted document, the pending redaction) are the ones
this window is supposed to have caught earlier. Reaching one of those here
means the document changed between the window opening and the press — which
is a real thing that can happen and is worth reading in the engine's own
words rather than in a paraphrase.

### `fn reservation_too_small`

`SignApplyError::ReservationTooSmall`'s message ends *"sign again with a
larger reserve"*, and there is no control here that sets one, deliberately
(`crate::sign::prepare`'s note argues why asking would be handing the
operator arithmetic). So the engine's sentence is shown **and then
corrected**, rather than shown alone as an instruction that leads nowhere.

### `fn author_imposed`

# The problem this string exists to solve

`Pass 10.13` enforces a signature field's `/SV` seed-value dictionary
(Table 234) **in full**, and the engine is deliberately **stricter than
Acrobat**. Three consequences follow, and the third is the dangerous one:

1. A REQUIRED constraint the request does not meet is refused by name with
   the satisfying values (`SeedValueViolated`).
2. A constraint pdfcer cannot evaluate — `/Cert`, a required timestamp, a
   legal attestation, revocation info, an unknown key — is **refused rather
   than skipped** (`SeedValueUnevaluable`), because a condition the form
   author wrote and the signer quietly ignored is worse than a refusal.
3. ⇒ **So the operator will meet refusals on documents Acrobat would sign.**

[`engine_refused`]'s wording — *"pdfcer did not sign the document: …"* — is
correct for every other refusal on this surface and is **wrong for these**.
It puts pdfcer in the subject position of a sentence about somebody else's
rule, and an operator who reads *"pdfcer did not sign it"* beside a document
Acrobat signs has been told, in plain English, that pdfcer is broken. He
would be right to conclude that from the sentence, and wrong about the
program, and the feature would be reported as a defect.

# What this sentence does instead

**It names the author first, states the engine's own message second, and
gives two remedies that do not involve pdfcer changing.** And it says the
strictness is a **choice**, in one clause, because an operator comparing two
programs deserves to know which one is doing something unusual and why —
hiding it would leave him to discover the difference on his own and draw the
worse conclusion.

The engine's message is quoted verbatim rather than paraphrased. It names
the constraint and the values that would satisfy it — *"requires SubFilter
one of: ETSI.CAdES.detached, adbe.pkcs7.detached"* — which is precisely what
the operator has to forward to whoever prepared the document. A paraphrase
would drop the values, which are the actionable half.

### `fn field_refused`

Reachable even though the window filters the list, and that is the point
of having it: the list is read once when the window opens, and the document
could have been signed by something else in between. Worded as a fact about
the box rather than as an error, with the engine's own sentence carrying the
detail.

### `fn appearance_overflow`

`SignApplyError::AppearanceOverflow`, new in `Pass 10.14`. The engine
refuses **before staging** rather than clipping, because a signature box
whose text is cut is a signature box that misstates who signed. Its message
carries the line count and the rectangle; this adds the remedy that is
actually available here, which is not the engine's `--visible` advice.
