# `checks::card_navigation` — navigating on the card never holds the window longer than the processor

Drives one drawing twice off the desktop, with the preferences bare (switch
off) and then with `draw_pages_on_graphics_card = true`. The drawing is
`--pdf` when one is given (the dense benchmark site plan,
`D:\Dev\pdfTests\ncored-benchmark-cad-drawing.pdf`), else
`fixtures/a1-titleblock.pdf`, always as a copy.

**The script**, with the pointer at the canvas centre, done twice: four
Ctrl+wheel notches in, four wheel scrolls up, four notches in, four scrolls
down, eight notches out. Notches are 40 ms apart, which is a hand spinning the
wheel. Gestures are 350 ms apart, long enough for a render to start between
them.

**Per run it reports:**

- frames over 50 ms (`frame-long`): their count, the longest, and their sum,
  which is how long the window was held;
- renders started (`render-spawn`);
- the card's own draws (`render-card drew=1`): the count, the total
  `draw_ms` and the longest;
- the time from the last gesture until the last render started has landed
  (`render-async-done` or `render-inline`), once no render has started for
  1.5 s: when the final picture arrived.

**Verdict.** It fails when the card run's longest frame is longer than both
100 ms and twice the processor run's longest frame. It SKIPs, and does not
pass, when either run started no render or the card drew nothing.

`frame-long` times the UI thread's work between the top of `ui` and the end of
the frame. The card's draw and read-back happen inside that window. egui's
texture upload, which happens at paint, does not, and it is the same on both
paths.
