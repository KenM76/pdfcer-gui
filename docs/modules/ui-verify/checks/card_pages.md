# `checks::card_pages` — the graphics card draws the page the processor draws

Opens `fixtures/a1-titleblock.pdf` (an A1 CAD title block: lines, text, fills)
twice, at fit, off the desktop: once with the preferences bare (switch off),
once with `draw_pages_on_graphics_card = true`.

It asserts three things, each with its own failure:

1. **Off asks nothing of the card.** The off launch traces no `render-card`
   line.
2. **On, the card drew.** The on launch traces `render-card drew=1`. A
   `drew=0 reason=…` line is quoted in the failure, so a refusal reads as the
   reason, not as "nothing happened".
3. **Same page.** The page's box, inset 4 pt because its corner sits on the
   canvas edge at fit, is compared pixel for pixel between the two
   screenshots. A pixel differs when any channel moves by more than 48; at
   most 0.5 % may differ. Anti-aliasing can move an edge pixel; a missing or
   moved line moves thousands.

Unequal box sizes between the two launches SKIP: the windows framed the page
differently and a pixel comparison would measure the framing.

Falsified by planting `= false` in the on launch (fails at arm 2) and a
negative ceiling (fails at arm 3). On the dense benchmark site plan the same
comparison gives 0 of 395,692 pixels differing.
