# `ui-verify/checks/compact_save`

`a_compacted_copy_is_actually_smaller` — **the save that reclaims the space
a deletion freed.**

# What this is for

`OPERATOR_REQUESTS.md` **O48**, answered *"yes to all three"* on 2026-08-28.
It was raised by this project rather than by him, from a limit found while
wiring Remove-embedded-fonts:


§7.5.6's update section is *appended*, so every space-reclaiming operation
pdfcer has produced a file that was very slightly **larger**. Only a full
rewrite drops the bytes, and this is the command that asks for one.

## The oracle is the FILE ON DISK, and nothing else would do

Every link in this chain can be satisfied by a build that saves nothing.
The window opens on a serialisation, quotes a number, and hands bytes to an
action; a trace line saying `compact-written after=1048576` proves that a
`Vec<u8>` of that length existed. It does not prove a file was written, that
the file has those bytes in it, or that the operator can open it.

So this reads the file back and asserts three things about it, in order of
how badly each one fails:

| # | assertion | what its failure means |
|---|---|---|
| 1 | the file exists and is non-empty | the bytes went nowhere |
| 2 | it begins `%PDF-` | something was written and it is not a PDF |
| 3 | it is **smaller** than the original | the rewrite reclaimed nothing, which is the entire feature |

Assertion 3 is the one this check exists for and the one no unit test can
make: it is a claim about two real files on a real disk, produced by two
different code paths in `pdfcer-core` — the incremental writer that made the
fixture and the full writer that made the copy.

## The fixture IS the operator-visible problem, built by the CLI

A tidy file compacts to roughly its own size, and the window says so in words
— *"this file has nothing unused in it"* — which is a correct answer and
would fail assertion 3. So the fixture is one with real waste in it, and it
was made by doing to a drawing exactly what O48 is about:

```text
a1-titleblock.pdf                                    39,509 bytes
  embed-font --all-missing --apply       ->       1,708,740     (fonts appended)
  unembed-font --all-removable --apply   ->       1,709,629     (fonts removed)
```

**Look at the last line.** Removing 1.6 MB of font programs made the file
**889 bytes bigger**, because §7.5.6 appends the removal as a new revision
and leaves the programs in the old one. That is the sentence O48 was written
about, reproduced in a file, and it is what this check measures the fix
against: a compacted copy of `reclaimable.pdf` should be tens of kilobytes,
not 1.7 MB.

Built by `pdfcer` rather than by this check, and committed. A check that
manufactured its own multi-megabyte fixture on every run would spend most of
its wall clock building the thing it measures, and the two CLI calls are a
provenance line rather than a program.

## What this does NOT cover

**That the compacted copy renders identically.** It is a different byte
sequence for the same document and the engine's writer tests own that claim.
What is asserted here is that the shell reaches the full writer, keeps the
bytes it measured, and puts them where the operator asked.
