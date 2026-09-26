# `dialogs::formfield` — the details a placed form control needs

**Operator request, 2026-08-26:** *"when I click one I should be able to
click on the canvas to place the position or drag a box for size then a pop
up lets me set the details for the feature."* This is the pop-up.

It opens on `Action::BeginFormField`, which the canvas raises on the click or
release that finishes placing, and it **authors nothing until Accept**. That
is the whole reason a dialog is in this path rather than a properties pane
after the fact: a form field is invisible on a printed page and swallows
every keystroke aimed near it, so a mis-drag that left one behind would be
both hard to notice and annoying to find.

## ★★★ The tooltip field is not a nicety — it is the feature's blocker

Every one of `pdfcer-core`'s five authoring verbs refuses a spec whose
tooltip is `TooltipChoice::Undecided`, because an interactive control owes a
screen reader a name and the engine will not invent one silently. That
refusal was recorded in this project's backlog as *"core's STRUCTURAL
certification gate"* and parked form authoring for nine days.

There is no gate. The blocker is **this text box**. An empty one becomes
`Declined` — the operator saying *"this control needs no name"*, which is a
decision the engine accepts and is sometimes right — and a filled one becomes
`Text`. What the engine will not accept is nobody having been asked, and now
somebody has.

## ★★ Why one dialog for five kinds, and how it stays legible

[`crate::canvas::formfield::draft::Draft`]'s header argues the model side:
the five engine specs share nine fields and differ in one to five, so five
GUI structs would mean writing the shared half five times. The same argument
holds for the surface, with one addition — **the shared half is the half an
operator adjusts.** Name, tooltip, required, read-only and border are asked
identically for all five, and only the kind-specific rows change.

So the layout is: the common rows, a separator, then [`Self::specific`],
which is the only `match` on kind in the file. A reader looking for "what is
different about a check box" has exactly one place to look.

## What is remembered, and where

Nothing, here. The dialog opens with a draft that
`Remembered::next` already prepared, and `Action::CommitFormField` is what
records the accepted one — **at the point it was accepted**, so a draft the
operator cancelled is not remembered. See `app::actions::apply`'s arm.
