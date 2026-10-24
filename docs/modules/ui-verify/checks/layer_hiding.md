# `ui-verify/checks/layer_hiding`

Three checks on `fixtures/layer-hiding.pdf` (built by
`fixtures/layer-hiding.PROVENANCE.py`): five layers, `/Order`
`[Structure [Steel Survey] Bolts Notes]`, Survey `/Locked`, the Bolts span
nested inside Steel's content span, and one square on no layer. Each check
launches its own session in Edit mode.

# Driven

| Check | Ask | Gesture | Passes when |
|---|---|---|---|
| `a_parent_switch_carries_its_sublayers` | O316 | click `panel.layers.switch.Structure` | the next `layer-row` lines read Structure and Steel `visible=false`; Survey (locked), Bolts and Notes `visible=true` |
| `a_band_takes_nothing_on_a_hidden_layer` | O317 | click the Notes square; switch Steel off; band around Steel and Bolts; switch Steel on; the same band | the first band's `canvas-selection via=pv.marquee` reads `sel=0`, the second `sel=2` |
| `objects_list_under_their_layers` | O318 | open the Objects tab | `objects-layers headings="Survey\|Notes\|Bolts\|Steel\|Structure\|-"` |

The Notes click before the first band makes the band's own selection line
differ from the one before it, so `trace_changed` de-duplication cannot hide
it. The band is a window band (left to right) started on empty paper.

Bolts' square is hidden when Steel is, through `oc_chain`; a filter reading
only the innermost `/OC` takes it (`sel=1`). The Objects headings use the
innermost layer, so Bolts' square heads under Bolts.

# Falsified

| Break | Result |
|---|---|
| `panels::layers::toggle_actions` skips the nested layers | switch check FAIL: Steel `true` |
| `objectprovider::unseen` reads only the innermost `/OC` | band check FAIL: `sel=1` with Steel hidden |
| `panels::objects::layered::groups` puts the unlayered group first | headings check FAIL: `-\|Survey\|…` |

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
