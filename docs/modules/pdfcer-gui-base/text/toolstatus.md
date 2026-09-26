# `pdfcer-gui-base/text/toolstatus`

## Item notes

### `fn status_line`

# Why an em dash and not the mock's middle dot

`mockups/pdfcer-shell.html` renders *"Select · click to pick · drag to
marquee"* — a name and two gesture fragments, all separated by `·`. That
shape would need a compressed gesture string per tool, none of which exist,
and it makes the name look like a third fragment rather than the subject of
the line.

An em dash says *this is the thing, and this is what it does*, which is
the actual relationship, and it lets the existing per-tool sentences be
used verbatim. The mock is a design reference; where it disagrees with a
sentence that is already written and tested, the sentence wins.

`name` is never formatted into the sentence and the sentence is never
truncated here. Truncation is the **strip's** business — it has a clip
rectangle and the caller elides against it — because a catalog function
that shortened its own output would put a layout decision in a file with
no way to measure one.

### `fn status_tooltip`

# It says where the controls are, because that is the strip's one hazard

Properties owns the text pen's font, size and colour, the measure
pick-list and the three scale switches. An operator hunting for them
reaches for the armed tool first, finds one line, and reasonably concludes
the capability was removed — the failure this project names
*"The feature works. He could not find it."*

So the strip's hover is not a description of the strip. It is a pointer to
the surface that owns the controls.
