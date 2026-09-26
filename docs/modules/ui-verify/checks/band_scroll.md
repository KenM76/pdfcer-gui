# `ui-verify/checks/band_scroll`

`band_scroll` — **a ribbon command past the fold is still reachable**, and
the harness proves it by walking the band to it.

# The defect this exists for was in the HARNESS, and it reported the
application as broken


On the operator's instruction — *"do the scroll like Word"*, asked twice —
the dropdown became a `›` arrow that **shifts the band by exactly one
group** (`egui-shell`'s `ribbon::band`: `set_first(.., scrolled + 1)`). The
published region name stayed `ribbon.overflow`, deliberately, because it is
a cross-repo stability contract. So nothing renamed, nothing failed to
compile, no test went red — and the helper carried on clicking once.

**And the exact mechanism is one step subtler than that, which is why
it was measured rather than reasoned about.** The old order was: open every
**collapsed** group at the band's starting position; scroll **once**; then
look at the band **bare**. So the hole is not simply "two or more scrolls" —
it is *any command that needs a collapsed group opened at a stop past the
first*, which at 1,100 pt is About, Shortcuts and Properties after exactly
ONE scroll. The first version of this check asserted `scrolls >= 2`, ran,
and SKIPPED against the very build it was written for. Driving corrected the
diagnosis; the reasoning had been plausible and wrong.

Either way the command was reported absent, in a confident sentence naming
it:

> *"no `ribbon.item.file.about` region on the File tab or in its
> overflow."*


⇒ **A helper's prose and the mechanism it drives agreed when the prose was
written.** That is the shape this project keeps meeting, and the reason a
fix to it needs a check of its own rather than a corrected paragraph.

# Why the assertion is more than "it was found"

Because *"it was found"* passes on the broken build. Every other caller of
the helper asks a yes/no question, and on a correct application the answer
is `Some` whether the search took one stop or five. A check that asserted
only reachability would be green against the single-click implementation for
any command that happens to sit one stop past the fold — which is most of
them, and is exactly why the bug survived a full sweep.


If a future ribbon change puts About one stop away, this check SKIPS
rather than passing, and says it needs recalibrating against whatever
command is then furthest right. A check that quietly loses its power is
worse than one that says so — see `crate::checks` rule 5.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | launch at the harness's default width, **no** `maximize()` | the File tab draws and `ribbon.overflow` is declared — i.e. the band really is scrolled short |
| B | confirm the subject is NOT on the band as it stands | `ribbon.item.file.about` absent; otherwise SKIP, the case cannot occur at this width |
| C | `driving::search_the_band` for it | found, by a walk the old one-click search could not have made |
| D | press it | `dialog:about` declared — the rect the search returned was live, not a fossil |
