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

## Item notes

### `fn persist`

Reads BOTH values back out of the cache rather than taking the one that
just changed as an argument, and that is the point rather than
convenience: the cache is the live truth, both controls have already
written to it by the time either calls this, and an argument list would
be a second copy of the same two facts for a future edit to get out of
step with.

One action, therefore one `Prefs::save`, therefore one whole-file write
per operator gesture.
[`PrefAction::PagePreviews`](crate::app::actions::prefs::PrefAction::PagePreviews)' own doc
carries the
argument for why the two are not separate variants.

### `fn parse_budget`

A `DragValue` re-parses its own rendered text the moment the operator
clicks into it to type. A parser that did not accept what the formatter
produced would leave the operator staring at an empty box every time they
clicked the control, which is a defect no unit test that only checks
numbers would see.

Three things are accepted, in this order:

1. The word the box shows at zero — case-insensitively, because an
   operator who retypes it will not match the capitalisation.
2. A bare number, which is what somebody who selects-all and types `0`
   produces, and the case O187 is actually about.
3. A number still wearing the prefix or the suffix the formatter added,
   which is what a partial edit of the displayed text produces.

⚠ Returns `None` rather than `Some(0.0)` on anything else. `None` means
*keep the value you had*, and that is the only safe answer: mapping
gibberish to zero would silently arm **no limit at all** from a typo.

### `fn the_parser_accepts_what_the_box_shows`

Written as a **round trip through the formatter's own output**, not
against hand-typed strings: the two are one convention, and a test that
quoted the rendered text verbatim would keep passing after somebody
changed the suffix.

### `fn gibberish_changes_nothing`

The clause that matters: it must NOT come back as `Some(0.0)`. Zero is
*no limit at all*, so a parser that mapped a typo to zero would arm an
unbounded render from a slipped keystroke — the worst outcome this
control has, reached by the likeliest accident.
