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
