# `pdfcer-gui/dialogs/print/regions`

## Item notes

### `const REGION_PROPERTIES`

Published for `ui-verify`, which is the only oracle this project trusts
for a layout claim. See `tools/ui-verify/src/checks/print_paper.rs` for
what it asserts, and for why a driven check reads this rect without ever
clicking it.

### `const REGION_PAPER_ITEM_PREFIX`

# Why the ENTRIES are published and not only the combo

Because a check that can open a list but not choose from it can only
assert that a control exists — and "the control exists" is exactly the
claim that was true of the tray checkbox for four months while it did
nothing. The property worth asserting is that **choosing a sheet changes
the plan**, and that needs a click on a specific entry.

An egui combo popup is an `Area` laid out at paint time; its entries have
no position anything outside the process could compute. Publishing them is
the only route, and it costs nothing when `PDFCER_DIAG` is unset.

### `const REGION_PAPER_AUTO`

# Why it is NOT `print.paper.item.1`

It sits second in the list on screen, so the obvious thing would have been
to give it index 1 and push the driver's forms up by one. That would have
been wrong in a way no gate would catch.

`REGION_PAPER_ITEM_PREFIX`'s numbering is a **contract with the driver's
own form list**: index 0 is "say nothing", and index *n* is `forms[n - 1]`.
A driven check reads those numbers to click a specific enumerated sheet and
then asserts that the planned sheet moved. Inserting a policy entry into
that namespace would leave the existing check clicking a different thing
from the one it names, still green, still reporting a sentence about a
form — the class of defect this project has now written down four times.

So auto gets a name of its own, outside the numbered namespace. Better for
the check that needs it, too: `print.paper.auto` cannot silently become a
different entry when the driver's list changes length.

### `const REGION_SCALE_PREFIX`

# Why the scale radios are published at all

The dialog opens on **Fit**, which scales a page down to the printable area
and therefore clips nothing. Every claim about what gets cropped — the
hatch, the ink verdict, the whole Position group — is unreachable from that
state, so a driven check that cannot choose **Actual size** cannot assert
any of it. `tools/ui-verify/src/checks/print_clip_claim.rs` skipped on this
machine for exactly that reason and said so in its own header.

A word and not an index, unlike [`REGION_PAPER_ITEM_PREFIX`]: an index here
would be a contract with the order of a `for` loop rather than with anything
outside the process, and this list has gained a mode once already.

### `const REGION_POSITION_RESET`

Published individually rather than as a group union because the group's
rectangle cannot answer *which* button was pressed, and the four placements
differ only in the number they write — see `check-region-names.py`'s third
failure shape.

### `const REGION_TAB_PREFIX`

Every claim the dialog makes about *what gets cropped* now lives behind the
**Position** tab, so a driver that cannot press a tab cannot reach any of
it. A word rather than an index, for the reason
[`REGION_SCALE_PREFIX`] gives: an index is a contract with the order of a
`for` loop, and this list has just gained a fourth entry.
