# `acrobat::discover` — turning what the registry says into a path, and
deciding whether the path is an Acrobat at all

Three small functions, none of which touches a registry or a disk. They are
separate from [`super::resolve`] because they are the part that is *fiddly*
rather than the part that is *decisive*, and the fiddly part is where the
bugs live: a value read out of the registry is a string somebody's installer
wrote, and installers have written every shape of it.

## ★★ The three shapes, and where each one comes from

| Source | Example value | What has to be undone |
|---|---|---|
| `App Paths` default | `C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe` | quoting, whitespace, a trailing NUL |
| `App Paths`, quoted | `"D:\Apps\Acrobat\Acrobat.exe"` | the quotes |
| `shell\open\command` | `"C:\…\Acrobat.exe" "%1"` | the quotes **and** the argument template |

The third is the awkward one, and it has an unquoted spelling too —
`C:\Program Files\…\Acrobat.exe %1` — which cannot be split on whitespace
because the path contains some. See [`executable_from_command`].

## ★★★ Why [`edition_of`] is a filter and not just a label

It answers *"is this an Acrobat?"*, and the answer is `None` far more often
than it looks like it should be. `super`'s §4 records the measurement that
made this a filter: on the operator's own machine, `HKLM\SOFTWARE\Classes\.pdf`
reads **`OpenPDFStudio.pdf`**. The registered PDF handler here is not Adobe's
product at all.

A fallback that trusted the handler would therefore have put a button
labelled *Open in Acrobat* over a launcher for a competitor's editor — and
the operator would find out after pdfcer had already closed their document,
which is the worst possible moment to discover that a button meant something
else. So the handler is consulted for a *path* and then asked to prove it is
an Acrobat by its file name, which is the only evidence available without
reading version resources out of the binary.
