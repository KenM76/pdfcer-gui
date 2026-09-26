# `canvas::marquee` — **what a rubber-band takes, and why the direction
decides it**


## The operator's report, `OPERATOR_REQUESTS.md` O88

> *"I can't box select the tables in the left or right top corners using the
> mouse — it only picks up the lines of each table, so I can't drag the
> entire thing and move it somewhere else, or cut/copy and paste it
> elsewhere."*

It was never a hit test that excluded text. Both tables sit hard against the
sheet edge, and this shell asked for `MarqueeMode::Enclosed` everywhere — an
object counts only if the band **completely surrounds** it. To surround a
table at the sheet edge the band has to start **outside the page**, and at
fit zoom there is barely a pixel of margin to start in. So the only band that
can actually be drawn is one *inside* the table, which surrounds a few short
rules and nothing else.

⇒ **"It only picks up the lines" is what an enclosing band returns when it
cannot be drawn big enough.**

## The remedy is a convention, not an invention

AutoCAD's direction-sensitive band, which SolidWorks drawings use too:

| drag | AutoCAD's name | takes |
|---|---|---|
| left → right | a **window** | only what it completely surrounds |
| right → left | a **crossing window** | anything it touches |

No modifier key, nothing new to learn, and it is the behaviour a
drawing-office hand already has. The standing instruction is to use the
conventional interaction rather than invent one, and the two alternatives are
both inventions here: Illustrator touches always, Inkscape puts touch on
`Alt`. The direction rule is the drawing-office one, and this is a drawing
program.

The enclosing band's answer does **not** change. `Enclosed` remains what a
left-to-right drag does and remains the right default on a dense sheet —
decision 011's reasoning is untouched. What was wrong was that it was the
only answer available.

## The half that was found by a failing test rather than by thinking

See [`without_page_wrappers`]. A crossing band touches a page-sized form
XObject on **every** drag, so on a wrapped drawing every crossing selection
would have quietly included the whole sheet — and the operator's next gesture
moves it. Under `Enclosed` that could not happen, which is why it is new.

## Item notes

### `const _`

The arms are counted by the compiler, so adding a variant to [`Combine`]
stops this crate compiling and names the variant it is missing. `ALL` is
the declaration immediately below, which is where the answer goes.

### `fn take_chunks`

`OPERATOR_REQUESTS.md` O215 ask 4 asks for the usual three multi-select
gestures on chunks. Shift-click and Ctrl-click were the click path's; this
is the band, and without it a rubber-band drawn across four lines of a note
ascended out of the block and selected the whole block instead — because
[`SelectionState::marquee`] resolves to the Object rung by construction.

# Why this is not a contradiction of that function's reasoning

Its argument is that *a region of the page contains objects*, and that
"every subpath of some other object this box happens to cover" has no
sensible reading. Both still hold. This is the different case those words
were not about: the operator has **entered** one object, and that object's
chunks are **drawn as boxes on the canvas**. The band is then a region over
a set of visible rectangles belonging to the one thing being worked on,
which is what every isolation mode — Illustrator's, Inkscape's,
PowerPoint's — does with a band drawn inside a group.

⇒ The gate is `chunks::boxed`, so the rung is offered exactly where the
boxes are. A band can never take a unit the operator cannot see.

# What a band that reaches no chunk does

With **no modifier**, nothing here claims it and the object-rung band runs:
a plain band over empty paper clears, which is the convention and is what
leaving a text block ought to feel like. With **Shift or Ctrl held** it is
claimed and changes nothing — a modifier says *refine what I have*, and a
refinement that reached nothing must not instead throw the set away.

### `fn combined`

Split out with no borrows in it so the three arms can be tested directly.
The arms are [`Combine`]'s and mean there exactly what they mean at the
Object rung — the rung changes what a hit *is*, never what a modifier
*does*.

### `fn the_three_band_modes_are_distinct`

Pinned as a table rather than three separate tests because the value of
the rule is that the three answers are *distinct*: a modifier scheme
where two of them coincide is one an operator cannot use to mean two
different things.

### `fn holding_both_modifiers_adds_rather_than_subtracts`

Adding is the non-destructive answer, and a band dragged with every
modifier at once is an operator who has not decided yet — so the
tie-break is the one that cannot lose them a selection they built.

### `fn the_direction_chooses_the_mode`

Trivial, and pinned anyway: the whole feature is one boolean choosing
between two enum variants, and a build in which both arms returned
`Enclosed` would behave exactly as this shell did before the change —
which is to say it would look like the feature had never been merged,
with every other test still green.

### `fn a_wrapper_that_is_the_whole_sheet_is_dropped`

The case the failing test surfaced. `Object(0)` wraps `Leaf(1)`, and
`Object(0)` covers the page — so a crossing band takes the leaf and not
the sheet.

### `fn a_container_worth_selecting_is_kept`

The falsifying half, and the one that stops this from being "drop every
container". A 320×220 form on a 400×300 page is a real object an
operator selects on purpose — this project has a driven check that
demands exactly that on the click path, and a crossing band must not
disagree with it.

### `fn a_page_covering_object_that_contains_nothing_is_kept`

The drawing border, and the reason the container set is derived from the
hits rather than by asking `worth_selecting` of everything. A border
covers the sheet and is not worth selecting *as a container* — but it is
not a container at all, it is a path the operator may well be reaching
for, and dropping it would be a second defect wearing the first one's
fix.

This is the assertion that would fail against the obvious simpler
implementation (`hits.retain(worth_selecting)`), which is why it is
here.

### `fn the_surviving_order_is_the_order_it_arrived_in`

The selection's paint order is what the ladder and the Objects panel
both read, and a filter that reordered would change which object a
subsequent double-click descends into.

### `fn a_plain_chunk_band_replaces_what_was_held`

The arm a driven check cannot see on its own: a build that added instead
of replacing still ends up with the band's chunks selected, so the drag
that follows still moves several and the check still passes — while an
operator banding a second group of lines silently keeps the first.

### `fn a_shift_chunk_band_adds_without_duplicating`

The overlap is the point: a band dragged across lines the operator
already had must not select them twice, because every Part-rung verb
loops the entries and a duplicate would move one line the distance twice.

### `fn subtracting_everything_leaves_nothing`

The same answer Ctrl-clicking the last held chunk gives, and the
alternative — silently keeping one, or falling back to the whole block —
would be the program overruling an explicit gesture.

### `fn every_combine_label_is_its_own_word`

A check reads `combine=` by equality, so two arms spelled the same would
collapse *the band added* and *the band replaced* into one answer — the
pair the tests above exist to tell apart.

### `fn mode_for`

One function rather than an `if` at the call site, so the convention is
stated once. A second spelling of it somewhere else is how a band that
*paints* as a crossing window comes to *select* as a window.

### `fn without_page_wrappers`

# Why this exists, and it was measured rather than anticipated

The first cut of the direction-sensitive band failed
`a_marquee_encloses_objects_inside_a_form` with `[Object(0), Leaf(1)]` where
only the leaf was wanted. That test's fixture is a page-sized form XObject
with squares inside it — the shape a CAD exporter produces, and the shape
`ncored-benchmark-cad-drawing.pdf` has.

A crossing band **touches** a page-sized wrapper wherever it is drawn. So
without this, every right-to-left drag on a wrapped drawing would silently
include the whole sheet in the selection, and the operator's next gesture —
a move, a delete, a cut — would act on all of it.

Under `Enclosed` this could not happen and that is exactly why it is new:
a band that *surrounds* a page-sized form has to surround the page, which
cannot be drawn. Touching one is unavoidable.

# The rule is the shell's existing one

`CanvasTargetProvider::container_is_worth_selecting` already answers *"is
this container really just the sheet?"* — measured against the page extent at
`COVERS_EVERYTHING`, with its own argument about why the threshold is
generous — and `canvas::smart` already applies it to the **click** ladder.
Reusing it here is the consistency argument the provider's own
`hit_test_rect` makes at length: two gestures that both mean *"select this"*
must not disagree about what is selectable.

Nothing new is measured here and no second threshold exists.

# Only a hit that CONTAINS another hit is tested

A lone path covering the whole sheet — a drawing border, which is on almost
every sheet this program is for — is **not** a container and must stay
selectable. Asking the container question about it would drop it, which
would be a second defect wearing the first one's fix.

So the container set is derived from the hits themselves: an id is a wrapper
only if some *other* hit in the same band reports it as its containing form.
Nothing is asked of the provider that the click path does not already ask.

# Parameters, and why they are closures

`container_of` and `worth_selecting` are the two provider queries, passed as
functions so that this rule is testable without a provider, a page or a
decomposition. The rule is the thing worth pinning; the queries are already
under test where they live.

### `enum Combine`

`targets` is `None` when the page has no decomposition, in which case the
band selects nothing. That is not an error and must not clear the selection
by a different route than a genuine empty band does: `SelectionState::marquee`
with an empty slice is the one path, and it is reached the same way either
way.
**What a band does to the selection it lands on** — `OPERATOR_REQUESTS.md`
O104.

A band subtracts as well as adds, because of the operator's report *"I can't
unselect things once I have selected them"*. On a CAD sheet with hundreds of
overlapping strokes, taking one object back out by clicking it precisely is
often not practical; a band is how the work is actually done.

### `const ALL`

The one authoritative list. Anything that has to visit each combine
walks this rather than writing its own copy, because a private copy
cannot go red when the set grows and the guard above only watches this
one.

### `fn label`

A word rather than the derived `Debug` form: a check reads this field by
equality, and a `Debug` spelling is a rename away from breaking one
silently.

### `fn mode`

Shift wins when both are held, because adding is the non-destructive answer
and a band held with every modifier at once is an operator who has not
decided yet.

### `struct Band`

The three travel together through every arm of the release and are decided
in one place, by the gesture. Bundling them is what keeps a rung's entry
point from growing an argument list nobody can read a call site of, and it
is also the seam that makes the two rungs' bands provably the same gesture:
the chunk arm and the object arm are handed the same value, so neither can
come to disagree about direction or about which modifier means what.

### `fn on_release`

**THE DIRECTION DECIDES WHAT THE BAND TAKES** (O88): left to right
encloses, right to left touches — AutoCAD's window / crossing-window rule.
This module's header carries the operator's report, why the fix is geometric
rather than about hit tests, and the page-wrapper hazard a crossing band
introduces.

And [`Combine`] decides what it does to what was already selected. The
two are independent: *what the band reaches* and *what it then does with
it*, which is why they are separate arguments rather than one flag.
