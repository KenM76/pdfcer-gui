# `panels::properties::toolregion` — the Region block in Tool Options

Shown for the snapshot tool. Without a box: the instruction. With one:

- the chip, *"Region: page 3, 4.2 × 3.1 in (107 × 79 mm), 7 objects"*; the
  count is the Delete count under the current choice, cached per box,
  revision and choice because a hit test on a dense drawing is not free;
- Copy text / Copy outlines;
- when the document is editable: **Delete takes: Inside only | Touching**,
  Delete in region, **Redact: Inside | Outside** with one button, and Crop to
  region.

Each button raises its registered command (`Action::Command`), so this block,
the ribbon and the context menu run one handler, and a build without a
command loses its button.
