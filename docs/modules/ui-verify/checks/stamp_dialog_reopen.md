# `ui-verify/checks/stamp_dialog_reopen`

`stamp_dialog_reopen` — **the second stamp's dialog still has its Add and
Cancel buttons on the screen.**

# The report this exists for, verbatim


> *"After I place the first stamp and go to make a second one the window for
> the options pops up but is undersized so I can't see the add or cancel
> button. Those buttons should always be available, and if there isn't size
> for all the features they get scrolled in their own space."*

Two facts in one sentence, and the second is the more important:

1. **The defect is on the SECOND open, not the first.** The window carries
   fitting state between openings; the first opening sized itself correctly
   and the second inherited a size that no longer fitted the body. A check
   that opened the dialog once — and every existing stamp check opens it
   once — is structurally incapable of seeing this. That is why this check
   exists alongside `stamp_size` rather than inside it.
2. **A dialog whose Accept is off-screen is a transaction that cannot be
   finished, only abandoned.** It is not a cosmetic complaint. There is no
   keyboard route to Add, so the operator's only exit was the title bar's X.

# Why this can assert with no size arithmetic anywhere in the file

Both buttons are published through `diag::ui_rect_visible`, which emits
**nothing** when the rect falls outside its own clip rect. So:

> **"was the region declared?" and "was the control on the screen?" are the
> same question**, and this check never computes, compares or guesses a
> single number.


# Phases

| Phase | Does | Expected |
|---|---|---|
| A | Review mode, Markup tab, arm **Stamp** | `markup-tool tool=TextAnnot(..)` |
| B | drag a box | `text-annot-open`, and `dialog:text-annot` declared |
| C | read the answer row | `text-annot.accept` **and** `text-annot.cancel` declared |
| D | press Add | the dialog closes |
| E | arm Stamp again, drag a **second** box elsewhere on the page | the dialog opens a second time |
| F | read the answer row again | both regions declared — **this is O171** |
| G | press Cancel | the check leaves the document as it found it, bar one stamp |

Phase C is not redundant with phase F. If the row is missing on the *first*
open too then the defect is not the one the operator reported and the fix
that was made would be the wrong fix — the message says so, rather than
letting the same failure text stand for two different faults.

# Rule 15

## Item notes

### `const SECOND_OFFSET_PT`

It has to miss the first stamp's rectangle. A second drag that starts
**inside** an annotation that already exists is a different gesture — the
canvas reads it as grabbing that object — and the dialog would then never
open, which this check would report as O171 recurring. A harness with a bad
input produces defects that do not exist.

### `fn second_point`

# Why this is a function and not `target + SECOND_OFFSET_PT`


> *document point (2520, 840) is outside the page's crop box (0, 0) —
> (2383.937, 1683.78)*

The sweep's shared aim is `0,2000,320` on a 2384 x 1684 sheet, so adding
300 pt of offset and 220 pt of box ran 356 pt off the right-hand edge. The
coordinate was never wrong in any absolute sense — it was wrong for the
document the sweep happened to hand this check, and a check that works only
for the aim points it was written beside is a check that stops running the
day somebody re-aims the chunk it lives in.

⚠ **A SKIP is not red.** This one sat in the sweep output reading like a
deliberate exclusion, next to a `detects:` line describing a real O171
regression nobody was watching for any more.

# What it does

Offsets **away from the nearer edge on each axis independently**, so the
second box lands on paper wherever on the sheet the first one was:

  * the offset is positive when `target + SECOND_OFFSET_PT + BOX_PT` still
    fits inside the page, and negative otherwise;
  * the result is clamped to `[0, page - BOX_PT]` on both axes, because a
    page smaller than `2 x (SECOND_OFFSET_PT + BOX_PT)` has nowhere that
    satisfies both directions and a clamp is a better answer than an
    off-page drag.

The 300 pt separation is what stops the second drag from starting inside
the first stamp, and the sign does not affect it — 300 pt left of the first
box clears it exactly as well as 300 pt right of it, because the box is
220 pt wide. That is the property [`SECOND_OFFSET_PT`]'s own doc is about,
and it is preserved by construction rather than by the clamp.

### `const TOOL_TEXT_ANNOT`

Matched on the prefix rather than the whole string deliberately: this check
is about the *dialog*, and it must not fail because the shell started
spelling the stamp variant differently.

### `fn armed_tool`

`markup-tool` is emitted on CHANGE, so the last one in the trace is the
current state and an absent line means *the tool has not moved since
launch*. Both readings matter to [`arm_stamp`], and conflating them is what
the first version of this check did.

### `fn place`

It does not assert that the dialog opened — each caller says that in its own
words, because *"the FIRST drag opened nothing"* and *"the SECOND drag
opened nothing"* are different defects and one shared sentence standing for
both is how a fix gets aimed at the wrong one.

# Errors

If the canvas mapping cannot be read, or the pointer cannot be driven.
