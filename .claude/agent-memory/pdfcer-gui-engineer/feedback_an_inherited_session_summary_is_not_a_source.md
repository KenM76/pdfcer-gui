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

Related: [[feedback_a_backlog_row_is_a_record_not_evidence]],
[[feedback_a_capability_claim_in_product_copy_needs_the_same_citation_as_a_limitation_claim]],
[[feedback_a_citation_that_supports_my_theory_may_be_about_a_different_document]],
[[feedback_operator_requests_live_in_a_file_not_a_conversation]].
