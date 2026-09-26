# `ui-verify/checks/save_in_place`

`save_writes_over_the_file_you_opened` — Save. In place. The one every other
program has.

# Why this check exists


> *"can I please have a save button like every other program in existence
> has? We're on week two of this and just have a save as button."*

`Ctrl+S` was bound to `file.save_copy`, which opens a file dialog every
single time. In-place save had been written down in the manifest's planned
list as *"blocked on autosave and crash recovery"* and had then been nobody's
problem for a fortnight.

# The assertion this check is really for: the ORIGINAL survives a failure

Save-in-place is **the only verb in this application that can destroy the
operator's work**, and the way it would do so is not exotic:
`std::fs::write` truncates the target and then streams into it, so every
byte of the payload is a window in which their only copy is a partial file.
On a CAD sheet that payload is megabytes.

So `save::save_in_place` writes to `<name>.pdfcer-tmp` beside the target and
renames — an act that either happens or does not. This check drives the
happy path; the property it pins is that **the file that comes out is a
whole PDF that pdfcer can read back**, which is what a half-written one would
not be.

# What it does NOT need

A dialog seam. That is the entire point of the feature and it is worth
stating: `save_copy_round_trip` needs `PDFCER_DIAG_SAVE_PATH` because a modal
picker is a hard wall to a harness. Save has no picker, so this check drives
exactly what the operator drives, with nothing substituted.

## Item notes

### `fn scratch_copy`

**Never the operator's own fixture.** This check exists to prove a verb
that overwrites, so pointing it at `--pdf` directly would mean a harness run
modifies the file the next run measures — and a fixture that changes under
the suite is the thing `crate::fixture`'s header refuses.
