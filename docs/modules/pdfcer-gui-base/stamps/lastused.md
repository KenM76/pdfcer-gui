# `stamps::lastused` — which stamp he reached for last, and why that is
a NAME rather than a stamp

`OPERATOR_REQUESTS.md` **O172**, and this module is the clause that was
left over when the placing half landed:

> *"in Acrobat a custom stamp is a menu entry beside the standard ones,
> grouped by the category it was authored under, chosen the same way,
> placed the same way, **and it remembers the last one used**."*

That sentence is not a nicety. Stamping a drawing set is a repetitive act —
the same `ISSUED FOR CONSTRUCTION` on thirty sheets — and a gallery that
opens on `Approved` every single time makes the operator re-find his own
stamp thirty times. Acrobat's Stamp menu re-opens on the last stamp placed;
under this project's standing *use the conventional interaction, never
invent one* rule, the convergence of the product class **is** the spec, so
this is not a design question and there was nothing to decide.

## The whole design is in the choice of what to store

The obvious implementation stores the [`CustomStamp`] that was placed. It
is wrong, and wrong in a way that produces a silent, operator-invisible
defect rather than an error:

A [`CustomStamp`] carries `file` and `page_index`. Between one dialog
opening and the next, the operator can go into Acrobat and edit his own
stamp collection — that is the *point* of it being his folder, and O169
built the authoring half precisely so he would. Adding a stamp to a
collection re-sorts the name tree (§7.9.6 requires lexicographic order),
which **renumbers pages**. A remembered `page_index` of 2 then names a
different stamp, with a straight face: the gallery would show the right
label selected and place the wrong artwork, and nothing in the program
would be in a position to notice.

⇒ **So the memory is a name, and it is re-resolved against a freshly
scanned library every time the dialog opens.** A name that no longer
resolves is not an error and not a disclosure — it is the same state as
never having placed a stamp, and it opens on the default. See
[`LastStamp::resolve`].

## Why `(category, label)` and not just `label`

Two collections may each carry a stamp called `Approved` — Adobe's own
shipped `Standard` and `StandardBusiness` both do, which is one of the
reasons [`crate::stamps::library`] excludes them. A label alone would pick
whichever came first in the scan, and the scan order is the file system's
opinion, not the operator's. The category is the collection's `/Info`
`/Title`, which is what the gallery draws as the section heading — so the
pair is exactly what the operator sees when he makes the choice, which is
the right thing to key a memory on.

## Scope: the session, deliberately not the disk

This lives on `dialogs::DialogsState` in the application-scoped half — it
survives closing a document, and it does **not** survive closing the
program. That is a deliberate stopping point rather than an oversight:

- Surviving the document is required by the use case. Stamping a drawing
  set means opening thirty files.
- Surviving the *program* would mean writing it into the settings store,
  and a preference has to be reachable — something the operator can see and
  clear. A hidden persisted selection that reaches back across a reboot to
  pre-select a stamp is the kind of state that produces "why did it put THAT
  on my drawing" reports, and the answer would be invisible to him.

⚠ If Acrobat is later measured to persist this across restarts, the
convention rule above says to follow it — but that would then arrive with a
visible control, not as a silent file. This paragraph exists so that is a
decision rather than a discovery.

## Item notes

### `fn standard_token`

**Deliberately an exhaustive match with no wildcard arm.** `StampName`
is not `#[non_exhaustive]`, so the day the engine adds a fifteenth face this
function stops compiling and names itself, which is the whole point. A
`_ => "other"` arm would keep building and quietly collapse two stamps into
one token, and the check that told them apart would go on passing.

### `enum LastStamp`

One of the two things a stamp gallery can be showing: a standard ISO 32000-2
Table 181 face, which pdfcer writes as a real named `/Stamp` annotation, or
one of his own, which is artwork imported out of a collection file.

Deliberately NOT `Copy` and deliberately holding owned `String`s: the
alternative is borrowing from a [`Library`] that is re-scanned on every
dialog opening, and the whole point of this type is to outlive one.

### `fn taken`

Takes the same pair the dialog holds — the standard selection, which is
always populated, and the custom selection, which shadows it when
`Some`. That mirrors `TextAnnotDialog`'s own rule (`custom.is_none()`
is what makes a standard radio read as selected), so the memory cannot
disagree with the window it was taken from.

### `fn resolve`

Returns the selection a freshly opened gallery should show:

- `(standard, None)` for [`Self::Standard`], or for a custom stamp that
  is no longer in the folder.
- `(standard, Some(stamp))` when the remembered pair is still there,
  with the *current* `file` and `page_index` — which is the entire
  reason this is a resolve and not a clone.

The `fallback` argument is the gallery's own default
([`crate::canvas::textannot::DEFAULT_STAMP`]); it is passed rather than
read here so this module does not acquire an opinion about what the
default should be.

A custom memory that fails to resolve deliberately produces **no
disclosure**. It is indistinguishable, to the operator, from the first
stamp of a session — and a sentence explaining that a stamp he deleted
is not being pre-selected would be noise attached to a window he opened
to do something else. The trace records it so the harness can tell the
two apart; the operator has nothing to be told.

### `fn token`

It is also not the gallery's operator copy: that is allowed to be
reworded on a Tuesday and is translatable in principle. Same rule and
same reason as `StampSize::trace_token` and
[`crate::canvas::stampfit::trace_token`].
