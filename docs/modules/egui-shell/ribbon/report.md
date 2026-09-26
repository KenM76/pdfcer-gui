# `egui-shell/ribbon/report`

## Item notes

### `fn the_reported_names_are_a_stability_contract`

These strings are consumed by a harness in another tool, possibly
in another repository, by literal comparison. A rename is
therefore a breaking change with no compiler to catch it: the
harness keeps building, its assertions simply stop matching
anything, and a test that matches nothing passes.

Pinning the exact spellings makes a rename a deliberate act with a
failing test in front of it. If this test is ever updated, the
harness's selectors have to be updated in the same change.

### `fn a_reporter_with_no_sink_never_builds_a_name`

This is the zero-cost claim, and the only way to observe it is a
side effect inside the closure that is supposed not to run. If
this fails, every rect call site in the paint loop is allocating a
`String` per frame to throw it away.

### `type RectSink`

Takes the name by reference so the ribbon never has to allocate a
`String` the sink might not keep, and the rect by value because it is
four `f32`.

### `const ENABLEMENT_EVENT`

**A rect cannot answer "is it greyed", and this repository needed it to.**
Every control publishes a rectangle whether enabled or not, deliberately:
the question a consumer asks is *where is this control*, and a control that
is greyed is still a control that was drawn somewhere. That rule is right,
and it leaves a hole the exact shape of an operator saying *"that entire
area is always greyed out"* — a harness can prove a band's controls were on
screen and cannot prove that any of them could be pressed. The join
(predicate, conditions, renderer, this frame) has no oracle outside the
process at all; without this line it can only be asserted by a unit test
that re-evaluates the predicate against the conditions, which is the two
halves agreeing with each other rather than the shipped frame answering.

It is a separate LINE rather than a field on the rect report because
[`RectSink`] is `FnMut(&str, Rect)` and is consumed by three other surfaces
in two crates. Widening that signature to carry one boolean that only
command controls have would put an `Option<bool>` on every group caption and
every mode segment for ever.

It lives HERE, beside the rect names, rather than beside the renderer
that emits it, because it is the same kind of thing those names are: a
spelling a harness in another repository greps for, and therefore a
stability contract rather than an implementation detail. That is also what
makes it reachable from an application drawing a custom control of its own.

The line is emitted **on change**, not per frame, and carries `id=` and
`enabled=0|1`. An application that renders a custom item itself is expected
to emit the same event for it, and may ADD fields; `pdfcer-gui`'s font band
appends `live=` because it greys on a second predicate of its own, and the
disagreement between the two is the measurement worth having.

### `fn group_collapsed`

A distinct name from [`group`], deliberately. A collapsed group is on the
band and its items are not, which is a third state — the other two being
*expanded on the band* and *in the overflow menu* — and a driven check that
could not tell them apart would report a collapse as a disappearance. The
suffix means an existing check asserting `ribbon.group.<tab>.<id>` keeps
meaning exactly what it meant: **this group is drawn, expanded**.

### `fn group_caption`

This is the rect a legibility assertion wants: the caption is the
smallest text the ribbon draws, it is drawn `weak()` and `small()`,
and it is therefore the first thing to become unreadable under a
theme change or a scale change.

### `fn auto_hide_trigger`

Published even when auto-hide is OFF, and that is deliberate. The
question a driven check asks of this region is *"is the way back to the
ribbon on screen and big enough to hit"*, and the honest answer has to be
available in both settings — otherwise the check can only run in the state
it is trying to prove safe. It is reported through the visibility-gated
channel like every other ribbon region, so a trigger that laid out off
screen publishes nothing.

### `fn auto_hide_overlay`

Absent whenever the band is inline or hidden, which is what makes it an
oracle: `ribbon.band` exists in all three states and this exists in exactly
one.

### `fn mode_segment`

`MODES_AND_PANELS.md` Part 1 requires that the selector *"render as a
real segmented control with all three labels visible — not a bare
track with a knob, where the available positions are invisible until
you drag."* A per-segment rect is what makes that assertable: a
harness can check that every mode in the manifest produced a segment
with a positive area, which a whole-control rect cannot distinguish
from a track.

### `const OVERFLOW`

Published so a harness can assert `MODES_AND_PANELS.md` Part 2's
failure mode #8 against a running window: at a width where groups are
hidden, this rect must exist and have a positive area.

### `fn tab_overflow`

Distinct from [`overflow`], which is the *band's*. Both exist on the
same ribbon at the same time and answer different questions — "which
groups of this tab are hidden" versus "which tabs are hidden" — so a
harness that could not tell them apart would assert about whichever one
happened to be published first.

Spelled under `ribbon.tabs.` rather than `ribbon.tab.` so it cannot be
mistaken for a tab whose id happens to be `overflow`: [`tab`] builds
`ribbon.tab.{id}`, and the two namespaces stay disjoint.

### `fn trailing_item`

Its own name rather than sharing [`qat_item`], because the two regions are
at opposite ends of the row and a driven check that could not tell them
apart would pass on a build that had put the control in the wrong one.

### `fn band_item`

# Why this exists at all

The group caption, the tab, the mode segment and the QAT control all
publish a rect; the controls an operator actually clicks are inside a
group, and without this name no process outside the application could
say where one of them is, so nothing outside the application could
click one and observe what happened.

That gap has a precise cost. A capability can be present, unit-tested,
and never wired into the frame — an icon painter that is never handed
to the ribbon is invisible to every unit test in two crates and shows
up only in the width of a rect the *running* window declares (see
[`qat_item`]'s consumer, `tools/ui-verify`'s `qat_icons` check). A
control whose rect is unpublished is a control no such check can ever
be written for.

# The name: `ribbon.item.<command_id>`

Two decisions, both deliberate.

**`item`** is [`crate::manifest::Item`]'s own word. The band draws a
group's `Item`s, and `Item::Command` is the variant this reports; a
reader who greps the manifest for what a band contains finds the same
noun. It also stays clear of `ribbon.group.` — a *group* rect is the
block, a *caption* rect is its label, and an *item* rect is one control
inside it, so the three namespaces answer three different questions and
a filter for one cannot catch another.

**The command id alone**, with no tab or group segment, exactly as
[`qat_item`] spells a QAT control. A command id is unique in a
[`crate::commands::Registry`] — that is what a registry *is* — so the
name is already unambiguous, and it survives the one event that would
otherwise break every selector built on it: `SHELL_FRAMEWORK.md` §5
permits an operator to **move a command between groups**, and a name
carrying `.<tab>.<group>.` would change out from under a harness the
first time somebody reorganised their ribbon. The identity being
reported is the control's, not its current address.

# The one ambiguity this leaves, stated

A manifest that places the *same command id in two groups of one tab*
publishes two rects under one name, and a consumer keeping the last
occurrence per name would see them alternate. Nothing here forbids
that, because nothing here can: the manifest is the application's. It
is a manifest defect rather than a naming defect — the same command
twice on one tab is two controls the operator cannot tell apart — and
the alternative spelling would trade a visible ambiguity in a
diagnostic name for an invisible one in the ribbon itself.

# Zero cost when nobody is listening

Called once per drawn command per frame, which is the busiest reporting
site in this module — hence the closure discipline [`Reporter::report`]
documents. With no sink installed, no name is built.

### `struct Reporter`

A struct rather than a bare `Option` so that [`Self::report`] can own
the "do not format the name unless someone is listening" rule in one
place. Every call site in the ribbon goes through it.

### `fn is_listening`

Callers use this only to skip work that is expensive *beyond* the
name — computing a rect that is not otherwise needed, say.
Formatting the name is already deferred by [`Self::report`].

### `fn report`

# Why the name is a closure

Because it is an allocation, and this is called once per caption,
per segment, per tab, per QAT item, **per frame**. At 60 fps with
a seven-tab ribbon that is thousands of `String`s a second built
to be dropped unread, which is how a diagnostic hook becomes
something a profiler blames and someone deletes.

With no sink installed this function is one `Option` test and a
return; the closure is never called and no name is ever built.
