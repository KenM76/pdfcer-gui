# `ui-verify/checks/quit_unsaved`

`closing_the_program_asks_before_losing_unsaved_work` — **the prompt that
did not exist.**

# The report


> *"when I close the program it should prompt to save changes if there are
> any, and it should do what other programs do — switch focus to the document
> that is being prompted for, and cycle through each unsaved document while
> it prompts, but also have a save all button that saves all changed
> documents."*

# Why this check is worth more than the feature it guards


And a driven check had been pressing `Alt+F4` all day without noticing.
`checks::page_display_pref` closes the program that way on purpose, and it
passed throughout, because it opens a document and never edits one. **A
driven check only ever sees what it drives**, and the state this defect lived
in — *a dirty document at the moment of close* — was one no check had ever
constructed.

⇒ That is this file's whole reason to exist: it constructs that state.

# What it drives

1. open a document, and **make an edit** — the state nothing else creates;
2. press `Alt+F4`;
3. assert the close was **held** (`quit-held`) and the question **asked**;
4. press **Cancel**, and assert the program is **still running**.

It ends on Cancel deliberately. The alternative endings — Save, or Discard
— either write a file or destroy work, and a check that runs unattended on
the operator's own machine should do neither. Cancel is the answer that
proves the whole chain worked and leaves nothing behind.

# The falsifying half, and why the count is asserted

A build that popped the dialog on **every** close — dirty or not — would
satisfy "a dialog appeared". So phase A drives a close on an **unedited**
document first and asserts the program exits with **no** `quit-held` line at
all. Without that, this check would pass against a build that had simply
learned to nag.

# Every way this reports SKIP

No binary, `--no-input`, no diagnostic channel, no way to make an edit (the
fixture or the tool is missing), or a window that will not close — the last
being a property of the machine on the day.

## Item notes

### `const SAVE_ALL_REGION`

Its absence is an assertion, not an omission: the button is drawn if and
only if more than one document is dirty, so a run with one must show no such
region at all. See `dialogs::unsaved`'s `REGION_SAVE_ALL`.

### `fn the_check_asserts_both_directions`

Phase A: a clean close must NOT be held. Phase B: a dirty close MUST be.
Either alone passes against a wrong build — A alone against one that
never asks, B alone against one that asks always — and the pair is what
pins the question as *conditional*.

### `fn the_ending_is_the_harmless_one`

Pinned as a sentence because it is a policy rather than a mechanism: this
suite runs unattended on the operator's own machine, and a check that
ended on Save would leave a file behind while one that ended on Discard
would throw work away to prove that it could.
