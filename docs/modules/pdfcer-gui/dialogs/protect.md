# `dialogs::protect` — the window behind **Encrypt…** and **Permissions…**

`OPERATOR_REQUESTS.md` **O119**, approved 2026-09-04: *"yes add encryption
and permissions"*, under the standing instruction *"Always add new features.
never ask. just do."*

This is the `Ui` half of the feature. Everything that can be decided without
one — what the document says today, which jobs it may be offered, which
engine verb a choice reaches, and the atomic write — is
[`crate::protect`], and that split is the whole reason the rules on this
surface are asserted headlessly rather than by driving a window once and
hoping.

---

# 1. The window's shape follows one sentence of the build brief

> *"Show the document's CURRENT state before offering to change it. A
> permissions dialog that opens with everything ticked, on a document that
> forbids printing, has told him a falsehood before he touches anything."*

So the body is **two sections in a fixed order**, and the order is not a
layout preference:

| § | heading | what it is | can the operator move it? |
|---|---|---|---|
| 1 | [`crate::text::protect::standing_heading`] — *"This document, as it is now"* | a **read-back**: the cipher, which password opened it, and every one of the eight permission bits with the document's own three-valued answer | no |
| 2 | [`crate::text::protect::change_heading`] — *"What to change"* | the job, the passwords, the permission ticks, the destination | yes |

Nothing in §1 is a control and nothing in §2 has a hard-coded default —
every tick is seeded from [`crate::protect::Standing::initial_ticks`], which
reads the file. On an unprotected document that read-back is *eight ticks*,
and [`crate::text::protect::permissions_start_open`] says so **in words**,
because eight ticks that are true and eight ticks that are a convenient
default look identical.

# 2. The three disclosures O119 named, and where each one is

The operator listed three things he already knows and would notice missing.
All three are on screen, none of them is behind a hover, and none of them
waits for a press:

| # | the fact | drawn where | drawn when |
|---|---|---|---|
| 1 | **a permission is a request, not a lock** — [`crate::text::security::permissions_are_advisory`], the ENGINE's own sentence, in the danger role | at the head of the permission list, **above** the tick-boxes | on every job that writes permission bits |
| 2 | **a signed document is refused** — [`crate::text::protect::signed_refusal`] | **instead of** the entire form | whenever the document carries a signature |
| 3 | **re-permissioning needs the owner password** — [`crate::text::protect::owner_password_note`] | **above** the current-owner field | on every job that touches an already-protected file |

Disclosure 1 is not re-worded here and must not be. The engine supplies it
as `EncryptionSettings::PERMISSIONS_DISCLOSURE`, the CLI prints it, and
`crate::text::security` catalogued it. Two surfaces wording one limitation
differently is worse than either wording.

Disclosure 2 is **R9** in its strongest form. The ribbon controls stay
present — whether *this* document is signed is not known when the registry is
built — so the window opens, states the refusal, names the count, explains
the mechanism (*it rewrites every byte the signature covers*) and offers
nothing. There is no greyed form behind it and no button that fails on press.

# 3. Saving: `dialogs::redact`'s answer, followed rather than re-invented

**This is deliberate and it is stated rather than left to be noticed.**
Protecting a document rewrites every byte, exactly as applying a redaction
does, so it raises exactly the question the operator settled hours earlier on
that surface, in his own words:

> *"why does it have to save to a new file right away? Why can't it just wait
> on saving until I choose to save over the existing file or save as a new
> file?"*

A second answer to a settled question would be the defect. So the mechanism
here is `crate::dialogs::redact`'s, part for part:

| | redaction | protection |
|---|---|---|
| default destination | a new file, chosen in the picker | **the same** |
| suggestion | never the source (`-redacted`) | **the same** (`-protected` / `-unprotected`) |
| replacing the original | offered, **one extra acknowledgement** naming the file, **no picker** | **the same** |
| the write | temp file, then rename — atomic | **the same** ([`crate::protect::Prepared::write_to`]) |
| confirm control | the label IS the consequence; an ellipsis promises a picker, naming the file promises none | **the same** |
| the open document afterwards | untouched, and the outcome sentence says so | **the same**, and it matters more — see §5 |

Replacing takes **no picker**, and that is the deliberate half. A picker
pre-filled with the source is a dialog whose safe answer is to change the
field, which is the shape of every accidental overwrite there has ever been.
The consent is taken before the click, in words, at a control the operator
had to select.

The half of his request the engine cannot express is the same half as
there — *defer the write to a later Save*. All three encryption verbs
**return bytes**; none stages anything in a session, and `EditSession` has no
`replace_document`. Approximating it would mean swapping a second session
under the open document and silently discarding its undo log, which
`crate::app::save::save_as` refuses for the same reason.

# 4. Why this dialog does not push an `Action`

[`super`]'s rule: a dialog uses the action funnel when it edits **this**
document, and this one never does. Every job here produces *bytes on disk*.
The open session is not mutated — see [`crate::protect`] §2 for the argument,
which is load-bearing rather than cautious: two of the three engine verbs
take `&mut EditSession` and what they clear is the guard that stops the next
ordinary `Ctrl+S` writing plaintext objects into a file of AES ciphertext.

# 5. After a replace the window is deliberately STALE, and it says so

`crate::dialogs::redact`'s outcome, and the divergence matters more here
because it is **invisible**: a redacted page looks different, and a protected
file looks identical to the one it came from. So
[`crate::text::protect::written`]'s replace form names the file to re-open.
Rule 4: report separately, and do not pretend.

# 6. The section headings are NOT `.strong()`, and that is deliberate

Every heading here was written `RichText::new(…).strong()` in the first
draft, and `tools/gates/check-strong-text.sh` caught all six. Its rule, and
`DEFECTS.md` D11 behind it: egui has no separate role for emphasised text,
so `.strong()` resolves to the **accent-filled widget** colour — which on an
ordinary panel is pale text on a pale background. Six labels have already
shipped that way once, and the Settings window repeated it three days after
the rule was written.

So the hierarchy here is carried by **layout and wording** instead: each
section is separated by a rule and a gap, the two headings are full phrases
(*"This document, as it is now"*, *"What to change"*) rather than one-word
captions, and the muted `.small().weak()` notes below them are what the
headings contrast against. A reader who wants to re-emphasise one of these
should read D11 first — in every observed case the label read **better**
without it.

# 7. Document-scoped, like every dialog that holds unsaved bytes

Closing the document discards them. A protected copy of a file nobody is
looking at any more, derived from a permission census that can no longer be
checked, is not a saving.

## Item notes

### `const REGION_STANDING`

Declared **unconditionally whenever the form is drawn**, because its absence
from a trace is the evidence for the build brief's own requirement: a form
with no standing section is a dialog that offered to change something it
never reported.

### `const BODY_FLOOR`

Without a floor, a small window produces a scroll area that draws **nothing
at all** — `available_height()` minus a reservation goes negative, and a
negative `max_height` is a silently empty area rather than an error. The
About, OCR and redaction dialogs all record the same trap.

### `enum Phase`

A state machine rather than several `Option`s, for
`crate::dialogs::redact::Phase`'s reason: the states are mutually exclusive
and an `Option` quadruple has combinations that would all compile and none of
which means anything.

### `enum Destination`

`crate::dialogs::redact::Destination`, and the reasoning there is this
type's reasoning — see §3 of this module's header for the part-for-part
correspondence and for the one half of the operator's request the engine
cannot express.

### `fn fmt`

Five fields of this struct hold a password the operator typed. A derived
`Debug` would print all five, and `crate::secret`'s header records
exactly what that costs: *"a `{:?}` on an action carrying a password
writes it into the trace file `tools/ui-verify` keeps as evidence."*

The passwords are `String` rather than [`Secret`] here only because
`egui::TextEdit` binds to a `String`, so the type cannot do the
protecting and this impl must. It prints the LENGTHS, which is what a
diagnosis of *"my password is not being accepted"* actually needs.

### `fn open`

Cheap — a `Standing::read` and a signature census, no rewrite. Contrast
`crate::dialogs::redact::RedactDialog::open`, which runs a full removal;
there is nothing here that could be computed before the passwords exist.

### `fn ready_to_confirm`

Pure, and the whole of the gate's rule, so every property of it is
asserted headlessly — `crate::viewer`'s standing split applied to the
control that can overwrite the operator's file.

The conditions, and each one is a different failure:

1. **A form is being filled in at all.** A refusal or a finished write
   has no confirm.
2. **The current owner password is present**, on every job that acts on
   an already-protected document — O119's third disclosure, enforced
   rather than merely printed.
3. **The new owner password is present**, on every job that sets one. A
   blank owner password makes a document whose protection anybody can
   remove; `EncryptionSettings` allows it and this surface does not —
   see [`crate::text::protect::owner_password_required`].
4. **Both copies of both new passwords match.** A password typed wrong
   twice is a document nobody can open.
5. **The two new passwords differ.** The owner password ignores `/P`
   entirely, so if it also opens the document every reader authenticates
   as owner and the permission list is decoration. The engine does not
   enforce this *"because the standard does not"*, so the surface does.
6. **The replace acknowledgement**, when and only when the operator has
   chosen to replace.

### `fn gates`

Outstanding rather than satisfied, and computed from the same
expressions that decide whether each control is drawn — so the
disabled-hover sentence can never send the operator to look for a field
that was never on screen. `OPERATOR_REQUESTS.md` O77's sweep found seven
greyed controls with no explanation; this is the shape that discharges
it, taken from `crate::text::redact::confirm_disabled`.

### `fn can_replace_original`

`is_file` rather than a flag, asked of the **file system**, exactly as
`crate::app::save::has_a_file` asks it and for the reason recorded
there: a second source of truth drifts, and the failure when it does is
writing over the wrong file.

### `fn choose_job`

Pure-ish and a method rather than a line inside the radio group, so
the rule can be asserted headlessly. Selecting *remove the protection*
and then going back to *change the passwords* must not leave the ticks
wherever a previous job's editing left them — the seed is always
[`Standing::initial_ticks`], i.e. always the file.

### `fn granted`

[`always_granted`] bits are forced in regardless of the tick, so this
list is what the written file will actually say rather than what the
controls happen to show. The two agree by construction because
[`Standing::initial_ticks`] forces the same bits on, but forcing it here
too means a future edit to the drawing code cannot make them disagree.

### `fn standing_section`

Drawn first and always, because of the build brief's own sentence: a
dialog that offers to change something it has not reported has told the
operator a falsehood before he touches anything.

### `fn has_non_ascii_password`

Asked of what is in the boxes rather than of an `EncryptionSettings`
that does not exist yet, so the warning appears **while typing** rather
than after the press. The engine's own predicate is
`EncryptionSettings::has_non_ascii_password`, and
[`crate::protect::prepare`] calls that one for the trace line — two
readings of one fact, taken at two moments, which is why this one is
spelled out rather than borrowed.

### `fn commit`

The two-path shape and the asymmetry between the paths are
`crate::dialogs::redact::commit`'s, and §3 of this module's header is
the whole of why they are copied rather than re-argued:

| destination | how the path is obtained | what stands between the click and the write |
|---|---|---|
| [`Destination::NewFile`] | the save picker, suggesting `-protected` / `-unprotected` | the picker itself, plus the OS's own overwrite prompt |
| [`Destination::ReplaceOriginal`] | [`Self::source`], **no picker** | a checkbox naming the file, and a confirm button whose label names it too |

The engine call happens **before** the picker on the new-file path,
deliberately. Every failure this surface can meet — a wrong owner
password, a signed document the census missed, an unreachable CSPRNG —
is discovered before the operator is asked to name a file, so a refusal
never arrives after a picker has been filled in and dismissed.

### `fn failure_line`

Free rather than a method, so the mapping from every failure the model can
report to the wording the operator reads is one pure function a test can
drive — and so a new [`PrepareFailure`] variant is a compile error here
rather than a silent fall-through to a catch-all.

### `struct Gates`

A named struct rather than five positional `bool`s, because five `bool`s at
a call site is where a future edit swaps two of them and every test still
passes.

### `fn job_label`

Free rather than a method on [`Job`], because [`Job`] lives in
`crate::protect` and that module holds no operator-facing strings — the
project's standing seam between the model and `text/`.

### `fn file_name_of`

The name rather than the whole path, for `crate::dialogs::redact`'s reason:
every sentence that needs one is read in a window about 700 pt wide and a
Windows path is routinely longer than that. The full destination is on the
trace line [`crate::protect::Prepared::write_to`] emits.
