# `an_open_paragraph_shows_each_words_look`

**Defect it guards.** O295: a paragraph opened for editing is drawn in the
shell-font editor box, or with every word in the first run's colour, so the
operator cannot see what the edit will look like.

**Fixture.** `fixtures/mixed-paragraph.pdf` (see `draft_mixed.md`): six lines
of 12 pt Helvetica, `notes` on line 2 in Helvetica-Bold, filled red. Opened
off-screen through the scripted pointer, Edit mode, Text tool armed.

**Steps and oracle.**

1. Screenshot before any click. Calibration: line 2's band (72–330 × 680–696
   pt) holds 20+ red pixels and line 1's band (72–400 × 698–712 pt) 40+ dark
   and no red. Otherwise the bands miss the text: an error, not a verdict.
2. Click (90, 703) on line 1; the paragraph opens.
3. The last `text-edit-shaped` carries `block=1 shaped=1`.
4. Screenshot: line 2 still holds 20+ red pixels, both lines 40+ dark, line 1
   no red. The in-font preview covers the page's own text, so red here was
   drawn by the preview.

A red pixel has red more than 60 above both green and blue; a dark pixel's
channels sum below 240.

**Falsified.**
- Every glyph inked in the first run's colour: FAIL at step 4, 0 red.
- `shapedblock::shape` always declining: FAIL at step 3, `shaped=0`. The
  shell-font box drawn in its place also reads 0 red.

**What it does not prove.** That the caret stops sit on the right glyphs:
the stop rules are unit-tested in `shapedblock`. A paragraph whose looks
differ in size, or a rotated page.
