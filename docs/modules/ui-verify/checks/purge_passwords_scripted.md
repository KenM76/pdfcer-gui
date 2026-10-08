# `ui-verify/checks/purge_passwords_scripted`

`stored_passwords_removed_without_the_mouse`: Security ▸ Remove old
passwords… writes a copy of `fixtures/password-history.pdf` that no longer
holds the password its first version stored. The window is off the desktop
and driven only through `ScriptedPointer`, so the check runs under
`--no-input`.

## Fixture

`fixtures/password-history.pdf` is built by
`fixtures/password-history.PROVENANCE.py`. It has one Password field, `Pin`.
Version 1 stores `/V (s3cret)`. Version 2 is an incremental update that
rewrites the field without `/V`. A reader shows an empty field, while the
bytes still hold the value.

## Steps

1. The save target is deleted first, so that an earlier run cannot pass for
   this one.
2. The check clicks `ribbon.tab.file`, then the collapsed
   `ribbon.group.file.security` if the band is narrow, then
   `ribbon.item.file.purge_password_values`. `PDFCER_DIAG_SAVE_PATH` answers
   the save picker.
3. A `purge-passwords` line must carry `fields=0 earlier=1 latest=0
   after_revisions=1`. `fields=0` is correct: the value sits only in the
   superseded version, so the current field has nothing to clear.
4. The written file must not contain `s3cret`, and it must have exactly one
   `%%EOF`.

## Falsification

This was falsified by writing the `to_incremental_bytes` output after the
command's re-scan. The copy then keeps both versions, and step 4 fails, naming
the file that still holds the value.

## What it does not cover

- the signed-document refusal;
- the encrypted-document failure;
- the read-only and inherited-value disclosures;
- values in object streams.

The engine's tests cover the purge on those shapes.
