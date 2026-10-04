# `ui-verify/checks/media_clip`

`a_clip_plays_from_a_region` — Markup ▸ Media clip puts a `/Screen` region on
the page carrying the chosen clip, under the type its name suggests, with the
trigger and temporary-file permission the window chose.

# What it drives

Its own fixture, `fixtures/layer-assign.pdf` (ignores `--pdf`; an 800 × 600
page), and a payload it writes to its output folder as `media-clip.payload.mp4`,
named to the picker through `PDFCER_DIAG_MEDIA_PATH`. The bytes are not a
video: the engine embeds them unread, so only the `.mp4` name matters.

1. Review mode, `ribbon.tab.markup`, `ribbon.item.markup.screen`:
   `markup-tool tool=TextAnnot(Screen)`.
2. A scripted drag from (150, 450) to (300, 350), clear of the fixture's box
   and annotation: `media-picked source=env`, then `screen-annot-open …
   suggested=video/mp4`.
3. `text-annot.screen-trigger.PageOpen`, `text-annot.screen-temp.TEMPALWAYS`,
   then `text-annot.accept`: `screen-annot-read … bytes=<payload size>
   type=video/mp4 trigger=PageOpen temp=TEMPALWAYS`, then `screen-annot-placed
   … id=<non-zero>`.

Both choices pressed differ from the engine's defaults (`Click`,
`TEMPACCESS`), so a choice dropped between the window and the action reads as
the default and fails. The byte count is the oracle that the file itself was
read; the type that the extension's suggestion reached the spec.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
