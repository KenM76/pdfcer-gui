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
