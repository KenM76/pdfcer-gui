---
name: an-inherited-session-summary-is-not-a-source
description: A quotation carried across a context summary has no re-measure rule, reads as the most authoritative sentence in the document, and cannot be falsified by reading code — it reached a commit message before anyone checked
metadata:
  type: feedback
---

**Attribute a sentence to Ken only when a file on disk says he said it, and
name the file. An inherited session summary is not a file on disk.**

**Why:** 2026-09-13. A context summary carried into a fresh session opened
O196 with a sentence in quotation marks attributed to him — *"the export
windows forget everything. every time I export a dxf I have to set it up
again."* It was used to justify the work, quoted in the commit message of
`28f5389`, and written into `RESUME.md`. Then `OPERATOR_REQUESTS.md`'s O196
row turned out to be headed **"FOUND 2026-09-13, not yet reported by the
operator"**, with a whole paragraph arguing why a row he never filed belongs
on the list. A grep of every `.md` and `.txt` in the repository found the
sentence in exactly two places and both were mine.

★★★ The asymmetry is the point. This project re-measures every **number**
that crosses a summary, by standing rule, because a number has a command that
produces it. A **quotation** has no such command. It is also the single most
authoritative kind of sentence a document can contain — nobody argues with the
operator's own words — and it is the only kind no reader can falsify by
reading the code. So the class with the least protection is the class with the
most weight.

**How to apply:**

- Before writing *"his words:"* or a blockquote attributed to him, grep the
  repo and `D:\Dev\FeatureRequests\` for the sentence. If the only hits are
  things you wrote, it is not sourced.
- `OPERATOR_REQUESTS.md` is the register and it records provenance
  explicitly — rows are marked **reported by him** or **found by us**. Read the
  row's heading, not just its body. That heading is the citation.
- A misattribution already published in a commit message is corrected **in the
  register and in `RESUME.md`**, not by rewriting history. A future session
  reads those; it does not re-read old commit messages.
- The same rule covers the softer form: *"he asked for X"* when what actually
  happened is that a sweep found X and we judged he would want it. Say which.

Related: [[a-backlog-row-is-a-record-not-evidence]],
[[a-capability-claim-in-product-copy-needs-the-same-citation-as-a-limitation-claim]],
[[a-citation-that-supports-my-theory-may-be-about-a-different-document]],
[[operator-requests-live-in-a-file-not-a-conversation]].

---

**★ 2026-09-19 — a FORMULA carried across a summary is worse than a count,
because it reads as re-derivable and nobody re-derives it.**

An inherited summary described `fixtures/paragraph.pdf` as having its text
baselines at `y = 704 − 16·index`. `fixture::text_block_target` actually states
them literally — 700, 684, 668, 652, 636, 620 — and `704 − 16·index` is the
formula for `text_chunk_point`, the **aim** point, which is `baseline + 4`. One
helper's arithmetic had been carried across the summary under the other
helper's name.

The plan built on it put a driven rubber-band's floor at `y = 668`, which under
the stated geometry is **exactly line 2's baseline**: a band whose edge lies on
a glyph row, so whether it took two lines or three would have depended on the
aim conversion's rounding. It would have passed, intermittently, and read as a
selection defect when it failed.

★★ A raw count (*"61 gates pass"*) announces itself as a measurement and this
project re-measures it by standing rule. A **formula** announces itself as
*derived*, which reads as self-justifying — and a formula naming the wrong
subject is wrong by a constant at every index, so it is internally consistent
and every spot-check agrees with it.

**How to apply:**
- Before aiming anything at a fixture, open the fixture **helper** and read the
  numbers it states. Never take geometry from a summary, a prior plan, or a
  check header — including one you wrote.
- Treat *"baseline"*, *"top"*, *"aim point"* and *"box"* as four different
  numbers until the helper says otherwise. The difference here was 4 pt and it
  was the whole error.
- Aim a driven gesture along the axis whose numbers the fixture **states**, and
  leave several points of clearance from the nearest thing it must miss. An
  edge that lands on a stated coordinate is a coin toss delegated to rounding.
- Related: [[a-verbatim-quotation-of-another-files-count-goes-stale-invisibly]],
  [[a-measured-note-that-names-the-wrong-axis-is-wrong-by-the-aspect-ratio]].
