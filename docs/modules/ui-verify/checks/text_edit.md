# `ui-verify/checks/text_edit`

`text_edit_pins_an_aligned_tail` — **the edit reached the bytes, and the text
the operator did not touch did not move**, proved by re-opening the written
file in a second process.

# What this is for

`DEFECTS.md` **D4** is the defect that began this project. Its D4b half names
two cases that are not merely unhelpful but **wrong on commit**, and both are
about text the operator never touched: a right-aligned tail pushed off the
edge it is flush against, and a rotated line's tail slid along user-space x
when its baseline runs up the page.

`canvas::textedit::proof` already asserts both against the written bytes, in
process, with the old shell's own `EditOptions::default()` executed beside
each as the falsifier. **This check asks a different question**: does the
*operator's route* reach that arithmetic at all — a mode, a ribbon control, a
click on the page, a commit, a save, and a file somebody else can open.

`save_copy`'s header lists the five links a unit test cannot see, and this
adds two more that are specific to typing:

| # | Link | Its own test |
|---|---|---|
| 1 | the ribbon control arms the caret tool | `shell::commands` — the registration, not the arm |
| 2 | a click on the page resolves a **run** | nothing: it needs a rendered page and a real hit test |
| 3 | a draft reaches the commit | nothing: the typing loop is `egui::Event` handling |
| 4 | the commit plans the **right disposition** | `textedit::proof` — the arithmetic, not the route |
| 5 | the engine writes it | `pdfcer-core` |
| 6 | it is in the saved file | `save_copy` — for an annotation, not for a text edit |
| 7 | a **second process** reads the new text back | nothing |

# NOTHING IS TYPED AND NO KEY IS PRESSED, and both facts are findings

**Text cannot be injected on this machine.** `crate::sys::vk` is a closed
list of eight non-character keys whose own comment refuses to grow into
`pub const A..Z`.

**Synthetic keyboard input DOES reach the target window.**
`crate::checks::add_text` types real characters and passes. A chord that
produces no trace line is evidence about *that chord's dispatch*, never
about the machine's ability to type — drawing the conclusion one layer too
low is how a dead chord gets recorded as an environment fact, and a
misdiagnosis recorded as fact protects the defect that produced it.

The seam below is a convenience, not a workaround: it supplies a *known*
string, which is worth having.

Typing is this feature's entire input, so a check that could supply no text
would be reduced to asserting *"the tool armed"* — **an assertion in the
right direction that measures the wrong thing**, which is satisfied by any
absurdity pointing the same way. So the draft's characters arrive through
`PDFCER_DIAG_TYPE`, a seam in
the shape of `PDFCER_DIAG_OPEN_PATH` and `PDFCER_DIAG_SAVE_PATH` — both of
which exist because a native modal cannot be driven from here. What the seam
**does not** replace is any other link: the mode still has to change, the
tool still has to arm from a real ribbon click, the click still has to
resolve a run through the real hit test, the commit still has to plan the
disposition, the engine still has to write, and the file still has to be
readable by a second process. It supplies characters and nothing else, and
pushes them through the same `insert` a keystroke would.

**And no key is pressed at all — not even Enter.** The commit is reached by
*clicking somewhere else*, which is the shell's own rule: `textedit::click`
commits an existing draft before starting a new one, because clicking away is
every editor's "that word is finished". The pointer-only path an un-typeable
machine forces on this check is also the path a mouse-driven operator takes.

# The phases

| Phase | Does | Expected |
|---|---|---|
| A | launch on `fixtures/tail-alignment.pdf`, click **Edit** mode | `ribbon-mode-selected mode=edit` |
| B | click Edit ▸ **Edit text** | `text-edit-tool tool=TextEdit(Edit)` |
| C | click the right-aligned line's first word | `text-edit-caret … run=N` |
| D | click blank paper | `text-edit-plan … disposition=Pin reason=Flush(Right)`, `edit-text` |
| E | File ▸ **Save a copy** | `save-copy …`, a file at the named path |
| F | read the copy | the source's bytes are its prefix, verbatim (§7.5.6) |
| G | scan the **appended** revision | the untouched line's `Tm` is there **verbatim** |
| H | launch a **second process** on the copy | it opens and draws the edited page |

# Why phase G is the verdict and phase H cannot replace it

Phase H proves the copy opens. It cannot see the defect at all: a
build that pushed the two untouched lines sideways would still have written
the new word, and the second process would still read it back. Only G looks
at the operator that was supposed to be left alone.

And G has to scan the **appended** bytes rather than the file, which is the
subtlety that would otherwise make it a false pass. §7.5.6 forbids an
incremental update from rewriting the base revision, so the original `Tm` is
still in the first `source.len()` bytes of *every* build's output. A scan
over the whole file answers "unmoved" for a correct build and a broken one
alike.

## Item notes

### `const MODE`

`edit.text` is gated on `Capabilities::edit_content`, which the shipped
manifest gives to Edit alone — and the application opens in **Read** (its
remembered default), so a check that did not switch would be measuring the
mode gate rather than the tool. That is exactly what this check's first run
did: `command-declined id=edit.text reason=mode-cannot-edit-content`.

### `const AIM`

The middle of `REVISION B`, which the generator reports as spanning
x = 431.31…500.00 at y = 700 on a 612 x 792 page. Written as fractions
because `crate::checks`' rules allow a check only `DocPoint` and `FracRect`
literals — a screen coordinate would be a number about this machine.
