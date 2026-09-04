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

## ★★★ AND HE DOES NOT ALWAYS SPEAK TO THIS SIDE — 2026-09-04

The rule above assumes his ask arrives in this conversation. On 2026-09-03 he
said, to the **engine** session:

> *"can you add the ability to export page(es) to png, jpg, svg. note that there
> had better be full support (including transparency where supported!). Also I'd
> like to be able to copy and paste anything to other software - like copy and
> paste vector graphics into word or inkscape for example if possible."*

The engine shipped **all of it** the same day across four passes and sent a note
— *"here is what a shell wires"* — with every call, the clipboard format order
validated against a real Word paste, and a 60-line worked example in the CLI.

**This shell built none of it, and there was no row.** Found a day later only
because a session happened to read the request channel looking for something
else.

⇒ **A note in `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\` can carry an
operator request, not just an engine fact.** Its header said *"informational, no
reply needed; consume when convenient"*, and "when convenient" never arrives on
its own: no gate reads that directory, no test fails, and the file sits in
`open/` looking like reference material.

**How to apply, and the first one is the cheap fix:**

1. **Read the channel at the START of every session, and grep the notes for his
   own voice** — a `>` blockquote attributed to him, "the operator asked", "his
   words". Anything quoting him is a request that needs an `O###` row on this
   side, whichever session he said it in.
2. **File the row before doing the work**, even when the work starts
   immediately. The row is what makes it visible that we owe something; a
   half-built feature with no row is indistinguishable from a feature nobody
   wanted.
3. **Treat `[x] core / [x] cli / [ ] gui` in the engine's `FEATURES.md` as a
   BACKLOG** — those rows are the engine telling us, in a machine-readable
   place, exactly what it has that we have not wired. Four of them were open
   for a day on this one.

★ Same family as the encryption verbs found the same morning: `set_encryption`
and `set_permissions` shipped with **no note at all**, and only
`check-verb-coverage.sh` noticed. That gate exists because a capability landing
unannounced is normal, not exceptional. **There is no equivalent gate for a
capability announced in prose**, and this incident is the argument for one.
