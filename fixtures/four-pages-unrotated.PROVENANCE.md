# `four-pages-unrotated.pdf` — four pages, and **not one `/Rotate` key anywhere**

## Why this file exists when `four-pages.pdf` is right beside it

`page_ops_round_trip` proves that Pages ▸ Rotate reached the *file* by saving a
copy and finding `/Rotate 90` in it that is not in the source. That evidence is
only evidence if the source has no rotation of its own — otherwise a `/Rotate`
in the saved copy is indistinguishable from the fixture's own furniture, and the
check cannot tell a working rotate from a dead one.

Measured 2026-09-12, on the two four-page documents this project had:

| file | `/Rotate` entries | usable as the rotate control? |
|---|---|---|
| `fixtures/four-pages.pdf` | `/Rotate 0` × 1 | **no** — carries the key already |
| `fixtures/a1-titleblock.pdf` | `/Rotate 0` × 1 | **no** — and it is one page |
| this file | none | **yes** |

★ `/Rotate 0` is semantically "no rotation" and renders identically to an absent
key, which is exactly what makes it dangerous: it is invisible on screen, it is
invisible in the page view, and it defeats a substring search that the check
performs on the saved bytes. The 2026-09-12 sweep SKIPPED `page_ops_round_trip`
for precisely this reason and its message named the right cure.

## Where it came from

A byte-for-byte copy of `D:\Dev\pdfcer\fixtures\synthetic\pageops\four-pages.pdf`
taken on 2026-09-12 — the engine repository's own page-operations fixture,
2,453 bytes, four `/Type /Page` objects, no `/Rotate`, no `/AcroForm`, no fonts.

It is **copied rather than referenced** because `D:\Dev\pdfcer\` is read-only to
this project and is a moving tree: a check that reaches across the repository
boundary for its input would go quiet the day the engine reorganised its
fixtures, and it would go quiet as a SKIP rather than as a failure. A fixture a
check depends on belongs in the repository the check lives in.

## What it is NOT for

Not a replacement for `fixtures/four-pages.pdf`. That one is a real
four-sheet drawing set with drawn content on every page, and it is what
`pages_drag_shows_where_it_lands` and the thumbnail checks need in order to see
something on a tile. This file's pages are nearly empty; its only property of
interest is the absence of a key.
