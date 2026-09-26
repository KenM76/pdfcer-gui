# `panels::comments::filter` — narrowing the reviewer's work list

One subject: **which rows the Comments panel shows, and in what order.**
The state, the pure predicate, and the control strip that sets them.

## ★★★ Why this exists — the gap the operator's report exposed

The operator, 2026-09-05: *"the review features should look and act the
same as they do in Acrobat Reader."* Acrobat's Comment pane is a **work
list**, and a work list you cannot narrow is a list you scroll. Its filter
offers reviewer, type, status and checkmark; its sort offers page, type,
author, date and colour.

This panel had **none of it**: every annotation in the document, in
document order, always. On `SW41177.pdf` — thirty-six sheets — a reviewer
looking for their own three comments scrolls past everybody else's.

## ★★ What is offered, and what is NOT, and why each absence

| Acrobat offers | here | why |
|---|---|---|
| filter by **reviewer** | ✅ [`Filter::author`] | `/T` is modelled and read |
| filter by **type** | ✅ [`Filter::subtype`] | `/Subtype` is modelled and read |
| filter to **comments with text** | ✅ [`Filter::with_note_only`] | ★ this shell's own, and it earns its place: pdfcer's own markup authoring cannot write `/Contents` on a geometric shape, so a drawing pdfcer marked up is a column of "no note" rows with the reviewer's actual remarks scattered through it |
| filter by **status** (Accepted / Rejected / …) | ✅ [`Filter::status`] | ★★★ **arrived 2026-09-06.** It was ❌ here for one day, on the grounds that *"`/State` and `/StateModel` have zero occurrences in `pdfcer-core` v0.38.0 — not read, not written, not modelled"*, filed as `request_review_status_is_not_modelled_at_all.md`. `Pass 253.1` closed it. The predicate is **not** in [`Filter::keeps`] and could not be — a status lives on *other* annotations (§12.5.6.3) — so it is answered by [`super::reviewstate::narrow`]; that field's doc carries the whole split |
| filter by **checkmark** | ❌ | a per-viewer flag Acrobat keeps outside the PDF. Not a document property, so not this panel's |
| sort by **page** | ✅ [`Sort::Document`], the default | |
| sort by **type** | ✅ [`Sort::Subtype`] | |
| sort by **author** | ✅ [`Sort::Author`] | |
| sort by **date** | ❌ | ★★ **`/M` is not reliably a date.** §12.5.2 gives its type as *"date **or** text string"* and requires a reader to accept any format, so `pdfcer-core` stores it raw and its own docs say *"do not assume it parses"*. A sort would have to either parse it — rejecting legal values — or sort the strings, which orders `D:2026…` before `17 January` and calls it chronology. `crate::text::panels::comments::comment_row_byline` makes the same ruling for display and this is it holding for ordering |
| sort by **colour** | ❌ | `/C` is **not in the engine's read model** at all — `annot.rs`'s parser reads `/CA` and never `/C`. Filed as `request_an_annotations_colour_cannot_be_read.md` |

⇒ Of the four absences this table opened with, **one has since closed** —
status, on the day after it was written — and it closed because the request
was filed rather than worked around. Two remain as engine gaps with requests
filed, and the fourth is a deliberate exclusion. None of them is drawn
greyed: R9.

★ That is worth leaving in place rather than tidying into a table of four
ticks: the row's history is the evidence for how this shell is supposed to
meet a capability it does not have — say what is missing, name the request,
draw nothing — and a table that had been rewritten to look as though status
was always there would have thrown that away.

## ★★★ The disclosure that makes filtering safe

A filtered list is a list that is **lying by omission** unless it says so.
`crate::panels::comments`' founding discipline is that *"nothing is
silently omitted"* — its exclusion line already states the arithmetic for
widgets, pop-ups and `/TrapNet` — and a filter is a fourth kind of
omission, and the only one the operator caused.

So [`Filter::is_narrowing`] exists, and the panel draws
`comments_filtered(shown, total)` above the list whenever it is true. A
reviewer who set a filter, went to lunch and came back must not conclude
from six rows that the drawing carries six comments.

## The sort is STABLE, and that is load-bearing

`sort_by_key` on a `Vec` is stable in Rust, so sorting by author leaves
each author's comments in **document order** — page order then `/Annots`
order, which is `pdfcer list-annotations`' ordering, reused by name rather
than re-decided. An unstable sort would reshuffle a reviewer's own comments
on every frame, which reads as the panel flickering.
