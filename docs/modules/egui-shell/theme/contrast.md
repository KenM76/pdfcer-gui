# `egui-shell/theme/contrast`

## Item notes

### `fn grounds`

Deliberately not the full cross product. A gate is only worth
what its reader believes, and a pair nobody renders is a line in a
failure list that sends somebody to look for a surface that does
not exist. So each role names the grounds it has a renderer for:

- [`Self::Body`] and [`Self::Weak`] reach [`Ground::TextEditBg`]
  because a `TextEdit` draws its content and its hint there
  (`egui-0.35.0/src/widgets/text_edit/builder.rs:463-466` and `:591`).
- Nothing draws `strong`, a hyperlink, a warning or an error
  *inside* a text field, so those four stop at the two chrome
  grounds.

### `fn fmt`

**This is diagnostic text, not operator-visible copy.** It is
written for a failing test, a CI log or a verification harness. An
application that wants to surface a theme problem to a user should
render the structured fields itself, in its own string catalogue —
the shell has no business deciding how another project words a
message to its operator.

The line keeps the ten-pair version's wording word for word for a
widget pair, because that message was good and the widening had no
licence to spend it. Two things were added, both because the
widening produced a failure the old wording served badly:

- the trailing ` — {why}` clause, for every origin — a reader who
  knows `widgets.Active.bg_fill` failed is better off still for
  being told, on the same line, that it is what tab buttons paint;
- **one decimal place on the gap.** The first real failure the
  widening found measured 89.74 against a floor of 90, and at
  `{:.0}` it printed as *"luminance gap 90, needs 90"* — a
  diagnostic that reads as a bug in the gate. The threshold keeps
  `{:.0}` because it is a round number by construction.

### `fn the_pair_matrix_covers_every_origin_it_claims_to`

Worth its own test because the widget half of this module's value
is that its coverage is defined by `egui`'s matrix rather than by a
list — and the *other* half is a list, which is precisely the part
that can silently shrink. If a state were dropped from
[`WidgetState::ALL`], or a role from [`TextRole::ALL`], the gate
would still pass everything it looked at and nothing else would
notice.

### `fn a_translucent_foreground_is_composited_not_treated_as_opaque`

The specific number matters: `rgba(250,250,250,220)` is the plate
colour from D2, and treating it as opaque is the mistake that
would make this gate wrong about its own defect.

### `fn a_translucent_selection_plate_measures_differently_on_each_ground`

This is defect T2's arithmetic in miniature and the reason
[`Origin::SelectedWidget`] carries a ground at all. A 27 %-alpha
wash is not a dimmer plate; it is a different colour over every
background it meets, so the same theme values must produce two
different gaps over two different grounds. If they did not, the
ground field would be decoration and a regression to a wash could
hide behind whichever ground happened to be measured.

### `fn check_reports_every_failing_pair_not_only_the_first`

A gate that stops at the first failure turns a theme edit into a
sequence of rebuilds. `check`'s contract says all of them; this is
what holds it to that.

### `fn a_failure_names_the_line_to_change`

The message is the deliverable. A gate that says "contrast failed"
has told the reader to go and re-derive what this function already
knew.

The widget case's wording is asserted verbatim, because the
widening had no licence to spend a message that was already good.
The only deliberate change is the gap's decimal place; see
[`ContrastFailure`]'s `Display`.
