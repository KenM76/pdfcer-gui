# `text::window` — the way back out of a mode that hides its own control

Six strings, and **not one of them names a key.** Every chord below arrives
as a parameter, resolved by [`crate::app::window::chord_for`] from the same
keymap `app::keyboard` dispatches from. That is the rule
[`crate::text::shortcuts`] states for the keyboard reference, applied here
for a harder reason: the reference is read by somebody browsing, and these
sentences are read by somebody **stuck**.


> *"I didn't see a way to get back out of read mode. if there is a shortcut
> for this it should have a note what the key combo is in the top bar that
> holds the window controls."*

`view.read_mode` hides the ribbon and the docks. The only control that turns
it off — View ▸ Window ▸ Read mode — is on the ribbon. So the moment the
mode is on, **the control that undoes it is hidden by the thing it
toggles**, and the only remaining route is a chord that nothing on screen
names.

`app::window`'s header used to answer this with *"the tooltip on the control
states the chord before the operator presses it"*. That reasoning is
corrected in place there, and the short form is: a tooltip is a disclosure
available to somebody who already knows where to point, and a bound chord
can be pressed from memory or by accident having pointed at nothing.

## These sentences are CLAIM-BEARING, and that governs their shape

A sentence that says *press this key to get your application back* is a
promise the operator will act on while already frustrated. If the key has
moved, the sentence is **worse than silence**: it spends the one attempt
they were going to make and teaches them that the surface lies.

Two consequences, both structural rather than editorial:

1. **No entry here is `const fn` returning a fixed sentence with a chord in
   it.** Every chord-bearing entry takes `chord: &str`. There is no spelling
   of `Ctrl+H` anywhere in `crate::text`, and
   [`tests::no_string_here_names_a_key`] fails the build if one appears.
2. **There is a wording for "no key is bound".** A build whose manifest
   binds nothing to `view.read_mode` is legal (`SHELL_FRAMEWORK.md` §5 lets
   an operator rebind keys, and R8 lets a stripped build drop commands), and
   in that build the honest thing to show is not a chord and not silence but
   **a control** — see [`leave_read_mode_button`]. Silence there would be a
   room with no door at all.

## Two surfaces, two lengths, one fact

| entry | surface | why the length differs |
|---|---|---|
| [`title_read_mode`] | the window title | competes with a file name, a document count, the product name and a build stamp in a strip the taskbar truncates. Four words and a chord |
| [`status_read_mode`] | the status bar | one line on a bar with room, read by somebody who has already started looking. Says what comes *back*, which is the part that tells them the mode is the cause |

Both are drawn only while read mode is on. A permanent hint would be
furniture nobody reads, and it would be false the moment the mode is off.

## Item notes

### `fn no_string_here_names_a_key`

The rule this module exists to hold, asserted rather than trusted — and
the probe list is the one `crate::text::shortcuts` uses, because the
habit being caught is the same one.

A hand-written `"press Ctrl+H"` here would look entirely reasonable in
review, would be correct on the day it was written, and would become a
sentence that names a dead key the first time anybody rebinds anything —
on the one surface an operator reaches for when they are already stuck.

### `fn every_chord_handed_in_reaches_the_sentence`

The vacuous failure this forbids: a format string that drops its
parameter still compiles, still returns a plausible sentence, and would
pass any test that only asserted the sentence is non-empty.

### `fn the_status_line_names_the_ribbon_and_the_panels`

An operator in this state has noticed two things missing and does not
necessarily know the mode's name. A sentence that only said *"leave read
mode"* would require them to have made that connection first.

### `fn the_bound_and_unbound_wordings_are_different_sentences`

Two states, two sentences, and an operator seeing one message for both
cannot tell which they have — `crate::text::shortcuts`' own rule about
its two empty states.
