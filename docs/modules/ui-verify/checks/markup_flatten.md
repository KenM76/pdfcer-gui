# `ui-verify/checks/markup_flatten`

`a_markup_can_be_made_part_of_the_page` — a markup's right-click "Make part of
the page" burns it into the page's drawing, and Ctrl+Z undoes it.

# What it drives

Pinned to `fixtures/four-pages.pdf`, whose page is blank where the shape goes.

1. Review mode, Markup tab, Polygon; draw a four-node polygon (three clicks and
   a double-click). `add-markup` must follow.
2. Count ink around corner 1 with nothing selected: the baseline. Too little
   ink is a harness error, not a pass or a fail.
3. Click the top edge: `canvas.markup-node.1` must be declared (a markup is
   selected). Right-click the same point; `menu.item.canvas.markup.markup.flatten`
   must be declared. Click it: `annotation-flattened … changed=true flattened=1`.
4. Ink around corner 1 must still be present: the shape still shows.
5. Click the edge again: no `canvas.markup-node.1` — it is page drawing now, and
   Review cannot select page drawing.
6. Ctrl+Z, click the edge: `canvas.markup-node.1` again.

# Why selection is witnessed by the node anchor

A selected markup shape publishes one anchor region per node, and a region's
declared-and-not-retired state witnesses absence as well as presence. Review
mode selects no page content, so after the burn the same click selects nothing.
