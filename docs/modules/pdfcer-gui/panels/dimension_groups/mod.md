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

## The control that was missing from the whole feature, not just from this
surface

`MeasureState::group` is the active authoring group the next dimension
joins. It is seeded to `DEFAULT_GROUP_ID`, and **this panel is the only
thing that writes to it**. Without that write a second group can be created
from the CLI, carry its own scale, and be joinable by nothing: every
dimension the shell authors goes into the default group, forever.

The *Draw into* column is that picker. It is the first control here not
because it is the most elaborate but because without it every other control
governs a group nothing can reach.

## Why this is a PANEL and not an [`egui::Window`]

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

## The growth loop a window would have here, and why the dock removes it

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

## Which folds start open — one of six, and the rule behind it

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

## Item notes

### `mod identity`

Its header carries the interesting half: deleting a populated group is the
**orphan question**, the engine refuses by default with the member count in
the refusal, and putting that question in front of an operator is a thing
only a surface can do.

### `fn default`

`Unit` implements no `Default`, deliberately — the engine declines to
have an opinion about which unit a drawing is in, which is exactly the
stance `crate::dialogs::settings`' header describes for every ambiguity
the spec leaves open. So the surface must have one, and a surface's
choice is a statement about *this operator's* drawings rather than about
the type.

The rest is `Option::None` and an empty `String`, which a derive would
have given for free. It is worth the eleven lines to keep the one real
decision visible instead of buried in a field initialiser.

### `fn show`

# One `ScrollArea` and nothing after it

The single most important line of layout in this file. **A control that
must be reachable cannot be placed after an unbounded `ScrollArea`, and
reserve-and-hope is the same defect with a tuning parameter** — four
surfaces in this shell have met it. This one has no footer at all: the
dock tab carries the close control and
the Add button lives inside a fold at the bottom of the scroll region —
so there is nothing that *can* be pushed past the end.

`auto_shrink([false, false])` so the region claims the dock column even
when its folded content is six lines high. Without it a fully folded
panel would shrink to a stripe and the tab would look half-drawn.

### `fn group_list`

# Why the row shows the scale and the member count

Because those are the two things that tell an operator *which group this
is* when the names are `Plan` and `Detail` and they set them up an hour
ago. A list of names alone would be a list of words.

### `fn section`

The operator asked for it by name — *"each section should be able to fold up
like the settings one"* — and [`crate::dialogs::settings`]'s `widgets::group`
is the model. This is a second implementation rather than a call to that one
for a single reason worth stating: that function publishes its rect under
`settings.heading.<key>`, and a check aimed at the settings window would then
find headings belonging to a panel in a different surface entirely. A shared
helper taking a prefix would be the tidier answer and is worth doing the day
a third surface wants folds; two is not yet a pattern.

`ui_rect_visible`, not `ui_rect`, for the reason the settings window
learned the hard way: these headings live in a `ScrollArea`, `egui` lays out
the ones below the fold before clipping them, and publishing a rect for a
heading nobody can see makes a contrast check measure whatever is genuinely
at those coordinates — which on the settings window's first live run was the
Pages panel and the drawing behind the dialog.

The caption is **plain text**, never `.strong()`. `DEFECTS.md` D11: no theme
this project ships renders `.strong()` legibly on a panel, and
`tools/gates/check-strong-text.sh` refuses a bare one. The disclosure
triangle beside the caption is the whole of the emphasis and it is enough.

### `const NARROW`

`crate::app::modes::defaults::NAVIGATOR_WIDTH` is 280; the panel gets
that less the dock's own margins and the scroll bar it reserves. 250 is
the number `panels::pages`' own column test uses for the same reason.

### `fn no_row_in_this_panel_outruns_a_narrow_dock`

A row wider than the side bar hides part of a control with no scroll bar
to show the part that is missing, and the defect is **invisible by
construction**, which is why it needs a number rather than a look. A
`ScrollArea::vertical()`
clips horizontally and offers no bar in that axis: a row wider than the
column is cut off at the right edge, does not scroll, and reports
nothing. The control that ends up outside is unreachable and there is
nothing on screen to say it exists.

The widest row is `"no scale set — showing raw page units"` followed by
the **Set scale…** button — about 310 pt of content in a 250 pt column.
A `ui.horizontal` does not wrap; every row in this panel is
`horizontal_wrapped`.

## Why the assertion is on a measured overflow rather than on a
screenshot

Because the panel can measure itself exactly — `content_size.x` against
the scroll viewport's width — and a number that the application
computes is a better oracle than a rendering a test has to interpret.
The screenshot rule (`D:/dev/rag/egui/`) is about *reachability*
defects a trace cannot see; this one the application can see, so it is
made to say so.

## What it does NOT prove

That every control is legible, or that wrapping put things somewhere
sensible. It proves nothing is off the edge, which is the operator's
complaint exactly.

### `const REGION_DRAW_INTO_PREFIX`

Indexed by the **`GroupId`**, not by the row's position in the list. A row
index would change under a check the moment a group was added, which is
exactly what a check that adds a group is doing.

### `const REGION_ROW_PREFIX`

Distinct from [`REGION_DRAW_INTO_PREFIX`] beside it, and the distinction is
the window's own: the radio chooses where the **next dimension** goes, the
name chooses which group's **settings are on screen**. Collapsing them would
make inspecting a group silently redirect the next dimension drawn.

### `const REGION_HEADING_PREFIX`

Existence is the "open" state, as everywhere in [`super`] — there is no
`open: bool` that could disagree with whether the state exists.

**Almost nothing is held here**, and that is the design. The groups, their
scales, standards, styles and member counts are all read from
`EditSession::dimension_model()` on every frame. A local copy would be a
second source of truth for a model that this very window edits through an
action queue applied *after* the frame — so the copy would be stale for
exactly one frame after every change the operator made, which is the frame
they are looking at.

What is held is the four things the *document* does not know: which row the
operator is configuring, what they have typed into the new-group fields, and
the two one-shot requests that have to survive past the window closure.
The region a fold heading publishes; the section's stable key is appended.

The key is deliberately **not** derived from the caption, for the reason
[`crate::dialogs::settings::widgets::group`] records: a caption is operator
copy and may be reworded, and a check aimed at a region named after it would
then report a heading that is not there rather than a heading that is
illegible. Those are different verdicts and only one of them is true.

### `struct DimensionGroupsUi`

**Almost nothing is held here**, and that is the design. The groups, their
scales, standards, styles and member counts are all read from
`EditSession::dimension_model()` on every frame. A local copy would be a
second source of truth for a model that this very panel edits through an
action queue applied *after* the frame — so the copy would be stale for
exactly one frame after every change the operator made, which is the frame
they are looking at.

What is held is the four things the *document* does not know: which row the
operator is configuring, what they have typed into the new-group fields, and
the one-shot request that has to survive past the frame that raised it.

# Why `Default` rather than a constructor taking the authoring group

As a window this was built by `open(active)` and seeded its selection with
the group the operator was drawing into — *"an operator who opens this while
working has a group in mind and it is the one they are drawing into."* That
reasoning is still right and is still honoured, but it cannot live in a
constructor any more: a panel is not constructed when it is shown. It lives
on [`crate::panels::PanelsState`] for the life of the document and is reset
by `forget_document`, exactly like the Redact panel's query and the
Bookmarks panel's draft.

So [`Self::selected`] is an `Option` and the seeding happens on the first
frame that draws — see [`Self::show`]. `None` means *"whatever the operator
is drawing into"*, which is a better default than any `GroupId` because it
keeps following them until they say otherwise.

### `fn take_scale_request`

Called by `PdfcerApp::docks` immediately after the dock draws. Returning
it rather than acting on it is what keeps this module free of any
knowledge of the dialog layer.

### `fn body`

The entry point [`crate::panels::Panel::show`] calls, in the shape every
panel body has: the empty-document case never arrives here, because it is
answered once for all panels rather than eleven times.
