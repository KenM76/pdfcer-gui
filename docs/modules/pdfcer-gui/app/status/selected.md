# `status::selected` — what is selected, said in words

One line at the left of the status bar, naming the thing the operator has
selected and — when it matters — how many other things were under the same
click.

## The state this exists to make legible

The operator:

> *"when I click on one of the objects all I get is the page selected."*

That report is precise. A file that wraps the whole visible body of the
sheet in a page-sized form XObject gives that form a bounding box which
wins every click at every point, and the engine does not enter one — so
what is selected really is a page-sized object.

Without this line **nothing on screen says so**. The selection outline is
drawn round the page edge, which looks exactly like *"the page is
selected"* — a state this program does not have. No other surface says
*"you have selected a Form containing 214 objects"*, which is a diagnosis,
and from a diagnosis the next question follows on its own.

This line does not fix the selecting. It makes the selecting **legible**,
which is what turns an unexplainable interface into a solvable one — and it
is the surface every refusal sentence is printed on.

## Why the left, and why it is the thing that yields

The right-hand cluster is fixed controls the operator reaches for — page,
zoom, fit, Find, the pick filter — and `status::fitting` may shed only two
of those, because the rest have no other home. This is a **readout**: it
costs nothing to lose, because everything it says is also visible in the
Objects panel and in the selection outline.

So it goes on the left with the other narration, where `egui`'s left-to-right
run gives up its space first, and it elides rather than pushing. That is
`status`'s own rule about what yields, applied to the newest thing on the
bar rather than exempting it.

## What it says, and what it refuses to say

| state | line |
|---|---|
| nothing selected | nothing at all |
| one object | its kind, and its size in points |
| one object, more underneath | `… · 1 of 5 here` |
| several objects | `3 objects selected` |

**Nothing when nothing is selected**, rather than *"Nothing selected"*.
A status bar that narrates the absence of a thing spends a permanent line on
the most common state in the program. A tutorial string there may well be
worth having, but that would be a decision about **teaching** and this line
is a decision about **reporting**. They should not be made at once and they
should not be made by the same code.
