# `ui-verify/checks/security_passwords`

`security_notes_say_old_passwords_are_kept` — a form whose earlier saved
version stored what was typed into a password field says so in Document
properties ▸ Security notes, naming the field and the version and never the
value, and names File ▸ Security ▸ Remove old passwords… as the remedy.

# What it drives

`fixtures/password-history.pdf` (ignores `--pdf`; built by
`password-history.PROVENANCE.py`): one Password text field, saved twice.
Version 1 stores `/V`; version 2 is an incremental update without it, so a
reader shows an empty field while the value stays in the bytes.

Launched with `PDFCER_DIAG_INVOKE=file.document_properties`
(`security_notes::launch_on`).

1. Owed: `security-passwords revisions=2 earlier=1 current=0`.
2. Owed: a visible `docprops.security-notes.passwords` region inside
   `dock.right.frame`'s right edge.

The region spans the dock's width whether or not the sentence wraps, so step
2 proves the lines are drawn and reachable, not that they wrap; a rendered
screenshot is the only oracle for that.

# Cost

The scan opens each saved version once. On the 5.7 MB benchmark CAD drawing
(one version) it took 7 ms, measured from the `ms=` field, so it runs on the
frame, once per document and base.

# Falsified

- With the lines suppressed, step 2 fails: no region.
- With `earlier` and `current` swapped in the trace, step 1 fails.

# Driven off-screen

Window at `-4200,-4200`; it runs under `--no-input`.
