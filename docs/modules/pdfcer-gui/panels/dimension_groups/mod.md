# `panels::dimension_groups` — where dimension groups are made, chosen and
configured

## The gap this closes

`measure.manage_groups` is a registered command drawn on Measure ▸ Scale,
and a command that opens nothing is a control an operator presses to no
effect. This panel is what it opens.

**Every verb a group manager needs is a shipped engine verb**, so nothing
here waits on the engine:

| | verb, in `pdfcer_core::edit` |
|---|---|
| create | `add_dimension_group` |
| calibrate | `set_group_scale` |
| drafting standard | `set_group_standard` |
| appearance defaults | `set_group_style` |
| show / hide the layer | `toggle_dimension_layer` |
| rename | `rename_dimension_group` |
| delete | `delete_dimension_group_with` |

## ★ The control that was missing from the whole feature, not just from this
surface

`MeasureState::group` is the active authoring group the next dimension
joins. It is seeded to `DEFAULT_GROUP_ID`, and **this panel is the only
thing that writes to it**. Without that write a second group can be created
from the CLI, carry its own scale, and be joinable by nothing: every
dimension the shell authors goes into the default group, forever.

The *Draw into* column is that picker. It is the first control here not
because it is the most elaborate but because without it every other control
governs a group nothing can reach.

## ★★ Why this is a PANEL and not an [`egui::Window`]

[`crate::dialogs`]' own test reads *a dialog is one transaction with a start
and an end; a panel is somewhere an operator dips in and out of while
working*, and group setup can be argued either way: it largely happens once
at the start of a sheet. Three findings outrank that taxonomy argument, and
all three are about what a tall free-floating window does on a laptop:

1. **A window taller than the screen cannot be closed.** Its title bar can
   leave the desktop, and with it the only ✕. That is not a layout
   complaint, it is a **trap** — the surface captures the operator and the
   application offers no way back. A dock panel cannot do it: the dock
   bounds its own body and the tab strip carrying the close control is
   pinned to the top of that body, on screen, always.
2. **The content is long and stays long.** Six subject blocks for the
   selected group, plus the list, plus the new-group controls. A surface
   whose natural height exceeds a laptop screen wants a **scroll region with
   a bounded parent**, and a dock column is exactly that. A free-floating
   window sized to its content is not — see the growth loop recorded below.
3. **The sections must fold**, on the model of
   [`crate::dialogs::settings`]. That is the same reasoning that module's
   header gives for its own seven groups: an operator arrives with a
   *symptom* — "this group's arrowheads are wrong", "this one is measuring
   in inches" — and headings are how a symptom finds its control. Folding
   turns a 700 pt column into a six-line table of contents.

The taxonomy is therefore **amended openly** rather than quietly bent. The
dock's own test in `crate::app::modes` is *"selection state is watched,
workflows are entered"*, and group setup is watched: which group the next
ce dimension joins is a fact an operator consults while drawing, in the same
breath as which layer is visible. That is the Layers panel's question with a
different noun, and Layers has never been a window.

## ★ The growth loop a window would have here, and why the dock removes it

Put this body — a vertical `ScrollArea` — in a window with a
`default_width` and no height and the window sizes itself to its content:
the scroll area asks for the height of everything inside it, the window
grows to fit, the scroll area gets more room and asks for more. A feedback
loop between a measured size and the thing being measured —
`D:/dev/rag/egui/`'s R128. A driven run of that arrangement laid
`dimension-groups.new_name` out at y=958 inside a body that ended at y=793:
**the Add button rendered below the bottom of the window, unclickable.**

A `default_size` carrying a height suppresses it. The dock removes the
condition instead: **a dock panel's height is the dock's, decided before
the body draws**, so no content this module lays out can influence it. The
class of defect is gone rather than tuned, and *reserve-and-hope is the same
defect with a tuning parameter*.

## ★ Which folds start open — one of six, and the rule behind it

**Scale and unit** alone. Everything else — Add a group, Rename or remove,
Drafting standard, Layer, Appearance defaults — starts shut.

The rule is *what does an operator need to READ without asking*, not *what
do they most often change*. Those are different questions and the second one
is the trap: Appearance is the most-changed section and the longest, and
opening it by default is precisely what made this surface taller than the
screen. A fold that is open because the content behind it is popular is a
fold that is never closed.

What has to be readable at a glance is **which group is which**, and the
list answers that — with the scale phrase already on every row. The Scale
and unit fold stays open because it is the one section that is *about the
selected row's identity* rather than about editing it: an operator who has
just clicked a row is asking "what is this group calibrated to", and making
them click again to find out would be the panel answering a question with a
question.

Folds persist in `egui::Memory` for the session, keyed by the section's
stable key, so this is a *starting* arrangement and not a policy. An
operator who works in Appearance for an hour opens it once.

## Everything is inside the one `ScrollArea`, and that is deliberate

**A control that must be reachable cannot be placed after an unbounded
`ScrollArea`** — an application-side pattern this shell has met on four
separate surfaces. One answer is to hoist the reachable controls out of the
scroll area and reserve a footer for them. This panel has **no footer**:
there is no Close button, because the dock tab carries one, and
Add sits at the bottom of the scroll region inside a fold of its own. With
nothing after the scroll area there is nothing to be pushed off the end of
it, and no `FOOTER_RESERVE` constant to be tuned wrong.

## What it does NOT do, deliberately

- **It does not pick which group a placed ce dimension belongs to.** There
  is no engine verb for that
  (`request_a_placed_ce_dimension_cannot_be_moved_to_another_group.md`), and
  more importantly it is a *per-ce-dimension* question — it belongs on the
  selection surface, beside the other per-ce-dimension overrides.
- **It does not set a per-ce-dimension anything.** Every control here is
  group-scoped, which is what makes the reach-backwards disclosure on each
  of them true and uniform.
- **It does not offer a scale field.** The Set-scale window already exists,
  already owns the two entry paths and the calibration gesture, and already
  raises the one action. A second scale entry here would be a second
  implementation of the hardest arithmetic in the feature — see
  [`DimensionGroupsUi::take_scale_request`] for how the button hands over.
