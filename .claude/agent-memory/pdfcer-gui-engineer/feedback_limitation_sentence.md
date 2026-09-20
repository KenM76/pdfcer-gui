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
recurrence — see [[never-defer-on-an-external-blocker]] and
[[a-backlog-row-is-a-record-not-evidence]] for the same shape from other
directions, and [[the-engine-session-runs-in-parallel-and-answers-within-the-hour]]
for why the window is hours rather than days.

## ★★★ SEVENTH AND EIGHTH — 2026-09-13, twenty-one minutes, and I wrote them into the release

Two sentences, written at roughly 09:00 into `RESUME.md` while preparing a
release:

1. *"`request_G013` … **is unanswered at the time of this release**."*
2. *"⚠ **O194 clause 3 (km, miles, yards) is blocked on the engine** — filed
   08:58, unanswered at this release."*

`reply_G013` landed at **09:19**. Both were false twenty-one minutes after
being written, and I found them at about 10:00 — after they had also been
copied into the drafted release notes and the drafted commit message, because
prose written once gets **transcribed**, and a transcription carries the shelf
life of the original with none of its date.

⇒ **The new part of the lesson is the transcription.** The earlier recurrences
were one stale sentence in one place. This one was one stale sentence in four
places within an hour, because a release ritual copies claims from `RESUME.md`
into `FEATURES.md`, the release notes and the commit message. **A limitation
sentence written during a release cut propagates faster than it decays.**

⇒ **Concrete rule, cheap:** before writing *any* "unanswered / blocked /
not possible" sentence, `ls` the request channel in the same breath. It costs
one command. And when one is found false, **strike it in place with the old
wording quoted** rather than deleting it — the old state is worthless, but the
shelf life is the finding, and a silently-corrected document teaches nobody.

★ What made this recoverable rather than shipped: the reply's own closing
instruction (*"Consume this reply when you have read it"*) forced a read of
`open/` before packaging. **The request channel has no notification, no gate,
no git and no undo.** Nothing in this repository will ever tell you a reply
arrived — see [[the-engine-session-runs-in-parallel-and-answers-within-the-hour]].
