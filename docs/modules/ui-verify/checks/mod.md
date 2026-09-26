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
