# `ui-verify/checks/wheel_flips_pages`

`the_wheel_turns_pages_when_the_operator_asks_it_to` — O30, driven end to
end from the status-bar toggle.

# The report


> *"when in single page view there should be an option on screen near the
> button to scroll or flip through pages, or the current way it is now when
> the scroll wheel is used."*

# What makes this check able to fail, and it is not the page number

The obvious check — *turn it on, roll the wheel, did the page change?* —
would pass against a build whose toggle wrote a preference nothing reads,
provided some **other** path happened to change the page. So the run does
three things in order and each is a separate claim:

1. **The default is silent.** Roll the wheel with the toggle OFF and assert
   the page does **not** change. Without this, a build that flipped pages
   unconditionally — ignoring the setting entirely — would pass everything
   below it. It is also the direct assertion that O30 did not change what
   the operator already had.
2. **The toggle turns it on**, and the very next notch turns a page. The
   *very next* matters: the preference is a snapshot on `OpenDoc` for every
   other setting in the program, adopted when the Settings window is
   applied, and a build that let this one wait for that would look correct,
   write the file correctly, and change nothing on screen. That is the
   silently-inert control this suite exists for.
3. **It goes both ways.** Roll the other way and assert the page goes back.
   A sign error is a viewer that works and feels wrong, which is harder to
   notice than one that is broken.

# And the control is not drawn where the choice does not exist

R9. Under a continuous display mode the wheel scrolls the whole document by
definition, so there is no second answer to offer and nothing is drawn —
not a disabled stub. The run switches to Continuous and asserts the
`status-wheel-paging` region **stops being declared**, which is the only
claim in this check that is about an absence.

An absence is admissible here under this module's rule 4 precisely
because the same region is shown to be present, in the same run, moments
earlier: the instrument that would have reported it is demonstrably
working.

# The document is pinned, not taken from `--pdf`

Every claim here is of the form *the page number changed*, so a one-page
document makes the correct behaviour indistinguishable from the defect. The
count is also asserted, because a pinned fixture is only a claim about what
is on disk.
