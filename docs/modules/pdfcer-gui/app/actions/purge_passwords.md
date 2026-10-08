# `app::actions::purge_passwords` — Security ▸ Remove old passwords…

Writes a copy of the open document in which no password field holds a stored
value in any version of the file. The open document is not changed.

## Why a copy

A PDF saved incrementally (ISO 32000 §7.5.6) keeps every earlier version. A
password typed into a form, saved, and later cleared is still in the bytes,
where any text editor finds it. This shell's Save always appends, so an
undoable edit followed by Save cannot remove it. Only a rewrite as one
version, which also drops any object stream that held a superseded copy of a
purged object, does. That is a save transform, so it lives beside Encrypt in
the band whose commands produce a new document.

## Pipeline

1. **Refuse a signed document.** A one-version rewrite changes every byte a
   signature covers. This is the same policy Encrypt follows.
2. **Serialize as Save would** (`to_incremental_bytes` with the settings'
   save options), so that unsaved edits are included.
3. **Scan every version** with `pdfcer_core::password_history::scan_stored_password_values`.
   If nothing is found, report that and write nothing. Unreadable versions are
   counted and disclosed; the rewrite drops them regardless.
4. **Purge a fresh session** built from those bytes through
   `SettingsExt::open_session`, calling `EditSession::purge_password_values`.
   The engine's fill guards refuse encrypted and certified documents; that
   refusal reaches the operator as a failure note.
5. **Rewrite as one version** with `to_full_bytes_decomposing_containers`.
6. **Re-scan the output.** A copy that still holds a value is not written.
   This is the guarantee the command makes, so the command checks it itself.
7. **Pick a path** (defaulting to `<stem>-no-passwords.pdf`) and write it.

## Counting

The receipt counts **values removed**, split between earlier versions and the
current one, taken from the scan. It does not count the engine's
`fields_purged`. A value that exists only in an earlier version leaves the
current field empty, so the engine clears nothing and the rewrite alone
removes it. A receipt counting cleared fields would say "0" for exactly the
case the command exists for.

The receipt discloses read-only fields that were cleared, values inherited
from a shared parent field (the engine leaves those in place), and unreadable
versions. It never quotes a value.

## Trace

| Line | When |
|---|---|
| `purge-passwords fields= earlier= latest= revisions= unreadable= read_only= inherited= appearances= containers= promoted= after_revisions= path=` | written |
| `purge-passwords-none revisions= unreadable=` | nothing stored |
| `purge-passwords-refused reason=signed signatures=` | signed |
| `purge-passwords-failed reason=serialize\|reopen\|purge\|write\|write-file\|still-present` | failed |
| `purge-passwords-cancelled` | picker dismissed |

Verified by `ui-verify` check `stored_passwords_removed_without_the_mouse`.
