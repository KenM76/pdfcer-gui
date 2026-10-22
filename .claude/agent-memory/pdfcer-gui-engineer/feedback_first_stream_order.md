---
name: first-stream-is-not-the-page
description: A test that takes "the first decodable stream" of a written PDF can pick an object stream; it passed alone and failed in the workspace run
metadata:
  type: feedback
---

A unit test that reads back a written PDF must pick the stream by what it
holds, never by position. `drawing_instructions_are_tidied_only_when_asked`
took `objects().find_map(decode)` and passed when run by name, then failed in
the full `cargo test --workspace` because the first decodable stream was the
object stream (`1 0 2 31 3 69 <</Type /Catalog…`).

**Why:** the optimizer may pack objects into an object stream and object
order is not part of any contract, so a positional pick is a coin toss that
a single filtered run can win.

**How to apply:** filter by an operator TOKEN (`b" re"`) — the first fix matched bare `re`, which `/Parent` in that same object stream contains, and failed the next workspace run the same way — or walk
`/Contents` from the page. A green filtered run of a new test is not its
verdict — the unfiltered workspace run is ([[feedback_filtered_test_run]]).
