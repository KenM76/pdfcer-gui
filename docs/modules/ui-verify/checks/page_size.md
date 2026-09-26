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

## Item notes

### `const A6_INDEX`

**6 is A6**, the seventh entry: `A0, A1, A2, A3, A4, A5, A6, …`. An index
because that is what the region name carries — the window publishes
`page-size.size.item.<N>` — and because this crate deliberately cannot ask
the engine: `ui-verify` has exactly one dependency, and a verification
harness that pulls in the crate under test fails to build for reasons
unrelated to the thing it is verifying, on the day it is most needed.

⚠ **So the index is checked at RUN TIME instead**, against
[`EXPECTED_SIZE_ID`] on the commit line. `PaperSize::ALL` is
`#[non_exhaustive]` and its own doc comment says the table will grow (ARCH,
JIS B, ISO B/C are all named as plausible); a size inserted before A6 would
silently make this check click A5 and then assert A6's dimensions — a red
run whose message would blame the application for a table that moved.

### `const EXPECTED_SIZE_ID`

Read from `size_id=` and **never** from the `choice=` field beside it.
That one is a `Debug` spelling, present for a human reading a trace;
Debug-formatting a domain type and then parsing it produced two false
failure reports in this project in a single week.

### `const A6_PT`

Converted from the **defining millimetres** rather than written out as
`297.64 x 419.53`, for the reason `dialogs::new_document`'s own test states:
a hand-rounded number looks right, is wrong in the fourth significant
figure, and will not compare equal to what the engine writes. Pinned against
`PaperSize::A6.size_pt()` by [`tests::a6_is_where_this_check_thinks_it_is`],
so the two cannot drift.

### `const TOLERANCE_PT`

A tenth of a point. The numbers are written by the engine and read by the
engine, so the only slack that has to be absorbed is the two-decimal
formatting of the trace line itself — 0.005 pt. A tolerance three orders of
magnitude tighter than the smallest gap between any two entries in
`PaperSize::ALL` cannot mask a wrong size.

### `fn sheets`

Reads the LAST line per index rather than the first. The window can be
opened more than once in a run — phase B opens it, phase D opens it again —
and the census is republished each time. Taking the first would hand phase D
a fossil from phase B, which is the *"`.last()` returns a fossil"* trap
`driving::declared` exists to solve for `ui-rect` and which applies to any
republished line.

### `fn open_and_census`

Used **twice** — once on the fixture in process 1 and once on the saved copy
in process 2 — which is the whole reason it is a call rather than eight
lines inline: the two censuses must be produced by the identical sequence,
or the comparison at the end is between two different measurements. That is
`save_copy`'s own `comments_count` lesson, which it learned by carrying two
copies of a census reader that were both wrong in the same way.

### `fn a6_is_its_own_millimetres`

The failure this exists for is the one `dialogs::new_document`'s own
test names: a hand-rounded `297.64 × 419.53` looks right, is wrong in
the fourth significant figure, and will not compare equal to what the
engine writes. The engine converts from the defining millimetres for
exactly that reason, and this pins that the harness does the same
arithmetic rather than a similar-looking one.

ⓘ It cannot assert against `PaperSize::A6.size_pt()` directly, and that
is deliberate: this crate has **one** dependency, `windows-sys`, and its
own manifest argues at length that a verification harness with a large
dependency tree is one that fails to build for reasons unrelated to the
thing under test. The agreement between this constant and the engine's
table is asserted at run time instead, from
[`EXPECTED_SIZE_ID`] on the window's own commit line, which is a
stronger check than a compile-time one anyway: it verifies the size the
**running program** committed rather than the one this file believes it
will.

### `fn the_fixture_can_carry_the_defect`

Two requirements, and each is a way this check silently stops meaning
anything: it needs **at least two pages**, or there is no negative
control; and page 0 must not **already** be A6, or the positive arm
passes on a build that writes nothing.

Asserted from the fixture's bytes rather than from a remembered number,
because the fixture is regenerated and a check that pinned 2383.94 would
go red for a reason that has nothing to do with sheet sizes.
