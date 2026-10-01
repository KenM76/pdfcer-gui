# Recipe: carry out markup instructions on a drawing

The user has a drawing with markup on it (clouds, arrows, notes such as "move
this", "delete", "add a door here") and wants the changes made to the drawing itself.

## With the CLI (file on disk)

1. **Work on a copy.** The user's tab must be saved and closed first
   (file-safety.md). Write every step to a new file in a scratch folder.
2. **Read the instructions.** `pdfcer.exe list-annotations <in> --pages N`
   prints each annotation's `subtype`, `rect`, `note`, `author` and `vertices`.
   The rect shows where the instruction points. The note says what to do.
3. **Know the coordinate frame.** `pdfcer.exe dump-structure <in> | grep MediaBox`.
   If the box does not start at `0 0`, every coordinate is relative to its origin.
4. **Look before acting.** `render-page <in> --page N --scale 2 --region x0,y0,x1,y1 -o before.png`
   around each instruction's rect, plus a margin. Read the PNG.
5. **Find the objects.** `object-list <in> --page N` prints one row per object:
   `index= kind= bbox= ...`. Keep the rows whose bbox meets the instruction's
   rect; a drawing can hold tens of thousands. `--hit X,Y` names the object a
   click there would pick.
6. **Change geometry.**
   - Move or resize: `object-transform --objects i,j --translate DX,DY` (also
     `--scale`, `--rotate`, `--pivot`). Check with `--preview` first.
   - Reuse an existing shape (a door, a symbol): `object-copy --objects i,j --clip c.clip`,
     then `object-paste --clip c.clip --translate DX,DY`.
   - Reshape: `node-move --object i --node k --x X --y Y`.
   - New geometry: `annotate --type line|polyline|polygon|square|circle|ink --as-content`.
     This draws straight into the page content, not as a comment.
   - Remove: `object-delete --object i`. Delete highest index first, because
     later indices shift down.
7. **Match the drawing.** Give new geometry the `line_width=` and layer (`oc=`)
   that `object-list` reports for its neighbours (`annotate`/`object-paste` take
   `--width`, `--color`, `--layer`). Read colours from the rendered PNG. Follow
   the drawing's legend, not the markup's colour.
8. **Prove it.** Render the same region at the same scale (`after.png`) and
   compare it with `before.png`. Then render the whole page and check that
   nothing outside the instruction moved.
9. **Finish.** Leave the markup annotations in place unless the user said
   otherwise. They record what was asked. Keep the saves incremental. Give only
   the final step `--mode full`, and only when `list-signatures` reports none.
   Deliver a new file. Tell the user its path and what was changed.

## With the live link (document open in the user's window)

`state` → `page N` → `render` → `objects` (`i  kind  bbox` on the current page)
→ `select i,j` → `run <id>` → `render`. Run `run mode.edit` first. Every step
can be undone in the user's window. Steps the live link cannot express
(numeric transforms, paste with offset) fall back to the CLI route above.

## Notes

- Markup is annotations. The drawing is page content. `annotate` without
  `--as-content` adds another comment, not drawing geometry.
- A pdf dimension on a CAD drawing is plain vectors and text. Edit it like any
  other object; the `dimension-*` commands only see ce dimensions.
- If a note is ambiguous, ask the user. Do not guess at a drawing.
