# `linkform` — the controls a link is made and changed with

One form serves the Add-link window and the Properties link section, so a link
is created and edited with the same controls and the same validation.

# State

`LinkForm` keeps every field for all three destination kinds, so switching the
choice loses nothing typed:

- `choice`: `Page`, `Web` or `Named`.
- `page_number` (1-based), `uri`, `name`.
- `border_on`, `width` (pt), `colour` (the engine `Color`), `dashed`.

The default is page 1 with no visible border: how most producers write a link,
and the choice that changes nothing on the printed sheet.

# Contract

- `from_reading(&LinkReading)` seeds the form from a link in the file. A link
  running an action this shell does not author (`Goes::Other`), or going
  nowhere, seeds as a page link; nothing is written until Apply.
- `target(page_count) -> Result<LinkTarget, &'static str>`:
  - Page: `DestView::Fit` on the clamped 0-based index. A link read with
    another view is re-written as Fit only when Apply destination is pressed.
  - Web: trimmed, and refused when empty, non-ASCII or containing a control
    character — the rules `set_link_target` applies, so the refusal shows
    before the button rather than after it.
  - Named: refused when empty. Whether the name exists is the engine's check;
    there is no public listing of names to pick from (request G179).
- `border() -> Option<LinkBorder>`: `None` when off. The width is clamped to
  0.25–12 pt, the colour written in its own space (grey, RGB or CMYK), and a
  dashed border gets a 3 pt dash.
- `goes_ui` and `border_ui` draw the controls and return whether anything
  changed. Each publishes the regions its caller names in `Regions`. The
  colour is `colourpick::show` with grey, RGB and CMYK and no opacity (`/C`
  carries none); its tabs publish under `{colour}.space.<rgb|cmyk|gray>`.

The words are in `text::linkedit`.
