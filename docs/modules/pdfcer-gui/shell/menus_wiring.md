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

# ★★★ Capability 1 — the rows publish where they were drawn


`right_clicking_a_form_field_opens_its_menu` is the evidence. It is the
first driven context menu in this project's history, it asserts that
the right menu *resolved* and that it *offered something*, and it stops
there — because the next step, pressing a row, had nothing to press.
Its own header records the shape: *"a gesture with no driver is a
gesture R1 cannot reach, and the gap left no failing test behind to
advertise itself."* That was the same finding one layer down: the
driver existed and the target did not.

★★ Why an `egui` popup makes this the ONLY possible answer, rather than
the tidiest one. `egui_shell::menu::report`'s header states it: a
context menu is drawn at the pointer, and `egui` may flip it to any of
several alignments to keep it on screen. There is no fraction of the
window it can be hard-coded to and no layout a harness could re-derive.
Publishing the rectangle is not the best of three options; it is the
only one.

★ The names are `egui_shell::menu::report`'s — `menu.body.<context>`
and `menu.item.<context>.<command id>` — and they go through
[`crate::diag::ui_rect`], the same sink the ribbon, the status bar and
the dock already publish to. So a harness filters one channel and one
prefix, and nothing here invents a naming scheme.

★ Cost when nobody is listening: the shell's `Reporter` does not format
a name unless a sink is present, and `crate::diag::ui_rect` is a no-op
without `PDFCER_DIAG`. A closure per attach, and nothing else.

# ★★★ Capability 2 — the rows draw the icons they already name


`ContextMenu::with_icon_painter` has existed since the menu engine
landed. Nothing called it. So every context-menu row in every build of
this application drew a label and nothing else — **including rows whose
command already carried an icon key**, resolved, catalogued and
rasterizable. `view.panel_float` and `view.panel_dock` name
`floating-panels` at their registration; the key was correct data
waiting for a surface that read it.

★★ The finding that made this a pass of its own is what the gap did to
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

★ The painter is [`crate::icons::paint_ribbon_icon`] — **the ribbon's
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

★ What this does **not** do is put a glyph on every row. The shell
decides the column per menu (`egui_shell::menu::plan::reserves_icon_column`)
and the glyph per command, so a menu whose commands have no icons is
laid out exactly as it was before, and a row with no key inside a menu
that has them indents and paints nothing. R9: an icon-less row leaves
no mark that reads as a missing picture.
