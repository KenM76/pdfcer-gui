# `ui-verify/checks/export_form_data`

`exporting_form_data_writes_a_file` — **press Export form data and a file
appears on disk with the form's values in it.**

# What this is for

`file.export_form_data` was registered, drawn on File ▸ Export, and **inert
for the whole life of the project**, behind a `SCAFFOLDED` entry claiming
*"blocked on a writer that does not exist"*. Three writers exist and two
have since `Pass 7.1`. It was wired on 2026-08-27, and this is the check
that keeps it wired.

## ★★★ Why the oracle is a FILE and not a trace line

Every other link in this chain can be asserted from the trace, and asserting
only those would leave the one that matters untested. An export's whole
product is a file somebody else opens. A build that computed the bytes,
traced `export-form-data fields=3`, and then wrote them nowhere — a
swallowed `Err`, a path that was never joined, a picker whose answer was
dropped — would satisfy a trace-only check completely and would ship an
Export button that exports nothing.

So this reads the file back and asserts on its **contents**: it must be
FDF, and it must contain the field the check itself just filled. The
second half is what distinguishes *"a file was written"* from *"the
operator's data was written"*, and they are not the same claim.

## ★★ The picker is answered, not clicked

`PDFCER_DIAG_SAVE_PATH` supplies the save dialog's result. That is the same
seam `save_copy` uses and its header carries the argument: a native modal
blocks the thread, so a harness that tried to drive it would be automating
the operating system's file dialog rather than this program.

★ What that costs is stated rather than hidden: **the dialog itself is not
covered here.** Its title, its suggested filename and its extension filter
are unasserted, and a build whose picker opened in the wrong directory would
pass. That is the same gap `save_copy` records, for the same reason, and it
is the price of not automating a foreign window.

## The format is the extension, so the extension is the test

There is no format dialog — the operator types `.fdf`, `.xfdf` or `.csv` in
the picker and the extension decides. So driving the picker with a chosen
extension is driving the format selector, which is why this check can cover
the branch at all without a second surface to click.
