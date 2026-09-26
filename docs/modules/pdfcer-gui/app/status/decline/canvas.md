# `pdfcer-gui/app/status/decline/canvas`

## Item notes

### `enum CanvasDecline`

Two arms, and the arity is the point rather than an accident of how many
have been written so far.

# Why this type exists when [`super::Declined`] is right there

The canvas holds `&OpenDoc` and not `&mut`, so a gesture with something to
declare cannot write the store; it raises an action and the apply phase
writes. That much is this crate's standing shape, and it is not the
interesting part.

The interesting part is what the action may **carry**, and the answer is not
a [`super::Declined`]. That enum holds a variant for very nearly every
surface in the application, and nearly every one of them is
a sentence some other surface owns — a save that failed, a bookmark that
would not move, a reflow the engine refused. An action able to carry any of
them would let the canvas assert facts it has no way to establish, and would
turn one choke point into a general *print any sentence* channel.

`Action::DeclineInsideForm` named that failure mode in its own docs and set
the condition for widening itself: *“Adding an `InsideFormRefusal` payload
would be milder and is still wrong today” — “Add it the day the canvas can
raise the second one, not before.”* O188 is the day the canvas can raise the
second one. This enum is the *milder* widening that block described, made
one degree milder still: not the engine's refusal taxonomy, but a list of
sentences written here.

⇒ **A new arm is a deliberate act with a visible cost.** It means naming a
sentence the canvas is allowed to say, in a type whose whole documented
purpose is to stay short, and then answering [`super::Declined::still_true`]
for it. That friction is what this type is for.
# Why `pub`, when everything around it is `pub(crate)`

Because it is half of [`crate::app::actions::Action::DeclineOnCanvas`]'s
signature and `Action` is `pub` and genuinely reachable. A `pub(crate)`
payload on a `pub` variant is what the `private_interfaces` lint is for,
and under `-D warnings` that is a build failure rather than a note. Both
of `Action`'s existing payload sub-enums, `VectorAction` and
`RedactAction`, are `pub` for the same reason.

`app::prefs` records the other way out of this collision — keep the type
`pub(crate)` and hold it on a `pub(crate)` **field** of a `pub` struct —
and it is not available here. A variant's payload is the variant; there
is no field to demote.

⇒ **The module did not widen, and neither did the store.**
`app::status` still declares `pub(super) mod decline`, so the path
`crate::app::status::decline` is nameable only from inside `crate::app`,
exactly as before. [`record_canvas`] below is still `pub(crate)`,
[`super::LAST`] is still private, and `super::record_inside_form` is
unchanged. What crossed the boundary is one **type**, through one named
re-export at `crate::app::actions::CanvasDecline`.

That distinction is the whole justification. The rule
`pub(super)` enforces is *“a decline is written by the one dispatcher and
read by the one bar”* — a rule about **who may write**. A two-armed list
of which sentence a refused gesture may ask for is vocabulary, and asking
is what the action is. `Action` itself is already re-exported for the
canvas to name, on precisely this reasoning.

### `fn token`

# Why a token and not `{self:?}`

Because [`record_canvas`]'s trace line is read by a machine, and a
`Debug` rendering is a property of how the variant is **spelled**. Rename
an arm and every driven check keyed on it stops matching — silently, with
no compiler anywhere in the chain, and the check then reports *the sentence
never reached the operator* about a build in which it did. That is the
worst shape a diagnostic can have: a confident false negative that quotes
the truth in its own failure message.

⇒ These two strings are part of the harness contract, like
`status-group:decline` itself. Changing one is changing an interface;
`tools/ui-verify/src/checks/move_line_of_text.rs` matches them literally.

Twin of `canvas::moving::Refusal::token`, written the same day and
for the same reason. The pair is deliberate: the canvas names the cause,
this names the sentence, and a driven check that reads both can tell a
refusal that reached the store from one that was raised and dropped.
