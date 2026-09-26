# `app::actions::reviewstate` — recording a comment's review status

One verb: [`RecordStatus::record`], which is
[`pdfcer_core::edit::EditSession::add_review_state`] behind the edit funnel.
Raised by [`crate::app::actions::Action::RecordReviewState`], which is
raised by [`crate::panels::comments::reviewstate`]'s *Record status*
chooser, and by nothing else. [`RecordStatus`] is that action's payload, and
its doc carries why the verb is not an `AnnotAction`.

## Why it is not in [`super::annots`]

That module's own header draws the line it lives on: *"this file is what
happens to a thing that already exists"* — the verbs whose subject is an
annotation the operator can see, and which **change** it. Recording a status
changes nothing about the annotation it names.
[`pdfcer_core::edit::EditSession::add_review_state`] says so in the
strongest terms available to it:

> *"THE STATUS IS NOT WRITTEN ONTO THE ANNOTATION IT DESCRIBES —
> §12.5.6.3 puts it on a **separate** `/Text` annotation that points at the
> reviewed one through `/IRT` … That is why this verb returns a new `ObjId`
> rather than mutating the target, and why nothing about the target
> changes."*

So this is an **addition**, and it belongs with the placement verbs by
subject even though it is reached from a list of existing ones. Its own file
rather than a third home, under R2 and for [`super::funnel`]'s reason: one
subject, one rate of change.

## What this module is careful NOT to do

**It does not decide what the current status is.** The engine ships no
resolver, deliberately — *"it says nothing whatever about ordering or
currency … a `/M`-sorted resolver would be guessing"* — and neither does
this. `add_review_state` walks the `/IRT` chain itself and reports where it
attached; this module reports what it was told and adds nothing.

**It does not draw anything on the canvas.** R8b: a review status is a
disclosure about a comment and lives off-canvas, in the panel and on the
status row. Nothing here touches an appearance.

## Undo is one press, and the engine made sure of it

A status is two dictionary keys on a new annotation, and the obvious
implementation writes the annotation and then the keys — two commands, two
undos, and a half-undone status that is a `/Text` with an empty
`/Contents` and no `/State`, which renders as an empty note nobody wrote.
`add_review_state` does not do that: it goes through `add_reply_with`'s
`extra` seam *"so the two keys land in the SAME command and one undo removes
the whole status"*, and the stack entry is
[`pdfcer_core::edit::CommandKind::AddReviewState`]. So this module needs no
grouping of its own, and the safety net under the *Record status* chooser is
the ordinary one press of Undo — which is why the control asks no
confirmation, exactly as `super::annots`' Delete does not.
