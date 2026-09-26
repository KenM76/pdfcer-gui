# `canvas::marquee` — **what a rubber-band takes, and why the direction
decides it**


## ★★★ The operator's report, `OPERATOR_REQUESTS.md` O88

> *"I can't box select the tables in the left or right top corners using the
> mouse — it only picks up the lines of each table, so I can't drag the
> entire thing and move it somewhere else, or cut/copy and paste it
> elsewhere."*

It was never a hit test that excluded text. Both tables sit hard against the
sheet edge, and this shell asked for `MarqueeMode::Enclosed` everywhere — an
object counts only if the band **completely surrounds** it. To surround a
table at the sheet edge the band has to start **outside the page**, and at
fit zoom there is barely a pixel of margin to start in. So the only band that
can actually be drawn is one *inside* the table, which surrounds a few short
rules and nothing else.

⇒ **"It only picks up the lines" is what an enclosing band returns when it
cannot be drawn big enough.**

## The remedy is a convention, not an invention

AutoCAD's direction-sensitive band, which SolidWorks drawings use too:

| drag | AutoCAD's name | takes |
|---|---|---|
| left → right | a **window** | only what it completely surrounds |
| right → left | a **crossing window** | anything it touches |

No modifier key, nothing new to learn, and it is the behaviour a
drawing-office hand already has. The standing instruction is to use the
conventional interaction rather than invent one, and the two alternatives are
both inventions here: Illustrator touches always, Inkscape puts touch on
`Alt`. The direction rule is the drawing-office one, and this is a drawing
program.

★ The enclosing band's answer does **not** change. `Enclosed` remains what a
left-to-right drag does and remains the right default on a dense sheet —
decision 011's reasoning is untouched. What was wrong was that it was the
only answer available.

## ★★ The half that was found by a failing test rather than by thinking

See [`without_page_wrappers`]. A crossing band touches a page-sized form
XObject on **every** drag, so on a wrapped drawing every crossing selection
would have quietly included the whole sheet — and the operator's next gesture
moves it. Under `Enclosed` that could not happen, which is why it is new.
