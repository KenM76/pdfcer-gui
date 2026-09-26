# `ui-verify/checks/mod`

The named checks — **the index of which checks exist.**

One `pub mod` per check, each carrying the argument for why that check is
here: what it drives, what its oracle is, and what a build with the wiring
absent would look like. The three files beside it hold the other subjects —
`roster.rs` the run order ([`all`]), `harness.rs` the contract ([`Check`],
[`CheckContext`]), and [`conventions`] the rules a new check is held to.

The principle every check here satisfies, and the one to hold a proposed
check to: **it must fail against a build where the wiring is absent, and
the wiring must be something no unit test in the workspace can observe.**
Every check here has been run against such a build and seen to fail; that
is what stage S1's acceptance criterion asks for, and it is not optional.

The four the suite started as are the defects that shipped past a green
test suite, turned into the tests that would have caught them:

| Check | Defect | Oracle |
|---|---|---|
| [`delete_key`] | **D1** — Delete stops working after the first canvas click | the trace |
| [`settings_headings`] | **D2** — section headings near-white on light grey | the pixels |
| [`ribbon_captions`] | group captions rendering illegibly, or not at all | the pixels |
| [`ribbon_mockup`] | the band drawn to different proportions from the mockup, and a resting control drawn in a box | the pixels |

A check may also assert that a control is correctly **DISABLED**, reading
the *absence* of `ribbon-command-invoked` as its evidence — admissible only
where the same control is then shown to invoke, in the same run, once its
operand exists. [`text_markup`] is the worked example.

Not every module here is a check: [`driving`] and [`comments_census`] hold
moves their callers share, and each header says why — including why
[`markup_rectangle`] deliberately keeps its own copies.

## Item notes

### `mod harness`

Its own file under **R2**, and the seam is argued in its header: this module
is the *index* of which checks exist, which grows with every landing, and
that one is the contract, which does not.

### `mod roster`

Its own file under **R2**, and the seam is argued in its header: this
module is the *index* of which checks exist, and that one is the *list*,
which grows again every time one is re-ordered.

### `mod annot_rotate`

The sibling of [`rotate`], which drives the same handle over page content.
It is a separate check rather than a parameter on that one because the two
exercise different verbs behind an identical gesture — `rotate_annotation`
here, `transform_objects` there — and the whole risk this check exists for
is a build that reaches the *other* one. It asserts which trace line
appeared **and** that the other's did not.

It also asserts the affordance BEFORE pressing, through the
`canvas.rotate-handle` region, which [`rotate`] could not: a build with no
ninth handle and a build with a mis-routed one produce the same silence
otherwise.

### `mod export_remembered`

Two launches: a CONTROL over a reset preferences file, to MEASURE this
build's shipped export defaults, then a seeded one. Any seeded value
that turns out to equal its measured default is reported as a SKIP
naming it, because that field would read back correctly whether the
file was consulted or ignored.

### `mod field_delete_gate`

[`annot_delete_gate`]'s fix closed one surface of three and left the
form-field door a no-op by construction — the condition's guard was false
whenever a field was selected, the `canvas.field` menu carried no
`visible_when` at all, and the Delete key's field rung asked nothing. This
aims at the merged signature widget in the SAME fixture pair that check
steers away from, and asserts the branch it avoids.

### `mod form_groups`

The second is the shape a unit test cannot see: the correct behaviour is an
**absence**, and a test asserting "the query returns Some" passes on a build
where nothing calls the query. That is the state the audit found, under
2,538 passing tests. Its header carries both fixtures' arguments.

### `mod formaim`

Its header carries the rule in full: authoring a field leaves it
SELECTED (`OPERATOR_REQUESTS.md` O53) and `canvas::forms::select_click`
traces only on a CHANGE, so clicking the field just placed asks the program
to announce a selection that has not moved — a silence that looks exactly
like a broken hit test and is not one. Every such check clears the
selection on blank paper first, which also makes it assert both halves of
`select_click`'s own table rather than one.

### `mod markup_style`

A **third** shape of invisible wiring, and the quietest: the manifest test
asserted the item was *declared* and passed correctly, and the reachability
check could not see it at all because a `Custom` item carries no command id.

### `mod navigate_commits_text`

The operator reported typed text not showing until he clicked the page
again, which is not a rendering fault: the draft had never been committed,
and the sheet was correctly drawing a document that did not contain it. Its
oracle is the engine's own commit line, because the broken build produces
the tool change and no commit at all.

⚠ It types at the real keyboard, so it cannot share the desktop with
anyone: a driven run needs the machine to itself.

### `mod off_sheet`

Its header carries the sign flip that made the measured anchor `(1199.50,
−0.54)` look like it was on the page when it was half a point off the top of
it, why the initial aim's magnitude is deliberately not load-bearing, and
why `DESIGNS.md`'s own obligation to *"confirm a page-flip gesture does
nothing"* is wrong — that gesture is gated on a non-default preference, so it
does nothing on any build and would have been recorded as evidence about the
clamp.

### `mod os_fonts_setting`

Its header carries why the oracle is the REQUEST STREAM and not the cache's
size: a build that held a gigabyte and still re-requested would pass a size
assertion and fail the operator, and a screenshot cannot help at all,
because a re-rendered page and a remembered one are the same picture.

### `mod pagetree_guard`

Its header carries the two things a reader has to know before touching it:
the fixture is **pinned** and nested, because on a flat page tree the defect
cannot occur and this check would pass against a build carrying it in full;
and it has ⬜ **NOT BEEN RUN**.

### `mod unshare_form`

Two things no other check in this file can claim. It **presses a
context-menu row**, which is possible only because pdfcer's menus publish a
`ui_rect` per row — without one there is no coordinate to aim at and the
whole "does the row do the thing" question cannot be asked. And its subject
is a
command whose SUCCESS is invisible: the copy `unshare_form` makes is
byte-identical to the original, so a page that was unshared renders
pixel-for-pixel as one that was not, and *"nothing appeared to happen"* is
what a pass and every possible failure look like alike.

**A pair rather than one check, and the pairing is the point.** The
command's two outcomes are decided by a property of the *file* — is this
form drawn on any other page — so each case needs its own fixture, and a
single check could only ever exercise one of them. ⚠ A check named for the
shared case but pinned to a document with exactly one invocation asserts
that "every other invocation site" is byte-identical about an EMPTY set,
and passes for that reason; each fixture is pinned so that cannot recur.
It measured nothing and passed. Whenever a behaviour is selected by the
input document rather than by the gesture, the fixture IS the test, and one
fixture is half of it.

### `mod layers_search`

A new shape of the founding failure and the reason this module exists: the
adapter's own unit test asserted that all four of its calls **refused**,
which was correct while `pdfcer-print` was unlinked and became a lock
holding the defect in place the moment the manifest line landed. A green
suite defended the absence of the feature. See the module header.
Drag a page thumbnail to a new position, and see where it will land
before letting go.
**The Layers panel's search field is on screen and reachable** — O126.
Its header carries why the check refuses to pass on an absence.

### `mod clipboard_annotation`

Defect O18 shipped under 1,628 passing tests because the failure is not in
any function's return value: it is in WHICH OF TWO HANDLERS reached the OS
last. A trace cannot see that either — `text-copy source=selection` can be
emitted truthfully by a frame whose clipboard is then overwritten.
**The annotation clipboard** — a sticky note, a stamp, a text box, a
link or a file attachment carried to another drawing with its baked
appearance intact. ⬜ **NOT RUN**; its module header says so in its own
words rather than leaving an absent result to imply it.

### `mod colour_clicked_text`

⚠ The swatch it aims at sits below the fold of a Properties panel whose
first section is three always-on switches, so the check must scroll before
it can click. Its falsification table is unexercised: driven-and-green
is not the same as known-to-notice.

### `mod deep_pan`

Every piece of the region tier had unit tests before he hit this, and all of
them passed while the feature did not exist, because nothing called the
strategy. A complete unreachable mechanism is indistinguishable from a
working one from inside a test suite.

### `mod escape_commits_text`

Both halves are the assertion. The operator's case for committing is that
the mistake it can cause is the cheap one, so a build that committed and
could not undo would have swapped an unrecoverable loss for an
unrecoverable gain.

⚠ It types at the real keyboard, so it cannot share the desktop with
anyone: a driven run needs the machine to itself.

### `mod font_group_real`

The twin above pins `fixtures/paragraph.pdf` on purpose, because its subject
is a discoverability route and a route has to be asserted on a page whose
contents are known. This one honours `--pdf` and `--doc-point` and has no
fallback, because its subject is the opposite question: does the route
survive a 36-sheet export whose faces are subset, whose labels are 5 pt and
whose first page carries 5,899 paths against 4 text objects.

Read as a PAIR. Green here and red there is a fixture problem; red here
and green there is something about real drawings; both red is a regression.
That diagnosis is why there are two checks rather than one parameterised
one — `OPERATOR_REQUESTS.md` O198 is an operator reporting a feature that
was green on a fixture and unusable on his file.

### `mod max_zoom`

A double toggle makes a button inert while every unit test and every
smoke launch stays green, because all of them observe the button's
PRESENCE — which was never the broken part. This check observes the
readout the press is supposed to move.

### `mod raster_wall`

Runs immediately after [`deep_zoom`] in the roster and is deliberately NOT
folded into it. That one asserts the ACTING page still renders past the
ceiling; this one asserts nothing is ordered for the pages AROUND it. One
check red for two causes would have said much less about either.

### `mod read_mode_chrome`

Named `read_mode_chrome` rather than `read_mode` because that name is
already taken by the check one line up, and the two are about genuinely
different things: that one is `mode.read`'s **capability** gate (a click in
Read must not select), this one is `view.read_mode`'s **chrome** toggle (the
ribbon and the docks stop being drawn). `app::window` §1 carries the
argument for why those are two commands rather than a duplicate.

### `mod read_mode_exit`

The operator fell through exactly that gap — *"I didn't see a
way to get back out of read mode"* — and the answer is a statement on the
window title and on the status bar naming the chord the **keymap** holds.
This reads that statement from a trace: no pointer, no keystroke, so unlike
its neighbour it can run beside somebody working.

### `mod scripted_keys`

Its header carries why it never counts `spelled=yes` lines: the seam runs
before the collector and emitted one for a rung whose effect was overwritten
later in the same frame — and the broken run and the sound one finish at the
same zoom, so an end-state assertion passes on both. One entry in its chord
list is deliberately unspellable, which is what shows `spelled=` to vary and
doubles as the negative control for the zoom reading.

### `mod refused_character_face`

The engine shipped the capability and the shell could not reach it: the
chooser built its list from `preview_font_resources`, which enumerates the
*page's own* `/Font` resources, so the one thing the release note is about
was absent from every surface in the program. Its header carries the six
links and names the fourth as the one worth writing the check for on its own
— pdfcer embeds nothing, so the text is drawn with the READER'S copy of the
face, and a disclosure that is catalogued, unit-tested and never painted has
discharged nothing.
**A refused character offers the face that can type it** — the engine
refuses by name when a run's font carries no code for a character, and
until O141 nothing joined that refusal to the chooser sitting one panel
away that already offers faces which do carry it.

### `mod scroll_input`

The experiment that decides whether O23's pasteboard failure is a feature
problem or a defect the operator already meets. It reproduces from an
ordinary wheel scroll with no pasteboard in the build, or it does not.

### `mod select_filter`

Deliberately NOT "the popup opens", which is a unit test and is also the
one claim that stays true of an INERT control: a double toggle leaves the
button drawing, hit-testing and reporting its rect while doing nothing, and
every observation of the button's presence passes against it.

### `mod page_drag_between_documents`

Two registrations, one implementation: the unmodified drag must **copy** and
the Shift-held drag must **move**. Running only one would pass against a
build that always did the same thing, which is exactly what a modifier read
at the wrong moment produces.

### `mod bookmark_move`

Its header carries the two oracles no unit test can reach: the moved row's
**level**, which a reorder cannot produce however wrong it is, and the
**disagreement** between the panel's item count and the number of rows it
draws after a collapse — which is the whole of *"the sign is honoured"*.

### `mod typo_refusal`

The only check in the suite that takes its **negative control through the
same instrument in the same process**: it commits an edit the engine refuses
and asserts the `⊗` slot drew, then commits one that succeeds and asserts it
did **not**. Its header carries why a one-sided reading of that region is not
a verdict, and why the tempting `Identity-H` forecast is falsified by
`pdfcer-core`'s own fixture.
