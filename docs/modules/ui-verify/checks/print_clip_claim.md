# `ui-verify/checks/print_clip_claim`

`print_clip_claim_follows_the_preview` — the commit button's count must be
corrected by what the preview has already looked at.

# What this is about

Operator request O113. The preview's clip hatch became ink-aware: on a 1:1
CAD sheet whose overhang is empty paper nothing is hatched, and the caption
reads *"This sheet hangs over the printable area, but nothing is printed
there — the overhang is blank."*

`Job::clipped()` stayed geometric, so the commit button went on reading
*"Print — 1 sheet will be clipped"* over a picture showing nothing lost.
Both sentences were true and they read as contradicting each other.

The fix — `crates/pdfcer-gui-base/src/printverdicts.rs` — remembers the
blank/not-blank verdict per sheet as the preview renders it, and labels the
button with `geometric − known_blank`, with unexamined sheets still counted.

# Why no unit test in the workspace can observe this

Two links, and neither is reachable from a test:

1. **The verdict is produced inside `paint`**, which needs an `egui::Ui`, a
   real device geometry from a driver, and a rasterised page. The
   arithmetic that consumes the verdict is pure and is proved in
   `printverdicts_tests`; the *recording* is not.
2. **The cache key is only interesting when it is live.** A key that
   silently never matches produces the old geometric count — which is a
   correct answer to a different question, and on a job where nothing is
   blank it is also the *right* answer. So a cache that never works looks
   exactly like a cache that works, on every machine where the fixture does
   not clip.

# What it measures

Two trace lines the dialog emits every frame it is open:

```text
print-preview canvas=[…] … tex=1 overhang=blank-band claim=none:0
print-plan    printer="…" … clipped=Some(1) claim=none:0 …
```

`clipped=` is `Job::clipped()`, the geometric count, unchanged. `claim=` is
`<state>:<count>` — what the button actually says. `overhang=` is what the
ink test found in the band of the sheet on screen.

Three assertions, in increasing strength:

| # | condition | requirement |
|---|---|---|
| 1 | always | `claim` count ≤ `clipped` count — the correction may never *invent* a clip |
| 2 | `clipped=Some(0)` | `claim=none:0` — nothing clipped, nothing said |
| 3 | `overhang=blank-band` | `claim` state is **not** `geometric` — the verdict landed and moved the number |

Assertion 3 is the one the request is about, and it is stated as "not
geometric" rather than as "none" on purpose: a multi-sheet job in which one
blank sheet has been examined and four have not is correctly `at-most`, not
`none`. Requiring `none` would fail a correct build on any job longer than
one sheet.

# The fixture assertion 3 requires, and why it is a fixture rather than a
harness setting

The scale mode defaults to **Fit**, which scales a page to the printable
area and therefore does not clip, so the check begins by clicking
`print.scale.actual` to force the 1:1 geometry the operator's case is
about.

That reaches the geometry but not the *ink*. Assertion 3 needs a sheet whose
overhang is empty paper, and whether it is depends on the document and on
the device's printable area together — neither of which a flag can set.

`fixtures/blank-overhang.pdf` is that sheet: 1,000 x 800 pt with every mark
inside a box in the upper-left corner, 95 pt clear of the nearest crop line
on either orientation of Letter paper, and with no page-wide background
rectangle, which would be ink under the `INK_MAX_LEVEL` threshold and would
make every band report `losing`. Its `.PROVENANCE.py` carries the geometry
and the reason for each number. `sweep-full.sh` names it in the `ALONE`
table, so the sweep drives this check on it and not on the shared sheet.

**It still SKIPS honestly, on two inputs.** Pointed at a full-bleed sheet
such as `a1-titleblock.pdf` the ink test correctly reports `losing`;
pointed at any document on a machine whose printable area contains the page,
nothing clips at all. In both cases assertion 3 is vacuous and the report is
SKIPPED with the values it read: it has learned that the two lines exist and
that assertions 1 and 2 hold, and nothing about the correction. The same
three-state discipline `print_dialog` applies to a machine with no printers.

# What it deliberately does NOT do

**It never presses the commit button**, and no future edit may make it do
so. Same rule and same reason as `print_dialog` and `print_paper`: that
button is the one control in the application that consumes paper and cannot
be undone, and a harness that can start a print job will eventually start
one by accident.
