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
