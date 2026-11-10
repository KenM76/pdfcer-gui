# `app::prefs::markuptools` — keeping the markup tools' choices

The highlighter's shape and Delete in region's *Inside only* / *Touching*
live in `egui` temp memory, where Tool Options and the canvas read and write
them. `keep(ctx, prefs)` runs every frame from `app::frame`, beside
`formstyle::keep`:

1. **First frame:** seed memory from `prefs.markup_tools` (`highlight::seed`,
   `region::set_touching`), mark memory seeded, trace
   `markup-tools-seeded shape= touching=`.
2. **Every later frame:** read memory back; on a difference from
   `prefs.markup_tools`, and only with no pointer button down, copy it in,
   save the file, trace `markup-tools-saved ok= shape= touching=`.

Comparing rather than hooking every writer means a new control that changes
either choice is remembered without knowing this module exists. Waiting for
the pointer keeps a drag from writing the file mid-gesture.

`highlight::seed` sets the shape without the side effects of choosing one
(no polygon abandoned, no trace), because nothing was chosen.

Driven by `ui-verify --check highlight_shapes_draw_as_chosen`, which requires
`markup-tools-saved ok=true shape=ellipse` after Ellipse is clicked.
