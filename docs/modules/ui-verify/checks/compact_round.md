# `a_compacted_copy_rounds_line_positions`

**Detects:** a Save a compacted copy window that rounds line positions by
default, offers no rounding list, does not pass the chosen distance to the
engine, does not say how many positions moved, or saves a copy different from
the one it measured.

## Drive

1. Open `fixtures/hole-in-a-big-object.pdf` in Edit and invoke
   `file.save_compacted`.
2. Wait for the first `compact-recompressed` line; it must read `rounded=0`.
3. Press the rounding list (`compact.content.round`), then its third entry,
   0.01 pt (`compact.content.round.2`).
4. Scroll the content sentence (`compact.content.said`) into view, then press
   Save (`compact.commit`) with `PDFCER_DIAG_SAVE_PATH` set.

## Oracle

The fixture's path coordinates carry four decimals (`516.5685`), so rounding
to 0.01 pt must move some and shrink the copy: `rounded>=1` and `bytes` below
the unrounded copy's. Rounding is lossy, so the default must be off. The file
Save writes must be exactly the byte count the window last measured; a stale
file from an earlier run is deleted first so it cannot satisfy that.
