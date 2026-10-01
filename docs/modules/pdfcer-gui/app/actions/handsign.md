# `app::actions::handsign` — writing the drawn signature into the page

`place` fits the mark into the box in **canvas space** (display-oriented, one
unit per point), then maps every point through `canvas_to_pdf_space`, so the
signature is upright on a rotated page. It writes one
`EditSession::add_markup_as_content` Ink call through the funnel: one undo
step, and content that renders exactly as it saves (R8b).

Why page content, not an Ink annotation or a `/Sig` value:

- it prints under every reader's default print scope, where an annotation
  depends on its print flag and the reader's settings;
- it can never be mistaken for a digital signature: no `/V`, no `/ByteRange`;
- the `/Sig` field is untouched, so a certificate signature can still be
  added later.

The ink is a fixed dark blue-black, a document colour rather than a theme role,
because it is written into the file. On success the field enters
`OpenDoc::hand_signed`, which hides its *sign here* tag (see the ledger in
`pdfcer-gui-base/handsign.md`).
