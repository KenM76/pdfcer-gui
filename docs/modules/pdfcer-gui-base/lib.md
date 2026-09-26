# pdfcer-gui-base — the floor of the pdfcer-gui crate stack

Modules that reference no other module of `pdfcer-gui`, lifted out of
it so that the compiler, rather than a review, is what keeps them beneath
the application.

## What may live here

A module whose dependency count on the rest of `pdfcer-gui` is **zero**, as
measured by `python tools/module-graph.py`. That is the whole admission
test, and it is mechanical on purpose — "is this low-level enough?" is a
question two people answer differently, and answering it differently twice
is how the crate above ended up with 25 mutually recursive module pairs.

## What this crate is NOT

It is **not** `egui-shell`, and the two must not be confused when deciding
where something goes:

| | `egui-shell` | `pdfcer-gui-base` |
|---|---|---|
| may name a PDF concept | **never** — `check-shell-purity.sh` fails the build | yes; [`units`] converts against `pdfcer_core` |
| intended reuse | another application entirely | this application only |

A reusable, domain-free thing belongs in `egui-shell`. A pdfcer thing that
simply sits low belongs here.

## Why the callers do not mention this crate

`pdfcer-gui`'s crate root re-exports these names, so every one of the
roughly 1,200 existing call sites still spells them `crate::diag::…`,
`crate::units::…` and so on, unchanged.

That is deliberate, and it is not laziness. Rewriting an import line in
several hundred files is a wide, shallow, mechanical diff — the worst
possible neighbour for anyone editing the same files that day — and it buys
nothing, because **the boundary is enforced by the crate graph, not by how
a caller spells the path**. A module in here cannot reach up into the
application whatever the call sites look like: the code does not compile.

The one thing the re-export costs is that a reader of `crate::diag` cannot
see from the call site that it crosses a crate. `pdfcer-gui`'s crate root
says so at the re-export.
