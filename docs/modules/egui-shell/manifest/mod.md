# `egui-shell/manifest/mod`

## Item notes

### `const TIDY_MAX`

Generous, because these lines are nested four levels deep and the whole
point is that a one-field item reads as one thing. Past this, three short
lines really are easier to read than one long one.

### `fn a_manifest_round_trips_through_ron`

The manifest's whole value proposition is that it is a file: an
operator edits it, an application ships one, a workspace is saved
as one, and `SHELL_FRAMEWORK.md` §6 lists "inspectable, diffable,
testable without a GUI, and serializable" as what the design buys.
Every one of those claims fails if the round trip is lossy.

Both forms are checked. The compact form is what a save writes;
the pretty form is what an operator opens, and a pretty printer
that emits something its own parser rejects would be discovered by
the operator rather than by CI.

### `fn an_unstated_field_stays_unstated_through_a_round_trip`

This is the property the whole layered design rests on: `None`
means "do not mention this" and must not come back as
`Some(empty)`. If a layer's omitted `groups` round-tripped into
`Some(vec![])`, saving and reloading an operator's customization
would silently empty every tab it mentioned.

### `mod rail`

A region carried as **manifest data**, for the reason [`Trailing`] is:
`SHELL_FRAMEWORK.md` makes the shell one serializable document, and a
region only the application knows about breaks that quietly — it cannot
be overlaid, filtered or validated with the rest.

### `trait CommandCatalog`

[`crate::commands::CommandRegistry`] implements it. The trait exists so
this module does not depend on that one: a manifest must be parseable,
mergeable and round-trippable by a tool that has no registry at all —
a schema linter, a diff viewer, `tools/ui-verify` inspecting a `.ron`
file without linking the application.

### `struct AnyCommand`

For tests, for tooling that has no registry, and for the first stage
of an application's own bring-up. Using it in production would disable
the check that makes an unknown id a disclosed skip, which is why it is
a named type at a call site rather than a default.

### `mod es`

`MODES_AND_PANELS.md`: *a mode is a named workspace layout*, and
Read/Review/Edit is a **configuration**, not a built-in. Nothing in
this crate knows those three names.

### `const SCHEMA`

Bump when a change would make an *older* build misread a newer
file — not for an added optional field, which an older build
already ignores safely.

### `fn with_trailing`

Takes [`Item`]s rather than ids, unlike [`Self::with_qat`], because the
whole reason this region exists is that its controls carry a
`visible_when` — see [`Trailing`]'s note on R9.

### `fn with_rail`

Takes [`RailGroup`]s rather than bare items because the rail's whole
scaling behaviour is per **group**: what folds, in what order, and what
never folds. A flat list would have nowhere to say that, and the answer
would have to be derived from position — which is precisely the
derivation `RIBBON_SCALING.md` §3.2 measured Word *not* doing.

### `fn from_ron`

# Errors

[`ManifestError::Parse`], carrying RON's own line and column. The
span is the useful part: this file is hand-edited, and "expected
`)` at 14:3" is the difference between a fixable typo and a file
the operator reverts wholesale.

Parsing does **not** validate. A layer is not expected to be a
complete manifest, so refusing to parse one that is incomplete
would make the layered design unrepresentable. Call
[`Self::validate`] on the merged result.

### `fn to_ron`

# Errors

[`ManifestError::Serialize`] if RON refuses the value, which for
this type's fields should not be reachable.

### `fn to_ron_pretty`

# Errors

As [`Self::to_ron`].

### `struct Mode`

`MODES_AND_PANELS.md` Part 1 describes what a mode is for, and one
rule from it binds anything rendering this type:

> **A mode changes what is *visible*. It never makes a visible control
> silently inert.**

That is what separates a mode from a master enable/disable toggle. A
toggle leaves the tools on screen and makes gestures quietly do nothing;
a mode *removes* the tools it disables, so there is no click that
mysteriously fails and no control whose appearance lies about what it
will do.

### `struct Tab`

`RIBBON_IA.md` §4 keeps an idiom worth preserving: every tab carries a
one-line **question** it exists to answer — *"What is on my screen, and
how is the page laid out?"* That is what [`Self::question`] is, and it
is not decoration: a tab whose question cannot be written in one line is
a tab carrying two unrelated jobs, and the fix is to split it.

### `struct Group`

The caption is required in a complete manifest. An uncaptioned group is
a row of controls whose relationship the operator has to infer, and the
caption is the only place that relationship is ever written down.

### `fn with_prefer_rows`

See [`Self::prefer_rows`] for what it does and does not promise. `rows`
below 2 is stored as given and ignored by the planner, which is the
honest handling: `1` means *"one row"*, which is already the default, and
silently rewriting it to `None` would make a manifest that round-trips
differently from the one that was written.

### `struct Qat`

`SHELL_FRAMEWORK.md` §5 states the one-command-one-tab rule so that this
is allowed: a command may appear on exactly one **tab**, and the QAT and
status bar may mirror it. A QAT that could not mirror would be a second
place to hunt for a command rather than a shortcut to a known one.

### `fn is_empty`

A present-but-empty `Trailing` is treated exactly as an absent one by
the renderer, so that an operator customization that removed the last
item reclaims the space instead of leaving a gap the width of nothing.

### `struct Keymap`

The chord is an opaque string here — `"Ctrl+E"`, `"F11"`. Parsing it
into modifiers and a key is the renderer's job, and doing it in this
type would mean a manifest could not be read by a tool that does not
link `egui`.

Ordered (`BTreeMap`) so a serialized manifest is byte-stable: an
operator's customization file that reordered itself on every save
would produce a diff on every run and make version control useless for
exactly the file most worth versioning.
