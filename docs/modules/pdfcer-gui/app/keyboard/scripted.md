# `pdfcer-gui/app/keyboard/scripted`

## Item notes

### `fn nth_chord`

Split out of [`scripted_press`] so the selection can be tested without
touching the process environment — `std::env::set_var` is process-global
and the test harness is multi-threaded, so a test that sets it is a test
that can change another test's answer.

### `const CHORD_GAP_FRAMES`

**Measured, not chosen.** With the chords delivered on consecutive
frames, a six-rung zoom ladder against `fixtures/a1-titleblock.pdf` climbed
only five rungs: the first `Ctrl` `+` arrived before the canvas had laid
out, so `FitMode::Page` was still standing and the next frame's fit solve
overwrote the zoom it had just set. The trace said `spelled=yes` for that
rung, because that is all it can honestly say — the seam runs before
[`super::collect`] and cannot know what became of the press.

⚠ **A swallowed rung is worse than a failed one.** It makes a ladder's end
state depend on how fast the machine got its first frame out, which is the
non-determinism a driven check exists to remove. A gap in front of the
first chord removes it structurally: by frame twenty the fit is long since
solved, on any machine.

⚠ **Frames, not milliseconds, and the two are not interchangeable here.**
[`scripted_press`] requests a repaint while chords remain, so the frames
come as fast as the application can produce them and twenty of them is not
a settle. What the gap buys is ordering — the state each rung acts on is
the state the rung before it left — and separability in the trace. What it
does **not** buy is a completed raster, and a check that needs one must
wait for the application's own `render-async-done`.

### `fn the_scripted_key_list_is_taken_in_order_and_skips_blanks`

Blanks are skipped rather than counted, so a trailing comma or a list
assembled by joining an empty slot does not spend a frame delivering
nothing — which would show up in a driven ladder as one rung silently
missing and a ceiling one step lower than the run intended.

⚠ The selection is tested here and not through the environment on
purpose: `std::env::set_var` is process-global, the test harness is
multi-threaded, and a test that sets it can change another test's
answer without either test being wrong.
