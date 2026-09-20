---
name: delete-the-workaround-when-the-cause-is-removed
description: When the engine removes the limitation a workaround existed for, delete the workaround the same day — do not keep it as a spare part
metadata:
  type: feedback
---

When `pdfcer-core` ships the verb that removes the limitation a shell-side
workaround was built for, **delete the workaround in the same session**.

**Why:** on 2026-08-27 I built `save::replace_file`, `Action::ReopenActive`,
`PendingIntent::Reopen` and two copy strings — about 300 lines, all
correct, all documented — as the honest way to give the operator
"save into my own file" when `ocr::layer::add_ocr_layer` could only
return a `Vec<u8>`. The engine landed `EditSession::add_ocr_layer` **the
same morning**. Keeping the mechanism would have left a well-documented
path with no caller, which is a thing that rots and that a later session
maintains, extends, or reasons from. The engine session said the same in
its own words: *"the interim you said you would ship anyway is no longer
needed."*

The reverse risk is real and is the one to weigh: deleting something that
still has a caller. The compiler answers that in a minute — remove it and
build. **Dead-code warnings are the instrument, not a judgment call.**

**How to apply:** every session, read `open/` in the request channel
*before* planning. A request filed yesterday is very likely answered
today — the engine session turns them around within hours. If the answer
obsoletes something you shipped in the interim, that removal is part of
consuming the answer, not a separate tidy-up to schedule. Say so in the
commit message so the deletion reads as intentional rather than as a
revert.

Related: [[the-engine-session-runs-in-parallel-and-answers-within-the-hour]],
[[never-defer-on-an-external-blocker]] — that rule says ship something now;
this one says take it back out when the real thing arrives.
