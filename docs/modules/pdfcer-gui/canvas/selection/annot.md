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
