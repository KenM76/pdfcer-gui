# `pdfcer-gui/app/actions/forms/tests`

Tests for [`super`] — the form-field authoring, naming and lifecycle verbs.

# Why these live in their own file

**R2**, and the seam is real rather than a convenience. `forms.rs` reached
1,614 lines on 2026-09-08 when the action-target disclosures gained their
wiring tests, and its three `#[cfg(test)]` modules were 200 of them — text
that *describes* the verbs rather than performing them.
`app::actions::textstyle::tests` is the established precedent in this crate
for the same cut.

# Two mistakes were made moving them here, and both are worth keeping

**1. Three tests were appended inside a function and ran zero times.**
The `disclosure_store` tests below were first added to the END of `forms.rs`
by a script inserting before the file's final `}` — a brace that belonged to
`fn move_widget`. All three landed as **nested functions inside another
function**, where `#[test]` is not collected. They compiled. They produced
no failure. The only signal was three `function is never used` warnings in a
build that reported success.

⇒ The tell was the count: `cargo test` reported the same total before and
after adding them. **A new test that does not raise the total did not run**,
and checking that costs one number. It is the cheapest instrument in this
project and it was nearly skipped because the file compiled.

**2. The first split swallowed `fn move_widget` entirely**, because it found
each module's end by counting `{` and `}` per line. `{:?}` in a format
string is an opening brace to a counter and is not one to a compiler, so the
depth never returned to zero where the module actually ended.

⇒ **Cut Rust by column-0 anchors, never by brace arithmetic.** A top-level
`}` is unambiguous; a counted one is a guess that reads like a measurement.
The compiler caught this one immediately — `cannot find function
move_widget` — which is the good case. The bad case is a cut that still
compiles.
