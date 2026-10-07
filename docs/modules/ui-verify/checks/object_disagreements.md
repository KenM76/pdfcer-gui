# `objects_panel_says_where_the_list_and_page_disagree`

**Defect it guards.** The Objects panel lists what `decompose_page` found,
and three things make that list disagree with the page the operator sees:
a path painted under `/ca 0` or `/CA 0` is listed and selectable but shows
nothing; an `sh` gradient is painted but produces no object; a path coloured
in `/Separation`, `/DeviceN`, `/ICCBased`, `/Indexed`, `/Lab` or a pattern is
listed with a best-effort device colour. The engine counts each in
`DecomposeDiagnostics` (`paths_invisible_by_alpha`, `shadings_unmodelled`,
`paths_with_undecoded_colour`); unread, the operator clicks empty paper and
selects something, or cannot find a gradient he can see, with nothing to say
why.

**Fixture.** `fixtures/object-disagreements.pdf`, built by
`object-disagreements.PROVENANCE.py`: one 340 × 100 page holding a black
square (the control), a square under `/ca 0 /CA 0`, a square in a
`/Separation` spot colour, and an axial shading clipped to a square.

**Steps.** Launch on the fixture with `view.panel_objects` invoked on
opening. The `objects-disagree` line must say `page=0 invisible=1 shadings=1
undecoded=1`, and the `panel.objects.disagree` region must be published as
visible.

**Falsified** by zeroing the invisible count in
`panels::objects::disagree::lines`: the check fails reading `invisible=0`.

**What it does not prove.** The wording of each line; that is read in
`text::panels::objects`. The black control square proves only that an
ordinary path adds to no count.
