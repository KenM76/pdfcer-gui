# `ui-verify/checks/escape_commits_text`

`escape_commits_text` — **Escape on a text draft WRITES it, and `Ctrl+Z`
takes it back.**

# The request

> *"I think for adding and editing text when using any tool that has text
> escape should also save changes to the text. The user can always undo if
> they want, but it is easy to accidentally press escape and lose a lot of
> text that has been entered."*

# Why this is a correctness rule and not a preference

The operator's sentence carries its own argument, and it is about
**asymmetric cost**. A draft written by mistake is one `Ctrl+Z`. A draft
discarded by mistake is minutes of typing with **no recovery anywhere**,
because a draft lives in `egui::Memory` and never reaches the undo stack —
there is nothing for undo to restore. The two mistakes are not comparable,
so the exit that happens by accident must be the recoverable one.

That is why phase E is part of this check rather than a separate one. The
ruling is not *"Escape commits"*; it is *"Escape commits **and** the commit
is recoverable"*, and a build that did the first without the second would
have replaced an unrecoverable loss with an unrecoverable gain.

# The oracle the wrong build cannot produce

`canvas-escape outcome=SettledTextDraft` says the Escape ladder reached the
text-draft rung. It does **not** say the text landed: the rung could fire
and the commit still fail. `add-text` is the line the engine writes when
the characters actually reach the document, and it is the assertion this
check turns on, because the behaviour being replaced — Escape discarding —
produces the rung's teardown and **no `add-text` at all**.

⚠ `text-edit-abandon` is not usable as an oracle in either direction. Its
own contract says it is not evidence that an edit was lost: it is emitted
by `settle`'s teardown half whatever brought the draft to an end, so it
appears on the correct build and on the wrong one alike.

# The phases

| Phase | Does | Expected |
|---|---|---|
| A | Edit mode, Edit tab, click **Add text** | `text-edit-tool tool=TextEdit(Add)` |
| B | click blank paper | `text-edit-caret kind=Add` |
| C | type two real characters | `text-edit-typing … len>0` |
| D | **press Escape** | `canvas-escape outcome=SettledTextDraft` *and* `add-text` |
| E | `Ctrl+Z` | `undo-applied` — the recovery the ruling rests on |

# What it cannot see

* **The other surface.** `checks::form_field` drives a form-field editor,
  which carries text too and settles the same way on Escape. This check
  drives the canvas draft only.
* **Whether the committed text is what was typed.** Phase D asserts the
  commit reached the engine, not its content — the same scope as the
  clicking-away commit in [`crate::checks::add_text`].
* **A draft orphaned by a document-tab switch.** A canvas draft is context
  global, so switching documents mid-draft is a separate and unaddressed
  question; nothing here exercises it.
