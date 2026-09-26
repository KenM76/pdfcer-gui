# `ui-verify/checks/font_group`

`the_format_tab_offers_font_controls_for_swept_text` — **the ribbon route
to a restyle, and the sentence that tells an operator how to reach it.**

# What this is for, and how it differs from `restyle_text`


O37 shipped with an admission written into its own row:

> You are in Edit mode, so a drag with the Select tool draws a marquee round
> objects. Press **T** first — that arms the text tool — then sweep across
> the words. ★ **That is a discoverability gap and it is ours, not a
> limitation. Nothing on screen tells you to press T.**

Three surfaces now do. This check asserts two of them in the state an
operator is actually in when they need them, which is the state **before**
anything is swept — and that ordering is the whole design of the check.

## ★★★ The two phases, and why the first one has to come first

| phase | the operator's state | what must be true |
|---|---|---|
| 1 | clicked a piece of text with the Select tool; **nothing swept** | the Format tab appears, it carries a **Font** group, and the Properties panel says how to get an operand |
| 2 | pressed `T`, swept the words | the ribbon's **Bold** commits a restyle to the document |

Phase 1 cannot be reached after phase 2, because a sweep is not undone by
clicking again — so a check that swept first would have destroyed the state
it is meant to observe. That is not a harness convenience; it is the
operator's own sequence. They click the thing they want to change *before*
they know a sweep is needed, which is precisely why the gap existed.

## ★★ What phase 1 can and cannot see, said plainly

The ribbon publishes `ribbon.item.<id>` for **every** command control,
enabled or greyed — deliberately, and `egui_shell::ribbon::control`'s own
note says why: *"the question a consumer asks is where is this control, and
a control that is greyed is still a control that was drawn somewhere."*

So a region tells this check the control is **on screen**. It does not tell
it the control is greyed. That is a real limit and it is not papered over:
the greying is asserted by
`app::conditions::tests::the_font_groups_visibility_follows_the_mode_and_its_enablement_the_sweep`,
which reads the registered command's own predicate against the published
conditions — the join, not either half. What *this* check adds, and what no
unit test can, is that the controls are **drawn at all**, on a real ribbon,
in a window, at a real width, on the tab that really appeared.

★ And the appearing is itself under test. The Format tab is contextual: its
`visible_when` moved from `selection.any` to `selection.formattable` when
the Font group landed, and a build that missed that change shows **no tab**
after a sweep and therefore no Font group — a whole feature with no surface,
which is exactly the shape of defect this project exists to catch.




★ The answer was in the trace the check was already holding:
`pdfcer-diag properties-panel object=832 kind=Path notes=0`. So
[`aimed_at_one_text_object`] now reads that line, plus
`canvas-selection … sel=N`, and **skips** — never fails — when the click did
not leave exactly one text object selected. Phase 2 had this guard from the
start (`chars == 0` is a skip, not a failure); phase 1 did not, and the
asymmetry is what let a correct program be blamed.

★★ The correct aim for this fixture is `--doc-point 0,1140,62`, which
`RESUME.md`'s aim table gives and the sweep did not use: a 5 pt title-block
run at PDF (1135.7, 58.4)–(1190.5, 63.4).

# The oracle

Phase 1: the precondition above, then the regions `ribbon.tab.format`,
`ribbon.group.format.font`, the five `ribbon.item.format.*`, and
`properties.text` with `properties.text.face` inside it.


Phase 2: `text-style-applied … applied=N` **and** the `format-text` label
`vector_edit` writes when the edit reached the engine — the same two-line
oracle `restyle_text` uses, for its reason: the first without the second is
a module that decided to act and whose action never landed.
