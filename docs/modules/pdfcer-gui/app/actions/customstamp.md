# `app::actions::customstamp` — placing one of the operator's OWN stamps

`OPERATOR_REQUESTS.md` **O172**: *"add our own custom stamps and use them,
preferrably exactly the same way acrobat does."* This module is the second
half of that sentence. The first half — reading his stamps folder and
finding what is in it — is [`crate::stamps::library`]; the half that writes
a NEW collection for Acrobat to load is [`super::stamps`].

## Why this is a module and not an arm of `super::textannot`

It looks like one. Both place a `/Stamp` annotation, both are raised by the
same dialog, both take a rectangle the operator dragged, and both give the
new mark the same upright turn on a rotated page. The temptation to add a
branch to [`super::textannot::commit`] was real and was declined, for one
reason that is not about line counts:

> **A standard stamp is a NAME; a custom stamp is a DOCUMENT.**

`super::textannot` authors a `/Stamp` whose `/Name` is one of §12.5.6.12's
closed vocabulary and whose appearance the engine draws from that name. It
needs no second file, has nothing to fail at before the edit begins, and
cannot report anything about a source. This module opens a **second PDF**
off the operator's disk, fails in ways that have nothing to do with editing
(moved, renamed, corrupt), imports an object graph, and comes back with a
report of what did and did not travel. Sharing a function
would have meant a `commit` whose first forty lines were about neither of
the things it does.

What they DO share is deliberately shared rather than duplicated: the
upright-turn rule for `/Rotate` pages is one argument, written once in
[`super::textannot`]'s header, and applied here by the same two calls.

## The fork — what `app::actions::apply` decides before it gets here

One action, `Action::CommitTextAnnot`, reaches two modules. The arm that
chooses between them is three facts long and each is easy to get wrong:

1. **`custom: Some(_)` wins over `stamp`.** The gallery's two radio groups
   keep exactly one of the two live — a click on a standard stamp clears
   `custom`, a click on one of his clears nothing because `stamp` is not
   read on this route. The invariant is held by two call sites in
   `crate::dialogs::textannot::gallery` rather than by a type, which is why
   that function's header states it and its tests assert it.

2. **`kind` is NOT consulted.** The gallery is drawn only for
   `TextAnnotKind::Stamp`, so `custom` cannot be `Some` on a sticky note or
   a text box. A guard on `kind` would be a condition that can never be
   false, and a condition that cannot be false is a line every future
   reader has to prove harmless before they may change anything near it.

3. **The custom arm destructures with `..`.** `kind`, `text`, `stamp`,
   `stamp_size` and `icon` belong to the standard route and mean nothing
   here. Naming them only to leave them unused would mean one
   underscore-prefixed binding each, and each is a place for a field added
   later to be silently dropped on this route with no warning.

Why the argument is written **here** and not at the arm: `apply.rs` sits
against R2's 1,500-line ceiling, and the seam it keeps finding is *"the
reasoning goes where the mechanism is"*. The arm carries a two-line
pointer to this section.

## The order of operations, and why the load is outside the funnel

1. **Load the collection**, from the path the library recorded. Outside
   `vector_edit`, for the reason [`super::pages::insert_from_file`] gives:
   `place_page_artwork` borrows a `DocumentView` over it, so it must
   outlive the call — and a *load* failure reported from inside an *edit*
   closure would arrive wearing the edit's refusal sentence.
2. **Work out the upright turn** from the target page's `/Rotate`.
3. **One `vector_edit`**, one undo entry: place the artwork, then turn it.
4. **The disclosures**, off-canvas, on the status line.

## Rule 4 — what is disclosed, what is traced, and what is neither

| fact | where it goes | why |
|---|---|---|
| `distorted` | **status line** | the shape of his signature changed and nothing on the page says so |
| `dynamic` | **status line** | the date on the mark is not today's, and it looks like it is |
| `source_widgets_ignored` | status line, unless already covered by `dynamic` | part of the artwork's design did not arrive |
| `source_annotations_ignored` | **status line** | same |
| `objects_imported` | **trace only** | see below |
| `resources_renamed` | trace only | permanently `0` by construction |
| `transparency_group_carried` | trace only | `false` means the source had none, not that one was dropped |

**`objects_imported` is traced and not said, and that is a judgement
worth writing down.** The engine exposes it because *"an operator stamping
a 5.6 MB drawing is entitled to know which act grew the file"*, and that is
a fair reason — but the number is a count of PDF objects, which is not a
size, and turning it into a sentence would mean either quoting a figure
that means nothing outside the format (*"imported 47 objects"*) or
inventing a size estimate pdfcer has not measured. Neither is a disclosure;
both are noise on a status line that also has to carry the three sentences
above. It is on the diagnostic channel, where a driven check and a
bug-hunting session can both reach it, and the day the operator asks *"why
did my file get bigger"* the answer is one grep away.

**Nothing is drawn onto the canvas.** R8b rule 4: a placed stamp renders
exactly as it will render once saved and reopened — stretched if it was
stretched, with last year's date if that is what its author typed. No
badge, no tint, no dashed outline. The report is words, elsewhere.

## Item notes

### `fn disclosures`

Split out so it can be tested without a document, and so the *"say it
unless `dynamic` already covered it"* rule for widgets sits in one place
rather than inside a closure inside a funnel.

# Order

Shape first, then the promise, then what did not arrive. That is the order
of how much each one can cost him: a stretched signature is wrong on the
page, a stale date is wrong in fact, and a missing form field is a design
detail of somebody else's stamp.

### `fn said`

`PlacedArtwork` is `#[non_exhaustive]`, so it cannot be built with a
struct literal from outside its crate. It is `Copy` and every field is
public, so the fixture is made by placing artwork once — which is
exactly what a unit test must not do. The rules are therefore tested
through a shape this module owns instead, and the mapping from
`PlacedArtwork` to it is the two-line `disclosures` signature above,
which a reader can check by eye.

This is a real limitation and it is written down rather than worked
around: when the disclosure rules grow another condition, this comment
is the signal to ask the engine for a constructor rather than to bolt
another boolean onto the test helper.

### `fn the_stretch_direction_follows_the_ratio`

Both signs, deliberately. A helper that only ever divides one way
passes on a symmetric bug: *a suite which only tries one SIGN is not
testing the value*.
