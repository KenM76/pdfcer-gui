# `text::panels::objects` — the Objects panel, and how a page object is
described

Two jobs, and they are one module because the second is only ever read
through the first and through [`super::properties`]:

1. The Objects panel's own chrome — intro, summary, empty states, tree
   row labels and tooltips.
2. **The wording of every fact in
   [`crate::panels::objects::summary::ObjectSummary`].** That type
   deliberately holds no prose at all; it classifies, measures and counts,
   and this module alone renders. The split is what makes the classifier
   unit-testable without an egui frame, and what makes a copy edit a
   one-file change.

## One description, several renderings

[`object_row`] and [`super::properties::property_rows`] are two
**renderings of one record**. A path's fill colour is therefore never
described one way in a tree row and a different way in Properties, and
that is structural rather than careful: both read the same
`ObjectSummary`, and the colour resolution ("which colour does a viewer
actually SEE for this paint disposition?") happens once, in
`summary::describe_object`.

## [`object_note`] is rule 4's disclosure, in words

Every one of these sentences is a fact `pdfcer-core` already computed and
never showed: `TextObject::approximate`, `PaintStyle::is_invisible`, an
exact zero-extent comparison on the bbox. **None of them is a guess**,
which is what makes surfacing them a disclosure rather than an inference
the operator would have to review.

Each sentence says WHAT is true and WHY the operator is seeing what they
are seeing, because "approximate bounds" on its own is a label, not an
explanation — and an explanation is the entire deliverable here. They are
long, and that is deliberate: they answer the operator's own report,
*"sometimes I click and get a box highlighting on the screen that doesn't
seem to correspond to anything."*

**They live in a panel, and nowhere else.** The disclosure rule is
explicit that inference reporting belongs off-canvas — a status line, a
results panel, a properties field — and equally explicit that no badge,
tint or dashed outline may be drawn into the page view. A panel is the
right home; this module supplies its words and nothing else's.

## What changed at salvage

- **`object_detail` and its three helpers are private here too**, exactly
  as they were. [`object_row`] is the public surface; the Properties
  panel asks for the same facts one at a time rather than reusing the
  joined clause, because a vertical list and a one-line row want
  different punctuation and joining them would give one of the two the
  wrong shape.
- **The row-label copy no longer promises Shift+click**, and
  [`objects_dock_intro`] no longer promises clicking selects anything on
  the page. There is no selection model at S3. The old sentences return
  with the selection they describe; stating them now would be a control's
  documentation shipping ahead of the control, which is the defect the
  old shell's own panel header records twice.
- **[`objects_dock_summary`] counts kinds** rather than counting a
  selection, for the same reason, and reads a
  [`SelectionCensus`](crate::panels::objects::summary::SelectionCensus)
  so the selection form is a caller change and not a copy change.
