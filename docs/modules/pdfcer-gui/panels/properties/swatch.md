# `panels::properties::swatch` — one colour control, three honest states

`OPERATOR_REQUESTS.md` **O89**, both pieces. It is the widget behind the
clicked-text colour ([`super::textobject`]) and behind the multi-object
fill/line colour ([`super::paint`]), and it exists because those two
surfaces have to answer the same three questions the same way:

| state | drawn as | picking a colour |
|---|---|---|
| the selection agrees | a swatch of that colour | sets it |
| the selection **disagrees** | the indeterminate plate and a dash | sets **all** of them |
| the colour cannot be shown | *nothing — the caller draws a sentence* | — |

★ The third row is a caller's job on purpose. What to say about an ink
pdfcer will not overwrite differs between a path (whose `/Separation` name
the file records and `pdfcer_core::vector::PathPaint::Other` carries) and a
text run (whose `pdfcer_core::text_extract::TextColor::Other` carries **no
name at all**), and a widget that tried to word both would word one of them
wrongly. This widget refuses to draw a control; the sentence that stands in
its place belongs to whoever knows what the ink is.

## ★★★ Why "mixed" exists at all, and why it is not an invention

O89 recorded multi-object recolouring as *not offered*, with a reason:

> *"when the objects disagree there is no honest colour to open on and
> picking the first one's would quietly propose flattening the rest to it."*

That reasoning is right about the danger and wrong about the conclusion.
**Every editor in this product class already solved it** — Illustrator,
Inkscape, Figma and Word all show an *indeterminate* control over a
disagreeing selection, and applying a value sets every member. This
project's standing rule is that *the convergence of the product class IS the
specification, and an invented interaction is a defect even when it works*,
so the mixed state is the answer rather than one of several.

★ And the em dash is not a new marker either: this shell already writes one
for *no value*, in `crate::text::panels::properties::text_value_absent`,
whose own doc comment makes precisely this argument — *"every property grid
in this class shows a blank or a dash for no value and for mixed values,
which are the same state as far as a single field is concerned."*

## ★★★ ONE UNDO STEP PER GESTURE, and why this widget could not be
## `ui.color_edit_button_srgb`

This is the load-bearing reason the widget is hand-built.

`egui`'s own colour button marks its response **changed on every frame of a
drag inside the picker** (`color_edit_button_hsva` calls
`button_response.mark_changed()` from inside the popup body, on every frame
`color_picker_hsva_2d` returns `true`). A caller that acts on `.changed()`
therefore authors **one document edit per frame** while the operator drags
across the saturation square — sixty content-stream rewrites a second, sixty
undo entries, and a `Ctrl+Z` stack the operator cannot get back through.

★★ That is the same defect `super::text`'s size field already avoids by
committing on `drag_stopped`/`lost_focus` and never on `.changed()`, with
the same stated reason. A colour popup has no `drag_stopped` to hang it on,
so the equivalent event has to be *the popup closing* — which means owning
the popup id, which means owning the button. Hence this module.

⇒ [`show`] returns `Some` **exactly once**, on the frame the picker closes,
and only if the operator actually moved it. Open-and-close-without-touching
returns `None`, so idly inspecting a colour never writes to the document.

## Rule 4 — nothing here reaches the canvas

This widget draws in a dock panel. It marks no page, tints nothing, and
renders no preview: the document changes only when the caller's action is
applied, and from that instant the canvas shows exactly what the saved file
will show. What was skipped, and why, is disclosed off-canvas by the caller.

## Theme

The two colours the indeterminate state needs come from
`egui_shell::Theme::indeterminate_pair`, as a **pair**, for the reason that
function's own doc comment gives: `tools/gates/check-theme-colors.sh`
forbids invented values and cannot forbid a wrong role, and picking two
roles that happen to look right is how this project shipped defect D2 three
times. The swatch's own fill is not a theme colour at all — it is the
operator's document content — and says so on the line.
