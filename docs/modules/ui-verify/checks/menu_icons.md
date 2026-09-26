# `ui-verify/checks/menu_icons`

`menu_rows_draw_their_icons` — the regression check for the menu icon
painter that existed, was tested, and was never handed to a context menu.

# The defect

`egui_shell::menu::ContextMenu::with_icon_painter` had existed since the
menu engine landed. Nothing called it. `shell::menus::MenuHost::attach_with`
built its `ContextMenu` with `reporting_rects_to` and no painter, so **every
context-menu row in every build of this application drew a label and nothing
else** — including 25 rows whose commands already named a resolved,
catalogued, rasterizable icon key.

This is [`crate::checks::qat_icons`]'s defect on a second surface, and the
two are the same shape to the letter: an icon set that draws, a shell seam
that accepts it, and a call site where nobody wrote the argument. That
header's central sentence applies here word for word — *"a green suite is
evidence about the code that was written and not about the code that was
not"* — and it is why this file exists rather than another unit test.

It went further here than it did on the QAT, and the extra distance is the
reason this check is worth its launch. The missing wire was **written into
the project's record as a design decision**: an icon-coverage audit refused
a glyph for `view.panel_close` on the ground that *"the icon column exists
on the ribbon, not in a context menu"*, and that sentence was then quoted
until it read as the operator's own ruling. A missing line and a considered
refusal are indistinguishable from inside the source; they are
distinguishable from a running window.

# Why this check needs a name the QAT's check did not

`qat_controls_are_icon_only` asserts a **shape**: an icon-only button is
roughly square, a text button is a word wide, and the reserved rectangle
separates them without a screenshot.


⇒ So `egui_shell::menu::report::icon` was added with the fix, and it is
published **only from the branch that calls the application's painter**:

```text
ui-rect name=menu.item.dock.tab.view.panel_float rect=…   <- the row was drawn
ui-rect name=menu.icon.dock.tab.view.panel_float rect=…   <- ...and a painter was handed its slot
```

The second line is absent when the slot is blank, when there is no slot, and
when no painter was supplied — which are exactly the three states this check
exists to tell apart from a working one. That module's header carries the
argument in full.

It is deliberately **not** proof that pixels changed. Whether a key resolves
to art is the icon set's business and `crate::icons`' own tests assert it
offline; the fact that was missing was never *"does a glyph render"* but
*"did anything ask for one"*.

# What this drives, and why it asserts on whichever menu opens

One right-click on the page, and then an assertion about **the menu that
actually resolved**, named by the `canvas-menu context=…` line rather than
chosen in advance.

That indirection is not vagueness, it is what makes the check robust against
the document it is pointed at. A right-click on a sheet may land on an
object (`canvas.object`), inside a form field (`canvas.field`), in a text
draft (`canvas.text`) or on blank paper (`canvas.empty`), and which one
depends entirely on the fixture. **Every one of pdfcer's nine context menus
reserves an icon column** — `shell::menus_wiring::tests` asserts that from
the shipped documents and the shipped registry, 25 glyph rows against 2
blanks — so whichever menu opens must publish at least one painted slot.
Pinning a context here would make the check fail on a fixture change for a
reason that has nothing to do with icons.

The blank rows are the reason this is "at least one" rather than "one per
row". `view.zoom_actual` and `view.panel_close` are argued refusals, not
gaps, and a check that demanded a glyph per row would be demanding art the
project has decided against — the wrong-picture failure, arriving through a
harness.
