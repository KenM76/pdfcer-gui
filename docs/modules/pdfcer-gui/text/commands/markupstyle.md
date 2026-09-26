# `text::commands::markupstyle` — the labels and tooltips of **Format ▸
Markup**, the six controls that restyle a mark that is already on the page

## ★ Why this is a file of its own

**R2** — no source file over 1,500 lines. [`super`] stood at 1,438 when
these five arrived, and five `CommandText`s written to this project's
register (every choice carrying its *why*) is comfortably more than the
62 lines of headroom that left. `tools/gates/check-file-size.sh` says in its
own header that shaving prose to fit a threshold is the behaviour it exists
to refuse, so the subject moved instead.

The seam is [`super::annotate`]'s, one step further along the same line.
That file holds the strings of the commands that **place** a mark — the
Markup and Measure tabs, *what you add on top of the page*. This one holds
the strings of the commands that **restyle a mark already placed**, which is
the contextual Format tab's half of the same subject and is governed by a
different engine verb (`EditSession::set_markup_style`, not
`add_markup`), a different operand (an `ObjId`, not a gesture), and a
different question in the ribbon (*what is selected?*, not *what am I about
to draw?*).


## ★★★ Every tooltip below has to read correctly in TWO states

The same constraint that shaped the Font block in [`super`], for the same
mechanical reason: `egui_shell::ribbon::control::render_command` shows a
command's tooltip with `on_hover_text` when the control is live and
`on_disabled_hover_text` when it is not — **the same string** — and
[`crate::app::markupband`] reproduces that behaviour by hand for the custom
items, because the shell does none of it for an `Item::Custom`.

The states differ from the Font group's, though, and the difference decides
the wording:

| group | absent when | greyed when | so the tooltip must also say |
|---|---|---|---|
| Font | the mode cannot edit content | nothing is swept | **how to sweep** — O37's *"nothing tells you to press T"* |
| Markup | no markup is selected, or the mode cannot author markup | the mark is **locked**, or its geometry did not read | nothing extra |

⇒ The Font tooltips end *"Sweeping text with the Text tool (T) chooses what
it applies to"* because an operator meets those controls greyed and has no
way to guess the gesture. These six are **absent** in that situation rather
than greyed — the group is not drawn at all until a mark is selected — so an
operator only ever meets them with an operand already in hand, and a clause
telling them to select something would be describing the thing they just
did. The one greyed state that remains has its own sentence, drawn by
[`crate::app::markupband`] from [`crate::text::panels::properties`], because
*"this mark is locked"* is a fact about that annotation and not about the
command.

## ★★ What the fill tooltip has to say, and why it is the longest

`canvas::markup::spec` authors every shape with `interior: None` — no fill —
and its reason is quoted in `panels::properties::markup`'s header: *"a
filled comment shape hides the drawing it is a comment about, which on a CAD
sheet is the whole content under it."* Acrobat's default is the same.

So the operator's mental model is *marks have no fill*, and a control that
offers one has to answer two questions at once: what it does, and how to get
back. Its tooltip therefore names the **no fill** state explicitly, because
that state is the one the mark started in and the one an operator will want
to return to after trying a fill on a drawing.

★ This does **not** change what new markup is authored with. The pen is
`canvas::markup::pen`'s and it is untouched; this restyles one existing
annotation, which is a different act with a different verb.
