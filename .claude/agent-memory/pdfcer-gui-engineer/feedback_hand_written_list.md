---
name: a-hand-written-list-inside-a-completeness-test-is-the-gap
description: A test that proves "the window draws everything the catalog describes" is only as complete as the hand-written file list inside it — a new module is invisible to the check built to find it
metadata:
  type: feedback
---

**When a completeness test enumerates its inputs by hand, adding a new input
makes the test silently narrower — and the count still adds up.**

**Why:** 2026-08-28, adding a Comments group to the Settings window.
`text::settings::tests::the_window_draws_exactly_the_settings_this_catalog_describes`
parses every settings module with `syn` and counts `widgets::header` calls
against `SETTINGS_COUNT`. The module list, `GROUP_SOURCES`, is a hand-written
`&[(&str, &str)]` of `include_str!`s.

A new `dialogs/settings/comments.rs` was not in it. So:

- its header was **not counted** → the drawn total stayed at the old number;
- its triple was **not in the catalog** → the described total also stayed;
- **both halves were wrong by one and the assertion passed.**

The whole suite went green with a setting that no completeness check had ever
looked at. The test is genuinely good — it caught two engine settings within
one `cargo update` each — and its blind spot is only reachable by *adding a
file*, which is rare enough that nothing routinely exercises it.

**How to apply:**

- **Before writing a new module that a sweeping test consumes, add it to that
  test's input list.** Not after — the green run in between is the trap.
- When you meet a test that walks "all the X", **grep for how it gets the
  list**. `include_str!` in a const array, a `match` over an enum, a `&[&str]`
  of ids — all of these are hand-written and all decay the same way. A
  `read_dir` or a build-script glob does not.
- The tell that a sweep is hand-listed: it compiles when a file it should cover
  is deleted.

★ This is the same family as [[a-check-that-cannot-fail-is-not-evidence]] and
[[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]], with one difference worth
holding: those two are about a check that never *fires*. This one **fires,
passes, and reports a number** — the arithmetic is internally consistent and
externally short.

## ★★★ THIRD RECURRENCE — 2026-09-03, and it hid FOUR operator-visible defects

`ui-verify`'s `dialogs_open_in_their_own_window` sweeps every command-reachable
dialog for *"is this a real OS window"*. Its subject list is a `const DIALOGS:
&[(&str, &str)]` typed by hand, and **Print was not in it** — the dialog whose
report (*"Print dialogue box doesn't pop up in its own movable window"*) started
that entire piece of work.

The header even rationalised the omission: *"Print was fixed that evening and
`print_dialog` asserts it."* And `print_dialog` asserts the job reaches the
**spooler**. That is not a claim about the window, its margins, its scrollbars
or its buttons — and all four of those were broken, for weeks, in the one
command-reachable dialog with no headless check.

⇒ The tell to look for: **a completeness sweep that names an exception in
prose.** "X is covered elsewhere" inside the list's own documentation is the
sentence that decays, because "elsewhere" asserts something about another test
that nobody re-reads. If it is genuinely covered elsewhere, the list costs one
line to include it anyway and the duplicate proves the claim.

## ★★★ THE OMITTED FILE WAS THE ONE THE GATE EXISTED FOR — 2026-09-09

`check-stale-blockers.sh` scans a hand-written `DOCS=(...)` for rows that
declare `BLOCKED` on a request the engine has closed. Its list held
`OPERATOR_REQUESTS.md`, `FEATURES.md`, `GUI_ROADMAP.md` — and **not
`ENGINE_BACKLOG.md`, the one file in the project whose entire purpose is
blocked rows.**

Three rows in it declared blocked on asks the engine had shipped **the same day
they were filed**, three days earlier. The gate ran green over them every
session because it never opened the file. Nothing looked wrong: the gate
reported OK, the documents it named were real, and the count of documents
scanned added up.

⇒ **When the check for a class has a hand-written scope, the first question is
"is the canonical home of that class in the list?"** — not "are the entries in
the list correct". The omission is invisible from inside; only naming the class
and asking where it lives finds it.

★ The list stays hand-written on purpose — `HANDOFF.md` and `CONTINUE.md` are
HISTORICAL and a past tick correctly says "blocked" about the day it was
written, so a sweep of every `*.md` would demand rewriting history to stay
green. ⇒ The fix is not "make it automatic"; it is a comment at the list
telling the next author to add their document **in the same commit**.

## ★★★ FIFTH — 2026-09-13, and this one was a hand-written list of ENUM VARIANTS, not files

`text::scale::tests::every_unit_is_named_distinctly` is the test whose entire
job is to catch a unit that reaches an operator with no English label, or two
units sharing one. Its subject list was:

```rust
let units = [Unit::Millimeter, Unit::Centimeter, Unit::Meter,
             Unit::Inch, Unit::DecimalFeet, Unit::FeetInches];
```

The engine shipped `Kilometer`, `Yard` and `Mile` that morning. **The test
would have passed, green and silent, with all three unlabelled.** What actually
caught them was the exhaustive `match` in `unit_name` directly above it — the
compiler refused the build. The test was decoration and had been since it was
written; the guard that worked was somewhere else entirely.

It reads `Unit::all()` now — the same slice the three unit dropdowns read, so
the test and the product cannot disagree about what the set is — and was
falsified with a planted duplicate label before being believed.

⇒ **Four earlier instances were hand-written lists of FILES. This one was a
hand-written list of VARIANTS of a type that already knows its own members.**
The family is wider than "module lists":

| hand-written | the thing that already knows |
|---|---|
| `&[(&str, &str)]` of `include_str!` | `read_dir` / a build script |
| `const DIALOGS: &[(&str, &str)]` | the command registry |
| `DOCS=(...)` in a shell gate | the class's canonical home, named |
| **`let units = [Unit::A, Unit::B, …]`** | **`Unit::all()`** |

⇒ **The sweep to run, once, and it is overdue:** grep every `#[test]` whose
name contains `every_`, `all_` or `complete` for a `let xs = [` or a `const X:
&[` in its body. That literal IS the gap, every time. The lesson has been
written five times; the instrument has never been built, which is
[[a-lesson-in-a-docstring-is-not-an-instrument]] applied to this very memory.

★ And note what the *engine* did with the same problem on the same day: when
`Unit::all()` stopped returning `[Unit; 6]`, the cardinality check it had been
relying on silently disappeared. They replaced it with an **exhaustive match**
rather than a length assertion, and wrote down why — *"a plain length assertion
would have gone red with a number to bump, which is the kind of failure people
fix by bumping the number."* That is the right shape: a guard whose failure
cannot be discharged by editing a constant.


## ★★★ SIXTH — 2026-09-13, AND THE INSTRUMENT EXISTS NOW

`tools/gates/check-completeness-tests.py`, registered in `run-all.sh` in the
same commit. It is the answer to the note five sections up that kept saying
*"build the grep"* and never did.

**What it looks for, and why the obvious version was useless.** The first draft
flagged any literal array inside a function named `every_*` / `all_*` / `each_*`
/ `*_complete*` and returned **53** hits on a clean tree. Most were test INPUTS
— `let widths = [90.0, 70.0, 130.0]`, a pair of drag corners — and a gate
that fires on those teaches people to write exemptions, which is how a gate
turns into scenery. The predicate that works is narrower: **three or more
elements of the array must be `Type::Variant` paths sharing one `Type`.** That
is not a list of inputs; that is a private copy of an enumeration. It returns
**29**.

★★ **The severity axis I did not expect: LOCAL versus FOREIGN.** Nine of the
twenty-nine copy a type this repository does not declare — `FormatError`,
`ReflowDecline`, `EditError`, `Object`, `SnapKind`, `RecompressReason`,
`BlendSpaceFrom`, `StampSizeSource`, `ButtonAction`. Those are the dangerous
ones by a category, and the reason is the thing this project keeps relearning:
**the engine's enums grow on a BRANCH pin that moves without a `cargo update`,
and nothing on this side is edited on the day it happens.** A hand-copied local
enum at least has the copy and the declaration in one repository, so some commit
touches both neighbourhoods. A hand-copied engine enum has no such day.

★ **And the classification was wrong by one until the aliases were resolved.**
Three sites enumerate `E`, `D` and `R`, which are `use ... as` aliases, not type
names. A first pass that classified origin by grepping this repository for
`enum X` called all three FOREIGN; `R` in `text/redact/tests.rs` is
`crate::redact::RedactApplyRefusal`, which is ours. The import statement is the
better oracle in both directions — it resolves the alias, and it also catches a
type this repository declares but that FILE imports from elsewhere. ⇒ **A
severity count wrong by one in the direction of alarm devalues the other nine**,
so this was worth the extra pass rather than a footnote.

★ **The register is called a debt register, in those words, inside the file.**
`completeness-snapshot.txt` holds the 29 so the gate can be adopted red-free,
and the header states the distinction that erodes: *an exemption says "this is
fine"; a debt entry says "this is wrong and has not been fixed yet."* Every run
prints the outstanding count. Both directions fail: a site missing from the
register (the debt grew) and a register line matching nothing (the register
stopped describing the tree).

## ★★★ SEVENTH — 2026-09-13, and the ENGINE named a third shape the six above had not

The bump that brought `SnapKind::all()` arrived with a notice, and its ★★ section
is the part worth keeping. `SnapKind` had no `all()`; its ordering test,
`priority_ranks_follow_decision_011_high_to_low`, carried the same hand-written
eight this project's `every_snap_kind_has_a_non_empty_marker_—` carried. Two
projects, same enum, same defect, found from opposite sides of a request channel
within a day of each other.

But the engine's analysis went where mine had not. `SnapKind::priority()` is an
exhaustive match, so a ninth kind **was** a compile error there — the
completeness half was covered. What no longer existed was any check that the
ranks are **unique**. Give a new kind rank `5`, colliding with
`DerivedCenterline`:

* `priority()` compiles, the match is satisfied;
* the ordering test passes, the new kind is not in its copy of the list;
* `snap_candidates` sorts by rank then by distance, so **two kinds sharing a
  rank makes the winner between them depend on the order candidates happened to
  be generated in** — a non-deterministic pick under the operator's cursor.

⇒ **Three shapes now, and they are genuinely different failures:**

| shape | what happened |
|---|---|
| a guard **deleted by an improvement** | `Unit::all()` stopped returning `[Unit; 6]` and the cardinality check vanished with the type |
| a guard that **never worked** | `every_unit_is_named_distinctly` was decoration from the day it was written |
| **a guard that worked for what it was WRITTEN for while never covering what it was NAMED for** | the snap ordering test covered completeness, via the compiler next door, and never covered the ordering — or the uniqueness the ordering assumes |

★★ The third is the hardest to see, because it is a *working* guard. Nothing
about it looks neglected: it is green, it is exercised, and the property it
happens to enforce is real. The question that finds it is **"what does this
test's NAME promise, and which line asserts that?"** — not "does this test
pass" and not "is this list current".

★ And the correction I had to make to my own fix while writing it: my first
doc comment called `SnapKind` `#[non_exhaustive]` and called this the severe end
of the class. It is neither. `SnapKind` derives plainly and
`snap_marker_shapes` matches it exhaustively with no wildcard, so a ninth
variant breaks this crate's build today. **The honest hazard is narrower and
more interesting:** the protection is a neighbour's, not the test's, and it
evaporates the day somebody adds a `_ =>` arm to that match — at which point
both safeguards fail in the same instant, because they were never independent.
⇒ **Check whether the compiler is already covering you before writing that it
is not.** An overstated severity in a memory is the same defect as an
overstated capability in product copy.

★ The register earned its keep in its first week, exactly as predicted: fixing
the site made `check-completeness-tests` go **red**, because the snapshot line
no longer matched anything in the tree. A debt register that only fails in the
direction of new debt would have sat there holding a row about a defect that no
longer existed. Outstanding is 28 sites, 8 of them foreign, down from 29 and 9.
