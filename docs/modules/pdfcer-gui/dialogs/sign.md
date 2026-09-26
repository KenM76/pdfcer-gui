# `dialogs::sign` — the window that puts the operator's signature on a
document

The surface for [`crate::sign`]; read that module's header first, because
every rule this window enforces is argued there and none of it is repeated
here. What this file adds is the **order things are asked in**, and that
order is the design.

## 1. THE IDENTITY IS OPENED BEFORE ANYTHING ELSE IS OFFERED

The window has two states while it is being filled in, and they are not
cosmetic:

```text
  ┌ certificate not yet opened ─────────────────────────────────┐
  │  Choose certificate…   [ path ]                             │
  │  Passphrase            [ •••• ]                             │
  │  [ Open certificate ]                                       │
  └─────────────────────────────────────────────────────────────┘
              │  the container verified, the key came out
              ▼
  ┌ identity on screen ─────────────────────────────────────────┐
  │  Signed by: CN=…            ← read out of the FILE          │
  │  Key: RSA-2048, chain of 3                                  │
  │  Integrity: checked / NOT checked                           │
  │  Reason / Location / page / destination / [ Sign and save… ]│
  └─────────────────────────────────────────────────────────────┘
```

**Nothing below the identity exists until the identity does.** That is not
progressive disclosure for tidiness — it is the one guard this surface can
offer against the mistake that matters. Signing is an act of identity, and
the only thing standing between "I picked a file" and "I attached my name to
a legal document" is the operator reading whose certificate came out of the
file. A form that let them fill in a reason, choose a destination and press
*Sign* with the certificate still unopened would put the identity check
**after** the decision, where it is a formality.

It is also the passphrase check, and it costs nothing extra: a wrong
passphrase is `Pkcs12Error::MacMismatch`, arriving at the moment the
operator is looking at the passphrase box rather than three fields later.

## 2. Why the write is an `Action` and not a call

`EditSession::sign` takes **`&mut EditSession`** and a dialog body is handed
`&OpenDoc`. That is not an inconvenience to route around — it is the rule
that stops a window mutating a document while the frame that drew it is
still reading one. `Arc::get_mut` is the funnel's second step and it fails
outright while the render worker holds its clone, so a mutation attempted
from inside a draw would be *silently declined*, which is the worst of the
available failures.

⚠ [`crate::dialogs::protect`] does the opposite — it calls the engine from
inside `commit` — and the difference is real rather than inconsistency:
`set_encryption` takes `&self`. Every verb that takes `&mut` reaches the
session through [`crate::app::actions::Action`], and this one does too.

## 3. THE PRIVATE KEY DOES NOT TRAVEL IN THE ACTION QUEUE

[`Action`] derives `Debug`, `Clone` and `PartialEq`. Every one of those is
wrong for a private key:

| trait | what it would mean |
|---|---|
| `Debug` | the key can be formatted into a trace `tools/ui-verify` keeps on disk |
| `Clone` | copies of the key material nobody is counting |
| `PartialEq` | a **non-constant-time comparison** over secret bytes |

So the loaded [`crate::sign::Identity`] stays in this struct, and the action
carries the certificate's **path** and the passphrase as a
[`crate::secret::Secret`] — a type whose whole guarantee is that its value
cannot be formatted, and which this enum already carries for
`Action::OpenWithPassword`.

⇒ The consequence, stated because it looks like waste: **the `.pfx` is read
and parsed twice.** Once here, whose job is to show the operator whose key
it is, and once in the handler, whose job is to sign. That is the right
trade — the alternative is smuggling key material through a queue that
derives three traits it must not have — and the second read is not
redundant: it is the read that actually signs, so a file that changed under
the operator between the two is caught rather than assumed away.

## The section headings are NOT `.strong()`

`crate::dialogs::protect`'s §6, taken rather than re-argued and caught by
`tools/gates/check-strong-text.sh` on the first draft of this file exactly
as it was on that one. egui has no separate role for emphasised text, so
`.strong()` resolves to the **accent-filled widget** colour — pale text on a
pale panel (`DEFECTS.md` D11). The hierarchy here is carried by layout and
wording instead: a rule and a gap between sections, headings that are
phrases (*"Your certificate"*, *"What the signature will say"*, *"On the
page"*) rather than one-word captions, and the muted `.small()` notes below
them to contrast against.

## 4. What comes back

The handler reports through [`super::DialogsState::sign_outcome`], which is
the same two-step every dialog here uses for anything that happens outside
its own closure. There is no polling and no shared cell: the app owns both
the dialog and the handler, so the outcome is handed over rather than
looked for.
