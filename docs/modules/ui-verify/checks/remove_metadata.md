# `remove_metadata_writes_a_copy_without_the_items_ticked`

**Defect it guards.** Security ▸ Protect ▸ Remove metadata (`OPERATOR_REQUESTS.md`
O289 item 21) does not list what the file carries, writes a copy that keeps a
ticked item or loses an unticked one, writes a copy that keeps the old values
in an earlier version, or changes the open document.

**Fixture.** `fixtures/four-pages.pdf`, which carries nine description
entries and twelve comments, copied and driven off the desktop with the
scripted pointer. `PDFCER_DIAG_SAVE_PATH` answers the save picker.

**Steps.**

1. Security tab ▸ Protect ▸ Remove metadata: `remove-metadata-listed` must
   name `info/Title`, `info/Author`, `comment/10-0` and `comment/11-0`.
2. Tick `info/Author` and `comment/10-0`; press Remove and save a copy; wait
   for `remove-metadata-wrote`.
3. Read the copy's bytes. It must not hold `/Author (OpenAEC Foundation)` or
   `/Contents (Construction drawing)`, must hold the Title entry and
   `/Contents (OA-2026-001-A100)`, and must have one `%%EOF`. Removing
   nothing, removing everything and saving incrementally each fail a
   different clause.
4. Open the window again: the open document must still list all four.

**Falsified** by removing every listed id instead of the ticked ones (step 3:
the Title is lost) and by writing `to_incremental_bytes` instead of the full
rewrite (step 3: the Author is still in the earlier version).

**What it does not prove.** The kind checkboxes and the other kinds' removal;
the engine's own tests own each carrier.
