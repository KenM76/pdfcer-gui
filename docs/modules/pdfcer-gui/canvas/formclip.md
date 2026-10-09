# `canvas::formclip` — copy and cut parts of a placed drawing

When the selection holds parts of a placed drawing (form leaves) and no page
object and no annotation, Copy and Cut call
`EditSession::copy_objects_in_form(page, leaves)` instead of
`copy_selection`. Both return an `ObjectClip`, so the result is stored as the
same `Clipped::Selection` and pasted by the same path as a page-object copy.

## Where the parts land

The engine bakes the drawing's placement into each copied item, so a paste
with the identity puts the part on the page as page content, where and as
large as it was drawn. The pasted part is no longer inside the drawing.

## Mutability

The engine verb takes `&mut EditSession` because it reads the cached page
model; it commits nothing and makes no undo entry. The shell reaches the
session with `Arc::get_mut`, as the edit funnel does. When something else
holds the session the copy is refused (`clipboard-copy-refused
reason=session-borrowed`) rather than waiting.

## Cut

Copy first, then `VectorAction::DeleteLeavesInForm` — one undo entry for the
deletion. A refused copy deletes nothing. The engine has no cut twin.

## Mixed selections

A selection holding page objects and parts copies the page objects through
`canvas::clipboard::copy`, and the status note counts the parts left behind
(`in_form_left`). Dragging a selection to another window still goes through
the page copy, so parts of a placed drawing are refused there.

## Trace

`clipboard-copy kind=form-leaves page= leaves= items= bytes=`, written only
when the engine returned a clip.
