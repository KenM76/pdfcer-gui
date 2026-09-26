# `ui-verify/checks/preset_group_reachable`

`the_standards_presets_group_is_reachable` — the conformance presets'
heading is on screen in the Settings window, not merely laid out in it.

# What this is for, `OPERATOR_REQUESTS.md` O100

> *"the engine I think has a couple of new options for colour rendering that
> we might need to surface and set for our standards presets."*

The *surfacing* half of that ask is already held by two unit-level
contracts, and they are strong ones:

* `every_setting_the_store_carries_has_a_control_in_this_window` enumerates
  the **engine's own** `Settings::write_to_string` at runtime and demands a
  control for each key. It has caught a newly-added setting four times.
* `every_key_a_standard_leaves_alone_has_an_operator_facing_title` does the
  same for `PresetKey`.

Neither of them, and nothing else, answers the question this check exists
for: **can the operator actually get to it?**



That is the second time in one session a check I wrote was wrong rather
than the code, and both had the same shape: **a measurement aimed at the
wrong surface looks exactly like a broken feature.** The rule that catches
it is to ask what a failing assertion actually *sampled* before asking what
is broken.

⇒ So the claim is: the **heading** is visible, which for a collapsed group is
the whole of "can the operator get to it". The row behind it is one click
away by design, and opening it needs the pointer — there is no command that
focuses this group the way `tools.font_folders` focuses Fonts, so a no-input
route to the row does not exist. Named rather than left as a silence.

# Why "reachable" needs its own check, and it is a named hazard here

`D:/dev/rag/egui/` records this project shipping **panels that were
unreachable in real builds with every gate green**. A control inside a
`ScrollArea` is laid out whether or not anyone can see it, and `egui` will
happily report a rectangle for a row a hundred points below the fold. So a
test that finds the widget in the tree, and a check that finds a rectangle in
the trace, can both be satisfied by a row nobody can reach.

Both the heading and the row publish through `crate::diag::ui_rect_visible`
— intersected with the clip rectangle, so an off-screen row publishes
nothing at all. This check asserts the region exists, which is therefore a
claim about **visibility** and not merely about layout.

The same reasoning already burned this suite once in the other direction:
`settings_headings_legible` measured three headings that were laid out below
the fold and sampled the Pages panel and the drawing behind the dialog,
reporting three illegible headings in a dialog whose visible headings
measured 13.91:1. The fix there was `ui_rect_visible`; this check is what
makes its absence detectable rather than silent.

# No input

`PDFCER_DIAG_INVOKE` raises the command at startup, so the Settings window
opens without a pointer. Like `title_build_stamp` and `field_shading`, this
can run beside somebody using the machine.

Settings is also one of the few windows that must work with **nothing open**
— the presets are a preference, not a property of a document — so this drives
it on an empty shell, which is what proves that.

# What a passing run does NOT prove

That the *claim sentences* are shown. Those are labels with no region of
their own, and they appear only under the selected standard — which means
selecting one, which means the pointer. Their content is held by
`a_pdf_x_preset_says_a_composite_viewer_will_differ_and_pdf_a_does_not` and
their delivery would need a second, input-gated check. Named here so the gap
is a decision rather than an oversight.
