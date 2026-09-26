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

## Item notes

### `fn cursor_for`

It lives here rather than in `canvas::interact`, on the same seam
[`crate::canvas::gesture::press_kind`] sits on: the first rung is
[`CanvasTool::cursor`], so putting the remaining three in the wiring would
scatter one question across two places, leave it untestable, and make every
new tool a thing to remember in the wiring as well as in the enum.

# The order is the rule

1. **The armed tool**, when the pointer is over the canvas or a button is
   down. This rung is the whole of *"the cursor must change, and must change
   back"*: it changes because this branch is taken while the tool is active,
   and it changes back because the answer is recomputed every frame from
   [`active`] with nothing stored to restore. A dropped key-up costs one
   frame of hand, not a canvas stuck showing a grab cursor over a select
   tool.
2. **A gesture in flight**, which keeps its own cursor even once the pointer
   has wandered off the thing it started on — otherwise a drag that outruns
   its object looks like it stopped working.
3. **A hovered grip**, which is how the eight resize handles are findable at
   all.
4. **Nothing**, leaving the cursor to whatever else set it.

`pointer_down` is *any* button, because a middle-drag pan must show the
closed hand too; `over_canvas` is measured against the scroll viewport
rather than the page, because the hand pans the grey surround as readily as
the paper and a hand tool that shows no hand over half the canvas reads as a
tool that is not armed.

### `fn resolve`

Space *borrows* the hand; it does not choose it. So this is a `max`, not a
swap: holding space over the hand tool changes nothing, and releasing it
returns whatever [`selected`] has said all along.

### `fn selected`

This is what a ribbon toggle or a tool palette should render as pressed:
showing the *active* tool there would make the button flicker under the
operator's thumb every time they held space.

### `fn select`

# Why the trace is here and not only at each gesture

Writes the module header's `canvas-tool` line on a change. The
same-screenshot argument the five gesture lines each carry is a property of
**arming**, not of any one gesture, and the three arming paths that never
grew a line of their own are the proof: `view.tool_select`,
`view.tool_hand` and `view.tool_node` all reach the canvas through this
function, and all three were invisible to a harness.

Keyed on an actual change because the retirement paths
([`disarm_any`], [`retire_forbidden`], [`disarm_markup`]) call this
unconditionally once their own guard has passed, and because a re-arm that
changed nothing is not an event a reader is looking for. The previous value
is read through [`selected`] rather than from the raw slot, so an empty
memory reads as [`CanvasTool::Select`] — which is what the canvas has been
behaving as all along — and arming Select on the first frame is correctly
silent rather than a change from nothing.

### `fn toggle_hand`

Returns the tool now chosen, so a caller that wants to report or check the
new state does not have to ask again and risk reading a different frame's
answer.

### `fn toggle_text`

[`toggle_hand`]'s twin, deliberately down to the shape of the `match`: these
are the two pointer tools that carry no kind, they sit in the same ribbon
group, and a single press of either is how an operator both enters and leaves
it. The same-press-retires rule is [`arm_markup`]'s argument applied to a tool
with one kind instead of four — *the button is pressed, so pressing it is how
you un-press it* — and without it an operator who armed Text by mistake would
have no way back to the select tool except by arming something else.

# Why it returns to `Select` and not to whatever was armed before

Because nothing is stored to return to, and that is the same refusal this
module's header makes about the space bar: a "previous tool" is state that can
be lost, and losing it leaves the canvas in a tool the operator never chose.
[`CanvasTool::Select`] is this enum's `#[default]` and the stance every other
retirement path in this file returns to ([`disarm_markup`],
[`disarm_measure`], [`retire_forbidden`]), so a reader has one answer to learn
rather than four.

Note what that means in a **reading** mode, and it is deliberate rather than a
gap: in Read and Review the select tool already sweeps text
([`crate::canvas::textsel::takes_the_press`]'s original rule), so toggling
this off there changes the pressed control and changes no behaviour. The tool
is not suppressed in those modes for that reason — a control that vanished
from View in two of three modes would be a per-mode visibility rule invented
to hide a redundancy, and View is shown in every mode precisely so its
contents do not have to be.

Returns the tool now chosen, honouring the same report-rather-than-re-ask
contract [`toggle_hand`] and [`arm_markup`] do.

### `fn arm_form`

Same shape as [`arm_markup`] and deliberately so: pressing the armed button
again retires the tool, which is what makes a mis-click cheap and what stops
an operator hunting for a way to cancel.

The trace line is not decoration. A canvas armed with a form tool and an
un-armed one are **the same picture** — a crosshair is a cursor — so this is
the only way a driven check can prove the ribbon button armed anything at
all. It is the lesson of defect 8, applied to a new tool before the defect
can recur.

### `fn arm_markup`

# Why pressing the armed button again retires the tool

*"Make it work the way other programs do"* is the operator's stated
tie-breaker, and every drawing application treats a tool button as a toggle:
the button is **pressed**, so pressing it is how you un-press it. The
alternative — a button that only ever arms — leaves an operator who armed
Rectangle by mistake with no way back to the select tool except Escape,
which they have to know about, or arming some other tool, which is not what
they want either.

Choosing a *different* kind is not a toggle; it is a change of kind, and it
arms. So the rule is: same kind ⇒ retire, different kind ⇒ re-arm. That is
what makes the four Markup buttons behave as a radio you can switch off,
which is what they look like once each renders pressed.

Returns the tool now chosen, so a caller that wants to report or check the
new state does not have to ask again and risk reading a different frame's
answer — the same contract [`toggle_hand`] honours.

### `fn arm_text_annot`

[`arm_markup`]'s third sibling, with the identical same-kind-retires rule
and for the identical reason: a tool button is pressed, so pressing it is
how you un-press it.

The trace line is deliberately the **same event name** `arm_markup` emits.
From a harness's point of view — and from the operator's — these are markup
tools; that they take a different route to the document is an
implementation fact, and a second event name would make a check asking
*"did a markup tool arm?"* have to know which family it was about.

### `fn arm_measure`

[`arm_markup`]'s twin, with the identical same-kind-retires rule and for the
identical reason — see that function's header, which is the argument for
both. The two are separate functions rather than one generic over the kind
because the tools are separate: a shared one would have to take a
`CanvasTool` already built, which moves the "which variant" decision back out
to the four call sites this pair exists to keep it away from.

**It arms a tool; it authors nothing.** The clicks are taken by
[`crate::canvas::measure`], and only the pick that completes a dimension
raises an `Action`.

### `fn disarm_measure`

**Escape's claimant, alongside [`disarm_markup`]**, and it sits at the same
rung for the same reason — see [`crate::canvas::keys`]'s precedence table.

Note what this does **not** do: it does not discard a half-finished pick.
A linear dimension with point A taken and point B not is in-progress work
held by [`crate::canvas::measure::pick`], and Escape retires *one* thing per
press (decision 025's L1). So the first Escape abandons the pick and the
second puts the tool down — which is the order the operator means, because
the pick is the more transient of the two.

### `fn disarm_markup`

**Escape's claimant.** Reports rather than being asked twice, for the same
reason `zoom::disarm_region_zoom` does: the caller cannot know whether the
key was spent here without asking, and a caller that re-derived it would be
the version that retires the tool *and* ascends a selection rung. See
[`crate::canvas::keys`]'s precedence table for where this sits and why.

Deliberately reads [`selected`] rather than [`active`]: a held space bar
borrows the hand, and Escape pressed mid-space must retire the markup tool
underneath it rather than doing nothing because the *active* tool happened
to be the hand at that instant.

### `fn disarm_any`

The operator: *"Escape should get me out of a tool."*

# Why it covers every tool rather than a list of them

[`disarm_markup`] and [`disarm_measure`] between them reach two of the seven
tools. A ladder built only from those leaves the answer to *"how do I stop
doing this?"* depending on which tool the operator picked, which is not
something they should have to know. The convention is universal and has no
exceptions worth carving — every drawing program, every CAD package, every
vector editor: **Escape returns you to the pointer.**

# It is the LAST rung of the tool group, not the first

A tool with a gesture in flight spends the first Escape on the gesture and
stays armed — an operator correcting a mistyped character must not also be
putting the pen down. `canvas::keys`' ladder enforces that ordering and this
function is only reached once every in-flight claimant has declined.

So the sequence an operator experiences is the one they expect from
everywhere else: **Escape abandons what you are doing; Escape again puts the
tool down; Escape again backs out of the selection.**

# Why it does not touch a SPACE-held hand

Because that hand is not armed — `resolve` composes it over the selected
tool for as long as the bar is down, and `selected` reports what the
operator actually chose. Retiring it here would be retiring something they
have not picked, and it would come straight back on the next frame anyway.

### `fn arm_text_edit`

[`arm_markup`]'s twin, down to the same-press-retires rule and for the
identical reason — *the button is pressed, so pressing it is how you un-press
it* — and to the discarded return value at the call sites.

**Changing the kind settles the draft, and that is not the same as the
mid-drag rule above it.** `arm_markup` can be careless about a drag in flight
because a drag is owned by the gesture machine and carries the kind it
started with, so a kind change cannot reach it. A draft is not owned that
way: it sits in `egui::Memory` between frames, and an operator who types
three characters into a run and then presses **Add text** has asked for a
different verb against a different anchor. Carrying it across would commit
`Edit`'s text through `Add`'s engine call, which is the one reading that is
wrong about the document; writing it out under the verb it was typed under
is the one that is merely eager, and eager is undoable.

It is not done here. `app::frame`'s step 2d settles any draft whose caret
tool is not the one armed, and the `select` below is what makes that true —
so the kind change goes through the same statement every other tool change
does, and the two cannot disagree about it.

### `fn store_capabilities`

# Why this exists at all, when `Capabilities` is already threaded

It is threaded to everything that *gates a gesture* — `retire_forbidden`,
`takes_the_press`, `press_kind` — because those are called from the canvas
pass, which has `PdfcerApp` in scope. A **dock panel** does not: `Panel::show`
is handed `(ui, Option<&OpenDoc>, &mut PanelsState, Option<&MenuHost>,
&mut Vec<Action>)` and nothing else, which is the seam that keeps a panel
from reaching into the application.

`crate::panels::tool` has to answer *"what does a press mean in this
mode"* and *"which tools does this mode have"*, and both are this value.
The alternatives were worse in specific ways:

* **widen `Panel::show`** — a sixth parameter every panel takes and one
  panel reads;
* **re-derive from the ribbon's active mode inside the panel** — a second
  copy of `Capabilities::for_mode`, which would eventually disagree with the
  canvas about what a mode may do, and the symptom would be a panel that
  lies rather than a panel that crashes.

Parking it beside the armed tool is the smallest thing that works and it
keeps **one** derivation: `PdfcerApp::capabilities()` computes it,
`on_mode_capabilities_changed` stores it, everything else reads it.

### `fn capabilities`

Falls back to [`Capabilities::FULL`], and the fallback is the same
decision `Capabilities::for_mode` makes for an unknown mode, for the same
reason recorded there: a build with no validated manifest has no mode
taxonomy, and a shell that silently withheld every capability would be a
broken product rather than a safe one. Here it is also the honest answer for
the first frame, before any mode change has occurred — the application
starts in the manifest's first mode with its capabilities already applied by
`modes::start`, and the panel simply has not been told yet.
