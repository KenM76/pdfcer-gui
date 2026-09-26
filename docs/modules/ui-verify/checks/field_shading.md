# `ui-verify/checks/field_shading`

`fillable_fields_are_shaded_on_the_page` — every box you can type into wears
a wash, the way Acrobat's does.

# The operator's ask, `OPERATOR_REQUESTS.md` O96

> *"in our display section we should have an option to shade the form fields
> like acrobat does."*

A form's fields are frequently invisible on the page — a CAD title block with
no borders and no background looks identical whether it is fillable or
painted on. The wash is what says *"you can type here"* before the operator
has clicked anything.

# ★★ Why this needs a launched binary, when the drawing is four lines

Because the four lines are the *last* stage of a chain, and every earlier
link is somewhere else:

1. the preference has to default to on (`app::prefs`);
2. it has to survive into `OpenDoc::prefs`, which is a **snapshot taken when
   the document opened** rather than a live read — deliberately, because a
   preference changing mid-frame would flicker;
3. the canvas has to have built a widget census for the document at all;
4. the boxes have to be on a page that is currently in view.

A unit test of the drawing function can only reach step 4, with the other
three supplied by hand. Nothing below the binary can tell you that the
default is on *and* reaches the canvas *and* finds the fields.

# ★★★ The trace had to distinguish three states before this could exist

Drawing nothing is the observable outcome of three completely different
situations, and only one of them is a defect:

| trace | meaning | defect? |
|---|---|---|
| `on=0` | the operator turned the wash off | no |
| `on=1 boxes=0` | the wash is on and the document has no fields | no |
| `on=1 boxes>0 drawn=0` | the wash is on, the fields exist, none was painted | **yes** |

Before the trace carried `on` and `boxes` as well as `drawn`, a check could
not tell the third from the first two — and, far worse, a build with the
feature entirely dead would have looked identical to a correct build run
against a document with no form. That is the failure mode where a green
suite is actively misleading, so the instrument was widened rather than the
assertion weakened.

# No input, so this runs at any time

The preference defaults to on and the fixture opens on the page carrying the
fields, so nothing has to be clicked. With `PDFCER_DIAG_VIEWPORT` the window
lays out without taking focus. Like `title_build_stamp`, this can run beside
somebody working — which is worth preserving if the check is ever extended.
An extension that needs the pointer belongs in a second check, not bolted
onto this one.

# ★★★ `boxes=` is the CANVAS CENSUS, not the document's field count

Measured on `demo-form.pdf`, which carries **two** widgets: a text field and
a check box. The trace says `boxes=1`, and that is correct rather than a
defect — the text field has no `/AP` `/N`, so the page draws nothing there,
and the census deliberately excludes it (`NotOnCanvas::NoAppearance`). It is
**disclosed off-canvas** in the Forms panel, with the remedy named:
*"N field(s) are not drawn on the page, so they cannot be clicked there.
Fill one here and it becomes drawn."*

⇒ So nobody reading a future trace should take `boxes=` for *"how many
fields this form has"*. It is *"how many the canvas is in a position to
draw on"*, and the difference is exactly the undrawn set, which
`form-boxes … undrawn=` reports separately.

⬜ Whether the wash *ought* to cover an undrawn field is a real open
question, and this check deliberately does not decide it. The request said
*"like acrobat does"*, and Acrobat's field highlight is generally understood
to cover fields it would generate an appearance for — but that has **not
been verified against the installed Acrobat**, and asserting it from memory
is exactly the failure this project forbids. Recorded rather than acted on:
see `OPERATOR_REQUESTS.md` O96.

# What a passing run does NOT prove

That the wash is the right **colour**, or visible against the page. That is a
contrast question with a pixel oracle, and `crate::checks::legibility` is
where that kind of claim is made. This asserts the wash is drawn over the
right number of boxes, which is the part that silently disappears.
