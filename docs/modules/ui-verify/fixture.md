# `ui-verify/fixture`

What the harness knows about the document it opened.

## Why this module exists at all

[`crate::coords`] needs one number the application does not have to supply:
the **page height in PDF points**, for the y-flip. That number is a
property of the *document*, not of the application, so the harness can read
it itself — and doing so keeps the document-space contract real rather than
aspirational. A harness that had to be told the page size by the program
under test would be trusting the program to describe the coordinate space
the harness is checking it against.

## The MediaBox scan, and its stated limits

[`page_geometry`] scans the raw file bytes for the first `/MediaBox
[a b c d]` and reads the size from it. That is a heuristic, and here is
exactly what it does and does not handle:

**Handled.** The common case, by a wide margin: a `/MediaBox` written as a
direct array in an uncompressed object header, which is what every producer
this project cares about emits for the page tree root or the first page.

**Not handled**, each of which returns `None` rather than a wrong answer:

* a `/MediaBox` that is an indirect reference (`/MediaBox 12 0 R`);
* a page whose box lives in an object stream (compressed);
* documents whose pages differ in size — the *first* box found wins, which
  is right for a single-page fixture and wrong for a mixed one;
* a non-zero origin (`/MediaBox [10 10 622 802]`) is handled for *size* but
  the harness's document coordinates are relative to the box origin, which
  for a shifted box is not the same as the PDF origin.

Returning `None` matters more than the list. A wrong page height produces a
click that is vertically mirrored about the page centre — it lands on the
page, hit-tests something plausible, and the resulting failure looks like a
selection bug. `None` produces a SKIP that names the missing number, and
[`crate::checks`] callers can then be told the size explicitly with
`--page-size`.

This is the same discipline the rest of the crate applies to coordinates:
**refuse rather than guess**, because a confident wrong coordinate is more
expensive than no coordinate.
