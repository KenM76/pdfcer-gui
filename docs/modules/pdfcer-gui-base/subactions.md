# `subactions` — the per-domain verbs an Action carries, as data

Each enum is pure data; its apply arm stays in `pdfcer_gui::app::actions`.

### `enum BookmarkAction`

See the module header for what makes them a family: every one of them names
its operand by `ObjId`, because an outline is a tree that every edit to it
renumbers.
**`PartialEq` and not `Eq`**, and the bound cannot be restored.
`pdfcer_core::outline::OutlineClip`, which [`BookmarkAction::Paste`]
carries, is `PartialEq` only — a bookmark's colour is three `f64`s and
floats have no total equality — and an enum holding one cannot be `Eq`.
Nothing needs it: `Eq` over `PartialEq` buys a `HashMap` key, and no action
is ever one.

### `enum FileAction`

# Why this enum exists, and it is R2 arriving on time for once

`Action` already carries four sub-enums — `Annot`, `Vector`, `Field` and
the write family — and adds a fifth here. The pattern is the same one
`AnnotAction`'s own note argues: a family of related verbs shares one
dispatch arm, one apply arm and one module, so the shared files carry a line
each and the family's reasoning lives with the family.

**It has one member, and that is the honest shape rather than premature
structure.** Spelled directly on `Action` instead, one feature's argument
would be written **three times in three files nobody owns** — `action.rs`,
`apply.rs` and `dispatch.rs`, each already at R2's 1,500-line ceiling. The
line count is the symptom; the duplication is the defect, and R2's rule is
*"when a file approaches the limit, that is the signal to find the seam"*.
This is the seam.

### `enum VectorAction`

Carried by [`super::action::Action::Vector`]. Every variant names a page and
paint-order indices into it; see the module header for why both travel
rather than being re-derived.

### `enum AttachmentRef`

# Why this is not an index, and not an `ObjId` either

`super::bookmarks`' header argues at length that an outline row must be
addressed by `ObjId` rather than by a position, because every edit to a tree
renumbers it. Both halves of that argument apply here and neither one
finishes the job:

- **A position is wrong for the same reason.** The `/EmbeddedFiles` name
  tree is sorted (§7.9.6: keys *"shall be sorted lexically in ascending
  order"*), and `attach_file` re-sorts the whole array on every insert. So
  *"the third row"* names a different file after any attach — and the queue
  drains **after** the frame, so a second action raised in the same frame is
  resolved against an already-moved list.
- **An `ObjId` is not available for the verb that needs one.**
  `EditSession::detach_file` takes *"its `/EmbeddedFiles` name-tree key"* —
  the raw bytes — and nothing else. The filespec's object id, which the
  listing does report, is not what that function accepts, and §7.9.6 makes
  the key a **byte string** with no declared encoding, so it cannot even be
  carried as a `String` without deciding an encoding the standard declines
  to.

⇒ Hence `Vec<u8>` for the document-level case: it is what the engine takes,
and `AttachmentKind::DocumentLevel::tree_key` exists to hand it over
*"exactly as the tree spells them"*.

The page-level case gets the annotation's `ObjId` instead, because that kind
has no key at all — it lives in one page's `/Annots` — and because the id is
what stays stable across a page reorder, which `page_index` does not.
