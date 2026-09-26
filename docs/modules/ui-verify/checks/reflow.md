# `ui-verify/checks/reflow`

`reflowing_a_paragraph_rewraps_it` — **put the caret in a paragraph, press
Reflow, and the block loses a line.**

# What this is for


> *"I think the paragraph reflow was implemented ages ago in the pdfcer core,
> so we should have that option too."*

He was right. `EditSession::reflow_block` shipped in `Pass 91`, was tested,
was documented, and **no control in this shell had ever raised it**. That is
the failure mode this check exists at: not a broken verb, an unreachable
one.

## Why a unit test cannot see what this sees

Because every link in front of the verb is wiring, and wiring is what this
project keeps shipping broken with a green suite:

| # | link | a unit test can see it? |
|---|---|---|
| 1 | `edit.reflow_block` is registered and drawn on the Edit tab | yes — a registry test |
| 2 | a click on it reaches `dispatch_command` | **no** |
| 3 | the arm finds the **caret's draft** in egui memory | **no** — the draft is frame state |
| 4 | the draft's `Anchor::Run` resolves to a **block index** | partly |
| 5 | the action reaches `reflow_block` and the engine re-wraps | yes |

Link 3 is the one with no other instrument. The operand is not a
selection the application holds — it is a caret in `egui`'s temporary data,
written by a click and read by a command, and nothing but a driven run puts
a real one there.

## The oracle is the LINE COUNT, and it is a real one

`reflow-block-applied … lines=6->5`. The fixture is built so a correct
reflow **must** change that number: six deliberately short ragged lines,
wrapping to the widest of them, packs to five. A build that raised the
action, reached the engine and re-wrapped nothing would report `6->6`, and
that is asserted as a **failure** rather than accepted as "it ran".

⇒ `tools/gen-reflow-fixture.py` and this file are one instrument. Its header
records why no existing fixture would do: a CAD title block has no
paragraph at all, and `tail-alignment.pdf`'s blocks are placed flush by
measurement, so re-wrapping them has nothing to do. **A check driven against
either would report the feature broken about a build whose reflow works.**

## What this check deliberately does NOT do, and what it CANNOT see

## Item notes

### `const INVOKE`

`edit.text` is rung here rather than clicked because arming the caret is
**not what this check is about**, and it already has its own driven check.
A check that re-verifies its own preconditions through the slowest possible
route fails for reasons that are not its subject.

### `const REFLOW_ITEM`

The pair is written once so the region and the id cannot drift apart: a
check that clicked one control and asserted about another would pass or fail
for reasons unrelated to either.

### `const APPLIED_EVENT`

`-applied`, per the convention this project adopted after making the
same-name mistake twice: `vector_edit` writes its own bare `reflow-block …`
line for the identical edit, and `.last()` on the bare name reads that one.

### `const CARET_AT`

Inside **line 2** of the block — `x = 120`, baseline `y = 668` — rather
than in the first or last line. A first-line caret would pass on a build
whose block lookup returned 0 unconditionally, and the last line is the one
most likely to be split off by a recogniser that disagrees about the block's
extent. The middle is the honest place to ask.

The numbers come from `tools/gen-reflow-fixture.py`, which prints the
geometry it computed for exactly this reason. They are quoted, not derived
twice.

### `fn parse_lines`

A parse rather than a `split` at the call site, so a malformed field is a
SKIP with a sentence instead of a silent `0->0` that would pass the
`after >= before` test by arithmetic accident.
