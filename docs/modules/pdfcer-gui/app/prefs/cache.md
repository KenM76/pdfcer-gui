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
