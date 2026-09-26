# `pdfcer-gui/app/dispatch/security`

`app::dispatch::security` — the File ▸ Security band's commands

## The subject, and why the module is not called `protect`

*Protection* describes encrypting and re-permissioning and not signing — a
signature protects nothing, it asserts authorship. The module is named after
what all of its commands share instead: the **File ▸ Security band**. Each
writes something into the file that is about the file rather than about any
page, none is an undoable content edit, and each produces a new document
rather than changing the one on screen.

`OPERATOR_REQUESTS.md` **O119**: *"yes add encryption and permissions"*.
`file.encrypt` and `file.permissions` open one window
([`crate::dialogs::protect`]) with two starting points.

# Why this is a module and not arms in [`super`]

**R2, on [`super::panels`]' precedent.** The encryption commands are the
only ones in the program that **write a whole new file out of the open
document's encryption**; they share a window, a `Task` and a set of
disclosures, and none of that has anything to say to the rest of
`dispatch.rs`.

⇒ **R2's line ceiling is not a budget to spend down to; it is a signal that
a file has stopped being one subject.** Compressing prose to get back under
it answers the number and not the signal.

The alternative — an exemption in `tools/gates/check-file-size.sh` — is
explicitly an operator decision, not a build session's, and the gate says so
in its own failure text. Splitting is what the rule asks for.

# What [`super`] keeps

One guard arm:

```ignore
id if security::claims(id) => self.dispatch_security(id),
```

…which is [`super::panels`]' arrangement precisely, including the pairing of
a `claims` predicate with a dispatcher over the same list. The pair is
pinned by [`tests::the_guard_and_the_dispatcher_claim_the_same_ids`], so a
Security command added to one and not the other fails a named test rather
than becoming a control that traces `command-unimplemented`.

# What this dispatch deliberately does NOT decide

**Whether the document is signed.** Neither the registry predicate
(`doc.open`) nor these arms ask, and that is the R9 ruling rather than an
omission: whether *this* document carries a signature is not known when the
command registry is built, so the control stays present and the **window**
refuses — by name, with the signature count, explaining that protecting
rewrites every byte the signature covers. A click or a chord on a signed
document therefore produces a sentence about the operator's document rather
than a failure, which is R9's *explained* branch: *the control is absent or
explained, never a button that fails on press.*

**And whether the mode allows it.** It does, in all three, and that is a
decision rather than an oversight — `crate::shell::commands::catalog::file`
carries the argument at the registrations. Protecting a drawing before
sending it out changes nothing on any page, so it is not authoring; an
operator reading a document in Read mode is exactly the operator about to
email it to somebody. `file.export_dxf` is gated the same way for the same
reason.

## Item notes

### `fn the_guard_and_the_dispatcher_claim_the_same_ids`

[`super::panels`]' test by the same name and for its reason: the two
lists are written separately and a command added to one and not the
other becomes a registered control that does nothing — indistinguishable
from the outside from one that was never wired.

Asserted against the **registry** rather than against another hard-coded
list, so a Security command registered tomorrow fails here rather than
passing a test that lists the same ids again.

### `fn each_command_reaches_its_own_task`

The one decision this module makes, asserted as a pure mapping. A build
that sent both commands to `Task::Password` would open a window that
works — and `Permissions…` would offer to set a password on a document
the operator wanted to re-permission, which is a wrong window rather
than a broken one and therefore the kind that ships.

### `fn task_of`

A second spelling of the mapping, and it is the honest cost of
asserting a decision that is otherwise only reachable through a
`&mut PdfcerApp`. It is pinned to the real one by
[`each_command_reaches_its_own_task`] reading the same ids the registry
test above proves are registered — so the two cannot drift about *which*
commands exist, only about what they map to, and that mapping is short
enough to hold both spellings in view.
