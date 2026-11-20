# `text::status::card` — which way the page on screen was drawn

The words the status bar's **Render notes** lead with while Settings ▸ Display
▸ *Draw pages on the graphics card* is on. With the switch off nothing is
added, so a user who never turns it on never reads about it.

- `card_drew(rest)` — the card drew the page; `rest` is the ordinary notes.
- `card_processor(why, rest)` — the switch is on and the processor drew it;
  `why` is one of the reason strings below, so the user learns *what* on the
  page kept it off the card, not that "something failed".
- One reason per `pdfcer_render::GpuRefusal` variant the shell names
  (picture, smooth shading, soft mask or blend mode, curves, size), one for a
  page the engine cannot record, one for a window with no GL context, and
  `card_failed(driver)` for a drawer that failed, quoting the driver's words.

This is disclosure off the canvas (R8b): the page itself is drawn the same
either way, so nothing on it says which way it was drawn.
