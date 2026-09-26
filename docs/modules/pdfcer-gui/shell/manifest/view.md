# `pdfcer-gui/shell/manifest/view`

The **View** tab — *what is on my screen, and how is the page laid
out?*

`RIBBON_IA.md` §5.2, amended by `MODES_AND_PANELS.md`'s two new Window
settings. Six groups: Page display, Render, Zoom, Display, Panels,
Window.

# The defect this tab exists to fix

`RIBBON_IA.md` §3, on the shipped build:

> **The View tab contains no view controls.** It has two groups:
> `Panels` and `Show`. There is no zoom, no page layout, no view
> rotation, no read mode, no full screen. Read mode and full screen
> have **no ribbon control at all** — they are keyboard-only (Ctrl+H,
> F11) on a tab literally named View. This is the single most confusing
> thing in the current ribbon.

So this tab gains four groups and loses one command (`Fonts`, to File ▸
Document, because it describes the file rather than the screen).

# Zoom here does not duplicate the status bar in spirit

The status bar keeps the *continuous* controls a user reaches for
constantly: −/%/+ and the fit toggles. This tab mirrors the three
**named** zoom levels under P1a — actual size, fit page, fit width — so
that a user looking under View for zoom finds zoom. The two *targeted*
zooms that would have no status-bar home, zoom-to-selection and marquee
zoom-to-region, are **N** and therefore absent.

Mirroring is legal because the status bar is not a tab; the same
amendment that lets the QAT carry Open lets the status bar carry Fit
page. What P1 forbids, and `egui-shell` still enforces, is one command
on two *tabs*.

# Two documented conflicts, resolved here

`RIBBON_IA.md` §5.2's table lists **`Thin lines` twice** — once under
Render and once under Display. One command cannot be on one tab twice;
`egui_shell::Shell::validate` refuses it by name. It is kept in
**Render**, because that is where the parameter acts (it is a
rasterization rule about minimum stroke width, not an overlay the
viewer draws) and because the Render group's contents were enumerated
explicitly, thin lines included, when this tab was commissioned. The
Display entry is treated as the duplicate.

§5.2 also lists **`Comments`** among View ▸ Panels' panel toggles,
while §5.5 gives Markup a `Comments` group holding the Comments panel
and §7's migration map sends the existing control there explicitly
(`Review ▸ Comments ▸ Comments` → `Markup ▸ Comments`). The migration
map is the more specific statement, so the command lives on **Markup**
and this tab's Panels group does not list it.

# The Render group is an operator decision, and it is a stated trade

pdfcer caches one whole-page texture and scales it with linear filtering
during the settle interval. Measured in use on a large drawing, that is
*smoother* to pan and zoom than the comparison product's progressive
tile rendering — no seams, no piece-by-piece fill-in — at the cost of a
full re-raster once motion stops.

Those are two legitimate trades, not a better and a worse, and which
one wins depends on the sheet and the machine. So the strategy is a
**choice on this tab**, with whole-page as the default because it is
what measured better. `ZOOM_SETTLE` and the raster-scale multiplier are
constants in the shipped code and become the two knobs beside it.

**Status note.** Three of the five Render entries and both new Window
settings are **N** in `RIBBON_IA.md`'s marking, and would be absent
under P3. They are present because the tab was commissioned with them
named individually and their defaults specified — see
[`super::DIRECTED`], which lists every such entry with the instruction
that put it there, so the exception is visible rather than inferred.
