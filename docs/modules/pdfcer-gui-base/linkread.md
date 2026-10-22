# `linkread` — a link's target and border, read from the file

The Properties link section seeds its form from what the link carries now.
`pdfcer_core::annot::Annotation` carries no target or border in a form the
link verbs accept, and `Destination::Named` appears only for names the engine
could not resolve, so a resolvable named link reads back as a page. This
module reads the raw dictionary until the engine offers a reader (request
G178), and is deleted with that answer.

# Contract

- `read(session, id, pages) -> Option<LinkReading>`: `None` when `id` is not a
  dictionary. `pages` is the document's page object ids in page order.
- `LinkReading{goes, border}`.
- `Goes`, in this order of precedence:
  - `/Dest` a name or string → `Named(bytes)`.
  - `/Dest` an array whose first element is a page reference → `Page(index)`,
    the reference's position in `pages`.
  - `/A /S /URI` → `Uri(string)`.
  - `/A /S /GoTo` → its `/D`, read as `/Dest` is.
  - Any other `/A` → `Other(S)`, with `S` empty when the action has no `/S`.
  - Nothing usable → `Nowhere`.
- `Border{width, colour, dashed}`, or `None` for an invisible border:
  - The width is `/BS /W`, else `/Border[2]`, else 1 pt.
  - A zero width, or a `/C` that is present and empty, reads as `None`.
  - `colour` is `/C` with 1 (grey), 3 (RGB) or 4 (CMYK) components, kept in
    that space so re-applying a border does not rewrite a CMYK ink as RGB;
    `None` when absent or another length.
  - Dashed means `/BS /S /D`, or a `/Border` array with a fourth (dash)
    element.
