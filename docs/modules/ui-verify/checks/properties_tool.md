# `ui-verify/checks/properties_tool`

`the_armed_tools_settings_are_in_properties` — every control the Tool panel
held is on screen in its new home.

# What this is for — `OPERATOR_REQUESTS.md` **O123**, part 2

> *"I never understood why there is a tool dock when everything can be in
> object and properties."*

There is no Tool panel. Its live controls — the text pen's face, size and
colour, the circular measure's pick list, and the three resize switches —
live in `crate::panels::properties::tool`. **Moving a control between
modules is the one thing in O123 that can silently cost a capability**: the
mapping keeps compiling, the unit tests keep passing, and the row simply
never draws.

# Why a unit test cannot close this, and this file can

`panels::properties::tool::block_for` is a pure function and it IS unit
tested: `each_moved_control_has_a_tool_that_reaches_it` asserts the shipped
mapping from an armed tool to its block. That is worth having and it is not
enough, and the reason is written into the module those controls came from:

> *"an option row added there is dead code that compiles, reads correctly,
> and draws nothing … Every unit test in the chain passed. Nothing tested
> that the control is on screen."*

That was `the_line_weight_switch_reaches_the_resize` catching `scale_switches`
written into a branch `CanvasTool::Select` cannot reach. The branch is
different now; the failure mode is identical, and a **move between modules
is exactly when it recurs**.

# Three launches, not three arming clicks

Each block belongs to a different armed tool, so one arming cannot exercise
them all:

| armed | block | regions asserted |
|---|---|---|
| the resting tool (`Select`) | the three resize switches | `properties.tool.scale.stroke`, `.insets`, `.distort` |
| *Add text* (`edit.add_text`) | the text pen | `properties.tool.text_pen` |
| radius/diameter (`measure.radius_diameter`) | the pick list | `properties.tool.measure_points` |

Each is a **separate launch** with its own `PDFCER_DIAG_INVOKE`, rather
than one session that arms three tools in turn. Two reasons, and the second
is the load-bearing one:

1. The commands live on three different ribbon tabs, so arming them in one
   session means driving the tab strip — which is a different check's
   subject and a way for this one to fail for a reason it is not about.
2. **The dock layout persists.** A session that raised the Properties tab
   leaves it raised, so a later case in the same process would be testing a
   state the earlier case created. `scale_switch.rs` records what that costs:
   an order-dependence that only became reliably wrong once `LayoutStore`
   started flushing on exit. One launch per case has no shared state to
   normalise.

Every region asserted is published through `crate::diag::ui_rect_visible`
against the panel's clip rectangle — which matters here more than anywhere,
because Properties is one `ScrollArea` and a control below its fold has a
perfectly healthy rectangle. `geometry_fields` is the recorded incident:
*"`Apply` was not [on screen]. The typed-geometry feature was complete,
wired, tested and unusable."*

# What it deliberately does NOT assert

That the controls **work**. `the_line_weight_switch_reaches_the_resize`
already drives the stroke switch through to a resized annotation, and
`measure_circular_points` already presses a pick row and watches the point
leave the set, and both read these regions. This check answers the one
question neither of them asks about all three at once:
*are they there at all, after the move?*

## Item notes

### `fn raise_properties`

A dock tab header, never a ribbon toggle: a toggle would *unmount* a panel
that is already mounted, and this check would then report absent controls
about a panel it closed itself. Every mode's default arrangement mounts
Properties, so an absence here means the operator's persisted layout removed
it — a dock question, not this check's subject.
