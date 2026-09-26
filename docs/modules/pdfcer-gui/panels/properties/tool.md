# `panels::properties::tool` — the armed tool's own settings, where a
property belongs

`OPERATOR_REQUESTS.md` **O123**, and this module is the whole of his
argument made real:

> *"I never understood why there is a tool dock when everything can be in
> object and properties."*

## Why the armed tool's options live in Properties

A one-line tool strip cannot hold them — a 28 pt row has no room for a font
picker, a size, a swatch and a disclosure note. But that is an argument
about a strip, not about where these controls belong. They are not the
tool's; they are **properties of what is about to be drawn**, which is the
same category of thing as the properties of what is already drawn, and this
panel is the surface that owns that category.

| control | block |
|---|---|
| text pen **font** picker | [`text_pen`] |
| text pen **size** | [`text_pen`] |
| text pen **colour** swatch | [`text_pen`] |
| the pen's disclosure note | [`text_pen`] |
| the circular measure's **pick list**, one removable row per point | [`measure_points`] |
| *Scale line weight* | [`scale_switches`] |
| *Keep the inner margins* | [`scale_switches`] |
| *Allow the artwork to distort* | [`scale_switches`] |
| the switches' note | [`scale_switches`] |

Every string is [`crate::text::tool`]'s; every store is the canvas's
(`canvas::textedit::pen`, `canvas::measure`, `canvas::scaling`). This module
draws them and owns none of them.

## Where it sits in the panel, and why it is TWO places

[`Slot`] decides, per block, and the two answers have different reasons.

**[`Slot::AboveTheSelection`]** — the text pen and the circular measure's
pick list — for the two reasons this section has always given:

1. **It is where the operator's eye already goes** — the top-right corner
   of the window, which is where the armed tool's settings have always been
   found.
2. **An armed tool is the more immediate subject.** When somebody has armed
   the text pen, the question they are about to ask is *what size?*, not
   *what is that path's line width?*

**[`Slot::BelowTheSelection`]** — the three resize switches — because
reason 2 is false for them. They *are* a statement about the next gesture,
but Select is armed nearly always, so placing them above the description of
the CURRENT selection puts a hypothetical ahead of the thing on screen.
`Slot`'s own doc carries the measurement. `OPERATOR_REQUESTS.md` O198.

## It is deliberately NOT part of `something_drew`

`OPERATOR_REQUESTS.md` **O75** collapses the *This document* section
whenever a selection-scoped section has spoken. This section is **not**
selection-scoped: [`scale_switches`] draws whenever the Select tool is armed,
which is most of the time and has nothing to do with what is selected.
Folding it into that predicate would collapse the document section for ever
and suppress *"nothing is selected"* for ever, which is O75 answered
backwards.

## The reachability rule this module inherits, and why it is written twice

`panels::tool`'s own header recorded it: the three scale switches were first
written into a branch `CanvasTool::Select` **cannot reach**, and *"an option
row added there is dead code that compiles, reads correctly, and draws
nothing … Every unit test in the chain passed. Nothing tested that the
control is on screen."* The check that caught it drove the real binary.

⇒ So every region here publishes through [`crate::diag::ui_rect_visible`]
rather than `ui_rect`, and one per **switch** rather than one per block. A
rect proves layout; only a rect measured against the clip in force proves
the operator could reach it — and this panel is a `ScrollArea`, so a control
scrolled past the fold has a perfectly healthy rectangle.

## Item notes

### `fn text_pen`

Moved from `panels::tool::armed::options`, unchanged. The two notes that
travelled with it, because both are the kind of thing a move loses:

* The faces come from `pen::FACES`, **not** a hand-written list. Its own
  test asserts all fourteen are offered exactly once — a list that quietly
  held thirteen would be a face an operator could never reach, with no error
  anywhere.
* The size's range is the **store's** bounds rather than two local literals,
  for `dialogs::settings`' reason: a control narrower than what the value
  accepts silently rewrites a setting the operator never touched.

### `fn measure_points`

> *"we should be able to unselect points/clicked locations, and it should
> have a box in the side panel showing what is part of our selection …
> clicking on a point or location listed should allow us to remove it."*

The canvas markers say *where* the points are. They cannot say **how many**,
and on a dense CAD sheet a marker sitting on a junction is not
distinguishable from the junction. The list is the only surface that answers
*"what is actually in this fit?"*

**The removal is applied AFTER the loop, never inside it.** `st` is a
copy read out of `egui::Memory` and `points` borrows it; a removal that
mutated mid-iteration would shift every row below the one pressed while the
loop was still drawing them — the classic one-frame mis-aim, where the
operator presses row 3, row 4 slides up under the pointer, and the next
frame's press lands on something they did not choose.

### `fn scale_switches`

The order is by how often it is wanted: stroke width first (the one he asked
for by name), insets second, and the distortion escape last, because it is
the one that makes the result imperfect and a control that degrades the
output belongs after the two that do not.

**Always drawn, never greyed** while Select is armed — live with nothing
selected and with a form field selected. An operator sets a modifier
*before* the gesture it modifies; greying them until an annotation happens
to be selected would hide the control exactly when somebody is deciding how
to resize.

**One published rect per switch**, not one for the block. A driven check
aiming at "the options row" and then guessing which line is the second
checkbox would be encoding a layout, and it goes wrong silently — by ticking
the wrong switch — the day a label wraps to two lines at a narrower dock.

### `fn each_moved_control_has_a_tool_that_reaches_it`

Asserted against [`block_for`] — **the function [`section_in`] dispatches
on**, not a copy of its `match`. That distinction is the test: a mirror
in this module would go on passing while the shipped decision drifted,
which is the shape of the defect that let three scale switches compile,
read correctly and draw nothing.

This is the unit half of the reachability claim. The other half is the
driven check, because a branch that runs is still not a control on
screen.

### `fn all_three_blocks_are_reachable`

The failure this catches is subtle and has happened here before: an arm
written above another that would have matched, leaving the second
unreachable with nothing to show for it. Sweeping every tool the
application can arm and collecting the blocks that come back is the only
way to see it.

### `fn the_resting_tools_block_is_the_only_one_below_the_selection`

This is the unit half of the placement decision, and it is worth a test
because the rule is easy to misread: it is not
*"tool settings go at the top"*, it is *"a block that draws with no
reference to the selection must not sit above the sections that describe
it"*, and the two read identically until you notice that `Select` — the
RESTING state — is a tool. `Block::ScaleSwitches` is therefore on screen
whenever an operator is doing the ordinary thing of clicking at objects,
which is exactly when the sections below it matter most.

Asserted through [`slot_of`], the function [`section_in`] dispatches
on, rather than against a copy of its `match` — `block_for`'s own test
gives the reason at length and it is the same reason.

### `fn one_block_and_only_one_draws_at_the_foot_of_the_panel`

A second one would stack two unrelated standing preferences under
whatever the panel had just said about the selection, and — because
`section_in` draws at most one block per call — the second would simply
never appear. That is the `scale_switches` defect again in the other
slot: a control that compiles, reads correctly and draws nothing.

### `const REGION_MEASURE_POINT_PREFIX`

Per ROW rather than one rect for the list, because the whole capability
`OPERATOR_REQUESTS.md` O107 asks for is *removing a particular point*, and a
check that could only find "the list" could not press one.

### `enum Block`

# A real function, not a `match` buried in a draw call

The mapping *tool → controls* is the whole of what this module decides, and
it is the thing that broke last time: the three scale switches were written
into a branch `CanvasTool::Select` **cannot reach**, and *"an option row
added there is dead code that compiles, reads correctly, and draws
nothing."* Every unit test in that chain passed, because none of them could
ask the question — the decision lived inside a function that needed a `Ui`.

It does not any more. [`section`] dispatches on this and nothing else, so
`each_moved_control_has_a_tool_that_reaches_it` is asserting the shipped
decision rather than a copy of it.

### `enum Slot`

# Why placement is a per-block decision and not one rule

*"An armed tool is the more immediate subject"* is true of the text pen and
the circular measure and **false of the resize switches**, because
[`block_for`] hands those back for `CanvasTool::Select` — the RESTING
state — so they are on screen whenever an operator is doing the ordinary
thing of clicking at objects.

The cost of getting that wrong is measured. Put every block at the top and,
in an 1100 x 800 window with one text object clicked, the panel's whole
visible height is *When you resize something*, its three switches and its
five-line note; the first two rows of the text editor are half clipped
(`properties.text.bold … shown=0.46 floor=0.60`) and the Colour swatch sits
at y 783-807 in a viewport ending at 766 — `shown=0.00`, off the bottom,
reachable only by scrolling past a preference the operator did not ask
about. `ui-verify clicking_text_offers_its_colour` is the check that holds
this.

The rule it enforces is `OPERATOR_REQUESTS.md` O75's, applied one block
down: **a section that draws with no reference to the selection must not sit
above the sections that describe it.** The operator's O198 sentence —
*"the properties area is uneditable"* — is what the violation looks like
from outside, when the editable part is below the fold.

### `fn slot_of`

A function rather than a `match` inside the draw call, for [`block_for`]'s
reason stated again: the placement is a DECISION, and a decision that needs
a `Ui` to observe is a decision no unit test can put a question to.

### `fn armed_section`

Returns `false` when the armed tool has no settings, or has some that belong
at the foot of the panel — which is the honest shape rather than a heading
with nothing under it (R9). See [`Slot`] for the split and why it exists.

### `fn preferences_section`

Called AFTER `object_section`, which is the only section that can say
*"nothing is selected"*. That ordering is deliberate and is the one thing
about this call that is easy to get backwards: these switches are not a
description of a selection, so they must not be able to push one off the
screen, and they must not read as though they were describing whatever the
panel just said. Last is the only position that is true in both states.
