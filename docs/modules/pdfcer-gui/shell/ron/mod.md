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

# ★ `IMPLICIT_SOME`, and why the round trip alone would not have caught
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
