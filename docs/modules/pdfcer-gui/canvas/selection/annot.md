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
