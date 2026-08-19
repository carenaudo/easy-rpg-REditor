# UX reference: EasyRPG Editor (Qt)

Notes from reading [EasyRPG/Editor](https://github.com/EasyRPG/Editor)'s actual
source (cloned to `D:\programacion\Editor` for this pass) — the official Qt
database/map editor for RPG Maker 2000/2003 projects, built on liblcf. This
isn't a screenshot tour; it's what the window/panel/navigation structure
*is*, read from the `.ui`/`.qml`/`.h` files, so REditor can borrow the parts
that make sense without guessing.

## Correction on tech stack

**It's not pure QML.** That's a reasonable assumption from the outside (the
`org.easyrpg.editor` QML module, Kirigami import, etc. are real and used
extensively), but reading `src/main.cpp` shows the actual app entry point
loads a classic C++ `QMainWindow` (`src/ui/main_window.h/.cpp/.ui`) — the
`MainWindow.qml` file exists but its load call is commented out, i.e. not
yet wired into the running app.

So it's a **hybrid**:
- **Qt Widgets** (`.ui` files, `QMainWindow`, `QGraphicsView`, `QDockWidget`):
  the main window shell, the map canvas, the tileset palette, event editing,
  and most dialogs (new/open/import project, resource manager, search, map
  properties).
- **QML + Kirigami** (KDE's QML app framework) + **kddockwidgets**: the
  Database editor window specifically, plus asset pickers/viewers (image,
  charset, faceset, sound, music). This looks like the actively-being-migrated
  part of the app — newer, cleaner, more declarative — while the map/event
  editing core is still the original Widgets implementation.

Relevant for REditor: it's fine (and apparently what upstream itself is
doing) to mix UI approaches by area rather than forcing one paradigm
everywhere. In egui terms there's no direct equivalent to that mixing, but it
validates *not* over-engineering a single unified pattern before it's needed.

## Main window layout

From `src/ui/main_window.ui` + `main_window.h`:

- **Menu bar**: Game &Project, Maps, Tools, Game, View, Debug, Help.
- **Two toolbars** (`mainToolBar`, `toolBar`, `toolBar2`) — project actions
  (Open/New/Close/Quit), map actions (Save/Revert), draw tools, layer
  switches, zoom.
- **Two dock widgets**, both defaulting to the left dock area (user can
  rearrange/float/hide them — standard `QDockWidget` behavior):
  - **Map Tree** (`dockMapTree`): a `QTreeWidget` with columns "Map Name"
    and "Map ID" — a real tree (folders/nesting, matching RPG Maker's
    tree_order/area-node concept), not a flat list.
  - **Palette** (`dockPalette`): a `QGraphicsView` (`graphicsPalette`)
    showing the chipset image directly as a clickable tile picker.
- **Central widget**: a `QTabWidget` (`tabMap`) — each open map is its own
  tab, each tab holding a `QGraphicsView` + `MapScene`. Closing/switching
  tabs is handled explicitly (`on_tabMap_tabCloseRequested`,
  `on_tabMap_currentChanged`), i.e. multiple maps can be open and edited
  concurrently, like a text editor's tabbed documents.

### Map editing toolbar actions (from `main_window.h` slots + `.ui` strings)

| Action | Shortcut | Notes |
|---|---|---|
| Lower Layer | F5 | switches the draw target layer |
| Upper Layer | F6 | |
| Events | F7 | switches to event placement/edit mode |
| Zoom In / Out / 100% | +, -, — | |
| Draw: Pen / Rectangle / Circle / Fill | — | classic paint-bucket-style tools, not just single-tile placement |
| Undo | — | |
| Save Map / Revert Map | Ctrl+S, Ctrl+R | per-map save, not just whole-project |
| Map New / Copy / Paste / Delete / Properties | — | tree-widget context actions |
| Search | — | project-wide search dialog |
| Play Test | F4 | launches the game via Player for live testing |
| Database | F8 | opens the separate Database window |
| Resource Manager, Jukebox | — | separate dialogs/windows |

Worth noting for REditor: **Save/Revert are per-map**, not a single global
"save everything" action — each open map tab tracks its own dirty state
(`on_mapChanged`/`on_mapUnchanged` slots update the UI accordingly). This is
a different granularity than our current single global "Save Changes" button
for the whole Actors table.

## Database window: 3-column drill-down

From `DatabaseWindow.qml`'s own doc comment — this is the clearest, most
directly reusable pattern for REditor since our "Actors" tab is exactly this
category:

```
Column 1: Categories        Column 2: Entries in category    Column 3: Edit form
(Actors, Items, Skills...)  (0001: Alex, 0002: Brian...)      (Name, Title, Level, Face, Equipment...)
```

Implemented via Kirigami's `pageStack` (a column-based navigation stack,
conceptually similar to `NavigationSplitView` on other platforms, or a
breadcrumb-driven master-detail-detail view). Selecting a category pushes
column 2; selecting an entry pushes/replaces column 3. `pageStack.pop()`
twice returns to category selection.

**Categories currently implemented** (`DatabasePage.qml`'s `pageModel`):
Actors, Items, Skills, Attributes, System (single-instance, no list),
Vocabulary/Terms (single-instance). Notably *not yet present*: Classes,
Enemies, Troops, Common Events, Switches/Variables — so even upstream's
editor doesn't cover the full RPG Maker database yet. Useful context: we're
not behind some "complete" reference implementation by only having Actors.

**Entry list formatting**: `"0001: Alex"` — 4-digit zero-padded index +
name, auto-selects entry 0 on opening a category, live-swaps the edit
column when selection changes (reuses the already-pushed page rather than
re-pushing, to avoid rebuilding the form on every click).

### Edit form pattern (`ActorPage.qml`)

A `Kirigami.FormLayout` with **section separators** (`Kirigami.Separator` +
`FormData.isSection: true`) grouping fields: "General" (Name, Title,
Level range, Critical hit chance), "Graphics" (Face/Character viewers,
clickable to open an asset picker), "Equipment" (a `Repeater` generating one
combo-box per equipment slot, each filtered to the right item type via a
proxy model).

**Live two-way binding, no explicit save button**: every field is bound
directly to a `jsonData`/`key` pair pointing at a live in-memory JSON view of
the project database (`Ez.TextField { jsonData: root.jsonData; key: "name" }`).
Typing in the field mutates the underlying model immediately — there's no
per-entry "Save" action at all in this page. This is a **materially
different UX model** from what we built for REditor's Actors tab (explicit
`actors_dirty` flag + "Save Changes"/"Discard Changes" buttons). Presumably
there's a separate project-wide save (matching the main window's Ctrl+S
map-save pattern), committing the whole live in-memory model to disk at
once, rather than per-field or per-entry saves.

**Worth deciding explicitly for REditor**, not just defaulting to whichever
was easier to build first:
- *Live-edit, project-level save* (their approach): fields mutate in-memory
  state instantly; one save action persists everything dirty across the
  whole project (maps + database) at once. Fewer buttons, but "unsaved
  changes" becomes a whole-project concern instead of a per-view one.
- *Explicit per-view save* (what REditor has now): each view manages its
  own dirty flag and save action independently. More granular feedback,
  more buttons, easier to reason about "what does this Save button do."

Given REditor's write path is still young (one entity type, backup-on-write
safety net), the current explicit-per-view approach is the more
conservative choice and probably right to keep until there's a real
project-wide save/dirty-tracking story worth building.

## Event editor

`event_dialog.ui` (Widgets, not QML): a dialog with a Name field, page
management buttons (New/Copy/Paste/Delete Page — RPG Maker events have
multiple numbered "pages", each with its own trigger condition), and a
`QTabWidget` (`tabEventPages`) — one tab per event page.

Each page tab (`event_page_widget.ui`) has: **Precondition** (switch/
variable/item-in-party/hero-in-party/timer conditions — these are the
conditions that decide which page is "active"), **Event Graphic** (with a
Trans/Set transparency toggle), **Trigger Condition**, **Event Layer** (+ a
"Forbid overlap" checkbox), **Movement Type** (+ Frequency, "Define
Pattern"), **Animation type**, **Move Speed**, and an **Event Commands**
list area (the actual scriptable command sequence).

Relevant to REditor's current event overlay (map markers + hover tooltip
showing id/name/position): this confirms events are meaningfully more
complex objects than what we surface today — each has multiple *pages*,
each page its own precondition/graphic/trigger/movement. If event editing
(not just viewing) becomes a goal, the page concept is unavoidable — a
single flat "name + position" view (what we show now) only covers page 1's
identity, not the page-switching logic that gives events their actual
behavior.

## Asset pickers/viewers (QML)

A consistent two-part pattern used for every image/sound/music reference
(character graphics, faces, chipsets, sound effects):

- **Viewer** (`ViewerBase.qml`): a thumbnail/preview `Item`. Click anywhere
  on it → opens the matching **Picker** as a modal dialog layer
  (`pageStack.pushDialogLayer`).
- **Picker** (`PickerBase.qml`): a 25%-width scrollable file list on the
  left (browsing the relevant resource directory, e.g. `CharSet/`), a
  live preview/form on the right, Cancel/Select actions. Selecting commits
  back to the viewer via an `accepted()` signal and closes the dialog.

This is directly analogous to what REditor would need if it ever lets users
*change* an actor's face/character graphic rather than just display current
stats — worth keeping in mind as the reference pattern rather than
reinventing a picker UX from scratch later.

## Theming

`resources/icons/{bright,dark,ezbright,ezdark}` — separate icon sets for
light/dark theme, switched at runtime based on `QStyleHints::colorScheme()`
(`main.cpp`'s `scheme_changed` lambda, listens for OS theme changes live,
not just at startup). REditor currently hardcodes `egui::Visuals::light()`
at startup with no dark-mode option — this is the kind of "obviously should
support both eventually" gap their approach highlights, though not urgent
given REditor's current scope.

## Takeaways worth acting on for REditor

Roughly in order of how directly they map onto REditor's existing structure:

1. **Database categories as a real navigation concept, not one flat tab.**
   REditor's "Actors" tab works for one category; once a second one (Items,
   Skills) gets added, a Categories → Entries → Detail drill-down (even a
   simplified two-pane version, not full Kirigami column-stack) scales much
   better than one flat grid per category, and avoids an ever-multiplying
   set of top-level tabs.
2. **Zero-padded numeric IDs in list labels** (`"0001: Alex"`) — small, but
   it's exactly how RPG Maker's own editor and the file-naming convention
   (`Map0001.lmu`) present things; worth matching for familiarity.
3. **Per-map (or per-entity) save/dirty state, not just one global flag** —
   already partly true for REditor (Actors has its own dirty flag), but the
   Qt editor's per-*map*-tab dirty tracking is the pattern to reach for once
   REditor's map view supports any editing.
4. **A tileset palette as a picker, not just a static image** — REditor
   already renders chipsets correctly (`tilemap::decode_chipset`/
   `render_map`); turning the *palette* dock into a click-to-select-a-tile
   widget (reusing that same rendering code) is a natural next step once
   REditor does any map editing rather than viewing.
5. **Multiple pages per event.** REditor's event markers currently show
   identity/position only; real event editing needs the page concept this
   editor treats as fundamental.
6. **Viewer+Picker as the standard pattern for any asset reference field**
   (graphics, sounds) — reusable once REditor lets users change rather than
   just display asset references.
