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
