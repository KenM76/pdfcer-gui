# `canvas::tabnav` — Tab moves through what the operator clicked on

`OPERATOR_REQUESTS.md` O204:

> *"when I press tab while in a form I end up tabbing through the menus
> instead of the form items. The tab should tab through whatever space I
> have clicked on (example if I have an object selected on the canvase it
> should tab through to the next object as expected, and if I've clicked on
> a form item it should tab forward and shift-tab backwards to the next
> one."*

## Contract

Two halves, and they are in different parts of the frame.

1. **Before egui sees the input.** [`claim`] runs from
   `eframe::App::raw_input_hook`, removes the Tab key event from the raw
   input, and parks a [`Request`]. It claims only when the widget that egui
   says currently holds focus is the one the canvas [`publish`]ed last pass.
2. **During the canvas's own drawing.** The owning surface calls [`take`],
   gets the [`Request`], and moves its own focus.

A surface that wants the ring therefore does exactly two things: publish the
`egui::Id` of the widget it has focused, every pass it draws it; and take
and act on the request.

## Why the seam is `raw_input_hook` and nothing else can work

egui latches the focus move in `Focus::begin_pass` **from the `RawInput`
events**, before any application `ui` code runs (`egui::memory`, the
`Key::Tab` arms of `begin_pass`). By the time a widget could call
`ctx.input_mut(|i| i.consume_key(…))`, `focus_direction` is already set and
the walk to the next focusable widget is already going to happen. Consuming
the key later removes the *evidence* and not the *effect* — which is the
shape of fix that passes a unit test and leaves the operator tabbing through
the ribbon.

## Why ownership is an identity test and not a flag

[`claim`] asks whether `memory.focused()` **is** the published id. That is
stronger than "the canvas thinks it has a field open", and it is stronger in
the two ways that matter: a dialog, a panel text box or a ribbon search
field that has taken focus makes the test fail, so Tab goes where the
operator is actually typing; and a canvas that stopped being drawn — a dock
tab switched, the panel collapsed — cannot leave a stale claim behind,
because egui's own end-of-pass dead-man's switch drops the focus of an id
that was not used. **The identity test is also the freshness test**, which
is why no pass counter appears in [`Owner`].

## Item notes

### `fn owner`

The identity test the module header argues for, in one place because two
callers need it: [`claim`], and [`owns_focus`] for the surfaces that have to
stand aside from a key the ring is about to read.

### `enum Scope`

Carried so the two canvas rings can share one seam without either acting on
the other's press: a form field and a selected page object are focused by
different code and move by different rules (O204 decision 3 — the field ring
crosses pages, the object ring wraps within one).

### `fn publish`

Called every pass the widget is drawn. Cheap by construction: one
`ctx.data_mut` insert of a `Copy` value, which is the same cost as the
focus-tracking every other surface in this shell does.

### `fn release`

Published ownership is otherwise dropped by egui itself: a widget that
stops being drawn stops being focused, and [`owner`]'s identity test then
rejects the stale entry. This exists for the case that test cannot see —
a surface that is still drawn, still focused, and has handed the key to
somebody else. The measure tools are that case: Tab cycles the snap mode
while one is armed, and the owner is the same page id either way, so
merely declining to re-publish would leave the hook swallowing a press
with nothing to spend it on.

### `fn owns_focus`

For the one place outside this module that must ask: the space bar is the
canvas's hand-tool modifier, and a focused form button reads Space as
*toggle me*. Without this the bar pans the paper and a checkbox cannot be
ticked from the keyboard at all — the same shape as the operator's
*"it doesn't accept spaces"* about the text caret.

Not a second spelling of the typing guard
(`crate::canvas::textedit::composing`): that one answers *is the operator
composing text*, this one answers *does this canvas ring own the keyboard*,
and a focused push button is the case where those differ.

# Why it takes a scope rather than answering for the canvas as a whole

A focused PAGE owns Tab and nothing else — the space bar is still the hand
tool, which is the gesture the operator uses most on a drawing. Asking the
unscoped question would hand Space to the object ring, which has no use for
it, and stop the paper panning the moment a page was clicked.

### `fn claim`

Called from `eframe::App::raw_input_hook` — see the module header for why
that and only that. Removes every `Key::Tab` event from `raw_input` when it
claims, so egui never sees one and `Focus::begin_pass` never latches a
direction.

Ctrl+Tab and every other modified Tab are left alone: they belong to the
dock, and a ring that swallowed them would take a chord it was never asked
for.

### `fn take`

Consumes unconditionally when the scope matches. A request is parked at the
start of a frame by [`claim`], which only claims when the owning surface is
drawn and focused — so the surface that could take it is guaranteed to run
in that frame, and a request that outlives the frame would be a bug rather
than a press to replay.

### `fn discard`

For the surface that took ownership and then found it had nothing to move
to — an empty ring, a field that vanished under an undo. Leaving the request
parked would let the *next* frame's surface act on a press aimed at this
one.

### `fn step`

`rings` is `(page index, number of stops on that page)`, sorted ascending by
page, with no empty entries. `at` is `(page, index within that page's
ring)`. `cross` is O204 decision 3: `true` walks off the end of one page's
ring onto the next page's, `false` wraps within the page.

Returns `None` only when `at` names a page that has no ring — a focus that
has gone, which the caller settles rather than moves.

Wrapping is unconditional in both modes: the last stop leads to the first.
A ring that stopped at its end would make the operator's recovery from an
over-press a mouse gesture, and the convention across every program that
tabs through fields is that it does not.
