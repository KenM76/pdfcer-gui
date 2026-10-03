# `replace_all_is_one_undo`

The Find bar's Replace and Replace all (WORDLIKE step 8). On
`fixtures/paragraph.pdf`, launched with `mode.edit,edit.text` off the desktop
through the scripted pointer.

1. Ctrl+F, type `the`, Enter: the search reports `hits=4` (case-insensitive;
   each hit lies inside one show operator).
2. Click the Replace toggle, click the replacement field, type `our`, click
   Replace: `text-replace-applied which=current found=1 replaced=1`, and the
   re-search reports `hits=3`.
3. Click Replace all: `which=all found=3 replaced=3`, and the re-search
   reports `hits=0`.
4. Escape (so Ctrl+Z is the document's Undo, not the field's), Ctrl+Z: one new
   `undo-applied` line.
5. Click the search field, Enter: `hits=3`. One Undo restored every hit the
   Replace all rewrote.

## What makes it fail

Deleting the `coalesce_last` call in `app::actions::replace::run`: the three
rewrites become three undo entries, one Ctrl+Z restores one hit, and step 5
reports `hits=1`.
