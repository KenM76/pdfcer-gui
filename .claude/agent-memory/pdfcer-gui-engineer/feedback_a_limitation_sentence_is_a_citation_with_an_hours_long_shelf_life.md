---
name: a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life
description: A sentence about what pdfcer-core cannot do goes stale within hours; spell such claims as test assertions, never as prose
metadata:
  type: feedback
---

A sentence in our source saying "the engine cannot do X" is a **dated citation
with a shelf life measured in hours**, not a verdict. Where the claim can be
spelled as a test assertion, spell it as one.

**Why:** on 2026-09-03 the same paragraph was wrong twice in one morning, in
opposite directions. Written first from `D:\Dev\pdfcer`'s *working tree* — which
described vector cutting the engine had not committed; the compiler caught it,
because a field did not exist on our pin. Corrected to "vector-path redaction is
not implemented this build", citing the pinned hash, with a careful paragraph on
why the dirty tree must not be trusted — and within the hour v0.27.0 shipped and
the correction became the false half.

★★ The half that behaved well says what to do. The unit test
`a_region_over_an_image_refuses_the_whole_apply` **went red the moment the engine
shipped**, and that red was a *report*, not a regression. The prose version of
the identical claim, living in a UI string, compiled and passed for as long as it
was false, and was corrected only because the engine's reply said "re-word this"
by name. Nothing in this repository could have caught it —
`check-stale-blockers.sh` cannot see a claim phrased as operator copy.

**How to apply:** before writing any sentence about an engine limit, re-resolve
the pin (`cargo update`, then read `Cargo.lock`) rather than reading
`D:\Dev\pdfcer`'s source, which is dirty by design because that session runs in
parallel. Then ask whether an assertion can carry the claim instead. Sixth
recurrence — see [[feedback_never_defer_on_an_external_blocker]] and
[[feedback_a_backlog_row_is_a_record_not_evidence]] for the same shape from other
directions, and [[project_the_engine_session_runs_in_parallel_and_answers_within_the_hour]]
for why the window is hours rather than days.
