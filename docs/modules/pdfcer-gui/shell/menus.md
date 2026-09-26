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
