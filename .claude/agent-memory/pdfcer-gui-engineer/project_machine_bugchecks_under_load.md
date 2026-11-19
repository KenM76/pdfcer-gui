---
name: project-machine-bugchecks-under-load
description: The PC has bugchecked six times 2026-11-06..19 with four codes, one 0x12B FAULTY_HARDWARE_CORRUPTED_PAGE and one while idle; mixed RAM kits; flaky compiles may be hardware
metadata:
  type: project
---

Bugchecks (System log, WER-SystemErrorReporting id 1001; list them with that filter):
0x1A MEMORY_MANAGEMENT 41790 ×3 (2026-11-06 09:32 and 23:18, 11-07 14:38), 0xD1 (11-07 08:39),
0x12B FAULTY_HARDWARE_CORRUPTED_PAGE c00002c4 (11-13 13:17), 0x7E access violation (11-19 09:56,
with the session idle, no build or sweep running). 0x12B is Windows finding a flipped bit in a
page it had zeroed: the strongest RAM evidence of the six. No WHEA events. RAM is two mixed kits
(2x16 GB Corsair CMW32GX4M2E3200C16, 2x8 GB G.Skill F4-3200C16-8GTZR) at 2133. A rustc ICE and a
flaky `zerocopy` compile came the same days. The minidumps in C:\Windows\Minidump need admin to read.

**Why:** an unexplained crash in a build or a sweep may be the machine; the idle crash means it is
not only load, so any long job can be cut off.

**How to apply:** retry an ICE before chasing it; `git fsck` after a reset; keep sweeps resumable
(`sweep-full.sh --start=N`, re-derive N since a commit changes the `--affected` list). Ken was told
on 2026-11-19 that 0x12B points at RAM; a memory test or pulling the G.Skill pair is his call.
