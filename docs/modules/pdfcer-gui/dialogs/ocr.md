# `dialogs::ocr` — the Recognise-text transaction

One dialog, three states, and a shape that is chosen rather than
conventional. It is the surface for `file.ocr`, and it is also the
**enforcement point for two rules** that would otherwise have nowhere to
live in this build.

## The three states

| state | what the operator sees | what exists |
|---|---|---|
| **ready** | what OCR does to the page, and one button that starts it | nothing |
| **working** | *Recognising…* | a thread |
| **answered** | the disclosure, then a Save-as button — or a named refusal | bytes, in memory |

## ★ Why the recognition is disclosed BEFORE it is written, not after

This is the whole reason the dialog has a third state instead of running
OCR and immediately opening a file picker.

Project rule 4 is *"fuzzy, never sneaky"*, and `pdfcer-core`'s own OCR
header sharpens it for exactly this feature: **every word an OCR layer
contains is a guess**, and this engine reports no confidence for any of
them. A surface that recognised a page and dropped the finished file in
front of the operator would be technically disclosive — the report would be
*somewhere* — while being, in practice, a program that silently inserted
several hundred unreviewed inferences into a document. This project's
characteristic failure is a surface that is *correct* and *unreadable at the
moment it matters* (`DEFECTS.md`).

So the order is: recognise, **show what was inferred**, and only then offer
to write it. The operator reads the disclosure while holding the one thing
that gives it force — the ability to not save. That is not a nicety; it is
the difference between a disclosure and a receipt.

## ★ Why the write is a Save-as, in every mode

The standing rule is *Read may produce a new document; it may not modify
this one*, with the enforcement at the **save** rather than at the
operation.

★ The rule is **vacuous** in this shell, and that is worth saying rather
than leaving as an apparent guarantee: `file.save_copy` asks for a
destination too, and `crate::app::save::suggested_path` guarantees the
*suggestion* is never the file that was opened, exactly as
[`suggested_path`] below does here. The two surfaces share one picker,
`crate::app::files::pick_save_path`, and the only thing that differs between
them is the dialog's title.

It bites here in the only way that is
honest: the destination is a path the operator names, so the rule holds in
Read **and** in Edit **and** in Review by construction rather than by a mode
check. Nothing here consults the mode, and nothing here should — the rule is
about what a save may overwrite, not about who is asking.

What is deliberately **not** done: no second save command, no in-place path,
no `Save`-labelled control anywhere. The day in-place `Save` lands it will
need its own Read-mode gate, and that gate belongs beside it rather than
being invented here in advance against a command that does not exist.

## ★ Why OCR is available in Read, with no capability flag

`app::modes::capability` governs **gestures** — what a press on the canvas
means. OCR is not a gesture; it is a command with a dialog, and it changes
no document that is open. Adding a capability flag for it would put a rule
about *saving* into the machinery that decides what a drag does, where the
next reader would neither look for it nor believe it.

Read is therefore offered OCR exactly as Edit is, and that is the operator's
instruction rather than an omission.

## Why this dialog does not push an `Action`

[`super`]'s rule: a dialog uses the action funnel when it edits **this**
document, and this one never does. The recognised bytes are a *new*
document; the open one is untouched, its `edit_epoch` does not move, and
there is nothing to order against or to undo. What the funnel's reasoning
does still demand is that irreversible work not happen part-way through a
layout pass — and it does not: the button sets a flag, and the file is
written after the window's closure returns.

## What is document-scoped about it

Everything. A recognition is of *this page* of *this file*, so
[`super::DialogsState`] holds it in the document-scoped group and closing
the document closes it. A finished-but-unsaved recognition is discarded
with it, which is the right answer: writing it afterwards would produce a
file derived from a document the operator has already put away.
