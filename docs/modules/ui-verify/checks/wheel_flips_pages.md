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

## Item notes

### `const NOTCHES`

One physical detent is 50 logical points on this platform and the
application's threshold is 40, so one notch is enough — but the harness's
wheel and the platform's may disagree about how much a "notch" is, and a
run that under-delivered would report "the wheel does not flip" for a build
where it does. Two notches is comfortably over one threshold and, because
the accumulator is **zeroed** on each turn rather than decremented, still
buys exactly one page.

### `fn wheel`

The instrument that makes this check **re-runnable**, and it did not
exist until this check needed it. The setting is PERSISTED, so a run that
turned it on left it on, and the second run of this check inherited the
first one's choice and reported the shipped default as broken — a
confident, specific, wrong accusation aimed at the part of the build a
reader can least easily check.

The standing rule this repeats: **a driven check that mutates persisted
state must normalise at the START**, and to normalise it must first be able
to read. A setting a check can change is a setting the trace must state.

### `const FIXTURE`

A wheel that turns pages needs a page to turn to, and the sweep's shared
fixture has exactly one. Run against it, every claim up to the toggle passed
and then the check accused the application of ignoring its own preference,
in three confident paragraphs, on a document where the correct behaviour is
to do nothing. Pinned rather than tabled so a hand invocation gets it too.
