# `protect` — putting a password on a document, changing what it allows,
and taking the protection off


The split is `crate::redact`'s and it exists for the same reason: every rule
on this surface is a rule about **the operator's file**, and a rule that can
only be exercised by driving a window is a rule that gets asserted once, by
hand, and then drifts.

---

# 1. The three engine verbs, verified against the source

`D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs`, at the revision this crate
pins (`Cargo.lock`: `pdfcer-core 0.32.0`, `aa27596`). Checked at that commit
rather than at `main`, because `main` is 0.34.0 and what compiles here is the
pin:

```text
set_encryption   (&self,     &EncryptionSettings, &SaveOptions) -> Result<(Vec<u8>, SaveReport), EncryptError>
set_permissions  (&mut self, &EncryptionSettings, &SaveOptions) -> Result<(Vec<u8>, SaveReport), EncryptError>
remove_encryption(&mut self,                      &SaveOptions) -> Result<(Vec<u8>, SaveReport), EncryptError>
```

`docs/core-api/02-editing-and-saving.md` §1.5 states the same three, and the
two agree.

## The one place the engine's docs and the engine's source disagree

`set_encryption`'s own rustdoc instructs a caller to *"surface
[`EncryptionSettings::permissions_disclosure`]"* and names
`EncryptionSettings::saslprep_gap` beside it. **Neither item exists.** They
are associated **constants** and they are `SCREAMING_CASE`:
`EncryptionSettings::PERMISSIONS_DISCLOSURE` and
`EncryptionSettings::SASLPREP_GAP` — which is what `docs/core-api` names and
what compiles. Two broken intra-doc links, nothing more, and recorded here
because the build brief asked for every difference between the docs and the
source: a caller who trusted the rustdoc would look for a method, not find
one, and conclude the disclosure was not supplied.

# 2. Why NONE of the three verbs is called on the open session

This is the decision the whole module is shaped around, and it was taken on
evidence rather than caution.

Two of the three take **`&mut self`**, and what they mutate is not an edit —
it is the session's own record of what the document IS:

* `set_permissions` calls `self.base.clear_encryption()`;
* `remove_encryption` calls `self.base.clear_encryption()` **and**
  `self.trailer.remove(b"Encrypt")`.

Now read `crate::writer::save_incremental`'s encryption guard: a document
whose base is encrypted is refused with
`WriteError::EncryptedSaveUnsupported`, and that error's own doc explains
why in terms this shell must not undo — saving an encrypted document
verbatim would produce *"one that no reader can open, including pdfcer, and
it would look like a successful save."*

⇒ **So calling either mutating verb on the open session removes the guard
that stops the NEXT ordinary Save producing exactly that file.** The base
stops reporting itself as encrypted, `save_incremental` stops refusing, and
the operator's next `Ctrl+S` appends plaintext objects to a file whose
existing objects are AES ciphertext. Nothing would report a failure.

That is not a risk to be weighed against convenience. It is the one outcome
this project's rules forbid outright, so the open session is never handed to
these verbs.

## What is done instead — and it differs by job, because the facts differ

| the document is… | the verb | the session it is called on | do unsaved edits travel? |
|---|---|---|---|
| **not** encrypted | `set_encryption` (`&self`) | **the open one** | **yes** |
| encrypted | `set_permissions` / `remove_encryption` (`&mut self`) | a **throwaway**, loaded from the file with the owner password | there are none — see below |

The first row needs no ceremony at all: `set_encryption` takes `&self`,
mutates nothing, and applies `dirty_set()` — so an operator who has moved a
dimension and not saved gets that dimension in the protected file. This is
strictly better than re-reading the disk and it costs nothing.

The second row loses no work, and that is a fact about the engine rather
than a claim about this code. `pdfcer-core` **refuses every content edit on
an encrypted document by name** — the engine's own regression test says so:
`an_encrypted_session_still_refuses_a_content_edit`, whose comment reads
*"the guards are load-bearing the moment an encrypted document can carry a
session (which it now can)"*. And an encrypted document cannot be saved
either, by `EncryptedSaveUnsupported` above. So an open encrypted document
has no unsaved edits to carry: there is no verb that could have made one.

**The throwaway load is also the authentication.** Both mutating verbs
are owner-only and refuse `NotOwner { opened_as }`; the way this module finds
out whether the operator has the owner password is by **using it** —
`Document::load_with_password(path, Some(owner))`, then reading
`encryption().auth`. One act, no second code path, and the failure is the
engine's own rather than a guess made here. It also means pdfcer never has to
keep the password that opened the document, which is what
[`crate::secret::Secret`] exists to prevent.

# 3. Why the write is a destination CHOICE, and whose precedent that is

`crate::dialogs::redact`'s, followed rather than re-argued — the operator
settled this shape hours before this work started, on the redaction, in his
own words: *"why does it have to save to a new file right away? Why can't it
just wait on saving until I choose to save over the existing file or save as
a new file?"*

So the same three things hold here:

1. **A new file is the default** and [`suggested_path`] never proposes the
   source. A safe default is a mechanism; a warning is something to click
   past.
2. **Replacing the original is offered**, gated behind one extra
   acknowledgement that names the file, and it takes **no picker** — a
   picker pre-filled with the source is the shape of every accidental
   overwrite there has ever been.
3. **The write is atomic** — temp file, then rename ([`Prepared::write_to`]).

The half of his request the engine cannot express is the same half here as
there: *defer the write to a later Save*. There is no verb for it. All three
encryption verbs **return bytes**; none of them stages anything in a session,
and `EditSession` has no `replace_document`. Approximating it would mean
swapping a second session under the open document and silently discarding its
undo log, which `crate::app::save::save_as` refuses for the same reason.

# 4. What happens to the open document: nothing, and it is disclosed

Exactly `crate::dialogs::redact`'s outcome, and the divergence matters more
here because it is **invisible**: a redacted page looks different, and a
protected file looks identical to the one it came from.

So after a replace, the window is deliberately stale and
[`crate::text::protect::written`]'s replace form says so by name, telling the
operator which file to re-open. Rule 4: report separately, and do not pretend.

## Item notes

### `fn owner_session`

The authentication and the throwaway in one act — see §2. The document is
read from disk rather than from the open session because the two mutating
verbs take `&mut EditSession` and what they mutate would disarm
`save_incremental`'s refusal on the session the operator is still using.

### `enum Task`

Two commands, one window. The alternative — two windows — would put the
password fields, the destination choice, the disclosures and the atomic write
in two files, and the second copy is where a disclosure goes missing. What
the task decides is the **title**, which of the jobs is offered, and which
section the window opens on; everything below that is one implementation.

### `enum Job`

Four values rather than three, because *change the passwords* and *change
what it allows* reach the same engine verb (`set_permissions`) with opposite
intentions, and the difference is what the surface must protect: a password
change **preserves** the permission bits the document already has, and a
permission change **preserves** nothing about the passwords because the
engine cannot recover them (see [`Standing::preserved_grants`]).

### `fn edits_permissions`

Decides whether the permission list is drawn as **controls** or as a
read-back, and therefore whether
[`crate::text::security::permissions_are_advisory`] is drawn beside
controls the operator is about to use or beside a report.

### `fn needs_current_owner`

True for every job that acts on an already-protected document, which is
every job but the first — and it is the condition that decides whether
[`crate::text::protect::owner_password_note`], O119's third disclosure,
is on screen.

### `struct Standing`

The reason this type exists at all is one line of the build brief:
*"A permissions dialog that opens with everything ticked, on a document that
forbids printing, has told the operator a falsehood before he touches
anything."* Every control on the window is seeded from a field here, and
nothing on it has a hard-coded default.

### `fn refusal`

Pure, and the whole of R9's rule for these two controls: *no
placeholders — the control is absent or explained, never a button that
fails on press.* The controls stay on the ribbon, because whether THIS
document is signed is not known when the registry is built; the window
opens and states the refusal instead of drawing a form whose only
possible outcome is a failure.

### `fn jobs`

Pure, and the single source of what appears on the window — so the radio
group, the confirm control's label and the engine call cannot disagree
about what is being done.

### `fn preserved_grants`

This is what makes *change the passwords* a safe verb. It reaches
`set_permissions`, which takes a whole `EncryptionSettings` and re-derives
`/O`, `/U`, `/OE`, `/UE` and `/Perms` from scratch — so a caller that did
not supply the current bits would silently **grant everything** to a
document that had been restricting things, and the operator would have
changed a password and quietly unlocked the drawing.

`None` — the bit is not meaningful at this document's revision — becomes
**granted**, and that is the conservative reading in the direction that
matters. pdfcer writes `/R` 6, where all eight bits mean something, so
every bit must take a side. An `/R` 2 document's author did not decline
to permit form-filling; the concept did not exist to decline, and turning
their silence into a prohibition would invent a restriction they never
wrote. `PermissionBit::applies_at`'s own doc says exactly that.

### `fn initial_ticks`

This is deliberately **not** the same list as [`Self::grants`], and
the difference is a difference of tense. `grants` is *what this file
says today* and is drawn under
`crate::text::protect::permissions_now_heading`; this is *what the file
pdfcer is about to write will say*, and it is the seed for controls the
operator can move.

They differ in exactly one place, and only on a document some other
program wrote: a bit for which [`always_granted`] holds is forced on
here even when the document declines it, because the engine will grant
it on the way out no matter what this surface passes. Seeding the box
from the file would show an unticked control that becomes ticked in the
written result — a promise the program cannot keep.

### `fn always_granted`

`pdfcer_core::crypto::encrypt::assemble_permissions` implements the engine's
write-path rule **W19**, and its own doc states the clause verbatim:

> bit **10** — writers `shall` always set it to 1 for 1.7-reader
> compatibility, regardless of whether accessibility extraction is granted
> (at `/R` 6 the bit no longer gates it).

Bit 10 is [`PermissionBit::AccessibilityExtract`]. The engine sets it on
**every** file it writes, whether or not the caller listed it in
`EncryptionSettings::permissions`, and the read side then reports it as
granted — correctly, because the file does say so.

⇒ A tick-box for this bit would be a control the operator can clear and
which comes back ticked in the file that is written. That is the exact shape
of falsehood the build brief forbids — *"a permissions dialog that opens with
everything ticked, on a document that forbids printing, has told him a
falsehood before he touches anything"* — only worse, because it would happen
**after** he touched it. So the row is drawn as a fixed statement with
`crate::text::protect::accessibility_always_granted` beside it, and this
function is the single predicate both the drawing and
[`Standing::initial_ticks`] consult.

It takes the whole [`PermissionBit`] and matches exhaustively rather than
comparing against one variant, so a future engine rule that pins a second
bit is a change in one place and a compile error if the enum grows.

### `struct Passwords`

`Secret` rather than `String` the moment they leave the text fields, for
`crate::dialogs::password`'s reason and its module's rule: the value never
enters a trace, a queue or an `Action` unwrapped. Everything traced about a
password on this surface is its **length** and whether it is ASCII.

### `fn write_to`

Temp file, then rename — `crate::redact::PreparedRedaction::write_to`'s
mechanism, taken deliberately. The destination may be the file the
operator has open, and a torn write there leaves them with neither the
protected document nor the one they started with.

The temporary is removed if the rename fails, for the same reason it is
there: a half-written copy of the operator's drawing sitting beside it
under a name nothing will ever open again is an artefact, not a recovery.

# Errors

[`WriteFailure`] — the file system refused.

### `fn prepare`

The one place any of the three engine verbs is called, and §2 of this
module's header is the whole of why the two branches differ.

# Errors

[`PrepareFailure`] — the document is out of scope, the owner password did
not open the file, it opened as somebody other than the owner, or the engine
refused the verb by name.

### `fn suggested_path`

The standing rule for every write that produces a second document, and the
suffix depends on the job because the two files it can produce are opposites:
a protected one and an unprotected one. Suggesting `-protected` for a removal
would name the file after the thing it no longer is.
