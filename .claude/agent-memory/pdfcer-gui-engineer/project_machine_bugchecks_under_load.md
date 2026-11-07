---
name: project-machine-bugchecks-under-load
description: The PC bugchecked 0x1A MEMORY_MANAGEMENT twice on 2026-11-06 under sweep and build load, with mixed RAM kits; flaky compiles may be hardware
metadata:
  type: project
---

Windows bugchecked 0x1A MEMORY_MANAGEMENT twice on 2026-11-06 (09:32 and about 23:15, the second during a desktop sweep). RAM is two mixed kits (2x16 GB Corsair CMW32GX4M2E3200C16, 2x8 GB G.Skill F4-3200C16-8GTZR) at 2133. No WHEA events. A rustc ICE and a flaky `zerocopy` compile came the same days. Suggests memory under load; not proven.

**Why:** an unexplained crash in a build or a sweep may be the machine, and a resumed sweep has to be planned for.

**How to apply:** retry an ICE before chasing it; `git fsck` after a reset; keep sweeps resumable (`sweep-full.sh --start=N`, and re-derive N, since a commit changes the `--affected` list). Ken was told and a memory test is his call.
