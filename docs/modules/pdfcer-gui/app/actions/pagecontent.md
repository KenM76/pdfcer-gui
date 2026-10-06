# `app::actions::pagecontent` — another PDF's page drawn into a page's content

`place(doc, file, source_page, page, rect)` is the apply half of
`VectorAction::PlacePageContent`, raised where the mode edits content:

- Ctrl+V of a PDF another copy put on the clipboard (`dispatch::ospaste`'s
  `drawing`, which a snapshot box's copy writes);
- *Place its first page here* in the window a dropped PDF opens
  (`dialogs::drop_pdf`).

Where the mode only authors markup, the same two routes place a stamp through
`customstamp::place`. Markup ▸ Paste picture as stamp always places a stamp.

## What it does

1. Loads the source outside the funnel. An unreadable file declines with
   `OsPasteRefusal::SourceUnreadable` (nothing was edited, so it is a ⊗ line,
   not a ⚑ one) and traces `page-content-refused`.
2. One `vector_edit` labelled `place-page-content` around
   `EditSession::place_page_content`: one undo entry,
   `CommandKind::PlacePageContent`. `vector_edit`, not the page-narrowed
   variant, because the verb imports a resource closure into the document's
   object table.
3. Traces `page-content-placed page= form= content= scale-x= scale-y=
   distorted= imported= annots-ignored= widgets-ignored= group=`.
4. When the edit landed, selects the page's last object (`picture::select_newest`):
   the engine appends `q … cm /Fx Do Q` as a new content stream, so the form
   is last in paint order and the next press moves or resizes it.

## Disclosures

| Fact | Sentence (`text::pagecontent`) |
|---|---|
| `distorted` | `stretched(scale_x, scale_y)` |
| a scale more than 0.05 % from 1, undistorted | `scaled(scale_x)` |
| `source_annotations_ignored > 0` | `comments_left(n)` |
| `source_widgets_ignored > 0` | `fields_left(n)` |

A paste and a drop both place at the source crop box's natural size, so the
scale sentence appears only when the rectangle was clamped to fit the page.

## Not compensated

A `/Rotate` page receives the artwork in unrotated user space, as `add_image`
does: content is drawn in the page's own coordinates, and a stamp's upright
turn (`customstamp`) is a property of an annotation's appearance, not of page
content.

Driven: `a_snapshot_pastes_back_as_a_drawing` (Edit half) and
`a_dropped_pdf_asks_open_insert_or_place` (Place in Edit).
