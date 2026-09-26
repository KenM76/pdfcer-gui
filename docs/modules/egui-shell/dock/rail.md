# `egui-shell/dock/rail`

## Item notes

### `fn visible_ids`

Filtering happens **before** the ladder runs, exactly as
[`crate::ribbon::trailing`] filters before it measures, and for the same
reason: a hidden item that was counted would make the rail fold a group to
make room for a control nobody can see.

This is where **mode gating** lands. An entry marked
`visible_when("mode.edit_content")` is, in a mode that does not set that
condition, not in this list, not measured and not folded — it is simply
absent, which is R9: *an unavailable capability renders nothing.*

### `fn read_mode_drops_the_points_tool_entirely`

R9: an unavailable capability renders nothing. Folding it would put it
behind the chevron, where the operator could reach a control the
mode's own dispatch refuses.

### `fn the_width_is_constant_at_every_rung_and_every_budget`

The R128 argument in a test: nothing about the content, the rung, the
number of folded entries or the length of a caption can move it. A
build that sized the rail from its widest word would fail here.

### `type RailHandler`

Called at most once per side per frame, with a [`egui::Ui`] whose
`max_rect` **and clip rectangle** are the strip. The clip is the
load-bearing half, for [`super::banner::BannerHandler`]'s reason: a caller
that draws a wider row than the strip gets it clipped rather than pushing
the panel body sideways, which is the R128 feedback loop this crate is
arranged to make unwritable.

### `type RailReach`

R7, in one line: the dock cannot answer this itself. It is handed
opaque [`PanelId`]s and a [`crate::manifest::Rail`] of opaque command ids,
and **the map between them is application knowledge** — an application is
free to name a panel's switch `file.fonts` or `markup.comments`, neither of
which a `view.panel_*` pattern would find. A shell that guessed at that
mapping by string shape would suppress a tab strip over a panel the rail
cannot reach, which is the unreachable-panel defect.

### `const WIDTH_PTS`

52 pt, which is the approved mockup's own value and wide enough for a
16 pt glyph with a short word under it. See the module header for why this
may not become a function of the content.

### `const PEEK_WIDTH_PTS`

Ten points, and the number is chosen against two floors rather than for
looks. [`crate::peek::Peek::MIN_TRIGGER_PTS`] is 8 and is the point below
which `Peek` refuses to hide the surface at all; Windows gives a window's
resize border 8 and VS Code's collapsed sidebar edge about the same. Ten
clears the first with margin and matches the second, and is wide enough to
draw a chevron in so the strip **says** it is there rather than being a
stripe the operator has to discover.

⚠ **It is reserved whether the rail is revealed or not**, and that is the
whole of the no-reflow guarantee: the panel body beside it is
`side_width − PEEK_WIDTH_PTS` in the hidden state and in the revealed state,
because the revealed strip is painted *over* the panel rather than beside
it. A build that reclaimed the sliver on reveal would resize the panel under
the pointer that revealed it, which is R128 exactly.

### `enum Rung`

Ordered, and the order is the ladder: [`plan`] walks these in sequence and
takes the first that fits. See the module header's table for what each one
gives up and why it is that one.

### `fn plan`

Falls through to [`Rung::Cramped`] when nothing fits, and that plan may be
taller than the budget — deliberately. The rail does not keep shrinking
past the floor; it **scrolls**, which is `RIBBON_SCALING.md` §3.3's third
rung, and the application wraps the rows in a `ScrollArea` to honour it.
The alternative — dropping rows until they fit — is the unreachable-control
defect, on the one surface whose whole promise is that every control is one
click away.

### `fn resolve_width`

Returns `0.0` when reserving the strip would leave the panel body below
[`super::plan::MIN_COLUMN_WIDTH`] — **absent rather than squeezed**, which
is [`super::banner::resolve_height`]'s rule and its reasoning: a strip that
publishes a rectangle beside a panel too narrow to read is a surface that
passes every gate and reaches nobody.

### `fn draw`

Returns `area` unchanged when there is no rail for this side or the width
resolved to zero, so the no-rail path costs one comparison and changes no
geometry — which is what keeps every existing dock layout test valid.

# The region is published against the SIDE's `Ui`, not the child's

[`report::Reporter::report`]'s own doc states the rule: reporting a region
against a clip derived from itself is *"the tautology `visible == 1.0`
dressed up as a measurement"*. The question asked of
`dock.<side>.toolrail` is *can the operator reach this strip*, and only the
side's clip can answer it.

# Why the region is not called `dock.<side>.rail`

That name is taken, by [`report::rail`], for a **different feature**: the
sliver a *collapsed* side leaves behind as the way back. The mockup's
legend draws the distinction explicitly — *"This one replaces the dock's
arrangement while the dock is open. VS Code's activity bar, not its
collapsed sidebar."* Two surfaces sharing one trace name is how a driven
check reads the wrong one —
`D:/dev/rag/egui/two_trace_lines_sharing_an_event_name_make_a_check_read_the_wrong_one.md`
carries the finding.
