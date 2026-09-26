# `acrobat::windows` — the two things only a real machine can answer

Everything impure about O122 is in this file, and it is deliberately the
**least interesting** file of the three: it reads registry values, it
spawns a process, and it makes no decisions at all. Every decision —
which candidate wins, whether Pro beats Reader, which dialog the operator
sees — is in [`super`] and [`super::discover`], over values, where a test
can reach it.

See [`super`]'s §3 for the seam and §6 for why the registry is read by
`reg.exe` rather than by a crate.

## `CREATE_NO_WINDOW`, and the flash it prevents

`reg.exe` is a console program. A GUI process on Windows that spawns one
**gets a console window created for it**, on top of everything, for as long
as the child runs. Discovery runs when the shell starts and again whenever
the operator changes the setting, so without this flag pdfcer would blink a
black rectangle over the document at exactly the moments the operator is
looking at it — three times in a row, since three roots are consulted.

It is the kind of defect that never appears in a test, never appears in a
trace, and is reported as *"something flashes when I open a file"*.

## Why every read is `reg query … /ve` and never `/s`

`/ve` asks for one key's **default value** and nothing else. A recursive
or wildcard query would return a tree whose size is not under pdfcer's
control, on the start-up path, parsed by a function that would then have to
decide which of several values it meant. One key, one value, one line to
parse.

## The output format, and the one thing that is fragile about parsing it

```text
HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\Acrobat.exe
    (Default)    REG_SZ    C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe
```


It is fragile in one specific way and that is worth stating rather than
discovering: a value whose **content** contains the literal text `REG_SZ`
would be split in the wrong place. That cannot occur for these keys — they
hold file paths written by Adobe's installer — and the alternative
(`reg query /f`, or reading the raw registry) costs more than the risk.

## A missing key is not an error

`reg.exe` exits non-zero and prints `ERROR: The system was unable to find
the specified registry key or value.` when a key is absent. That is the
**ordinary** answer for a machine with no Acrobat, so it is mapped to
`None` and nothing is traced as a failure. Verified here: querying
`App Paths\AcroRd32.exe` on this machine exits 1, and this machine simply
has no Reader.

## Item notes

### `const APP_PATHS_ROOTS`

`HKCU` last rather than first, which is the one non-obvious entry: a
per-user registration is the least common by far, and the two `HKLM`
spellings are what a machine-wide Adobe installer writes. All three are
consulted because a per-user install is exactly the case a hard-coded
`C:\Program Files\…` would miss, and missing it is the failure §4 of
[`super`] exists to prevent.

The `WOW6432Node` mirror is separate rather than implied: a 64-bit process
reading `HKLM\SOFTWARE\…` does **not** see what a 32-bit installer wrote,
and Acrobat has shipped 32-bit for most of its life. `reg.exe` inherits the
bitness of the caller, so the mirror has to be named.

### `const CREATE_NO_WINDOW`

Spelled as a literal rather than taken from a crate because pdfcer-gui
depends on no Windows crate and is not permitted to gain one. The value is
fixed by the Win32 ABI (`processthreadsapi.h`) and has never changed.

### `fn reg_query`

Every failure is `None` and none of them is traced as a fault: `reg.exe`
missing, the key absent, the output unparseable and the value empty are all
the same fact from this module's point of view — *Windows does not register
that here* — and a start-up path that logged an error for the ordinary case
of "no Acrobat installed" would train a reader to ignore the log.

### `fn the_separator_is_whitespace_of_any_width`

The four spaces `reg.exe` prints on the machine the fixture came from
are not a documented promise, so nothing may split on a fixed width. A
build of Windows, a locale, or a longer type name that padded
differently would yield a parser that silently found no value — and "no
value" is indistinguishable from "no Acrobat installed", so the button
would simply never appear and nothing would say why.

### `fn output_with_no_value_line_yields_nothing`

`reg.exe` also exits non-zero in that case and [`reg_query`] returns
before reaching here, so this is the second of two guards. It is kept
because the exit code is the platform's promise and this is ours.

### `fn an_expandable_value_is_read_as_the_string_it_holds`

pdfcer does **not** expand the variable, and does not need to:
[`super::super::Registrations::exists`] will answer `false` for a path
with a literal `%ProgramFiles%` in it, so such a registration is
declined rather than launched. Saying so here is the honest thing —
this is a known, bounded gap, not an oversight — and the operator's
escape hatch is the Settings field.

### `struct Windows`

A unit struct with no state: every answer is read fresh, because the
operator can install Acrobat while pdfcer is running and a cached "no"
would outlive the fact it recorded.

### `fn value_from_reg_output`

Public to the crate rather than private so that its tests can be real: the
format is the thing most likely to be wrong, and it is the only part of
this file that can be tested without a registry.
