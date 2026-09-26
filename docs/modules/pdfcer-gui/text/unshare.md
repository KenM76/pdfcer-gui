# `text::unshare` — every sentence "give this page its own copy" can say


## Why this feature needs the biggest refusal catalog on the canvas

Because **the refusals are the feature's whole shape**, and because the
commonest one is not an error at all.

`EditSession::unshare_form` takes `(page_index, form: ObjId)` and clones one
form XObject's stream so that this page — and only this page — names the
copy. Every other invocation site keeps naming the original and is left
byte-identical. That is a **structural** edit: it allocates an object, it
rewrites the page's `/Resources`, and it therefore runs the same guard
ladder every structural verb in the engine runs (encryption, certification,
`/Size` suppression) before it does anything at all.

⇒ So a control that says *"give this page its own copy"* declines for
**several distinct reasons** — the list is [`UnshareRefusal`]'s variants and
this sentence deliberately no longer counts them, for the reason
`tools/gates/run-all.sh`'s header spends six corrections on: *a number
written in prose beside the thing it counts is a claim that decays*. Two of
them — [`UnshareRefusal::Nested`] and [`UnshareRefusal::NotShared`] — are
considered design positions rather than limits, and none of them is
visible on the page. The operator presses a button, the drawing looks
identical (it *must*: the copy is byte-identical to the original, which is
the point), and without a sentence the only difference between success and
every failure is a status row that says nothing either way.

This is the project's founding defect shape with the volume turned up:
*a gesture that is made, is refused, and reports nothing.* `DEFECTS.md` D4a.
And it is worse here than for a drag, because a successful unshare also
looks like nothing happened — see [`unshared`], which is why the success
path owes a sentence too.

## The vocabulary, decided once

| the file's word | the operator's word here | why |
|---|---|---|
| form XObject | **drawing** / **the shared drawing** | §8.10.1's own illustration is a CAD system's standard component; the operator calls their title block a drawing, not an XObject |
| invocation | **place it is drawn** | an invocation is a `Do` operator; a place is something they can point at |
| page | **sheet** *(only where the fan-out is the subject)* | a 36-sheet drawing set is "sheets" in every room this software is used in. Elsewhere "page", because that is what the page box in the status bar says |
| `ObjId` | **not named at all** | see [`unshared`]: an object number is evidence, and evidence goes to the trace |

[`crate::text::rotating`]'s rule is inherited unchanged: **name the thing
the operator can see, never the thing pdfcer models.** A refusal phrased in
the file format's vocabulary reads as an internal error, and an internal
error is a thing an operator reports rather than acts on.

## What is deliberately NOT worded here

**Nothing.** That is unusual in this directory and it is the point: every
`EditError` this verb can return has a variant below, including the three
that are unreachable on a well-formed file. [`crate::text::rotating`]'s
argument for keeping its two unreachable variants is the argument for all of
these — *"a routing bug with a sentence is a bug report; a routing bug
without one is a handle that does nothing"* — and this verb has more ways to
be routed wrongly than a rotation does, because its operand is derived
through two hops (a selected leaf, then that leaf's outermost enclosing
form) rather than being the thing that was clicked.
