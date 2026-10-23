# `overprintseed` — the document reading the overprint default waits on

The two answers Settings ▸ Display ▸ *Use overprint preview*
([`prefs::overprint`](prefs/overprint.md)) needs from a document, read by the
engine's `pdfcer_core::prepress`.

## PDF/X — `claims_pdfx`

True when `pdfx_claim` finds a `GTS_PDFXVersion` (XMP `pdfxid:`, which is
how PDF/X-4 and later state their level, or the Info dictionary), or
`pdfx_output_intents` finds a `/GTS_PDFX` output intent. A **claim**, not a
validation. An intent alone counts: every PDF/X part requires one, and only a
file made for a press carries one. Read at once, on the UI thread: it is the
catalog, the Info dictionary and one metadata stream.

## Overprint — `walk_bytes`, `walk_here`, `Walked`

`overprint_use` reports whether a graphics state that a `gs` actually invokes
sets `/OP` or `/op` true, in page content, annotation appearances, form
XObjects, patterns, soft-mask groups and Type 3 glyphs. A state merely listed
in a resource dictionary does not count.

`Walked::asks` is:

| engine | `asks` |
|---|---|
| a request found | `Some(true)`, whether or not the walk finished |
| no request, walk complete | `Some(false)` |
| no request, something unwalked | `None` — unproven, never read as no |

A "no" parses every content stream, so `walk_bytes` runs it on a thread
(`overprint-walk`). The thread parses **its own copy of the bytes the tab
parsed**, with the tab's password (`OpenDoc::opened_with`) and
`LoadOptions`. It never touches `OpenDoc::session`, because a thread holding
the session's `Arc` makes every edit's `Arc::get_mut` fail until it ends.
The bytes, not the path: a tab can be read from a file other than the one it
is bound to. Bytes that will not parse again, or a thread that ends without sending, is `None` with the
reason in `unwalked`. `walk_here` is the same walk on the calling thread, for
a document with no file behind it (a new blank document).

## `OverprintSeed`

On `OpenDoc::overprint_seed`:

- `Settled` — decided; nothing waits.
- `Walking { answer, shown }` — the thread is running; `shown` is what the
  preview was seeded to, so a late answer does not override a switch the
  operator has made.
- `Unknown { unwalked }` — the walk ended unproven; *Automatic* left the
  preview on and the status line says so.
