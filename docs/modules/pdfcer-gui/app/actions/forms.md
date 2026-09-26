# `app::actions::forms` — everything done to a form FIELD

A sibling of [`super::dimensions`], [`super::pages`], [`super::vector`] and
[`super::export`], and it owns both halves of its subject: the action enum
[`FieldAction`] and the apply logic every one of its variants reaches.

## Why the family is a family

Fill, select, place, author, rename, delete a field, delete one of its
widgets, register an unclaimed one, ask what deleting a grouping node would
take and then take it — every verb here shares a property nothing else in
`actions` has: **each of them addresses a control by its fully
qualified NAME, or by the widget's `ObjId`. None of them uses a paint-order
index.** That is not a coincidence of style; it follows from where the data
lives. `/AcroForm` is in the document catalog (§12.7.2), a field is reached
through it, and a widget is reached through the field that claims it — never
through the page that happens to draw it.

It is the same test the neighbouring seams pass: [`super::vector`] is the
verbs that address paint-order indices, [`super::pages`] the verbs that
address page positions. A size-driven cut would not produce this grouping
and would not survive the next variant; this one tells you where the next
verb goes without anyone having to decide.

## Doc comments concatenate silently

Two `///` blocks left contiguous above one variant render as that variant
carrying both explanations while the variant the first block was written for
carries none. Nothing warns: `cargo doc` is clean, clippy is clean, every
test passes, and a variant that has lost its documentation is
indistinguishable from one that never had any. Keep exactly one block per
variant and keep it adjacent to the variant it documents.

## Registering a form control the document lists but no field claims

The *disclosure and refusal wording* is the substantial part of that verb,
which is why it earned a module of its own before the family joined it.

## What an unclaimed widget is, and why the shell can produce one

A `/Widget` annotation in a page's `/Annots` that no entry of the document's
`/AcroForm` `/Fields` reaches. It **draws** — border, background, the whole
appearance stream — and nothing can fill it, because every filling verb
addresses a field by its fully qualified name and this box is in no field.

This project's recurring failure mode, a visible control that is silently
inert, arriving through a **document** rather than through a ribbon. The
operator clicks it, types, and nothing happens.

pdfcer makes them itself. `EditSession::insert_pages` copies everything
reachable from a page, and a page's `/Annots` reaches its widgets — but
`/AcroForm` is document-level and is not merged, so a source with 12 fields
inserted into a blank document produces 13 widgets and no form at all. The
engine measured exactly that (`examples/orphan_probe.rs`, pdfbox corpus) and
returns the count in `InsertOutcome::orphaned_widgets`.

## Two shapes, and only one of them can be put back

The engine's measurement is the reason this module has two refusal arms
rather than a success path and a shrug:

| shape | of 13 measured | carries | registering it |
|---|---|---|---|
| **merged field-widget** (§12.7.3.1) | 11 | its own `/FT`, `/T`, `/V`, `/DA` | **recovers the field exactly** |
| **bare kid** (a radio group's member) | 2 | nothing at all | **creates a new, empty field** |

The second row is `insert_pages` dropping `/Parent` from every dictionary it
copies. For a page that is correct — following it would drag the source's
whole page tree across. For a widget, `/Parent` **is** its link to its
identity, so those two arrived having lost the name `GroupOption`, the type
`/Btn`, the radio flags `0xC000` and the value `Option2`. Nothing in the
target document holds any of it.

An operator cannot see which shape a box is, and the difference decides
whether pressing Register restores something or invents something. That is
why [`crate::text::status::adopt_declined_no_name`] refuses to use the word
*restore*, and why it names re-inserting from the source as the only route
that gets the original back.

## Why this uses the funnel

`adopt_widget` writes `/AcroForm` and `/T`. It is a document edit with one
undo entry, so it goes through [`super::apply::vector_edit`] like every
other one — the render worker stopped, the mutation, the epoch bumped, the
page invalidated. Nothing here is special except the wording.

## Item notes

### `fn from`

The same reasoning [`super::vector`]'s `From` carries: which sub-enum a
verb is filed under is this module's business, and a panel button that
renames a field has none of it. `.into()` at the push, `From` here.

### `fn set_button_action`

# The disclosure this owes, and it is the whole reason `replaced` exists

`ButtonActionChange::replaced` names what was destroyed — **as a `String`,
including `"JavaScript"`, deliberately**. `pdfcer-core`'s own reasoning:
`Option<ButtonAction>` would have made a removed script inexpressible and
forced it to be reported as `None`, i.e. as *"there was nothing there"*.

A form editor overwriting another tool's work needs to know it did, and
this is the one moment it can be told. The status line carries it.

pdfcer will not write a script back. That asymmetry is deliberate and is
disclosed on the row rather than here: a `Foreign` action renders no Change
control at all, so the only way to reach this function with a script in the
way is through a route that has already said so.

### `fn correctable`

A free function taking `&EditError` so it is testable without an
`EditSession`, a document or a frame — the same shape
`crate::dialogs::insert_image`'s arithmetic was pushed into, and for the
same reason: `pdfcer_core::edit::EditError` is `#[non_exhaustive]`, so this
match needs a wildcard, and a wildcard inside a closure inside a funnel is
a place a future variant goes to be silently ignored.

Here it is one visible function with a test beside it. The wildcard means
*"anything else is a fault, not a chore"*, which is a real distinction and
the right default: a new refusal variant appearing in a future engine build
reaches the trace with its own words and does not silently acquire one of
these two sentences, which would be worse than saying nothing.

### `mod delete`

A submodule rather than more lines here, and the seam is subject rather
than size: everything in this file addresses a control an operator can see
and fill; a grouping node is a name with no type, no value, no widget and no
rectangle, whose entire difficulty is that its removal is invisible. That
module's header carries the two-press protocol, why the preview cannot run
in a panel, and where the armed preview is kept.

### `fn apply`

# Why two variants are NOT here

[`FieldAction::Begin`] and [`FieldAction::Commit`] stay in
[`super::apply`], and the reason is a borrow rather than a preference.
`Begin` opens a dialog and `Commit` remembers the operator's settings, so
both need `PdfcerApp`'s own fields — and `doc` in that function *is*
`&mut self.status`, so no signature exists that takes both. That arm's
comment carries the full argument.

The split is therefore a fact about the data, not an oversight, and it is
written down in both places so a later tidy has to argue with it.

### `fn adopt`

`name` is `None` when the operator left the box blank, which is the common
and correct answer: a merged field-widget carries its own `/T` and typing a
name would **override** it rather than supply something missing.

# Why the refusal is inspected here and the error is still returned

[`super::apply::vector_edit`] takes `Display` and does one thing with an
`Err`: it traces it and leaves the document alone. That is right, and it is
not enough for this verb, because two of `adopt_widget`'s five refusals are
**things the operator can fix in the next three seconds** — retype the name,
or supply one. A refusal an operator can act on that reaches only
`PDFCER_DIAG` is a control that does nothing when pressed.

So the closure records a decline on the way past and then hands the error
back unchanged. Both halves matter:

- **recording, not returning a message**, because `crate::app::status::decline`
  already owns the store, the retirement rule and the one line in the bar,
  and a second mechanism beside it would be the one that forgot to retire
  itself — that module's own header says so;
- **returning the error anyway**, so the trace still carries the engine's own
  `Display` prose. The decline is a sentence for an operator; the trace is
  the record for whoever is debugging, and they are not the same text and
  must not become each other. `check-ui-strings.sh`'s exclusion 3 is explicit
  that an error type's prose is not permission to route UI text through it.

The three refusals with no arm are unreachable from this surface rather than
unhandled — see [`decline::record_adopt_refusal`], which carries the table.

### `fn field_names`

Read fresh from the session rather than cached, and the reason is a
hazard rather than tidiness: the answer decides what the next field is
**named**, a name that collides makes the new widget a second view of an
existing field (see [`author`]'s header), and the set changes under any
undo, any redo, any page insert and every previous placement. A cache would
be correct until the first Ctrl+Z and silently wrong afterwards.

A document with no `/AcroForm` returns an empty list rather than declining —
which is the common case, since most drawings have no form at all, and the
first field placed on one has nothing to collide with.

### `fn edit_properties`

# Three disclosures Acrobat performs SILENTLY, and this is where they
are said out loud

The engine's brief is explicit that pdfcer neither refuses nor repairs these
three, and that the shell must surface them — *"shortening a limit is a
legitimate authoring act and the old value is the author's problem to
resolve"*, while truncating their data or re-pointing their selection would
be inventing document state:

| change | what actually happens |
|---|---|
| `/MaxLen` shortened below the current value | the field is over its own limit |
| a selected choice option removed | Acrobat re-points the selection **by numeric index**, so it can silently land on a *different* option |
| a check box's export value changed while checked | it renders **unchecked**, with no warning |

`FieldEditOutcome::value_no_longer_fits` is a ready-made sentence naming
exactly what no longer fits, and it is passed through **verbatim** rather
than re-worded — the same rule `textstyle` follows for a synthesis
disclosure, and for the same reason: the engine knows which of the three
happened and this crate would have to guess.

A fourth, `sort_claim_unmet`: `Sort` records what the *writer* did, and
Table 230 makes conforming readers display `/Opt` in the order it occurs.
Setting it over an unsorted list makes the file claim something untrue, and
pdfcer will not silently reorder a list whose order the standard makes
significant.

# `widgets_affected` is reported when it is more than one

A field's flags are one write and every widget follows — the engine's scope
table, taken from Acrobat's own scripting model. So setting *required* on a
field drawn in three places changes three things on screen, of which the
operator can see one. Said, and only when it is surprising: on the ordinary
one-widget field the number is noise.

# The selection is KEPT, unlike rename and delete

Those two clear it because the name they address stops resolving. A property
edit changes no name, so the pane must go on describing the same field —
and it must, because the operator's next act is very often a second flag on
the same field. `edit_epoch` bumps, which is what re-reads the pane's draft.

### `fn edit_widget`

# The disclosures, in the order the operator reads them

**1. A recorded-but-not-painted appearance first**, because it is the only
one about something they can *see* and will misread. Read from
`AppearanceOutcome`, which distinguishes *nothing needed redrawing* from
*something did and pdfcer could not*; the older `appearance_regenerated` /
`appearance_stale` pair could not, and this crate got it wrong for as long
as it read them. The engine's own sentence is carried verbatim and the
shell's framing is chosen by `resized`, because only a resize stretches
artwork — a colour edit that could not be repainted simply shows the old
colour.

**2. What changed.** `rect_after` is `Some` only when the rectangle
actually moved, so a border or colour edit gets the `touched` fragment the
panel supplied rather than *"The box was moved."*

**3. `siblings_untouched`**, and only when there are any. It is the mirror
of `widgets_affected` on the field verb, and the pair exists so an operator
working on a field drawn in three places knows which kind of control they
just used. On the ordinary one-widget field it is zero and says nothing.

## The selection is kept

Like [`edit_properties`] and unlike rename and delete: no name stops
resolving, and the operator's next act is very often a second nudge.

### `fn import_data`

The mirror of `actions::export::form_data`.

# Why this lives in `forms` and its twin lives in `export`

`actions::export`'s header draws that boundary and it is a real one: *"none
of them changes the document at all. No `vector_edit`, no undo entry, no
epoch bump, no cache invalidation. They read the open file and write a
different one."*

An import is the exact opposite. It reads a different file and **changes the
open document** — thirty fields at once, on a good day — so every rule the
mutation funnel enforces applies, and it goes through `vector_edit` like
every other edit. Putting it beside its twin would have put the one verb in
that module that breaks the module's stated property.

# One undo entry for the whole file, because the ENGINE makes it one

`import_form_data` is a single `EditSession` command however many fields it
sets. That is not this shell's doing and it is worth knowing, because the
same is emphatically *not* true of the panel's recompute — which writes one
command per field and says so.

It also asks the document-wide gate **once, up front**: a certification
that forbids filling forbids it for every entry, so discovering it on entry
seventeen would be both late and destructive. The engine's own comment says
so, and it is why a refusal here leaves the document untouched rather than
half-imported.

# The three failures are told apart, and they have nothing in common

| | what it means | what the operator does |
|---|---|---|
| unreadable | the path or the permissions | find the file |
| unparseable | the bytes are not form data pdfcer reads | pick a different file |
| refused | the **document** will not take an import — no form, certified, encrypted | nothing about the data file will help |

A single "import failed" would send an operator whose document is certified
off to re-export their data, twice.

### `fn rename`

## The engine takes a PARTIAL name and the selection holds a FULLY
QUALIFIED one, and conflating them corrupts a form

`rename_field(fqn, new_partial)` is asymmetric on purpose. A field's
fully-qualified name is its own `/T` joined to its ancestors' with dots —
`Address.Line1` is a field named `Line1` inside a parent named `Address`.
Passing a dotted string as the new *partial* name would author a `/T`
containing a dot, which no reader can resolve back: the field becomes
unaddressable by every fill verb, including pdfcer's own.

So the dialog offers the partial name and this passes it through untouched.
The engine is the one that rebuilds the qualified name, because only it
knows the parent chain.

## The selection is cleared, not updated

After a rename the old fully-qualified name reaches nothing. Recomputing the
new one here would mean deriving the parent chain a second time — the exact
duplication the paragraph above warns about — so the selection is dropped
and the operator's next click re-establishes it. One extra click, no chance
of a panel describing a field by a name that no longer exists.

# A doc comment attached to the wrong item reads as no doc comment at all

A contiguous run of `///` lines is one doc comment whatever it says, so
paragraphs written for this function but left above its neighbour render as
the neighbour documenting both while this one documents nothing. That
compiles, formats and passes `clippy -D warnings`, and from this function's
side it is indistinguishable from never having been written.
`tools/gates/check-orphan-docs.py` looks for the shape crate-wide; when it
fires, the fix is to move a doc, not to write one.

# The two early returns, and why neither is a decline

An empty `to`, or one equal to `from`, returns silently. Neither is a
command the operator gave: the panel gates its Rename button on
`!typed.is_empty()`, so an empty string can only arrive from a caller that
is not that panel, and `to == from` is the engine's own no-op case, reached
by pressing Rename without having changed anything. Declining either would
put a sentence on the bar for a gesture that asked for nothing.

This is **not** called per keystroke.
`FieldAction::Rename` is pushed only on the Rename button's click
or on Enter in the box, both gated on the same readiness flag — so every
call here is a deliberate commit, which is what makes recording a decline
safe rather than noisy. Measured at
`crate::panels::properties::formfield`, not inferred from these returns.

# Rule 4: pdfcer rewrites buttons the operator did not touch

`/ResetForm` and `/SubmitForm` name their targets as fully-qualified **name
strings**, so a rename that did nothing else would leave them pointing at
nothing. `rename_field` repairs them, correctly and invisibly, and no view
in this shell shows an action's target list — so without the conditional
sentence below the repair is unobservable. The condition is the point: a
receipt reciting *"0 buttons updated"* after every rename is a form, and by
the third one nobody reads the line that matters.

# What the error arm is actually for, measured rather than assumed

The panel greys Rename on `!typed.is_empty() && !typed.contains('.')`,
which pre-empts every refusal `rename_field` can derive from the string
alone — a dotted partial name, an empty name, a path too deep. **One**
refusal survives that gate, because predicting it needs the field tree
rather than the string: a **collision** with a name something already
bears. That is also the one an operator meets — rename `Rev1` to `Rev2` on
a form that has a `Rev2` — and without an arm for it the operator would get
`decline::floor`'s generic *"That change was refused"*.

So in [`correctable`] the dotted arm is defensive on this route and the
collision arm is the one that carries it. Both are asserted from outside in
`tests::renaming_onto_a_name_that_is_taken_says_so` and
`tests::a_dotted_rename_is_worded_even_though_the_panel_greys_it`.

### `fn disclosures`

A free function so the rule-4 obligation is testable without a session, a
document or a frame — and so that a new flag on `FieldAuthorDisclosures`
appearing in a future engine build has one obvious place to be handled and
one test that notices it was not.

Order is deliberate: **`merged` first**, because it is the only one that
changes what the operator believes they just made. The rest are advisory.

### `fn move_widget`

The disclosure is CONDITIONAL and reports what the operator cannot see:
`WidgetMove` names whether the field's other widgets stayed put, and on a
field drawn on three pages that is the whole question. Moving one box of a
three-box field is correct — they are separate placements of one value — and
it is also exactly the thing an operator would assume had gone wrong when
the other two did not follow.

Nothing is disclosed for the ordinary one-widget field, for
`text::embed`'s reason applied here: a sentence that fires on every drag is
one an operator learns to skip, and the day it says something is the day
they skip it too.
