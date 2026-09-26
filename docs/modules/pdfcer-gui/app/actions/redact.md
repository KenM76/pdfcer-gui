# `app::actions::redact` — the three arms that MARK content for removal,
and the one that removes it

A file of its own under rule R2, and the seam is a real one rather than a
line count.

These are the only arms whose subject is **marking content for removal**.
They share a vocabulary nothing else in the funnel uses — `RedactAppearance`,
the mark census, the annotation ids a review surface addresses a mark by —
and their comments carry the argument for the one operation pdfcer cannot
undo. Moving the arms and leaving the reasoning behind would have been
exactly the split this project's own R2 note warns against.

## NOTHING IN THIS FILE REMOVES ANYTHING — and that is a property of the
engine, not a principle

Marking authors annotations. [`crate::redact::stage_into_session`] does not
remove anything either and does not touch the session's content: it **arms
the next save**. Written as a principle — *"the irreversible half can never
reach this funnel"* — that sentence would be a claim about an engine this
project does not build, and it would go false the day the engine grows a
verb that applies into the open session. It is stated as what it is: a fact
about the verbs that exist, re-checked rather than inherited.

⇒ The arm below is [`RedactAction::Pending`], and it goes through the
identical `vector_edit` funnel as the three marking arms, for a reason that
is worth stating because the funnel does more than this verb needs: the
engine's staging and cancelling verbs both take `&mut EditSession`, and
`Arc::get_mut` — which is the funnel's second step and cannot be had any
other way here — is what makes them reachable at all. The epoch bump comes
with it and is wanted for its own reason (below).

The hazard the principle was reaching for is real and has a different
answer: routing an operation that cannot be undone through a queue that
**replays** would be a defect. **This queue does not replay.**
`crate::app::actions` drains it once per frame in order and discards it.
And the arming is not irreversible — [`crate::redact::Staging::Cancel`] is
the second half of this very arm.

## Why the epoch is bumped for an edit that changes no pixel

Staging alters nothing a rasteriser would draw (rule 4: no badge, no tint,
no provisional layer — see `crate::redact` §1.0.3), so on the face of it a
funnel that drops every page texture and rebuilds the decomposition is pure
waste. It is bumped anyway, and the reason is one specific consumer:

`crate::app::save::has_unsaved_edits` is
`(is_modified() || has_pending_redaction()) && edit_epoch != saved_epoch`.
**Without the bump the second term is false**, and a document whose marks
were already in the file when it was opened — arm the removal, change
nothing else — answers *clean*, closes with no prompt, and loses the arming
in silence. The waste is one re-raster of an identical picture. The
alternative is the exact silent loss the third term was added to close.

## What is still NOT here

**The write.** This arm arms the save and stops. Where the bytes go, and
when, is `file.save` / `file.save_as` / `file.save_copy`'s decision, exactly
as it is for every other edit — which is what the operator asked for in
`OPERATOR_REQUESTS.md` O125. `crate::app::save::write_copy` is what routes
them through the removal. The two *write-now* destinations still live in
`crate::dialogs::redact` and still reach no arm in this file.
