---
name: disk-is-tight-and-target-grows-unbounded
description: Disk AND RAM are tight; clear debug/doc target routinely, run the suite with a job limit, wait for orphaned linkers after a kill; the BIGGEST disk item is one full engine source tree per pin bump under ~/.cargo; and 0xc0000142 / fork failures are usually HANDLE exhaustion by an orphaned process, not free RAM
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
rebuild is expensive and the harness depends on that binary. ⚠ The clause
that used to sit here, *"selectively deleting files out of `release/deps`
risks a half-valid cache for a ~2 GB return that is not worth it"*, was
**wrong on both counts and is corrected below** (2026-09-10): the return
was 3.76 GB and the prune is provably safe if cargo names the live set.

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

## ★ RAM is tight too — 2026-09-09

`cargo test --workspace` links its test binaries in parallel and each `link.exe`
sits at ~1 GB; six of them on the 16 GB machine left 2.2 GB free and the harness
**killed the background command for low memory** — while leaving cargo, rustc
and the linkers running as orphans whose output went nowhere. A killed shell is
not a killed build.

**How to apply:** run the suite as `CARGO_BUILD_JOBS=4 cargo test --workspace`
(or `-j 4`), and never chain gates → tests → release build in one background
command: each is a separate memory peak and one kill voids all three. Before
relaunching after a kill, `tasklist | grep -iE 'cargo|rustc|link'` and wait —
the orphans are finishing the work you need cached.

**The failure signature names the wrong culprit — 2026-09-19.** A full gate
sweep was killed for low memory during its last gate, `cargo clippy
--workspace --all-targets`, and what clippy wrote before dying was
`clippy-driver.exe ... (exit code: 0xc0000142, STATUS_DLL_INIT_FAILED)`,
four times over. That reads as a corrupt toolchain or a missing DLL and it is
neither: **0xc0000142 is what a Windows process reports when there is not
enough memory to initialise its DLLs.** Check free RAM before touching
`rustup`. Two further consequences: `run-all.sh` prints its SUMMARY only at the
end, so a killed sweep leaves **no tally at all** — fifty-nine green gates and
an unknown result are the same output; and a `clippy-driver` orphan survived
the kill holding 1 GB, so the relaunch must wait for it rather than race it.
Relaunch with `CARGO_BUILD_JOBS=2` for the sweep specifically — clippy peaks
higher than `cargo test`, and 4 was not enough head-room here.

**What the watchdog actually kills is the REBUILD, not the checks — 2026-09-19.**
Four background packaging runs in a row were killed for low memory while
4.3–3.1 GB was free, no orphaned `rustc`/`cargo`/`clippy-driver` was running and
every working set was far below that. The common factor was not the gates or the
tests: it was that each run followed a **commit**, and `build.rs` stamps the
commit into the binary, so every commit forces a full rebuild of the
167k-line `pdfcer-gui` crate — one `rustc` holding several GB. **How to apply:**
pre-compile by hand in the FOREGROUND Bash tool (600 s ceiling, not watched)
after any commit, then run the packager with `--no-build`; or detach it through
PowerShell `Start-Process`, whose children the watchdog does not reap — but read
[[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]] first,
because a different shell brings a different `PATH` and that changed which
`bash` the gates ran under.

## ★★★ The biggest item is NOT in `target/` — 2026-09-10

Every entry above treats `target/` as the problem. Measured on 2026-09-10 with
C: at **98 % full (23 GB free)**, the ranking was:

| Item | Size | Where |
|---|---|---|
| stale engine-pin source trees | **36 trees**, dominant | `~/.cargo/git/checkouts/pdfcer-*/` |
| `target/debug` | 8.0 GB | project |
| superseded `release/deps` artifacts | 3.76 GB | project |
| dead pre-rename `pdfce` git cache | 1.5 GB | `~/.cargo/git/db/` | <!-- old-name-exempt: the stranded cache directory is literally named for the pre-rename project; naming it is the whole point of the row -->

**Why this project specifically.** The engine is a `git+file://` dependency
**pinned by revision**, and the pin moves almost daily. Cargo materialises a
**complete working tree of the engine repo per revision** under
`~/.cargo/git/checkouts/<name>-<hash>/<rev>/` and **never** removes one.
`cargo clean` does not touch it — that command only knows `target/`. Thirty-seven
trees had accumulated. It is on **C:**, the tighter drive, and it is invisible
from inside the project.

**How to apply.** Only the revision in `Cargo.lock` is live —
`grep -A2 'name = "pdfcer-core"' Cargo.lock` gives it (`369d4de` on this date).
Delete every sibling directory; cargo re-creates one from the bare `db/` repo on
demand. Separately, `db/` is per **URL**, so it does not grow with pins but does
strand a whole bare repo when a dependency is **renamed** — the pre-rename
`pdfce` entries were dead for a week <!-- old-name-exempt: as above, this names a directory that exists on disk under the old name -->. Grep every `Cargo.lock` and `Cargo.toml`
for the old URL before dropping one.

### ⚠ The `release/deps` correction, and the method that makes it safe

The old advice not to touch `release/deps` was right about the *risk* and wrong
about the *return* and the *method*. **mtime is the wrong criterion** — cargo
legitimately reuses a third-party `.rlib` built weeks ago, so age-based pruning
forces a full rebuild of everything unchanged. Ask cargo instead:

```sh
CARGO_BUILD_JOBS=4 cargo build --release --workspace --message-format=json > live.json
```

Every unit emits a `compiler-artifact` line with `filenames` — **including fresh
ones** — so the union of their `-<16 hex>` metadata hashes IS the live set.
Delete any file in `deps/` whose hash is in none of them. Here that was **344
files / 3.76 GB**, dominated by **62 accumulated copies of `libpdfcer_gui`**, one
per pin bump. The proof it was correct: the very next
`cargo build --release --workspace` printed **`Finished ... in 1.47s`** — nothing
rebuilt — and an off-screen smoke launch drew a page at `covered=1.000` with no
panic.

### ⚠ And the `du`-vs-`df` note above is backwards for large deletions

2026-08-21 recorded `df` returning *less* than `du` accounted. On 2026-09-10 it
returned **far more**: C: 23 → 55 GB free after the pin trees, then 54 → **156
GB** after a 1.5 GB delete, and D: 94 → **260 GB** after removing 8 GB. NTFS
flushes large deletions lazily, so `df` lags by minutes and the delta attributed
to the last command is meaningless. ⇒ **Re-measure `df` once at the end, and
report the session total — never the per-step delta.**

Full method, with the pitfalls, in `D:/dev/rag/rust/`
(`cargo_git_pin_checkouts_accumulate_one_full_tree_per_rev.md`).

## ★★★ 0xc0000142 was HANDLE exhaustion, not free RAM — 2026-09-20

The section above says to check free RAM when a process dies with
`0xc0000142 / STATUS_DLL_INIT_FAILED`. That is one cause and it was not this
one. A gate sweep began failing to `fork` with **3.3 GB free**, which does not
fit the story. Two commands named the real cause:

```powershell
(Get-Process | Measure-Object -Property HandleCount -Sum).Sum
Get-Process | Sort-Object HandleCount -Descending | Select-Object -First 6 Id, Name, HandleCount
```

**9,661,559 handles system-wide, of which 8,873,053 belonged to one process** —
an orphaned `find.exe / -name *.rs -path *pdfcer*core*` started the previous
afternoon and still walking the whole volume twenty-four hours later, its parent
gone. Killing that single PID took the system total to **788,123** and the sweep
resumed forking within seconds.

**Why the code misleads.** Windows builds a process by handing it handles; when
the system table is exhausted `CreateProcess` fails during DLL initialisation
and reports `0xc0000142` regardless of free memory. The same code means two
different things and a free-RAM reading cannot separate them.

**How to apply.** On any `0xc0000142`, any `fork: Resource temporarily
unavailable`, or any stall with no memory-pressure reading to justify it,
**measure the handle sum first** — it is one command and it names the offending
process, where free RAM only confirms that the thing you already suspected is
not the cause. Two riders:

- **`find /` on Git-Bash walks every mounted volume**, OneDrive placeholders
  included, and nothing in the habit that spawns it adds `-xdev`. Scope the
  search to the tree, or use `Everything`, which is already resident here. An
  unscoped one survives the session that launched it and keeps accumulating.
- ⚠ **The 2026-09-19 clippy failures blamed on RAM above overlap this orphan's
  lifetime**, so that attribution is unproven. What is measured here is the
  mechanism, not which of the two causes a given failure —
  [[a-launch-failure-blamed-on-a-resource-count-needs-a-control-binary]].
- ⚠ **A gate that loses a subprocess to the fork storm still prints its clean
  sentence.** `check-selection-channel` emitted `fork: retry: Resource
  temporarily unavailable` mid-scan and then `clean — the widget channel is read
  only where it is defined`, because the runner scores the gate on its exit code
  and a lost child does not change it. Any gate whose log carries a fork error
  has to be re-run on a healthy machine before its green is quoted —
  [[a-runners-sentinel-is-a-claim-about-the-runner]].
