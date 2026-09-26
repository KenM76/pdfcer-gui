# `text::protect` — every operator-facing string on the two Security
controls that **write** protection into a file

`OPERATOR_REQUESTS.md` **O119**, approved 2026-09-04 with the instruction
that ended the question: *"yes add encryption and permissions … Always add
new features. never ask. just do."*

## Why this is a second module rather than more of [`crate::text::security`]

Three reasons, and the first is the one that decides it.

1. **The two modules make opposite kinds of claim.** `text::security` is the
   READ side: it reports what a document already says about itself, and its
   header's standing rule is that *"it makes claims about what protects a
   document, and a wrong one is worse than silence."* This module is the
   WRITE side: every sentence here describes something pdfcer is about to
   **do to a file**, and the failure mode is not a wrong report, it is a
   wrong file. Mixing an "it is like this" catalogue with a "this is what
   will happen" catalogue would put the two under one reviewer's eye with
   one standard, and they need two.
2. **R2.** `text::security` is 375 lines of dense argument and several of
   its neighbours in `text/` are at or near the 1,500-line ceiling. The
   build brief for this work names a new module for exactly that reason.
3. **Everything the read side already wrote is REUSED rather than
   restated.** [`crate::text::security::permissions_are_advisory`],
   [`crate::text::security::permission_name`],
   [`crate::text::security::permission_state`],
   [`crate::text::security::cipher_line`],
   [`crate::text::security::auth_line`] and
   [`crate::text::security::not_encrypted`] are all called from the protect
   dialog verbatim. **A second wording of one fact is the defect**, and it
   is the defect `text::security`'s own header names about the engine and
   the CLI. So the split is by *kind of claim*, not by subject, and nothing
   is duplicated across it.

## The three disclosures, and why they are three

`OPERATOR_REQUESTS.md` O119 lists three things the operator said "change the
answer" before he gave one. All three are on screen, and the surface may not
ship without them:

| # | fact | where it comes from | where it is drawn |
|---|---|---|---|
| 1 | **a permission is a request, not a lock** | the ENGINE's own sentence, already catalogued at [`crate::text::security::permissions_are_advisory`] | at the top of the permission list, in the danger role, on every job that writes permission bits |
| 2 | **a signed document is refused** | [`signed_refusal`] here | instead of the whole form — the dialog opens, states it, and offers nothing |
| 3 | **re-permissioning needs the owner password** | [`owner_password_note`] here | above the current-owner-password field, on every job that touches an already-protected file |

Number 1 is the important one and it is **not re-worded here**. The engine
supplied it, the CLI prints it, `text::security` catalogued it, and
`EncryptionSettings::PERMISSIONS_DISCLOSURE` is the same sentence in the
engine's own source. A UI that presented permissions as enforcement would be
lying, and softening the clause *"a request, not a lock"* is exactly the edit
a marketing instinct makes.

## What is deliberately NOT said

No sentence here promises that a protected document is safe, secure, or
locked. The only true version of that claim is about the **user password**
and nothing else, and [`crate::text::security::permissions_are_advisory`]
already says it in the engine's own words.

## Item notes

### `fn group_file_security`

It lives here rather than in [`crate::text::ribbon`] with the other group
captions, on the precedent [`crate::text::acrobat::file_open_in_acrobat`]
set: when a feature's copy is one subject and one module, the caption is
part of that subject, and splitting three words off into another file buys
nothing but a second place to look. `crate::shell::manifest::file` calls it
by its full path, so the seam is visible at the call site.

### `fn file_encrypt`

# The label is a verb and the ellipsis is a promise

*Encrypt…* rather than *Password…*, because the label has to be true of all
three things the control does — set a password, change it, remove it — and
*Password…* reads as "set one". Encryption is what the file gains or loses;
the password is how it is keyed.

**The tooltip names all three jobs**, because the control's most surprising
property is that the one button also takes protection OFF. A user who wants
to unprotect a drawing will not look under a button called Encrypt unless it
says so, and the alternative — three ribbon controls for one subject — is
three chances to press the wrong one.

And it names the refusal in the same sentence. A signed document is
refused by the engine, by name, and finding that out by pressing is the R9
failure this project has paid for: *an unavailable capability renders
nothing, and a button that fails on press is worse than either.* The control
stays present — whether THIS document is signed is not known when the ribbon
is built — so the hover says what will happen.

### `fn file_permissions`

The tooltip's **first job is the disclosure**, not the description. This
is the one control in pdfcer whose plain reading is false: a list of
tick-boxes labelled Print, Copy and Change looks exactly like a set of
locks, and it is not one. The full sentence is on screen the moment the
window opens ([`crate::text::security::permissions_are_advisory`]); the
tooltip carries the short form so an operator who only ever hovers still
meets it.

### `fn standing_heading`

This section exists because of one line in the build brief, and it is
the strongest requirement on this surface: *"A permissions dialog that opens
with everything ticked, on a document that forbids printing, has told the
operator a falsehood before he touches anything."*

### `fn job_change`

It says **passwords**, plural, and *keep* what it keeps. The one thing an
operator fears about this button is that it silently re-opens a document
they had restricted, and saying so at the control is cheaper than a receipt
that says it afterwards.

### `fn job_remove_note`

Not a scold. The operator asked for this verb by name; what the sentence
adds is the fact a label cannot carry — that the *result* is a file anybody
can open, which is the point and is also the thing to be sure of.

### `fn passwords_explained`

The build brief made this explicit: *"Owner and user passwords are different
things; do not collapse them into one field without saying what you did."*
They are not collapsed, so what is owed instead is an explanation of why
there are two boxes where every other program in the operator's day has one.

It is two sentences and each carries one fact: what each password *does*,
and what an empty user password *means*. The second is not a footnote —
`EncryptionSettings`' own doc calls an empty user password a
**permissions-only document**, and it is a genuinely useful thing to want
(the drawing opens with no prompt and still declares what it allows). An
operator who left the box blank without being told would think they had
failed to protect anything.

### `fn owner_password_note`

The engine asked for this to be surfaced by name, in its 2026-09-03 reply:
*"`AuthKind` tells you which one opened the file — surface that, because
`remove_encryption` will refuse a user-authenticated session and the operator
should see WHY before pressing it."*

It is drawn **above the field**, not after a refusal. A refusal that
arrives on press is a program that knew the answer and waited.

And it says *typed here even if you already used it*, because the honest
alternative is worse. pdfcer does not keep the password that opened the
document — [`crate::secret::Secret`] exists so it does not linger — so a
session opened with the owner password still cannot re-key without being
given it again. Without this clause the operator meets a field they believe
they have already filled in and concludes the program has lost track.

### `fn permissions_start_open`

The build brief's rule is that the dialog must show the CURRENT state before
offering to change it, and on an unprotected document the current state is
*everything is allowed* — there is no `/Encrypt` dictionary, so there is no
`/P`, so nothing is being declined. Eight ticks is therefore the true
read-back and not a convenient default, and saying so is what stops it
looking like one.

### `fn permission_becomes_stated`

`Permissions::granted` returns three values and the third one is not
"refused". `text::security::permission_state` already renders it; what this
adds is the consequence *for the change about to be made*: pdfcer writes
`/R` 6, where every one of the eight bits means something, so a box that is
currently "not stated" will be stated after this.

### `fn permission_row`

A catalogued function rather than a `format!` at the two call sites, and
the reason is not bookkeeping. The window draws this shape **twice** — once
under *"What it allows today"*, reporting the document's own three-valued
answer, and once in the editable list, where the one row that cannot be a
tick-box carries [`accessibility_always_granted`] instead. Two `format!`s
would be two separators, and the day one of them became a colon the two
lists would stop reading as one kind of thing.

The name comes from [`crate::text::security::permission_name`] and the state
from [`crate::text::security::permission_state`] or from this module; this
function owns only the join.

### `fn accessibility_always_granted`

`pdfcer-core` sets bit 10 on **every** file it writes, regardless of what the
caller asked for — its rule W19, for compatibility with PDF 1.7 readers. So
pdfcer cannot produce a document that declines accessibility extraction, and
a tick-box the operator could clear would come back ticked in the file.

The sentence says what the program cannot do **and** why the limitation is
benign, in that order. An operator who reads only the first clause has been
told the truth; one who reads both knows it is not worth working around.

### `fn encrypt_metadata_note`

The default is ON, and the reason to turn it off is a real one rather than
an expert's curiosity: a search indexer that cannot read the title and author
of a drawing cannot find it. That is the trade, stated as a trade.

### `fn signed_refusal`

The engine refuses it by name (`EncryptError::SignedDocument`) and this
surface refuses it *before* the form is drawn — there is nothing to fill in,
because there is no answer that would work.

# Why the sentence explains the mechanism rather than just the rule

Because the rule sounds arbitrary and the mechanism does not. "Encryption is
not allowed on signed documents" invites the operator to look for a setting.
*"It rewrites every byte the signature covers, so the signature would no
longer match"* is a fact about how signing works, and it tells them the real
remedy: protect first, sign second.

It names the count, because a document with one approval signature and a
document with a certification plus four approvals are different problems and
the operator is the one who knows which theirs is.

### `fn not_encrypted_refusal`

Not an empty list and not eight greyed boxes. Permissions live inside the
`/Encrypt` dictionary — an unprotected document does not permit everything,
it *says nothing*, and drawing eight ticked boxes would be this surface
inventing a declaration the file never made.

It names the other control, because the operator's next move is a real one
and a refusal that does not say what to do instead is half a sentence.

### `fn no_file_refusal`

Reachable only in theory, and refused rather than unwrapped. Changing the
protection on an already-protected document is done by re-opening the FILE
with the owner password (see [`crate::protect`] for the whole argument), and
a document created in this session has no file to re-open. A document created
in this session is also never encrypted, so the two conditions cannot both
hold — which is exactly why this is a named refusal rather than a `panic!`
on an "impossible" branch.

### `fn redaction_pending_refusal`

**Deliberately the same shape as
[`crate::text::sign::refusal_redaction_pending`], because it is the same
engine refusal reaching a second surface.** One step, not a wall: the
operator armed the removal, and Edit ▸ Redact holds both the button that
finishes it and the button that calls it off. Two surfaces describing one
refusal in two different voices is how an operator comes to believe they are
two different problems.


It names **encrypting** rather than the three operations the engine's
message lists, because the reason is identical for all three and the
operator only ever pressed one of them. Naming the other two would describe
a decision they did not make.

### `fn engine_refusal`

One function with a match rather than six strings at six call sites,
because the whole value of `EncryptError` is that every variant names
something the operator can act on, and a `to_string()` of the engine's own
message would put an implementer's sentence in front of a draughtsman.

### `fn not_owner`

It names which password DID work, which is the fact that turns a dead end
into a next step: an operator told only *"wrong password"* re-types the one
they have, and an operator told *"that is the user password"* goes and finds
the other one.

### `fn reopen_failed`

It carries the engine's own detail rather than flattening every failure to
"wrong password", for `crate::dialogs::password`'s reason: pdfcer reports a
non-ASCII password that it cannot normalise as a **different** error
precisely so the operator is not sent to re-check a password that was
correct.

### `fn owner_password_required`

Refused rather than allowed, and this is a decision the standard does not
make for us: `EncryptionSettings` will happily take an empty owner password.
A document with one is a document whose protection **anybody can remove**,
which is the opposite of what the operator pressed the button for, and it
would be an empty box's silent consequence rather than a choice.

### `fn passwords_must_differ`

Refused for the reason that makes the permission list mean anything: the
owner password ignores `/P` entirely, so if it is also the password that
opens the document, every reader authenticates as owner and the permissions
are decoration. The engine does not enforce this (*"because the standard does
not"*), so the surface does.

### `fn confirm_disabled`

`OPERATOR_REQUESTS.md` O77's sweep found seven greyed controls with no hover
explanation, and the reasoning `crate::text::redact::confirm_disabled`
records applies unchanged: several different conditions gate this one button
and they appear at different times, so *"fill in the form"* would be vague
exactly when it matters.

The flags are **outstanding** conditions, not satisfied ones, and each is
computed from the same expression that decides whether its control is drawn.

### `fn destination_heading`

The whole destination mechanism is `crate::dialogs::redact`'s, followed
deliberately rather than re-invented — see [`crate::dialogs::protect`]'s
header. The wording differs only where the act differs: a redaction destroys
content, and this replaces a file.

### `fn destination_replace_tooltip`

Softer than the redaction's equivalent, and deliberately: nothing here
destroys content. What it destroys is the **unprotected copy**, and on the
remove-protection job the opposite — the protected copy. Both are recoverable
only by having kept the other one, which is what the sentence says.

### `fn suggested_suffix`

A suggestion, and it is never the source file — the standing rule this
project applies to every write that produces a second document
(`crate::text::redact::suggested_suffix`,
`crate::text::files::save_copy_suffix`). Here the reason is milder and still
real: the two files differ only in their protection, and two identical-looking
drawings one of which is protected is the pair an operator most needs told
apart by name.

### `fn confirm_button`

The label is the consequence and the ellipsis is a promise that a further
question is coming — `crate::text::redact::confirm_button`'s rule, and the
same one decides the replace form below.

### `fn written`

It carries the fact the operator would otherwise discover by looking at a
window that disagrees with the file: **the open document is unchanged.**
This is `crate::dialogs::redact`'s ruling and it is stronger here, because
the divergence is invisible — a redacted page looks different, and a
protected file looks identical.

The replace form names the file to re-open. The new-file form does not need
to, because nothing the operator is looking at has become wrong.

### `fn saslprep_gap`

The engine hands this over as `EncryptionSettings::SASLPREP_GAP` and asks for
it to be shown when `has_non_ascii_password()` is true. It is **conditional**
for `crate::dialogs::redact`'s standing reason about acknowledgements: a
warning that is always on screen is a warning nobody reads, and this one is
irrelevant to the overwhelming majority of passwords.

It is a warning rather than a refusal, because the password may well be
perfectly interoperable and pdfcer cannot know. Refusing every accented
character would be this program declining to write a file the standard
permits.
