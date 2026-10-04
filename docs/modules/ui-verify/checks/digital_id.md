# `ui-verify/checks/digital_id`

`digital_id_created_and_chosen` — the Sign window creates a self-signed
digital ID, refuses two different passwords, makes the new file the chosen
certificate, and shares a `.cer` matching the fingerprint shown.

# What it drives

`fixtures/four-pages.pdf`, with `PDFCER_DIAG_DIGITAL_ID_PATH` and
`PDFCER_DIAG_SHARE_CERTIFICATE_PATH` naming files in the run's output folder
(deleted first).

1. File tab (expanding the Security group if collapsed), `file.sign`,
   `sign-create-id`; name `Jane Example`.
2. Password `pw-one`, confirmation `pw-two`, `sign-create-go`:
   `digital-id-refused kind=mismatch`, and no `.pfx` on disk.
3. Confirmation `pw-one`, `sign-create-go`: `digital-id-created` within
   about twenty seconds, then a `sign-identity` line and a live
   `sign-confirm` — the window opened the new file as its certificate.
4. Independent oracle: `certutil -p pw-one -dump` on the `.pfx` exits 0 and
   names the holder. Windows' own PKCS#12 reader, not pdfcer's.
5. `sign-create-share`: `digital-id-shared`, the `.cer` exists, and
   `certutil -hashfile … SHA256` equals the traced `fingerprint=`.

Fields are cleared with Ctrl+A before typing. Every `sign-` region is wheeled
into `sign-body` before it is clicked.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
