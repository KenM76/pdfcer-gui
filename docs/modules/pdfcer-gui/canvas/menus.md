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
