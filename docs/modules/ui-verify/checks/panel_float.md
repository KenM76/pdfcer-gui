# `ui-verify/checks/panel_float`

`panels_float_close_and_dock` — **a panel tears out into a real OS
window, draws its body in it, comes back where it came from, and can be
closed.**


# What this is for


> *"you understand that there are options to float, close, and dock those
> panels … No shortcuts or lazy half-implementation."*

The unit tests in `egui_shell::dock::float` prove the **state machine**:
float records the home, docking rebuilds it, a stale home still lands, a
close removes the entry. Every one of them is headless and none of them
opens a window. This check answers the question they structurally cannot:
**did the platform actually give us a window, and is the panel's body in
it?**


The 2026-09-05 full sweep reported this check as **FAIL** with two
findings, filed as A5: *"the float window published no viewport-tagged
`ui-rect`"* and *"`view.dock_all_panels` traced `docked=0`"*. It also
recorded that the same check had been failing with `moved=false` "for
days" in other sweeps.

**All three were this file's, and not the application's.**

## 1. The check was not hermetic — it fed its own next launch

Each of the four sections launches the binary afresh, and the application
**persists the dock layout to `userdata/layout.ron` on exit**. So section
A's launch left Layers *floating*, and section B's launch — which begins
by floating it — found it already floating and traced `moved=false`.
Section C left it *closed*, so section D's float found no panel to take
out of a stack, floated nothing, and `view.dock_all_panels` then honestly
answered `docked=0`.


```text
float     04:51:34  panel-float  moved=true    → saved: layers FLOATING
dock      04:51:36  panel-float  moved=false   ← already floating
                    panel-dock   moved=true    → saved: layers DOCKED
close     04:51:38  panel-float  moved=true
                    panel-close  closed=true   → saved: layers ABSENT
dock-all  04:51:40  panel-float  moved=false   ← nothing to float
                    panels-dock-all docked=0   ← HONEST
```

⇒ And the leak crosses whole runs, which is why the check looked
intermittent: a run beginning after a previous run's section C starts with
Layers **closed**, and every one of its four sections fails. That is
exactly the `moved=false` sweep, reproduced by nothing but running the
check twice.

**`docked=0` was a true statement about a state the check had put the
application into.** This project keeps meeting *"a count that reads as
success when nothing happened"*; this is its mirror — a count that reads
as failure when nothing was there to count — and the defence is the same
one: make the precondition an assertion instead of an assumption.

**Fix:** every section now begins with `view.reset_layout`, which restores
the mode's default dock whole, and the check **asserts the reset happened**
(`layout-reset`) before believing anything downstream of it. A section that
could not reset says so instead of reporting the feature broken.

## 2. The window-is-empty oracle was blind — it could not have passed

The old assertion was *"some `ui-rect` in the trace carries a `viewport=`
tag"*. Nothing published one, and nothing could have:

* `egui_shell::dock::floatwin` publishes **no regions at all** — the shell
  has no diagnostic channel and R7 forbids giving it one.
* The Layers panel publishes exactly two regions, `panel.layers.search`
  and its clear button, and draws **neither** unless the document has two
  or more optional-content groups.
* **No fixture in `fixtures/` contains an `/OCProperties`.** On every one
  of them the panel's whole body is the single sentence *"this document
  has no layers"*, which publishes nothing.

⇒ So the check reported *"nothing is known to have been drawn in it"*
about a window that was drawing the only thing there was to draw.
**A measurement of the wrong surface looks exactly like a broken one** —
and this one would have gone on failing against every future build,
articulately, for ever.

**Fix, in the instrument rather than in the wording.**
`crate::app::surfaces::floating_panels` now publishes two regions from
inside the float body, where the panel's identity, its `Ui` and the
`ViewportScope` are all in scope at once:

| region | answers |
|---|---|
| `float.body.<panel>` | the shell gave the panel a compartment, and where — the float twin of `dock.body.<panel>` |
| `float.content.<panel>` | how much of it the panel **filled**, published after the body draws. A window whose panel allocated nothing publishes a zero-sized rect here |

and `egui_shell::dock::floatwin::FloatFrameReport::empty_bodies` counts
the same fact from the other side, surfacing as `empty=` on the
`float-windows` trace line. **Two independent witnesses**: one measured by
the application from the `Ui` it drew into, one by the dock from the `Ui`
it handed over. A regression that silenced one would have to silence the
other separately.

# The assertions, and why each is not vacuous

| assertion | what a green result would otherwise be compatible with |
|---|---|
| `layout-reset` fired | a section that silently inherited the previous section's layout — defect 1 |
| `panel-float moved=true` | nothing; this half always worked |
| a `viewport-inner` line | an embedded fallback window drawn inside the application's own — `dialog_windows`' reason for the same line |
| `float.body.<panel>` tagged with **that same viewport id** | a rect published by the docked copy, or by another window entirely |
| `float.content.<panel>` with a **positive area** | an open window with nothing in it, which is the defect named in the check's own `defect()` string and which every other line here reports as a success |
| `float-windows … empty=0` | the same, from the dock's own count |
| the window's `viewport-inner` region **retires** on dock-back and on close | a window that stayed open after the panel went home — the panel drawn twice, or a leaked entry |
| `panels-dock-all docked=1` | a recovery verb that answers cheerfully about nothing, having been handed nothing to recover |

The retirement assertions read `ui-rect-gone` and **name the id from
this run's own `viewport-inner` line** rather than a constant, because the
id is a hash of the panel id and a check that hard-coded it would keep
passing after a panel rename while asserting about a window that no longer
exists.

# Why no pointer is needed

The three verbs act on *the panel the operator right-clicked*, and a
harness has no pointer. `PDFCER_DIAG_INVOKE` therefore carries an
**operand**: `view.panel_float@view.panel_layers`. That widening is
documented at the seam in `crate::app::frame`, and it is the same finding
as the comma list before it — *a seam that can express less than the
interface can leaves part of the interface unverifiable*.

It means this check takes no cursor and is safe to run on a machine
somebody is using, exactly like `dialog_windows`.

# What is deliberately NOT covered, so a green run does not imply it

* **The tab's context menu.** The rows are `shown_when` a per-tab
  condition, and reaching them needs a right-click on a tab plus a click
  on a row — two pointer gestures. `menu_rows` is the check that would
  grow to cover it; until then the *menu* is verified by
  `shell::menus::tests::each_menu_holds_exactly_the_documented_items` and
  the *act* is verified here.
* **Dragging or resizing the window.** The position and size round trip is
  `dock::float::a_float_survives_serialization`, and moving a real window
  needs the window manager.
* **A monitor being unplugged.** Not drivable. The recovery is
  `view.dock_all_panels` and a layout reset, and section D drives the
  first of those.
* **What the panel's content SAYS.** This asserts that the body filled a
  rectangle, not that the rectangle holds the right words. On a fixture
  with no optional content the honest content of the Layers panel is one
  sentence, and one sentence is what R9 requires of a panel with nothing
  to list.
