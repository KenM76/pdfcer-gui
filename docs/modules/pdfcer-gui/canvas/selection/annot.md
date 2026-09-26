# `canvas::selection::annot` — clicking the things pdfcer itself put on the page

## The gap this closes, and how long it was open

`FEATURES.md` recorded it on 2026-08-17, under the Format contextual tab:

> *"**The canvas selection cannot address an annotation** — `Selection` is
> `page + object + subpath + node`, four integers naming a paint-order
> index into page *content*, which is what makes it immune to zoom and also
> means a markup or dimension **is not selectable at all**. The second is
> ours; the first is filed."*

Both halves of that are now discharged. The engine's half — no verb that
modifies an annotation — cleared on 2026-08-18 with `set_markup_style`.
This is ours.

The operator's report is what it cost: *"How do I edit a stamp I've
applied?"*, *"I still can't get to edit dimension groups when I click on
it"*, and *"it feels like nothing is moving forward on these things"* —
three symptoms of one missing capability. A stamp placed in the wrong spot
could not be moved, restyled, or even **deleted**, except by `Ctrl+Z`
immediately afterwards.

## Why this is a sibling of [`super::Selection`] and not a variant of it

They look similar and are structurally different in four ways, every one of
which would have to be special-cased if they shared a type:

| | page content | annotation |
|---|---|---|
| identity | a **paint-order index** — position in `PageObjects::objects` | an **`ObjId`**, stable across edits and across saves |
| arity | multi-select, built up over several clicks | one at a time |
| structure | a ladder — object ▸ subpath ▸ node, because one CAD path can hold 1,194 subpaths | flat; an annotation has no parts in pdfcer's model |
| geometry | needs `decompose_page`, which resolves and walks every content stream | `/Rect`, read straight off the dictionary |

The last row is why annotation selection needs no cache and no
`resolved_for` epoch key: the rectangle is four numbers in the annotation
dictionary, and asking for it costs a dictionary lookup rather than a
content-stream walk.

**They are still mutually exclusive**, and [`super::SelectionState`]
enforces that in one place rather than by convention — see its `annot`
field. One canvas, one selection; `panels::ObjectTreeUi::focus`' refusal of
*"a second selection"* stands.

## Why the KIND is in the type

[`AnnotKind`] distinguishes a **ce dimension** from ordinary markup, and it
is carried on the target rather than re-derived where it is needed. That is
not tidiness — it is the shell's half of a refusal the engine makes by
name.

A ce dimension is a `/Line` annotation with `/IT /LineDimension`. It passes
every *"is this markup pdfcer can author?"* test, and restyling one through
`set_markup_style` would regenerate its appearance as a **bare line, with
its label and witness lines gone** — from an operator who asked only to
recolour it. `pdfcer-core` refuses it by name
(`EditError::AnnotationIsCeDimension`) and points at `set_dimension_style`,
and the reply that shipped the verb said so in as many words: *"Your Format
tab must route ce dimensions there."*

Carrying the kind on the target makes that routing a `match` the compiler
checks, rather than a condition somebody has to remember at each of the
places a style is applied. The engine's refusal stays as the backstop; this
is what stops it being reached.

## Rule 4: this draws nothing on the page that a save would not

A selection outline is **the cursor**, which the rule permits by name — the
same class as a snap indicator, a rubber band or a resize handle, and the
same treatment content selection already gets. Nothing here tints, badges
or flags an annotation, and the one-line test still passes: a screenshot of
the canvas with a stamp selected differs from a screenshot of the saved
file only by the marching outline, which is where the pointer is and not
what the document says.

## Item notes

### `fn distance_to_segment`

The standard projection-and-clamp. Clamping is what makes it a *segment*
rather than an infinite line — without it, a click level with a dimension
line but far off its end would still hit, which is exactly the
claims-too-much failure this whole path exists to remove.

A degenerate segment (`a == b`) falls out correctly: the projection divides
by a zero length, which is guarded, and the answer becomes the distance to
the point. A zero-length segment is a real thing here — a perimeter does not
de-duplicate its vertices — so this is a case rather than a defence.

### `fn the_last_painted_annotation_takes_the_click`

`/Annots` is paint order, so a stamp dropped over a rectangle is drawn
last and is what the operator sees. A hit test that took the first
match would select the thing underneath — which looks like the click
missing entirely, because the outline appears somewhere the operator
was not pointing.

### `fn a_click_in_a_dimensions_empty_space_does_not_select_it`

> *"selecting space not actually occupied by the lines or text of the
> dimension still selects it if I am selecting within the box area it
> occupies — I can't select objects underneath it."*

An L of two thin segments across a 100×100 box. A click in the middle of
that box is nowhere near either segment, and must miss — which is what
lets the click reach the drawing underneath. Under the old
`rect.contains` test it hit.

### `fn a_segment_does_not_claim_the_line_it_lies_on`

A click level with the horizontal arm but well past its end must miss.
Without the clamp in `distance_to_segment` it would hit, and a dimension
would claim a stripe across the whole sheet.

### `fn a_click_outside_every_annotation_is_not_a_hit`

Stated because the alternative — nearest-match — is a plausible
implementation that would make it impossible to *deselect* by clicking
away, which is the gesture every operator tries first.

### `fn a_ce_dimension_stays_a_ce_dimension`

The one property that routes a later restyle to `set_dimension_style`
rather than `set_markup_style`. If it were dropped here and re-derived
downstream, the re-derivation would be the thing that could be
forgotten — and forgetting it turns a recolour into a dimension that
loses its label.

### `fn an_annotation_the_canvas_does_not_draw_cannot_be_clicked`

# The defect

`selectable_on` filtered on `flags.hidden()` — `/F` bit 2 alone — while
the renderer and the note pop-up both ask
`AnnotFlags::suppressed_on_screen()`, which is `hidden() || no_view()`
(§12.5.3, Table 165). So a **`/NoView`** annotation was **selectable
with nothing drawn under the pointer**: click blank paper and an outline
appears, with handles, around a mark the operator cannot see and did not
know was there.

Found by the note-pop-up work, which noticed the two layers disagreeing
about which annotations exist on screen, and reported rather than fixed
because this file belonged to another track that afternoon.

# Why no existing test could have caught it

**Two predicates over the same flags, each self-consistent.** Every
test of the selection layer used the selection layer's own notion of
visible, and every test of the painter used the painter's. A
disagreement between two correct halves is invisible to any test of
either half — which is why this one asserts them **against each other**
rather than against a constant, and why the fix calls the engine's
predicate instead of re-spelling `hidden() || no_view()` here.

# What it deliberately does NOT assert

⚠ That a `/NoView` annotation is unreachable. It is not, and must not be
— it still prints, the Comments panel still lists it, and the page's
notes still count it. R50: *"a page carrying content the operator cannot
see is a fact they are entitled to know."* The claim is narrower and
exact: **it is not clickable on a canvas that is not drawing it.**

### `enum AnnotKind`

Two variants, and the distinction is load-bearing rather than descriptive —
see the module header. Deliberately not `is_ce_dimension: bool` on a struct:
a bool is a fact a caller may forget to read, while a variant is one the
compiler makes them handle.

### `fn selectable_on`

# What is excluded, and why each one

The same four exclusions [`crate::panels::comments`] makes, for the same
reasons, plus one this surface needs that the panel does not:

| excluded | why |
|---|---|
| `/Widget` | the form field surface owns it — a click there focuses an editor, and two owners of one press is how a field becomes unfillable |
| `/Popup` | §12.5.6.14 is a `shall`: a pop-up *"shall not appear alone but is associated with a markup annotation"*. It is a reader-UI window, not content |
| `/Link`, `/Movie`, `/PrinterMark`, `/TrapNet` | not authored by the operator and not restylable. `/TrapNet` in particular is prepress output state |
| **hidden** (§12.5.3 bit 2) | **this surface's own**, and it is not shared with the panel |

The hidden case is the one worth stating. The Comments panel *lists* a
hidden annotation, deliberately — it is on the page and the operator has a
right to know. The canvas must not **select** one, because nothing is drawn
there: a click on blank paper would produce a selection outline around
nothing, and a Delete would remove something the operator cannot see. The
panel is where a hidden annotation is reached, which is exactly the split
the forms surface already makes for an undrawn field.

# Ordering

`/Annots` order, which is paint order — later entries draw on top. The
caller takes the **last** match, so the topmost annotation wins a click,
which is the rule page content already follows.

# Cost

One `/Annots` walk and one dictionary read per entry, bounded by
`pdfcer_core::annot::MAX_ANNOTS_PER_PAGE`. No decomposition, no content
stream, no cache — see the module header's table.

### `fn hit`

`point` is **canvas space**, the same space `selectable_on` returns and the
same space the content hit test works in.

# A RECTANGLE IS NOT ALWAYS THE SHAPE, and assuming it was cost the
# operator the ability to select anything under a dimension

This function tested `rect.contains(point)` and nothing else. The reasoning
below about tolerance was careful and correct — and it never asked the prior
question, which is *is the rectangle the thing?*

For a stamp, a highlight or a sticky note, yes: the `/Rect` **is** the mark.
For a **ce dimension** it is emphatically not. A dimension is two thin
witness lines, a dimension line, two arrowheads and a small label — and its
`/Rect` is the box around all of that, which for anything but a perfectly
horizontal dimension is mostly empty air. A perimeter traced round a
building is worse still: its rectangle covers the entire footprint and its
ink is the outline.

So clicking inside that box selected the dimension, and the operator could
not reach the drawing underneath:

> *"selecting space not actually occupied by the lines or text of the
> dimension still selects it if I am selecting within the box area it
> occupies — I can't select objects underneath it. Where did you learn that
> behaviour? It's not in any program I've seen."*

He is right, and the convention is universal: **a click selects what is
under the cursor, not what merely encompasses it.** An unfilled shape's
interior belongs to whatever is behind it — every drawing program, every CAD
package, every vector editor. A bounding box is what a MARQUEE tests
against, and a marquee is a different gesture.

A candidate may therefore carry a precise `shape` — its drawn segments in
canvas space — and where it does, that is what is tested. Where it does not,
the rectangle stands, because for those kinds it is the truth.

# Tolerance: none for a rect, and necessarily some for a segment

The engine's argument for testing `/Rect` bare:

> *"`bounds_of` applies the pen half-width at **authoring** time, so the
> stored `/Rect` already contains it. A shell hit-testing `/Rect` is
> already correct today."*

The rectangle is the geometry **plus** the margin a tolerance would add, so
adding a second would make two adjacent markups claim each other's clicks.

A **segment** is the opposite case: it is a mathematical line with no width
at all, and without a tolerance nothing could ever be clicked. So the shape
path takes one, and it is the same click tolerance the content hit test uses
— which is what makes a dimension line as easy to hit as the drawing line
beside it, rather than easier or harder.

# Topmost wins

The **last** match in `/Annots` order, which is the last one painted. A
stamp dropped on top of a rectangle is the thing the operator sees and
therefore the thing they mean.

### `fn under_pointer`

# Why this is a function and not four lines at the call site

Because answering it takes four collaborators — the annotation list, the set
of which ones are ce dimensions, those dimensions' drawn ink, and the click
tolerance — and every one of them has to agree with what is on screen. Four
lines inline in `canvas::interact` is four lines that can each be got subtly
wrong somewhere else, and the "somewhere else" is what this project keeps
paying for.

It also puts the whole hit-testing story in one file with [`hit`]'s
argument, which is where a reader will look for it.

# The tolerance comes from the MAPPING, so it is zoom-invariant

[`crate::canvas::mapping::PageMapping::tolerance`] is the same click
tolerance the content hit test uses. That is deliberate: a dimension line
must be exactly as easy to hit as the drawing line beside it, and a
separately chosen number here would drift from it the first time either was
tuned.
