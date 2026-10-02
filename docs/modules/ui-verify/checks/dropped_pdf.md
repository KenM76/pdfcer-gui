# `ui-verify/checks/dropped_pdf`

`a_dropped_pdf_asks_open_insert_or_place` — a PDF dragged onto an open
document asks whether to open it, insert its pages after the page on screen,
or place its first page as artwork where it was dropped.

It drives the same scripted `drop` step as `dropped_file`, for that check's
reason: a real drop cannot be synthesised without the operator's mouse. The
dropped file is a two-page PDF of 144×72 pt pages the check writes, small
enough that its natural size fits the target page and the placement can be
measured against the drop point.

# The oracles

Each of the three drops must trace `drop-pdf-asked insert=true place=true`,
since Edit offers both.

- **Insert** (clicking `drop-pdf.insert`) must trace `insert-pages page=1 n=2`
  — both pages, landing at index 1 after page 0 where the document opened —
  and Ctrl+Z must trace `undo-applied`.
- **Place** (clicking `drop-pdf.place`) must trace
  `drop-pdf-chosen choice=place` with a 144×72 pt rectangle centred on the
  drop point, then `custom-stamp-placed`, and Ctrl+Z must undo it.
- **Open** is pressed with Enter in the window, which proves it is the
  default: a new `open` line must name the dropped file.
