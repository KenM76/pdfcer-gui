# `ui-verify/checks/scripted_keys`

**The keystroke seam is a route to the viewer verbs, and this is what
proves it** — a zoom ladder climbed by a window that takes no OS input at
all.

# What this file is for

`app::keyboard::scripted` reads `PDFCER_DIAG_KEYS`, a comma-separated list
of chords, and pushes a real `egui::Event::Key` into the frame ahead of the
keyboard collector. It exists because the four viewer verbs — zoom in, zoom
out, next page, previous page — have **no registered command id**, so
`PDFCER_DIAG_INVOKE` cannot ring them, and a window placed off the desktop
has no other way to be driven. Its own argument is in that module;
`DESIGNS.md` carries why a seam is legitimate there where registering a
command would not be.

This check is the seam's only assertion. Until it existed the seam was
evidenced by a driven run recorded in a commit message, which is a
measurement that stops being re-taken the moment it is written down.

# Why it reads the application's zoom and never counts rungs

The seam traces one line per chord:

```text
diag-keys index=0 chord=Ctrl++ spelled=yes
```

`spelled=yes` means the key was **pushed**, and nothing more. The seam runs
before the collector and cannot know what became of the press — and it
emitted exactly that line for a rung whose effect was destroyed later in
the same frame. Six chords delivered on consecutive frames produced six
`spelled=yes` lines and **five** rungs of zoom: the first chord landed
before the canvas had been laid out, `FitMode::Page` was still standing,
and that frame's fit solve overwrote the zoom the chord had just set.

⇒ *"I asked for N and got N `spelled=yes` lines"* is an assertion both
outcomes satisfy. So is an end-state assertion: the broken run and the
sound one **both finish at 125 %**, differing only in how many steps they
took to get there.

What this check reads instead is the application's own `status … zoom=`,
**windowed between one rung's `diag-keys` line and the next one's**, so a
rung that produced nothing is a failure rather than a silence, and a rung
whose effect was undone shows up as a zoom that did not move.

# The unspellable rung, and what it is for

One entry in the list is a chord that cannot be spelled. It is there so
that `spelled=` is **shown to vary**: a seam that wrote `spelled=yes`
unconditionally would satisfy every other assertion here, and the field
would be decoration. The same rung doubles as the negative control for the
zoom reading — it must move the zoom by nothing at all, which no other rung
in the list is allowed to do.

# What this does NOT reach, and it is the row next door

⚠ **This says nothing about `OPERATOR_REQUESTS.md` O220.** The operator's
report is that `Ctrl`+wheel stops zooming out once the raster refusal has
appeared. `Ctrl`+wheel is a continuous `zoom_delta` arriving through the
pointer, which is a different route from the discrete keyboard action this
ladder climbs; a green run here is not evidence about that route. It also
stays entirely below the raster ceiling by choice of fixture, so it is not
evidence about O218 or O219 either.

It also does not assert that a raster **completed** — only that one was
ordered. A check needing a finished raster waits for the application's own
`render-async-done`.

## Item notes

### `const FIXTURE`

An A1 CAD sheet fits at about 30 %, so six rungs of zoom-in land at
125 % — a discrete climb that stays entirely under the whole-page raster
ceiling. The suite's usual clean control opens near 100 % and the same
ladder would run into the refusal, which would make a rung's silence
ambiguous between *"the seam did not deliver"* and *"the rasterizer
declined"*, and those two send a reader to opposite ends of the program.

### `const OFFSCREEN`

Nothing is ever aimed at this window and no OS input is sent to it — which
is the whole point of the seam — so this check can run while the operator
is working.

### `const RUNG_DEADLINE`

The seam paces its chords twenty frames apart and requests a repaint on
every frame while chords remain, so on any machine this runs on the whole
list is delivered in about a second. The generous deadline is for a loaded
machine; exceeding it is reported as a finding rather than a hang.

### `fn check_delivery`

A seam that delivered a chord twice, skipped one, or renumbered them would
pass every zoom assertion below by accident — the windows would still be in
ascending order and the zoom would still climb. This is what makes the
rung-to-chord mapping real rather than assumed.

### `fn wait_for_rung`

Not [`Session::settle`]. That polls the frame counter, and this
application stops drawing the moment the seam stops asking it to — so a
settle asked for more frames than the run has left would spin out its whole
cap on every green run. The thing being waited for is a trace line, so the
trace line is what is waited for.
