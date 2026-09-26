# `ui-verify/checks/cut_gate`

`cutting_a_redaction_mark_is_refused_before_anything_is_removed` — the
driven proof of `OPERATOR_REQUESTS.md` **O59**'s first item.

# What this is about


> **Do not offer Cut as enabled and let it fail.** A **copy** of something
> pdfcer cannot carry costs nothing — the original stays. A **cut** of the
> same thing is a deletion wearing a clipboard's clothes.

A `/Redact` annotation is a **pending destructive operation**. Pasting one
arms a redaction nobody reviewed, so the engine refuses to carry it — and
therefore refuses to cut it, before anything is removed.

# Why greying the button is not enough, and this check is about the gap

`edit.cut` is greyed on `selection.cut_permitted`, so a **pointer** cannot
reach the verb. **A chord can.** `Ctrl+X` is dispatched through the keymap
and the keymap does not consult command enablement — so the handler runs
whatever the ribbon is showing.

That gap is the whole subject here. A build that greyed the button and left
the chord unguarded would look completely correct in every screenshot, pass
every unit test of the gate, and delete a redaction mark on `Ctrl+X` while
putting nothing on the clipboard.

⇒ So the assertion is not *"the button is grey"*. It is **the keystroke was
refused and the mark is still there.**

# The oracle, and why it is two lines rather than one

| line | question |
|---|---|
| `clipboard-cut-refused reason=would-not-survive subtype=Redact` | did the gate fire, and did it name the right thing? |
| the **absence** of `clipboard-copy` | did it refuse *before* the copy, or after? |

The second is the one that matters and it is easy to leave out. A cut that
refused *after* copying would leave the mark on the page **and a copy of it
on the clipboard** — so the next `Ctrl+V` arms a redaction somewhere else,
which is precisely the outcome the refusal exists to prevent. The order is
asserted, not assumed.

# Why this fixture and not `--pdf`

`demo-marked-output.pdf` from the engine's own corpus is a document with
**redaction marks already on it** — three of them. A check whose subject is
*"what happens when you cut a redaction mark"* cannot take an arbitrary
drawing: on the operator's own CAD sheets there are none, and the honest
answer there is *"there was nothing to try it on"*, which is neither a pass
nor a defect.

Marking a page from the ribbon instead was the alternative and was rejected:
it would make a check about the clipboard fail whenever the redaction panel
moved, and `checks::redaction` already owns that sequence.

## Item notes

### `const ANNOT_SELECT`

`annot-select`, not `canvas-selection`. The general selection line
carries `sel=`, `level=` and `first=` — all about the **content** index
spaces — and an annotation selection is not in either of them. Reaching for
the familiar line would have made the hunt below click twenty-five times and
conclude there were no marks on a document with three.

It also carries `subtype=`, which turns the hunt from *"did something get
selected?"* into *"did a REDACTION MARK get selected?"* — the difference
between a check that proves its own subject and one that proves a click
landed on some annotation and then blames the cut gate.

### `fn engine_fixture`

`None` rather than a panic turns a missing corpus into a SKIP with a reason
instead of a crash mid-suite. `D:\Dev\pdfcer` is READ-ONLY to this project;
this reads from it and writes nowhere near it.
