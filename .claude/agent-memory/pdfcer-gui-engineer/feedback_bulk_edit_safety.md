---
name: a-safety-argument-for-a-bulk-edit-is-read-off-the-gate-not-recalled
description: Before a bulk rename or rewrite, read the header of the gate that measures it — that is where the last session's findings about exactly this operation live, and it will contradict what you remember
metadata:
  type: feedback
---

Before a bulk rename or bulk rewrite, **read the header of the gate that
measures the thing you are about to change**, and write the safety argument
from what it says. Do not write it from recall.

**Why:** shortening 141 memory filenames was justified in the script's own
docstring with *"`[[...]]` wikilinks resolve on a memory's `name:` frontmatter
slug, never on its filename"*. That sentence was confident, load-bearing, and
wrong — wikilinks resolve against **either** form. It broke 126
cross-references in one command. The correction was not somewhere hard to
reach: `check-memory-index.sh`, the gate run immediately afterwards, already
said *"resolve against a memory's `name:` frontmatter OR its filename stem,
and the folder uses both forms"* — and had even measured the split. A previous
session had paid for that sentence and written it down at the exact spot a
future session would stand.

**How to apply:** the tell is a docstring or commit message containing the word
*safe*, *harmless*, *nothing else references*, or *never*. Each is a claim
about every route into the thing being changed, and a bulk operation executes
before any of them can be checked. Two moves, both cheap:

- **Grep for the gate that owns this artefact and read its header first.** A
  mature gate's header is an accumulated incident record, not documentation.
- **Run the gate before the bulk operation as well as after.** A green
  before-run establishes that a red after-run is yours, which is the only thing
  that makes the recovery tractable — and here the after-run was the sole
  reason the breakage was found at all.

Related: [[an-absence-claim-is-a-claim-about-every-route]] (the same shape, one
level down — this is that rule applied to a *justification* rather than to a
finding), [[tidying-an-input-changes-every-instrument]] (why the sweep was
argued for at all), [[an-injected-file-is-a-dated-snapshot]] (what prompted
this one).
