# `pdfcer-gui/shell/menus_wiring`

**The optional capabilities pdfcer hands to every context menu**, and
the account of why each one exists.

`egui_shell::menu::ContextMenu` is a builder with four optional seams —
an icon painter, a custom-item renderer, a rect sink and a shortcut
override — and each one exists because the shell *cannot* supply it:
an icon set is a licensing and rasterization decision, a rect sink is a
harness decision, and neither belongs in a crate that must never learn
what a PDF is (R7). This module is pdfcer's answer for the two of the
four that every menu in this application wants.

It is a module of its own rather than eight lines inside
[`super::menus::MenuHost::attach_with`] for two reasons, and the second
matters more than the first:

1. `menus.rs` is at R2's 1,500-line ceiling, so it has no room for a
   new account.
2. **Both of these are seam decisions, not menu-host bookkeeping.**
   `MenuHost` exists to bind *this frame's* document, registry and
   conditions together; what a menu is *capable of drawing* is a
   property of the build, identical on every frame and at every call
   site. Keeping them apart means the answer to "why does a menu row
   have a glyph?" is in one file, next to the answer to "why does a
   menu row publish a rectangle?", rather than buried in the middle of
   a lifetime-juggling struct.

# Capability 1 — the rows publish where they were drawn


`right_clicking_a_form_field_opens_its_menu` is the evidence. It is the
first driven context menu in this project's history, it asserts that
the right menu *resolved* and that it *offered something*, and it stops
there — because the next step, pressing a row, had nothing to press.
Its own header records the shape: *"a gesture with no driver is a
gesture R1 cannot reach, and the gap left no failing test behind to
advertise itself."* That was the same finding one layer down: the
driver existed and the target did not.

Why an `egui` popup makes this the ONLY possible answer, rather than
the tidiest one. `egui_shell::menu::report`'s header states it: a
context menu is drawn at the pointer, and `egui` may flip it to any of
several alignments to keep it on screen. There is no fraction of the
window it can be hard-coded to and no layout a harness could re-derive.
Publishing the rectangle is not the best of three options; it is the
only one.

The names are `egui_shell::menu::report`'s — `menu.body.<context>`
and `menu.item.<context>.<command id>` — and they go through
[`crate::diag::ui_rect`], the same sink the ribbon, the status bar and
the dock already publish to. So a harness filters one channel and one
prefix, and nothing here invents a naming scheme.

Cost when nobody is listening: the shell's `Reporter` does not format
a name unless a sink is present, and `crate::diag::ui_rect` is a no-op
without `PDFCER_DIAG`. A closure per attach, and nothing else.

# Capability 2 — the rows draw the icons they already name


`ContextMenu::with_icon_painter` has existed since the menu engine
landed. Nothing called it. So every context-menu row in every build of
this application drew a label and nothing else — **including rows whose
command already carried an icon key**, resolved, catalogued and
rasterizable. `view.panel_float` and `view.panel_dock` name
`floating-panels` at their registration; the key was correct data
waiting for a surface that read it.

The finding that made this a pass of its own is what the gap did to
the *record*. An icon-coverage audit had recorded, against
`view.panel_close`, that a menu row cannot draw a glyph because *"the
icon column exists on the ribbon, not in a context menu"*. That
sentence is a statement about this application's wiring dressed as a
statement about menus, and once written it was quoted — a refusal
resting on a line nobody wrote reads exactly like a refusal resting on
a decision somebody took. The operator's standing ruling (2026-08-06,
quoted in `crate::icons::Icon::Back`'s doc comment) is that **a missing
glyph is authored, not worked around**, and the test that separates a
valid refusal from an invalid one is whether adding the slot would be
*wrong* or merely *work*. Here it was merely work: one builder call.

The painter is [`crate::icons::paint_ribbon_icon`] — **the ribbon's
own**, not a second one. The alternative was a menu-specific painter,
and it is worth naming why that is the wrong shape: the two surfaces
would then resolve the same key through two catalogues, and the day one
learned a new glyph the other would silently keep drawing the missing
mark. One painter means a key that draws on the ribbon draws in a menu,
by construction rather than by diligence.

A plain `fn` item satisfies the shell's `FnMut` bound, so there is no
closure and no captured state — the same property `app::surfaces`
keeps for the ribbon, and for the same reason: *a painter with no state
cannot be the thing that goes stale.*

What this does **not** do is put a glyph on every row. The shell
decides the column per menu (`egui_shell::menu::plan::reserves_icon_column`)
and the glyph per command, so a menu whose commands have no icons is
laid out exactly as it was before, and a row with no key inside a menu
that has them indents and paints nothing. R9: an icon-less row leaves
no mark that reads as a missing picture.

## Item notes

### `fn rows`

Reads the shipped documents rather than a list written here: a
second copy of the menu contents would agree on the day it was
written and drift silently afterwards, which is the failure
`NO_SURFACE.md` §1 records with a colour.

### `fn the_icon_column_lights_up_the_rows_whose_commands_already_name_a_glyph`

The value of the change, asserted rather than claimed. Every count
below is derived from the shipped menu documents and the shipped
registry — nothing here restates a menu's contents — so the numbers
move only when a menu or a command's icon does, and then this test
says so instead of the change being invisible.

The three columns are the three states of
`egui_shell::menu::plan::IconSlot`:

* **glyph** — the command names an icon; the painter draws it.
* **blank** — the command has no icon but a sibling in the same
  menu does, so the row indents and paints nothing.
* **absent** — no command in that menu has an icon; the menu is
  laid out exactly as it was before the column existed.

Note what the blank count is for. It is the cost of the rule, and
it is the number to watch: if it ever exceeds the glyph count in a
menu, that menu is an icon column that is more than half empty, and
the right answer at that point is to argue about *that menu* rather
than to weaken the rule for all of them.

# What the numbers were on the day the painter was wired

**25 glyph, 2 blank, 0 absent, and all nine menus reserve.** Every
one of those 25 rows was already carrying a resolved, catalogued,
rasterizable icon key and drawing a bare label, because one builder
call was never written. No per-row work was done to light them up
and none was needed.

The `absent` column being **zero** is the sharper reading: there is
no menu in this build without icons, so the "a plain menu is laid
out exactly as before" guarantee is real but unexercised here — it
is exercised in `egui_shell::menu`'s own tests, which is the right
place for it.


* `view.panel_close` in `dock.tab` — **GONE.** It took `close`, so
  `dock.tab` reads 4 glyph / 0 blank where it read 3 / 1. The
  refusal that kept it bare had already been rewritten once by the
  pass that wired the icon painter (it had rested on this surface
  having no icon column, which stopped being true that day), and
  what was left underneath was *"there is no close art in this
  set"* — false since the day the set landed, because `file.close`
  has worn `close` throughout. The row was drawing a bare label one
  line under a Float that had a picture, which is the same word
  twice with only one of them drawn.
* `view.zoom_actual` in `canvas.empty` — **STAYS**, and it is now
  the only blank row in the build. Argued against BY NAME in the
  icon ui-spec §3.2 and marked `{noicon:1}` in the approved mockup:
  a numeral read at a glance is clearer than any glyph substitute,
  and both add a decode step a bare percentage does not need.

⇒ The two blanks looked alike in this list for weeks and were not
alike at all. One named a **wrong picture** — an argument no amount
of drawing answers — and the other named a **missing picture**,
which under the operator's standing rule is not a reason but a work
item. Grouping them under one sentence (*"both refusals argued at
their command's own registration, and neither is a gap waiting to be
filled"*) is what let the false one ride along on the true one's
credibility. **When a list of exceptions shares a justification,
check that they share a KIND.**

The counts are of the **documents'** rows, not of one frame's.
A `shown_when` row is counted whether or not its condition holds, so
`dock.tab` contributes both `view.panel_float` and
`view.panel_dock` although an operator is only ever shown one. That
is the right shape to assert: it is the menu as authored, and it
cannot vary with a condition set the test would have had to invent.

### `fn a_reserved_icon_column_is_never_mostly_empty`

The rule this wiring rests on is *"reserve the column iff any row in
this menu has a glyph"*, and the argument against the alternatives
(per-row, so the labels zig-zag; or a glyph forced onto every row,
so the column fills with pictures that do not mean anything) is in
`egui_shell::menu::plan::reserves_icon_column`.

What the argument does not cover is the case where the rule is
*technically* satisfied and reads badly anyway: one glyph at the top
of nine bare rows is a column that looks broken rather than
deliberate. That is a judgement about the menus this build ships,
not about the rule, so it is asserted here rather than in the shell
— R7: `egui-shell` must not learn which of pdfcer's menus look
right.

The bar is *most of the rows*, not all of them: `dock.tab` draws
three glyphs beside one bare `Close`, which is the shape every
desktop menu has and is fine. A menu that fell below it would be
asking for either art or an argument, and this test is where it
would be asked.

### `fn attach`

**Executes nothing.** The returned tokens are *intent*; the caller
dispatches them at the application's one choke point. See
[`super::menus::MenuHost::attach_with`], which is the only caller and
which owns the frame-ordering account for `conditions`.

The `&mut` bindings are what the shell's builder asks for
(`with_icon_painter(&'a mut (impl FnMut(..) + 'a))`), and they must
live until `attach` returns — which is why they are locals here and not
fields on anything.
