# `canvas::runmenu` — **the right-click route to ONE LINE of a text block**

## The operator's report, and the half of it this file is

`OPERATOR_REQUESTS.md` O188, verbatim:

> *"In text that is grouped together or whatever it is called, such as in
> my title blocks, I would like a way to move the individual text blocks
> within it around, and have the ability to delete them…"*

That request has three halves and this is the third.

* **(B)** delete one line — shipped: `EditSession::delete_text_run`,
  reachable at the Part rung.
* **(C)** move one line — shipped: `EditSession::move_text_run`, reachable
  by dragging the same Part-rung selection. ⚠ Its unit is **one show
  operator**, which on a producer that writes a line in fragments is not the
  whole visual line; `OPERATOR_REQUESTS.md` O214 is that gap.
* **(A)** *find* either of them — **this file**.

The note filed with (B) and (C) stated (A) in one line, and it is the line
this module exists to discharge:

> ⚠ **(A) is still open.** The delete is reachable only through the Points
> tool (`A`, then click); a double-click on text opens the caret instead,
> which is O70's ruling and correct. The new sentence tells him Delete
> works, but only after he has tried to drag. **A route he can find
> *before* failing is owed.**

## One route, announced nowhere

Measured across every gesture this canvas has: without this module the Part
rung on a text object — the rung at which *one line* is the operand — is
reachable by **exactly one** sequence:

> be in Edit mode, arm the **Points** tool (chord `A`, `view.tool_node`)
> *before* clicking, then single-left-click the line.

Everything else lands at the Object rung and stays there:

* **plain click, Select tool** — Object: `SelectionState::click` →
  `click_at_object_rung`, which discards the hit's part outright.
* **marquee** — Object, hard-set and correctly: a band names a region, and
  a region contains objects.
* **double-click** — the **caret**. O70's ruling, and correct: text under a
  double-click means *type here*.
* **right-click** — Object: `menus::right_clicked_object` asks `hit_test`,
  not `probe`.

⇒ So the one working route required knowing a chord that no surface names,
*and* arming it in advance. An operator who clicks the line first and then
goes looking has already lost the rung. That is not a discoverability
weakness; it is a capability that ships unreachable to anybody who was not
told.

## Why a MENU ROW and not a new gesture

The standing rule is *use the conventional interaction, never invent one*,
and it bars a menu row that **performs an edit**: the conventional way to
move a piece of something is to drag it, not to pick a row called *Move*.
That is the *move* half, and it is not this file.

A row that **changes what is selected** is the other kind of row entirely.
This menu already carries two — `format.select_form` and
`format.unshare_form` both re-aim the selection from the canvas, for the
reason this row exists too: the operator's hand is on the thing, and the
ribbon is three inches away. `canvas::annotnodes::menu`'s two shipped rows
are the same shape one surface along — a Points-tool chord route nothing
announced, given a row somebody can read.

And `canvas::menus`' own rule that *"a right-click never descends"* is
not breached. The right-click still selects the **whole object**, exactly
as it always has; the descent happens only when the operator reads a row
that says so and presses it. The rule is about what a click does silently.

## The operand problem, and where it is parked

A menu row carries **a command id and nothing else**
(`egui_shell::manifest::Item::Command`). *"Select this line of text"* needs
to know **which line** — a fact that exists only at the instant of the
secondary click, on a surface the dispatcher never sees.

So the pick is parked in `egui::Memory` for the life of the popup, beside
the two things already parked there for the identical reason: which canvas
menu is open (`canvas::menus`' `MENU_MEMORY_KEY`) and which corner a markup
right-click landed on (`canvas::annotnodes::menu`'s `PICK_MEMORY_KEY`).
`egui` opens a popup ON the secondary click and redraws it every frame until
it is dismissed, and the pointer moves during those frames — onto the menu
itself, which is not over the text any more — so recomputing from the live
pointer would swap the row's meaning out from under the operator's hand
while they were reading it.

⇒ [`park`] is called once, on the click. [`parked`] is read on every frame
the menu is drawn (to decide the row's condition) **and** again when the
command is dispatched (to build the selection). One value, three readers,
no second derivation of *"which line did they mean"*.

## R9: offered or absent, never greyed

There is no *temporarily unavailable* state here. Either the pointer is on
a line of a multi-line text object — in which case the row works — or it is
not, and nothing the operator can do while the menu is open changes that.
R9 reserves greying for the recoverable case, so this row has one condition
([`crate::shell::menus::RUN_SELECT_OFFERED`]) carried on **both** the
item's `shown_when` and the command's `enabled_when`: shown implies
pressable, and a stale frame cannot press a dead row.

**The `of > 1` gate is part of that decision and is not an
optimisation.** On a text object with a single run, *"select this line"*
and *"select this object"* name the same ink, and `delete_text_run` on the
only run is `delete_objects` spelled at a deeper rung. Offering the row
there would be offering a descent into a place identical to where the
operator already is — a row that appears to do nothing. See [`pick_at`].

## Rule 4 / R8b

Nothing here paints, and nothing here marks the document. This module
answers two questions — *which line is under the pointer*, and *is that a
thing worth offering* — and both are about the **cursor**. What the row
does when pressed is change a selection, which is an affordance and not
content. The one disclosure this feature owes — *which* line, of how many,
is now selected — lives off-canvas in the status row, where
`app::toolstatus` puts it.

## Rule 15

The word used throughout is **line**, never *dimension*. The CAD-exported
labels these runs usually are on the operator's drawings are **pdf
dimensions** — page content pdfcer reads and must not silently alter — and
this module neither reads them as measurements nor alters them. It moves a
selection.

## Item notes

### `const PICK_MEMORY_KEY`

One `Id`, per `egui::Context`, for the module header's reason: the pick is
taken at the click and read on every frame the popup is drawn, so it has to
outlive the click and must not outlive the session. `Memory` rather than
`PdfcerApp` state because this is frame-local interaction state with no
meaning across a document — closing one starts the next frame with no pick
and no popup in flight.

### `fn only_a_line_pick_is_offered`

Falsified: defining `offered` as `true` makes the second assertion
fail, which is the regression that would draw the row over paths,
images and blank paper.

### `fn the_default_pick_names_no_line`

Falsified: making a `TextLine` variant the default makes this fail, and
would have offered the row on every right-click anywhere on the canvas
before the operator had pointed at anything.

### `fn a_pick_from_another_page_is_refused`

The provider is `None` here, which is a second reason to decline — so
the page guard is asserted where it is the FIRST reason, above the
`targets?`. That ordering is load-bearing, which is why the second
assertion exists: it shows the two refusals are independent rather than
one of them covering for the other.

Falsified: moving the page comparison below `let targets = targets?`
leaves the first assertion passing for the wrong reason.

### `fn an_elsewhere_pick_resolves_to_nothing`

Falsified: an `_ =>` arm in [`resolved`] falling through to
`Some((TargetId::Object(0), 0))` makes this fail — the shape of the
*"unwrap_or_default into line 0"* defect the enum's two variants exist
to prevent.
