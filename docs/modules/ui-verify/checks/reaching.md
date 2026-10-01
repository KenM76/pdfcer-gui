# `ui-verify/checks/reaching`

`checks::reaching` — **putting a control where the pointer can hit it.**

# Why this is its own file


The seam is a real subject rather than an arbitrary cut, and the tell is
that all three functions here answer the **same question** —

> *the application says this control is at that rectangle. Can the pointer
> actually reach it there, and if not, what has to happen first?*

— where the rest of `driving` answers *"where is it"*, *"what frame is it
in"*, and *"press it"*. This project has spent real sessions on the gap
between those two questions, and every incident is written up on the
function that closed it:

| function | the failure it exists to prevent |
|---|---|
| [`scroll_to`] | a control below a pane's fold, never declared, reported as missing |
| [`raise_dock_tab`] | a docked pane that is **not in front publishes nothing**, which is indistinguishable from a panel with nothing to say |
| [`bring_into_body`] | a control declared through the ungated `diag::ui_rect`, laid out **past the bottom of its panel**, clicked at a centre that is outside it |

The three are stated as one family because getting the wrong one produces
the same class of report every time: **a confident, articulate failure about
a mechanism the trace can rule out.** [`bring_into_body`]'s header carries
the worked example — a Paste button nineteen points inside its panel, and a
failure message naming `paste_outline_item`, which had never been asked.

They are re-exported from [`super::driving`], so `driving::scroll_to` and
the rest keep working at every existing call site. The file is what R2
limits; the module path a check reads is a separate question and moving it
would have been churn with no reader served.

## Item notes

### `fn raise_dock_tab`

`RESUME.md` records the finding in its own words: *"A docked pane that is
not in front publishes nothing, which is indistinguishable from a panel with
nothing to say."* The dock draws only the **active** tab's body, and
`dock.tab.<id>` is published for every tab whether or not it is active — so
a check that reads a panel's output and does not raise it first is reading
silence and reporting it as a defect. That has already cost this project one
misdiagnosis (a Properties pane behind another tab, reported as a panel that
failed to describe the selection) and it was fixed **at one call site**.

It was fixed at one call site twice more the same day — `bookmark_add`
carries its own copy, with a comment saying it *"made the first version of
this check pass with the defect planted back in"*. Three checks in the same
family did not have it. So it lives here now, and a new check that reads a
panel gets the correct behaviour by calling one function rather than by
having read someone else's comment.

# What it does

Three routes, tried in order, and the order is the safety argument.

| # | region | what it does |
|---|---|---|
| 1 | `dock.tab.<id>` | clicks the tab. The original behaviour. |
| 2 | `dock.body.<id>` | the panel is already drawn and active — presses NOTHING and returns `true`. |
| 3 | `rail.tabs.<id>` | presses the left-rail entry, then checks it did not close the panel. |


Ken asked that day for *"no tabs in the left side bar when the left rail
is visible"*, and [`egui_shell::dock::Dock::tabs_suppressed`] delivers it:
when a rail was drawn and can raise **every** panel in the stack, the dock
draws no tab strip at all. In this application all three of its conditions
hold in every mode, so **`dock.tab.*` has not been published since.**

This function went on returning `Ok(false)` — correctly, by its own
contract — and three of its five callers **discard the bool**. They then
failed later with messages naming bookmark mechanisms that were never at
fault. Three checks skipped for a week and the suite stayed green, because
a SKIP is not red.

⇒ The rule earned, and it generalises past this file: **a helper that can
decline must not hand back a `no` a caller is free to ignore.** The
durable form of that is for the helper to stop declining while a route
exists, which is what route 3 is. Fixing the three checks instead would
have left the fourth and fifth callers — and every future one — holding
the same loaded bool.

# Why route 2 must be tried BEFORE route 3


`dock.body.<id>` is published only while the panel is the drawn, active
tab of a visible side, so it answers the toggle's question directly.

# What `false` still means

The contract is unchanged and it is still not an error: **this panel has
no route on screen in this mode.** A caller's own precondition, which
knows what it needs, is the right place to judge that. What changed is
that `false` no longer *also* means "the dock happens to be drawing no
tabs today", which is what made it uninformative.

# Errors

If the trace cannot be read, the window frame cannot be resolved, or the
pointer cannot be driven.

### `fn bring_into_body`

[`scroll_to`] stops as soon as the region is **declared**. That is the right
test for a control published through `diag::ui_rect_visible`, which is
silent while the control is off screen. It is the wrong test for one
published through the plain `diag::ui_rect`, which reports the rectangle
egui laid the widget out at whether or not any of it is on screen.

`a_bookmark_subtree_can_be_copied_and_pasted` failed on exactly that. The
Bookmarks panel's body was `[[52 181.7] - [274 766.0]]`; its **Paste**
button — which only exists once something has been copied, so it appears at
the very bottom of a list that has just grown — was declared at
`[[52 761.0] - [158.8 785.0]]`. Nineteen points of it were inside the panel
and the rest, including its centre, was not. The harness clicked the centre,
hit whatever lay under the dock, and reported:

> *the Paste button was pressed and no `bookmark-paste-applied` line
> followed, so the action was raised and never applied — or
> `paste_outline_item` refused*

which names two application mechanisms and is about neither. **This is the
project's recorded worst outcome — a confident, articulate failure about the
wrong subject** — and it is the third recurrence of the same root: *a rect
proves layout, not visibility.*

⇒ The durable fix is here rather than in the check, for the same reason
[`raise_dock_tab`] is: the next check to read a control near the bottom of a
scrolling panel should get the right behaviour from a function call.

It is **not** a substitute for the application publishing the gated form.
That remains the better fix and it is product code: `panels::bookmarks::clip`
calls `crate::diag::ui_rect`, where `panels::layers` and the rotation row
call `ui_rect_visible`. Reported, not changed, from here.

# What it does

Reads `body` and `wanted`. While `body` does not wholly contain `wanted`, it
rolls the wheel one notch down over the body's centre and re-reads. Returns
the rectangle once it fits, or `None` if `wanted` is never declared, or the
last rectangle seen once `attempts` are spent — with a note saying so, so a
caller that then fails is failing with the reason on the page.

# Errors

If the trace cannot be read, the frame cannot be resolved, or the wheel
cannot be driven.

### `fn scroll_to`

# Why this is a helper and not two copies of a loop

It was two copies for about ten minutes, and the second copy is what forced
the extraction: the field-scoped controls sit below the fold of the
Properties slot, and the widget-scoped controls sit below *those*. A check
that scrolled once found the first and reported the second missing — which
is the failure this function's existence prevents, and it is worth naming
because the message it produced was confident and wrong (*"the section is
not being called"*, about a section that was in the same trace).

**It scrolls at the DOCK PANE, not at the content**, and that took three
wrong anchors to arrive at.

A wheel event has to land inside the scroll area, so the anchor's centre has
to be **on screen**. Three candidates were tried and each failed differently:

| anchor | why it failed |
|---|---|
| `properties.widget_edit` (a section's `min_rect`) | published ungated, so it exists even when the section is entirely off screen. The wheel went outside the window |
| `properties.form_field` published as `max_rect` | that is the space the `Ui` was ALLOWED, not the space it took — it named a rect over the **Objects** panel, and six notches scrolled the object list |
| `properties.form_field` published as `min_rect` | correct about where the section is, and the section is 741 pt tall in a 180 pt slot, so its **centre is below the window** |

⇒ The generalisation: **content rects are not scroll anchors.** Any region
belonging to scrolled content can have its centre outside the viewport, by
definition, because that is what scrolling means. What is always visible is
the **pane**, and `egui_shell::dock` publishes it as
`dock.body.<panel command id>`.

`D:/dev/rag/egui/` carries the family this belongs to: harness coordinates
go stale when a layout changes, and a wheel aimed at a remembered position
scrolls whatever is there now.

Returns `None` when the region never appears — the caller decides whether
that is a failure or a skip, because only the caller knows what it means.

# `attempts` is the caller's, and it is not a tuning knob

It is *how far the caller is willing to say it looked*, and it belongs in
the caller because the failure message does. A properties pane is a few
notches deep; a settings dialog with seven collapsed groups is more. A
constant here would make every caller's "I looked and it was not there"
mean a different distance without saying so.

## Which pointer

`raise_dock_tab`, `open_footer`, `bring_into_body` and
`driving::click_mode_segment` take `&impl input::Click`, which both the OS
`Driver` and the scripted pointer implement (`click_rect`, `scroll_rect`), so a
check moves to the off-screen pointer without a second copy of each helper.

## `open_footer`

Opens a panel's collapsed footer (`panels::footer`) so the controls inside it
publish their regions. It reads the footer's own `panel-footer id=… open=…`
line and clicks the header only when that says closed, because the header is
a toggle and a second click would hide what the caller came for. Returns
whether the footer is open afterwards; `false` when no footer was traced (the
panel is not showing, or the mode draws none).
