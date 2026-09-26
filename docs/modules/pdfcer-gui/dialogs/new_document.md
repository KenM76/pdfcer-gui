# `dialogs::new_document` — the other half of New

`RIBBON_IA.md` §5.1's File band specifies one row as
**`New (blank / from template)`**, and only the blank half shipped:
`file.new` makes an A4 page with no question asked, which is what Acrobat
and Inkscape both do. This is the other half — `file.new_from_template`,
the command that asks what kind — and Inkscape's split is the shape being
followed: `Ctrl+N` makes a document, `Ctrl+Alt+N` chooses what kind.


`crate::app::blank`'s §3a is the record and is worth reading before
touching this file. In short: nothing in `pdfcer-core` wrote a `/MediaBox`,
so the only implementation available to a shell was **one checked-in
template asset per size** — ten with landscape, more with ANSI, *and a
custom size impossible at any count*. That half-capability was refused
rather than built, because a surface that answers twelve of thirteen cases
forecloses the fix for the thirteenth.

`EditSession::set_media_box` and `pdfcer_core::paper` shipped, so the whole
thing is one asset, one dialog, every size, both orientations, custom
included.

## What it does NOT do, and why each is a decision

**It does not resize an existing page.** `set_media_boxes` supports that
and it is a genuinely different capability with genuinely different
questions — does content move, does `/CropBox` follow, is shrinking below
the content a refusal. None of those arise here because `file.new`'s page
is empty by construction, which is exactly why the request that unblocked
this deliberately asked only for the narrow answer. A page-resize surface
belongs in Document ▸ Properties and is not this module's.

**It does not remember the last size chosen.** The obvious convenience, and
it is left out on the evidence rather than on principle: nobody drafts a
sheet in pdfcer, so the *second* use of this dialog is rare enough that a
remembered value would more often be stale than helpful — and a New command
that silently produced A1 because of something the operator did last
Tuesday is worse than one that always starts where its sibling does. If the
operator reports otherwise, `crate::app::prefs` is where it would go, beside
the opening-view preferences, and this paragraph is the argument to
overturn.

**It offers no templates**, despite the command's name. `RIBBON_IA.md`'s
parenthetical for this row is `(page size)`, which is the IA's own
annotation of scope — the same shape as `Export image… (PNG/JPEG/TIFF, DPI
picker)`. So the label follows the IA, and
[`crate::text::new_document::intro`] states in the window's first line what
it actually offers, because that is the cheapest correction available to a
session that may propose IA amendments and may not make them.

## Application-scoped, like About and Settings

It draws with nothing open and must: an operator with an empty shell is the
one most likely to want it, which is the same argument `file.new` and
`file.open` are registered with no `enabled_when` for. So it is drawn
**before** `crate::dialogs::DialogsState::show`'s document guard, beside
About, and closing a document does not close it.

## Rule 4: nothing here is drawn on a page

There is no page yet. This window states what it will make and makes it;
the sheet it reports is the sheet that lands, and there is no inference to
disclose because there is no inference — the operator typed or picked every
number in it. The one thing pdfcer decides on their behalf is the **refusal**
of an out-of-range custom size, and that is stated in words with its limits
named (`crate::text::new_document::custom_refused`).
