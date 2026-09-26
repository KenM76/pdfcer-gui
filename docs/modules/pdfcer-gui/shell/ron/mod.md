# shell::ron — the built-in manifest, on disk

[`built_in_ron`] is the text of `built_in.ron`, compiled in with
`include_str!`, and [`parse_built_in`] is that text parsed back into an
`egui_shell::Shell`.

# Why a file exists at all when the Rust already builds the manifest

Because otherwise the claim at the centre of `SHELL_FRAMEWORK.md` would
be untested. The claim is:

> **The shell is data.** Tabs, groups, commands, panels, layouts, modes
> and key bindings are a serializable document that the application
> *supplies* and the operator *edits* — not code that has to be
> recompiled to change.

A manifest that exists only as a `Shell` value built by a Rust function
satisfies the *types* and none of the point. The operator's
customization layer is a file; the application-override layer is a
file; a saved workspace is a file. If the format cannot express the
real ribbon — all eight tabs, thirty-seven groups, three modes, the QAT
and forty key bindings — then "the shell is data" is a description
of a data structure rather than of a product, and nobody finds out
until an operator opens `userdata/shell.ron` and it does not round
trip.

So the built-in manifest is emitted as RON, checked in, and
[`the_ron_file_and_the_rust_agree`] asserts the two are the same shell.
That test is the proof, and it is deliberately an equality of *parsed
values* rather than of text: comparing strings would fail on
whitespace and would say nothing about whether the document means the
same thing.

# `IMPLICIT_SOME`, and why the round trip alone would not have caught
the defect

`egui-shell` reads and writes with RON's `IMPLICIT_SOME` extension
enabled on the [`ron::Options`] used for **both** directions. Without
it, every `Option` field — which is nearly every field in the manifest,
because `None` is what "this layer does not mention this" means — has
to be written `tabs: Some([…])`, and the obvious spelling

```ron
Shell(tabs: [ Tab(id: "tools") ])
```

fails to parse with `ExpectedOption` — a message naming a Rust type the
operator has never heard of, at a position, with no hint that the fix
is four characters.

**The trap is that a round-trip test cannot see this.** `to_string` →
`from_str` passes either way, because the serializer emits `Some(…)`
and the deserializer accepts it. The writer and the reader agree by
construction; the population that breaks is the one that never goes
through the writer — a file hand-authored from scratch, a snippet
pasted out of documentation, a customization one operator shared with
another.

Hence [`a_hand_written_snippet_parses`], whose input is a string
literal written by a person and never produced by any serializer. That
is the only test in this module that is about the *format* rather than
about pdfcer's manifest, and it is the one that would have failed.

# Regenerating the file

`built_in.ron` is generated, not hand-maintained. When the manifest
changes, run the ignored test that rewrites it:

```text
cargo test -p pdfcer-gui rewrite_built_in_ron -- --ignored
```

and commit the result. Ignored rather than automatic because a test
that writes to the source tree as a side effect of `cargo test` makes
every run a potential working-copy change, and because the diff is the
most reviewable artefact this module produces: it is the ribbon,
stated as data, in a form that shows up in a pull request.

## Item notes

### `fn the_ron_file_and_the_rust_agree`

The proof that the format is genuinely the manifest rather than a
Rust-only fiction with a file beside it. Equality of parsed values,
not of text — see the module header.

When this fails it is almost always because the Rust changed and
the file was not regenerated, so the message says how.

### `fn the_ron_file_is_a_complete_manifest`

Distinct from the equality test above, and not implied by it. A
layer is not required to validate; the built-in layer is, because
it is what every other layer patches and what a reset restores. If
this file could only be understood as a diff against something
else, it would not be the built-in layer.

### `fn a_hand_written_snippet_parses`

Deliberately **not** produced by the serializer. Every string here
was typed: no `Some(…)` wrappers, a comment, a trailing comma, and
the shape the documentation shows. This is the input class an
operator-editable format exists to accept, and the one a round-trip
test structurally cannot cover.

It is written as a *customization layer* rather than a whole
manifest, because that is what an operator actually writes: a small
file that patches the built-in one per item. It therefore also
checks that an incomplete layer parses without validating — which
is the property the whole three-layer design rests on.

### `fn the_generated_file_carries_no_option_wrappers`

This is the observable consequence of `IMPLICIT_SOME` on the
*writer*, and it is the property that makes the generated file a
usable template: an operator who copies three lines out of it and
pastes them into `userdata/shell.ron` gets a fragment in the same
dialect their own file is read in. If the extension were ever lost
from `egui-shell`'s `ron::Options`, this file would fill up with
`question: Some("…")` — the round trip would still pass, and the
format would have quietly stopped being hand-editable.

Note what is *not* asserted: the file carries **no**
`#![enable(implicit_some)]` header and **no** struct names —
`egui-shell`'s `PrettyConfig` sets the extension but ron 0.8
emits neither, so the file opens with a bare `(` rather than
`Shell(`. Neither costs correctness: the reader defaults the
extension on independently of any header (which is the whole
finding recorded in `D:/dev/rag/rust/`), and RON accepts both the
named and the anonymous struct spelling, so the documented
`Shell(tabs: [ Tab(id: "tools") ])` form still parses — see
[`a_hand_written_snippet_parses`], which uses it. Both would make
the generated file more legible and both are `egui-shell`'s to
change, not this crate's.

### `fn the_ron_file_reads_as_a_ribbon`

A weak assertion on purpose: it is not checking the layout, which
the equality test covers exactly. It is checking that the *file*
contains the words an operator would search for — that the
serialized form is legible enough to edit, which is the property
the whole format choice was made for.

### `fn built_in_ron`

Compiled in rather than read at run time: this is the **built-in
layer**, the one that is always available as the reset target and can
never be missing or malformed on an operator's machine. A layer read
from disk is layer two or three.

### `fn parse_built_in`

# Errors

[`ManifestError::Parse`], carrying RON's line and column. Unreachable
in a shipped build — the text is compiled in and a test parses it — but
returned rather than unwrapped so that the same function can be pointed
at an operator's file by a tool that wants the span.
