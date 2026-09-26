# `ui-verify/checks/chunk_band`

`a_rubber_band_inside_a_note_takes_its_lines` — **O215 ask 4, the band
half: a rubber-band drawn inside a text block selects the LINES it reaches,
and Shift and Ctrl refine that set the way they refine any other.**

# The request

`OPERATOR_REQUESTS.md` **O215** ask 4 names three gestures in one breath —
*shift-click, ctrl-click and rubber-band*. The first two are driven by
[`crate::checks::chunk_multi_move`]. This is the third, and it is the one
the operator reaches for when a note has eight labels and he wants five of
them: he does not click five times, he sweeps.

# The defect it was written against

`SelectionState::marquee` hard-set `SelectionLevel::Object`. So a band drawn
*inside* a note — with a line of that note already selected — ascended out
of the chunk rung and took the whole block. The operator's experience: he
sweeps three of six lines and gets all six, at the rung above the one he was
working at, and his descent is gone.

# The oracle — and the negative that names the defect exactly

```text
chunk band   marquee-parts page=0 object=0 mode=touched reached=3 kept=3 combine=replace
selection    selection-set page=0 object=0 part=0 level=part held=3 via=band
status bar   status-rung kind=text part=0 held=3 of=6
apply phase  move-text-lines page=0 n=3 epoch=1 disclosures=none
history      undo kind=MoveTextRun undo_depth=1
```

★★★ `marquee-mode` is the **object-rung** band's own line, and the defective
build writes it where this check requires `marquee-parts`. The two are
mutually exclusive — `take_chunks` returns before `select_with` is reached —
so the trace says which of the two bands ran, in one word, and a failure
here can quote the defect rather than describe it. A check that read only
*"three chunks are selected"* would be satisfied by silence on both.

| line | question it answers |
|---|---|
| `marquee-parts` present | did the band stay at the chunk rung at all? |
| `marquee-mode` present | did it ascend and take the whole block — the defect |
| `reached=` | did the geometry find the lines the band actually covers? |
| `kept=` / `combine=` | did the modifier arm combine rather than replace? |
| `status-rung held=` | does the surface that names the next verb's subject agree? |
| `move-text-lines` | is a band's set honoured by the same plural verb a Shift-click's is? |
| `undo_depth=1` | did N engine calls fold into ONE undo entry? |

# The geometry, which is arithmetic and not an estimate

`fixtures/paragraph.pdf` — six lines of one text object, baselines 16 pt
apart, glyphs about 8.4 pt tall, stated line by line in
[`crate::fixture::text_block_target`]:

```text
line 0  baseline 700.0   glyph band 700.0 .. 708.4   x 72.0 .. 338.8
line 1  baseline 684.0              684.0 .. 692.4     72.0 .. 241.4
line 2  baseline 668.0              668.0 .. 676.4     72.0 .. 276.8
line 3  baseline 652.0              652.0 .. 660.4     72.0 .. 298.1
line 4  baseline 636.0              636.0 .. 644.4     72.0 .. 216.7
line 5  baseline 620.0              620.0 .. 628.4     72.0 .. 110.0
```

[`WIDE_BAND`] runs from (400, 730) to (60, 666) — **right to left**, so it
is a crossing band and reaches whatever it touches. Its floor at y = 666
sits 5.6 pt above line 3's ceiling and comfortably inside line 2, so it
takes lines 0, 1 and 2 and no others.

[`NARROW_BAND`] runs from (400, 644) to (60, 634), 5.2 pt clear of line 3
below and 5.6 pt clear of line 5 above, so it reaches line 4 alone.

★★ Crossing rather than enclosing on purpose. An enclosing band would have
to discriminate on the lines' **right edges**, which are a claim about what
`ObjectModelProvider::text_line_bounds_canvas_of` counts as the end of a
line — trailing space, the text object's own width, the advance past the
final glyph. The vertical extents are the ones the fixture states, so the
bands are aimed along the axis whose numbers are known. The enclosing arm is
covered where it can be measured exactly, in `canvas::chunks`' own test of
the direction rule.

★★★ Both bands **begin at x = 400**, which is past the right edge of the
longest line and past the text object's box. That is a requirement, not a
margin: `canvas::pressing::body_under` claims a press inside the block's box
but on no line of it as a move of the selection — deliberately, so the white
between two lines of a note stays draggable. A band that began there would
be a drag of the set, and this check would be measuring the wrong gesture
while reading as if it measured this one.

# Fixture — pinned, and `--pdf` is ignored

⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed here, so its
absence is a broken checkout rather than an unavailable precondition.

# ⚠ What this check can see, and where its reach ends

It reads what four subsystems wrote down, not the pixels. That the band was
*drawn* while the pointer travelled, and that three outlines appeared when
it was released, have one oracle — a rendered screenshot — and they are
`text_chunks`' subject rather than this one's.

# ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS

Five plants. **Copy each file aside first** and restore from the byte copy;
never revert with git, because this project runs parallel tracks and a
chained revert discards another track's uncommitted work.

1. **The band never reaches the chunk rung.** Make `marquee::take_chunks`
   return `false` unconditionally. Step C goes red quoting `marquee-mode` —
   the object-rung band — which is the original defect exactly.
2. **The direction rule collapses.** Make `chunks::reaches` answer
   `band.intersects(chunk)` in both arms. Step C stays green, because it
   drives a crossing band; the unit test
   `only_a_crossing_band_takes_a_chunk_it_merely_clips` is what catches this
   one, and it is named here so a reader does not conclude this check covers
   it.
3. **The modifier arms do not combine.** Make `marquee::combined`'s `Add`
   arm return `reached.to_vec()`: step C stays green, step D goes red on
   `kept=1` where 4 was required — the Shift band throwing away the set it
   was meant to extend. Then put the `Subtract` arm back to `held.to_vec()`:
   steps C and D stay green, step E goes red on `kept=4` where 3 was
   required. The two arms are separately falsifiable because the check
   drives them over two different rectangles.
4. **The set is built and not honoured.** Delete the `None if lines.len() >
   1` arm of `canvas::moving::eligible`'s page-object text branch. Steps C
   to F stay green; step G goes red naming `move-text-line`, the singular
   verb over a band of three.
5. **Prove the plant is in the artifact.** `cargo build --release -p
   pdfcer-gui` AND `-p ui-verify`, then confirm the exe is newer than the
   source: a stale binary is the commonest cause of a falsification that
   "did not reproduce", and its tell is an **absent** trace line rather than
   a wrong one.
6. **Require the `[FAIL]` line**, not the exit code — a SKIP exits the way a
   PASS does.
