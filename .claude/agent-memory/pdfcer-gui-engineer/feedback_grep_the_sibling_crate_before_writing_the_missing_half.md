---
name: grep-the-sibling-crate-before-writing-the-missing-half
description: A crate's "we never do X, the caller does" is about THAT crate — a sibling probably already implements X; a mirrored enum across the boundary is the tell
metadata:
  type: feedback
---

Before implementing the caller's half of a documented seam, grep every
**sibling** crate for the operation by name. A crate saying *"we never go
looking, the shell resolves it"* is stating a rule about itself, not a claim
that nothing in the workspace does it.

**Why:** on 2026-08-28 I wrote ~300 lines of font-donor resolver in the shell
because `pdfcer-core` says *"the shell resolved for it"* and *"pdfcer never goes
looking"*. `pdfcer_render::FontEnvironment::resolve_for_embedding` already had
the whole thing, three rungs deep, with the exact case (`Helvetica` → `Arial`)
in its own doctest. My version matched on names only, so it found nothing on
every CAD drawing — no Windows machine has a font called Helvetica — and it
shipped for one commit with eight green tests. The rule stated was *"the shell
owns the **filesystem**"*; I read it as *"the shell owns **resolution**"*.

**How to apply:** the tell is a **mirrored enum across the crate boundary** —
`pdfcer_core::FontMatch` and `pdfcer_render::EmbedMatch`, same three variants,
with a doc comment saying a shell converts between them in one line. Nobody
defines a three-variant provenance enum for a rung nobody computes. Second
tell: if every test of a lookup asks for a string its own index literally
contains, the tests cover the map and not the resolution — add one input the
data source does not hold.

Legitimate reasons to still wrap rather than delegate wholesale: the engine's
type is missing something the operator is owed (a source path), or its grading
is coarser than a disclosure needs. Wrap and re-grade; do not re-implement.

See [[the-engine-session-runs-in-parallel]] and
[[a-backlog-row-is-a-record-not-evidence]].
