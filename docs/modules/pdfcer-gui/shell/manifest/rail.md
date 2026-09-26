# `shell::manifest::rail` — what is in the left rail

`OPERATOR_REQUESTS.md` **O123** part 7, his words:

> *"What I'd also added in the bar at the left side that we are adding: the
> navigate selectors and some other related selection controls (lasso tool
> when we implement one, etc) and these will fold up into a drop down arrow
> if space becomes scarce."*

and **O126**'s addendum, same conversation:

> *"also add rotate pages to that area, and those should be available in
> every mode including read."*

`mockups/pdfcer-shell.html` draws this and he approved it. This file is the
list; [`egui_shell::dock::rail`] is the geometry and the fold ladder.

## The four groups, and the one that is deliberately last

| group | fold | why |
|---|---|---|
| the **panel tabs** | [`RailFold::Never`] | *"every panel one click away"* is the rail's entire argument for existing |
| **navigate** | [`RailFold::PinArmed`] | four mutually exclusive modal tools **and the smart selector**; at the floor the strip must still say which TOOL you are holding, which is why the pin never goes to the toggle |
| **select** | [`RailFold::Whole`] | the specialist gestures; nothing in it is reached by habit |
| **rotate** | [`RailFold::Whole`] | O126, and see the ⚠ below about Read |

## ⚠⚠ THIS LETS READ MODE DIRTY A DOCUMENT, AND THAT IS HIS CALL

`pages.rotate_left` and `pages.rotate_right` write `/Rotate` into the page
dictionary. **That is a document edit, not a view transform** — the file is
modified, the undo log gains an entry, and the title bar gains its dirty
marker. Both commands are `enabled_when("doc.pages")` with **no mode gate**,
so they already worked in every mode; what this file adds is their
*placement*, on a strip that is on screen in Read.

⇒ So after this change **Read mode can dirty a document**, which cuts
against the standing *"Read authors nothing"* invariant that shapes every
other gate in this manifest.

**It is built this way because he asked for it, in those words, and the
record should say so rather than let a later reader find it and file it as
an oversight.** The alternative that was considered and rejected in O126 is
a view-only rotation in Read and a real one elsewhere — *"two behaviours
wearing one button, which is worse than either"*. If he wants view-only
rotation, that is a different control and it should say so on its face.

## The lasso is NOT here, and the empty-group question was decided

He named the lasso himself — *"lasso tool when we implement one"* — and it
does not exist: no command, no handler, no icon asset. The mockup draws it
dashed with the freehand pen's borrowed art, which is a thing **a mock may
do and the product may not**: R9 says an unavailable capability renders
*nothing*.

Two shapes were available and the choice matters for what happens next:

1. a `select` group **present but empty**, as a placeholder; or
2. a `select` group carrying the selection controls that **do** exist.

(1) is refused. An empty group draws no caption and no rule — the planner
skips it — so it would be data that renders nothing, and the next reader
would have to run the code to discover that. (2) is what ships, and it is
possible only because of the second finding below.

## Every rail row needs an icon, and that is what admits `edit.select_all`

A rail row is a picture with an *optional* word under it: at `Rung::Tight`
and below there is no word left, so a command with no icon would draw a
blank rectangle. `edit.select_all` has its own glyph, so it qualifies, and
it belongs in `select` by meaning — he asked for *"some other related
selection controls"*, and Select all is one. The rule is enforced in both
directions by the icon test below: removing an icon from a rail command
turns its row into a blank, and nothing else in the build would notice.

⇒ The lasso's seam is therefore a **one-line** change in a group that is
already on screen: add `Item::command("edit.lasso")` beside Select all when
the tool exists. It is marked in the code below.
