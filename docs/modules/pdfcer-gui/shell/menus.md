# shell::menus — pdfcer's context menus, as data

[`built_in`] returns every context menu pdfcer defines, as an
`egui_shell::menu::Menus` value carried on the same
[`egui_shell::Shell`] the ribbon is. [`MenuHost`] is the one place a
right-click site turns that data into drawn rows and invoked commands.

This module is the third of the three surfaces `RIBBON_IA.md` §5.8
names, and it is the one that was missing entirely:

| Surface | Answers | Lives | Built |
|---|---|---|---|
| the contextual **Format** tab | *"what do I change mid-gesture?"* | in the ribbon, on selection | `manifest::format` |
| the **properties panel** | *"what *is* this thing?"* | in the dock, always | `panels::properties` |
| the **context menu** | *"act on **this**, now"* | at the pointer | **here** |

`RIBBON_IA.md` §6 says what its absence cost:

> **Context menus** — currently zero in the entire crate
> (`grep context_menu` → no hits). Every selection type above needs one,
> carrying the same commands as its Format tab section plus
> Cut/Copy/Paste/Delete. This is not a ribbon question, but it is **the
> other half of making selection meaningful**, and no amount of ribbon
> design substitutes for it.

and §5.8 says why a third surface is not duplication:

> A third surface, the **context menu**, carries the same commands again
> for the user who right-clicks. That is not duplication in the P1 sense
> — **context menus are not tabs** — and it is the path most users try
> after the keyboard.

That sentence is why this module holds no vocabulary of its own. A menu
item is [`egui_shell::manifest::Item`] — the *same* type a ribbon band
holds — so a command id resolves through the *same* registry into the
*same* [`egui_shell::HandlerToken`], dispatched at the *same* choke
point. "The same commands again" is literally true, and the
confirmation gate, the undo entry and the refusal that `format.delete`
is subject to are written once and cover both.

# The rule that decided what is in each menu: only real commands

`RIBBON_IA.md` P3, **no placeholders**, applied here exactly as
[`super::manifest`] applies it to the ribbon. §6 asks for
*"Cut/Copy/Paste/Delete"* on the selection menu. **Cut, copy and paste
do not exist**: there is no object clipboard in this build, which
[`super::manifest::PLANNED`] records against `edit.cut`, `edit.copy`,
`edit.paste` and `edit.paste_in_place`. So they are **absent from
`canvas.object`**, not present and greyed.

The distinction matters and the menu engine implements both halves:

| Situation | What the operator sees | Why |
|---|---|---|
| the command **is registered** and its predicate is false | the row, **greyed**, with its tooltip | it exists; it is not applicable *right now* |
| the command **is not registered** in this build | nothing at all | it does not exist, and a greyed row for a command that can never be enabled is a promise the build cannot keep |

An unregistered id would in fact still be *dropped* by
`egui_shell::menu::plan::resolve` and disclosed on the verify channel —
but relying on that would be shipping a document that names commands
this build does not have and calling the omission a feature.
[`tests::every_command_every_menu_names_is_registered`] is what stops
that: the check `egui_shell::Shell::validate_against` does **not**
perform, because `command_references()` walks tabs, the QAT and the
keymap and deliberately not the menus (a command in a menu is not a
reachability claim — see [`super::tests`]).

# And the rule that decided the shape: a menu with nothing to offer
never opens

Right-clicking something that has nothing to offer must do **nothing** —
not flash an empty box, and not open a menu of greyed rows. The engine
takes that decision *before* it asks `egui` for a popup
(`egui_shell::menu::plan::offers_anything`), and [`MenuHost::attach`]
is the only path this application uses, so every pdfcer right-click
inherits it. [`MenuHost::would_open`] answers the same question without
drawing, for a caller that wants to know.

It is not a theoretical case here. `objects.row` offers `file.properties`,
which is gated on `doc.open`; with no document the Objects panel draws no
rows at all, but a build that compiled `file.properties` out would leave
that menu empty, and the right-click would then correctly do nothing
rather than opening a box with a rule in it.

# The menus, and why each holds what it holds

⚠ This heading read *"The four menus"* until 2026-09-06 and the table under
it listed four while [`built_in`] returned eight. That is this project's
recurring shape — **a prose count beside the thing it counts, decaying while
a test pins the truth one screen down** — and it is why the heading no
longer carries a number at all. [`CONTEXTS`] is the count, and
`tests::the_catalog_defines_exactly_the_documented_contexts` is what makes
it true.

| Context id | Right-click site | Items | The reasoning |
|---|---|---|---|
| [`CANVAS_OBJECT`] | a selected object on the page | `view.zoom_selection`, `format.properties`, `format.select_text_line`, `format.select_form`, `format.unshare_form`, `format.merge_text_runs`, `edit.redact_selection`, `format.delete` | The Items column was **wrong** until 2026-08-28 — it had never been updated for `format.select_form`, added the previous day, which is this project's recurring shape of a prose claim beside the thing it describes decaying while a test pins the truth one screen down. The two form commands arrived with the form-XObject work: `format.select_form` because a click now reaches *inside* a form and the container has to be reachable on purpose, and `format.unshare_form` because O53 forbids a command existing only on the ribbon — and because the operator who needs it is mid-gesture, about to type into a title block, and the pointer is where they are looking. Zoom to selection is here because **SolidWorks and Acrobat both reach it by right-click** and only Inkscape binds a key for it — operator instruction of 2026-08-14 to match those three; see the registration site for why no chord was invented. Then §5.8 lists Delete in **every** selection type's row. It is the one command in that section that exists (see `manifest::DIRECTED`), and it is wired: `PdfcerApp::dispatch_token` reads `SelectionState::deletable_objects_on`, the same rule the Delete key reads. **`format.properties` joined them on 2026-08-18**, with the ce-dimension properties section: a selected ce dimension's group, measurement, style overrides and radius/diameter switch are otherwise reachable only by noticing that a contextual tab appeared or by opening a dock panel by name, and the operator's report was *"I click and can't figure out how to enable some of the basic stuff."* It sits above Delete because the destructive row is last in every menu here. **`format.select_text_line` joined on 2026-09-14** (O188(A)) and is the only row here that is absent rather than greyed when it does not apply: it descends to ONE LINE of a multi-line text object, a rung that until then was reachable only by arming the Points tool with a chord nobody had been told about. It is a re-aim, not an edit, which is why `DESIGNS.md` §6.2's ban on a context-menu item does not reach it — see the registration site. |
| [`CANVAS_MARKUP`] | a selected markup shape on the page | `format.properties`, `markup.add_node`, `markup.remove_node`, `edit.cut`, `edit.copy`, `edit.paste`, `format.delete` | **The sixth canvas context, 2026-09-06, and the reason it is not [`CANVAS_OBJECT`] is that four of that menu's five rows are meaningless on an annotation.** `format.select_form` and `format.unshare_form` are about page content inside a form XObject; a markup annotation is not page content and is never inside one, so both would resolve, draw and do nothing — the *live and silently inert* class this project's `DEFECTS.md` is made of. What replaces them is the pair the operator asked for by name: *"I also can't edit or delete nodes of a markup shape once it is drawn."* See the block comment at the registration for the order, and [`crate::canvas::annotnodes::menu`] for why one of them can be greyed and the other absent on the very same shape. |
| [`CANVAS_EMPTY`] | blank page, or the paper beside the drawing | `view.zoom_fit_page`, `view.zoom_fit_width`, `view.zoom_fit_height`, `view.zoom_actual` | The four **named** zoom levels, all of which have a live dispatch arm today. A right-click on paper is about the *view*, because there is no object to be about. |
| [`DOCK_TAB`] | a panel tab in the dock | `view.reset_layout` | The only registered command that acts on the dock. The **command** is wired (`PdfcerApp::dispatch_command` calls `Modes::reset` with `ResetScope::All`); the **menu** still cannot be attached — see the warning below. |
| [`OBJECTS_ROW`] | a row in the Objects panel | `file.properties` | The Properties panel is *where an object row is described*; right-clicking a row focuses it and this is the command that puts the description on screen — which it now does: `PdfcerApp::show_panel` activates the panel, mounting it first if the operator's arrangement no longer holds it. |

## `dock.tab` — the note below described a gap that CLOSED

⚠ **The heading and the paragraph under it were true until the
tab-menu seam landed, and are kept only for the record.**
`egui_shell::dock::Dock::with_tab_menu` exists,
`crate::app::surfaces::docks` supplies a handler, and this menu is
attached on every drawn panel tab — with its conditions corrected per
tab, so `view.panel_float` and `view.panel_dock` are never both offered.
It is additionally attached to a **floating** panel's header strip by
`crate::app::surfaces::floating_panels`, which is why one menu
definition serves two surfaces.

The original note, unedited:

## `dock.tab` is defined and **cannot be attached from this crate**

`egui-shell`'s dock draws its own tabs and already owns their secondary
click: `crates/egui-shell/src/dock/tabs.rs` calls
`response.context_menu(…)` with a hard-coded **Close** button, and
`egui_shell::dock::Dock` exposes no seam — no `with_tab_menu`, no tab
`Response` handed back — through which an application could attach one
of its own. Two context menus on one `Response` would fight over the
same popup id.

The menu is defined here anyway, and that is a deliberate choice rather
than an oversight left lying about:

- it is **data**, and a document that describes pdfcer's context menus
  with one of the four missing would be wrong about the application even
  while it happened to match the wiring;
- the operator-customization layer merges against it, so an operator can
  already state what they want on a panel tab;
- it costs nothing at run time — a menu nobody looks up is a `Vec` entry.

What it needs to come alive is a change in `egui-shell`, not here: the
dock must either take a context id for its tabs and call
`egui_shell::menu::Menu::attach` itself, or hand the tab's `Response`
out. `Close` is not a pdfcer command and is not registered
(see [`super::manifest::PLANNED`]'s `dock.close_panel` entry), so the
hard-coded button is not a duplication of anything in this file.

# Where the strings are

Nowhere in this module, and nowhere in [`crate::text::menus`] either.
Every row's label and tooltip is the **command's**, from
[`crate::text::commands`], because a context menu carries the same
commands again and a second copy of "Delete" is a second copy that can
drift. [`crate::text::menus`]' header carries the full argument and the
list of what *would* land there.

The context ids below (`"canvas.object"`, `"dock.tab"`, …) are never
displayed. They are lookup keys the application chooses and the shell
never interprets — the same kind of string a command id is.

## Item notes

### `const CANVAS_READ_OBJECT`

Two rows: take a copy, and look closer. Its own context rather than a
filtered [`CANVAS_OBJECT`], because every other row of that menu edits and
R9 says a mode that cannot edit renders nothing rather than a greyed list.
`canvas::menus::CanvasMenu::ReadObject` carries the argument.

### `const CANVAS_TEXT`

⇒ **Without this, reflow would be reachable only from the ribbon**, and the
standing rule is that anything the engine can do to a thing on the page is
reachable by clicking that thing on the page. A paragraph's "click" is the
caret; its right-click is this.

### `const CANVAS_FIELD`

The fourth canvas menu, added 2026-08-28. Keyed on `doc.selected_field`,
which is neither a `SelectionState` entry nor a caret — a `/Widget` is
deliberately not an annotation selection — so none of the other three ever
resolved for one, and a right-click on a text box offered *"zoom to fit
width"*.

⇒ `OPERATOR_REQUESTS.md` **O53**: *"always always always I need objects on
the canvas to be clickable and editable as one would expect."* A context
menu is the fourth of the five gestures that sentence covers, after click,
drag and Delete.

### `const CANVAS_MARKUP`

# Why not just widen [`CANVAS_OBJECT`]

Because four of that menu's five rows are about **page content**, and an
annotation is not page content:

| that menu's row | on a markup shape |
|---|---|
| `format.select_form` | meaningless — an annotation is never inside a form XObject |
| `format.unshare_form` | meaningless, same reason |
| `view.zoom_selection` | works, and is kept |
| `format.properties` | works, and is the route to the Properties panel's markup section |
| `format.delete` | works, and stays last |

Two rows that resolve, draw and do nothing is the *live and silently inert*
class `DEFECTS.md` is made of, and R9's answer to *"this cannot apply"* is
nothing rather than greying — the shape will not become page content while
the operator looks at it.

⇒ So a context of its own, carrying what a placed markup can actually
answer for: what it is, its two node verbs, the clipboard, and Delete.

### `const NODE_INSERT_OFFERED`

Set per right-click by [`crate::canvas::menus`], never by
`PdfcerApp::conditions`, for [`PANEL_DOCKED`]'s reason one step further
along: it is a fact about *one click on one edge*, and the frame's condition
set describes the frame. [`crate::canvas::annotnodes::menu::rows`] is what
answers it, and it answers it by **asking the engine**, so this name means
*the engine did not refuse this on grounds of the shape's kind*.

### `const NODE_REMOVABLE`

This is the one condition in the pair that is genuinely *temporary*: a
closed shape keeps three points and an open one keeps two, and drawing
another corner makes the row live again. That is precisely why the row is
greyed rather than hidden, and why the command's tooltip states the floor.

### `const RUN_SELECT_OFFERED`

O188(A). The Part rung on text — the rung at which *one line* is the
operand, and therefore the only rung at which `delete_text_run` can be
reached — was reachable by exactly one gesture: arm the Points tool by
chord, *then* click. No surface named it. This condition is what lets a row
name it instead. [`crate::canvas::runmenu`] carries the measurement of
every other gesture and why each lands one rung up.

Set per right-click by [`crate::canvas::menus`], never by
`PdfcerApp::conditions`, for [`NODE_INSERT_OFFERED`]'s reason: it is a fact
about *one click on one line*, and the frame's condition set describes the
frame.

**One name, carried on both the item and the command**, where the node
pair above needs two each. That is not an inconsistency — it is R9 applied
to a question with no recoverable state. A node can be un-removable *for
now* (the shape is at its vertex floor; draw another corner and the row
lives again), so that row is drawn and greyed and explains itself. There is
nothing the operator can do, while this menu is open, that turns a
single-run text object into a multi-run one or moves the pointer onto a
line it is not on. So the row is **offered or absent**, and shown implies
pressable — which also means a stale frame cannot press a dead row.

### `const PANEL_DOCKED`

Set per drawn tab by `crate::app::surfaces`, never by
`PdfcerApp::conditions` — because it is a fact about *one tab*, and the
frame's condition set describes the frame. `MenuHost::with_conditions`
is the sanctioned way to correct a condition to a value the caller has
just computed, and its docs carry the argument for why that is not a
second source of truth.

### `const PAGES_ROW`

Spelled here **and** in `crate::panels::pages::PAGES_ROW`, which is the one
duplication this module tolerates and only because the panel attaches the
menu before this file could hand it a constant: the two are asserted equal
by that panel's own test, so a rename that touches one fails rather than
silently detaching every tile's menu.

### `const DOCUMENT_TAB`

The two are deliberately different contexts because they name different
things: [`DOCK_TAB`] names a *panel*, which is a tool, and this names a
*document*, which is an operand. Sharing one menu between them would offer
*Reset layout* on a drawing and *Close others* on the Bookmarks panel.

### `const CONTEXTS`

Hand-written, and pinned by
[`tests::the_catalog_defines_exactly_the_documented_contexts`] against
[`built_in`] itself, so a menu added to the document without an entry
here — or an entry here with no menu — fails rather than silently
halving a test sweep.

### `fn built_in`

Deterministic and side-effect free, exactly as
[`super::manifest::built_in`] is, and called from the same place: once,
at start-up, and from the tests. It is the **built-in layer** of
`SHELL_FRAMEWORK.md` §4's three-layer merge, so it has to be complete
and has to validate — it is what every other layer patches and what an
operator gets back when they reset.

# Order is presentation

Items appear in the order they are written. Within a menu that order is
argued at each site; between menus it does not matter, because a lookup
is by key.

### `struct MenuHost`

Carries the three things every `egui_shell::menu::Menu::attach` call
needs — the document to look the context up in, the registry to resolve
its ids against, and the conditions to evaluate their predicates
against — so a call site names only *which* menu and *what* it was
attached to.

# Why a borrowing struct rather than three arguments

Because it is passed through two layers that have nothing to do with
menus. `canvas::show` and `panels::Panel::show` hand it on to the
functions that actually right-click, and threading three parameters
through each of them would make every one of those signatures a place
the three could be mismatched — a registry from one frame with the
conditions from another, say, which produces a menu that is *plausible*
and wrong.

# Why it is `Option` at every call site

[`crate::app::PdfcerApp::shell`] is `Option<Shell>`: if the built-in
manifest ever fails to validate, the ribbon does not render and the
application deliberately stays usable for reading. A build in that state
has no menus either, and `None` is the honest way to say so — not a
stand-in for "menus are not wired yet".

### `fn label`

# Why a panel is given this rather than a string of its own

`crate::panels::tool` names the armed tool, and the only honest name for
it is **the name on the control that armed it**. A second string would
compile, would read identically on the day it was written, and would
drift the first time either was reworded — and the drift is invisible,
because nothing renders both at once.

`NO_SURFACE.md` §1 records exactly that failure with a colour rather
than a label: a duplicate of a value that already existed, plus a test
that *"asserted the literal triple against a function returning the
literal triple. Two copies of one constant cannot disagree."* Reading
the registry makes the second copy unrepresentable instead of merely
unlikely.

Returns `None` for an id this build does not register, which is a real
state rather than a defensive one: `SHELL_FRAMEWORK.md` §5b's whole
point is that a capability compiled out loses its command, and a caller
must render **nothing** for it rather than a name with no control behind
it.

### `fn chord`

From the manifest's keymap, inverted, exactly as a menu row's accelerator
hint is — `egui_shell::menu::shortcut::Shortcuts::of`. So an operator who
rebinds a key sees every surface follow, with nothing to keep in step.

A panel that hard-coded `"Ctrl+E"` would be telling an operator to press
a key their manifest may not bind, which is worse than telling them
nothing: a chord that does not work reads as the *feature* not working.

### `fn with_condition`

# The frame-ordering hazard this exists for, in full

`PdfcerApp::conditions()` is evaluated **once**, at the top of the
frame, before the ribbon is drawn — so its `selection.any` describes
the selection as it stood *at the start of the frame*. The canvas is
composed last, and a right-click over an unselected object **selects
it** (see [`crate::canvas::menus`]). The click and the menu it opens
therefore happen on a frame whose snapshot still says nothing is
selected.

Left uncorrected the consequence is total, not cosmetic:
`format.delete` is gated on `selection.any`, so it resolves disabled,
so `offers_anything` is false, so **the menu does not open at all** —
and it never opens later either, because `egui`'s popup is opened by
the secondary click and there is no second click. The first
right-click on an object would silently do nothing, which is
precisely the class of defect (`DEFECTS.md` D1) this whole stage
exists to end.

# Why this is not a second source of truth

It corrects **one named condition to a value the caller has just
computed**; it does not re-derive the condition set. The rule for
*when* `selection.any` holds still lives in exactly one place —
`PdfcerApp::conditions`, reading `OpenDoc::selection` — and the caller
here passes `!selection.is_empty()` read from the same field, one
frame later. The spelling of the condition comes from
[`super::manifest::SELECTION_ANY`], which is the same constant the
Format tab's `visible_when` and `format.delete`'s `enabled_when` are
built from.

Returns an owned set, because the borrow it corrects is shared and
the correction lasts exactly as long as the one `attach` call that
wants it.

### `fn with_conditions`

⇒ Both conditions the canvas corrects are *the same fact one frame
later*, and they should arrive by the same route. `with_condition`
delegates here so there is one implementation.

### `fn attach`

**Nothing is executed.** The returned tokens are *intent*; the
application dispatches them at the one choke point the ribbon
already uses, which is where the confirmation gate and the undo entry
belong.

A context with no menu, a menu whose every command is missing from
this build, and a menu whose every command is disabled all produce
the same thing: **no popup, and an empty `Vec`**. The engine takes
that decision before `egui` is asked for a popup, and it also closes
an already-open popup whose offer has evaporated — so a menu left
open over a selection that is then deleted vanishes rather than
lingering with a dead Delete in it.

### `fn would_open`

Pure and cheap, and the *same* question [`Self::attach`] asks itself
— so a caller that wants to draw a "⋯" affordance beside a row, or a
test that wants to assert the empty-menu rule without opening a
window, gets the answer the operator will actually get.
