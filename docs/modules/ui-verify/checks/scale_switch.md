# `ui-verify/checks/scale_switch`

`the_line_weight_switch_reaches_the_resize` — **tick the switch, drag a
grip, and the border thickens with the shape.**

# What this is for

`OPERATOR_REQUESTS.md` **O51**:

> *"if that was the resize question about scaling line weight, etc with
> resize it got the answer wrong. default should be what it said, but there
> should be an option that they do scale with resize. Inkscape has options
> for this and I want the same."*

## What this check exists to catch, which is not "the switch is missing"

The switch is a checkbox writing a `bool` into `egui::Memory`. Nothing about
that can plausibly fail. **The chain in front of the engine is what fails**,
and it has five links, three of which are pure wiring:

| # | link | a unit test can see it? |
|---|---|---|
| 1 | the Select tool's option row is drawn at all | partly — `armed::options` returns early for every other tool |
| 2 | the checkbox writes the store | yes — `canvas::scaling`'s round trip |
| 3 | the value reaches `resizing::Frame` on the commit frame | **no** |
| 4 | it travels on the action rather than being re-read at apply time | **no** |
| 5 | it reaches `ResizeOptions` and the engine acts on it | yes — `to_options` |

Link 3 is the one that was wrong for the life of the feature and in the
opposite direction: `annots::resize` **derived** `scale_stroke_width` from
whether the drag was proportional, so an operator's answer could not reach
the engine at all. A build that regressed to that would pass every unit test
in the chain, because each end of it is correct in isolation.

## The oracle is `stroke=` on the applied line, not a pixel

`resize-annotation-applied … stroke=true|false` reports whether the engine
wrote a new `/BS /W`. A screenshot cannot separate *"the border thickened
because `/BS /W` changed"* from *"the border thickened because §12.5.5's
matrix scaled the drawn stroke"* — those are different outcomes with the
same picture, and only the first is the switch doing its job.

⇒ **The picture is the same in the case this check is about.** That is why
it reads the trace, and it is a fact about the format rather than a
limitation of the harness — the identical argument `markup_move` makes for
its `keys=` field.

## The sequence

| # | step | oracle |
|---|---|---|
| A | Review mode, rectangle tool, drag a shape | `markup-commit` |
| B | click View ▸ Select to put the pen down, then click the shape | `ribbon-command-invoked id=view.tool_select`, `annot-select` |
| C | open the Tool panel, click the *Scale line weight* switch | `resize-modifiers stroke=true` |
| D | drag a corner grip **proportionally** | `resize-annotation-applied … stroke=true` |


This check passed its real assertion — `resize-annotation-applied …
stroke=true`, the switch reaching the engine — and also failed three runs
out of six **before getting that far**, at step B. The subject was never
implicated. What was unreliable was putting the markup pen down:

| attempt | result |
|---|---|
| `V` (the `view.tool_select` chord) | never arrived — no invocation traced at all |
| one Escape | arrived sometimes |
| five Escapes, polling for the region | arrived on attempt 1, or not in five |

⇒ **A keystroke is not a reliable harness primitive while a dock panel this
check itself raised is open.** A chord is routed through whatever holds
keyboard focus, and this check opens the Tool panel by construction — it has
to, the switches live there. Note the shape of the failure: `V` produced
*no line anywhere*, so the check reported the Tool panel as drawing the
wrong block when the truth was that nothing had ever reached the
application.

**The fix is a pointer, not a key**: step B clicks the View tab and then
`ribbon.item.view.tool_select`. Clicking a ribbon control is this harness's
most exercised primitive, it does not depend on focus, and it carries its
own oracle — the shell writes `ribbon-command-invoked id=view.tool_select`,
so *"the click did not land"* and *"the panel did not follow"* are now two
different messages instead of one ambiguous one.

And the arm it reaches is idempotent. `view.tool_select` calls
`canvas::tool::arm::select`, a plain write; the two neighbouring pointer
commands (`view.tool_hand`, `view.tool_text`) are **toggles** and would flip
on a second press. Choosing the one control on that row that cannot be wrong
about its own state is what makes this step deterministic rather than merely
more reliable.

Recorded here rather than left to be rediscovered: a check that fails
half the time is worse than one that fails always, because the failure gets
attributed to whatever changed most recently.

**Not every keystroke in this suite is suspect, and the distinction
matters.** Typing into a field the check has just clicked is fine — focus is
where the keys are meant to go, which is what `dimension_groups` and
`bookmark_add` do. What is unsafe is a keystroke that has to be *routed to a
command* while a raised panel holds focus. And a chord that IS the subject —
`tool_row`'s bare `T`/`A`, `find_bar`'s `Ctrl+F`, `read_mode_chrome`'s
`Ctrl+H` — must stay a chord: converting it would delete the assertion.

Step D drags **diagonally by equal amounts** on purpose. A non-uniform
resize of a pdfcer-authored appearance is fine — it is rebuilt — but making
the drag uniform keeps this check about the switch rather than about the
distortion refusal, which is a different feature with a different sentence.
