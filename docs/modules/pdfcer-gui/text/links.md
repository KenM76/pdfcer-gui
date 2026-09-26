# `text::links` — what the program says about a link it cannot follow

Consumed by [`crate::canvas::links`]. Five sentences, and **four of them are
about failure**, which is the shape of the problem rather than pessimism.

## ★★★ Why a link that works says nothing at all

Because it navigates. `pdfcer_core::outline::Destination::Page` is the only
variant this program can perform, and performing it *is* the feedback — the
page turns, the zoom changes, the operator arrives. A status line reading
*"followed a link"* over a view that has visibly moved is noise, and R9's
reading of the whole surface is that a capability which works is not
something to announce.

The **cursor** is the whole of the pre-click affordance: a pointing hand
over a link that can be followed and nothing over one that cannot. That is
rule 4's pre-commit clause — a cursor is an affordance, not a mark on the
content — and it is also every reader ever written.

## ★★ Why the four failures are FOUR sentences and not one

`Destination` has five variants and only one navigates. The engine's own
note on shipping the reader is the argument, quoted because it is exact:

> *"A viewer that maps the last four to 'no link here' reports a document
> full of working links as empty. One that maps them to a page jump lies
> about where it goes."*

And they fail for **different reasons with different remedies**, which is
why one generic *"this link doesn't work"* would be worse than useless:

| variant | cause | what the operator can do |
|---|---|---|
| `UnmappedPage` | the target page is not in this document | usually **a page delete** — theirs or an earlier tool's |
| `Named` | a name neither namespace defines | usually a **page-range extraction that dropped `/Names`** |
| `Remote` | `/GoToR` — a page of another file | open that file |
| `NonNavigation` | `/URI`, `/Launch`, `/JavaScript`, `/SubmitForm` | nothing here; pdfcer does not perform these |

Telling somebody their link is broken when the truth is *"this link opens a
web page and this program does not open web pages"* sends them looking for
a defect in their document that does not exist.

## ★ Where these appear, and where they must never appear

**Off-canvas, in the status line, on a click.** Never as a mark on the page,
never as a tint over the link's rectangle, never as a badge. Rule 4's
disclosure clause is explicit that an inference is reported *beside* the
content and not drawn *into* it, and this project's own record of the old
GUI is that *"the nagging and red flagging … made for a lot of extra bugs in
the visibility when editing"*.

★★ They are also raised **only on a click**, never on hover. A sentence that
appeared merely because the pointer crossed a rectangle would fire dozens of
times crossing a table of contents, and a status line that changes without
the operator having done anything is a status line they stop reading.
