# `ui-verify/checks/model_light_map_card`

`the_light_map_is_measured_on_the_card` — the 3D viewer's *Colour by light
level* is measured on the graphics card while the card draws the picture,
and the card's map is the processor's.

# What it drives

`fixtures/glow-room.prc` (`fixtures/glow-room.PROVENANCE.py`), opened with
*View…* under `--no-input` with a `ScriptedPointer`, twice:

1. As shipped. Wait for a steady `model-view-rendered`, tick
   `model3d.light_map` in the options menu (`model_light_map`'s gesture) and
   wait for `model-light-map-ready`.
2. The same with `PDFCER_3D_BOUNCE_CPU=1`, which keeps every card build on
   the processor.

# Oracles

- The shipped run traced `model-light-map-card built=card` and its
  `model-light-map-ready` says `card=1`.
- The forced run traced no card build and says `card=0`.
- Both maps cover the same `triangles`: one lattice.
- Their `lo` and `hi` agree within 2% of the processor's `hi`. A pass that
  skips a run of gather directions, samples the wrong input, or finishes
  from one run's read-back instead of the sum leaves the lit room's range
  far outside that.

The oracle is the processor's map of the same room, not a figure written
into the check.

# Fails when

- The map is measured on the processor although the card draws the picture
  (`card=0`, no card build).
- The environment variable does not keep it off the card.
- The card's figures are not the processor's.

# Falsified

Making the card's pass skip every other row band leaves those points unlit
and the summed map's top at 2.976 against the processor's 1.934: the range
oracle fails.
