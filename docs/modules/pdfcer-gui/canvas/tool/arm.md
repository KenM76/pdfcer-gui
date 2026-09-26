# `canvas::tool::arm` — how a tool is CHOSEN

## The seam against `super`

`super` answers *"what IS a tool?"* — the enum, and the predicates that are
properties of a variant: which cursor it wants, whether it pans, which kind
it carries, which capability it needs. Every one of those is a pure
function of the value and none of them touches the world.

This file answers *"which tool is chosen, and how does that change?"*. Every
function here reads or writes `egui::Memory`, and the interesting content is
the **transition rules**: pressing an armed button retires it, pressing Hand
from anything takes Hand rather than toggling through Select, a mode change
retires a tool the mode may not use.

Those two subjects change for different reasons. A new variant is a `super`
change; a new rule about what pressing something does is a change here, so
every "why does pressing this twice do that?" argument is in one file.

## The lines this module writes

An armed canvas and an un-armed one are the same screenshot — the same
screenshot even with the pointer in it, because a captured window does not
carry the cursor. These are the whole of what a driven check can ask.

canvas-tool armed=Hand from=TextEdit(Add)

[`select`] writes it on every change, whatever armed it. Ask for this one
when the subject is *which tool is armed*.

text-tool tool=Text
text-edit-tool tool=TextEdit(Add)
markup-tool tool=Markup(Cloud)
measure-tool tool=Measure(Linear)
form-tool-armed kind=Text now=Form(Text)

One per arming gesture, naming the button pressed and whether the press
armed or retired it. Ask for one of these when the subject is *that control
works*.
