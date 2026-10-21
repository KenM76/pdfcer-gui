# `colourpick` — a document colour in Grey, RGB or CMYK, with its opacity

## Purpose

This is the one colour control for anything that writes a colour into the
document. It replaces egui's stock picker, which offers RGB plus a blending
switch that means nothing to a PDF. The control lets the operator store a
colour in the device space the document will hold — `DeviceGray`,
`DeviceRGB` or `DeviceCMYK` — and set its opacity where the target can carry
one (O321).

## Contract

- **`Pick { ink: Ink, opacity: Option<f32> }`.** `Ink` keeps each component in
  `0.0..=1.0`, exactly as the file stores it. `opacity: None` means the target
  has no opacity, and no slider is drawn.
- **`Spaces`.** The spaces the *engine verb* behind the target accepts. Only
  those tabs are drawn. A verb that takes RGB only gets `Spaces::RGB`, and its
  picker shows no tabs.
- **`show(ui, id_salt, Shown, Spaces, region, mixed_hint) -> Option<Pick>`.**
  Returns `Some` only on the frame the popup closes after a change, so one
  gesture is one edit and one undo entry. Opening the popup to look and then
  closing it authors nothing. A fresh open re-seeds from the current value.
- **Regions:**
  - `{region}` — the swatch.
  - `{region}.picker` — the popup.
  - `{region}.space.<rgb|cmyk|gray>` — each tab.
  - `{region}.opacity` — the opacity row.

## Decisions

- **A value is shown in its own space and never converted on commit.**
  - A CMYK `/C` opens on the CMYK tab with its four numbers, and is written
    back as CMYK.
  - Narrowing CMYK to RGB for display would make the first edit rewrite the
    colour as RGB — a silent change of colour space.
- **Switching tabs converts by ISO 32000-2 §10.4.2's plain formulas.**
  - CMYK→RGB is `1 − min(1, c + k)`.
  - RGB→CMYK takes full undercolour removal.
  - These seed the new tab as a starting position. They are not a press
    simulation.
  - The swatch itself draws CMYK through `pdfcer_core::color::cmyk_to_srgb`,
    the conversion the renderer uses, so it matches the page.
- **Grey is entered as percent black**, so 100 % is solid black, matching how
  every editor and the old GUI's property bar present it. The file stores
  `1 − black`.
- **No Normal/Additive switch and no alpha checkerboard.** Those are egui's
  screen-compositing notions. Opacity is a separate, labelled slider and is
  only present when the target writes one (`/CA`, `ca`).
- **Which spaces each surface gets is decided by the engine verb, not by
  taste.**
  - Annotation `Color`, `vartext::TextColor`, `forms::MkColor` and
    `FormatRequest::set_fill` take all three spaces.
  - Add Text, path paint and ce dimensions take RGB only, until engine
    requests G219–G221 land.
  - Page-text opacity is absent until G222 lands.
