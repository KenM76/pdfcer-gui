# `enter_newline` — Enter makes a second line, and Ctrl+Enter finishes it

`OPERATOR_REQUESTS.md` **O127**, defect 2:

> *"also can the enter key create new lines when we are editing or creating
> text?"*


The session that wrote it was instructed not to launch the GUI — the
operator was at his keyboard and a second run would have fought his pointer.
It is registered anyway, deliberately and on the precedent this file's
neighbours set: `left_rail`, `properties_tool` and `protect` all carry the
same note. **A check that is not in the list is a check nobody will ever
run**, and an unregistered file is a promise rather than an instrument.

⇒ Whoever runs the suite next is the first thing that executes this. If it
is red on its first run, read §"what each failure means" before assuming the
shell is broken: an unrun check's first failure is as likely to be in the
check as in the subject, and that is not a reason to weaken it.

## What this check is for, and why the unit tests are not enough

`canvas::textedit::keys::enter_means` is a pure function with four unit
tests, and they prove **the rule**. They cannot prove any of these:

| link | provable without driving? |
|---|---|
| the Enter keystroke **reaches** the draft at all | **no** |
| it is not eaten by a guard, the ribbon keymap or a focused widget | **no** |
| the caret ends up on the second line rather than at the end of the first | **no** |
| the newline **survives the commit** into `add_text` | **no** |
| `Ctrl+Enter` is not intercepted before the draft sees it | **no** |

Every one of those has failed in this project at least once. `caret::newline`
exists **because** `insert` silently ate the exact keystroke this check
presses, for a whole driven run, while every unit test stayed green — the
trace said the key arrived and the length did not move. That is the shape
this file is against.

## The oracle, and why it is a COUNT

`add-text page=… n=…` — the funnel's operand count, which
`app::actions::addtext` sets to the number of **hard newlines** in what was
committed. So:

* a build where Enter did nothing commits `n=1`;
* a build where Enter committed instead of inserting commits `n=1` **and
  commits early**, so the second half of the typing never arrives;
* a build where the newline was dropped between the draft and the engine
  commits `n=1`;
* only a build where the whole chain works commits `n=2`.

A count rather than a screenshot, for the reason `text_edit`'s check gives
about the same choice: two lines of 11 pt text on an A1 sheet are a few
pixels, and an oracle that cannot tell one line from two is not an oracle.

## Why the text is SEEDED and the Enter is REAL

`PDFCER_DIAG_TYPE` puts characters in the draft, because this machine's
harness cannot inject arbitrary characters — `sys::vk` is a deliberately
closed list of non-character virtual keys and its own comment refuses to
grow into `pub const A..Z`.

**Enter is not seeded.** It is `sys::vk::ENTER`, pressed for real, because
the keystroke is the entire subject: a seam that inserted the newline would
be verifying that this check can write a `\n` into a `String`.

⇒ So the shape is: seed a word, press Enter, seed nothing more, press
Ctrl+Enter, and read the count. The seed runs once per draft (`Draft::seeded`
is consumed on the first frame), so the second line is empty — which is
fine and is deliberate: `n` counts hard newlines, and one Enter is one
newline whatever is on either side of it.
