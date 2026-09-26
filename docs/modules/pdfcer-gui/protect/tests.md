# `pdfcer-gui/protect/tests`

## Item notes

### `fn scratch`

Named per test rather than shared, because `cargo test` runs these in
parallel and two tests writing one path is a flake that reproduces about a
third of the time — the worst kind.


⇒ The doc comment above named the hazard and fixed half of it, and a
half-fix under a confident note is worse than no note: the next reader
sees the hazard named and stops looking. `tools/gates/check-test-temp-paths.py`
is what stops the other half coming back.

### `fn the_permission_model_round_trips`

The whole permission model, end to end: a chosen subset goes into
[`prepare`], through `EditSession::set_encryption`, out as bytes, back in
through the reader, and out of [`Standing::read`] as the same subset.

It asserts the **negative** half as hard as the positive one — that
`Copy` comes back `Some(false)` — because the failure this catches is not
"the ticks were lost", it is "everything was granted anyway", which looks
like a success from the operator's side until somebody copies the drawing.

### `fn a_document_that_forbids_printing_does_not_open_with_everything_ticked`

> *"A permissions dialog that opens with everything ticked, on a document
> that forbids printing, has told the operator a falsehood before he touches
> anything."*

So: make a document that forbids printing, read it the way the dialog reads
it, and assert the box for Print comes back **unticked**.

It also asserts the unencrypted case in the same test, because the two
answers are what make each other meaningful: all-ticked is the *truth* about
a plaintext document and a *lie* about this one, and a `Standing::read` that
returned a constant would pass either assertion alone.

### `fn a_signed_document_is_refused_before_the_form_is_drawn`

R9: *no placeholders — the control is absent or explained, never a button
that fails on press.* Both tasks refuse, because every one of the three
engine verbs returns `EncryptError::SignedDocument` and there is no answer
the operator could type that would change it.

The count is carried, because *"this document carries 1 signature"* and
*"…carries 5"* are different problems and the operator is the one who knows
which is theirs.

### `fn permissions_on_an_unprotected_document_is_refused_by_name`

A PDF states what it allows only inside its `/Encrypt` dictionary. A document
without one does not *permit everything* — it says nothing — and drawing
eight ticked boxes would be this surface writing a sentence the file never
wrote.

### `fn changing_the_password_keeps_what_the_document_allowed`

`set_permissions` re-derives `/O`, `/U`, `/OE`, `/UE` and `/Perms` from a
whole `EncryptionSettings`, so a caller that did not pass the document's
current bits would grant **everything** — and the operator, who came to
change a password, would have un-restricted a drawing without being told.
[`Standing::preserved_grants`] is what stops that, and this is the assertion
that keeps it stopped.

It also asserts the password change itself in both directions: the new one
opens the file and the old one no longer does. Either half alone would pass
on a build that wrote the bytes out unchanged.


It was written asserting `preserved_grants() == [Print]` on a document
written with `permissions = [Print]`, and it **failed**, reporting
`[Print, AccessibilityExtract]`.

That is not a defect in [`Standing::preserved_grants`]. It is
`pdfcer_core::crypto::encrypt::assemble_permissions`'s rule **W19**, stated
in its own doc: *"bit 10 — writers `shall` always set it to 1 for
1.7-reader compatibility, regardless of whether accessibility extraction is
granted (at `/R` 6 the bit no longer gates it)."* The engine sets bit 10
unconditionally on the write path, and the read side then reports it as
granted, correctly, because the file does say so.

⇒ **`AccessibilityExtract` cannot be declined by anything pdfcer writes**,
and the assertion below now says so rather than being loosened. The
consequence for the surface is [`super::always_granted`] and
`crate::text::protect::accessibility_always_granted`: the row is drawn as a
fixed statement rather than as a tick-box, because a tick-box the operator
can clear and that comes back ticked in the written file is precisely the
falsehood this whole surface exists to avoid.

### `fn removing_the_password_produces_a_file_that_opens_with_none`

Asserted as an absence of `/Encrypt` rather than as "it opened", because a
document with an empty user password also opens with no prompt and is still
encrypted — and telling an operator their drawing is unprotected when it is
permissions-only would be the same class of lie this whole surface exists to
avoid.

### `fn the_user_password_will_not_authorise_a_change`

And it comes back naming which password *did* open the file, which is the
difference between a dead end and a next step — an operator told only
"wrong password" re-types the one they have.

### `fn an_unstated_bit_is_carried_over_as_granted`

`PermissionBit::applies_at`'s own doc states the rule this asserts: *"the
author of an `/R` 2 file did not decline to permit form-filling; the concept
did not exist to decline"*, and reporting it as refused would invent a
restriction nobody wrote. pdfcer writes `/R` 6, where all eight bits mean
something, so every bit must take a side — and the side silence takes is
*allowed*.

Built by hand rather than from a fixture, because the condition is a property
of the model and an `/R` 2 document is not needed to state it.

### `fn the_suggested_name_is_never_the_source_file`

The standing rule for every write that produces a second document. The
second half matters because the two files this surface can produce are
opposites: suggesting `-protected` for a removal would name the file after
the thing it no longer is.

### `fn the_write_lands_and_leaves_no_temporary_behind`

Asserted at the file rather than at the return value, because the failure
this guards is a rename that did not happen and a `.pdfcer-tmp` left beside
the operator's drawing.
