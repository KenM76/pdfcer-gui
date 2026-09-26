# `app::actions::export` — writing part of the document out as something
else

## Why this is its own file

The sixth sibling of [`super::apply`], drawn along the same seam the other
five are: *what class of thing does this verb act on?* — pages there,
annotations in `annots`, the dimensioning model in `dimensions`, page
content in `apply`, redaction marks in `redact`. This is **what leaves the
document**.

It is a real subject rather than a size-driven cut, and the evidence is the
property every verb here shares and no verb elsewhere does: **none of them
changes the document at all.** No `vector_edit`, no undo entry, no epoch
bump, no cache invalidation. They read the open file and write a different
one, which makes every rule the mutation funnel enforces irrelevant to them
and every rule about *file* handling — a save picker, an overwrite, a
partial write — apply instead.

`super::pages::extract` is the same shape and stayed in `pages` because its
subject is a page set. If a third export lands, that is the moment to move
it here.

## The third export landed, and the trigger above FIRED

[`image`] — `OPERATOR_REQUESTS.md` **O120**, PNG / JPEG / SVG — is the third,
and it is written here rather than in a module of its own, which is the
easier half of what the sentence above asks for. The harder half is
**`pages::extract` has not moved**, and that is recorded rather than quietly
not done:

* The condition is met. Three exports now exist and the family is real.
* Moving `extract` is a change to `pages`, to `apply`'s dispatch and to
  whatever names it, made in the same pass as a new feature — and this
  project's own record of what that produces is `RIBBON_IA.md`'s repeated
  lesson that a taxonomy move and a capability arriving together make a diff
  nobody can review as either.

⇒ So the trigger is left **armed and stated** rather than silently reset. The
next reader of this header is looking at a condition that has fired, with
the reason it was not acted on written beside it, which is the shape this
project uses everywhere else for a decision deferred on purpose. What must
not happen is the sentence above being read as still-waiting: it is not.

## Why an export is an `Action` at all

`super::apply`'s header answers it for `SaveCopy` and the answer is the same
here: **a native file dialog must not open inside a layout pass.** It is a
modal OS window that blocks the thread, and opening one from a widget's
`clicked()` branch means egui is part-way through building a frame that will
not finish until the operator has answered.

Nothing about the document is being ordered — there is nothing to order —
so the funnel's *invariant* does not apply. Its **reason** does.

## Item notes

### `fn suggested_form_path`

Beside the document and named after it, with `.fdf` — [`suggested_path`]'s
rule and its reason. The extension is the **default format** as well as a
suggestion, which is why it is FDF: see [`form_data`]'s header.

### `fn suggested_path`

Beside the document, named after it, with a `.dxf` extension. The same rule
`super::pages::suggested_path` follows for an extract, and for its reason: a
picker that opens in the last-used directory of some other application is a
picker that makes the operator navigate back to their own project every
time.

### `struct Output`

A struct rather than a tuple because the caller has to pick a *different
sentence* per kind, and a `(Vec<u8>, u32, u32, usize, Vec<String>)` would
carry two fields that are meaningless for one of the two branches.

### `enum Failed`

[`Self::Render`] is about the **page** — it will happen again whatever
format is chosen, and the resolution or the document is the thing to look
at. [`Self::Encode`] is about the **format**, and its commonest cause is
`ExportError::TooLargeForJpeg`: JPEG stores its dimensions in sixteen bits
(ITU-T T.81 §B.2.2), so a raster over 65,535 pixels on a side has no JPEG
form at all while having a perfectly good PNG one.

⇒ A single "export failed" would send an operator whose only problem is a
format's arithmetic limit off to investigate their drawing. Each carries
the engine's own message, which names the numbers.

### `fn emf_bytes`

# Why this is a third function and not a flag on [`svg_bytes`]

The two share a *recording* — `pdfcer_render::emf` walks the same export
display list `pdfcer_render::svg` does — and share nothing else. Different
options type, different outcome type, different disclosure, different
receipt line, and a different answer for every one of the five things EMF
cannot express. A `if metafile { … } else { … }` inside one function would
be two functions sharing a brace.

# The background follows [`svg_bytes`]'s rule, for the same reason

`EmfOptions::background` is `Option<Rgb>` exactly as `SvgOptions`' is, and
`None` is the transparent state — the engine's CLI calls it *"EMF's
natural state (nothing is drawn where nothing was painted)"*. So the same
flag drives both, and the `RenderOptions` backdrop the caller set is again
**not** what decides it.

⇒ This is the shape of mistake that ships a window promising a clear
background and a file with a white rectangle at the bottom of it, and it
is worth the second comment because the two option structs are the two
places in this file where the backdrop is a decoy.

# Nothing here validates the metafile, and that is a decision

`pdfcer_render::emf::walk_records` exists precisely so a consumer can
check a metafile's record structure before handing it to
`SetEnhMetaFileBits`, and it is deliberately not called on this path. A
**file** export hands the bytes to `std::fs::write`, which cannot be made
to misbehave by a malformed record; the reader that would choke on one is
somebody else's program, tomorrow. Walking every record to produce a
sentence nobody could act on would cost a second pass over the whole
metafile on every export.

The **clipboard** path is the one that owes this check, because there
`SetEnhMetaFileBits` is handed a raw buffer and a bad one is a GDI failure
rather than a refusal. See `crate::clipboard`, which is where that call
will live when the placement half is buildable.

### `fn honesty_notes`

A helper rather than four inline `if`s at two call sites, because **both**
the refusal path and the success path owe exactly this set. A document whose
every page is Identity-H-without-`/ToUnicode` refuses with a zero character
count that looks identical to a scan, and the counter is the only thing that
tells them apart.

`TextDiagnostics` carries roughly thirty counters and this takes four. The
other twenty-seven are true and are not **actionable**: `spaces_derived` and
`lines_derived` are facts about every extraction ever run, and the window
already said so in `loses_breaks` where an operator can read it before
deciding. A status bar that lists everything measured is one nobody reads,
and rule 4's whole value is in being read.

### `fn named`

`suggested_path` and `suggested_form_path` take an `OpenDoc`, which
carries a session and cannot be built in a unit test. Their arithmetic
is one line each and it is that line that was wrong, so it is
reproduced here **from the same expression** rather than re-derived —
if either site changes shape, this stops describing it and the comment
below is the instruction to whoever notices.

Not a seam worth extracting: a shared helper would be a third place
the rule lives, and the rule is `format!("{stem}.{ext}")`.

### `fn a_dotted_document_name_keeps_its_revision`

`Path::set_extension` replaces everything after the **last** dot, so
under that call `plan.rev2` becomes `plan` and the suggested name is
`plan.dxf`.

⇒ The consequence is data loss, not untidiness: `plan.rev2.pdf` and
`plan.rev3.pdf` would both suggest `plan.dxf`, so exporting the second
**overwrites the first** — behind nothing but the operating system's
generic "a file with that name already exists". `.rev2` / `.rev3` is an
ordinary CAD naming shape, and the two files that collide are the two an
operator is most likely to want side by side.

The reasoning that makes `set_extension` look safe reads as true until
the helper meets a stem with a dot in it. **A claim in a comment is not
a test.** This is that comment, executed.

### `fn an_ordinary_name_and_a_bare_one_both_gain_the_extension`

The old comment's one true claim was that a document called `plan`
with no extension *"would gain one only through this call"*. It still
does: the stem of `plan` is `plan`, and the format string appends
unconditionally.
