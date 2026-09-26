# `app::modes` — Read / Review / Edit as named workspaces

`MODES_AND_PANELS.md` closes its analysis by identifying the two
requests that arrived together — a three-position mode selector, and
flexible panel areas — as one system, in one sentence:

> **A mode is capability (g).** Read, Review and Edit are three built-in
> named workspaces, shipped as defaults, each remembering the operator's
> arrangement of it.

This module is that sentence, implemented. It binds each mode the
**manifest** declares to a named workspace in the layout document that
[`crate::app::persistence`] keeps on disk, so that:

- each mode starts from a default arrangement suited to what that mode
  is *for*;
- the operator's own rearrangement of a mode is remembered, per mode;
- Read → Edit → Read restores **your Edit**, not a default.

## Three modes are configuration, not a built-in — on both sides

`egui-shell`'s workspace store ships **no names at all**, and says why:
*"an application that wants three modes registers three workspaces; one
that wants eleven registers eleven; one that wants none never calls this
module."* `SHELL_FRAMEWORK.md` §4 states the same rule from the
manifest's side — *Read/Review/Edit is a configuration, not a built-in*.

That rule binds **here** too, and this module honours it in the one way
that matters: [`Modes`] holds whatever mode ids
`crate::shell::manifest::built_in` declares, in that order, and has no
opinion about how many there are or what they are called. Adding a
fourth mode to the manifest is one line in the manifest; nothing here
changes, and nothing here needs to.

## Why the arrangements themselves live in [`defaults`]

`app/modes.rs` was one file until it reached 1,512 lines against the
1,500-line gate (R2). It was split rather than trimmed, for the reason
the gate itself gives — *"the right response to this gate firing is to
SPLIT THE MODULE, not to shrink the prose"* — and along the seam the file
had already drawn between its own two halves. `app/mod.rs` has been split
twice under the same rule, into [`crate::app::dispatch`] and
[`crate::app::conditions`], and the pattern is deliberately the same one.

The two halves answer two different questions:

* [`defaults`] answers **what a mode's arrangement *is***. Which panels
  Read mounts, on which side, how wide. It is pure: a mode id in, a
  [`DockLayout`] out, with no file, no dock and no document anywhere in
  its argument lists. It changes when the *information architecture*
  changes.
* **This file** answers **how an arrangement is *remembered***. Workspace
  naming, the adopt-on-mode-change sequence, the debounced write, the
  reconciliation that stops a newly shipped panel being born invisible,
  and the start-up order. It changes when *persistence* changes.

That is a seam and not arithmetic: the last change to [`defaults`] was a
taxonomy answer from the operator (Read fills forms), the last change to
this file was an upgrade-path mechanism (`Unseen`), and neither would have
needed to touch the other. The dependency runs one way — this file calls
[`layout_for_build`]; [`defaults`] calls nothing here — which is what makes
the split honest rather than a pair of files that both have to be open.

[`layout_for`], [`layout_for_build`] and [`ABSENT_PANELS`] are re-exported
below, so every path a caller used before the split still resolves.

## What a mode change must **not** do

`MODES_AND_PANELS.md` Part 1's behavioural rules, and the first two are
the ones this module is accountable for:

> 1. **Switching modes never destroys work.** Read ⇄ Edit is a view
>    stance, not a save boundary. Unsaved edits survive a trip through
>    Read mode untouched.
> 2. **The undo stack is not cleared, ever.**

That is enforced **structurally** rather than by care:
[`Modes::on_mode_changed`] takes a [`DockState`] and a
[`LayoutStore`], and neither of them can reach a document, a selection,
an edit session or an undo stack. There is no path from this module to
any of them, and `switching_modes_touches_neither_the_document_nor_the_selection`
asserts the consequence against a real open document anyway — because a
later edit that *adds* such a path should fail a test rather than pass a
review.

## What this module deliberately does not do

- **It does not change which tabs the ribbon shows.** That is the
  manifest's `Mode::tabs` and `egui-shell`'s renderer; a mode's tab set
  and a mode's panel arrangement are two different things that happen to
  share a name.
- **It does not set the page-display default.** `MODES_AND_PANELS.md`
  Part 1: *"Read defaults to continuous scroll; Review and Edit default
  to single page."* That is a `crate::viewer` concern — there is no
  continuous-scroll display mode in this build yet — and it is recorded
  here only so the next reader knows it is a known, deliberate omission
  rather than a missed row of the table.
- **It does not decide the start mode.** [`start`] does: it adopts the mode
  remembered from the last session, falling back to the manifest's first.
