# `ui-verify/checks/widget_rotate`

`turning_a_field_right_turns_it_right` — the driven proof of
`OPERATOR_REQUESTS.md` **O62**'s rotation half.

# This check exists for ONE arithmetic sign

`/MK /R` is **counterclockwise**. The page's `/Rotate` is **clockwise**. The
engine flagged this as *"the single most likely thing for a shell to get
backwards"*, and the standard makes it easy — the two entries are word for
word parallel:

| | |
|---|---|
| `/MK /R` (Table 189) | *"…rotated **counterclockwise** relative to the page…"* |
| page `/Rotate` (Table 30) | *"…rotated **clockwise** when displayed or printed…"* |

**The direction word is the only difference between those two sentences**,
and the *movie* dictionary's `/Rotate` uses the same *"relative to the page"*
phrase with the opposite sense — so the phrase carries no convention at all.

⇒ A shell that got this backwards would ship two buttons that both work,
both write a legal angle, and both turn the box **the wrong way**. Nothing
fails. Every unit test of the arithmetic passes, because the arithmetic is
not what is wrong — the *meaning* is.

# The oracle, and why a sign is exactly what it reads

Press **Turn right** once on a widget at 0, and assert the engine received
**270**.

- right = clockwise = **−90**
- −90 normalised into `0..360` = **270**

If the negation were missing, the same press would produce **90** — a legal,
successful, silent rotation the wrong way. That single number is the whole
subject, and it is why this check asserts a value rather than a change.

It reads `rotate-widget-applied … now=`, the **engine's** report, not the
shell's request line. The request says what the panel computed; only the
applied line says what `rotate_widget` was actually given and accepted.

## Item notes

### `const INVOKE`

The properties panel is **not** opened here, and the first version's
attempt to is why.

`view.panel_properties` is a TOGGLE, and opening it re-docks the canvas
narrower — after which the coordinate this check computed for its placement
click pointed somewhere else. The symptom was not a missed click: it was
*"the window containing (1224, 538) could not be brought to the front"*,
three runs running, because the point had moved over a different window
entirely.

⇒ `D:/dev/rag/egui/` already carries this as **harness coordinates going
stale when a dock width changes**. The panel is open in Edit mode by
default, so the toggle was never needed — it was doing nothing but moving
the canvas out from under the check.

### `const ROTATE_RIGHT_REGION`

Its own, not a fraction of the row's. The first version took the row's
rect and aimed 78 % across it — coordinate arithmetic the harness already
has `declared_center` for — and landed outside the window entirely, which
surfaced as *"the window could not be brought to the front"* three runs
running. A named control is aimed at by name.
