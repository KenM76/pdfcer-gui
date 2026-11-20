# `text::settings::card` — the Display page's graphics-card setting

Title, silence, radius, checkbox label and note for *Draw pages on the
graphics card*, in the *Drawing the page* group of Settings ▸ Display.

- **Silence:** pages are drawn on the processor. The switch is off by default
  because the card path does not yet take every page; off is the path every
  other check in the suite measures.
- **Radius:** only how the pages on screen are drawn. The file, printing and
  exports never use it.
- **Note:** says the picture is the same either way and where to read which
  way a page went (Render notes). It promises sameness because the engine's
  card path is specified byte for byte against the processor, and
  `the_graphics_card_draws_the_page_the_processor_draws` measures it.

The triple is catalogued in `app_tests.rs` with every other setting.
