# `panels::properties::widgetedit` — the **box** a form field is drawn in,
as opposed to the field itself


## ★★★ Why this is a second file and not four more rows in [`super::fieldedit`]

Because the engine has two verbs, and it has two verbs because Acrobat's own
scripting model has two scopes. Taken verbatim from the design brief: some
properties *"apply to all widgets that are children of that field"*, others
*"are specific to individual widgets"*.

| scope | verb | properties |
|---|---|---|
| **field** — one write, every placement | `edit_field` | required, read-only, tooltip, and the type flags |
| **widget** — per placement | `edit_widget` | rect, border, visibility, caption |

> **Getting this backwards is invisible on the ordinary one-widget field and
> wrong on every radio group** — where "the border" can only sensibly mean
> one button and "required" can only sensibly mean the group.

A single file holding both would make that distinction a comment. Two files
make it the module boundary, and the pane draws them as two headed sections
so an operator meets it as well.

## ★★★ Moving is free and resizing is not. Both, and the difference shows

§12.5.5 derives a widget's appearance matrix from the appearance box's
corners and the `/Rect` corners. A **pure translation** makes that matrix a
pure translation, so the baked artwork moves with the box, exactly and for
nothing — which is why `move_widget` regenerates no appearance and is right
not to.

A changed **extent** puts the same algorithm to work as a *scale*. A text
field dragged twice as wide would render its text twice as wide rather than
gaining room for more text. So `edit_widget` compares the **extent, not the
corners**, and rebuilds only when it changed. `WidgetEditOutcome::resized`
reports which happened, and the pane says so, because *"the box moved"* and
*"the box was resized and its contents were redrawn"* are different things
to have done to a file.

★★ **`appearance_stale` is the one an operator will see and misread.** A
resize that could not rebuild the artwork — a push button's baked caption, a
signature — leaves the widget rendering **distorted**. The engine names it;
this pane prefixes the engine's own string with what it means on screen.

## ★★★ All of `WidgetEdit`'s properties are here, and two of them took an hour

⚠ **This heading said *"All four"* until 2026-09-11, by which time
`WidgetEdit` carried seven** — `rect`, `resize`, `border`, `border_color`,
`background`, `caption`, `visibility`. A completeness claim that names a
NUMBER goes stale the moment the other side grows a field, and it goes
stale **silently**: nothing fails to compile, no gate counts it, and the
sentence keeps reading like an audited fact. Phrase such a claim against
the type, never against a count, so that the only way to falsify it is to
look at the type.

This section read *"`WidgetEdit` carries four properties and this pane
offers **two**"* for about an hour on 2026-08-27, and the reason is worth
keeping because the outcome is what decision 058 promises and rarely gets to
demonstrate.

**border** (`/BS`) and **visibility** (`/F`) were writable and **not
readable**: `annot_author::read_border_width` was private, `border_style` is
a *writer*, and `forms::Widget` modelled no border at all. So the controls
were **absent rather than offered**, and the reason was not effort:

> A properties control has to show the current value. One seeded from a
> default would display *Solid 1 pt* over a widget whose file says *Dashed
> 3 pt* and write the invention back on the first press.

That was filed rather than worked around, and `Pass 146.0` shipped
`Widget::border`, `Widget::visibility` and `Widget::annot_flags` within the
hour. The engine checked every claim in the request against their tree
before scoping it, and quoted the sentence above into the field's own doc
comment, into `docs/core-api/`, and into their test file header — *"because
the next person to touch this will be tempted to simplify it."*

## ★★★ `None` is a FACT, and this pane must never substitute a default

Both new fields are `Option`, and both `None`s are load-bearing:

* **`border: None` means the file states no border.** Not
  `BorderSpec::default()` — that default is solid/1 pt because it reproduces
  the bytes pdfcer *authors*, which is correct for a writer and a lie from a
  reader. Their load-bearing test is named
  `a_widget_whose_file_states_no_border_reads_a_dash_not_a_default`, and
  sabotaging the reader to return the default turns it red.
* ★ **A border of width 0 is a VALUE**, not an absence — Table 166 states it
  as *no border*. It reads `0 pt`. Collapsing it to `None` would tell an
  operator the file is silent when it has said something definite.
* **`visibility: None` means the file's flags are ones pdfcer cannot set.**
  The mapping is exact-or-nearest-is-refused: `/F` admits dozens of
  combinations and `Visibility` is the four pdfcer can write, so a file
  carrying `Print | NoZoom` has no nearest of the four that is not a lie.
  `annot_flags` carries the raw word so the pane can say so.
* ★★ `None` there can never mean *absent*: Table 164 makes an absent `/F`
  equal `0`, which **is** one of the four. So the sentence is always about a
  file that said something inexpressible.

## Rule 4

Nothing here marks the canvas. A moved box renders exactly where the saved
file will render it, and every disclosure — resized, stale artwork, siblings
untouched — lands in the status bar.
