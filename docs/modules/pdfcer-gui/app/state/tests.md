# `pdfcer-gui/app/state/tests`

## Item notes

### `fn every_view_input_that_changes_the_picture_changes_the_render_key`

The acceptance criterion for the `RenderKey` completion, from the
shell's side rather than the worker's.
[`PdfcerApp::settle_and_rasterize`] asks "is the texture still a
picture of what I am looking at?" by comparing this key, so an input
it does not carry is a control that ticks and redraws nothing.

### `fn a_layer_or_annotation_change_commits_at_once_rather_than_settling`

A click has no gesture in flight, so waiting out the 150 ms zoom
settle would be latency buying nothing. Asserted through the key's own
categories — what `settle_and_rasterize` reads — so an input that
lands in the wrong one fails here rather than being noticed later as
sluggishness.

### `fn obeying_the_document_is_not_the_same_as_hiding_nothing`

Core API trap T-12.9: [`LayerVisibility`] REPLACES the document's
default configuration rather than merging with it, so `None` and
`Some(empty)` are not two spellings of one state. Collapsing them
reveals every layer the document turned off — on a drawing whose
"Confidential" watermark is an off-by-default layer, that is a
disclosure defect, not a cosmetic one.

### `fn the_first_layer_toggle_seeds_from_the_documents_own_defaults`

[`LayerVisibility`] wants the complete hidden set, so a control that
handed in only the group the operator touched would reveal every
other layer the document had turned off. The fixture declares four
groups, two of them off by default; turning a third off must leave
those two off.

### `fn every_layer_mutation_moves_the_generation`

The generation is the staleness key; the set is not. A mutator that
changed the set and forgot the counter would leave the texture
looking current — the inert-control defect with the override
*correct*, which is the most confusing possible version of it.

### `fn hiding_annotations_or_a_layer_is_not_an_edit`

Hiding annotations or a layer changes what is drawn and nothing that
is saved, so it must not bump `edit_epoch` — which would throw away
the decomposition and the font inventory for nothing, and would make
the diagnostic `objects n=` line re-trace as though the document had
changed.

### `fn a_selection_cannot_outlive_the_document_it_was_made_on`

The property holds by construction: opening a document builds a whole new
`OpenDoc`, so its selection is `SelectionState::default()`. There is no
document-identity key to compare, and there must not be one — an `Arc`
address is not an identity, and a reused allocation with a matching page
count would carry a stale selection into a new file.

Written as a replacement **in the same binding**, the sequence an address
reuse would have needed, so reintroducing any document-identity key here is
a test failure rather than a review finding.

### `fn with_hold`

The `ShapePreview` is **empty**, and that is deliberate: every assertion
below is about the liveness *decision*, and a preview carrying real geometry
would make the tests depend on a fixture decomposing — a second reason to
fail, in tests about a rule that has nothing to do with geometry.

### `fn a_committed_edit_whose_raster_has_not_landed_keeps_its_preview`

This is the whole feature. Without it the operator watches the object snap
back to where it started and then jump forward when the raster lands, one to
two seconds later on a dense drawing.

### `fn the_preview_goes_the_moment_the_page_catches_up`

A preview left up over a correct raster would be drawing a
selection-coloured tracing over the real thing — the operator's own
complaint about the old GUI's marking, arriving by a new route.

### `fn an_edit_the_epoch_never_moved_for_stops_drawing_almost_at_once`

# The failure this pins

Actions are drained after the frame that raised them, so there is a real
window in which a hold is legitimate and `edit_epoch` has not moved. There is
also a state where it **never** moves: the engine refused. By epoch alone the
two are identical.

Without the time bound, a refusal would leave a preview of a move that did
not happen sitting over a document that disagrees with it — for the full four
seconds of the backstop. That is a picture of a lie rather than a picture
that is late, and it is the worst outcome this feature can produce.

### `fn the_backstop_drops_a_preview_no_raster_ever_arrived_for`

It exists because *"the raster will arrive"* is an assumption, and a stuck
preview is indistinguishable from a corrupted document. Four seconds is
roughly four times the measured whole-page raster on the operator's hardest
drawing, so it cannot fire on a render that is merely slow.

### `fn the_catching_up_line_waits_before_it_speaks`

# Why the silent half is the one worth pinning

The picture is behind after **every** edit, for a few milliseconds on a
simple page. A line that appeared each time would flash on and off on every
keystroke, and a status bar that flickers is one the operator stops reading —
which costs every *other* sentence the bar carries. Losing that bound is a
larger regression than losing the feature.

### `fn the_catching_up_line_stops_when_the_raster_lands`

No retirement rule and nothing to remember to clear: it is a STATE, unlike
every other line in that half of the bar, which are events keyed on the
epoch. A test rather than a comment because "it stops on its own" is exactly
the kind of claim that quietly stops being true.

### `fn a_page_edit_elsewhere_does_not_strand_the_catching_up_line`

The two tests above cannot see this: they set `edit_epoch` and
`page_texture_epoch` by hand to equal or adjacent values, which never makes
the two counters *diverge*, and divergence is the whole defect.
`page_texture_epoch` carries a `PageEpochs` value; `edit_epoch` is a
different counter. Let an edit land on a page the operator is not looking at
and `edit_epoch` moves while this page's entry does not — the two numbers
pass each other and **nothing ever brings them back**, so *"the picture is
catching up"* sits on the bar for the rest of the session over a correct
picture.

This test drives the counters through their **own issuers** rather than
assigning both fields, which is what makes it able to fail at all.

### `fn an_unedited_document_never_says_it_is_catching_up`

The guard this pins is `last_edit_at: None`. Without it, a freshly opened
document whose first raster has not landed would announce that it is catching
up — on open, before the operator has done anything at all, which is the
worst possible first sentence for a program to say about itself.
