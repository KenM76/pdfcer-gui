# `panels::properties::workaround` — making a refused text edit another way

When the engine's exact surgery refuses a committed text edit, it may name a
`Workaround` (`EditError::workaround`): join the text objects a line is split
across, rewrite a `'`/`"` operator as the `T*` and `Tj` it stands for, or
retype the run. The engine applies one only when the caller's `EditOptions`
carry `WorkaroundPolicy::Apply`, and this shell never sets that by default.

## The route

| step | where |
|---|---|
| the canvas editor commits; the engine refuses | `app::actions::textcommit::commit_text_edit` |
| the refusal names a workaround; the typed edit is kept | `app::status::decline::textedit`, reading `canvas::textedit::last_commit` |
| the offer is drawn under the refusal | this module's `section`, below `refusedchar` |
| the press re-raises the same edit with `workarounds: true` | `Action::CommitTextEdit` |
| the engine's report names the workaround it used | its disclosure, shown on the status line; `edit-text-workaround` on the trace |

## Why an offer, and why in Properties

The workarounds change the content stream's structure, and `Retype` is not
exact: the letter spacing may differ. Applying one silently would be the
sneaky half of "fuzzy, never sneaky"; asking every time with a dialog would
interrupt an edit the operator can simply abandon. An offer in the panel that
already carries the refusal costs nothing when ignored, and the press is the
consent. Nothing is drawn on the page either way (R8b): the result renders as
it will save, and the engine's disclosure goes to the status line.

## Lifetime

The offer is retired by any change to the document (`OpenDoc::edit_epoch`),
by switching documents (`forget_document`), and by the press itself. If the
workaround is refused too (`EditError::WorkaroundRefused`), the section shows
the engine's reason instead of a button.

Driven by `a_refused_edit_offers_its_workaround`.
