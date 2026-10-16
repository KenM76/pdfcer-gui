# `checks::page_previews_default`

`a_fresh_profile_draws_page_previews_with_no_limit` drives O225: with no
preference written, the Pages panel's preview time limit is `0`, the
operator's word for *no limit*.

1. The profile's preferences are reset to the header alone. The check
   errors (SKIP) if the reset file still names `page_preview_budget_ms`,
   because a planted value would then be measured instead of the default.
2. `four-pages.pdf` is launched off-screen with `mode.review`, where the
   Pages panel docks by default. The check waits for the panel's
   `pages-panel` trace line.
3. That line must carry `budget_ms=0`.

`the_page_preview_limit_is_remembered_and_zero_means_never` cannot assert
this. It plants 2,000 ms first so that a zero it keeps can be told from a zero
it never set.

Falsified by setting `PAGE_BUDGET_DEFAULT` to 2,000 ms: the line read
`budget_ms=2000` and the check failed.
