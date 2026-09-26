# `sign` — putting the operator's own digital signature on a document

The answer to the request this shell filed on **2026-09-03**: *"a document
cannot be signed."* `pdfcer-core` answered it on 2026-09-05 with
`pdfcer_core::sign` — 101 public items across `Pass 10.7` (PKCS#12 identity
loading), `10.8` (the CAdES `SignedData`, PAdES B-B) and `10.9`
(`EditSession::sign`) — whose own module header says it is *"the family the
`pdfcer-gui` request of 2026-09-03 asked for."*

The window is [`crate::dialogs::sign`]; this module is everything that can
be decided **without a `Ui`** — what the document says about itself, what
may be offered, which identity is loaded, and the atomic write at the end.
The split is [`crate::protect`]'s and [`crate::redact`]'s, taken rather than
re-argued: every rule on this surface is a rule about **the operator's
file**, and a rule that can only be exercised by driving a window is a rule
that gets asserted once, by hand, and then drifts.

---

# 1. THE SUBSYSTEM WAS SHIPPED, ANSWERED OUR OWN REQUEST, AND WAS NOT
IN THE BINARY FOR THREE DAYS

This module exists because of a defect worth stating before any of its
design, since the defect is the more transferable half.

`crates/pdfcer-gui/Cargo.toml` took `pdfcer-core` with
`default-features = false` and forwarded `jpx` and `ocrs`. The engine's
`signing` feature is **default on**; a `default-features = false` dependency
that does not re-name it strips it. So `pdfcer_core::sign` did not exist in
this build at all. **Nothing failed to compile. No test went red.**

That is the JPX incident, repeating, three days after the warning about
it was written into the very manifest that repeated it — the comment at
`Cargo.toml`'s feature block records the day the GUI silently lost JPEG 2000
decoding to the identical omission and says, in as many words, *"forgetting
to forward does not fail to compile."*

⇒ **A warning does not protect a code path written after it.** The
mechanism that does is `tools/gates/check-forwarded-features.sh`, which
reads the engine's own `default = [...]` and fails the build when a name in
it is neither forwarded nor listed with a reason.

And it went unnoticed for two of those days because
`tools/gates/check-verb-coverage.sh` scored `EditSession::sign` as
**consumed** on the bare word `sign` appearing in `app/actions/bookmarks.rs`
— in a documentation table about the arithmetic **sign of `/Count`**. That
gate now matches call shape. Two independent instruments, both green, both
about nothing.

---

# 2. The engine's verbs, verified against the source at the pinned revision

`D:\Dev\pdfcer\crates\pdfcer-core\src\`, at the revision `Cargo.lock` pins
(`pdfcer-core 0.42.0`, `d6b998f`). Checked at that commit rather than at
`main`, on [`crate::protect`]'s standing rule: what compiles here is the
pin, and a sentence about the engine has a shelf life measured in hours.

```text
sign::pkcs12::Pkcs12Signer::from_der(&[u8], &str) -> Result<Pkcs12Signer, Pkcs12Error>
Pkcs12Signer::report()                            -> &Pkcs12Report
EditSession::sign(&dyn Signer, &SignRequest, &SaveOptions)
                                                  -> Result<(Vec<u8>, SignReport), SignApplyError>
forms::parse_acroform(&SessionGraph)              -> Option<AcroForm>
```

---


The lock went `f9bc7c8` (v0.41.0) → `d6b998f` (v0.42.0), thirteen commits,
carrying three signing Passes. **Nothing failed to compile** — every one of
them is additive on a `#[non_exhaustive]` struct — which is precisely why
the interesting half is the prose:

| Pass | what arrived | what it falsified here |
|---|---|---|
| `10.12` (`02bb1ba`) | `SignRequest::certify` — a **certifying** (`/DocMDP`) signature | nothing; the capability was simply absent |
| `10.13` (`ab40127`) | `SignRequest::field_name` resolving an EXISTING empty `/FT /Sig` field — signing **into** a box somebody else placed | nothing; absent |
| `10.14` (`187fa09`) | a **composed** visible appearance: signer CN, date, reason, location, Helvetica, shrink-to-fit | [`crate::text::sign::placement_note`], which told the operator in as many words that *"the box is an empty frame"* |

That third row is the one worth carrying, because the falsehood was
**under-promising** and therefore unreportable: an operator told the box
would be empty, who then finds his own name in it, has been pleasantly
surprised and will never file a defect. Nothing on the screen, in a test, or
in a gate could have gone red. What caught it was that the string's own doc
comment carried the date, the engine commit and the instruction *"when the
pin moves past `f9bc7c8`, re-read this string first"* — a dated citation
that names its own expiry.

⇒ **A claim about the engine is a citation, and a citation gets a date and a
successor.** The corrected string carries the same apparatus.

---

# 2c. SIGNING INTO A BOX THE SENDER PLACED — the half that matters

What shipped on 2026-09-06 signs by **creating** a signature field. That is
the wrong half for this operator's ordinary day: a drawing goes out for
approval, the sender places a *"sign here"* box on the title block, and it
comes back needing a signature **in that box**. `Pass 10.13` is the other
half, and it changes three things here.

**1. The field is chosen, not created.** [`Standing::empty_fields`] lists
every `/FT /Sig` field in the document that has no `/V` — read once, when the
window opens, out of `forms::parse_acroform`. [`Placement::ExistingField`]
names one.

**2. Placement becomes a THREE-way choice, and the combination the
engine refuses is made unrepresentable.** `SignRequest::visible` beside a
`field_name` that resolves to an existing field is
`SignApplyError::RectRefusedForExistingField` — *"the existing field already
has a rectangle; --visible/--page do not apply."* This shell could have sent
both and shown the refusal. It does not: [`Placement`] is one enum with three
arms, so *"draw the box here"* and *"use the sender's box"* cannot both be
true, and the window **retires** the page chooser rather than greying it —
R9's *absent* branch, with [`crate::text::sign::placement_field_note`]
saying why it went.

**3. TWO ENFORCEMENT FAMILIES ARRIVE WITH IT, AND BOTH ARE THE AUTHOR'S
RULES RATHER THAN PDFCER'S.** This is the design decision the wording has to
carry, and getting it wrong makes a working feature read as a defect.

* **`/Lock` (Table 233)** on the chosen field is *honoured*, as a
  `/FieldMDP` signature reference (§12.8.2.4) whose Action and Fields are
  copied from the lock. So signing that box can legitimately **freeze other
  fields the author nominated** — a real consequence for a form the operator
  may still have to fill in. [`SigField::locks`] carries it, and the window
  says so **beside the field, before the press**, not in the summary
  afterwards.
* **`/SV` (Table 234)** is enforced **in full**: a required constraint the
  request does not meet is refused by name with the satisfying values
  (`SeedValueViolated`); a recommended one unmet is disclosed on
  `SignReport::notes`; and anything pdfcer does not evaluate — `/Cert`, a
  required timestamp, a legal attestation, revocation info, an unknown key —
  is **refused rather than skipped** (`SeedValueUnevaluable`).

**The engine is therefore deliberately stricter than Acrobat, and the
operator will meet refusals on documents Acrobat would sign.** A sentence
that reads *"pdfcer could not sign this"* would be true and would be taken as
a defect in pdfcer. [`crate::text::sign::author_imposed`] is the one wording
for all of them and it says whose rule it is: **the person who prepared the
document wrote the condition**, pdfcer will not sign around it, and the
remedy is another box or a word with the sender. The strictness is stated as
a choice, because an operator comparing two programs deserves to know which
one is doing something unusual and why.

---

# 2d. Certifying — an option in this window, not a second command

`Pass 10.12`'s `SignRequest::certify` writes the `/DocMDP` transform and the
catalog `/Perms`, which is *"the author's signature"*: it says what may be
changed afterwards without invalidating it (Table 254 — `P` 1, 2 or 3).

It is a **radio pair inside the Sign window**, not `file.certify` on the
ribbon, and that is a design decision rather than an economy. The two acts
share every field on this form — the same identity, the same reason, the same
placement, the same destination, the same private key handled the same way —
and differ in one value. A second command would put all of that in a second
file, which is where a disclosure goes missing; and it would ask the operator
to know the word *certify* before he could find out what it means. Here the
choice is beside its explanation.

⚠ Both of the engine's certification refusals are **states of the document**,
not of the request: a certification must be the document's FIRST signature
(`CertificationNotFirst`) and there is at most one per document
(`AlreadyCertified`). Both are knowable when the window opens, so
[`Standing::may_certify`] answers them there and the option is **absent with
a sentence** rather than offered and then refused.

## What the engine does that this module must NOT duplicate

`EditSession::sign` **self-verifies**: step 5 of its own documentation
re-parses the bytes it is about to return and runs `signature_verify` over
them, and *"anything but `Integrity::Verified` with full coverage is a
refusal, and no bytes are returned. pdfcer does not hand out a signature it
cannot itself verify."*

So there is no verification step in [`prepare`], and adding one would be a
second derivation of one fact — which is how two surfaces come to disagree.
What this module does instead is **state** it: [`Prepared::self_verified`]
carries the engine's own `SignReport::self_verified`, which that struct's
documentation says is *"always `true` on `Ok`; present so the fact is
stated, not assumed."*

⚠ That is emphatically **not** the same as this project's R1 bar. A field
saying the engine checked its own output is still the engine's word inside
one process. The independent read is `tools/ui-verify`'s
`a_document_can_be_signed_and_the_signature_is_in_the_file`, which reopens
the written file **in a fresh process** and reads it through the Signatures
panel — the verification side that shipped as `Pass 10.5` — so the oracle is
a different subsystem from the one under test.

---

# 3. Why the session is handed to the verb, and why the file is reopened
afterwards

The opposite of [`crate::protect`]'s answer, and the asymmetry is the
engine's rather than a preference here.

`EditSession::sign` takes **`&mut self`** and stages a real, undoable
`CommandKind::AddSignatureField` — the signature field, its widget and the
signature dictionary with its zero-filled `/Contents` hole — then serialises
the session as an **incremental update**. It must be the open session: the
operator's unsaved edits are in it, and a signature that did not cover them
would cover a document they are not looking at.

But the session is **left holding the placeholder**, not the signature.
The engine says so outright: *"the session still holds the staged
placeholder objects (zeros in `/Contents`) … a caller that wants to keep
editing must re-open the returned bytes. A CLI writes the bytes and is done;
a GUI reloads."*

⇒ So after a successful write this surface offers, and does not perform,
**[`crate::dialogs::sign`]'s *Open the signed document*** — a second
document tab on the file that was just written. Offering rather than
performing, because replacing the operator's open document out from under
them would discard an undo history they can still see; and offered rather
than omitted, because the open document is now in a state whose only honest
description is *"this is not the file you signed"*, and an operator left to
discover that by pressing `Ctrl+S` would append a second revision on top of
a stale base.

⚠ **What is deliberately NOT done: no `Ctrl+Z` is pushed and no state is
rewound.** Undoing the staged command in the old session is, in the
engine's own words, *"harmless and pointless"*. Doing it would look like
tidying up and would put a spurious entry on the operator's undo stack for
an act that produced a file.

---

# 4. The five refusals, all of which are STATED rather than discovered

R9: *the control is absent or explained, never a button that fails on
press.* Whether **this** document can be signed is not knowable when the
command registry is built, so `file.sign` stays on the ribbon and the window
opens and says why instead of drawing a form whose only possible outcome is
a failure. [`Standing::refusal`] is the whole of that decision, as a pure
function.

| [`Refusal`] | engine variant | why this shell can reach it |
|---|---|---|
| [`Refusal::Encrypted`] | `SignApplyError::Encrypted` | File ▸ Security ▸ Encrypt… ships (`O119`), so this shell can *make* an encrypted document and then be asked to sign it |
| [`Refusal::RedactionPending`] | `SignApplyError::RedactionPending` | deferred redaction ships (`Pass 250.2`), and a pending removal is a normal mid-session state |
| [`Refusal::CertificationForbids`] | `SignApplyError::CertificationForbids { permission: 1 }` | a certified document opened from disk |
| [`Refusal::RecoveredBase`] | `SignApplyError::RecoveredBase` | a damaged file that loaded through cross-reference recovery |
| [`Refusal::NotOnDisk`] | *(none — see below)* | File ▸ New makes a document that has never been written |

The first two are the ones the build brief names, and they are the two
this shell can produce **in one session without leaving the application**,
which is what makes them reachable rather than theoretical.

The fifth has **no engine counterpart**, and that is the interesting
one. `EditSession::sign` would not refuse a document that was never on
disk — it would sign it, incrementally, over whatever base the session
holds. The refusal is this shell's, and the reason is that an incremental
update is *an appendix to a specific file*: signing a document that has
never been saved produces bytes whose base revision exists nowhere, so the
operator's next ordinary Save would write a different file that the
signature does not describe. Saying *"save it first"* is one sentence; the
alternative is a signed file with no ancestor.

---

# 5. THE PASSPHRASE, AND A RULE STRICTER THAN THE ONE BESIDE IT

This module handles a **private key**, and that changes the standard from
the one [`crate::protect`] works to.

* The passphrase becomes a [`Secret`] the instant it leaves the text field,
  for `crate::secret`'s reason: `Action` derives `Debug`, this crate traces
  to stderr under `PDFCER_DIAG`, and **`tools/ui-verify` captures that
  stderr to a file it keeps as evidence**. A single `{:?}` on the path would
  write it to disk in plain text.
* ⚠ **No trace line here carries the passphrase's LENGTH either**, and that
  is a deliberate departure from `protect::prepare`, which traces
  `user_chars=` and `owner_chars=`. There, the length plus "is it ASCII" is
  what explains a SASLprep normalisation refusal completely, and a document
  password is the operator's own gate on their own file. Here the secret
  guards a **private key** — the material an impersonation needs — and a
  length is a search-space reduction written into a file that outlives the
  session. `passphrase=set|empty` answers the only diagnostic question
  ("was one supplied at all?") and reduces nothing.
* [`Identity`] holds the engine's `Pkcs12Signer`, whose own `Debug` impl
  prints the report and never the key. This type does not derive `Debug` at
  all; see its note.
* Nothing writes the `.pfx` path into any persisted preference. A path is
  not key material, but a file picker that remembered where the operator
  keeps their identity would be a durable pointer at it, written by a
  convenience nobody asked for.

---

# 6. Where the bytes go — [`crate::redact`]'s shape, part for part

Settled by the operator hours before the redaction work started, in his own
words: *"why does it have to save to a new file right away? Why can't it
just wait on saving until I choose to save over the existing file or save as
a new file?"* So, unchanged here:

1. **A new file is the default**, and [`suggested_path`] never proposes the
   source. A safe default is a mechanism; a warning is something to click
   past.
2. **Replacing the original is offered**, behind one extra acknowledgement
   that names the file, and it takes **no picker** — a picker pre-filled
   with the source is the shape of every accidental overwrite there has ever
   been.
3. **The write is atomic** — temp file, then rename ([`Prepared::write_to`]).

Replacing is genuinely reasonable here in a way it is not for a redaction:
a signature is an *incremental update*, so the replaced file still contains
every byte it had. Nothing is lost by signing in place. It is still not the
default, because "the file I sent out" and "the file I signed" being one
keystroke apart is worth one deliberate act.

## Item notes

### `fn non_empty`

Trims first. A field holding one space is an untouched field as far as
anybody looking at the screen is concerned, and writing `/Reason ( )` into a
legal document because of a stray keystroke is the kind of thing nobody ever
finds.

### `struct Standing`

[`crate::protect::Standing`]'s twin, and it exists for that type's stated
reason applied to this surface: a form that opened offering to sign a
document the engine will refuse has told the operator a falsehood before
they touched anything.

Read **once**, when the window opens, and never re-read per frame. The
sentence under the heading must describe the document the operator chose to
act on; a value re-read every frame could change out from under the choices
seeded from it.

### `struct SigField`

`Pass 10.13`. Everything on this type is read out of the document and
nothing is inferred; see [`read_empty_signature_fields`] for where each
value comes from and for the two keys the engine models and
`pdfcer_core::forms::Field` does not.

### `const rained`

A boolean rather than the parsed constraints, deliberately. `/SV` is
seven `/Ff` bits over five entry families and the engine evaluates all of
them; re-deriving that here would be a **second** answer to a question
with one answer, and the two would disagree the first time either
changed. What this shell owes the operator before the press is *"the
sender attached conditions to this box"*; what the conditions ARE is the
engine's sentence, arriving by name if one is unmet.

### `fn read_empty_signature_fields`

`Pass 10.13`'s input. Signed fields are excluded — the engine refuses to
re-sign one (`SignApplyError::FieldAlreadySigned`) and offering it would be
an option whose only outcome is a refusal — and so is anything that is not a
signature field, which is a different refusal
(`SignApplyError::FieldNotSignature`) and equally not worth offering.

# Two of the five values are read from the raw dictionary, and that is
not a shortcut

`pdfcer_core::forms::Field` models `/FT`, `/T`, `/V`, `/Kids` and the
widgets' rectangles, and it models **neither `/Lock` nor `/SV`** — the engine
reads both directly off the field dictionary inside its own signing path
(`EditSession::reusable_sig_field`), where they are consumed rather than
projected. So there is no projection to read them from, and this function
asks the object graph the same question the engine asks.

It asks only whether they are **present**, never what they say. Parsing
`/SV`'s seven `/Ff` bits here would be a second implementation of a rule the
engine enforces in full — see [`SigField::constrained`].

`pages` is the flattened page vector, passed rather than re-walked, so a
widget's `/P` can be turned into the page number an operator counts.

### `fn read`

`pages` is passed rather than re-derived, because `OpenDoc::pages` is
*"the flattened page vector, resolved once at open"* and re-walking the
tree here would be a second answer to a question the document already
has one answer to.

### `fn may_certify`

Pure, so both arms are asserted headlessly. §2d: both of the engine's
certification refusals are states of the **document**, knowable when the
window opens, so the option is absent with a sentence rather than offered
and then refused.

The order is the engine's own guard order — `AlreadyCertified` is
checked before `CertificationNotFirst` — so a document that is both
gets the same sentence here that it would get from the engine. Two
surfaces disagreeing about which of two true things to say is how an
operator learns to distrust both.

### `fn refusal`

Pure, so every arm is asserted headlessly rather than by driving a
window. See §4 of this module's header for the table and for the one
refusal that is this shell's rather than the engine's.

# Order

The order is *how early the operator can act on it*, not severity.
A pending redaction is one press away from being applied or cancelled,
so it is named first even though encryption is the harder wall: telling
somebody about the wall when the gate beside it is merely latched wastes
the one sentence they will read.

### `enum Refusal`

A closed set with one sentence each in [`crate::text::sign`]. Every variant
but [`Self::NotOnDisk`] mirrors a `SignApplyError` the engine would raise;
stating them here means the operator meets the refusal **instead of** a
form, rather than after filling one in.

### `enum CertifyBar`

Distinct from [`Refusal`] and it must stay distinct: every [`Refusal`]
closes the window, and each of these closes exactly one option on a window
that still works. Flattening them would turn *"you cannot be the author of
this document, but you can approve it"* into *"this document cannot be
signed"*, which is false and is the more expensive direction to be wrong in.

### `struct Identity`

⚠ **No `Debug`, derived or hand-written, and that is the point.** The
engine's [`Pkcs12Signer`] has a careful hand-written one that prints the
report and never the key, so a derive here would in fact be safe today. It
is still absent, because the thing that protects `crate::secret::Secret` is
that *the value cannot be formatted* — a property that survives somebody
adding a field. A `Debug` on the container is one refactor away from
printing whatever is put next to the signer.

Use [`Self::report`] for anything a human or a trace needs.

### `enum Placement`

THE DEFAULT IS INVISIBLE, AND IT IS A DECISION RATHER THAN A COPY OF
THE ENGINE'S.

`SignRequest::visible`'s own documentation says invisible *"is the default
for batch/CLI signing"*, which is an argument about batches. The argument
here is about what would be drawn on a CAD sheet the operator is about to
send out: **a box is applied content**, it renders exactly as the saved file
renders, and there is nothing provisional about it. So the default draws
nothing, the box is offered, and the copy on the control says what will be
inside it before it is chosen.

**What is inside it changed under this shell on 2026-09-06.** At the old
pin the appearance was *"a thin frame only — no text"*, and this type's
documentation and [`crate::text::sign::placement_note`] both said so. Engine
`Pass 10.14` (`187fa09`, in the pin since `d6b998f`) **composes** the signer
CN, the date, and the reason and location when given, in Helvetica, shrunk to
fit, and refuses a rectangle too small for them by name
(`SignApplyError::AppearanceOverflow`) before anything is staged. See §2b.

⇒ The default is still invisible, and the argument for that survives the
correction intact but is now a **different** argument: not *"the box would be
empty and read as a defect"* but *"a signature the reader shows in its own
panel does not need a stamp on the drawing, and a stamp is content the
operator did not draw."* An operator who wants the box now gets a box with
his name in it.

**The third arm is not a placement at all, and that is the point.**
[`Self::ExistingField`] names a box **somebody else already placed**; its own
`/Rect` and page decide where the appearance goes, and the engine refuses a
`visible` rectangle beside it by name. Modelling all three as one enum makes
the refused combination unrepresentable rather than reachable-and-explained.

⚠ **Not `Copy`**, because [`Self::ExistingField`] owns the field's name. The
name is carried rather than an index into [`Standing::empty_fields`] for the
reason every stale-index bug has: the vector is read once when the window
opens and the request is built later, and an index that survives into a list
that changed points at the wrong field silently, while a name that no longer
exists is refused by the engine, by name.

### `fn default_rect`

Every number here is stated rather than tuned, because this is content
written into the operator's file and *"about a third of the way up"* is not
a specification anyone can check. 36 pt is a half-inch margin — the same
inset a title block leaves and the value ISO 32000-1's own examples use;
180 × 60 is the box Acrobat's own signature appearance defaults to at 100 %,
which is the size an operator's eye already expects.

Bottom-**right** rather than bottom-left because a CAD sheet's title block
is bottom-right and a signature belongs beside it — and because the
alternative, bottom-left, is where every drawing frame in this operator's
own files puts its revision table.

⚠ It is clamped to the page: on a page smaller than 252 × 132 pt the box
would otherwise be placed partly or wholly outside the media box, which the
engine would accept and no reader would draw. The clamp is
[`Self`]-contained arithmetic on `media` rather than a refusal, because a
small page is not an error and a signature on it is still wanted.

### `struct Authored`

Everything on this struct is the operator's own words or the operator's own
choice. pdfcer infers nothing into a signature dictionary — the engine's
rule-4 note says the reason: *"the signing time, name, reason, location and
contact are the caller's words, written verbatim."*

### `fn write_to`

Temp file, then rename — `crate::protect::Prepared::write_to`'s
mechanism, taken deliberately. The destination may be the file the
operator has open, and a torn write there leaves them with neither the
signed document nor the one they started with.

# Errors

[`WriteFailure`] — the file system refused.

### `enum Outcome`

The handler produces this and hands it back through
[`crate::dialogs::DialogsState::sign_outcome`]. A single type with two
variants rather than a `Result`, because the *failure* side here is already
a finished operator-facing sentence — every producer of one has more context
than the dialog does about which of five things went wrong — and a `Result`
whose error is a `String` invites a caller to add its own wording on top,
which is how one event comes to be described twice.

### `fn prepare`

The one place `EditSession::sign` is called. See §3 for why it is the open
session and not a throwaway, and why nothing is undone afterwards.

# The reservation is the engine's default and is not offered as a control

`SignRequest::reserve` defaults to 12 KiB, which the engine's own note says
*"fits a SHA-256/RSA-4096 CAdES signature with a three-certificate chain
about three times over"*. It is not a question an operator can answer — the
number is a property of their certificate chain, which pdfcer has just read
and they have not — so asking would be handing them arithmetic. If it is
ever too small the engine refuses by name with **both** numbers, and
[`crate::text::sign::reservation_too_small`] states that the reservation is
fixed, so the refusal is a fact rather than an instruction the operator
cannot follow.

⚠ The alternative considered and rejected was **retrying automatically at a
larger reserve**. It would work, and it would mean the size of the hole in
the operator's file depended on a retry they were never told about. R8b
Rule 4: what is written is disclosed.

# Errors

[`PrepareFailure`] — the document is out of scope, or the engine refused.

### `fn signer_ref`

Private-in-spirit: it is `pub(crate)` rather than `pub` so that the key
operation is reachable from [`prepare`] and from nowhere a future module
might casually put it.
