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
