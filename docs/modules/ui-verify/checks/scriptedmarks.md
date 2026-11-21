# `checks::scriptedmarks`

The marking, picking and Comments-census helpers for checks that drive the scripted pointer. They are split out of `checks::scripteddrive`, which re-exports them, so callers name them as `scripteddrive::…`. Their design is documented with that module: [scripteddrive.md](scripteddrive.md).

- `mark_through_the_menu`: right-click a point, press the object menu's redact row, and read back the `quads=` and `bbox=` it requested.
- `resolve_pick`: the scripted twin of the OS-driven pick resolver.
- `census_refresh` / `census_baseline`: read the Comments panel's listed count, and refuse a fixture that is not clean.
