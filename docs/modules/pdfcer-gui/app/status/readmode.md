# `app::status::readmode` — the one line that says how to get the
application back

One sentence, drawn only while `view.read_mode` is on, at the left end of
the status bar and ahead of everything else the bar has to say.

# The report, and why the status bar of all places

The operator:

> *"I didn't see a way to get back out of read mode. if there is a shortcut
> for this it should have a note what the key combo is in the top bar that
> holds the window controls."*

He named the title bar, and the title carries it too — see
[`crate::text::doctabs::window_title`]. This is the **second** surface, and
two surfaces for one fact needs an argument: **read mode composes with full
screen**, and with both on there is no ribbon *and no title bar*, so a
title-only hint would be missing in exactly the state with the least chrome
left. The status bar is the one piece of chrome [`crate::app::window`]
deliberately keeps through both. In the other direction the title is legible
from the taskbar and from Alt-Tab, which this bar is not, and on a maximised
window whose bottom edge nobody is looking at. **Neither surface alone covers
the state space, and each is the other's blind spot.**

# It is FIRST on the left, ahead of the page-drag caption

`app::status`' header ranks the left half: the transient caption about a
gesture in progress outranks disclosures about finished gestures, which
outrank the narrator. This line outranks all three:

> **A sentence about how to reach the interface outranks every sentence
> about the document**, because an operator who cannot reach the interface
> cannot act on any of the others.

# It is never shed, and it never sheds anything else

* **Never shed.** `fitting::SHED_ORDER` governs the fixed cluster on the
  *right*, so no width pressure can remove a left-hand line. `fitting`'s
  reachability clause — *nothing it may drop is the operator's last route to
  that capability* — would forbid dropping this one anyway, and a rule
  enforced by construction beats a rule enforced by a list.
* **Sheds nothing.** It takes a bounded fraction of the width and elides with
  the whole sentence on hover, as the render notes do. An unbounded label here
  would push the page box and the zoom stepper off the right-hand end.

[`EXIT_WIDTH_FRACTION`] is wider than the notes' fraction for the reason the
ranking gives: this is the sentence somebody is *hunting for*, where the notes
are volunteered.

# R128 — it cannot change the bar's height

One label, on the row [`super::show`] has already allocated, of
[`super::ROW_HEIGHT_PTS`]. Nothing here wraps and nothing adds a line. Let
this sentence wrap and R128 arrives directly: the bar grows by a line, a
fit-to-viewport zoom recomputes from the smaller canvas, and the page shrinks
frame after frame.

# The two shapes: a statement, and (rarely) a control

| the keymap binds `view.read_mode` to… | what is drawn |
|---|---|
| a chord | **a statement** naming it. R9: this is not a placeholder and not a greyed control; it is a fact |
| nothing | **a button** that leaves read mode |

The second row is not hedging. A build whose manifest binds no chord to
`view.read_mode` is legal — the manifest's operator layer may rebind keys, and
R8 lets a stripped build drop commands — and in that build the ribbon control
is hidden, the chord does not exist, and **there is no way back at all short
of restarting the application.** A statement has nothing true to say there;
the choice is between a control and a trap.

With a chord bound the statement is the better surface: it teaches the
keyboard and leaves the bar a readout, where a button invites the operator
back to the mouse for something they now know a key for.

The button writes `egui::Memory` directly rather than raising an
[`crate::app::actions::Action`], on the same licence the Find toggle takes:
nothing here touches a document, so there is nothing for the undo log to hold
and nothing to order against.

## Item notes

### `const EXIT_WIDTH_FRACTION`

Wider than [`super::NOTES_WIDTH_FRACTION`] (0.45) on purpose — see the
module header's ranking argument. Bounded all the same, because the controls
on the right are what an operator uses to *read* the document they are in
read mode to read.

### `const EXIT_SLOT`

It carries the **sentence**, not a boolean, and that is what makes a
driven check able to fail correctly. `read-mode-exit shown=true` is
identical for a build that names the right chord, a build that names a chord
nothing is bound to, and a build that names no chord at all — so a check
reading it could only assert *something appeared*, which is the vacuous
shape this project has shipped before. The text is the claim; the trace
carries the claim.

### `fn statement`

The same four rules `super::disclosure::disclosure_line` applies — bounded
width, fixed row, elide rather than wrap, full text on hover — written here
rather than reused because that helper draws `.small()`, which is right for
narration the operator did not ask for and wrong for the one sentence they
are hunting.

### `fn the_ordinary_state_says_nothing_about_read_mode`

A check that only asserted the sentence *appears* would pass on a build
that showed it permanently — which would be furniture, and would be a
false statement for every minute the mode is off.

### `fn the_sentence_names_the_binding_the_keymap_holds`

The vacuous shape this forbids: a test that asserts a sentence exists
passes on a sentence naming the wrong key. What is asserted is the
identity of two derivations — the one this module draws, and the one
taken from `shell::manifest::built_in`'s keymap — so a rebind moves both
or fails here.

### `fn an_unpublished_context_names_no_key`

The failure this forbids is a default: `Ctrl+H` as a fallback would be a
second spelling of the binding wearing a fallback's clothes, and it would
be wrong in exactly the case it was reached for.

### `fn show`

Returns nothing: like the rest of the left half this is a readout, and the
one case that is not — the unbound button — acts on `egui::Memory` rather
than raising an action (see the module header).

**Called before [`super::show`]'s `Status::Open` guard**, deliberately. Read
mode is per *window*, not per document (`app::window` §3), so an operator can
close their last file while in it — and a bar that only explained the way out
when a document happened to be open would go silent in the state where the
window has the least in it.
