# `acrobat::discover` — turning what the registry says into a path, and
deciding whether the path is an Acrobat at all

Three small functions, none of which touches a registry or a disk. They are
separate from [`super::resolve`] because they are the part that is *fiddly*
rather than the part that is *decisive*, and the fiddly part is where the
bugs live: a value read out of the registry is a string somebody's installer
wrote, and installers have written every shape of it.

## The three shapes, and where each one comes from

| Source | Example value | What has to be undone |
|---|---|---|
| `App Paths` default | `C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe` | quoting, whitespace, a trailing NUL |
| `App Paths`, quoted | `"D:\Apps\Acrobat\Acrobat.exe"` | the quotes |
| `shell\open\command` | `"C:\…\Acrobat.exe" "%1"` | the quotes **and** the argument template |

The third is the awkward one, and it has an unquoted spelling too —
`C:\Program Files\…\Acrobat.exe %1` — which cannot be split on whitespace
because the path contains some. See [`executable_from_command`].

## Why [`edition_of`] is a filter and not just a label

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

## Item notes

### `fn a_command_line_yields_the_executable_and_not_its_first_word`

The single most likely place Acrobat is installed contains a space, so
the naive `split_whitespace().next()` yields `C:\Program`. That failure
presents as "discovery does not work on any normal machine", which is
exactly the kind of thing a test is for.

### `fn edition_of`

Matched on the **file name only**, case-insensitively, because the
directory is exactly the part that varies: `C:\Program Files\Adobe\…`,
`D:\Apps\…`, a network install, a per-user install under `AppData`. The
file name is fixed by Adobe and is what `App Paths` is keyed on.

A path with no file name — `C:\`, or an empty string — is `None` rather
than a panic. This runs over values a person may have typed.

### `fn executable_from_registration`

The value is *supposed* to be a bare path, and usually is. It is
nonetheless unquoted and trimmed here, because:

- some installers quote it anyway, and a `PathBuf` built from
  `"C:\…\Acrobat.exe"` **with the quote characters in it** is a path that
  will never exist, so the button would silently never appear;
- `reg query` output carries whatever trailing whitespace the value had,
  and a `REG_SZ` written by a careless installer can carry an embedded NUL
  that survives into the string.

Returns `None` for a value that is empty once cleaned, because an empty
path is not a location and `Path::new("").exists()` is `false` on every
platform — a distinction worth making here rather than discovering as a
mysterious absence three functions away.

### `fn executable_from_command`

# Why this is not `raw.split_whitespace().next()`

Because the overwhelmingly common installation directory is
`C:\Program Files\…`, which contains a space. Splitting on whitespace
yields `C:\Program`, which exists on no machine, and the failure presents
as *"discovery does not work"* rather than as a parsing bug.

Two shapes are handled, in this order:

1. **Quoted** — `"C:\…\Acrobat.exe" "%1"`. Everything between the first
   pair of double quotes is the path. This is what every modern installer
   writes and what Windows itself requires for a path with a space.
2. **Unquoted** — `C:\…\Acrobat.exe %1`. Split immediately after the first
   case-insensitive `.exe`, which is the only reliable boundary available:
   the extension is the last thing before the arguments begin, and a
   directory named `…exe…` does not end a component with `.exe`.

Anything else — a command with no `.exe` at all, an empty string, a bare
argument template — is `None`. A wrong guess here is worse than no answer,
because [`super::resolve`] would carry it forward to a `Viewer` and the
operator would press a button that starts nothing.
