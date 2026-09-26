# `text::tool` — the words the tools say, wherever they are said

## This file OUTLIVED the panel it was written for


| what | who says it now |
|---|---|
| the per-tool instructions and live stages | [`crate::app::toolstatus`] — the right dock's permanent one-line strip |
| the second sentence of the stages that had one | the same strip, in its hover |
| the text pen's labels, the measure pick list, the resize switches | [`crate::panels::properties::tool`] |
| the disclosure heading | [`crate::panels::properties::disclose`] |

⚠ **What was DELETED, and it is the only deletion**: the fifteen strings the
panel's tool LIST used — `tools_heading`, `tools_hint`, `row_home` and the
nine `row_*` sentences, plus `pointer_heading`, `armed_heading` and
`no_document`. Every one of them labelled a button that duplicated a ribbon
control, and the operator's instruction was *"its buttons duplicate the
ribbon and go."* They are gone rather than left orphaned, because an unused
catalog entry is a sentence nobody can find and nobody can retire.

Worth naming what that cost: those rows were the answer to a
discoverability defect — *"The feature works. He could not find it."* The
strip that replaced them cannot list what is NOT armed. That is a real
subtraction and it is the operator's own call; it is recorded in
`crate::app::toolstatus`'s header rather than argued here.

## The three rules the whole file follows, unchanged

**1. No label is written here that the command registry already owns.**
The armed tool's name comes from `CommandRegistry` through
`crate::shell::menus::MenuHost::label`, and the chord comes from the
operator's own keymap. A second copy of a label compiles, reads identically
the day it is written, and drifts the first time either is reworded —
invisibly, because nothing renders both at once. `NO_SURFACE.md` §1 records
that exact failure with a colour.

**2. Every sentence states a fact about the program, never a tip.** The
operator's own report about the shell this replaces: *"the nagging and red
flagging in the original GUI made for a lot of extra bugs in the visibility
when editing."* *"Drag marquees objects on this page"* is a statement.
*"Try dragging to select several objects!"* is a tip, and there are none
here.

**3. An instruction says how the gesture ENDS.** Half the gestures in this
application do not end by themselves — a run of clicks does not, a text
caret does not — and *"click each corner"* is not a complete instruction
because nothing in it says when to stop. Every instruction below that
describes an open-ended gesture names its ending.
