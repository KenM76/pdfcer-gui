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
