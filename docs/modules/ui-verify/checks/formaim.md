# `ui-verify/checks/formaim`

`checks::formaim` — **where a check must click so that a form-field
selection CHANGES**, and the census parsing that finds it.

# The finding this module exists to encode

Three driven checks — `form_field`, `widget_move` and `field_menu` — all
begin the same way: arm `edit.form_text_field`, click the page, and get a
field. Then all three clicked that field's centre and asserted the
application traced

```text
pdfcer-diag form-field-selected page=0 field=Text1 widget=0
```

On 2026-08-29 all three failed on that assertion, with the same sentence:
*"THE FIELD COULD NOT BE SELECTED"*. **The click was landing exactly where
the field is.** From `widget-move.trace.txt`:

```text
form-target page=0 field=Text1 widget=0 rect=(473.8,529.4)+(160.0,20.0)
canvas-pointer screen=(460.0,429.0) page=(555.08,539.21) …
```

The rect spans x ∈ [473.8, 633.8] and y ∈ [529.4, 549.4]; the click resolved
to page (555.08, 539.21), which is 1.3 pt from its centre in x and 0.2 pt in
y. Nothing missed.

What actually happened is one frame earlier in the same trace:

```text
add-form-field page=0 n=1 epoch=1 …
ui-rect name=canvas.selection-outline rect=[[436.0 426.1] - [483.3 432.0]]
```

47.3 × 5.9 px at zoom 0.2955 is 160 × 20 canvas units — **the new field,
already drawn selected**. `app::actions::forms`' authoring arm sets
`doc.selected_field` to what it just placed (`OPERATOR_REQUESTS.md` **O53**:
*"every program in this class leaves a newly drawn object selected"*), and
`canvas::forms::select_click` raises `FieldAction::Select` — and writes its
trace line — **only on a change**:

```text
if picked == doc.selected_field { return; }
```

⇒ So the click was correct, the program was correct, and the *check* was
asking a question with no answer: it clicked a field that was already
selected and then required the program to announce a selection that had not
moved. Neither a stale coordinate mapping nor a broken hit test — the page
did not move between the two clicks (`paint=` is identical on every
`canvas-pos` line of `form_field.trace.txt`, from the placement through the
selection), and the widget was hit dead centre.

# The repair, and why it makes the checks say MORE than they did

A check that wants to observe *"clicking a widget selects it"* has to make
the selection different first. The program documents exactly one gesture
that does so, in `canvas::forms::select_click`'s own table:

| | primary | secondary |
|---|---|---|
| over a field | select it | select it |
| over the selected field | no change | no change |
| **over blank paper** | **clear** | change nothing |

So: click blank paper (the application traces `form-field-selected none`),
**assert that clearing line arrived**, then click the field and assert the
naming line. Two observations where there was one, and the first is what
makes the second admissible — `crate::checks`' rule 4, that an absence is
evidence only once the thing that would have produced a presence is shown
working. A build that stopped tracing selection at all now fails at the
clearing step, naming the trace channel rather than the hit test.

# Why "blank paper" is computed rather than named

The three callers place their field at different points on different
documents — `form_field` at the sweep's `--doc-point`, the other two at page
fractions — and the sweep's document (`SW41177.pdf`, 36 sheets of CAD) may
carry widgets of its own. A constant offset would eventually land on one,
and the check would then fail reporting a selection that in fact changed
from one field to another. [`blank_canvas_point`] therefore consults the
application's **own** census of where every selectable widget is, tries a
short ring of candidates around the target, and returns the first that is
clear of all of them and comfortably inside the sheet.

Everything here is in **canvas space** — y increasing downward from the top
of the sheet — because that is the space `form-target` publishes. Callers
flip to PDF space with `page.height_pt - y` before handing a point to
`CanvasMapping::doc_to_window`, exactly as they did before this module
existed.
