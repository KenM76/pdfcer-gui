# `canvas::rightclick` — which menu a secondary click opens


## The frame-ordering hazard, which is the whole reason this is subtle

**`egui` opens a popup ON the secondary click.** There is no later frame on
which a wrong answer could be corrected — the menu that appears is the menu
decided by this frame's evaluation, and if that evaluation reads state which
the click itself is about to change, the operator sees the *previous*
answer, permanently.

Two pieces of state are one frame behind here, for two different reasons:

| what | why it is stale | how it is answered |
|---|---|---|
| the object selection | a right-click over an unselected object **selects it**, and the ribbon's condition snapshot was taken at the top of the frame | [`crate::shell::menus::MenuHost::with_conditions`] corrects the conditions |
| `doc.selected_field` | `canvas::forms::selecting::select_click` raises `FieldAction::Select`; the queue applies it at the **end** of the frame | a **hit test**, not a state read — see [`Click::field_menu`] |

⇒ Both are the same defect shape and both had to be found the same way: by
asking *"what does this read, and who writes it, and when?"* Neither is
visible from a unit test, because a unit test constructs the state directly
rather than letting a click produce it.

## The menus, in the order they are asked

| # | condition | menu | why it outranks the next |
|---|---|---|---|
| 1 | a caret is in existing page text | `canvas.text` | the operator is *in* the words; a text run is not a hit-testable object, so deciding by hit test first would give them the view menu |
| 2 | a form field is under the pointer or selected | `canvas.field` | a widget sits on top of whatever page content is beneath it |
| 3 | a **markup shape is selected**, and the pointer is on it or on paper | `canvas.markup` | an annotation is not in the content model, so no hit test below could ever find it — and its two node verbs exist on no other surface |
| 4 | this mode reads rather than edits, over an object | `canvas.read-object` | O71: every row of the object menu edits |
| 5 | an object is under the pointer | `canvas.object` | there is a thing to act *on* |
| 6 | otherwise | `canvas.empty` | no thing, so the menu is about the *view* |

⚠ Rows 3 and 4 were missing from this table while both were live — the
heading said *"The four menus"* and the file resolved five. Recorded rather
than silently corrected: it is the third copy of one count in this
subsystem, and the answer adopted with the sixth menu is that the count
lives in `canvas::menus::CanvasMenu`'s variant list and nowhere else.

1 and 2 are mutually exclusive by construction — `canvas::forms` owns
`/Widget` presses and only Edit mode offers field selection, while a caret
belongs to the text tool — so their order is documentation of that fact
rather than a precedence anybody has to enforce.

## Called on EVERY frame, not only on the frame of the click

`egui` draws an open popup until it is dismissed, and the popup exists only
while something is attached to the response. On a frame with no secondary
click and nothing open this does nothing at all.
