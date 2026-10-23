# `printclaims` — what a document says about how it prints

Two booleans, read from the object graph as a document opens, for the
overprint-preview default ([`prefs::overprint`](prefs/overprint.md)).

**This module is a workaround.** The engine answers neither question; both are
filed (`request_detect_pdfx_claim.md`, `request_document_uses_overprint.md`).
When the engine's readers land, delete the module and call them.

## `claims_pdfx`

True when the catalog's `/OutputIntents` holds an intent with `/S /GTS_PDFX`,
or the Info dictionary has `/GTS_PDFXVersion` (ISO 15930). A **claim**, not a
validation: a file saying PDF/X is answered as PDF/X, which is what Acrobat's
"Only for PDF/X files" reads too.

## `asks_overprint`

True when any `/ExtGState` reachable from a page sets `/OP true` or
`/op true` (ISO 32000-1 §8.6.7). The walk:

- a page's `/Resources`, or the nearest ancestor's up `/Parent` (§7.7.3.4,
  at most 32 levels);
- each resource dictionary's `/ExtGState` entries, then its `/XObject`
  entries that are `/Subtype /Form`, recursively;
- a visited set on references, so a form drawing itself terminates;
- a budget of `VISIT_BUDGET` (20,000) dictionaries, past which it answers no.

**Not searched:** annotation appearances, patterns, Type 3 glyph procedures,
and graphics states set with an inline `gs` name absent from resources. A
document whose only overprint lives there opens with *Automatic* off; turning
the preview on by the View button still draws it. The engine request asks for
a reader that covers every stream the renderer draws.

## Cost

Only reached under `pdfx` or `automatic`. One dictionary lookup per visited
object; on a single-page drawing it is microseconds, and the budget bounds a
pathological file.
