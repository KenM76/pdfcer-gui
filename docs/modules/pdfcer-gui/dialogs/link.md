# `dialogs::link` — the Add-link window

# Flow

1. `edit.insert_link` (Edit mode only) arms the canvas placement tool with
   `PlaceKind::Link`.
2. The operator drags a box, or clicks. The box is settled before any question
   is asked — the order Acrobat's Link tool and Bluebeam's Hyperlink tool use.
3. `DialogsState::deliver_placement` opens `LinkDialog` on the page the box
   was drawn on.
4. Add link pushes `LinkAction::Add`, traced `add-link-requested`, and closes
   the window. Cancel, or closing the window, writes nothing.

# Contract

- `LinkDialog::open(page_index, page_count, rect)`: a zero-area box (a click)
  becomes `CLICK_SIZE_PT` (72 × 18 pt) with its lower-left corner at the click.
- The Add button is absent while `LinkForm::target` refuses, with the reason
  beside it.
- Regions: `dialog:add-link`, `add-link.add`, and the form's `add-link.page`,
  `.web`, `.named`, `.field`, `.border`, `.colour`.
