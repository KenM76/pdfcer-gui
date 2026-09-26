# `ui-verify/checks/button_action`

`a_placed_button_can_be_given_something_to_do` — **the whole of
`OPERATOR_REQUESTS.md` O60/O61's open half, driven.**

# The report

O60 and O61 both carried the same open line for a fortnight:

> ⬜ *"push buttons that actually do something"*

And the Button tool was **greyed**, with a sentence saying pdfcer *"cannot
give a button something to do yet, so it will not place one."*

## The finding that matters more than the feature


> *"Please check your own copy. If your surface tells the operator that
> pdfcer never authors an action, it is now saying something untrue in the
> direction that matters."*

**Two days passed.** The reply was read, filed, and the sentence stayed on
screen — because nothing in this repository fails when a capability lands.
Three things now do: this check, the tripwire in
`canvas::formfield::action`, and
`canvas::formfield::tests::no_kind_is_authorable_but_inert`.

## Why this cannot be a unit test, and cannot be a screenshot

The chain has six links and only the last two are unit-testable:

| # | link | its own test |
|---|---|---|
| 1 | the ribbon item is not greyed | **nothing** — greying is drawn by `egui` from a condition string |
| 2 | the command arms the tool | now `the_push_button_arms_its_tool_like_every_other_kind` |
| 3 | a drag opens the placement dialog | **nothing** |
| 4 | the dialog draws an action chooser, and it opens | **nothing** |
| 5 | a popup row is clickable and changes the draft | **nothing** |
| 6 | Add authors the button **and then writes the action** | unit-tested per half, never together |

And **a screenshot cannot judge link 6 at all.** That is rule 4 working
correctly: a button carrying `/A` is drawn exactly as a button without one,
because applied content renders as saved content will. There is no badge, no
tint and no dashed outline — so the only oracle that exists is the trace.

```text
button-action-applied name=Button1 kind=ResetForm replaced=none
```


## The sequence

| # | step | oracle |
|---|---|---|
| A | Edit mode, arm the button tool, drag out a button | the dialog's chooser region is declared |
| B | click the chooser | the popup's rows are declared |
| C | click *Clear the form* | `button-action-chose kind=ResetForm` |
| D | press Add | `button-action-applied … kind=ResetForm` |

Steps B–D read a **child viewport's** rectangles, so every click goes
through [`frame_of`] rather than `session.frame()`. The placement dialog is
a real OS window; its rects begin at `x=0` because they are relative to its
own client origin, and converting them against the main window aims hundreds
of points away at numbers that look perfectly ordinary. That mistake has
been made four times in this harness and each time produced a confident,
precise and entirely wrong diagnosis of the subject.

Step A **drags** rather than clicks. A clicked button is authored at its
default 80×22 pt, which is fine — but a drag is the operator's own route
(O53) and it makes the dialog's arrival unambiguous.
