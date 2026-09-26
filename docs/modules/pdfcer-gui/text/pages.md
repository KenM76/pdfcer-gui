# `text::pages` — every string the Pages panel shows

One area of the catalog described in [`crate::text`]'s header, consumed by
[`crate::panels::pages`] — the grid, its captions and its tile states — and
by [`crate::app::actions::pages`], which words what a page **delete** broke.
Those are the only two readers.

The second joined the first rather than getting a module of its own because
both are sentences about *pages* in the same vocabulary — sheets, page
numbers, this document — and a reader who came here for one half would not
find the other. See the disclosure section at the foot of this file for the
rule-4 obligation those strings discharge.

It is a sibling of [`crate::text::panels`] rather than a module inside it
for the same reason [`crate::text::forms`] is: that directory's own header
declares it covers *"the three document-structure panels"* and their two
inspector siblings, and the Pages panel is neither. It is a **navigator**
whose copy is about pictures, page geometry and the cost of drawing —
vocabulary that has nothing in common with a font inventory or a signature
byte range, and that would be read past by anyone maintaining either.

## ★ The posture: an undrawn thumbnail must SAY it is undrawn

This is the whole reason half the strings below exist, and it is the
project's no-placeholders rule (`RIBBON_IA.md` P3) applied to a picture
rather than to a control.

A page thumbnail that has not been rasterized yet is, on screen, a
rectangle. A rectangle the colour of paper **is a picture of an empty
page** — and an empty page is a thing a real PDF can contain. So a
thumbnail grid that draws blank rectangles while it works is not
"loading"; it is *asserting something false about the document*, and the
operator has no way to tell the two apart. The old shell drew exactly that
(`main.rs`'s `thumbnail_rail`: a bordered rect in `extreme_bg_color` with
the page number), and it is the one part of that rail this panel did not
carry across.

Every state a tile can be in therefore has **words**:

| State | String | Says |
|---|---|---|
| queued, previews on | [`thumbnail_not_drawn_yet`] | *this is not a picture of the page yet* |
| previews off | [`thumbnail_previews_off`] | *and it will not become one until you say so* |
| the render hit the time ceiling | [`thumbnail_abandoned`] | *pdfcer started and stopped* |
| the page would not draw | [`thumbnail_failed`] | *this page is the problem, not the panel* |

No spinner, and that is deliberate rather than an omission: a dozen
spinning icons is motion, not information, and only one page is ever
being drawn at a time anyway.

## Conventions, restated from [`crate::text`] because they bind here

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.**
- **Name the thing and what the operator can do about it.**
  [`previews_skipped_note`] is the worked example: it names the page, the
  limit it exceeded, and the box that changes that limit.
- **Never state a capability the build does not have.**
