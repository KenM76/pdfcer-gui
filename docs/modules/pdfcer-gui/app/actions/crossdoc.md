# `app::actions::crossdoc` — pages dragged out of one open document and
into another

One arm, and it is here rather than in [`super::pages`] because it is the
only edit in the application that reads **two documents at once**. Every
other verb takes a `&mut OpenDoc` and is done; this one needs a
`DocumentView` over a *parked* session while it holds `&mut` on the active
one, which is a different shape with a different set of things that can go
wrong.

---

## 1. The gesture

The operator presses on a page tile in the Pages panel, drags — spring-
loading a document tab on the way if the destination is another document
([`crate::app::doctabs`] §3) — and releases over the destination's page
list or page view at a caret between two sheets. See [`crate::pagedrag`]
for the state that survives the document switch in the middle.

## 2. It is a COPY, and the reason is undo

[`crate::text::doctabs::drag_landing_other`] carries the argument in the
words the operator reads. The engineering form:

A cross-document *move* is two edits — an insert into the target and a
delete from the source — recorded on **two independent undo stacks**,
because `EditSession` owns one command log per document and this
application has one session per document. There is no ordering of those two
commands under which a single Ctrl+Z means *"undo what I just did"*: undo
goes to whichever document has focus, so the operator gets half of their
gesture reversed and no indication that the other half is still applied.

Half-undone is worse than not-undone, and much worse on a drawing set,
where the evidence is a page count nobody checks.

Windows Explorer reaches the same conclusion from a different direction and
copies between volumes by default. So this is a copy, and the caption says
so before the operator releases the button.

## 2b. And Shift makes it a move, which is the operator's call

Requested 2026-08-20: *"can you also make it so you can move the pages
between documents instead of just copy by holding one of the keys like shift
or control, whichever on windows uses to switch from copy to move
operation."*

**Shift.** Windows has bound the drag modifiers the same way since the
mid-nineties — Ctrl copies, Shift moves, Ctrl+Shift makes a shortcut — and
[`crate::text::doctabs::drag_landing_move`] carries the table. Copy is
already what an unmodified cross-document drag does here, so Ctrl asks for
what it already gets and Shift is the modifier that changes the verb.

Everything §2 says about undo is still true, so **it is disclosed rather
than designed away**: [`crate::text::doctabs::moved_out_of`] states in words
that one Ctrl+Z reverses one half of a move, on the status row, immediately
after it happens. That is the honest shape — the operator asked for a
capability whose cost is real, so they get the capability *and* the cost,
rather than a refusal that protects them from a choice that is theirs.

### The order is insert, then delete, and it cannot be the other way

The source's pages are removed **only if the target's insert actually
happened**, which is why [`super::pages::insert_from_view`] returns a count
rather than nothing. Deleting first — or deleting regardless — would lose
the operator's sheets to a refusal they never saw: a certified target, an
encrypted one, a page tree that will not walk. All three are reachable and
all three decline silently as far as the source document is concerned.

And if the *delete* is the half that fails, the pages are now in **both**
documents. That is a third state, neither of the two things anybody asked
for, and [`crate::text::doctabs::move_left_the_source_alone`] says so
plainly. Silence there would leave an operator believing they had moved
something they had duplicated.

## 3. What the source document is guaranteed **when it is a copy**

**Nothing is written to it and nothing is read out of it destructively.**
The engine takes a `DocumentView` — a read-only projection — and copies
every object it needs at fresh object numbers in the *target*. The source's
`EditSession` is not borrowed mutably, its undo stack is untouched, its
`is_modified` answer does not change, and its tab does not acquire the
unsaved marker.

That is worth stating because it is the property that makes the gesture
safe to try. An operator who drags the wrong sheet has changed one document
and can undo it there.

## 4. What does not come across, and why the operator is told at the moment
it happens

Exactly what [`super::pages::insert_from_view`] reports for an insert from
a file, through the same [`crate::text::pages::inserted`] sentence: page
content, resources, fonts and XObjects arrive; the source's **document-level**
structures — outlines, the AcroForm field tree, named destinations, page
labels — do not, because merging those rewrites objects an incremental save
exists in order not to touch.

R8b rule 4's surviving half is the reason this is disclosed rather than
left to be discovered: *"inferences the operator cannot see … still owe an
off-canvas report"*. A form field whose widget arrived without its
definition looks exactly like a form field until it is filled in.

## Item notes

### `fn take_pages_from`

# Why the disclosure is stamped with the TARGET's epoch

Because the target is the document on screen. `crate::app::status` draws
the **active** document's disclosure and nothing else, so a sentence
filed against the source's revision would be recorded, correct, and
invisible — the shape of failure `app::actions::vector_edit`'s own
header calls *recorded, not disclosed*.

### `fn apply_insert_from_open_document`

**The target is always the active document**, and is not carried by the
action. That is not an omission: the drop landed on a surface, the
surface was showing the active document, and a slot carried from the
press would name whatever was active when the *drag started* — which,
with spring-loading, is precisely the document it is not.

# How it declines, and why each refusal is silent or spoken

| condition | what happens |
|---|---|
| the source tab has gone, or never held an open document | traced, nothing said — unreachable without a close mid-drag, and there is no remedy to offer |
| nothing is open to drop into | traced, nothing said — same |
| the source **is** the target | traced and **refused**; a same-document drag is a reorder and reaches a different arm entirely |
| the engine refuses the insert | `vector_edit`'s own decline path, which puts the engine's reason on the status row |
