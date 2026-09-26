# `pdfcer-gui/panels/pages/previews`

**The page-previews row** — the operator's instruction about thumbnails,
the time limit that bounds one, and the sentence explaining a tile that
has no picture.

# Why this is its own file

R2, immediately: the O151 rewrite pushed `panels/pages/mod.rs` past 1,500
lines. But the seam was already there, which is the part worth stating —
everything here reads and writes exactly one object,
[`crate::panels::pages::thumbnails::ThumbnailCache`], and nothing else in
the pages panel reads what it writes. A subject that touches one type and
is touched by nothing is a file.

# What it draws, and the one rule that governs all of it

```text
[x] Draw page previews   [≤ 2.0 s]
Page 3 needed more than 2.0 s to draw and was skipped. Raise the …
```


> *"the drawing page previews checkbox should never automatically turn
> off. You can add a box next to the checkbox to enter a timeout value."*

Before that, an expensive page cleared the checkbox on the operator's
behalf; `thumbnails.rs`'s "the skipping rule" section carries the full
argument for why that was wrong and what replaced it. What binds *this*
file is the consequence: [`row`] writes
[`ThumbnailCache::force_on`](crate::panels::pages::thumbnails::ThumbnailCache::force_on)
from a `changed()` checkbox and from nowhere else, and the cache's own
source-scan tripwire holds the other half.

# Rule 4 — where the disclosure goes

Off-canvas, above the grid, and it names **one page**. A skipped page has
no picture for a specific reason, and rule 4's *report separately* means
that reason must be stated somewhere the operator will meet it before
they draw their own conclusion — which, for a grid, is above it rather
than under it. It is not drawn on any tile: the tile carries a word
([`crate::text::pages::thumbnail_abandoned`]) and nothing else.
