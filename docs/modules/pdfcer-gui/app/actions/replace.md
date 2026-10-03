# `app::actions::replace` — Replace and Replace all

The Find bar's Replace row pushes `Action::Find(FindRequest::Replace { all })`.
`app::actions::apply` routes it here, ahead of the base crate's
`find::apply`, because a replace edits the document and the base crate does
not own the edit funnel. The row is drawn, and the action is honoured, only
where the mode can edit content (`Capabilities::edit_content`).

## What a press does

1. `FindState::replace_ask` takes the hits: the current one, or all of them.
   A bar that is not on a current hit of this revision asks for nothing, and
   the press traces `text-replace-declined reason=no-current-results`.
2. A wildcard search is refused (`ReplaceRefusal::Wildcards`): its hits match
   a pattern, and a pattern says nothing about which characters to keep.
3. `locate` maps each hit onto the show operator that draws it (below).
4. Every rewrite goes through one `vector_edit` labelled `text-replace`, with
   the typing disposition augmented by the installed faces, exactly as a
   typed edit is. Each rewrite is its own `edit_text`; the successes are then
   merged with `coalesce_last(n, EditText)`, so **one press is one undo
   step** however many hits it replaced. If the engine will not merge them,
   the status line says how many Undos the press will take.
5. The search runs again, because the hits moved, and the bar lands on the
   same position in the list, which after a single Replace is the hit that
   followed the one rewritten.

Trace: `text-replace-applied which=all|current found= replaced= skipped=
refused= rewrites=`; one `text-replace-refused page= span= error=` per
rewrite the engine declined.

## Locating a hit

A Find hit is a box in unrotated PDF user space; an edit is a pinned byte span
in a content buffer. `locate` joins them per page:

- it recognises the page's `EditableTextModel` and, for every run, lists the
  show operators (`canvas::textedit::pin::operators_in_run`);
- it finds the query in each run's text with the search's own case rule, and
  a match claims the first unclaimed hit whose box, grown by a quarter of the
  glyph size, holds the glyph origin at the match's first character.

A claimed match is rewritten only when it lies wholly inside one operator
that draws nothing on any other line. Every other hit is left alone with a
reason, and the status line names each reason with its count:

| Skip | Meaning |
|---|---|
| `NotOnOneLine` | no run holds the whole hit, so it wraps or spans lines |
| `CrossesStyles` | the hit crosses two operators: a change of font, size or colour |
| `SharedOperator` | the operator also draws another line, so rewriting it would move that line |

## Why each rewrite is `find_replace(old, new).pinned(span)`

The request finds the operator's whole extracted text under its pin and
replaces it with the spliced text. Rewrites are applied last operator first
within each page, so an applied rewrite never moves a span still to be used.
If a span does move anyway, the find text no longer matches under the pin and
the engine refuses that rewrite instead of editing whatever now sits there;
the refusal is counted and disclosed, never silent.

## What it does not do

- Hits that cross styles, lines or shared operators are left for the Edit text
  tool, which can split and restyle; the status line says so.
- Replacing with a character the font lacks follows the typing disposition:
  the installed face is used where one is found, and a refusal is reported.
