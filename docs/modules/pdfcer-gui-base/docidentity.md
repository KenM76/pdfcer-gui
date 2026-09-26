# `pdfcer-gui-base/docidentity`

## Item notes

### `enum Origin`

Two variants rather than an `Option<PathBuf>` on [`OpenDoc::path`], and the
choice is deliberate. Every document — created or opened — needs a
*identity* that is path-shaped: the forms cache keys on it, the Pages panel
captions from it, the trace names it, and a save suggestion would be built
from it. Making the path optional would push an `unwrap_or_default()` into
each of those, and `""` is the identity every unnamed document would then
share. What actually varies is one much narrower fact — *is there a file
there* — so that is what is stored, and [`OpenDoc::stored_under`] is the
only place it is asked.

### `struct SelectedField`

Both halves are needed and neither is redundant. The **name** is what
every field verb takes — `rename_field`, `delete_field` — because a field is
identified by name and not by object id. The **widget index** is what
`delete_widget` takes, and is the only way to say *"the box on page 3"* when
one field is drawn in two places.

The page is carried so the properties panel can say where the clicked box is
without re-walking the form to find out.
