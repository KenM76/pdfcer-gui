# Acrobat's form-dialog vocabulary — how to get it back

The two files this directory holds at rest — `strings.txt` and
`form-dialogs.txt` — are **not in git**, and the omission is deliberate. See
`.gitignore` for the reason in full: they are Adobe's own resource strings
lifted verbatim out of `AcroForm.api` and `AcrobatRes.dll`, and this repository
is public. `FORMS_PARITY.md` quotes the few dozen control labels a parity table
has to name; republishing a product's entire string table is a different act.

## Regenerate

```
python tools/acrobat-form-strings.py
```

Offline, no input device, no desktop, repeatable. It reads UTF-16LE runs out of
the installed Acrobat's binaries and writes both files here. Safe to run while
the operator is working — it opens no window and touches no document.

## Before following an `ACRO:` citation

`FORMS_PARITY.md` cites these files as `ACRO:strings.txt:<line>, offset <n>`.
**Those coordinates are Acrobat-version-specific.** The labels are stable across
versions; where they sit in the concatenated run is not.

`FORMS_PARITY.md` §10 records the version the table was measured against
(`AcroForm.api` 20,508,568 bytes, file version 25.1.20435.0). The tool writes
the version it found into `form-dialogs.txt`'s header. If those disagree, the
labels in the parity table are still right and the offsets are not — search for
the quoted label rather than seeking to the offset.

## What this evidence can and cannot support

**Can:** the complete set of control labels, kind names, enumerated values
(border styles, check-box glyph styles, calculation operators, icon layouts,
highlight behaviours) and Acrobat's own guidance sentences — all verbatim.

**Cannot:** which control sits on which tab, for certain. The resource table's
ordering is strong evidence and `FORMS_PARITY.md` §2 uses it, but it is
adjacency, not structure. That is what `ASK` row A1 is for; the tool's own
module docstring makes the same warning at greater length.
