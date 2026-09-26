# `app::toolstatus` — the one line that replaced the Tool panel

`OPERATOR_REQUESTS.md` **O123**, verbatim:

> *"I never understood why there is a tool dock when everything can be in
> object and properties. … The Tool panel becomes a one-line tool status
> (name, one sentence, 'Put this tool down'); its buttons duplicate the
> ribbon and go."*

## What moved where, and why nothing was deleted

The standing objection to collapsing the Tool panel is that it deletes the
armed block's live controls and orphans a disclosure slot. That objection
is right about the cost and wrong about the remedy, and the operator's
first sentence is why: the controls were never the tool panel's to hold.
They are properties of what is selected or about to be drawn.

| what the Tool panel held | where it is now |
|---|---|
| the armed tool's **name** and **stage** | here, on one line |
| **Put this tool down** | here, at the end of that line |
| the pointer sentence (Block A) | here, as the sentence for the resting tool |
| every stage's **second** sentence | here, in the strip's hover — see [`sentence`] |
| the **tool list** (Block B) | **gone**, on the operator's instruction — every row was a route to a ribbon command |
| the text pen's **font, size and colour** | `crate::panels::properties::tool` |
| the circular measure's **pick list** | `crate::panels::properties::tool` |
| the three **scale switches** | `crate::panels::properties::tool` |
| the **disclosure block** (Block C) | `crate::panels::properties::disclose` |

The tool list is the only genuine subtraction, and it is the one he asked
for by name. It answered a **discoverability** defect — `panels::tool`
exists because *"The feature works. He could not find it."* — and removing
it is his call to make and not this module's. What survives of that argument is the sentence on
this strip: it is permanent chrome, it names what is armed at frame one with
no clicks, and it cannot be closed, which is more than the panel could say.

## Why a dock banner and not a status-bar item

The strip has to be **beside the document**, permanently, and it has to have
somewhere to put a button. The status bar is under R128 — its row must not
grow — and it already carries an *elided* copy of the same disclosure slot
this change re-homes ([`crate::app::status::disclosure`]). A second, wider
claimant on that row is the exact feedback loop R128 exists to forbid.
[`egui_shell::dock::banner`] gives the right dock a reserved strip whose
height is a constant the dock takes off the top before it resolves the
columns, so nothing here can drive a width or a height.

## The resting state draws no button, and that is R9 rather than an omission

`CanvasTool::Select` **is** the resting state; putting a tool down returns
to it. A *Put this tool down* button beside `Select` would be a control
whose press changes nothing, and R9 forbids a dead control. So the button
appears when something is armed and is absent otherwise.

## And the strip draws its sentence even when it cannot name the tool

`OPERATOR_REQUESTS.md` **O66**. A `CanvasTool::Place` is armed from inside a
dialog that then hides itself, so there is no ribbon control to name and
[`command_for`] answers `None`, so there is no identity row to draw in
exactly that case. The *stage* line still draws, and its own comment says
why:
*"This one is the ONLY place the gesture and the way out are stated …
Deleting this line would strand an operator who has forgotten what they
armed."*

⇒ So a missing name suppresses the **name**, never the sentence. Written as
its own early-return rather than folded into the format string, because the
tempting shape — one `format!` with an empty name — silently ships a line
beginning with an em dash.

## Item notes

### `fn put_down`

It still writes `canvas::tool::select` directly rather than raising an
`Action`, and the argument is unchanged and worth repeating because a move
is exactly when somebody would "fix" it: **the armed tool is not document
state.** It contributes nothing to the undo log and has nothing to order
against, so routing it through the action funnel would add a variant `apply`
could only answer by writing the same memory slot.

### `fn name_of`

**Never a string of this module's own.** A second copy of a label compiles,
reads identically the day it is written, and drifts the first time either is
reworded — invisibly, because nothing renders both at once. The rule is
inherited from the armed identity row this replaces.

### `fn sentence`

# Why this returns a pair instead of one string

Six of the armed stages drew two labels, and the second was never
decoration. `t::text_annot_release` is described in its own module as *"The
sentence that stops a working tool reading as broken"*; `t::hand_borrow`
says how the borrowed hand is given back; `t::node_shift` names the modifier
that makes the node tool usable. A one-row strip has nowhere to put them,
and **dropping them would be the content regression this whole change is
under instruction not to commit.**

So they go to the hover, which is where this project sends a sentence that
has been elided rather than shortened — the discipline
[`crate::app::status::disclosure`] states as *"eliding defers rather than
loses"*.

# The primary is the LIVE stage where there is one

One slot, two contents — the armed block's rule, and it applies here with
more force rather than less. *"3 vertices placed"* is worth more than
*"click each corner"* the moment the operator has clicked one, and it
collapses back to the instruction when the run ends.

### `fn perimeter_stage`

Moved unchanged from the armed block, and the reasoning moves with it,
because it is the reason this function is not two lines of arithmetic:
[`pdfcer_core::dimension::format_measurement`] is the ENGINE's own
formatter — the same one the committed label goes through — so the running
total and the final label cannot disagree about scale, unit, precision,
fraction style or decimal marker. The operator's ask was that the tool
behave *"the same as the other dimensioning tools"*, and a live readout in
points beside a committed dimension in metres would be two numbers for one
measurement.

Falls back to the instruction when the group cannot be read: a total whose
scale is unknown is not a total.

### `fn circular_stage`

`OPERATOR_REQUESTS.md` O105 — *"selecting more points around a hole doesn't
always get it to narrow down to the size of the hole."* An operator adding
points to a fit is watching a number converge, and with no number to watch
every correction is a commit and an undo.

**Radius or diameter follows the pick set's own display toggle**, so the
number the strip shows is the number the placed dimension will show. A
readout that always reported the radius would disagree with a committed
diameter label by a factor of two, silently.

### `fn command_for`

# Derived from the existing id maps, never written a second time

`shell::commands::markup_command` and `measure_for_command`'s inverse are
the single binding between an id and a kind, exactly as
`Panel::from_command_id` is for panels. Re-listing them here would be a
second table to keep in step, and the failure when it drifted would be a
strip naming the wrong tool — which is the one thing it exists to get right.

Moved verbatim from the armed block; the `None` arms are the interesting
ones and each keeps its reason.

### `fn the_reserved_height_survives_the_shells_clamp`

Falsifiable in one edit: drop `BANNER_HEIGHT_PTS` below the shell's
floor and this goes red rather than the strip quietly disappearing at
run time.

### `fn only_the_two_dialog_armed_tools_have_no_command`

The table moved modules, and a move is where an arm gets dropped. This
asserts the `None` arms are exactly the two documented ones — a
placement, and the scale kind that is armed from inside a window — so a
tool silently losing its name shows up as a red test rather than as a
strip that renders a sentence with no subject.
