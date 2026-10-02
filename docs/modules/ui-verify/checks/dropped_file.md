# `ui-verify/checks/dropped_file`

`a_dropped_picture_lands_where_it_was_dropped` — a picture dragged onto a
page lands at the drop point, at its natural size, with no dialog.

# Why a scripted drop

A drop originates in another program and arrives as an OLE transaction the
window manager delivers; no pointer movement can begin one. The scripted
pointer's `drop X Y [mods=…] PATH[|PATH…]` step puts the paths into egui's
`dropped_files` on the frame after a move to X Y, and while a script drives
the window `filedrag::poll` takes a drop's position from the scripted pointer
rather than the OS cursor. So the window stays off the desktop and the
operator's mouse is never touched.

| proved here | **not** proved here |
|---|---|
| classification, routing, the import, the placement at the drop point, the cascade, Alt opening the placement window, the GIF refusal | that Windows delivers a real drag to this window, and that `filedrag::aim` converts the OS cursor correctly (`drop_onto_thumbnails` drives that half) |

# The oracles

`image-dropped i= page= llx= lly= urx= ury=` per placed picture: a 48×24-pixel
PNG with no declared resolution is 48×24 pt, centred within 2 pt of the drop
point; a second picture in the same drop is centred one cascade step
(`app::dropped::CASCADE_PT`) down and right. Ctrl+Z must trace
`undo-applied`. A GIF must trace `drop-refused ext=gif` and add no
`image-dropped`. An Alt drop must add no `image-dropped` and declare the
`dialog:insert-image` region, which must not have been declared before it.
