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
