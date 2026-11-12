# `panels::tags` — the structure tree, read-only

**Purpose.** Show the document's tags — what a screen reader follows — and
let the operator see which content each one owns (O410, X077).

**Shape.**

- A summary: the element count and any engine diagnostic (elements met
  twice, pages that did not resolve, types with no role mapping).
- The tree, flattened each frame into the rows its open set shows, drawn
  through `ScrollArea::show_rows` so a large tree costs only the visible rows.
  Rows above depth 3 start open, which shows a table's cells; the operator's
  toggles are kept per document (`TagsUi`, keyed on the document serial).
- The picked element's facts: type, the written type when role-mapped, `/T`,
  `/Alt`, `/ActualText`, `/Lang` (or the inherited one, marked so), and for
  cells Scope, Headers, RowSpan, ColSpan; the pages it is outlined on, or the
  reason it has none.

**Picking** raises `Action::PickTag` with the element's boxes, turns to the
first page and reveals the first box (`RevealRect`, why `tags-pick`). The
canvas outlines every box on every drawn page (`canvas::tagoutline`).

**Termination.** Kids are followed only to a larger element index — the
engine's own rule — so a `/K` cycle ends, and depth stops at 200 with a note.

**Not built.** Clicking the page to select the deepest owning element.

**Trace.** `tags-panel state=<untagged|failed|read> elements=N` on change;
`tags-pick element= type= boxes= at=p:llx,lly,urx,ury;…` per pick, page
1-based, default user space. Row regions are `tags.row.<index>.<type>`.
