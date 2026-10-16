# `checks::open_in_acrobat`

`open_in_acrobat_hands_the_file_over` configures a viewer and drives
*Open in Acrobat* twice on `dimension-scaled.pdf`.

The viewer is a stub compiled with `rustc` into the scratch folder. It is a
windows-subsystem program that writes its arguments to the file named by
`UIV_ACROBAT_STUB_RECORD` and exits. Its path is written to the profile's
preferences as `acrobat_path`. The profile's preferences are reset when the
check ends, on every path out.

1. **Cancel.** Pressing the File tab's trailing item must trace
   `acrobat-asked` and no `acrobat-launched`, because the command asks before
   it acts. Cancel must then trace `acrobat-cancelled` and no `close`.
2. **Proceed.** The check reads the `acrobat-asked` line first. Unless it
   names `source=Configured` and the stub's path, the check stops with an
   error and the proceed button is never pressed. A profile that fell back to
   discovery would otherwise launch the machine's real Acrobat on the desktop
   the check runs on. After proceed it requires `acrobat-launched`, then
   `close`, and the stub's recorded argument must canonicalise to the
   document's own file. This run has no edits, so the question must be
   `prompt=ConfirmClose` and the file must not gain `/Rotate 90`.
3. **Save first.** The same steps, launched with `pages.rotate_right`. The
   question must be `prompt=SaveFirst`, and the file the stub is given must
   carry `/Rotate 90`: the save happened before the hand-over.

Falsified with three planted defects:

- Cancel closing the document fails step 1.
- Proceed dropping the close fails step 2.
- Proceed skipping the save fails step 3.

No `rustc` on the PATH is an error (SKIP), not a failure.
