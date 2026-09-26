# `ui-verify/checks/markup_rectangle`

`markup_rectangle_arms_from_the_ribbon` — the regression test for a
**four-link chain in which every link has a passing unit test and nobody
had seen the button work**.

# The defect class this exists for

A chain in which every link has a passing unit test and no test observes a
join. `icons` proves the painter draws, with 52 tests. `egui-shell` proves
its seam accepts a painter. Both can be true and green while the
application hands neither to the other and the ribbon draws text buttons —
because the join is a property of a **call site**, and a call site's effect
is observable only in a running window.

Clicking `Markup ▸ Shapes ▸ Rectangle` is the same shape with four links
instead of one:

| # | Link | Where | Its own test |
|---|---|---|---|
| 1 | the ribbon click reports the command | `egui-shell`'s `band::render_command` | yes |
| 2 | dispatch routes the id to a markup kind | `app/dispatch.rs`, via `shell::commands::markup_for_command` | yes |
| 3 | the kind arms the canvas tool | `canvas::tool::arm_markup` | yes |
| 4 | the armed tool renders the control **pressed** | `app/conditions.rs` publishing `selected:markup.rectangle`, read by `band::render_command` | yes |

Four passing tests, four joins, and **no test anywhere observes two
adjacent links being connected**. Deleting the guard arm in step 2 breaks
the feature completely and breaks no test in the workspace.

# What this check depends on to aim a click

`egui_shell::ribbon::report::band_item`, called from
`band::render_command`, publishes a `ui-rect` per individual command
control. Without it the trace carries rectangles for group captions and
mode segments only, there is nothing to aim a click at, and therefore no
way to prove that clicking Rectangle does anything.

# What it does, through the operating system

Mouse only, because nothing below needs a key — not because a key could
not be sent. Synthetic keyboard input does reach the target window; see
[`crate::checks::add_text`], which types real characters into a caret draft
and asserts they landed. Every gesture below is a real `SetCursorPos` +
`mouse_event` click at a point derived from a rectangle **the application
itself declared on the frame it drew it**; see
[`crate::coords::WindowFrame::declared_center`].

1. Click the **Review** mode segment. The Markup tab is in Review's and
   Edit's tab lists and not in Read's, and Read is the default, so without
   this step there is no Markup tab to activate.
2. Click the **Markup** tab.
3. Capture the window — the *before* picture.
4. Click **Rectangle** in the Shapes group.
5. Capture the window again — the *after* picture.

# ★ The assertions, split by oracle, and why both are needed

## Trace evidence — that the arm happened

| Assertion | Line | What its absence means |
|---|---|---|
| the click reached the control | `ribbon-command-invoked id=markup.rectangle` | the click missed, or the control is disabled |
| the tool was armed | `markup-tool tool=Markup(Rectangle)` | **link 2 or 3** — see the chain above |

The second is the one that matters, and it is genuinely necessary: the
armed tool is otherwise invisible from outside the process. A crosshair is
a cursor, and a screenshot of an armed canvas and an unarmed one are the
same picture, so the only way to see the arm is to have the running
program print what it chose.

## Pixel evidence — that the control renders pressed

**And this is the half a trace line alone would not have caught.** A trace
line is written by the code under test, about itself. `arm_markup` traces
unconditionally the moment it is called, so `markup-tool` proves links 2
and 3 and says *nothing whatsoever* about link 4: a build whose ribbon
never renders a pressed state — because `conditions.rs` stopped publishing
`selected:markup.rectangle`, or because `render_command` stopped reading
it — emits an identical trace and looks identical to a reader of that
trace. That is defect 2's structure precisely, one layer up: the thing
works, and the surface the operator looks at does not say so.

So the pressed state is asserted from the **captured window**, three ways,
and the third is the one that cannot be faked:

| # | Comparison | What it rules out |
|---|---|---|
| P1 | Rectangle after ≠ Rectangle before | the control never changed |
| P2 | **Rectangle after ≠ Ellipse after**, in one capture | a *global* repaint — a theme change, a hover, a resize — masquerading as a pressed state |
| P3 | Ellipse after = Ellipse before | the whole band changing, i.e. P1 passing for a reason that has nothing to do with the click |

P2 is the load-bearing one. It is a differential inside a single frame, so
nothing that happens to *both* controls can satisfy it; only something
that happened to the one that was clicked. Together P1–P3 say: this
control, and only this control, changed, and it changed across this click.

The measured quantity is the **dominant colour bucket** of the control's
region ([`crate::pixels::contrast_at`]'s `background`), which is the
button's fill: in `egui` 0.35 a `Button::selected(true)` takes
`visuals.selection.bg_fill` for its frame instead of the widget state's
`weak_bg_fill` (`widget_style.rs`'s `button_style`, `SELECTED_CLASS`
branch). Contrast *ratio* is deliberately not the measure — under the
palette this was calibrated against the two fills were a light grey and a
light blue, about 1.3:1 apart, which a legibility threshold would call
identical. That channel now carries an opaque accent and the pair is far
apart, but the measure stays a channel difference so the check keeps
working under every palette this program can be given. See
[`MIN_PRESSED_DELTA`], which enumerates all three.

# Why the check does not simply assert on `selected:markup.rectangle`

Because the condition set is not published in the trace, and if it were,
asserting on it would be link 4's own unit test written a second time in a
slower harness. What is unverified is not whether the condition is
computed — that is tested — but whether computing it changes what the
operator sees. Only pixels answer that.

# Every way this reports SKIP, and why none of them is a pass

* no binary, no `--pdf`, `--no-input` — the harness never began;
* the diagnostic switches did not reach the process;
* the mode segment, the Markup tab, or the Shapes group's controls were
  never declared — each names the specific surface that is missing, and
  the `ribbon.item.*` case names `report::band_item` and its call site,
  because a build without Part 1 of this work is the one build where this
  check has nothing to aim at;
* a tool was already armed before the click — `arm_markup` **toggles** on
  the same kind, so a click on an already-armed Rectangle correctly
  *disarms* it, and a check that did not notice would report the feature
  broken;
* the two controls already looked different before the click, so a
  difference afterwards could not be attributed to it.
