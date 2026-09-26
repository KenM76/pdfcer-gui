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
