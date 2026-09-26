# `app::prefs::cache` — how much memory pdfcer may spend so a page it has
already drawn does not have to be drawn again

One preference. The operator's ask: *"increase cache to maximum for page
view so they don't constantly redraw with larger files."*

## A budget only bites if the cache keeps pages that are not on screen

`render::strip::StripRasters::retain` evicts by distance from the current
page, so the resident set is many times the visible set and the number here
is what bounds it. That ordering is what makes this preference mean
anything. A cache whose contents *are* the visible set is not a cache; it is
a frame buffer with extra steps — scroll a sheet off the top and it is gone,
scroll back and it is rendered again from the content stream, which
`BENCHMARK.md` measures at **691 ms** for a dense A1 drawing. Over such a
cache the eviction loop never runs and raising the budget changes nothing at
all, which is worth knowing: *"increase the cache"* is an instruction a
reader can carry out by editing one constant and reporting success.

## Why it is a preference and not a bigger constant

Because the honest answer to *"how much of this machine's memory may pdfcer
spend on page pictures"* is that only the person sitting at it knows. A
constant is this project guessing about a machine it cannot see, and the
guess has a bad failure mode in one direction: too large is not slow, it is
an allocation failure in a program that is now holding unsaved edits.

It follows [`crate::dialogs::settings`]' own standing rule, stated by the
operator — *where standards are ambiguous those should become settings that
the user can choose, with the initial installed default as the best guess of
what is usually followed*. There is no standard here, but the shape is the
same: a defensible default, named steps, and every one of them stating its
cost.

## Every step states its cost in megabytes, and that is not decoration

*"Large"* is not a number anybody can budget against. An operator with 8 GB
and an operator with 64 GB are making different decisions, and neither can
make theirs from an adjective. So the labels carry the figure —
`crate::text::settings::shell::page_cache_label` renders it — and the figures
are exact rather than rounded up, because a memory number that flatters
itself is the one kind of disclosure worse than none.

The arithmetic, once: a texel is one RGBA pixel, four bytes, and a megabyte
here is 1,048,576 bytes, so 256 M texels is 1,024,000,000 bytes and reports
as **976 MB**. There is no compression and no shared storage — these are GPU
textures — so the figure is what it says. The texel counts are round in
decimal and the megabyte figures therefore are not; the megabyte figure is
the one the operator is shown, so it is the one the tables below carry.

## Item notes

### `fn the_steps_go_up`

A control whose second entry held less than its first would read as
broken, and the labels are generated from the numbers, so a duplicate
would render as two identical rows.

### `fn the_label_and_the_spend_are_one_number`

Asserted as the *relation* rather than against four literals, which is
the whole point: two copies of one constant cannot disagree, so a test
written against literals would pass on a build whose label lied.

### `fn small_is_the_value_this_shell_shipped_with`

The row that makes the default reversible by name. An operator who finds
the default heavy must be able to ask for a smaller budget without
knowing that it is 48 million of anything, so the number behind the name
is held here rather than left free to drift.

### `fn the_default_is_large_and_maximum_is_offered_above_it`

Both halves are the decision recorded on the variant: the operator asked
for the maximum, and taking 2 GB on his behalf risks an allocation
failure in a program holding unsaved edits. The larger step exists and is
one click away.

### `enum PageCache`

# Why four steps and not a slider

[`super::quality::RenderQuality`]'s argument applies unchanged: *the useful
range is narrow, the middle of it is almost always right, and a slider
invites an operator to spend attention tuning a number that will not repay
it.* It applies with more force here, because the effect of an intermediate
value is unobservable — an operator cannot tell 900 MB from 1,024 MB by
using the program, so a slider would be asking for precision that cannot be
felt.

# The steps, and what each is for

The megabyte column is what [`Self::megabytes`] reports and what the label
shows; it is the texel count times four bytes over 1,048,576.

| | texels | RGBA | roughly |
|---|---|---|---|
| [`Self::Small`] | 48 M | 183 MB | a few large sheets |
| [`Self::Medium`] | 128 M | 488 MB | a report, or a dozen large sheets |
| [`Self::Large`] | 256 M | 976 MB | **the default** — about twenty-five large sheets at screen size |
| [`Self::Maximum`] | 512 M | 1,953 MB | a whole drawing set resident |

[`Self::Small`] is kept so an operator who finds the default heavy has a
smaller budget available **by name**, rather than having to discover a
number.

### `const ALL`

Smallest-first so the control reads left to right as *less … more*,
which is [`super::quality::RenderQuality::ALL`]'s rule and the direction
a reader expects of a quantity.

### `fn texels`

Texels rather than bytes because that is the unit the cache counts in,
and it counts in texels because **a page is not a unit of memory**: a
thumbnail and an Annex C sheet differ by four orders of magnitude, so a
page count that admitted six of the latter would admit 1.5 GB without
saying so.

### `fn megabytes`

Derived from [`Self::texels`] rather than written beside it, which is
this project's recurring lesson applied before it bites: two spellings
of one quantity drift, and the drift here would be a settings window
promising an operator 488 MB while the cache spent 2 GB. It is
`NO_SURFACE.md`'s standing rule — assert the *relation*, because two
copies of one constant cannot disagree — applied to a label.

### `fn key`

Stable across releases and deliberately not the display name: a display
name is operator copy and may be reworded, and a file whose keys moved
when the wording did would silently reset everybody's preference. Same
rule [`super::quality::RenderQuality::key`] follows.

### `fn from_key`

`None` rather than a default, so the loader can *report* an unreadable
value rather than silently substituting one — the per-key recovery
contract in [`super`]'s header.
