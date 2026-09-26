# `ui-verify/checks/field_menu`

`right_clicking_a_form_field_opens_its_menu` — **the first driven context
menu in this project's history.**

# What this is for

`OPERATOR_REQUESTS.md` **O53**, the standing acceptance criterion:

> *"always always always I need objects on the canvas to be clickable and
> editable as one would expect given our research of other programs."*

Click, drag, grips and Delete all reached a form field by 2026-08-28. The
**right-click** did not: `canvas::menus` chose between an object menu and a
view menu, and a `/Widget` is neither — it is not in `SelectionState` at all
— so right-clicking a text box offered *"zoom to fit width"*.

## Why this check is the FIRST of its kind, which is the finding

**This harness had driven 92 checks and had never once opened a context
menu.** pdfcer has had canvas right-click menus since Phase 1. Everything
asserted about them is a unit test over `MenuHost::would_open`, which asks
whether the *manifest* would offer something — a real question, and not the
same question as *"does a right-click on this pixel open a menu"*.

⇒ There was no `Driver::right_click_at`. A gesture with no driver is a
gesture R1 cannot reach, and **the gap left no failing test behind to
advertise itself**. It surfaced only because a fourth menu was added and
somebody went looking for the driver to exercise it with.

The same shape as `DEFECTS.md`'s two headline bugs: invisible to a green
suite, obvious within thirty seconds of using the program.

## The oracle is `canvas-menu context=…`, and it is not a screenshot

An `egui` popup is positioned by the pointer and sized by its content, so a
harness that clicked *"the second row"* would be encoding a layout, and
would silently start choosing the wrong verb the day a menu grows an entry.

The application publishes which menu it resolved. That line is the fact
under test — *did a right-click on a field produce the FIELD menu* — with no
coordinate in it to go stale.

## What would be missed without the second assertion

`canvas-menu-invoked` is written only when the resolved menu **has something
to offer**, and for ten minutes this feature's menu had nothing: both its
items were gated on `selection.any`, which is **false** while a form field is
selected, so `offers_anything` was false and the popup never opened.

⇒ A check asserting only the context id would have **passed** on that build
— the context resolved correctly and no menu appeared. `DEFECTS.md` D1's
shape reached through a new door, and it is why both lines are asserted.

## The sequence

| # | step | oracle |
|---|---|---|
| A | place a text field (`edit.form_text_field`, self-accepting dialog) | `form-target` |
| B | Escape to disarm, clear on blank paper, click it | `form-field-selected none`, then `form-field-selected field=…` |
| C | **right-click it** | `canvas-menu context=canvas.field` |
| D | …and the menu had something in it | a `menu.item.canvas.field.*` region per row, inside `menu.body.canvas.field` |

**D's oracle was `canvas-menu-invoked` and that was a misreading**, kept
here because the misreading is instructive. `MenuHost::attach_with` returns
*"the commands the operator CHOSE"*, and the line is written only when that
vector is non-empty — so it reports an ACTIVATION, not an offer, and a check
that opens a menu and presses nothing can never see it. The rows' own
published rects are the offer, they name which commands were drawn, and they
exist for the same reason this check does: `MenuHost::attach_with` began
reporting them on 2026-08-28 precisely so a harness could see a menu.

Steps A and B are `widget_move`'s, identical in shape including the
Escape — see that file for why the placement tool staying armed is recorded
there as a defect rather than as scenery.
