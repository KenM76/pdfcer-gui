# `ui-verify/checks/links`

Three checks on `fixtures/link-edit.pdf`: two 400 × 400 pages. Page 1 draws
"See page 2" and carries one link at `/Rect [100 180 300 220]` that runs a
JavaScript action and has a visible 1 pt blue border.

# `a_link_is_drawn_and_given_an_address`

Driven with `PDFCER_DIAG_INVOKE=mode.edit,edit.insert_link`.

1. Drag from (100, 300) to (260, 340) on page 1, clear of the existing link.
2. In the Add-link window, choose `add-link.web`, type
   `https://example.com/drawings` into `add-link.field`, tick
   `add-link.border`, and press `add-link.add`.
   Before the drag, require `place-armed kind=Link`; after it, require the
   `dialog:add-link` window to have drawn. Either missing is a FAIL, not the
   SKIP a failed `press` would file.
3. Require `add-link-applied` with `page=0`, `border=true` and
   `target=uri:https://example.com/drawings`. A page link (the form's
   default), a borderless link, or a link on another page each fail one term.

A screenshot is taken after Add.

# `a_selected_link_is_repointed_from_properties`

Driven with `PDFCER_DIAG_INVOKE=mode.edit,file.properties`.

1. Click (200, 200). Require `annot-select` with `subtype=Link`, then the
   `properties.link` section drawing after the click.
2. Choose `properties.link.web`, type the address, and press
   `properties.link.apply-target`. Require `set-link-target-applied` with the
   address and `replaced=JavaScript`.
3. Untick `properties.link.border` (ticked, because the fixture's border is
   visible) and press `properties.link.apply-border`. Require
   `set-link-border-applied` with `visible=false`.
4. Drag the top-right grip from (300, 220) to (340, 250). Require
   `resize-annotation-applied`. A link carries scale grips only.

# `a_review_click_follows_a_link_and_does_not_select_it`

Driven with `PDFCER_DIAG_INVOKE=mode.review,file.properties`. Click
(200, 200); require `link-click` and no `annot-select subtype=Link`.

# Driven off-screen

The checks use the scripted pointer, with the window off every monitor. They
run under `--no-input`.
