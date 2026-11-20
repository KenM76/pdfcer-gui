# `straighten_scans_stores_the_scan_as_chosen`

**Defect it guards.** Straighten scans' *Store the straightened scan* choice
does not reach the engine: JPEG is chosen and the saved scans are not JPEG, or
*As the original* is left and a Flate scan comes back JPEG.

**Fixture.** Two copies of `fixtures/skewed-scans.pdf`, whose scans are
Flate-compressed 8-bit grey and two of whose four pages are tilted. Opened off
the desktop through the scripted pointer, with `PDFCER_DIAG_SAVE_PATH` naming
each launch's saved copy.

**Steps.**

1. Launch one: File ▸ Straighten scans, Straighten with the defaults. The
   `deskew-started` line must read `codec=keep`. Save a copy; it must hold no
   `/DCTDecode`.
2. Launch two: the same, clicking `deskew.codec.jpeg` first. The start line
   must read `codec=jpeg`. The saved copy must hold at least one `/DCTDecode`
   per straightened page (two).

**Falsified** by passing `DeskewEncoding::SameFamily` in
`app::actions::deskew::page_turn` regardless of the window: the JPEG run's copy
held 0 JPEG streams.

**What it does not prove.** The JPEG quality value's effect on the bytes, the
*Without loss* choice, and a JPEG source staying JPEG under *As the original*.
That last is the engine's `SameFamily` contract, owned by its tests.
