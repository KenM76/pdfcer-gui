---
name: disk-is-tight-and-target-grows-unbounded
description: D: runs near-full and this project's target/ reaches 50GB+ of stale build cache within a week; clear it periodically without being asked
metadata:
  type: project
---

`D:\` is a 954 GB volume that sits in the low-90s percent used. This
project's `target/` grows to **50 GB+ within about a week of active
work** and cargo never reclaims any of it, so the project alone can be
the difference between comfortable and out-of-space.

Measured 2026-08-21, after ~8 days of work: `target/` was 56 GB, of
which `target/debug/incremental` was 30 GB across **305 separate
generation directories** and `target/debug/deps` was 24 GB of which
20 GB had not been touched in three days. `target/release/deps` was
2.6 GB and comparatively well-behaved.

**Why:** cargo's incremental cache and dep artifacts are append-only in
practice — every rebuild writes a new generation beside the old ones and
nothing garbage-collects. Debug is the offender because the gates
(`clippy --all-targets`, `cargo test`) build debug constantly while
almost nothing *runs* debug; the ui-verify harness drives the release
binary.

**How to apply:** treat `rm -rf target/debug` as routine housekeeping,
not a destructive act — it costs one full debug rebuild and nothing
else. Do the same for `target/doc` (regenerable) and for any
`target/ui-verify-*` scratch directories older than the current line of
work; anything worth keeping was already copied into `evidence/`, which
is tracked and small. **Leave `target/release` alone** — a release
rebuild is expensive and the harness depends on that binary; selectively
deleting files out of `release/deps` risks a half-valid cache for a
~2 GB return that is not worth it.

Expect the reclaimed figure to come in **well under** what `du` predicts
— on 2026-08-21 `du` accounted 53 GB deleted and `df` showed 29 GB
returned, with no NTFS compression on the folder to explain it. Quote
`df` before and after, not `du`, when reporting how much was actually
freed.

One-shot Python patch scripts have twice ended up committed at the repo
root (`patch15.py`, `patch_pixels.py`, `.tmp_text.py`, removed
2026-08-21). If a scratch edit-applier is needed, write it under
`.tmpwork/`, which is gitignored.

**★★ Two DURABLE fixes landed 2026-08-27, so do not re-derive them:**

1. **`package-portable.py` prunes.** It had written a dated folder into
   `D:\builds` on every keeper build since 2026-08-13 and never removed
   one — **39 folders, 1.27 GB**. It now keeps the newest three (the two
   OneDrive slots plus one), by *modification time* rather than by name,
   and only touches `pdfcergui-*` because `D:\builds` is shared with
   ScripTree and with the engine's own packages.
2. **`[profile.dev] debug = "line-tables-only"`** in the workspace
   `Cargo.toml`. Measured before: 4.1 GB in `target/debug` against 1.3 GB
   in release, with a single **324 MB `.pdb`** as the largest file on the
   disk. After: **2.15 GB**, a 48 % cut, tests unchanged. It keeps panic
   backtraces with file and line — verified by falsifying a test and
   reading `ocr.rs:654:9` — and drops only what a step debugger wants,
   which nothing in this workflow uses. Release stays at `debug = 1`.

**★ When reporting what a rebuild costs, check whose it is.** On
2026-08-27 a rebuild appeared to consume 25 GB; `target/` accounted for
only 5.4 GB of it and `D:\Dev\pdfcer\target` had grown **5.4 GB in the
same window** because the engine session compiles in parallel. The rest
was neither. Measure the specific directories before attributing a
drop in free space to your own build.

Related: [[always-publish-the-latest-build-to-onedrive]] — the packaging
step needs a valid `target/release`, another reason not to clear it.
Related: [[the-engine-session-runs-in-parallel-and-answers-within-the-hour]] —
its `target/` is not yours to clear and its growth is not yours to report.
