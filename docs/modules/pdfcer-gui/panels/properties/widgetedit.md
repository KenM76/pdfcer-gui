# `panels::properties::widgetedit` — the **box** a form field is drawn in,
as opposed to the field itself


## Why this is a second file and not four more rows in [`super::fieldedit`]

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

## Moving is free and resizing is not. Both, and the difference shows

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

**`appearance_stale` is the one an operator will see and misread.** A
resize that could not rebuild the artwork — a push button's baked caption, a
signature — leaves the widget rendering **distorted**. The engine names it;
this pane prefixes the engine's own string with what it means on screen.

## All of `WidgetEdit`'s properties are here, and two of them took an hour

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

## `None` is a FACT, and this pane must never substitute a default

Both new fields are `Option`, and both `None`s are load-bearing:

* **`border: None` means the file states no border.** Not
  `BorderSpec::default()` — that default is solid/1 pt because it reproduces
  the bytes pdfcer *authors*, which is correct for a writer and a lie from a
  reader. Their load-bearing test is named
  `a_widget_whose_file_states_no_border_reads_a_dash_not_a_default`, and
  sabotaging the reader to return the default turns it red.
* **A border of width 0 is a VALUE**, not an absence — Table 166 states it
  as *no border*. It reads `0 pt`. Collapsing it to `None` would tell an
  operator the file is silent when it has said something definite.
* **`visibility: None` means the file's flags are ones pdfcer cannot set.**
  The mapping is exact-or-nearest-is-refused: `/F` admits dozens of
  combinations and `Visibility` is the four pdfcer can write, so a file
  carrying `Print | NoZoom` has no nearest of the four that is not a lie.
  `annot_flags` carries the raw word so the pane can say so.
* `None` there can never mean *absent*: Table 164 makes an absent `/F`
  equal `0`, which **is** one of the four. So the sentence is always about a
  file that said something inexpressible.

## Rule 4

Nothing here marks the canvas. A moved box renders exactly where the saved
file will render it, and every disclosure — resized, stale artwork, siblings
untouched — lands in the status bar.

## Item notes

### `const SPEED`

A quarter of a point, matching `super::geometry`'s `SPEED`, and the
reason is the same: these are **drafting** numbers on a drawing sheet, where
a whole point of drift is visible. An operator who wants a big move types
the number.

### `fn rotation_row`

# THE DIRECTION IS THE WHOLE DANGER, AND IT IS NEGATED HERE

`/MK /R` is **counterclockwise**. The page's `/Rotate` is **clockwise**. The
engine flagged this as *"the single most likely thing for a shell to get
backwards"*, and the standard makes it easy: the two entries are word for
word parallel —

| | |
|---|---|
| `/MK /R` (Table 189) | *"…rotated **counterclockwise** relative to the page…"* |
| page `/Rotate` (Table 30) | *"…rotated **clockwise** when displayed or printed…"* |

**The direction word is the only difference between those two sentences.**
Worse, the *movie* dictionary's `/Rotate` uses the identical phrase
*"relative to the page"* with the **opposite** sense, so that phrase carries
no convention at all — only the direction word does.

⇒ So the two controls here are labelled **left** and **right**, which is
what an operator means, and the negation happens **here, at the UI layer**,
exactly as the engine instructed: *"if your rotate control has a clockwise
affordance, negate at the UI layer and pass counterclockwise degrees to us.
Do not negate inside anything that touches `/MK`."*

A **right** turn is what the operator sees the box do. That is `-90`
counterclockwise, and this is the only place in the program where those two
facts meet.

# Why ±90 buttons and not a typed angle

The engine refuses anything that is not a multiple of 90 — Table 189 says
*"shall be a multiple of 90"* — so a free number is a control most of whose
values are refusals. Two buttons offer only what can succeed, which is R9's
posture rather than a simplification.

# Why the current angle is shown even at zero

Because `Widget::rotation` is `Option<i64>` and `None` means **the file
states none**, which is not the same fact as `Some(0)` — the distinction
`Widget::border`'s own docs call *"a fact to display, not a value to
substitute"*. An operator debugging why a box looks wrong in another viewer
wants to know which of the two their file says.

### `fn geometry_rows`

# Why an Apply button and not commit-on-release

[`super::fieldedit`]'s max-length spinner commits on release, and this one
deliberately does not — the difference is that **these four are one edit**.
A box is moved by changing X *and* Y; committing each on release would
author two `edit_widget` calls, two undo entries, and an intermediate state
in which the box has moved sideways and not down. `super::geometry` reached
the same conclusion for the same reason and this follows it, including the
button's placement.

The button is **greyed when nothing was typed**, which is R9's temporarily
unavailable case: there is a capability and no operand, and the hover says
so.

### `fn border_rows`

# It reads from the DOCUMENT and shows a dash when the file is silent

There is no draft, deliberately, and the style combo reads
`widget.border` fresh every frame — the same argument
[`super::fieldedit::flag_row`] makes for its checkboxes: a press the engine
refuses leaves the control where it was, because the document did not
change. A draft-backed control would show the operator's intent while the
document silently disagreed.

**`None` renders [`t::border_unstated`] and offers no width at all.** The
alternative — a combo pre-set to Solid and a spinner at 1 — is exactly the
invention this whole exchange with the engine was about, and the first press
would write it into the operator's file. Choosing a style from the combo is
how a widget with no stated border gets one, which is an act rather than a
default.

A width of **0** is a value, not an absence, and shows as `0 pt`.

### `fn visibility_row`

**`None` is a sentence, not an empty combo.** The engine's mapping is
exact-or-refused, so `None` means the file carries flags pdfcer cannot set —
`Print | NoZoom`, say — and it can never mean *absent*, because Table 164
makes an absent `/F` equal `0` which is one of the four.

So the pane says which flags, in hex, and says pdfcer is leaving them alone.
The alternative — showing the nearest of the four — is the border defect
wearing a different hat, and the operator's first press would collapse a
combination the file meant.

### `fn caption_row`

**Not cosmetic on a push button**, which is why the engine models this
one key out of `/MK` and none of the other ten. A push button has no `/V` at
all (§12.7.4.2.2), so the caption is the only thing distinguishing *Submit*
from *Reset* to anyone reading the field list.

Empty commits `Some("")`, which **removes** it. That is the engine's
spelling and it is unambiguous, unlike the tooltip's three-state choice —
there is no "leave it alone" to express here, because not touching the
control is how you leave it alone.

### `fn chrome_rows`

# This row was REFUSED until the engine painted the colours

`/MK` `/BG` and `/BC` were read and written perfectly for months and
**painted by nothing**: R43 makes pdfcer draw the baked `/AP` and never
reconstruct an appearance from `/MK`, so writing the key and stopping is
*"a record of an intention nothing acts on"* in the engine's own words. A
swatch over that would have been the defect rule 4 names in one line —
this shell tints its on-canvas field editor from `/BG`, so the operator
would have picked a colour, watched the box take it, saved, reopened, and
found it grey. A screenshot of the editing canvas differing from a
screenshot of the same file saved and reopened is the whole test.

`Pass 308.0` bakes both colours into all four appearance builders and
`edit_widget`'s `needs_regen` now covers a colour-only edit, so the canvas
and the saved page agree. The refusal rested on exactly that and is
withdrawn.

# The two keys are not symmetric, and the panel must not pretend they are

| | absent | empty array | what the panel offers |
|---|---|---|---|
| `/BG` | the kind's own default — nothing, or a push button's plate | **paints nothing at all** | a *No background* entry |
| `/BC` | black | **black** | no such entry |

`WidgetChrome::stroke` resolves the empty array and the absent key to the
same black, deliberately: a border's *thickness* lives in `/BS` `/W`, and
treating an empty `/BC` as "omit the stroke" would give two unrelated keys
one meaning. So an entry writing an empty `/BC` would change a byte,
rebuild an appearance stream, cost an undo entry, and alter no pixel — R9's
case for rendering nothing rather than a control that does nothing.

⚠ O202's decision 1 assumed the opposite (*"an empty `/BC` positively means
draw no border"*). It was written against the engine as it stood before
`Pass 308.0`. This follows the measurement.

# Rule 4

Nothing here marks the canvas. The colour is applied and from that instant
the page shows what the saved file will show; a *recorded, not painted*
outcome is disclosed in the status line by `actions::forms::edit_widget`
and never drawn.

### `struct Setters`

A pair rather than two parameters because they are one fact — *which key
this row owns* — and a call site that got one of them from the background
row and the other from the border row would compile.

### `fn chrome_row`

[`Setters`] carries the `WidgetEdit` verbs for this key. Passing them
rather than a discriminant keeps the two rows from ever writing each
other's key — a
`match` on "which row am I" is a thing a maintainer can get backwards and a
function pointer is not.

### `fn sync`

Takes the values rather than a `&Widget`, for the reason
[`super::fieldedit::FieldPropsDraft::sync`] does: `forms::Widget` has no
`Default`, so a unit test cannot build one without a document, and this
function reads exactly five things off it.

### `fn differs`

An epsilon rather than `!=`, because the spinners round to two
decimals for display and a `/Rect` read out of a file routinely carries
more. Without it the Apply button would be live the moment the pane
opened, on every widget whose box is not exactly hundredths — which is
most of them, and which reads as the program thinking the operator has
unsaved changes they never made.

### `fn resizes`

The engine makes the same comparison and its answer is authoritative;
this one exists only so the Apply button's hover can say which act the
operator is about to perform, **before** they perform it.

### `fn a_draft_follows_the_widget_and_not_just_the_field`

The failure this stamp's middle term exists for, and it is invisible on
every one-widget field: a radio group is one field with several boxes,
so a draft keyed on the name alone would carry the first button's
geometry onto the second, and pressing Apply would move a box the
operator was not looking at.

### `fn apply_is_dead_until_a_number_actually_moves`

The second half is the one worth testing. Without the epsilon the
button would be live the moment the pane opened on any widget whose box
is not exactly hundredths — which reads as unsaved changes the operator
never made, on most real documents.
