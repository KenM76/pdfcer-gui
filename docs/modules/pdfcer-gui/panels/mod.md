# `panels` — the dock's panel bodies

**Thirteen** panels, each a **function the dock can call**. This module owns the
set, the dispatch, the little state the bodies share, and the two layout
rules that every one of them has to get right.

| Panel | Ribbon command | Salvaged from |
|---|---|---|
| [`attachments`] | `edit.attachments` — **new**; see that variant for the tab argument | **new** — no old-shell surface existed |
| [`bookmarks`] | `view.panel_bookmarks` | `panels_structure.rs` |
| [`layers`] | `view.panel_layers` | `panels_structure.rs` |
| [`signatures`] | `view.panel_signatures` | `panels_structure.rs` |
| [`fonts`] | `file.fonts` | `panels_structure.rs` |
| [`objects`] | `view.panel_objects` | `main.rs` + `object_provider.rs` + `object_summary.rs` |
| [`properties`] | `file.properties` | **new** — `RIBBON_IA.md` §5.8 |
| [`docprops`] | `file.document_properties` — see that variant | **new** — `RIBBON_IA.md` §5.1's File ▸ Document band |
| [`forms`] | `view.panel_forms` — moved off Edit so Read can reach it | `panels_forms.rs` |
| [`pages`] | `view.panel_pages` — **not registered; see that module** | `main.rs::thumbnail_rail` + `raster::ThumbnailCache` |
| [`comments`] | `markup.comments` — **not** a `view.panel_*` id; see that variant | `main.rs::comments_panel` |
| [`redact`] | `edit.redact` — the reversible half of redaction; its irreversible twin is [`crate::dialogs::redact`] | `main.rs::redact_panel` |

## These panels once had no way in

Recorded here because this is now the file someone reads when they touch
them. The old shell's `panels_structure.rs` header:

> All three shipped with a `PaneSubject`, a panel body, a rail entry and
> a diagnostic step — and no control an operator could click. Their only
> callers were the harness step handlers, so every verification passed
> while the panels were unreachable in a real build.

That is what [`Panel::command_id`] and
[`tests::every_panel_is_reachable_from_the_ribbon`] exist for, and the
check here is stronger than the one it replaces. The old gate was a
**source-text grep** — it read `main.rs` as a string and looked for a
`show_pane_subject(…)` call outside the harness function. This one asks
the shell manifest whether a real ribbon command names the panel, and
asks the command registry whether that command exists, so it is
satisfied by the same data the ribbon draws itself from rather than by
the presence of a substring.

A panel added here without a ribbon command does not compile a warning;
it fails a test with the name of the panel in the message.

## Actions, not mutations — and it still has teeth

A panel body never touches a document. It is handed `&OpenDoc` — a
**shared** reference, so this is a compile-time fact and not a
convention — it reads, and it pushes a
[`crate::app::actions::Action`]. `PROJECT_PLAN.md` §3 lists this first
among the invariants that are *"not up for renegotiation"*, and
`crate::app::actions`' own header explains why retrofitting it is
expensive.

**Three** panels can act on the document, and the count is worth stating
plainly rather than discovering. Bookmarks pushes [`Action::GoToPage`].
Layers pushes [`Action::SetLayerVisible`] and [`Action::ResetLayers`],
which arrived at S4 and are what restored its visibility checkbox.
[`comments`] pushes [`Action::GoToPage`] as well, and *only* that: the old
shell's Comments panel could also delete an annotation, and that half is
deliberately absent here because no [`Action`] variant can carry the
intent — see that module's header for what the day it lands needs. Every
other panel is still a report — and where the old shell had a control that
this build does not (the Fonts unembed and embed buttons), that panel's
own module docs say which control is missing and what it is waiting for,
because a control with no action behind it is an affordance for something
that cannot work (`RIBBON_IA.md` P3, R83).

### A note for whoever restores one of the remaining controls

The Layers checkbox is the worked example, and its three preconditions are
written up in [`layers`]' own header. In summary: the renderer had to
accept an override (it always did), the render worker's cache key had to
vary with it (S4: `RenderKey` carries `layers_generation`, and
`crate::app::state::OpenDoc` carries the override plus
`set_layer_visible`, `set_hidden_layers`, `reset_layers`), and an
[`Action`] variant had to carry the intent from the panel to `apply`
(S4, last to land). **A control that is missing any one of the three
renders nothing at all** rather than shipping and looking broken.

It is tempting, seeing an override behind a `RefCell` on `OpenDoc`, to
reach for interior mutability and let the panel toggle it through the
shared reference. **Do not.** The `RefCell`s there hold *derived caches*,
whose filling nothing can observe; layer visibility is state that decides
what appears on the page. Mutating it from a widget would make "what can
change what is drawn?" un-greppable, which is the fourth of the four
properties `crate::app::actions`' header says the funnel buys.

Note also that a panel may raise **several** actions for one gesture, and
that this is the intended shape rather than a workaround: the Layers
panel's `/RBGroups` radio behaviour is one `Action::SetLayerVisible` per
layer that moves, applied in order, each recomputing from the state the
previous one left. That keeps one greppable action per changed layer
instead of a single variant carrying an opaque set.

## Rule 4 lives here

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`'s first
non-negotiable, in one clause:

> **Disclosure lives off-canvas**: a status line, a results panel, a
> report after the command, a properties field. Never blocking, never
> requiring acknowledgement, never positioned relative to the document.

A panel is the *right home* for everything pdfcer inferred — a substituted
font, a best-fit residual, a snapped point, an approximate text extent —
and the page view must carry none of it. No badge, no tint, no dashed
outline, no "provisional" layer. Nothing in this module draws on the
canvas, and nothing in it may start to: the one-line test is *would a
screenshot of the editing canvas differ from a screenshot of the same
document saved and reopened?*

[`objects::summary::ObjectSummary::bounds_are_approximate`] is where that
bit: in the old shell it drove a **dashed outline on the page**. It
survives as a question, and its answer is now a sentence in
[`properties`].

## Two layout rules every panel obeys

### 1. Scrollbars must be visible

egui's default `ScrollStyle` is `floating()`: a 2 pt sliver that
allocates **zero** space and has `dormant_handle_opacity: 0.0`, i.e. is
fully transparent when the pointer is elsewhere. The area scrolls
correctly and a screenshot of it is indistinguishable from content
clipped at the container edge.

`ScrollStyle::solid()` is not enough on its own: it sets
`foreground_color: false`, which draws the handle from
`visuals.widgets.inactive.bg_fill` — a near-white on a near-white panel
under a light preset. Measured in the old shell: the bar was present,
opaque, correctly sized, reserving its 10 pt of layout, and invisible in
a capture.

[`scroll_style`] sets all three, and every panel calls it. See
`D:\dev\rag\egui\scrollstyle_solid_draws_the_handle_in_bg_fill_which_is_invisible_on_a_light_panel.md`.

### 2. A fixed-size child inside a scroll area needs the container's
width stated

`Ui::allocate_ui*` and `add_sized` CLAMP their requested size to the
space left in the parent region, so a row wider than the viewport is
silently squeezed, the area measures content == viewport, and no bar
appears anywhere — the overflow is clipped by the outer container with
nothing to say so.

[`content_width`] is the fix, and it is a pure function precisely so it
can be tested: the container's width is `max(widest row, viewport)`,
never a measurement of the laid-out row. See
`D:\dev\rag\egui\allocate_ui_clamps_to_remaining_space_so_a_horizontal_scrollarea_squeezes_a_column_instead_of_scrolling.md`.

## Item notes

### `fn sync`

Called once per frame, before any panel body runs, so no two panels
can disagree about which revision they are describing — which is the
whole point of doing it here rather than in each body.

**A page or revision change clears the focus and the expansion sets.**
Paint-order indices are positions, not identities: deleting one object
renumbers every object after it, so a retained focus would silently
describe a *different* object with the same number, and a retained
expansion set would open the wrong rows. Forgetting is the only honest
response, and it is cheap.

The key is `(page index, edit epoch)` and nothing else. It does not
need to say *which document* — a different document reaches
[`Self::forget_document`] through `PdfcerApp::open_path` before any
panel draws, so there is nothing left to confuse it with. That is what
let the old four-field `DocKey`, with the `Arc` address in it, be
deleted rather than repaired; see this struct's own header.

### `enum Panel`

An enum rather than a trait object, for one reason that matters and one
that follows from it. The reason that matters: [`Panel::ALL`] makes the
set **enumerable**, which is what lets a test sweep every panel and
assert something about each one — the reachability check below is exactly
that, and it is the check three panels shipped without. A registry of
boxed closures would be extensible and unsweepable.

The reason that follows: a dock hosting these needs to persist which
panels are open, and a `Copy`, `Eq`, `Debug` enum serialises to a token
that survives a restart. A closure does not.

### `const ALL`

Hand-written, because Rust cannot enumerate an enum. That makes it
the classic array that silently stops being exhaustive when a variant
is added — so [`tests::the_panel_catalog_is_complete`] pins its
length against a match that the compiler *does* check, which is the
only way to make a hand-written catalog self-defending.

### `fn command_id`

**This is the reachability contract**, and it is the answer to the
defect in this module's header. Every panel names a command; the test
below asserts every one of those commands is both registered in
`crate::shell::commands` and referenced by
`crate::shell::manifest::built_in`. A panel with no route from the
ribbon cannot get past that.

Seven of the thirteen are **not** on View ▸ Panels, and every placement is
`RIBBON_IA.md`'s:

- **Fonts is `file.fonts`.** §7's migration map moves it from View ▸
  Panels to File ▸ Document, because the Fonts panel answers "what is
  inside this file", not "what is on my screen".
- **Properties is `file.properties`.** The document's own title,
  author, subject and keywords are a second panel and a second command,
  so this tooltip says only what its own panel does. See
  [`Self::DocumentProperties`].
- **Document properties is `file.document_properties`**, beside it in
  File ▸ Document for the reason Fonts is there: it answers *"what is
  inside this file"*.

### `fn from_command_id`

The dock stores opaque ids, so something has to turn one back into a
panel, and this is deliberately the only thing that does. Written as
a search over [`Self::ALL`] rather than a second `match`: a second
`match` is a second list to keep in step, and the failure when it
drifts is a panel that opens from the ribbon and draws nothing in
the dock — which looks like a rendering bug and is not.

Returns `None` for an id this build does not have, which is a
reachable state: a saved layout can name a panel whose capability
was compiled out.

### `fn show`

The one entry point a dock calls. `doc` is `None` when nothing is
open, and that case is handled **here** rather than once per panel: the
answer does not vary by panel, and a bespoke "open a document to…"
sentence per panel would be one chance each for one of them to drift.

The bodies below therefore all have the shape
`fn body(ui, doc: &OpenDoc, state: &mut PanelsState, actions: &mut Vec<Action>)`
and never see the empty case.

# Two routes out, and why context-menu commands take the second

`actions` carries what a panel decides for itself — the Bookmarks
panel's `GoToPage`, the Layers panel's `SetLayerVisible`. The
**return value** carries `egui_shell::HandlerToken`s: the commands an
operator chose from a panel's context menu.

A panel must not translate those into `Action`s, for the same reason
the canvas must not. A token is resolved to an id and dispatched by
`PdfcerApp::dispatch_token`, which is the single choke point where a
confirmation gate, an undo entry or a refusal lives; a panel that
translated `file.properties` for itself would be a second
implementation of a command that already has one, and the two would
drift the first time the command grew a precondition.

`host` is `None` when the application has no validated shell (see
[`MenuHost`]), in which case no panel attaches a menu and a
right-click does nothing.

**Two panels attach a menu today**, and only one of them can open
one. Objects attaches `objects.row`, which `crate::shell::menus`
defines. [`pages`] attaches `pages.row`, which it does **not** — so
that right-click opens nothing at all, which is the correct behaviour
for a surface with nothing to offer and becomes the intended menu the
day the context is defined, with no edit in the panel. The other five
return an empty `Vec` because no context is defined for them either.

### `struct PanelsState`

# Why this exists at all, and why it is not on `PdfcerApp`

Two panels are not pure functions of the document: the Objects
panel remembers which rows are expanded and which row was last picked, and
the Properties panel reads that pick. None of it is document state, and
none of it is derivable from anything — but all of it has to outlive a
frame.

It lives here rather than as fields on `crate::app::PdfcerApp` because
*this* is the module that owns the concepts. A dock hands one `&mut
PanelsState` to whichever panel it is drawing, and the app holds it the
way it holds any other subsystem's state. Spreading these fields across
`PdfcerApp` would put the Objects panel's expansion set next to the render
worker.

# What is NO LONGER here: the two caches, and their identity key

Until S4 this struct also held the page decomposition and the font
inventory, guarded by a `DocKey` assembled from the `Arc<EditSession>`'s
**address** plus the path, page count and edit epoch. The header of that
type documented its own residual hazard: an address is not an identity,
because a dropped `Arc`'s allocation can be reused, so a reopened document
could in principle have been served the previous one's decomposition. It
also documented why the obvious fix was worse — holding an `Arc` or a
`Weak` clone would make it a real identity and would break
`crate::app::state::OpenDoc::session`'s `Arc::get_mut` mutation path,
disabling document editing to fix a cache.

Both caches now live on `crate::app::state::OpenDoc`, where the document's
own lifetime bounds them and **no identity key is needed at all**:
`OpenDoc::new` constructs a whole new document state per open, so a cache
inside it can never describe a previous file. `DocKey` was deleted rather
than repaired, because an identity key existed only to compensate for a
cache outliving the thing it described.

What is left here genuinely does outlive a document — it hangs off the
application — and it is handled by *forgetting* rather than by keying:
[`Self::forget_document`] is called from `PdfcerApp::open_path`, the one
place a document is ever opened, and from [`Panel::show`] when nothing is
open. A single statement at the one moment it is true beats a comparison
made sixty times a second.

Within one document, [`Self::sync`] still drops this state on a page or
revision change, keyed on `(page index, edit epoch)` — two plain values,
no address, no ABA hazard.

### `struct ObjectTreeUi`

Every field is cleared when the page or the document revision changes
(see [`PanelsState::sync`]), because a paint-order index is a **position
on one page of one revision**, not an identity.

### `fn forget_document`

Called from two places, and both are needed: `PdfcerApp::open_path`,
because a new document makes every paint-order index here meaningless,
and [`Panel::show`] when nothing is open, because a panel that never
draws while the shell is empty would never get the chance.

`*self = Self::default()` rather than clearing fields one at a time,
so a field added later is forgotten by construction — which is right
for everything here that describes a DOCUMENT, and wrong for the two
fields that describe the OPERATOR.

**The page-preview tick and its time limit are carried across the
reset**, and the body says at length why. In one sentence: they are
preferences read from `preferences.txt` at construction, this function
runs after construction on every launch that opens a file, and a reset
over them makes `OPERATOR_REQUESTS.md` O187 do nothing at all.

**One thing this struct's reset cannot reach**, and it is named here
rather than left to be discovered: `properties::refusedchar` keeps the
refusal that has been *recorded and not yet adopted* in a thread-local,
because it is written by the dispatcher and read by a body that is handed
`&OpenDoc` shared — there is no `&mut` path between them. A refusal left
there when a document closes would be adopted by the next document's
first draw, where its `(page, run)` names different text. So the reset
says so explicitly, and the "forgotten by construction" property above
holds for every field that a `Default` can reach.

### `fn tree_mut`

Handed out whole rather than through a method per field, because the
Objects panel reads the expansion sets while it draws and writes them
after; splitting that across four accessors would gain nothing and
cost the panel the ability to hold one borrow for the frame.

Note what it is **not** paired with any more. Until S4 this came back
alongside the page decomposition from one method, because the panel
needed `&provider` and `&mut tree` simultaneously and Rust permits
that only as two disjoint borrows of one struct. The provider now
lives on `OpenDoc`, so the two come from different objects entirely
and the pairing has no reason to exist.

### `fn focus`

Delegates to [`ObjectTreeUi`], which is where the field lives. The
forwarder exists so a panel that only needs to *read* the focus — the
Properties panel — does not have to reach through the grouping.

### `fn pages_mut`

Handed out whole for the same reason [`Self::tree_mut`] is: the body
reads the cache while it lays tiles out and writes the selection while
it reads the clicks, and splitting that into accessors per field would
cost it the ability to hold one borrow for the frame.

### `fn layers_search_mut`

Handed out whole for [`Self::pages_mut`]'s reason: the body reads the
query while it draws the field and writes the mode while it reads the
switch, and splitting that into accessors per field would cost it the
ability to hold one borrow for the frame.
The Layers panel's search box, mutably.

One accessor for one `String`, matching [`Self::redact_mut`]'s shape:
the panel needs the `&mut` to hand to a `TextEdit` and needs to read
the trimmed value back in the same frame.

### `fn docprops_mut`

Same shape as [`Self::pages_mut`] and [`Self::redact_mut`]: the body is
handed `&mut PanelsState` and reaches its own state through an
accessor, so the field stays private and no other panel can write it.

Named after [`Panel::DocumentProperties`] and not after Properties:
an accessor named for the panel that does not own the state is how a
caller writes the wrong drafts.

### `fn field_rename_mut`

The re-seeding is the whole reason this is a method rather than a
bare `&mut String`. Without it, clicking field A, typing a new name, then
clicking field B leaves A's half-typed name in the box — aimed at B. The
operator presses Rename and renames the wrong field to a name they chose
for a different one, and nothing on screen said which field the box
belonged to.

So the key travels with the draft and a mismatch reseeds. `for_field` is
the FULLY-QUALIFIED name (the identity), and the draft is seeded with the
**last dotted segment** — the partial name, which is what
`rename_field` takes. Seeding it with the qualified name would invite the
operator to press Rename on a string containing a dot, authoring a `/T`
nothing can address.

### `fn text_style_mut`

No re-seed argument, unlike [`Self::field_rename_mut`]: the draft owns
its own stamp and decides for itself when what it holds is stale, which
is right here because the staleness condition includes the edit epoch
and a caller would have to be handed that as well.

### `fn text_object_mut`

No re-seed argument, like [`Self::text_style_mut`] and for its reason:
the draft owns its `(page, object, epoch)` stamp and decides for itself
when what it holds is stale.

### `fn refused_char_mut`

No re-seed argument, like [`Self::text_style_mut`]: the struct owns its
own `(page, run, epoch)` stamp and its own retirement rule, and decides
for itself when what it holds has stopped being true.

### `fn annot_delete_mut`

No re-seed argument, like [`Self::text_style_mut`]: the memo owns its own
`(id, epoch)` stamp and decides for itself when what it holds is stale,
which is right here because the staleness condition includes the edit
epoch and a caller would have to be handed that as well.

### `fn attachments_mut`

Same shape as [`Self::bookmarks_mut`]: the body is handed
`&mut PanelsState` and reaches its own state through an accessor, so the
field stays private and no other panel can write it.

### `fn selected_pages`

Read-only, and this is the accessor a `pages.*` dispatch arm must
use when the first one lands. The ribbon's Pages tab already promises
this set in every one of its tooltips — `pages.delete` is *"Remove
**the selected pages** from this document"* — and
`crate::shell::commands`' comment on that band says those verbs
*"respect the thumbnail rail's selection when there is one"*.

It is exposed *before* anything reads it, deliberately, because the
alternative is that the first arm to arrive invents a second page
selection of its own — the exact drift [`ObjectTreeUi::focus`]'s docs
refuse for objects. Empty is a defined answer, not a missing one: with
nothing picked those commands act on the current page.

### `fn scroll_style`

Scoped to the `Ui` that owns the scroll area rather than to the app
style, because "always show a solid bar" is right for a narrow panel
column and not obviously right for every surface in the application.

Three settings, and all three are needed — see this module's header for
the measurement behind each:

1. `solid()` over the `floating()` default, so the bar allocates layout
   and is drawn when the pointer is elsewhere.
2. `foreground_color = true`, so the handle is drawn in the visuals' TEXT
   colour rather than `widgets.inactive.bg_fill` — which on a light
   preset is a near-white handle on a near-white panel. This is also the
   theme-respecting form: the handle inherits whatever contrast the
   active theme gives its text, so it stays correct across light and dark
   without a hard-coded colour.
3. `bar_width = 10.0`, wide enough to grab with a mouse.

### `fn content_width`

# The defect this prevents

`Ui::allocate_ui_with_layout_dyn` fits its requested size into the space
**remaining in the parent region**, so a row that asks for 600 pt inside
a 370 pt viewport receives 370. The row's `min_rect` therefore measures
exactly the viewport width, `ScrollArea` compares content against
viewport, finds them equal, and draws no bar. The visible symptom is a
label cut off at the panel's edge with no way to reach the rest of it,
and nothing errors or warns.

`auto_shrink([false, false])` does not help — it stops the area shrinking
*below* the viewport, it does not let content exceed it. `max_width` does
not help either; it bounds the viewport, which was already right.

The fix is to state the content's own width on the container:
`Ui::set_width` calls `set_max_width`, and `Placer::set_max_width` GROWS
`max_rect` rather than only shrinking it, which is what gives the rows
their real width and lets the area measure content > viewport.

# Why `.max(viewport)`

So a wide panel still fills rather than leaving a dead strip to the right
of the rows.

# Why this is a function and not three lines at the call site

So it can be tested. The RAG note this comes from is explicit that the
value must not be a measurement of the *laid-out* row — measuring is what
produced the squeezed number in the first place — and the difference
between "the intrinsic width of this text" and "the width this row ended
up with" is invisible at a call site and obvious in a test.

### `const ELLIPSIS`

One code point, not three periods. Three periods measure wider, and at the
width where a row is being shortened at all, three periods is another
character and a half of the operator's text spent on the punctuation that
says text was spent.

### `fn elide_to_width`

Returns `None` when the whole label fits, and `Some(shortened)` when it does
not. The caller draws whichever it got and attaches the **full** text on
hover in the `Some` case.

# Shortening is not the clipping that is ruled out

The standing requirement is that **row text must not clip**: a panel that
cuts a row at the pane's edge with no bar, no mark and no recovery loses
text **silently**, and silence is what is forbidden. This shortens the row,
says so with a character the eye reads as *there is more*, and puts the
whole string one hover away. The thing that must not happen — an operator
seeing `AAAAAA+SpaceGrotesk-Bold 1` and having no idea a `2` was cut off —
cannot happen.

# Why a `measure` closure rather than a `&Ui`

So the decision is a pure function and can be tested against a synthetic
font. Every earlier attempt at this in this crate ended as three lines
inside a draw closure, and [`content_width`]'s own doc records what that
costs: *"the difference between 'the intrinsic width of this text' and 'the
width this row ended up with' is invisible at a call site and obvious in a
test."*

# The search

Binary search over **character** counts, never bytes: slicing a UTF-8 string
at a byte offset panics mid-code-point, and this crate's rows carry the
middle dot, the em dash, the multiplication sign and font names with
accents. The predicate is monotone — a longer prefix is never narrower — so
the search is sound, and it costs `log2(len)` measurements on the rows that
need it and one on the rows that do not.

Returns `Some` of the bare ellipsis when not even one character plus the
ellipsis fits. That is a legitimate state at a very narrow dock and it is
**still better than a clipped row**: a lone ellipsis says *this is a row,
and it has content you cannot see here*, and it still carries the hover.

### `fn text_width`

The *intrinsic* width — what the text would occupy with no wrapping and
no container — which is the number [`content_width`] needs and the one a
laid-out row cannot give (a laid-out row has already been clamped).

`layout_no_wrap` is what makes it intrinsic — the same call the widget
itself will make, so the number is the width the row would want rather
than an estimate of it.

The colour is [`egui::Color32::PLACEHOLDER`] because a galley's *width*
does not depend on its colour, and naming a real one here would tie a
measurement to a theme decision.
