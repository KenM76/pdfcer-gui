# `ui-verify/checks/link_named`

Two checks on `fixtures/link-named.pdf`: three 400 × 400 pages. It defines
two named destinations, one in each §12.3.2.3 namespace: `appendix` in the
catalog's `/Dests` dictionary, and `chapter-two` in the `/Names /Dests` name
tree. Page 1 carries one link at `/Rect [100 100 200 140]` going to
`appendix`.

Both checks require the Named choice to list exactly two rows,
`<names>.0` and `<names>.1`, and no `<names>.2`. A list that reads only one
namespace has no second row. A list that repeats a name has a third. Then
both checks pick row 1. The engine returns names in byte order, so row 1 is
`chapter-two`. A list in namespace order would put `appendix` second, and the
pick would write the wrong name.

# `a_named_destination_is_picked_for_a_new_link`

Driven with `PDFCER_DIAG_INVOKE=mode.edit,edit.insert_link`.

1. Require `place-armed kind=Link`, then drag from (100, 250) to (260, 300)
   on page 1, clear of the link. Require `dialog:add-link` to draw.
2. Press `add-link.named` and require the two-row list `add-link.names`.
3. Press `add-link.names.1`. Require `add-link.add` to draw: Add is absent
   while the name is empty, so a pick that sets nothing fails here and is
   not filed as the SKIP a failed `press` would give.
4. Press Add. Require `add-link-applied` carrying
   `target=named:chapter-two`.

Screenshots: the Add link window after the pick (`link-named-add-list.png`, when
its viewport answers one), and the main window after Add.

# `a_named_destination_is_picked_for_a_selected_link`

Driven with `PDFCER_DIAG_INVOKE=mode.edit,file.properties`.

1. Click (150, 120). Require `annot-select` with `subtype=Link`.
2. The link goes to a name, so its form opens on the Named choice. Require
   the two-row list `properties.link.names`.
3. Press `properties.link.names.1`, then `properties.link.apply-target`.
   Require `set-link-target-applied` carrying `target=named:chapter-two`.
   The link went to `appendix`, so an unchanged target fails.
