# `canvas::take` — lift the selection without touching a clipboard

`take::selected(&mut doc)` lifts what is selected into an `ObjectClip` and
returns it as a `Taken`. Copy stores the clip on the clipboard, Cut also
deletes what was taken, and Duplicate pastes it back at the paste offset.
All three read the selection through this one function, so the three cannot
disagree about what "the selection" is.

## Which verb lifts it

- **Only parts of one placed drawing selected** (`formclip::leaves_only`):
  `EditSession::copy_objects_in_form(page, leaves)`, `Kind::FormLeaves`. The
  engine bakes the drawing's placement into each item, so the clip pastes as
  page content where and as large as it was drawn.
- **Otherwise**: `EditSession::copy_selection(page, objects, annots)`,
  `Kind::Selection`, with every selected annotation (`annotclip::selected`).

`copy_selection` commits nothing, which is what makes a cut one undo entry:
only its delete half is an edit.

## The page

The annotations' page wins over `view.page_index`. The view can be scrolled
onto another sheet with a comment still selected, and `copy_selection`
indexes into the page it is given. The annotation selection holds one page by
its own invariant.

## Refusals

| Refusal | When |
|---|---|
| `NothingSelected` | no page object, annotation or drawing part |
| `InsideForm(n)` | only drawing parts, beside nothing `leaves_only` accepts |
| `Unreadable` | a selected annotation is no longer on its page |
| `CannotCarry(what)` | every selected annotation was refused and no object was selected |
| `EngineRefused` | the engine verb failed, or another holder has the session |

`copy_objects_in_form` takes `&mut EditSession` because it reads the cached
page model. The session is reached with `Arc::get_mut`; when something else
holds it the lift is refused rather than waiting.

## `Taken`

`kind`, `clip`, `page`, `objects` (page objects or parts selected), `plan`
(the engine's carrier decision per annotation), `annot_ids` (the annotations
lifted, in `/Annots` order — what a cut deletes) and `in_form_left` (parts
selected beside page objects, which `copy_selection` cannot take).
`Taken::clipped()` is the clipboard's `Clipped::Selection` record of it.

## Trace

`selection-take-refused reason=` on a refusal that has a cause worth naming
(`inside-form n=`, `cannot-carry what=`, `session-borrowed`, `engine detail=`).
The callers trace their own success line.
