# `pdfcer-gui/app/prefs/file`

## Item notes

### `fn parse`

# The `match` is the file format

There is no key table, no `HashMap` and no derive: every key this build
understands is an arm below, and the `_` arm reports everything else as
[`PrefNote::UnknownKey`] and **keeps it in the file**. That last part is
what makes it safe for an operator to run two versions of pdfcer out of
one `userdata` folder — the older one does not delete the newer one's
settings on its next Save, because [`Self::write_to_string`] writes what
this build knows and the loader never rewrites on load.

The honest limit of that: an unknown key survives until the operator
presses Save in the older build, which writes a fresh file from the
fields it has. Preserving unknown lines across a *write* would mean
carrying them on `Prefs`, and a struct holding values it cannot use is
worse than the narrow case it protects.

### `fn write_to_string`

Commented, because the file is meant to be opened in a text editor and
a bare `render_quality = faster` tells an operator nothing about what
else they could write. Same posture as the engine's store, which spends
a comment block per key for exactly this reason.
