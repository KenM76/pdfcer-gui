# `pdfcer-gui-base/prefs/families`

**The keyed preference groups a `preferences.txt` line is offered to, in
order.** Each group (`offpage`, `printing`, `exporting`, `shortcuts`,
`snapshot`) owns its keys, its parser and its writer in its own module.

# Contract

- `parse_key` offers the line to each group in turn and returns the first
  answer that is not `NotMine`. `NotMine` from every group is an unknown key.
- No two groups claim one key spelling, so the order changes nothing but cost.
  A group whose keys could collide with another's takes a prefix, never a
  position in this list.
- A new group is one entry in `FAMILIES` and one `write_block` call in
  `prefs::file`'s writer.
