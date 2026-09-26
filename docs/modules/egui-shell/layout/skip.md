# `egui-shell/layout/skip`

## Item notes

### `fn a_missing_file_is_not_noteworthy_but_a_dropped_panel_is`

An application that surfaced every skip would tell every operator,
on the first launch of a fresh profile, that their layout could
not be restored — from a profile that has never had one. That is
how a disclosure surface trains people to ignore it.

### `enum LayoutSite`

Positional, because the model is positional — see
[`crate::dock::model`]'s note on why there are no generated handles to
name. A site is enough for an application to say *"the second stack of
the left dock's first column"* or to offer to open the file at
roughly the right place.

### `enum LayoutSkipReason`

Every variant carries the offending value, so an application can act
on it — offer to remove the entry, name the capability that is not in
this build, or simply log it with enough detail to be actionable.

`PartialEq` but **not** `Eq`, because [`Self::InvalidSize`] carries the
`f32` the file actually said. Carrying it is worth losing `Eq` for: an
application that wants to tell the operator *which* number was
rejected cannot get it from anywhere else, and a reason that said only
"a size was wrong" would be one an operator can do nothing with.

### `struct LoadReport`

Returned **by value** so the caller must deal with it, which is the
same shape and the same argument as [`crate::manifest::MergeReport`]:
returning the rejects alongside the result makes them a value the
caller must handle instead of a side effect it may forget.

### `fn is_noteworthy`

[`LayoutSkipReason::FileMissing`] is the first run of a fresh
profile and is not news. Everything else is a difference between
what was saved and what was restored, which is exactly the class
of fact this project's disclosure convention exists to surface.
