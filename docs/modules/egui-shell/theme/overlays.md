# `egui-shell/theme/overlays`

## Item notes

### `fn merged_roles_are_refused_and_both_are_named`

The message is the deliverable: "two roles collided" sends the
reader back to work out which two, on a preset with a dozen of
them.

### `fn an_undefined_role_fails_rather_than_passing_vacuously`

This is the difference between a gate and a decoration. If unknown
roles were treated as trivially distinct, a check whose role names
were both misspelled — or whose roles were renamed in the palette
and not in the test — would go green forever while measuring
nothing. The salvage source's whole family of "green is not
evidence" lessons is this same failure in other clothes.

### `struct Overlays`

Ordered (`BTreeMap`) rather than hashed, so iteration is deterministic
and a failure message lists roles in the same order on every machine.
A diagnostic that reorders itself between runs is one nobody can diff.

### `fn set`

Silently replacing is deliberate: a preset is normally built by
spreading a base set and overriding a few entries, exactly as
`Theme::dark` does with `..quiet.palette`, and making the
override an error would forbid the idiom that makes presets
readable. The risk that idiom carries — two roles quietly becoming
one — is what [`Self::assert_distinct`] is for.

### `fn get`

`Option` rather than a fallback colour. A missing role is a
programming error — a typo, or a role the preset forgot — and
returning magenta or transparent would make it a *rendering*
question the reader has to notice, on the frame where it happens,
on the preset where it happens.

### `fn assert_distinct`

This is the generic form of the salvage source's
`distinct_overlay_roles_stay_distinct_in_every_preset`. The
application supplies the list, because only the application knows
which of its roles answer different questions — two roles that
happen to share a colour because they are the *same* semantic in
two places are fine and must not be forced apart.

# Errors

[`RoleCollision::Merged`] when two of the named roles resolve to
the same colour, naming both. [`RoleCollision::Undefined`] when a
named role is not defined at all — because a test that passes
because both of its role names were misspelled is worse than no
test, and "unknown roles are trivially distinct" is exactly how
that happens.

Reports the first collision rather than all of them: unlike the
contrast gate, one merged pair is almost always one edit, and the
pairwise product of a large role set makes an exhaustive report
noisier than it is useful.

### `fn install`

Wrapped in an `Arc` on the way in, so [`Self::of`] is a refcount
bump rather than a map clone. This is called once per frame beside
[`super::Theme::apply`] and read by every painter, so the
asymmetry is the right way round.

### `fn of`

An empty set rather than a panic: a shell that aborts because an
application has not published overlays would be making an optional
extension point mandatory. Every [`Self::get`] then returns `None`,
which is the same signal a missing role gives.
