# `ui-verify/checks/harness`

## Item notes

### `fn out`

# Why the `create_dir_all` is here and not at every call site

[`crate::launch`] creates the parent of the trace file it is about to
open and [`crate::image`] creates the parent of a PNG it is about to
save, so a check whose first use of this directory is a launch or a
screenshot would work without this. But several checks write a
**fixture** into it *before* they launch anything —
`save_writes_over_the_file_you_opened` copies the document it is going
to overwrite, `redaction_removes_and_proves_it` writes the PDF it will
redact, and `insert_image_places_a_picture`,
`the_insert_window_steps_aside_so_you_can_point` and
`a_dropped_picture_lands_where_it_was_dropped` each write a PNG to drop
— and for those, nothing has created the directory yet when `--out`
points at a fresh per-check path.

**A path that cannot resolve produces a SKIP, and a SKIP is not a
failure, so a check can be dead for ever while the suite looks
healthy.** That is why the guarantee is a FUNNEL and not a
`create_dir_all` at each of the writing call sites: a sixth such check
added later inherits it, whereas a sixth call site has to remember.

# Why the error is swallowed

The return type is a path, not a result, and forty call sites read it in
expression position. A directory that genuinely cannot be created — a
read-only volume, a name that is not a directory — still produces an
error at the moment of the write, from the code that knows what it was
writing and can say so. Making this fallible would trade a precise
message at the write for a vague one here.

### `fn resolve_exe`

A default that does not exist is `None` rather than a path, so the SKIP
reason says "no binary" once rather than describing a path the caller
never chose.
