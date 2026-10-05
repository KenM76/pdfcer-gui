# `ui-verify/checks/security_notes`

`security_notes_name_the_cover_and_the_actions` — a §7.6.7 wrapper says on
the status row, the moment it opens, that the visible page is a cover; and
File ▸ Document properties ▸ Security notes lists every action the file would
run in a viewer that runs them, the one reached only through `/Next`
included, inside the dock.

# What it drives

`fixtures/security-notes.pdf` (ignores `--pdf`), built by
`fixtures/security-notes.PROVENANCE.py`, with
`PDFCER_DIAG_INVOKE=file.document_properties`. No pointer and no keys: it
runs whole under `--no-input`.

1. `wrapper-disclosed payloads=1 named=1` (`app::opennotes::disclose`): the
   fixture's catalog `/AF` names one `/EncryptedPayload`, `drawing.pdf`.
2. `security-notes` (`panels::docprops::security`, traced when measured):
   `wrapper=1`, `truncated=false`, and at least one each of `page`,
   `outline`, `annot`, `js`, `chained`, `network` and `launch` — the page's
   open script, the bookmark's web link, the link's go-to and the launch
   chained behind it.
3. `ui-rect docprops.security-notes` declared visible, and its right edge
   inside `dock.right.frame`'s: a line that does not wrap is cut off.

# Falsified

- A build drawing the rows in an `egui::Grid` put the section's right edge at
  1243.6 against the dock's 1200.0 and failed step 3.
- Dropping the `wrapper::detect` call from `opennotes::disclose` leaves no
  `wrapper-disclosed` line and fails step 1.
- Counting `chained_actions` as zero fails step 2 on `chained`.
