# `ui-verify/checks/panel_tab_reorder`

`panel_tabs_can_be_rearranged` — dragging a panel's tab along its dock
strip moves it, marks where it will land while the pointer is down, and
does **not** change which panel is on screen.

# Why this needs driving

`egui_shell::dock::DockLayout::reorder_tab`'s arithmetic has unit tests,
including both directions of the boundary-to-index conversion and the
active-tab case. **Every one of them passes on a build where a panel tab
cannot be dragged at all**, because the arithmetic is a pure function of a
layout and two integers, and the gesture is three frame-level facts none of
them can reach:

1. the tab's `Button` senses a **drag** and not only a click — `egui`'s
   default is clicks alone, which is the state a strip of plain buttons
   ships in and looks entirely correct until somebody tries;
2. a drag begun on a tab survives to a release read from **raw pointer
   input**, because a drag begun on a tab ends anywhere;
3. the boundary the caret marked and the boundary the release used are the
   same decision, resolved once.

The crate's own driven tests cover the same three in a headless
`egui::Context`. This one covers what they cannot: the real binary, its
real arrangement, and a pointer driven through the OS.

# ★ Which strip, and why it is not the left one

**The left side draws no tab strip in this application.** The operator
asked for *"no tabs in the left side bar when the left rail is visible"*,
and `egui_shell::dock::Dock::with_rail_reach` delivers it — so the panels
reachable from the rail have no tabs to drag. The strip this check drives
is therefore whichever one the application actually drew, found by reading
the tab bars out of the trace rather than named here. A check that hard-
coded a compartment would start reporting a broken feature the day the
default arrangement moved a panel.

# ★★ The two assertions that are the point

**The caret was drawn.** A reorder that commits correctly and marks nothing
while the pointer is down has answered the wrong half of the feature — and
it is the half no capture taken afterwards can see, because the caret
exists only during the gesture. It is observable because the dock publishes
it as a region and the application's `ui-rect` channel is a change log, so
a completed drag leaves the declaration behind.

**The panel on screen did not change.** Rearranging tabs is tidying, not
navigation. The failure — a stack's active tab tracked as an *index* across
the move — leaves the strip in the right order with a different panel
showing, and nothing errors.

# What a passing run does NOT prove

* That the caret was drawn in the operator's accent colour, or that it is
  visible against the strip behind it. This reads its rectangle. A pixel
  oracle is the instrument for the other question and this is not it.
* That a tab can be dragged to a *different* compartment. It cannot yet,
  deliberately: there is no drop grammar behind such a gesture, so the dock
  proposes nothing over a strip the drag did not start in.
