# `ui-verify/checks/double_click_text`

`double_clicking_a_text_box_edits_the_text` — **the last rung of the
Smart-Selector chain.**

# The request

`OPERATOR_REQUESTS.md` **O70**:

> *"Selecting a text box or similar item does the same thing, but
> double-clicking inside the bounding box should edit the text."*

Every other rung of that chain goes **deeper into the geometry** — a
container to what is in it, an object to its parts, a part to its points.
Text is where the chain means something else: the thing below a text object
is not a smaller shape, it is *the words*, and a double-click that descended
to a show-operator run would be technically consistent and useless.

## The tool is armed, and that is the convention rather than a side
## effect

Inkscape's selector switches to the text tool on this gesture; Illustrator's
does the same. pdfcer could place the caret without arming — its typing path
runs whatever tool is selected — and that would leave the operator in a
state no other program has: a caret blinking in the page while the arrow is
still the tool, so their next click means *select* when everything on screen
says they are typing.

⇒ So the check asserts **both**: the caret opened, and the tool that owns
carets is the one now pressed.

## The sequence

| # | step | oracle |
|---|---|---|
| A | Edit mode, click the text | a selection line |
| B | double-click it | `text-edit-caret kind=Edit page=0 …` |
| C | …and the caret tool is armed | the Tool panel draws its `tool.armed` block |

Step C reads a **dock region**, not the ribbon's pressed state, and the
first run is why: the ribbon shows one tab at a time, so `view.tool_text`
was simply not on screen and the check SKIPPED on a build where the feature
worked. A region absent because it is on another tab looks exactly like one
absent because the feature is missing. The Tool panel is drawn whatever tab
is showing, and it is the surface an operator reads while a tool is armed.

## Item notes

### `const ARMED_BLOCK`

Chosen over `ribbon.item.view.tool_text`'s pressed state after the first
run SKIPPED on it: the ribbon shows one tab at a time, and this check leaves
the operator on whichever tab the mode selector last drew, so the View row
is simply not on screen. A region that is absent because it is on another
tab looks exactly like one that is absent because the feature is missing.


⇒ So its presence is a stronger claim than the old armed block's was: the
block drew for any armed tool *and* sat inside a panel an operator could
close, while this is chrome that cannot be closed and appears for exactly
the condition under test.
