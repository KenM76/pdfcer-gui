---
name: a-detectors-scope-is-a-claim
description: Gates and cargo alike report clean over a scope nobody read - three blind spots in one gate, a line-unit trap, and a `-p` flag that narrows clippy to one package
metadata:
  type: feedback
---

**A detector has two independent claims in it: *what it looks at* and *what it
looks for*. Only the second one ever gets tested. "No violations found" is
evidence about the second and says nothing at all about the first.**

**Why:** `tools/gates/check-orphan-docs.py` has now reported `clean` over three
separate blind spots, one after another, and each time in exactly the words a
real all-clear uses:

| date | the scope it did not have | how much it could not see |
|---|---|---|
| 2026-09-12 | scanned `crates/` only | **77%** of the tree's `.rs` files |
| 2026-09-12 | read files as LF | **16%** of files are CRLF |
| 2026-09-13 | title regex required `**` immediately after `/// ` | **893 titles, 42%** of the convention |

The third one is the cleanest illustration. The gate exists to police doc-comment
titles. This crate's titles may open with a decoration run — `★`, `★★★`, `⚠`,
`→`, `⇒` — and every decorated title was invisible **to the instrument built to
police titles**. Widening the regex to `(?:[★⚠→⇒]+ )?` took the title count from
1,223 to 2,116 and the reported seams from 1 to 8. Seven real defects had been
sitting behind the scope, in a tree the gate called clean, for as long as the
gate had existed.

The failure mode is worse than a red gate that gets ignored, because a green
gate over a shape it cannot express reads as *positive evidence*. Every "clean"
it printed made the next person less likely to look.

**A fourth shape, and it is the INPUT UNIT rather than the glob.**
`check-ui-strings.sh` scans line by line, so it sees a literal only when the
opening AND closing quote sit on one physical line. Falsified on a scratch tree:
`"Save the document now"` on one line gave rc=1; the same sentence
backslash-continued gave rc=0. A `#[expect(..., reason = "...")]` — text only
the compiler reads — therefore fails un-continued and passes continued, and the
gate's verdict is about **typography, not audience**. The trap it sets is the
fix: re-wrapping to make it quiet teaches the next reader that the rule is line
length. Exempt it in place instead, and write the asymmetry into the gate's
header.

**A fifth shape, and it is not a gate at all - it is a FLAG on the command
you reach for while fixing one.** `cargo clippy --release -p ui-verify` came
back clean on a change that touched two crates, and the workspace gate then
failed on `too_many_arguments` in `pdfcer-gui`. `-p` is a scope, so a green
`-p X` is a claim about X and nothing else - and the temptation to narrow is
strongest exactly when a gate sweep is slow and you want a quick answer about
the file you just wrote.

The workspace command is the only one whose green means what the gate's green
means. Anything narrower is a pre-check and must be reported as one.

**How to apply:**

- When you touch or trust a detector, read **what it globs, what it opens, and
  what its regex requires** before reading what it asserts. Write down the input
  set as a number: files scanned, matches found. A gate that prints its own
  census (`titles matched: 2,116`) can be seen to go blind; one that prints only
  `clean` cannot.
- ★★★ **Widening without falsifying is documentation, not reach.** After
  admitting a new shape, plant that exact shape and confirm rc=1 — twice: once
  in the self-test (cheap, proves the regex) and once in a real source file run
  as a subprocess (proves the call site). And if the widened thing is a *set*,
  exercise **each member**: a test that plants a `★` orphan measures `★`, not
  `DECOR`.
- The narrow-regex leg is done by **overriding the constant on the imported
  module** from the falsification script, so nothing on disk changes and there
  is no restore to get wrong.
- Before quoting a build or lint green, read its **scope flags** the same way
  you read a gate's glob. `-p`, `--lib`, `--tests` and a bare `cargo check` all
  answer about less than the gate does.
- Suspect the scope first whenever a long-green instrument suddenly finds a lot.
  The finding is usually not "seven defects landed"; it is "seven defects were
  always there and the scope just moved".

Siblings: [[a-check-that-cannot-fail-is-not-evidence]] (an assertion that
cannot go red), [[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]] (the right
assertion pointed at the wrong surface), [[a-rename-can-blind-an-instrument-silently]]
(a scope that *was* right and got rewritten),
[[a-gate-whose-input-set-comes-from-git-measures-the-index]] (a scope that
changes under you), and
[[a-hand-written-list-inside-a-completeness-test-is-the-gap]] (a scope
enumerated by hand). And the two-leg discipline above is the same one as
[[falsify-the-gate-against-the-real-files-and-the-fix-against-a-control-binary]].

## The rule's NAME can be the whole claim

`check-memory-index.sh` had a rule called **DANGLING**, described as *"a link
with no file behind it"*. It read the markdown links in `MEMORY.md` and nothing
else. But a memory folder has **two** kinds of link — the index's `(file.md)`
pointers and the `[[wikilink]]` cross-references inside the bodies — and only
the first was ever checked. Eight cross-references had rotted through renames
and prefix typos, and the gate had been green over every one of them, because
its name described link-checking in general and its body did one kind.

⇒ **When a rule's name is a general noun, ask which instances of that noun it
actually enumerates.** "Links", "tests", "modules", "surfaces", "commands" —
each has more than one population in a codebase of any size, and the gate
usually walks the population that was convenient to walk when it was written.

★ The second half is a design note, not a scope note. Widening the rule to
redden on *every* unresolved wikilink would have been wrong: an unresolved
`[[name]]` is the documented way to mark a memory worth writing later, so that
version forbids the convention and gets switched off. The rule that shipped
reddens only when the dead name **nearly** matches a live one. A scope widened
past what the convention allows is not a stricter gate; it is a gate with a
short life.
