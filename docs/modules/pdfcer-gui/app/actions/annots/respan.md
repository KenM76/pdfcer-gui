# `app/actions/annots/respan`

The apply arm for `AnnotAction::RespanTextMarkup`.

`respan(doc, id, quads)` calls `EditSession::respan_text_markup` through
`vector_edit` under the label `respan-text-markup`. The call:

- stamps `/M`;
- makes one undo entry;
- discloses every property the re-bake dropped, through `markup_dropped`.

It traces:

```
respan-text-markup-applied id= subtype= quads=before->after rect=before->after dropped= m=
```

The trace records the quads and the `/Rect` separately. A re-span has to
change both, so a build that writes the quads but leaves the box behind shows
it in the trace.
