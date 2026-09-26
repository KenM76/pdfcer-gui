# `ui-verify/checks/unembed_fonts`

`removing_embedded_fonts_reaches_the_document` — **press Remove and the
font programs come out.**

# What this is for

`tools.unembed_fonts` was registered, drawn on the Tools tab and **inert for
the whole life of the project** — the last of ten scaffolded commands to be
wired, and the only one whose recorded blocker turned out to be *true*. It
said the confirmation window did not exist. It did not. It does now, and
this is the check that keeps the whole path wired.

## Why the oracle is `bytes=` and not `removed=`

A count of removed fonts is satisfied by a plan the shell built and the
engine executed on an empty selection — `removed=0` would be reported as
success by any assertion that only looks for the line. `bytes=` is the
engine's `bytes_reclaimable`, summed from each target's measured
`data_span`, and it is non-zero only when real programs were freed.

It is also the number this project has the **most** reason to assert on,
because it is the one pdfcer cannot deliver. `crate::app::save` writes
incrementally; §7.5.6's update section is appended, so the freed bytes stay
in the prior revision and the file gets *larger*. The window says so. That
makes `bytes=` a figure with two audiences and exactly one meaning, and a
check that never read it would let the meaning drift.

## Why this fixture, and why it is the same one the embed check uses

`a1-titleblock.pdf` carries three embedded TrueType faces, all subsetted —
`AAAAAA+JetBrainsMono-Regular` and two more — which is what a modern
producer writes and is the case with the most consequences at once: the
programs are removable, the subset tags come off, and the names change.

Running both font checks on one fixture is deliberate. They are inverses,
and a fixture that only one of them could use would mean the pair could
never be run as a round trip. (This check does **not** round-trip today —
see below.)

## What this check does NOT cover, stated rather than implied

- **The round trip.** Remove-then-embed in one session would be the
  strongest test of both, and it needs the harness to drive two commands
  with a click in each window. `PDFCER_DIAG_INVOKE` runs commands and the
  clicks are separate acts; sequencing them is a harness feature that does
  not exist. Named here so the gap is a decision rather than an omission.
- **The saved file.** Removal is one `EditSession` command and this asserts
  on the session's report. Whether the file on disk is larger afterwards —
  which it will be — is exactly what the window discloses and is not
  asserted, because pdfcer cannot currently make it otherwise.
