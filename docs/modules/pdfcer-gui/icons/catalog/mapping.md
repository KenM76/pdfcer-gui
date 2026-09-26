# `pdfcer-gui/icons/catalog/mapping`

## Item notes

### `const ALL`

This is the list the catalogue-wide tests walk, and it is what makes
"every shipped asset is valid" an enforced property rather than a
hope — so a new [`Icon`] variant MUST be added here or it ships
unverified. `all_is_exhaustive` guards the omission that would
otherwise be invisible.

### `fn source`

`include_str!` at compile time rather than a runtime file read,
because pdfcer ships single-folder portable: the executable must not
depend on an `assets/` directory travelling beside it, and an icon
that fails to load at startup is not a failure mode worth having when
the whole set is ~79 KB of text. See [`super::assets`] for why the
`.svg` files live inside `src/icons/`.

### `fn name`

Two jobs, and they are the same string on purpose:

1. **It is the application's icon key**, the thing a command names
   with `.with_icon("…")` and the thing `egui-shell` hands back in
   `IconRequest::key`. The shell never interprets it — an icon set is
   a licensing and rasterization decision, which is the application's
   business — so this is the only place the vocabulary is defined.
2. **It is the texture's debug name.** egui keys textures by handle,
   not by name, so that part is purely for debuggers and texture
   inspectors — but a texture list full of "icon" tells you nothing,
   and one full of `icon:rotate-ccw@32:Bold` tells you everything.

Kebab-case throughout, matching the command ids and the asset
filenames it was salvaged from.

### `fn from_key`

This is the lookup [`super::paint_ribbon_icon`] performs on every
icon-bearing ribbon control, every frame.

# Why a linear scan and not a `match` or a `HashMap`

A reverse `match` would be a second copy of the key vocabulary, and
two copies of a mapping is exactly how a rename lands in one of them.
[`Icon::name`] stays the single source of truth and this walks it.

The cost is one pointer-length comparison per catalogue entry
([`Icon::ALL`]`.len()`) with an early exit, for the
handful of icons a ribbon draws per frame — comfortably under a
microsecond, against a frame budget of 16 ms. A `HashMap` would need
a lazily-initialised static, would hash the key anyway, and would buy
nothing measurable. If the set ever reaches the hundreds, revisit;
`every_name_round_trips_through_from_key` makes the swap safe.
