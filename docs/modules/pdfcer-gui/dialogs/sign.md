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

## Item notes

### `const BODY_FLOOR`

Without a floor, a small window produces a scroll area that draws **nothing
at all** — `available_height()` minus a reservation goes negative, and a
negative `max_height` is a silently empty area rather than an error. The
About, OCR, print, protect and redaction dialogs all record the same trap.

### `enum Phase`

A state machine rather than several `Option`s, for
`crate::dialogs::redact::Phase`'s reason: the states are mutually exclusive
and an `Option` quadruple has combinations that would all compile and none
of which means anything.

### `fn fmt`

Two fields here touch a private key: [`Self::passphrase`], which is a
`String` only because `egui::TextEdit` binds to one, and
[`Self::identity`], which holds the key itself. A derived `Debug` would
print the first, and `crate::secret`'s header records exactly what that
costs — a `{:?}` anywhere on the path writes it into the trace file
`tools/ui-verify` keeps as evidence.

The **certificate's path is omitted too**, which goes further than
`crate::dialogs::protect`'s equivalent. A path is not key material; it
is a durable pointer at where somebody keeps their digital ID, and a
trace file is kept and shared. What is printed is whether one was
chosen.

### `fn open`

Cheap — a census and a `metadata` call. Nothing is computed that could
be wrong later: the identity does not exist yet and the bytes are not
produced until the press.

### `fn ready_to_confirm`

Pure, and the whole of the gate's rule, so every property of it is
asserted headlessly — `crate::viewer`'s standing split applied to the
control that attaches somebody's legal identity to a file.

The conditions, and each one is a different failure:

1. **A form is being filled in at all.** A refusal, a signing in flight
   and a finished write all have no confirm.
2. **An identity is OPEN** — not merely chosen, and not merely a
   passphrase typed. §1: the operator has seen whose certificate this is.
3. **The replace acknowledgement**, when and only when the operator has
   chosen to replace.

There is deliberately **no** condition on `/Reason` or `/Location`.
Both are optional in the standard, both are omitted when empty, and a
surface that required a reason would be inventing an obligation the
format does not impose — on a control where inventing obligations is how
people learn to type anything at all into the box.

### `fn placement`

Pure, so every arm is asserted headlessly — including the one that
matters most: *Existing* falling back to [`Placement::Invisible`] when
there is no field at that index.

The fallback is `Invisible`, never `Visible`, and the choice is a
safety one rather than an arbitrary default. `Invisible` writes
`/Rect [0 0 0 0]` and draws nothing; `Visible` would stamp a box with the
operator's name on a page he did not ask to have marked. When a surface
has to guess on an unreachable branch, it should guess toward *writes
less into the operator's file*.

⚠ Unreachable from the window — `Place::Existing` is only offered when
there is a selectable field and the index comes from the list that was
drawn — so this is the standing preference against panicking on a branch
a guard has already excluded, not a live path.

### `fn can_replace_original`

`is_file` rather than a flag, asked of the **file system**, exactly as
`crate::app::save::has_a_file` asks it: a second source of truth drifts,
and the failure when it does is writing over the wrong file.

### `fn choose_destination`

`crate::dialogs::redact::choose_destination`'s rule, pure for the same
reason. Without it, an operator could tick the box, think better of it,
select *a new file*, change their mind again, and arrive back at
*replace* with the button already live — the consent standing from a
decision they had explicitly withdrawn in between.

### `fn commit`

The destination is settled here — before the action — for
`crate::dialogs::protect::commit`'s reason inverted: there, the engine
runs before the picker so a refusal never arrives after a picker has
been filled in and dismissed. Here the engine call is in the handler and
cannot run first, so the picker comes first and the two known refusals
were already answered when the window opened. What is left that can fail
after the picker is a wrong passphrase — and the passphrase was already
proved correct by *Open certificate*.

### `fn disabled_reason`

R9: greying is only ever for temporarily unavailable, and it is
**always explained on hover**. `OPERATOR_REQUESTS.md` O77's sweep found
seven greyed controls with no explanation; this is the shape that
discharges it.

It stays HERE rather than moving to [`sections`] with the row that
draws it, because it is a statement about the window's gate — the same
gate [`Self::ready_to_confirm`] enforces two functions above — and the
two must be read together or they drift into disagreeing about which
condition is being explained.
