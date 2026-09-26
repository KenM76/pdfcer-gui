# `egui-shell/ribbon/ctx`

## Item notes

### `fn the_local_condition_evaluator_agrees_with_enable`

There are two implementations of one rule: the canonical one in
`commands`, and this allocation-free copy used for contextual
tab visibility. That is a drift hazard with a nasty failure mode —
a `Format` tab that appears under conditions its author's enable
predicate would have refused, so the tab is present and every
control on it is disabled.

The copy is justified (it runs once per contextual tab per frame
and `Enable::When` would allocate a `String` to be dropped), but
it is only safe while something checks the two agree. This is that
something.

### `fn an_empty_condition_says_nothing_rather_than_never`

The one deliberate divergence from `Enable::When`, which has no
empty case because a command always carries a real predicate. A
manifest that spells `visible_when: ""` has said nothing, and
reading "nothing" as "never" would silently delete a tab from the
interface with no message anywhere.

### `struct IconRequest`

The application resolves [`Self::key`] against its own icon set —
`Command::icon` is a `String` key for the reasons given on that field
— and paints into [`Self::rect`].

### `struct Ctx`

Not `Clone`, not `Copy`, and never stored: it borrows the
application's callbacks for the duration of one
[`super::Ribbon::render`] call and is dropped at the end of it.

### `fn command`

# Why an unknown id is a skip and not a panic

[`crate::manifest::Shell::validate_against`] is supposed to have
caught this at load, and [`crate::manifest::merge`] is supposed to
have turned an operator's stale reference into a disclosed
[`crate::manifest::Skip`] before that. Reaching here means an
application rendered a manifest it did not validate — a
programming error, but one whose correct penalty is *one missing
control*, not a crash in the paint loop with a document open.

The trace is what stops it being silent. `SHELL_FRAMEWORK.md` §4
calls an unknown id a **disclosed skip**, and an undisclosed skip
is indistinguishable from a rendering fault — the lesson
[`crate::verify`]'s header records about a step that was dropped
without saying so.

### `fn id`

Derived rather than auto-generated so that ids are **stable across
frames even as the layout changes**. `egui` keeps focus, hover and
popup state per id; an auto-generated id shifts when a group is
collapsed or scrolled out of the band, and the symptom is a control
that loses keyboard focus when the window is resized — which reads
as a focus bug rather than as an id bug and is very hard to
attribute.

### `fn condition_holds`

Mirrors [`crate::commands::Enable::When`]'s language exactly — a bare
name is "this condition is set", a leading `!` negates — but without
allocating a `String` per contextual tab per frame, which is what
building an [`crate::commands::Enable`] to evaluate it would cost.

An **empty** condition is `true`: a manifest that says
`visible_when: ""` has said nothing, and the tab is not usefully made
permanently invisible by an empty string.

`the_local_condition_evaluator_agrees_with_enable` pins this against
the real implementation, because two copies of a rule that can drift
is exactly how a contextual tab ends up appearing under conditions its
author's enable predicate would have refused.
