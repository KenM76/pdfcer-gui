# `pdfcer-gui/app/actions/mod`

## Item notes

### `mod addtext`

Beside [`funnel`], along that file's seam: it routes, and this one
**decides**. A PDF has no paragraph, so a multi-line add needs a width to
wrap against — and once Enter makes a line
break at a *clicked* caret, which has no extent, somebody has to answer
where the second line ends. Its header carries the answer and, more
importantly, why the answer is read off the operator's own sheet rather than
invented.

### `mod apply`

Split out under **R2**; see its own header for the seam. Not `pub`: nothing
outside `app` applies an action, and the two entry points it adds are
inherent methods on [`crate::app::PdfcerApp`] rather than free functions, so
they are reachable exactly where they were before the split.

### `mod chrome`

Its own file because it is not an action: it is the one *operand* in this
vocabulary with a type of its own. Re-exported below, so no call site has
to name this module.

### `mod customstamp`

Beside [`textannot`] because both end in a `/Stamp` annotation, and NOT
inside it because a standard stamp is a *name* and a custom stamp is a
*document*: this one opens a second PDF off his disk, imports an object
graph, and comes back with four disclosures about what did and did not
travel. Its header carries the argument in full.

### `mod stamps`

Beside [`extract`] and [`merge`] for the property all three share: they
produce **new file bytes** and touch neither the session nor the undo log.
Its own header carries why it is not an arm in [`export`] — the difference
is not size, it is that this one authors a document whose *structure* means
something to a second application. What goes IN the file lives in
`crate::stamps`, which needs no picker to test.

### `mod crossdoc`

The only edit in the application that reads two documents at once, which is
why it is a file of its own rather than a sixth member of [`pages`]. Its
header carries the argument for the drop being a *copy* — it is about undo,
not about caution.

### `mod document`

Its header carries the guard table and why the two guards are two
predicates rather than one. The second exists because without it all four
destroy every edit made since the file was opened, silently, while
`file.close`'s tooltip promises otherwise.

### `mod history`

A sibling of [`apply`] rather than an arm in it: that module answers *what
does this verb do to the document*, and undo and redo describe **no edit at
all** —
they ask the session to replay one it has already recorded.

### `mod prefs`

Its header carries the four properties every member shares, and the
first of them is why the family exists at all: a preference needs **no
open document**, so its arm is matched above the `Status::Open` guard
that every other arm in [`apply`] lives under. `pub` rather than
private because the surfaces that raise these verbs are outside `app`
— `find::bar` and `panels::pages::previews` both name
[`prefs::PrefAction`] to build one.

### `mod attachments`

Its header carries the property that makes them a family rather than a
subject label: **every one of them opens a native file dialog**, so all
three are `Action`s for the reason [`write`]'s three are *as well as* for
the funnel's own — and **nothing any of them does is visible on the
canvas**, so every one of them owes a sentence to `app::status`, which is
the exact inverse of [`bookmarks`]' deliberately-silent rename.

`pub` rather than private, unlike [`apply`], because the surface that raises
these verbs is outside `app`: `panels::attachments` names
[`attachments::AttachmentAction`] and [`attachments::AttachmentRef`] to
build one.

### `mod bookmarks`

Its header carries the property that makes them a family rather than a
size-driven
cut — **every one of them addresses its operand by `ObjId`, never by a
position in the tree**, because an outline is renumbered by every edit to
it — and the §12.3.3 `/Count` table the engine sent this shell unprompted.

`pub` rather than private, unlike [`apply`], because the surface that raises
these verbs is outside `app`: `panels::bookmarks::add` and
`panels::bookmarks::edit` both name [`bookmarks::BookmarkAction`] to build
one.

### `mod merge`

Beside [`extract`] rather than in `pages`, because the two share the
property that decides where they live: both produce **new file bytes** and
touch neither the session nor the undo log. See its header.

### `mod dimensions`

A sibling of [`annots`] and [`pages`], drawn along the same seam they are:
*what class of thing does this verb act on?* Its own header carries the one
fact a reader needs first — that some of its verbs regenerate every member
of a group, on every page, and the rest touch exactly one annotation.

`pub` rather than private, unlike [`apply`], because the surfaces that raise
these verbs are outside `app`: `dialogs::scale`, `dialogs::dimension_groups`,
`panels::dimension` and `canvas::measure` all name
[`dimensions::DimensionAction`] to build one.

### `mod export`

Its header carries the property that makes it a subject rather than a
size-driven cut: **no verb in it changes the document at all**, so every
rule the mutation funnel enforces is irrelevant to them and every rule about
file handling applies instead.

### `mod pages`

A sibling of [`apply`] rather than part of it, on rule R2's own reasoning:
that file's subject is the cancel–mutate–bump–invalidate protocol, and this
one's is *a page index is a position, not an identity*. See its header for
the table of what each kind of page edit invalidates.

### `mod pagesize`

A sibling of [`pages`] rather than part of it, and the seam is a real one:
that file's subject is the resync a **structural** edit owes, and a media
box change adds, removes and renumbers nothing — every row of its table is
"unchanged" for this verb. Its header carries the measured answer to the
question every other page-size control in the world gets wrong: the paper
changes and the drawing does not move.

### `mod reviewstate`

`pub` because [`Action::RecordReviewState`] carries
[`reviewstate::RecordStatus`] as its payload — the pattern `annot`,
`forms` and `vector` already use, and the one that keeps `action.rs` under
R2's ceiling.

### `mod xobject`

One verb today, `unshare_form`. Its header
carries the property that makes this a family rather than a stray: **the
operand is a stream object paired with the page that invokes it**, a
`(usize, ObjId)` whose halves are not independent, and no other family in
this crate addresses anything of that shape.

It also carries the one fact a reader must not get wrong — the granularity
is **one page, not one invocation**, which is the engine's decision — and
the reason every one of the verb's refusals is worded rather than collapsed
into a shrug: after a refusal here the page looks exactly as
it does after a success, so silence reads as *"it worked"* and sends the
operator on to edit content they still share.

### `fn plant_edit_disclosure_for_test`

`#[cfg(test)]` so it cannot become a second way to record one — the real
path is [`record_edit_disclosure`], called from [`vector_edit`] with the
epoch the edit produced, and a second entry point is how two callers come
to disagree about what "the last edit" means.

It exists because the status bar draws this and must prove it does not grow
the bar while doing so (R128), and that measurement has to happen in
`crate::app::status`, which cannot reach a `thread_local` here. Exactly the
reason `crate::panels::forms::edit::plant_fill_disclosure_for_test` exists,
which is the shape this follows.

### `mod exporttext`

Its header carries the several different features "import text" could mean,
which matters because the operator asked for export and import in one
sentence and only one of those meanings is [`importtext`].
