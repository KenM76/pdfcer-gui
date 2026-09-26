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

## Item notes

### `fn registered_panels`

Derived from [`Panel::ALL`] filtered through the live catalog, so it is
the same set the §5b capability rule uses — a build with a capability
compiled out reports fewer, which is exactly right: a panel that does not
exist here must not be recorded as one this layout has seen, or removing
and restoring a capability would leave it permanently invisible.

### `fn adopt`

Returns the ids actually added, for the trace.

# Why the default's placement rather than a fixed corner

A panel appended to the end of the first stack would land beside
whatever happens to be there, which for Pages means tabbed behind
Bookmarks on the day it appears — the operator sees a tab bar grow by one
and has no reason to think a feature arrived. Placing it where the mode
intended puts it where the documentation, the mockups and the next
release's default all agree it goes.

A panel already present is skipped rather than duplicated. That is not
defensive: `Unseen::Unknown` deliberately reports every registered panel,
including ones the restored layout already mounts.

### `fn assemble`

The operator, 2026-08-26, reporting the consequence rather than the cause:
*"I can't figure out how to click on objects to edit them."* Part of that is
an engine limitation and is filed as one — but the part nothing explained is
that the program opened in **Read** on every launch, where a click on page
content selects nothing at all, and no surface said so. Someone who spent an
afternoon in Edit came back the next morning to a program that had silently
forgotten.

# The three ways this can decline, all of which land on the first mode

1. **No stored id** — a fresh profile, or a file written before this field
   existed.
2. **An id the manifest no longer declares** — a mode renamed or removed in
   a customized manifest between two runs. `is_known` is what catches it,
   and the fallback is what stops a rename leaving the shell with no mode.
3. **No modes at all** — a manifest that failed to validate. `first()` is
   `None` too, and nothing is adopted.

Note what is *not* checked: whether the stored mode is one this build
considers safe or sensible. It is the operator's own last choice, made in
this program, and second-guessing it would be the program deciding it knows
better than the person using it.

### `fn registry`

Duplicated in [`defaults`]' own test module rather than shared,
because a `#[cfg(test)]` helper reachable across module boundaries has
to be made visible in the non-test build too. Six lines of fixture is
the cheaper of the two costs.

### `fn the_mode_list_is_whatever_the_manifest_declares`

`SHELL_FRAMEWORK.md` §4 makes Read/Review/Edit a configuration
rather than a built-in, and `egui-shell`'s workspace store refuses
to ship three magic names for the same reason. This module is the
third place that rule could have been broken — and the place where
breaking it would look most reasonable, because it is the one that
legitimately knows what "read" means as an arrangement.

Asserted by driving a manifest with *different* modes: a fourth mode
must be adoptable, and a mode the manifest does not declare must
not be.

### `fn a_mode_remembers_the_operators_own_arrangement_of_it`

The behaviour the whole module exists for, and the one
`MODES_AND_PANELS.md` Part 1 rule 3 states: *"Each mode remembers
its own panel layout. Leaving Edit and coming back restores the
arrangement, not a default."*

### `fn a_modes_arrangement_survives_a_restart`

The round trip that makes the previous test worth anything: the same
sequence, through a real file, across two `Startup`s. A rearrangeable
layout that forgets itself each restart is worse than a fixed one.

### `fn a_stored_mode_the_manifest_no_longer_declares_is_declined`

The case is real rather than theoretical: the mode list comes from a
manifest an operator may customize, and renaming a mode between two runs
leaves the previous run's id stored and unresolvable. Without the
`is_known` filter the shell would adopt nothing, which is a state with no
ribbon tabs and no way back.

### `fn switching_modes_touches_neither_the_document_nor_the_selection`

`MODES_AND_PANELS.md` Part 1 rule 1: *"Switching modes never
destroys work. Read ⇄ Edit is a view stance, not a save boundary."*

The argument list of [`Modes::on_mode_changed`] already makes this
impossible — it can reach a `DockState` and a `LayoutStore`, and
neither can reach an `EditSession` — so this test exists to make a
*later* edit that widens that argument list fail here rather than
pass a review.

### `fn a_layout_that_predates_a_panel_gains_it`

A layout written before a panel existed — no `known_panels` at all,
which is every file any existing install has — must gain the panel,
and must gain it *where the mode default puts it*.

Simulated the way it actually happens rather than by constructing the
end state: a workspace is saved holding ONLY Bookmarks (which is what
Read's remembered layout contained before Pages was registered), the
store is then read back through the real `on_mode_changed`, and the
result is asserted.

### `fn adoption_does_not_re_open_a_side_the_operator_collapsed`

The other half of the rule, and the reason `adopt` keys on
`columns.is_empty()` rather than on `!visible`. `SideLayout::visible`'s
own documentation calls hiding *"a view state, not a destruction"* — so
a populated side that is hidden carries a decision, and a new panel
must join the arrangement it keeps rather than overrule it.

Asserting the collapse survives is what makes the pair a rule instead
of a patch: a fix that simply set `visible = true` whenever anything
was adopted would pass the test above and re-open a dock the operator
closed, on every release that adds a panel, forever.

### `fn the_upgrade_adoption_happens_only_once`

The property that makes the `Unknown` branch acceptable. Without it,
every launch would re-open every panel the operator had closed, which
is a far worse bug than the one being fixed — it would undo a decision
they made, repeatedly, forever.

### `fn a_fresh_install_gets_the_default_and_nothing_else`

There is no remembered workspace, so the default arrangement is used
whole and the adoption path is never entered — asserted because a
reconciliation that also fired on first run would be indistinguishable
from one that worked, right up until it added a panel twice.

### `fn an_adopted_panel_is_raised_and_not_merely_mounted`

The regression test for the defect that made the (since retired) Tool
panel — built to answer *"no side bar area showing what tool is
active"* — invisible to the operator who reported the gap, for its
entire life, on the profile he had been running for two weeks.

The panel in the assertion is now Layers, because the Tool panel was
dissolved by `OPERATOR_REQUESTS.md` O123. The property under test is
`adopt`'s and has nothing to do with which panel arrives — but naming a
panel that no longer exists would have made the test read as being
about a surface, which it never was.

`stack.tabs.push` mounted it and left whatever was active still active,
so it landed behind another tab: present in `layout.ron`, present in the
tab strip, and never seen. For a panel whose whole purpose is
discoverability that is identical to not shipping it.

Found by a driven check asking *what does a first frame show* — not by
any of the tests of `adopt`, every one of which asked whether the panel
was PRESENT. Presence was never in doubt.

### `const MODE_WORKSPACE_PREFIX`

A workspace name is free text an operator chooses, so a mode's own
workspace has to be distinguishable from one the operator made and
happened to call "Read". The prefix does that, and it does two more
things worth having:

- it is the **mode id**, not the label, so renaming or translating
  "Review" does not orphan the arrangement behind it;
- it makes the machine-owned entries filterable, so a future "load
  workspace" menu can list the operator's own and leave these out — see
  [`mode_of_workspace`].

### `struct Modes`

The ids come from the **manifest**, in the order it declares them, and
this type has no opinion about how many there are or what they are
called — see the module header. What it owns is the binding between a
mode and its remembered arrangement.

### `fn from_shell`

`None` — a manifest that failed to validate — yields no modes, and
every method below then declines rather than inventing one. A build
whose ribbon could not be assembled must not silently acquire a
three-position mode model from somewhere else.

### `fn on_mode_changed`

The whole feature, in five steps:

1. A mode the manifest does not declare is declined — an unknown id
   must not acquire a workspace, or a typo in a customized manifest
   would quietly accumulate arrangements nothing can ever restore.
2. Re-adopting the mode already in force does nothing, so a caller
   may drive this straight from `RibbonState::mode()` every frame
   without checking first.
3. The **outgoing** mode's workspace is written from what is on
   screen right now. This is what makes "each mode remembers your
   arrangement of it" true without an explicit save.
4. The **incoming** mode's workspace is restored if it has one, and
   otherwise its built-in default is used — filtered through
   `catalog`, so a saved-but-stale panel and a compiled-out one are
   handled identically.
5. The result is recorded, which arms the debounced write. A crash
   after a mode change therefore costs nothing.

Returns whether the arrangement was changed.

**It cannot touch a document.** See the module header: the argument
list is the proof, and a test asserts the consequence anyway.

### `fn record_layout`

Called when the dock reports
[`egui_shell::dock::DockFrameReport::layout_changed`].

**Both, and that is the point.** The document's `active` is the
arrangement in force; the mode's workspace is the arrangement to
come back to. Writing only the first would mean a crash mid-session
cost the operator every rearrangement they had made since the last
mode change, which is the "only saved at exit" failure wearing a
different hat. Writing only the second would leave `active` stale in
a file an operator may read.

Idempotent: recording an arrangement that is already recorded arms
no write, so a caller that calls it unconditionally costs nothing.

### `fn reset`

`RIBBON_IA.md`'s rule is why this has a scope at all: *"an operator
who only wanted the right dock back must not lose their left one."*
The scoping itself is `egui-shell`'s; what this adds is the one
thing the shell cannot know — **which** default, given that the
right default depends on the mode in force.

Saved workspaces are untouched, including the current mode's, which
is then immediately overwritten by [`Self::record_layout`] with the
reset arrangement. That is the intended reading of "reset this
mode": the mode goes back to its default and remembers that it did.

Returns whether anything changed.

### `struct Startup`

Returned as a struct rather than a tuple because three values whose
types are `Modes`, `LayoutStore` and `DockState` are easy to bind in the
wrong order and hard to notice having done so.

### `fn start`

The whole start-up sequence, in one call, because its order is
load-bearing and getting it wrong is silent:

1. The mode list comes from the manifest.
2. The **fallback** handed to the loader is the opening mode's default,
   so a first run — or a file that could not be parsed at all — starts
   from an arrangement that suits the mode the application opens in
   rather than from some other mode's.
3. The document is loaded, fail-soft, with `catalog` deciding which
   saved mounts this build can honour.
4. The opening mode is adopted, which restores its remembered
   arrangement if it has one.

## Why step 4 may discard the file's `active` arrangement

The document's `active` is *"the arrangement in force"* — in force in
whichever mode was showing when the application last closed, which is
not necessarily the one it now opens in. The mode's own workspace is the
better answer to "what should Read look like", so it wins. `active` is
still kept current by [`Modes::record_layout`], because it is what a
person reading the file expects to find and what the loader falls back
to if a workspace has to be dropped.
