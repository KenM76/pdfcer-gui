# `app::pageonly`

The line over the top of a page-only window: *Press F11 or Esc to leave full
screen*. Full screen draws the page and nothing else, so the keys that end it
would otherwise be undiscoverable; this is what every full-screen viewer does.

It shows for three seconds from the moment page-only starts and takes no clicks,
so it can never swallow a gesture on the page. The chord comes from the live keymap
through `window::fullscreen_chord`; an unbound chord drops out of the sentence and
Esc remains.

It floats because there is nothing left for it to sit in; `RIBBON_IA.md` P5 names
it as an exception. It is the cursor's kind of thing: about the window, gone in
three seconds, and no part of a document's picture.

Trace: `fullscreen-hint line=`. Region: `fullscreen-hint`. Driven by
`read_mode_hides_the_chrome`.
