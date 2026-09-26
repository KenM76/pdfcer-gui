# `dialogs::settings::measuring` — the group the source did not have

One setting: the angular tolerance below which two lines are dimensioned as
a **distance** rather than as an **angle**.

## Why this is a group of its own

In the old shell `parallel_epsilon_degrees` lived under *Copying and
extracting text*, where it has nothing whatever to do with either. It was
there because it happened to be a slider, like the word-gap one beside it.

The group headings in this window are its **whole navigation model**: an
operator arrives with a *symptom* and the headings are how a symptom finds
its setting. The symptom here is *"my dimension came out as an angle"*, and
nobody with that symptom opens a heading about copying. A setting filed
under the wrong one is not untidy, it is unreachable — which for a setting
the operator specifically asked for is the worst outcome available.

It is a group rather than a move into *Pages and printing* because
dimensioning is a growing subject with an obvious next tenant: the scale
model, the drafting standard, and the precision and unit defaults that the
measure tools currently compile in. Those belong beside this, and a group
that exists now is one they can arrive into rather than a second
reorganisation later.

## Rule 15 applies here, in the operator-facing copy

What this governs is a **ce dimension** — one pdfcer authors — and not a
**pdf dimension**, which is CAD-exported page content pdfcer reads and must
not silently alter. The copy avoids the bare word entirely and says *"new
dimensions you draw"*, which is unambiguous without making the operator
learn a distinction that is ours rather than theirs.

## Item notes

### `fn a_hand_edited_value_inside_the_stores_range_is_not_rewritten`

The regression test for the silent-edit hazard both sliders in this
window carry — stated once here and once in [`super::super::text`].

# Why it drives the parser instead of comparing constants

The obvious test is `assert!(MIN <= 0.0 && MAX >= 45.0)`, and it is
worthless twice over: both operands are `const`, so the compiler folds
it away and clippy rightly refuses it, and it asserts a relationship
between two constants rather than the property that matters. The
property is about **behaviour**: a number the settings file accepts must
still be that number after this window has had it.

So it goes through `Settings::parse`, which is the store's own
validation, and asserts on its **notes**. A value the parser clamps
pushes a `Clamped` note; a value it accepts pushes none. If the slider's
bounds ever narrow below the parser's, this window would rewrite a
legal hand-edited value on open — and Save would write the changed
number back, an edit the operator never made and cannot see, because
they never touched the control.

### `fn the_store_clamps_beyond_its_range_and_discloses_it`

The other side of the test above, and the reason the first one proves
something. If the parser accepted everything, "the slider matches the
parser" would be satisfied by a slider with no bounds at all.

### `fn the_shipped_default_is_reachable_on_the_slider`

A default outside its own control's bounds would be silently rewritten
the first time anybody opened this window, on every machine, without a
click.
