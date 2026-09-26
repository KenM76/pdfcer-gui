# `egui-shell/dock/mod`

## Item notes

### `fn exactly_one_body_per_stack_is_drawn_and_it_is_the_active_one`

Failure mode #3's design rule — *size a container to its active
child* — stated as behaviour rather than as arithmetic. Four
stacks hold five panels; four bodies are constructed.

### `fn a_collapsed_side_draws_no_panels_and_leaves_a_rail`

The operator's ask — *"add the little tabs that allow the left and
right panels to be minimized."* — is an affordance in both directions,
and the way back is the half that is easy to omit. A side that drew
nothing could only be restored from a ribbon command the operator had
to know existed, so a panel collapsed by accident would be a panel
**lost** rather than minimised. Every program in the class leaves a
rail: VS Code's activity bar, Visual Studio's auto-hide tabs,
Photoshop's collapsed dock strip.

So a collapsed side **is** listed in `sides_drawn` — a rail is on
screen and the report is the honest answer to *"what is on screen"* —
and the two assertions that carry the rule are that **no panel body is
constructed, and nothing on that side counts as on-screen.** A
collapsed side costs nothing but its rail.

### `fn an_empty_side_leaves_no_rail_because_there_is_nothing_to_bring_back`

A collapsed side has panels waiting behind it, so a rail is a promise it
can keep. An empty side has nothing to bring back, and a control that
opened an empty compartment would be an affordance for something that
cannot happen — the no-placeholders rule, which this crate holds to as
strictly as its host does.

### `fn the_layout_survives_a_round_trip_through_a_narrow_window`

The observed defect is that un-maximising and re-maximising loses
the panel proportions. Here the whole `DockState` is compared for
equality before and after three frames at three window sizes — so
this catches a write-back anywhere in the module, not only in the
span arithmetic.

### `fn a_column_drag_leaves_the_other_columns_shares_alone`

[`plan::drag_boundary`]'s own test proves the slice arithmetic;
this proves nothing between the intent and the model
renormalises the others on the way past — which is exactly how
coupled splitters happen, and is invisible to a reading of either
half alone.

### `fn a_panel_body_inherits_a_visible_scrollbar_style`

`D:/dev/rag/egui/scrollstyle_solid_draws_the_handle_in_bg_fill_...md`
records two independent reasons a working `ScrollArea` shows no
scrollbar — the default is `floating()` (transparent when the
pointer is elsewhere), and `solid()` alone draws the handle in
`widgets.inactive.bg_fill`, which on a light panel is near-white on
near-white. Either alone hides the bar, so fixing one of them looks
exactly like no fix at all. Both are settled here, once, for every
panel body, and asserted so they stay settled.

### `fn new`

Usable as-is: with no registry every tab is labelled with its own
id, which is ugly and truthful. See `ctx::Ctx::describe` on why the
fallback is a fallback rather than a skip.

### `fn with_id_salt`

Only needed by an application hosting two independent docks — a
document window and a preview window, say. Without it every
interactive element in both would share ids, and `egui` would
treat a click on one as a click on the other.

### `fn show`

# The order of the three phases, and why it is that order

1. **Snapshot.** The layout is cloned. Everything drawn this frame
   is drawn from the snapshot, so one frame shows one truth.
2. **Draw**, recording `Intent`s. No `&mut` to the layout exists
   anywhere in this phase, which is what makes it structurally
   impossible for a resize pass to write a *computed* span back
   into a stored share — failure mode #6, closed by construction
   rather than by care. See `ctx`'s header for the full argument.
3. **Apply.** The intents are applied, in order, in one place.

The cost is one frame of latency on a splitter drag, during a
gesture `egui` is already repainting continuously for.
