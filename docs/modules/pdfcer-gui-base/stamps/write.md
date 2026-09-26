# `stamps::write` — turning a [`Plan`] into a file Acrobat will load

Four engine calls in a fixed order, and the order is the whole content of
this module. Everything else here is disclosure.

```text
pageops::extract(view, plan.pages_to_extract())  →  bytes of a NEW document
Document::from_bytes(bytes)                      →  reopen those bytes
settings.open_session(doc)                        →  the SESSION funnel
  .set_info_field(InfoField::Title, category)    →  the CATEGORY
stamp_file::name_stamp_pages(&mut session, …)    →  the /Names → /Pages tree
session.to_full_bytes(options)                   →  the file
```

## Why it extracts instead of editing the open document

Because the open document must survive the operation completely untouched.
`Save as stamp collection…` is a **Read-mode-legal act** by the operator's
own standing rule — *Read may produce a new document; it may not modify
this one* — and the cheapest way to honour that is never to have a mutable
handle on his document at all. `pageops::extract` reads a `DocumentView`
and returns bytes; nothing upstream of it can be changed by anything
downstream of it.

The view is the **session's**, not the loaded file's, which carries his
unsaved edits into the collection. `app::actions::extract` makes the same
choice for the same reason (decision 018), and the alternative — silently
writing the file as it was opened — is the kind of wrong answer that looks
completely right.

## Why it reopens the bytes rather than reusing a session

`name_stamp_pages` names `stamps[i]` to **page `i` of the session it is
given**. Handing it the operator's session would name his drawing's pages.
The extracted bytes are the only document whose page numbering matches the
plan, so they have to become a document before they can be named. The
reopen costs one parse of a file we just built and buys the positional
contract for free.

⚠ It also means **a failure to reparse our own output is reachable**, and
is reported as its own outcome rather than folded into "extraction failed".
If pdfcer ever writes bytes pdfcer cannot read, that is a finding about the
engine and the operator should not see it described as something his file
did.

## What is disclosed, and why each one

Under R8b rule 4 every inference this path makes is reported **off-canvas**
— here, in the dialog that is about to write the file, before it writes.
[`Written`] carries them; the dialog turns them into sentences. Nothing in
this module marks a page, because nothing in this module has a page to
mark.

## Item notes

### `struct Written`

Returned rather than logged because the operator is entitled to all of it
**before** the file lands somewhere Acrobat will read it — a stamp
collection that silently dropped one stamp is a picker with a hole in it,
discovered weeks later in the middle of signing something.

### `enum WriteFailure`

One variant per stage, because each stage fails for a different reason and
there is a different sentence to say about each. Collapsing them would hand
the operator the shrug this project's text conventions forbid.

### `fn detail`

Returned rather than re-worded. `crate::text` owns the *frame* —
which of the four things failed — and the engine owns the detail, for
the same reason `app::save`'s refusals quote rather than paraphrase: a
sentence this shell invents about a failure it did not diagnose is a
sentence that will eventually be wrong.

### `fn build_and_write`

Kept separate from [`build`] on `app::actions::extract`'s rule: the half
that touches the filesystem and the half that does the work are separable in
the reading as well as in the testing, and only one of them needs a
temporary directory to exercise.

# Errors

[`WriteFailure`] from [`build`]. A failed `std::fs::write` is traced and
reported through the return value's `Ok(None)`-shaped absence — see below.
