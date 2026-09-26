# `egui-shell/manifest/merge`

## Item notes

### `fn prune_absent_capabilities`

An application may gate a feature behind a build-time flag, so a command
its built-in manifest names can legitimately be unregistered. Expressing
that in the manifest rather than with a `#[cfg]` in a ribbon is what keeps
the ribbon layout one document instead of one per build configuration.

# Why this runs over the whole merged shell, built-in layer included

It is the one deliberate exception to this module's standing rule that the
built-in layer is not filtered. That rule's reason is unchanged: an unknown
command compiled into the binary is a programming error, and quietly
repairing it at start-up hides the bug on every machine that runs it. But
`SHELL_FRAMEWORK.md` §5b names a **second, legitimate** case — the built-in
manifest names an optional command and that capability is compiled out.
Treating it under the strict rule leaves only two outcomes, both wrong: a
validation failure that blocks start-up, or a live control with no handler.

The resolution is not to weaken the strict rule but to make the two cases
distinguishable, which is what
[`Item::Command::capability`](super::Item::Command::capability) does. A
mandatory item is still untouched here and still reaches
[`super::Shell::validate_against`] to fail loudly.

# What it reports, and why every skip says `BuiltIn`

Anything this pass finds came from the built-in layer **by construction**:
an overlay's items were already filtered by [`filter_group_items`] and
[`filter_items`] as that layer was applied, and those use the same
[`absence_reason`], so a conditional overlay item naming an absent command
was dropped then — with its own layer on the skip. What survives to here is
what `built_in` supplied.

# Empty groups and empty tabs are left where they are

§5b's *"a group left with no items does not render"* is a **rendering**
rule, already honoured by the renderer. Deleting the group here would also
destroy the operator's saved customization of it, which would come back
blank the day they installed a build that has the capability again. An empty
group is invisible; a deleted one is forgotten.

### `fn filter_items`

The item twin of [`filter_ids`]. Kept separate rather than generalised
because the two carry different payloads — a bare id and a whole `Item` —
and a generic over "things that might contain a command id" would cost
more machinery than the duplication it removes.

### `fn absence_reason`

The one place the two absence reasons are told apart, so they cannot drift:
every filter in this module routes through it, and adding a third caller
means adding a call rather than re-deriving the distinction.

`None` for anything that stays — a separator, a custom item, and a command
the registry holds.

| the item | the command | reason |
|---|---|---|
| mandatory (no `capability`) | registered | `None` — it stays |
| mandatory | **not** registered | [`SkipReason::UnknownCommand`] — someone made a mistake |
| conditional (`capability: Some`) | registered | `None` — it stays; the build has the capability |
| conditional | **not** registered | [`SkipReason::CapabilityAbsent`] — this build is smaller, on purpose |

Note the third row: a conditional item whose command **is** registered is
indistinguishable at render time from a mandatory one, and must be. The
field says how to read the item's absence, never how to draw its presence.

### `fn merge_keymap`

An **empty command id unbinds** the chord. That is the only way a
later layer can express "remove this binding" in a per-key merge, and
without it an operator could rebind every chord but never free one —
which matters because a chord an application binds by default may be
one the operator's screen reader or window manager wants.

### `fn prune_mode_tabs`

Runs last, once, for the reason recorded on `mode_source` in [`merge`]:
a mode may legitimately name a tab that a *later* layer introduces, so
checking as each layer is applied would reject correct files.

A stale reference here is disclosed and dropped rather than rejected —
consistent with every other merge failure, and specifically so that an
operator's mode survives the removal of a tab minus one entry rather
than failing to load. [`Shell::validate`] would reject what survives
only if this had left something unresolvable, which by construction it
cannot.

### `fn a_layer_overrides_per_item_and_leaves_everything_else_alone`

This is the contract in one test. An operator layer that renames
one tab must not delete the two it did not mention, must not empty
the groups it did not mention on the tab it *did*, and must not
drop the question or the keymap it said nothing about.

The failure this guards against is not hypothetical: a
`tabs: Vec<Tab>` with no `Option` makes "I did not mention it"
indistinguishable from "I want it gone", and every field on the
tab has the same problem one level down.

### `fn a_bare_tab_reference_reorders_without_changing_anything`

The documented idiom from this module's header. If this ever
stopped working, the alternative would be a `tab_order` field —
a second place tab identity is written down and a second place for
it to go stale.

### `fn an_unknown_command_loses_one_item_and_is_disclosed`

`SHELL_FRAMEWORK.md` §4: *"A customization referencing a command
that no longer exists loses that one item and says so in the status
surface; it does not discard the layout."*

Both halves are asserted, and the second is the one that would
otherwise rot: it is easy to write a filter that drops the group,
and the symptom — an operator's Window group quietly emptying after
an update — is attributed to the update, not to the filter.

### `fn three_layers_apply_in_order_and_each_overrides_per_item`

The middle layer's contribution must survive the outer one saying
nothing about it — which is what makes an application override
worth shipping at all.

### `fn a_layer_from_a_newer_schema_is_skipped_whole`

Applying the fields a build happens to recognise, from a document
written against a schema it does not, produces a shell nobody
wrote. The fail-soft answer is to fall back to the layer below and
say so.

### `fn mode_tab_references_are_pruned_once_at_the_end`

The second half is why pruning happens once at the end rather than
as each layer is applied: an operator may legitimately define a
mode that names a tab their own layer creates further down the
file, or that the application override introduced.

### `fn the_built_in_layer_is_never_filtered`

It is compiled in and is the reset target. A stale id there is a
programming error that must surface as a validation failure in the
application's own tests, not be quietly repaired at start-up on
every machine that runs it.

### `fn a_built_in_item_provided_by_an_absent_capability_is_dropped_by_name`

The case §5b is written for: the manifest compiled into the binary
names a command the build did not register, because the build was
compiled without that capability. The alternatives are a hard
validation failure at start-up and a live control with no handler.

### `fn a_mandatory_built_in_item_is_still_left_for_validation_to_reject`

The other half, and the one a careless implementation loses. The pass
that drops conditional items runs over the same lists, and reusing the
ordinary item filter there would silently repair a programming error in
the built-in manifest — the exact behaviour this module's header says
must never happen, because it hides the bug on every machine that runs
it.

So the item survives the merge, and `validate_against` is left to fail
on it, which is asserted here rather than assumed.

### `fn a_conditional_item_whose_capability_is_present_is_kept_untouched`

The negative control, and it is not a formality: an implementation
that dropped every conditional item — or drew one differently — would
pass the first test above and ship a build in which the capability is
compiled in and invisible. The field says how to read an item's
*absence*; it must say nothing about its presence.

### `fn an_operator_item_provided_by_an_absent_capability_reports_the_capability`

The two paths are different code — an overlay is filtered as it is
applied, the built-in layer in a final pass — and they share
[`absence_reason`] so they cannot disagree about which of the two
reasons applies. This is what pins that sharing: an operator who put
`file.sign` on their own QAT-adjacent group in a build without signing
must be told *"this build does not include that"*, not *"you made a
mistake"*, because they did not.

### `struct MergeInput`

A struct rather than three positional arguments because two of them
are `Option<&Shell>` of the same type, and a call site that swapped
them would compile and would apply the operator's customization before
the application's override — producing a shell that is wrong in a way
no test of either file could find.

### `fn merge`

Never fails. Anything it cannot carry across becomes a [`Skip`] in
[`Merged::report`]; see this module's header for why that is the right
posture for inputs that come from outside the build.

The `catalog` is what makes a stale command id detectable. Pass
[`super::AnyCommand`] only in tooling that has no registry — in an
application it would disable the check that turns a stale reference
into a disclosed skip instead of a control that does nothing.
