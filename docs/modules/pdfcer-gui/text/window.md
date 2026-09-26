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

### `fn title_read_mode`

It goes at the **front** of the title, and that is the same decision
`crate::text::doctabs`' header makes about the unsaved marker, for the same
measured reason:

> *A tab is truncated from the right with an ellipsis when the strip is
> crowded … A trailing marker is the first thing the ellipsis eats.*

A taskbar button holding `SW41177.pdf — pdfcer — 2026-…` has already eaten
everything after the file name. A hint at the end of the title would be
legible only on a window whose title bar has room, which is not the window
of somebody who has just hidden all their chrome.

It also leaves the **build stamp last**, which
`tools/ui-verify`'s `the_title_bar_carries_the_build_time` parses by
splitting from the right. Prefixing was free there; appending would have
silently re-aimed that check at this sentence.

Terse — *exit*, not a full sentence — because it shares the strip with four
other facts and is read at a glance.

### `fn title_read_mode_unbound`

It names the surface that does have a way out rather than a key that does
not. The status bar is the one piece of chrome read mode keeps, so this is a
direction the operator can follow rather than a shrug.

### `fn status_read_mode`

It says what comes **back**, not what is hidden. An operator reading this
bar is looking at a window with no ribbon and no panels and is trying to
work out whether that is a mode or a fault; *"the ribbon and the panels"*
names the two things they have noticed missing and attaches them to a key.
A sentence that said only *"press this to leave read mode"* would require
them to have already worked out that read mode is what they are in.

Named in the same two words the command's own promise uses
([`crate::text::commands::view_read_mode`]: *"Hide the ribbon and the panels
…"*), so the sentence that turns it off is the sentence that turned it on,
read backwards.

### `fn status_read_mode_and_fullscreen`

This is the one state in which `view.fullscreen` is also a trap, and the
reason is compositional rather than intrinsic: full screen hides no chrome
of pdfcer's own, so its ribbon control is normally right there — but read
mode has taken the ribbon away, and with it that control. Two hidden
controls, two chords, one line.

It is deliberately **not** shown for full screen alone. See
`crate::app::window`'s header: an always-on `F11` hint would be furniture,
and it would be wrong the moment the mode is off.

Read mode leads because it is the mode that removed the chrome; full screen
merely removed the title bar, which is why this sentence cannot live in the
title.

### `fn status_read_mode_unbound`

Paired with [`leave_read_mode_button`], which follows it on the bar. It
states the fact that makes a button necessary rather than leaving the
operator to infer it, because *"there is a button here"* and *"there is a
button here because your build has no key for this"* are different amounts
of information and the second one is free.

### `fn leave_read_mode_button`

A control rather than a sentence, and this is the one place in the
feature where that is right. R9 forbids drawing a control that cannot work
and forbids placeholders; it does not forbid the only working route to a
capability. With a chord bound, a statement is the better surface — it
teaches the keyboard and leaves the bar a readout. With **no** chord bound,
a statement has nothing true to say, and the alternative to a button is an
application whose ribbon and panels cannot be recovered without restarting
it.

A label, not a glyph: this button appears on a bar the operator has never
seen it on, in a state they did not mean to be in, and an unfamiliar icon
there is a puzzle rather than a route.
