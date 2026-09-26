# `ui-verify/checks/tool_row`

`the_text_tool_types_on_one_click` and `the_points_tool_shows_points_on_one_click`
— **the operator's own two gestures**, driven.

# What these are for


> *"How do I select and edit end points on the canvas? How do I edit text
> when on the canvas? I get a box and the I cursor, but I can't type
> anything. How do I make new text when I click on the canvas and expect to
> edit there? Same problem as the previous. How do I get to see the end
> points of an object and select them to drag and move? This doesn't work
> either."*

And then the diagnosis, which was correct and is the reason both of these
checks exist:

> *"The selector should be predictable like other programs. It seems a lot of
> ideas are getting invented instead of just using the … most common method
> expected."*

Both features **existed**. Reaching them was invented:


Neither ritual is discoverable and neither resembles any other program. The
fix was to make the **tool the rung**: press `T`, click, type; press `A`,
click, see the points. That is Illustrator, Inkscape, Figma, CorelDRAW and
Word, and it is what these two checks assert.

# ★★ Why the assertion is "ONE click"

Because the count is the feature. A check that armed the tool from the
ribbon, clicked, and asserted a caret would pass on the **old** build too —
the old build could do all of that, it just needed four steps to get there.
So each check performs exactly one press of one key and exactly one click,
and asserts the outcome. Anything that needs a second click fails.

★ And the key is pressed as a **bare letter through the OS**, not as a
command dispatched by name. `V`/`A`/`T`/`H` being bare is the whole
convention being adopted, and a bare letter is the one chord shape that can
be broken by a stray focus — `canvas::keys` gates every keystroke on
`text_edit_focused()`, which is `DEFECTS.md` D1's guard, and D1 is this
project's canonical example of a keyboard rule that was right in the test
harness and wrong in the running window.
