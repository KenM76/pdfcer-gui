# `pdfcer-gui-base/imposition`

## Why sheets, not plans

The print dialog's job is a list of sheets. Imposition replaces it with one
entry per printed side, so the stepper, the sheet count, the commit and the
spool keep walking sheets and need no second path.

## Why copies come after imposition

The engine's job sequence already repeats pages for copies. Imposing that
sequence would put two copies of a page on one sheet, so imposition takes the
sequence once (`sequence_once`) and repeats whole sheets (`with_copies`). A
booklet is always collated: uncollated copies would pair each front with the
wrong back.

## Why blank booklet sides are kept

On a duplex printer, sides alternate front, back. A side the fold leaves
empty still consumes its face of the paper; dropping it would shift every
later page onto the wrong sheet.
