# `canvas::moving::nudge` — the arrow keys, and the one point they move

Every drawing program moves a selected thing with the arrow keys, and until
today this one did not: an operator who had placed a revision cloud two
points too far left had to grab it with the pointer and drag, at whatever
precision the zoom happened to give them. At 100 % that is a two-pixel drag
nobody can make reliably, and the correction is the one edit an operator
makes most often.

## Why this is small, and why it is in `canvas::moving` rather than in
`canvas::keys`

**Because the verb already exists and is already called.** A nudge is a
⚠ **Annotations only.** `MANUAL.md` tells the operator the arrow keys nudge
*"what is selected"*, and with a content object selected they do nothing and
say nothing. `DEFECTS.md` D43.

**delta**, and `EditSession::move_annotation` takes a delta —
[`crate::app::actions::annot::AnnotAction::Move`] is raised by
[`crate::canvas::annotdrag`] on the release of a pointer drag and this raises
the identical action with the identical operands. There is no second verb, no
second arithmetic and no second set of refusals; what is new is a second way
to say *how far*.

That is also why it lives here. [`super`] owns the one function in `canvas/`
that crosses into PDF space ([`super::page_delta`]) and owns the module docs
that argue the crossing. A nudge that did its own trigonometry in the key
handler would be a **second derivation of the page transform**, which is the
precise failure `viewer`'s header warns about:

> *"PDF user space is y-UP; canvas and screen are y-DOWN. The failure is
> silent — the page looks perfect until someone selects a line and gets a
> different one."*

## THE Y SIGN, and how it is got right without deciding it here

An operator's **Up arrow means up on the screen.** PDF user space has y
increasing *upward* from the bottom-left corner (§8.3.2.3), canvas space has
it increasing *downward*, and a page may additionally carry `/Rotate 90`, in
which case screen-up is page-**left**.

Three facts, and this module knows exactly one of them: **screen-up is
negative y in canvas space.** So Up builds the canvas-space vector
`(0, -step)` and hands it to [`super::page_delta`], which is the function the
pointer drag already uses and which composes the flip and the rotation
together by inverting the page's own device transform. Nothing here writes a
minus sign into a `dy`, nothing here reads `/Rotate`, and a page turned on
its side nudges in the direction the operator pressed **for free**.

⇒ The precedent is [`crate::canvas::annotdrag::drag`], which converts its
pointer travel the same way and for the same reason, one line before it
raises the same action. A wrong sign here would be invisible in a unit test
that asserted `dy > 0.0` and obvious in one second of use; this arrangement
makes the assertion *"Up produces the same delta a one-point upward drag
produces"*, which is a claim about agreement between two surfaces rather than
about arithmetic.

## The step, and which program it is borrowed from

**One PDF point bare, a quarter point with Ctrl. That is Acrobat's
convention**, and it is chosen over the drawing programs' for one reason that
is local to this canvas.

| program | bare arrow | modifier |
|---|---|---|
| **Acrobat** | 1 pt | a modifier gives a **smaller** step |
| Illustrator / InDesign | the keyboard increment | **Shift** gives a **10×** step |
| Inkscape | 2 px | Shift 10×, Alt 1 px |

⇒ **Shift is spoken for on this canvas and must not be given a second
meaning.** It constrains a drag to one axis
([`crate::canvas::constrain`], applied above both branches of the annotation
fork), it locks a node drag to one axis
([`crate::canvas::annotnodes`]), and it makes a resize uniform
([`crate::canvas::scaling`]). Three gestures, one meaning — *this movement
shall be along one axis* — and a fourth gesture in which it meant *ten times
further* would be the chord that means two things, which is worse than a
missing chord.

There is a tempting counter-argument and it is worth writing down so it is
not re-made: an arrow key is **already** axis-locked by construction, so
Shift's existing meaning is vacuous for a nudge and the chord is "free". That
is true and it is not enough. What the operator learns is *Shift constrains*;
a Shift that multiplied would teach them that Shift means whatever the
current gesture felt like, which is the thing a convention exists to prevent.

**Alt is spoken for too**, and mechanically rather than by convention:
the built-in keymap binds `Alt+Up` and `Alt+Down` to `pages.move_up` and
`pages.move_down`. So this module refuses **any** modifier shape but the two
it claims, and does so by reading the modifiers itself rather than trusting
[`egui::InputState::consume_key`] — that function matches with
`Modifiers::matches_logically`, which **ignores extra Shift and Alt**, so a
pattern of `NONE` would have fired on `Alt+Up` and nudged a mark while
reordering a page. See [`step_for`].

## One undo entry per keypress, including auto-repeat

`egui`'s `key_pressed` counts key-repeat events, and this raises one
`AnnotAction::Move` per press — so holding the key walks the mark across the
sheet a point at a time and leaves one undo entry per point. That is what
Illustrator, InDesign and Acrobat all do, and the alternative (coalescing a
held key into one entry) needs a notion of *gesture end* that a keyboard does
not offer without a timer. It is a nice-to-have and it is deliberately not
built: correctness first, and one press → one entry is the correct half.

## Item notes

### `fn step_for`

# Why the modifiers are read rather than matched by `consume_key`

[`egui::InputState::consume_key`] matches with
[`egui::Modifiers::matches_logically`], whose documented behaviour is that
**extra Shift and Alt modifiers are ignored**. So `consume_key(NONE,
ArrowUp)` fires for `Shift+Up`, for `Alt+Up` and for `Ctrl+Alt+Shift+Up`.

`Alt+Up` is bound in the built-in keymap to `pages.move_up`. A nudge written
the obvious way would therefore have moved the selected mark **and** the page
it is on, from one press, and the second effect would have been invisible in
any unit test that injected a bare arrow.

⇒ So the shapes are enumerated here, exhaustively and exclusively, and
anything else declines. `command` rather than `ctrl` for the fine step: it is
Ctrl everywhere and Cmd on macOS, which is `crate::app::keyboard`'s standing
rule for every chord this shell reads.

### `fn direction`

The one place a sign is written in this module, and it is a screen fact
rather than a PDF one: canvas space is y-down, so *up* is negative. The flip
into PDF's y-up and any page rotation are [`super::page_delta`]'s, which is
the whole point of routing through it — see the module header's section on
the Y sign.

### `fn refuse`

Both, not one. The trace is what a driven check reads and what a harness on
a machine nobody can see reports from; the status row is what the operator
reads. They carry the same fact in two registers, and neither substitutes for
the other — `crate::canvas::deleting::decline` is the same shape and states
the same reason.
