# `pdfcer-gui-base/layeroverride`

## Item notes

### `struct LayerOverride`

# `None` is not "hide nothing"

`pdfcer_render::LayerVisibility` **replaces** the document's own default
configuration rather than merging with it (core API trap T-12.9). So:

| state | meaning |
|---|---|
| `hidden: None` | obey the document's `/D` configuration (§8.11.4.3) |
| `hidden: Some({})` | show **every** layer, including ones the document turns off |
| `hidden: Some({…})` | exactly these are hidden |

Collapsing the first two would silently reveal every layer a document had
turned off, which on a drawing with a "Confidential" watermark layer is a
disclosure defect rather than a cosmetic one.

That is also why a set is stored rather than operator *deltas*: the
renderer wants the complete answer, so the complete answer is what is
held. A delta would have to be resolved against the document's defaults at
render time, in a second place, with the merge rules the engine
deliberately refused to define.

# The operator's toggle is session-only, and nothing here can save it

§8.11.2.1 puts the live state outside the document entirely: the toggle is
*"session-only state, held nowhere the save path can see it"*, lost on
reopen. That is a property of the format rather than a gap in this build,
it is what `crate::text::panels::layers_session_only_note` discloses, and
it is why changing it must not bump [`OpenDoc::edit_epoch`].
