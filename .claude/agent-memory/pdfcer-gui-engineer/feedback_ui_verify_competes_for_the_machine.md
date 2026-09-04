---
name: ui-verify-competes-for-the-machine
description: ui-verify drives the real desktop, so it cannot run while Ken is using the PC — batch harness runs, ask once with a cost, and sweep the WHOLE suite when he grants it
metadata:
  type: feedback
---

**`tools/ui-verify/` may not be run while Ken is working at the machine.** It
launches the release binary, raises its window, and injects synthetic mouse and
keyboard input through the OS. That steals focus and clicks into whatever the
operator has in front of them.

**Why:** stated by Ken on 2026-08-17 — *"i am using the pc as well, so you can't
use the mouse display or keyboard until i give the go ahead."* This is not a
one-off: the harness is a single-desktop instrument by design (that is what
makes it the only oracle R1 trusts), so it will always contend with the operator
for the same machine.

**★★★ The permission is REVOKED as readily as it is granted, and the signal is
easy to miss.** *"PC is yours"* opens the desktop; *"I'm back on the PC"* closes
it again, and he may say so **mid-turn**, in a sentence that reads like small
talk rather than an instruction. On 2026-08-28 it arrived as *"I'm back on the
PC so that's why a few of your tests may have gone wonky"* — phrased as an
explanation of MY results, not as a demand.

★★ Two obligations follow, and the second is the one that gets skipped:

1. **Stop driving immediately**, including anything already running in the
   background — a backgrounded sweep keeps stealing his pointer for minutes
   after the message arrives.
2. **Say which results are now suspect.** He offered the contamination as a
   courtesy; treat it as a finding. A driven run overlapping his input is
   *unverified*, whatever verdict it printed — and a PASS is as suspect as a
   FAIL, because a stray click can satisfy an assertion as easily as break one.

**How to apply:**

- Treat display/keyboard/mouse as a **shared resource requiring an explicit
  go-ahead**, not as something to ask about per-run.
- Compute-only work stays available under the constraint: `cargo test`,
  `cargo clippy`, `tools/gates/run-all.sh`, source reading, and *writing* new
  `ui-verify` checks all run headless. Only *executing* the harness is blocked.
- Do the work anyway, then **batch the harness runs** and present them as a
  queue when the go-ahead comes. Do not stall a build waiting for the desktop.
- **Never let the constraint soften R1.** Work completed under it is
  *unverified*, and must be reported in exactly those words — this project was
  founded on a commit that said *"analysis-confirmed, NOT empirically
  verified"* and was treated as done anyway. A green `cargo test` is not a
  substitute and saying so is the whole point of [[r1-drive-the-binary]].

## ★ Asking: one line, at the end, with the cost — and then sweep everything

Confirmed 2026-08-27. A session report ended with *"Next — and I need the
machine for it. I wrote the driven check … it has never been run; it moves the
cursor, so it can't run while you're working. It's about ninety seconds."* He
answered **"the PC is yours go ahead"** with nothing else.

Two things that worked and should be repeated:

1. **One line, last, naming the cost.** Not a paragraph, not a mid-report
   digression, not a question he has to decide something about. What he is
   granting is *time on his own desktop*, so the only fact he needs is how much.
2. **"The PC is yours" means the whole suite, not the one check you asked
   about.** That grant was spent on all 82 checks in slices, plus falsifying the
   new one against a deliberately-broken rebuild, plus four harness repairs the
   run exposed — and it was the right reading. A go-ahead is expensive to obtain
   and cheap to use fully; asking twice in a day for two halves of one sweep is
   the waste.

While driving, his desktop is yours to tidy **reversibly**: killing leaked
`pdfcer-gui.exe` processes, and `Shell.Application.MinimizeAll()` paired with
`UndoMinimizeALL()` at the end. Do not close anything of his, and put it back
before reporting.

See also [[project-operator-report-2026-08-17]].

## ★★★ AND DO NOT EDIT SOURCE WHILE THE SWEEP IS RUNNING — 2026-09-03

The harness refuses to drive a binary older than `crates/`, and it is right to:
*"the traces you are about to collect would describe code that is NOT the code
you just wrote, and a missing trace looks exactly like a broken feature."*

So a sweep started, and then edited under, does not fail — it **skips**. A
90-minute run came back **8 passed, 4 failed, 141 SKIPPED**, every skip reading
`STALE BINARY — refusing to run`, because source edits landed ten minutes into
it. The staleness guard did exactly its job; the whole run was wasted anyway.

⇒ **A driven sweep is a lock on the source tree, not just on the pointer.**
While one is running, do documentation, triage, memory and RAG work — never a
`cargo build`, never an edit under `crates/` or `tools/`.

★ Two smaller ones from the same run:

- **`nohup … &` inside the Bash tool does not survive.** The wrapper shell exits
  and takes the job with it; the log had two lines and no process. Use the
  tool's own `run_in_background`.
- **Do not pipe the sweep through `tail`.** The output buffers, so nothing is
  readable until it finishes, and the pass/fail lines are then truncated away.
  Redirect to a file and read that.
