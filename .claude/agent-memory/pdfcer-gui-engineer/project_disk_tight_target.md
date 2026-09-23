---
name: disk-is-tight-and-target-grows-unbounded
description: Disk AND RAM are tight; clear debug/doc target routinely, run the suite with a job limit, wait for orphaned linkers after a kill; the BIGGEST disk item is one full engine source tree per pin bump under ~/.cargo; 0xc0000142 / fork failures are usually HANDLE exhaustion by an orphaned process rather than free RAM; and a low-memory kill with no process to blame is kernel PAGED POOL, which no working-set scan can see
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
Relaunch with **`CARGO_BUILD_JOBS=1`** for the sweep specifically — clippy peaks
higher than `cargo test`, 4 was not enough head-room, and **2 was killed too**
on 2026-09-20. The kernel-pool section below is why lowering the job count
keeps failing to be enough.

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

## ★★★ The gigabytes with no process to blame are in the KERNEL — 2026-09-20

A gate sweep at `CARGO_BUILD_JOBS=2` was killed for low memory with 3.7 GB free
and **no orphan to blame**: the usual scan found no surviving `cargo`, `rustc`,
`clippy-driver` or `bash`, and the largest working set on the machine was
`Memory Compression` at 1.0 GB. Every instrument in the sections above
exonerates everything, which is the signature that the instrument is looking in
the wrong address space.

Measured on the idle machine minutes later:

| Reading | Value |
|---|---|
| physical total / free | 16,306 MB / 3,784 MB |
| **sum of ALL process working sets** | 9,729 MB |
| physical in use | 12,492 MB |
| **unattributed to any process** | **2,763 MB** |
| **paged pool** | **4,317 MB** |
| nonpaged pool | 1,158 MB |
| largest working set | `Memory Compression`, 1,020 MB |

**Paged pool alone is 26 % of the machine**, where a healthy figure is low
hundreds of MB. It is kernel memory, so it appears in no process's working set,
and the 2,763 MB gap is only the part still resident — the rest is paged out and
still charged against commit, which stood at 30,854 MB of a 48,306 MB limit.

**How to apply.** When a build is killed for low memory and the top working set
is under a gigabyte, do not re-run the working-set scan with a lower job count.
**Sum the working sets and subtract from physical-in-use**, then read the two
pool counters:

```powershell
$ws=(Get-Process|Measure-Object WorkingSet64 -Sum).Sum
$os=Get-CimInstance Win32_OperatingSystem
"unattributed MB: {0}" -f [int]((($os.TotalVisibleMemorySize-$os.FreePhysicalMemory)*1KB-$ws)/1MB)
(Get-Counter '\Memory\Pool Paged Bytes','\Memory\Pool Nonpaged Bytes').CounterSamples |
  ForEach-Object { "{0}: {1} MB" -f $_.Path, [int]($_.CookedValue/1MB) }
```

### ★★★ The obvious suspect was measured and CLEARED — handles are not pool

The handle sum was **921,962, of which OneDrive held 587,490 — 64 % of every
kernel object on the machine**, on a working set of 447 MB and a process that
had been up three days. The cheapest process by the usual measure holding two
thirds of the kernel's objects, next to a pool four times its healthy size, is
as strong a circumstantial case as this kind of hunt produces. It is wrong.

Control, then the release, then the re-read:

| | handles | paged pool | free |
|---|---|---|---|
| before | 921,962 | 4,319 MB | 3,801 MB |
| after killing OneDrive | 334,441 | **4,310 MB** | 4,218 MB |
| after relaunching it | 339,263 | **4,312 MB** | 3,474 MB |

**587,521 handles were released and the paged pool did not move** — 9 MB, noise.
The 417 MB that came back was the process's own working set, and the relaunched
client took it straight back scanning. Handles and paged pool are independent
resources on this machine, and the 4.3 GB is still unattributed.

**How to apply.** Do not conflate the two. A handle sum in the millions predicts
`0xc0000142` and `fork` failures — that mechanism is measured and holds. It does
**not** predict a low-memory kill, and freeing handles buys no RAM. ⚠ The
attribution in the paragraphs above was a hypothesis with the right shape and no
evidence, of exactly the kind
[[a-launch-failure-blamed-on-a-resource-count-needs-a-control-binary]] describes;
what made it safe was reading the pool **before** touching anything. Keep the
control reading; it is the whole difference between a finding and a story.

**Still open:** what holds 4.3 GB of paged pool. Per-process attribution cannot
answer it (`\Process(*)\Pool Paged Bytes` totals 153 MB) and tag-level
attribution needs `poolmon`, which is not installed here.

### ★★★ The job limit is not the lever, because the sweep is not the consumer

Thirty minutes after the section above, the relaunched sweep at
**`CARGO_BUILD_JOBS=1` was killed too** — and the state at the moment of the
kill settles what the watchdog is actually reacting to:

- it died inside **`check-trace-names`, a pure shell gate**: no `cargo`, no
  `rustc`, no `clippy-driver`, no linker existed on the machine;
- the surviving sweep shell held **9 MB**;
- the largest working set on the whole machine was `Memory Compression` at
  953 MB, and free stood at 2,847 MB — where it had been sitting, idle, all
  morning.

**The watchdog watches the SYSTEM, not the command.** It reaps the background
Bash-tool process tree when free memory is low no matter what that tree is
doing, so a sweep is killed for the machine's steady-state occupancy — three
`claude` processes at 1,464 MB between them, `Everything` at 738 MB, a 4.3 GB
paged pool — and lowering `CARGO_BUILD_JOBS` cannot help because the job count
governs only the final `clippy`, which in three of these kills never started.

**How to apply: stop tuning the job count and detach the sweep instead.** A
PowerShell `Start-Process` child is outside the watchdog's tree and runs to its
SUMMARY. Point it at a two-line wrapper — `cd` to the repo, run `run-all.sh`,
`echo "SWEEP-EXIT: $?"` — and invoke that through the **explicit Git
`bash.exe`**, because a different shell brings a different `PATH` and that
changes which `bash` the gates themselves run under —
[[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]].

```powershell
$bash = 'C:\Program Files\Git\bin\bash.exe'
Start-Process -FilePath $bash -ArgumentList @('-lc', "$sp/sweep.sh") `
  -RedirectStandardOutput "$sp\gates3.log" -RedirectStandardError "$sp\gates3.err" `
  -WindowStyle Hidden -PassThru
```

⚠ **Pass the wrapper path as the whole `-lc` string. Do not nest a `bash` in
front of it.** `-lc "bash <path>"` left a live `Git\usr\bin\bash.exe` that never
ran the script: three shells, 0.4 s of CPU in seven minutes, a zero-byte log,
and — the tell — **no `grep`, `python` or `cargo` child anywhere on the
machine**, where a running sweep always has one. It reads exactly like a stall
inside the first gate, so the diagnosis goes to the gate rather than to the
launcher. It failed the same way twice, including once with a POSIX path and
`< /dev/null`; the recorded `@('-lc', "$sp/sweep.sh")` form runs. When a
detached launch will not start, fall back to an ordinary background Bash-tool
run — the watchdog may or may not reap it, and a reaped sweep is recoverable
where a silent one is not.

⚠ A `Remove-Item` anywhere in the same PowerShell block is refused —
*"Remove-Item on system path ''C:\Program' is blocked"* — because the sandbox's
static read pairs the delete with the `'C:\Program Files\Git\bin\bash.exe'`
literal beside it, not with the scratch path it was actually given. Write a
fresh log name rather than pre-deleting one.

**The detachment was proved by a control, not assumed.** Minutes after that
launch the watchdog fired again and killed a background Bash-tool command that
was nothing but a `sleep 20` poll loop — while the detached sweep, 6.5 MB and
six seconds older, kept running. Two processes of the same size at the same
instant, one inside the watchdog's tree and one outside it, and only the inside
one died. Size is not the criterion; membership is.

⚠ **So do not poll a detached run from a background Bash command** — that
waiter is in the tree and gets reaped, which reads as the sweep having died.
Detaching also costs the completion notification: the `SWEEP-EXIT:` line the
wrapper appends is the only signal the run finished rather than died, and a
timed wake-up has to come back and read it.

## The FOREGROUND is outside the watchdog too — 2026-09-22

A background sweep was reaped at `cargo clippy` with 5.1 GB free (commit
22.6/47.2 GB; paged pool 1.9 GB). Finished instead in the **foreground Bash
tool**: `CARGO_BUILD_JOBS=1 cargo clippy --workspace --all-targets` took 51 s,
`cargo test --workspace` at jobs=1 under 10 min — both inside the 600 s ceiling,
neither reaped. Since clippy is `run-all.sh`'s LAST step, a sweep reaped there
has already scored every other gate; read the log for FAIL, then finish clippy
in the foreground rather than re-running 80 gates.

⚠ **`run-all.sh` does not run `cargo test`.** The same session's full test run
caught a settings-catalog count the whole sweep could not see. "The gates are
green" is not "the tests are green"; run both before a commit.
