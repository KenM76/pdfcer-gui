# `ui-verify/checks/page_size`

`resizing_a_sheet_changes_the_paper_in_the_saved_file` — picking a sheet
size for the page an operator is looking at must change **that page's
`/MediaBox` in the file he then saves**, and must leave the page beside it
alone.

# What this is about


# HOW I WOULD FALSIFY THIS CHECK

Stated first, because a check nobody can falsify is a claim rather than a
measurement, and this one asserts an absence in phase D as well as a
presence.

**Four plants, each of which this check must go red for.** Each is a real
way this feature can be built wrong, and each produces a *perfect* trace in
the writing process — which is exactly why the verdict is taken elsewhere.

| plant | what the writing process still reports | what phase D reads |
|---|---|---|
| make `actions::pagesize::set` return `Ok(Vec::new())` without calling the engine | `page-size-commit`, and `page-size-changed … n=1` from the funnel, unchanged | page 0 still at its original size ⇒ **FAIL** |
| send `(0..doc.pages.len())` instead of the operands | everything, unchanged | page 1 **also** A6 ⇒ **FAIL on the negative control** |
| trace the *requested* rectangle instead of `session.pages()` | a perfect `page-size-sheet w=297.64` | phase D is a different process reading a file off disk; the plant cannot reach it ⇒ still **FAIL** if the write was dropped |
| transpose in `sheet_pt` | a plausible summary line | page 0 at 419.53 × 297.64 ⇒ **FAIL**, and the message says *transposed* rather than *wrong size* |

**How to run the falsification.** Plant one, `cargo build --release -p
pdfcer-gui`, run this check alone, and require its own `[FAIL]` line in the
output — not merely a non-zero exit, which a SKIP also produces. Restore
from a byte copy, never from `git checkout`.

**The one thing that would make this check worthless** and must be watched
for: if `page-size-document` were ever published from the *request* rather
than from `doc.pages`, every phase below would still pass on a build that
wrote nothing. That line's own comment in `dialogs::page_size` carries the
argument; this is the check that would silently stop meaning anything if it
were ignored.

# The five phases, and why the verdict is in a second process

| phase | process | what it establishes |
|---|---|---|
| **A** | 1 | the fixture's page sizes **as this build resolves them** — the baseline every later number is compared against, so the check does not depend on a hard-coded fixture geometry |
| **B** | 1 | Pages ▸ Sheet size opens a window, and picking A6 portrait reaches its commit |
| **C** | 1 | Save a copy writes a file |
| **D** | **2** | **THE VERDICT** — a fresh binary opens the written file and reports the page sizes *it* resolves from the bytes |
| **D′** | 2 | **THE NEGATIVE CONTROL, in the same trace line family, from the same instrument, in the same run** — page 1 was not an operand and must read exactly what phase A read for it |

⇒ **The oracle is not the code under test.** Phase A and phase D are the
same reader over two different files; the thing being judged is the
difference between them. `signing.rs` established this shape here and its
own falsification run is the argument: a planted flipped byte left the
writing phase's trace *character for character identical* and the reading
phase caught it.

# The operand rule is used rather than driven

Nothing is picked in the Pages panel, deliberately. `pages.resize` takes the
same operand as every other `pages.*` command —
`panels::pages::ops::operands`: the picked sheets when there are any, **the
current sheet when there are none** — so a launch with no picks aims the
command at page 0 and leaves pages 1–3 as the control. Driving the panel's
multi-select would test the panel, which has its own checks, and would put a
second failure mode between this check and its subject.

# A6, and why not A4

A6 is 297.64 × 419.53 pt. `four-pages.pdf` carries three distinct sheet
sizes — 2383.94 × 1683.78, 612 × 792 and 306 × 396 — and A6 differs from
every one of them **in both dimensions**, so neither a no-op nor a
transposition can pass vacuously. A4 would have been the lazy choice and is
also the size the *other* size window opens on, which is precisely the
coincidence a check should not rest on.
