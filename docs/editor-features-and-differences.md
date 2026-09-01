# EasyRPG REditor — Features & Differences from Classic RPG Maker

This document provides a comprehensive overview of the capabilities of **EasyRPG REditor** and details how its architecture, user interface, and workflow differ from the original **RPG Maker 2000 / 2003** editors and other tooling.

---

## 1. Executive Summary

| Feature / Aspect | Classic RPG Maker (2000 / 2003) | EasyRPG REditor |
|---|---|---|
| **Architecture & Platform** | 32-bit Win32 / MFC, separate `RPG2000.exe` and `RPG2003.exe` | 64-bit Rust (`egui`/`wgpu`), unified single executable |
| **Engine Compatibility** | Strict single engine per editor | **Adaptive Engine Mode**: seamlessly edits RM2000 & RM2003 |
| **Data Safety & Integrity** | Overwrites in place without automatic backups | Automatic session `.bak` backups before writes; crash-safe parsing |
| **Event Placement** | Cut/Paste or manual coordinate entry | **Interactive Drag & Drop** with anti-collision snap-back |
| **Event Templates** | None (manual scripting required) | **Quick Event Templates**: Doors, Chests, Inns, Springs, Transfers |
| **Save File Management** | None (no built-in LSD inspection) | **Built-in Save Editor**: View/edit Gold, Party, Switches, Variables, Items |
| **Audio & MIDI** | Relies on OS Windows MIDI (Midi Mapper) | Built-in **SoundFont (`.sf2`) Synthesizer** + Sound Test Jukebox |
| **Graphics & Palette** | Strict 8-bit BMP/XYZ; magenta transparency | PNG, BMP, XYZ with palette-index-0 transparency preservation |
| **Developer Tools** | None | **XML Import/Export**, **Project Health Analyzer**, **Global Search (`Ctrl+F`)** |
| **User Interface** | Fixed Win32 dialogs; 1 language per binary | Resizable UI, 12 Dark/Light Themes, 8 runtime languages |

---

## 2. Key Differences from Classic RPG Maker

### 2.1 Unified Adaptive Engine Mode (2000 vs 2003)
In the original software, **RPG Maker 2000** and **RPG Maker 2003** were sold and distributed as two separate applications with mutually incompatible database layouts:
- RM2003 introduced Classes, Battle Commands, System2 UI graphics, 10-condition Battle Page triggers, and 2003-specific event opcodes.
- Opening an RM2000 project in RM2003 or vice versa often corrupted database fields or altered version headers.

**In EasyRPG REditor**:
- A single editor application natively loads both RM2000 and RM2003 projects.
- The editor inspects the project signature (`RPG_RT.ldb`) and dynamically adapts available tabs (e.g. Classes, System2, Battle Commands, Timer 2) and event opcodes to match the target engine.
- Saving preserves the authentic engine format without unwanted version upgrades or structure degradation.

---

### 2.2 Modern Canvas, Multi-Tile Painting & Event Interaction
Classic RPG Maker required users to open event properties to check positions, or cut/paste events across tiles, and limited tile painting to single 16x16 tiles or predefined upper-chip stamp patterns.

**EasyRPG REditor Enhancements**:
- **Multi-Tile Brush & Marquee Selection Tool (`🔲 Marquee`)**:
  - Drag-select multi-tile rectangular blocks directly on the tile palette or map canvas.
  - Press `Ctrl+C` or switch tools to copy custom $W \times H$ patterns into the active brush.
  - Real-time $W \times H$ hover bounding preview stamps the entire copied region across the map canvas in a single click.
- **High-Fidelity Autotile Engine (Block D 46-Subtile Math)**:
  - Precise 4-quadrant edge and corner calculation connecting neighbors in 8 directions ($N, NE, E, SE, S, SW, W, NW$).
  - Seamlessly composites all 46 standard RPG Maker autotile configurations (isolated 1x1 islands, horizontal/vertical strips, inner/outer corners, and solid center ground).
- **Direct Map ID Tree Navigation**:
  - The map tree hierarchy resolves map selection directly by unique `map.id` rather than array slice offsets, guaranteeing 1:1 fidelity between sidebar clicks and loaded map canvas data.
- **Drag-and-Drop Event Repositioning**: Simply click and drag any event across the map canvas. If dropped onto an occupied tile or dragged outside the canvas boundary, the event automatically snaps back to its origin without locking editor interactions.
- **Move Route Visual Builder**:
  - Complete interactive step constructor supporting all **42 movement opcodes** (Step Up/Down/Left/Right, Turn, Jump, Speed/Frequency change, Transparency/Phasing, Switch ON/OFF, Graphic change, Play SE).
  - Accurate direction mapping conforming with `liblcf` (`MoveType_vertical = 2`, `MoveType_horizontal = 3`).
  - Reorder, insert, and delete movement steps with instant directional icon feedback, repeat toggles, and skip-if-blocked flags.
- **Visual Overlays & Directional Passability Editor**:
  - **High-Contrast Map Canvas Passability Mode**: Renders prominent circular status badges over every tile: $\bigcirc$ (Passable), $\times$ (Blocked), $\bigtriangleup$ (Counter/Bush), and $\bigstar$ (Above character).
  - **Interactive Directional Passability Mode**: Renders 4 high-contrast directional arrow heads ($\blacktriangle$, $\blacktriangledown$, $\blacktriangleleft$, $\blacktriangleright$) on each tile. Click the top/bottom/left/right quadrants to toggle individual direction passage bits (`0x08`, `0x01`, `0x02`, `0x04`) while preserving upper classification flags, or click the center to toggle all/none.
  - **Map Canvas Terrain Tag Overlay (`🏷 Terrains`)**: Color-coded pill badges (`T1`..`T8`+) displayed directly over canvas tiles representing their assigned terrain types from the chipset.
  - **Inactive Layer Dimming**: Dims the lower or upper layer when editing the other layer, preventing visual confusion.
  - **Grid Lines**: Toggleable tile grid with coordinate tooltip inspection.
- **Anchor-Aware Map Resizing with Automatic Event Coordinate Shifting**:
  - Resizing map dimensions with 9-point anchor origins (TopLeft, TopCenter, TopRight, CenterLeft, Center, CenterRight, BottomLeft, BottomCenter, BottomRight) intelligently offsets and clamps both lower/upper tile layers and all map events (`map.events[].x/y`) as well as party and vehicle starting points (`tree.start`).
  - Saving map properties immediately triggers a live project tree and map list reload in the sidebar.
- **Quick Event Templates**: Right-click any tile to generate pre-configured, fully scripted game events:
  - **Treasure Chest**: Customizable graphic, reward (Item/Gold), sound effect, and self-switch page.
  - **Door Transition**: Multi-frame opening animation, door SE, screen fade, and destination teleport.
  - **Inn**: Payment check, night screen fade, ME playback, and full party recovery.
  - **Save Point & Recovery Spring**: Interactive healing or save prompt.

---

### 2.3 Comprehensive Save File (`.lsd`) Management
In classic development workflows, testing mid-game switches, variables, or party states required adding temporary debug events to maps.

**EasyRPG REditor Includes a Dedicated Save View**:
- **Slot Overview & Lifecycle Tools**:
  - Lists all `Save01.lsd` .. `Save15.lsd` files with hero name, level, timestamp, map ID, and party members.
  - `➕ New Save`: Generates a standard default initial save file in the next free slot.
  - `📄 Clone Slot`: Duplicates the selected save slot into the next available save file.
- **Party & Stat Editing**:
  - Add (`➕ Add Hero`) or remove (`🗑`) party members dynamically.
  - Modify hero names, levels, current HP/SP, and party members with LSD persistence.
- **Live Switch & Variable Inspector**:
  - Full matrix of save switches with live ON/OFF checkbox toggles.
  - Full table of save variables with numeric drag/input fields.
- **Editable Inventory**: Add, count-adjust, or remove inventory items from existing save files.

---

### 2.4 Built-in Audio & Cross-Platform MIDI Synthesis
Classic RPG Maker relied on the default Windows MIDI Mapper, which often causes issues on modern 64-bit Windows, Linux (Wine), and macOS, or sounds drastically different depending on OS sound cards.

**In EasyRPG REditor**:
- Real-time software MIDI synthesis via `rustysynth` with `.sf2` SoundFont support.
- **SoundFont Manager**: Auto-detects installed SoundFonts (e.g. `FluidR3_GM.sf2`, `GeneralUser_GS.sf2`, Windows system fonts) or allows selecting custom SoundFonts.
- **In-App Sound Test**: Dedicated sound test jukebox with pitch, volume, loop previews, and BGM/ME/SE playback directly inside the editor.

---

### 2.5 Animation 2D Cell Composer & Timing Effects
Classic RM2000/2003 animation editing required clumsy numeric tables or limited previewers.

**In EasyRPG REditor**:
- **Frame-by-Frame 2D Cell Placement**:
  - Inspect and place individual sub-cells (0..49) per animation frame with pixel-precise $(X, Y)$ offsets, Zoom scaling (10%..300%), and Alpha Opacity (0%..100%).
  - Fast workflow buttons: `➕ Add Cell`, `📋 Copy Previous Frame`, `🎯 Center All`, `🔄 Reset Zoom`, and `🗑 Clear Frame`.
- **Live Visual Playback Stage**:
  - Plays back multi-cell compositions in real-time at configurable frame rates (4..30 FPS).
  - Target alignment preview with customizable dummy battler silhouettes.
  - Flash & Sound cue triggers synchronized directly with visual flash overlays and screen shake cues.

---

### 2.6 Comprehensive Event Command Form Suite (50+ Bespoke Editors)
Classic editors used Win32 sub-windows that frequently broke on newer Windows display scaling or Wine.

**In EasyRPG REditor**:
- 50+ bespoke, responsive form dialogs for all standard RM2000/RM2003 event commands:
  - **Messages & Dialog**:
    - **Show Message (`10110`)**: Message text editor with face graphic selector and one-click escape code quick insertion buttons (`\c[n]`, `\v[n]`, `\n[n]`, `\.`, `\|`, `\!`, `\$`, `\>`, `\<`).
    - **Show Choices (`10140`)**: 4 distinct Choice text boxes with dedicated Cancel behavior selection (Disallow, Choice 1..4, or Branch Case).
    - **Message Options (`10120`)**, **Input Number (`10150`)**.
  - **Progression & Variables**: Control Switches, Control Variables (with all operands & operations), Gold, Inventory, Party, Key Input Processing.
  - **Character & Equipment**: EXP, Levels, Stats, Skills, Equipment Slot changes, HP/SP, Conditions, Full Recovery, Hero Name/Title/Graphic/Face changes.
  - **Movement & World**: Teleport, Memorize Location, Recall Location, Board/Exit Vehicle, Set Vehicle Location, Set Event Location, Set Move Route, Wait.
  - **Audio & Visual**:
    - **Tint Screen (`11030`)**: Real-time live composite color preview swatch as Red, Green, Blue, and Chroma are adjusted.
    - **Flash Screen (`11040`)**: Real-time live flash color swatch with alpha/power intensity preview and duration readout.
    - **Weather Effects (`11070`)**: Weather effect type picker (Rain, Snow, Sandstorm) and low/medium/high severity controls.
    - **Show Picture (`11110`) & Move Picture (`11120`)**: Full parameter suite including Picture # (1..50), coordinate positioning with "Pin to Map" toggle, Magnification % (10..400), Transparency % (0..100), RGB color tint adjustments, Chroma, Rotation/Wave effect modes with speed control, and move duration timing with "Wait for Completion" synchronization.
    - Play BGM, Fade Out BGM, Play Sound Effect, Screen Transitions, Screen Shake, Erase Picture, Battle Animations, Map Chipset Switching, Parallax Backgrounds.
  - **Flow & Logic**: Conditional Branches (Switches, Variables, Gold, Items, Actors), Loops, Break Loop, Exit Event, Erase Event, Common Event Calls, Comments.
  - **Scenes & System Access**:
    - **Shop Processing (`10720`)**: Shop transaction mode selector and interactive Goods inventory list manager (`➕ Add Item`, `🗑 Remove`).
    - **Inn Processing (`10730`)**, **Hero Name Input (`10740`)**, Save Menu, Main Menu, Game Over, Return to Title, Access Permission Locks.
  - **Maniac Patch Extensions**: Native support for extended 2003 Maniac commands with descriptive parameter hints.

---

### 2.7 Developer & Power-User Tools

1. **XML Import & Export (LibLCF Compatible)**:
   - Export any project component (`.ldb`, `.lmt`, `.lmu`, `.lsd`) to human-readable XML.
   - Edit XML files in external text editors or version control systems (Git) and import them back with binary roundtrip fidelity.
2. **Project Health Analyzer**:
   - Scans the entire project for missing assets across `CharSet`, `ChipSet`, `Battle`, `Sound`, and `Music` folders (checking both project folder and RTP).
   - **Broken Reference Checker**: Detects out-of-range switches, variables, items, actors, common events, invalid teleport destinations, and duplicate Event IDs across all map and common events.
   - Detects **Maniac Patch** extensions and lists every extended command used across maps and common events.
3. **Project-Wide Global Search (`Ctrl+F`)**:
   - Search across all maps, events, dialog messages, switches, and variables in one unified search dialog.
4. **Master Database Toolbar & Dynamic Data Rebuilding**:
   - `➕ Add`, `📄 Duplicate`, `🗑 Delete` (with automatic sequential ID re-indexing), `🔍 Filter...`, and `📏 Max...` batch array capacity resizing across all 15 database categories.
   - **Dynamic Vector Synchronization**: Automatically preserves custom entries added beyond initial project sizes and prunes deleted records cleanly across all 11 array categories in `RPG_RT.ldb`.
   - **Integrated System Tab Save Toolbar**: Full dirty state tracking and direct database toolbar saving on the System configuration tab.
   - **Resistance Matrix Mass Presets**: One-click `Set All [A]..[E]` rank presets across States and Element Resistances on Enemies, Actors, and Classes.

---

### 2.8 Graphics & Asset Decoding
- **Selective 8-Bit Palette Transparency**:
  - Classic RPG Maker uses palette index 0 as transparent for 8-bit indexed BMP and XYZ sprites (`CharSet`, `ChipSet`, `Monster`, `FaceSet`, `BattleCharSet`).
  - EasyRPG REditor features custom decoders for indexed PNG, 8-bit BMP, and XYZ that strictly preserve palette index 0 transparency for sprites while keeping palette index 0 solid opaque for full-screen background graphics (`Panorama`, `Title`, `GameOver`, `Battle`, `Battle2`), preventing unwanted alpha holes in dark backgrounds.
- **Resource Manager**:
  - Browse project assets with checkerboard transparency backgrounds, import new files, export assets, and preview animations.

---

## 3. Supported RPG Maker File Formats

| Format | Extension | Description | Supported Features |
|---|---|---|---|
| **LDB** | `RPG_RT.ldb` | Main Game Database | Actors, Classes, Skills, Items, Enemies, Troops, Terrains, Attributes, States, Animations, Chipsets, Common Events, System, Terms |
| **LMT** | `RPG_RT.lmt` | Map Tree & Hierarchy | Map tree hierarchy, indentation, encounter lists, BGM, Battle Backdrops, Teleport/Escape flags |
| **LMU** | `MapXXXX.lmu` | Map Layer & Event Data | Lower layer, Upper layer, Event pages, Event commands, Move routes, Trigger preconditions |
| **LSD** | `SaveXX.lsd` | Save Game State | Party members, Map location, Gold, Inventory, Switches, Variables |

---

## 4. Internationalization & Customization

- **8 Supported Languages**: English (`en`), Spanish (`es`), French (`fr`), German (`de`), Italian (`it`), Japanese (`ja`), Portuguese (`pt-BR`), Simplified Chinese (`zh-CN`), with full multi-language coverage across menus, toolbars, database categories, sound test jukebox, SoundFont synthesizer, XML import/export, and project health analysis dialogs.
- **12 Visual Themes**: Dark, Light, Slate, Amber, Nord, Dracula, Tokyo Night, Gruvbox, Solarized, Cyberpunk, and Classic RPG Maker Gray.
