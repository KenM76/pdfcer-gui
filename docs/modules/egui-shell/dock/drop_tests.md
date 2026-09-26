# `egui-shell/dock/drop_tests`

## Item notes

### `fn closing_the_active_tab_selects_the_one_before_it_not_the_one_after`

**The closed tab is in the middle, and that is the whole test.** Closing
the *last* tab selects its predecessor whether the rule exists or not,
because [`super::model::DockLayout::normalize`] clamps a stale index to the
end of the list and the end of the list is the predecessor. Only a tab with
something after it separates "the previous one" from "whatever slid into
this index" — closing `bookmarks` selects `pages` under the rule and
`layers` without it.
