# `ui-verify/checks/text_annot_focus`

`text_annot_focus` — **the dialog takes the keyboard without being clicked.**


> *"adding text does bring up a window and a prompt, but it doesn't type
> anything in the box when I type and nothing gets added."*

# ★★ Why this exists when `text_annot_places_and_authors` already types

Because that check **clicks the text field before typing into it**, and an
operator has no reason to. A dialog that opens with a caption asking for
words is asking to be typed into; nobody clicks a field that is already
showing a caret. So the existing check sets up the very state it is meant to
be testing, and passes on a build where the dialog never takes focus at all.

That is the third time in this project a green assertion has pointed in the
right direction and measured the wrong thing, and it is worth naming the
shape rather than the instance: **a driven check that arranges the
precondition it is asserting about is not a driven check.** If a step in the
script is one the operator would not perform, it belongs in the assertion,
not in the setup.

# The defect it was written against

`dialogs::textannot::field` latched on having **asked** for focus:

```ignore
if !self.focused_once {
    response.request_focus();
    self.focused_once = true;
}
```

Asking and holding are different facts. The dialog's first draw is the frame
*after* the gesture that opened it — the canvas raises `BeginTextAnnot` and
the action queue drains at frame end — so the pointer release that finished
the drag is still being resolved around the request, and egui keeps the
earlier of two requests made in one pass. The one attempt was spent on the
frame most likely to lose it, and nothing ever asked again.

# The oracle, and why it is indirect on purpose

Nothing traces the draft's length, and adding a trace for it would be adding
a seam to observe a thing the operator observes directly. So this asks the
question the way they do: **Accept is greyed while the field is empty**
(`kind.uses_gallery() || !text.trim().is_empty()` — `dialogs::textannot`),
so clicking Accept after typing either authors an annotation or does
nothing, and those two outcomes are exactly "the keystrokes landed" and
"they did not".

Read the failure message before believing a green: an authored annotation
proves the field had focus, which is the whole claim.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | Review mode, Markup tab, arm **Text box** | `markup-tool tool=TextAnnot(..)` |
| B | drag a box on the page | `text-annot-open`, `dialog:text-annot` declared |
| C | **type immediately — no click on the field** | nothing traced; the draft is not observable |
| D | click Accept | `add-text-annot`, one more than before |
