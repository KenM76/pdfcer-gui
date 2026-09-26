# `ui-verify/checks/export_dxf`

`export_dxf_writes_the_pages_geometry` — the DXF reaches disk, and the file
agrees with what the shell said it wrote.

# The gap this closes

`file.export_dxf` was the first entry in `shell::commands::reach`'s
`SCAFFOLDED` list, with the recorded reason *"No recorded reason anywhere.
Scaffolded by omission, not by decision."* `pdfcer-core`'s `export::dxf` had
shipped the whole time and the **old shell has the feature**, so this was a
regression against `FEATURES.md`'s `gui` column rather than a gap.

# Why this needs driving

Because the interesting half is not the writer — that is `pdfcer-core`'s, and
it is tested there. It is the six links between a ribbon press and a file:

1. the arm builds a window and computes a scale suggestion from the page's
   **own** dimension groups;
2. the window raises an `Action` carrying a `DxfOptions`;
3. the apply arm fetches the page's decomposition **from the shared cache**,
   which may not be filled yet;
4. it calls the writer;
5. it opens a save dialog — a modal OS window, which is why this is an
   action at all;
6. it writes the bytes and discloses what was left behind.

Link 3 is the one that cannot be unit-tested: the decomposition is produced
by the canvas's own provider, keyed on `(page, epoch)`, and whether it is
populated at the moment an export runs is a question about a **running
frame**.

# ★ The assertion that makes this more than a smoke test

**The file on disk is counted, and the count is compared against the
trace.** The trace says what the shell believed it wrote; the file says what
was written. A build whose apply arm reported an outcome from one page and
wrote bytes from another — or wrote nothing and reported a success — passes
every other assertion here and fails this one.

It is the same shape as `pages_drag`'s caret-versus-gap cross-check, and for
the same reason: two values that are *supposed* to describe one thing are
exactly the pair a refactor separates.
