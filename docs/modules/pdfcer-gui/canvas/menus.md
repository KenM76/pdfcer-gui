# `canvas::menus` — the page's right-click

Which menu opens is decided by **what the pointer was over** and by **what
is selected**, in the precedence [`CanvasMenu`]'s variants are written in:

| Situation | Menu | Because |
|---|---|---|
| a caret in existing page text | [`crate::shell::menus::CANVAS_TEXT`] | the operator is *in* the words |
| a form field | [`crate::shell::menus::CANVAS_FIELD`] | a widget sits on top of whatever is beneath it |
| a selected **markup shape** | [`crate::shell::menus::CANVAS_MARKUP`] | it has verbs — including its corners — that no other menu carries |
| an object, reading | [`crate::shell::menus::CANVAS_READ_OBJECT`] | every other object row edits, and this mode cannot |
| an object | [`crate::shell::menus::CANVAS_OBJECT`] | there is a thing to act *on*, so the menu is about it |
| blank page | [`crate::shell::menus::CANVAS_EMPTY`] | there is no thing, so the menu is about the *view* |

⚠ That table said **"Two menus"** and listed two until 2026-09-06, while
this file resolved five. It is the same decay the parent document's own
heading carried, found the same way — by adding the sixth and reading what
was already written. The count is now [`CanvasMenu`]'s variant list, which
`each_canvas_menu_names_a_context_the_shell_defines` walks.

`GUI_ROADMAP.md` Phase 1 is why this file exists at all:

> Right now, selecting an object produces a highlighted tree row and no
> way to act on it. Everything a user would try next — **right-click**,
> drag a handle, press Delete, change the colour — either does nothing
> or does not exist.

# A right-click over an unselected object selects it first

This is the behaviour every editor has and the reason is not
convenience. A context menu's implicit promise is *"these verbs apply to
the thing you pointed at"*. Without the select-first step the promise is
broken in the most damaging possible direction: point at object B while
object A is selected, choose **Delete**, and A is destroyed. The
operator's evidence for what was about to happen — the pointer — and the
application's operand list disagree, and the verb is irreversible in the
sense that matters (it costs an undo and a moment of not knowing what
just went).

So [`select_under_right_click`] runs **before** the menu is attached, on
the same frame, and the ordering is the enforcement.

Three rules, and each of the last two is a case the naive version gets
wrong:

1. **Over an unselected object** — replace the selection with it, at the
   Object rung. Exactly what a left click would do, through
   [`SelectionState::click`] itself rather than by assembling entries
   here, so the two gestures cannot disagree about what "select this
   object" means.
2. **Over an object that is already selected** — *change nothing*. A
   marquee over eight objects followed by a right-click on one of them
   must offer to delete all eight, which is the whole point of building
   the set. Selecting the one under the pointer would silently discard
   seven, and the menu would then be about something the operator did
   not ask for. This is also what preserves an *entered* rung: right
   -clicking inside the object you have descended into leaves you inside
   it.
3. **Over blank page** — *change nothing*, and in particular do **not
   clear**. A left click on paper deselects, and that is right: it is an
   unambiguous statement. A right-click is not — it is the opening of a
   question — and an operator who right-clicks slightly wide of their
   selection, sees a menu that is not the one they wanted, and presses
   Escape should still have their selection. Clearing here would make a
   mis-aimed right-click destroy work.

# Rule 4: a right-click marks nothing

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`, first
non-negotiable:

> **A pre-commit affordance is not content marking.** A snap indicator,
> a hover highlight, a rubber-band, a selection handle — these are the
> *cursor*; they describe what is about to happen and they are welcome.
> What is forbidden is styling content that has **already been applied**
> as though it were pending.

Nothing in this file paints. The only visible consequence of a
right-click is the **selection overlay** — which [`super::overlay`]
already draws for a left click, which is a pre-commit affordance by that
clause's own list, and which is drawn identically however the selection
arrived. The one-line test — *would a screenshot of the editing canvas
differ from a screenshot of the same document saved and reopened?* — is
answered by the selection overlay in exactly the way it already was, and
this file adds no second answer.

# Why the chosen menu is remembered rather than recomputed

`egui` opens a context-menu popup on the secondary click and then draws
it on **every subsequent frame** until it is dismissed. The pointer
moves during those frames — onto the menu itself, which is not over the
object any more — so recomputing the context id per frame would swap the
menu's contents out from under the operator's hand while they were
reading it. Worse, both contexts resolve to the *same* popup id
(`egui::Popup::default_response_id` is derived from the response, not
from the context), so the swap would happen in place, with no
reopen and nothing on screen to explain it.

The decision is therefore taken once, at the click, and stored in
`egui::Memory` for the life of the popup. `Memory` is the right home for
the same reason [`super::GESTURE_MEMORY_KEY`] is there and the selection
is not: this is frame-local interaction state — *which menu is open right
now* — with no meaning across a document, and `Memory` is per-`Context`,
so a document change starts the next frame with no popup in flight.

# What this file does not do

**It runs nothing.** [`attach`] returns `egui_shell::HandlerToken`s —
intent — which travel out through `canvas::show` to
`PdfcerApp::dispatch_token`, the same choke point the ribbon's Delete
goes through. That is what makes *"the context menu carries the same
commands again"* literally true: `format.delete` invoked from here and
`format.delete` invoked from the Format tab reach the identical arm, so
the rung guard in `SelectionState::deletable_objects_on` covers both and
cannot be stated twice.

## Item notes

### `const MENU_MEMORY_KEY`

See the module header: the choice is made at the click and read on every
frame the popup is drawn, so it has to outlive the click and must not
outlive the session. One `Id`, per `egui::Context`.

### `const MENU_SLOT`

Separate from `super::trace::SELECTION_SLOT`, which reports what the *selection*
did. The two answer different questions and de-duplicate on different
timescales — a right-click that lands on an already-selected object
changes no selection at all and would otherwise be invisible.

### `fn markup_menu`

Three conditions, and each of the last two is a case the one-line version
gets wrong.

1. **A markup annotation is selected.** [`AnnotKind::Markup`] and not a ce
   dimension — Rule 15. A ce dimension is also a `/Line`, its corners are
   [`crate::canvas::dimdrag`]'s, and its verb `move_dimension_vertex`
   **re-measures**. Routing one to this menu would offer *Add a point here*
   on a dimension, where the engine refuses by name
   (`EditError::AnnotationIsCeDimension`) and where the operator's next
   question would be why the measurement did not follow.
2. **The mode may author markup.** `author_markup`, not `edit_content`, so
   Review — whose whole subject is comments — gets the menu it exists for.
3. **The pointer is over the shape, or over nothing.** A markup selection
   plus a right-click on a path forty points away are about different
   things; taking the menu there would leave the operator with the shape's
   verbs over the object they had just pointed at. Over blank paper the
   shape wins, on `select_under_right_click`'s rule 3 reasoning — a
   right-click is the opening of a question, and an operator who aims a
   little wide of the shape they have selected meant the shape.

The containment test is on the annotation's own `/Rect` outline, in
**canvas** space, which is the space both that outline and
[`PageMapping::to_page`] speak. It is expanded by the mapping's own click
tolerance rather than by a number invented here, so *"near the shape"* means
the same distance a click means everywhere else on this canvas.

### `fn caret_in_existing_text`

The one question that separates [`CanvasMenu::Text`] from the other two, and
it is asked of the draft rather than of the tool: an *armed but unclicked*
text tool has no paragraph, and an operator who has armed it and then
right-clicked a rectangle wants the rectangle's menu.

### `fn a_right_click_over_an_unselected_object_selects_it`

The rule that makes the menu about the thing the operator pointed at.
Without it, right-clicking B while A is selected and choosing Delete
destroys A — the pointer and the operand list disagreeing, with an
irreversible verb between them.

### `fn a_right_click_inside_a_multi_selection_keeps_it`

The case the naive version gets wrong. A marquee over eight objects
followed by a right-click on one of them must still offer to delete
all eight; collapsing to the one under the pointer silently discards
seven and makes the menu about something nobody asked for.

### `fn a_right_click_on_blank_page_opens_the_view_menu_and_keeps_the_selection`

A left click on paper deselects, and that is right — it is an
unambiguous statement. A right-click is the opening of a question,
and an operator who aims slightly wide, gets the view menu and
presses Escape must still have their selection.

### `fn a_right_click_inside_an_entered_object_does_not_ascend`

A descended rung is expensive to reach — one measured CAD export
holds a whole drawing view as a single path object with 1,194
subpaths, so finding the subpath you meant took aim. Re-selecting the
whole object because the operator right-clicked it would throw that
away, and the ascent is the one thing Escape is *for*.

This falls out of rule 2 rather than being a special case, which is
why it is asserted: the object is already in
`object_indices_on`, so nothing runs.

### `fn a_right_click_on_a_different_object_leaves_the_entered_one`

The rule that stops an operator being stranded inside an object they
have forgotten they entered. It is `SelectionState::click`'s own
behaviour, reached rather than reimplemented, which is the point of
routing through it.

### `fn each_canvas_menu_names_a_context_the_shell_defines`

A `&'static str` in `Memory` could store a context id that no menu is
keyed by, which degrades into "right-clicking the canvas does
nothing" — the symptom this whole change removes. The enum is what
makes that unrepresentable; this is what proves the two arms point at
menus that exist.

### `fn a_right_click_on_a_selected_markup_opens_its_own_menu`

The whole point of the sixth context: the operator has a shape selected,
points at it, and gets the menu that carries its two node verbs.

Falsified by returning `false` from `markup_menu` unconditionally —
which is the state before this change, where the same right-click
resolved to `canvas.empty` and offered four zoom levels.

### `fn a_selected_ce_dimension_does_not_open_the_markup_menu`

Its corners are `canvas::dimdrag`'s and its verb re-measures. Offering
*Add a point here* on one would name an engine verb that refuses by name
(`AnnotationIsCeDimension`) and would leave the operator asking why the
measurement did not follow.

Falsified by dropping the `kind != Markup` clause: this test fails and
no other one does, which is exactly why it is written separately from
the one above rather than as a second assertion inside it.

### `fn a_mode_that_cannot_author_markup_gets_no_markup_menu`

Falsified by passing `!reading` (i.e. `edit_content`) instead: Review
has `edit_content == false` and `author_markup == true`, so the mode
whose entire subject is comments would lose the comment's own menu.

### `fn a_right_click_on_a_distant_object_is_about_the_object`

The pointer and the operand must agree — `select_under_right_click`'s
rule 1, arriving from the other side. Taking the markup menu here would
leave the operator holding a shape's verbs over the path they had just
pointed at.

Falsified by dropping the containment test and returning `true`
whenever a markup is selected.

### `fn a_right_click_on_paper_beside_a_selected_markup_keeps_its_menu`

`select_under_right_click`'s rule 3 reasoning: a right-click is the
opening of a question, and an operator who aims a little wide of the
shape they have selected meant the shape. It is also what stops the
commonest miss — a shape drawn thin, aimed at from just outside its box
— from silently becoming the zoom menu.

Falsified by removing the `object.is_none()` early return.
