# `ui-verify/checks/attach_sound`

`a_sound_attaches_as_an_icon` — Markup ▸ Attach sound puts a `/Sound` icon on
the page carrying the chosen WAV, converted as the window asked.

# What it drives

Its own fixture, `fixtures/layer-assign.pdf` (ignores `--pdf`; an 800 × 600
page), and a WAV it writes to its output folder (16-bit stereo PCM, 44,100 Hz,
4,410 frames), named to the picker through `PDFCER_DIAG_SOUND_PATH`.

1. Review mode, `ribbon.tab.markup`, `ribbon.item.markup.sound`:
   `markup-tool tool=TextAnnot(Sound)`.
2. A click at (150, 450), clear of the fixture's box and annotation:
   `sound-picked source=env`, then `sound-annot-open … bytes=<WAV size>`.
3. `text-annot.sound-icon.Mic`, `text-annot.sound-resample`, then
   `text-annot.accept`: `sound-annot-read … rate=22050 channels=2 bits=16
   conversions=1 icon=Mic`, then `sound-annot-placed … id=<non-zero>`.

The rate is the oracle that the resample tick reached `SoundData::from_wav`:
left at the default the recording keeps 44,100 Hz and no conversion is
reported. Channels and bits are the oracle that the file itself was read. The
icon is the oracle that the window's choice survives to the action.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
