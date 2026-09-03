---
name: pdfcer-is-multi-document-since-2026-08-20
description: pdfcer-gui holds several documents at once behind a tab strip; the active one is still PdfcerApp::status and everything else reads that field unchanged
metadata:
  type: project
---

**pdfcer-gui went multi-document on 2026-08-20**, on Ken's request: *"make it so
we can open multiple PDFs at once and drag and drop pages from one thumbnail
image sidebar to another or onto the canvas…"*

**Why it matters for anything you touch next:** the active document is **still
`PdfcerApp::status`**, unchanged, and every panel, the canvas, the status bar,
the condition set and the find bar read that one field exactly as before.
`parked: Vec<Status>` and `active_slot: usize` sit beside it, and only
`app/documents.rs` knows the encoding. So code written against the
single-document model is still correct — do not "modernise" it into
`documents[active]`, which cannot be split by the borrow checker at the ~105
sites that take `&mut self.status` alongside another field.

**How to apply:**

- Anything that must act on *another* document borrows `self.parked[i]` and
  `&mut self.status` as **two disjoint field borrows**, never through a method
  on `&self`. `app/actions/crossdoc.rs` is the worked example and the only
  edit in the application that reads two documents at once.
- Anything that must survive a **document switch** cannot live on
  `PanelsState` — `forget_document` is `*self = Self::default()`. See
  `crate::pagedrag`, which is in `egui::Memory` for exactly that reason.
- A cross-document page drag is a **copy**, deliberately: a move would be two
  commands on two undo stacks and no single Ctrl+Z could reverse it.
- Open and New **no longer prompt about unsaved edits** and that is correct —
  they add a tab and discard nothing, so the old prompt would have been a false
  statement. Only the two closes prompt.

**Still open at the time of writing:** quitting the window with unsaved
documents asks nothing (pre-existing, now across N documents); document tabs
cannot be dragged to reorder; parked documents keep their page rasters
unboundedly (deliberate — 877 ms per full-page render on the benchmark
drawing).

See also [[smoke-launch-offscreen-when-the-desktop-is-blocked]].
