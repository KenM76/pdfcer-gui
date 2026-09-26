# `pdfcer-gui-base/text/settings/print_colour`

## Item notes

### `fn field_shade_title`

The copy leads with what the operator gains, not with what is drawn: they
are not looking for *"a wash over rectangles"*, they are looking for *"which
of these boxes can I type in"*. The radius line carries the fact that matters
most and is the one nobody would assume — **it never reaches the file.**

### `fn spot_model_title`

# Why this is a real choice and not a bug with a switch

`SpotColorantDeviceModel`, new in `pdfcer-core 0.20`. The two values are
**both conformant** and they disagree about what a spot colour is:

* *simulate separations* renders for a device that **has** the ink — the
  spot keeps its own plate and overprint preserves it. ISO 32000-2 §10.8.3.
  The engine's default, and the one the print-conformance corpus expects.
* *alternate space substitution* renders for the **actual composite device**,
  which has no such ink: the separation is converted through its tint
  transform the moment its space is set and is ordinary process ink from then
  on. ISO 32000-1 §8.6.6.4's `shall`, and what Acrobat's default view shows.

The visible difference is narrow and sharp: a **white object over a spot
colour knocks the ink out** under one model and **preserves it** under the
other. That is the whole of it, it only happens under overprint, and it is
the difference between a proof that matches the press and one that matches
the screen.

The copy below leads with the SYMPTOM rather than the standard, because
the operator arriving here got here by seeing white behave unexpectedly on a
print-ready drawing — not by reading §10.8.3.
