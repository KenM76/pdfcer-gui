# `app::state::pageepoch` — **which PAGE changed, not just that something
did**

One type, [`PageEpochs`], and one rule that the whole thing exists to
enforce: **the safe answer is the default, and precision is opted into per
verb.**

## The report this closes

`OPERATOR_REQUESTS.md` row **O74**, in the operator's words:

> *"When I make edits or even just fill out a form I notice all of the page
> previews get re-rendered instead of just the one that is being changed,
> and it seems to really slow down clicking a checkbox in a form. The last
> thing that should matter is updating the preview, and it should just
> update the pages that were actually altered."*

He has given the priority rule as well as the bug, and the priority rule is
the more valuable half: **a thumbnail is the lowest-priority work in the
program and must never sit between a click and its result.**

## What this buys, and what it costs to get wrong

[`OpenDoc::edit_epoch`](crate::app::state::OpenDoc::edit_epoch) is a
**document-wide** counter. Keying a per-page cache on it invalidates every
page on every edit, and the caches that would do so are expensive: the
thumbnail rail re-renders inline on the UI thread (twelve visible tiles on
the operator's 36-sheet SolidWorks set measure 666 ms of UI-thread work per
edit, 282 ms worst frame), and a full-size strip raster on the benchmark CAD
drawing is ~950 ms *per page*.

## ★★★ Why the default is `bump_all` and precision is opt-in

Because getting this wrong is **worse than the slowness it fixes.**

A thumbnail kept because its page's epoch did not move is a claim that the
picture is current. If that claim is ever false, the rail shows the
operator content he has already changed — and under rule 4 (*"fuzzy, never
sneaky"*) a stale picture presented as a current one is precisely the
category of defect that outranks a performance complaint. A slow program is
annoying; a program that shows you the wrong drawing is dangerous.

So the asymmetry is deliberate and structural, in three places at once:

1. **Both epoch-bumping sites call [`PageEpochs::bump_all`] unless the
   caller has *proved* the edit was confined to one page.** A verb that has
   not been examined behaves exactly as it did before this module existed.
2. **[`PageEpochs::get`] returns `max(all, per_page[page])`**, so a
   `bump_all` can never be undercut by a stale per-page number, in any
   ordering, ever.
3. **A page index out of range answers `all`**, not zero — a question about
   a page that does not exist is answered with the most conservative number
   available rather than with a value that would look fresh.

The verbs that must never be narrowed, and the reason each is dangerous:

- **Anything that changes the page SET** — insert, delete, reorder, merge,
  extract. Page *n* is a different sheet afterwards, so every per-page
  number describes the wrong page. `pages::resync` already computes exactly
  this fact (`renumbered`) and it is what drives the `bump_all`.
- **Undo and redo**, which run any command backwards and cannot say which
  page they landed on.
- **Anything document-scoped**: font embed/unembed, `/AcroForm` `/DR`
  changes, `RegenerateAppearances`, `Flatten`, metadata.
- **A form field whose widgets straddle sheets.** A single `/T` may have
  widgets on several pages; filling it changes all of them.

## What `edit_epoch` still means, and why it is untouched

Everything. `page_objects`, `page_text`, `form_runs`, `saved_epoch` and
every rule-4 disclosure slot still key on it and still behave identically.
This is a **finer answer laid beside the existing one**, not a replacement,
so a reader who does not know this module exists cannot be wrong about
anything.
