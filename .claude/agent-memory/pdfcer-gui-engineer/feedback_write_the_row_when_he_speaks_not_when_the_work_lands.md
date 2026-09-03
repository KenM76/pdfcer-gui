---
name: write-the-row-when-he-speaks-not-when-the-work-lands
description: Log a request into OPERATOR_REQUESTS.md the moment Ken makes it — three of his 2026-09-01 requests were built but never filed, and the work getting done is exactly why nothing looked wrong
metadata:
  type: feedback
---

Enter the row in `OPERATOR_REQUESTS.md` **when he speaks**, before any work
starts — not when the work lands, and not "once I know what the answer is."

**Why:** on 2026-09-01, writing a handoff, I grepped that file for his own words
and got **zero hits** for three requests he had made that day: the OCR progress
feedback with Stop and Cancel, copying OCRed text, and selecting an object
dropped off the side of the page. All three were asked for. All three were
worked on. **None had ever been entered.** That is rule 1 of the contract he set
that file up to enforce — *"Every request you make goes in this file, the moment
you make it, before any work starts on it."*

★★ **The work getting done is precisely why nothing looked wrong.** There was no
missing feature to notice. The requests existed only in a chat transcript and in
commit messages — neither of which he reads, and neither of which a cold session
opens. A row is not a to-do list; it is the **record**, and the record is what
survives a session ending.

★ Back-filling them also corrected two things I would otherwise have carried
forward as shipped: the OCR progress UI had never been **driven** (so under R1 it
is not shipped, whatever the unit tests say), and the off-page selection had
shipped only its Select All half. Writing the row late meant the status was
wrong in the only place he would look.

**How to apply:** the instant he states a want — even mid-sentence, even as an
aside inside a message about something else — stop and write the row. Mark
back-filled rows as back-filled so the dates are not misread as evidence of a
process that worked. Related: [[feedback_operator_requests_live_in_a_file_not_a_conversation]],
[[feedback_a_backlog_row_is_a_record_not_evidence]],
[[feedback_scope_a_request_to_the_whole_expected_behaviour]].
