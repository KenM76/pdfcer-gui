# `dialogs::sign::create_id` — Create a digital ID

The Sign window's first step for an operator with no `.pfx`. A button,
*Create a digital ID…*, sits beside *Choose certificate…* and opens a form in
the certificate section: name, organisation, e-mail, country, key
(RSA-2048 default, RSA-3072, ECDSA P-256), *also usable for encryption*
(RSA only; greyed with its reason for ECDSA), validity (1–100 years,
default 5), the password twice.

## Order of events

1. *Create and save…* compares the two passwords. A difference is refused
   here (`digital-id-refused kind=mismatch`); it is the one check the engine
   cannot make, because it is given one password.
2. `pdfcer_core::sign::digital_id::create_self_signed_id` runs on a worker
   thread. RSA generation takes seconds; the form shows a spinner and stays
   greyed. The start instant is `app::clock::unix_now` — the engine reads no
   clock.
3. Only after success is the save location asked
   (`files::pick_new_digital_id_target`). Generating first means every
   `IdError` — all of which the engine checks before generating a key —
   arrives before a save dialog, so the operator is never asked where to put
   a file that then is not made. A cancelled save discards the ID.
4. The `.pfx` is written; the window's `certificate` becomes its path, the
   password becomes the `passphrase`, and `open_identity` runs, so the
   identity read-back and the rest of the Sign form appear at once.

Each `IdError` has its own sentence (`text::digital_id::refusal`); the trace
carries a token per variant, never the sentence.

## After creation

The form shows the file name, key and end of validity, and the SHA-256
fingerprint as two lines of sixteen hex pairs — the form a recipient reads
back over the phone. *Copy fingerprint* puts it on the clipboard.
*Save the certificate to share…* writes `DigitalId::certificate` (DER, no
key) as `.cer` (`files::pick_shared_certificate_target`). *Done* closes the
form; the chosen certificate stays.

## Disclosure

The form says, above its fields, what a self-signed ID proves (the document
is unchanged; the signer held this file and password) and what it does not
(who the signer is, until the reader trusts the certificate).

## Privacy of the trace

`digital-id-requested key= years= encrypt=`, `digital-id-created key= years=
encrypt= bytes= fingerprint=`, `digital-id-shared bytes=`,
`digital-id-refused kind=`. The fingerprint is public by design. The holder's
name, the path and the password are never traced.

## Regions

`sign-create-id`, `sign-create-name`, `sign-create-password`,
`sign-create-confirm`, `sign-create-go`, `sign-create-error`,
`sign-create-fingerprint`, `sign-create-share`, `sign-create-done`.
