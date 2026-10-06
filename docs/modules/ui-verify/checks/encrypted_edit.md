# `ui-verify/checks/encrypted_edit`

`an_encrypted_file_is_edited_under_the_password_it_opened_with` — a file whose
password grants editing is edited and saved still encrypted; a file whose
password does not is refused with the password's own sentence and a button
that reopens it for its owner password, after which the edit lands.

# What it drives

Two launches under `--no-input` with a `ScriptedPointer`, each on a scratch
copy (ignores `--pdf`). The viewport is 2800 wide so the File tab draws
Unlock and Sign without overflow.

**granted** — `fixtures/encrypted-aes-128.pdf`, opened with `userpw`, which
grants every permission.

1. The password prompt is answered (`password.field`, typed, `password.open`).
2. `rail.rotate.pages.rotate_right` gives `rotate-pages`.
3. Ctrl+S gives `save-in-place outcome=ok`; the copy is longer and still names
   `/Encrypt`: an appended revision under the file's own encryption.

**print_only** — pdfcer's
`fixtures/synthetic/encryption/enc-emptyuser-print-only.pdf`, which opens with
no password and grants only Print; its owner password is `ownerpw`.

1. The File tab declares `ribbon.item.file.unlock` (`doc.locked`).
2. Sign traces `sign-opened refusal=encrypted-password`; Escape closes it.
3. The rotation gives `rotate-pages-refused` and
   `edit-encrypted-refused cause=password` (`app::unlock::record`, from the
   funnel's error arm).
4. `status-group:decline.remedy` is clicked: `Declined::PasswordRefused` names
   `file.unlock`, which traces `unlock-reopen`.
5. The prompt is answered with `ownerpw`; the rotation gives a new
   `rotate-pages`.

# Why the engine's cause and not the message

The verb's error says only that encryption refused it.
`EditSession::encryption_refusal_cause` says which of the two causes holds,
and each has its own remedy: Allow edits under RC4, or Unlock.

# Falsified

- Without `unlock::record` in the funnel's error arm, step 3 of print_only
  fails: no `edit-encrypted-refused`.
- With `file.unlock`'s dispatch a no-op, step 4 fails: no `unlock-reopen`.
