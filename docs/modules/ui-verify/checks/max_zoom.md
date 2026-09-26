# `ui-verify/checks/max_zoom`

`the_zoom_readout_opens_the_maximum_zoom_popup` — a readout that turned into
a button, proved to actually do something.

# Why this exists


> *"put the max zoom setting on the bar at the bottom."*

The status bar's zoom readout is now a button that opens a list of maximums.
★ That is precisely the shape this project has shipped broken twice in one
week: the **Select** popup went out with a double toggle that made its button
do nothing, green on 1,628 unit tests, 17 gates and a smoke launch that
confirmed the button's rect was drawn in the right place. Every one of those
observed the *button*, and the button was never the broken part.

So this asserts the thing an operator would: **the popup opens, and it
contains its rows.**

# ★★ What it deliberately does not assert, and why that is honest

It does not click a row and check the preference changed. That would be the
stronger claim, and it needs the preferences file — which is written to a
shared profile directory the harness would then be mutating under a running
application. `select_filter_changes_what_a_click_hits` learned that lesson
the expensive way: it left a persisted filter behind, its next run started
with everything switched off, and it blamed the fixture.

The gap is covered from the other side instead. `app::status::maxzoom`'s
unit tests assert every preset is a value the preference accepts and that
the default is one of the rows, and `app::prefs` asserts the round trip
through the file. What no unit test can see is whether the popup opens at
all — and that is exactly what this check is for.
