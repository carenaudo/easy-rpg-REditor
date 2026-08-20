# EasyRPG REditor — Visual Showcase & User Guide

A visual tour and user guide for **EasyRPG REditor**, a desktop-native map and database editor for RPG Maker 2000 and RPG Maker 2003 projects (`.ldb`, `.lmt`, `.lmu`, `.lsd`), built in Rust with `egui`.

---

## Table of Contents
- [Main Workspace & Map Editor](#main-workspace--map-editor)
  - [Tile Editing Mode (F5 / F6)](#tile-editing-mode-f5--f6)
  - [Event Editing Mode (F7)](#event-editing-mode-f7)
  - [Right-Click Context Menu & Quick Templates](#right-click-context-menu--quick-templates)
  - [Drag-and-Drop Event Repositioning](#drag-and-drop-event-repositioning)
  - [Map Tree & Hierarchy](#map-tree--hierarchy)
  - [Map Properties](#map-properties)
- [Database Suite & Engine Compatibility (2000 vs 2003)](#database-suite--engine-compatibility-2000-vs-2003)
  - [Adaptive Engine Modes (2000 vs 2003)](#adaptive-engine-modes-2000-vs-2003)
  - [Actors & Classes](#actors--classes)
  - [Skills & Items](#skills--items)
  - [Enemies & Battle Troops](#enemies--battle-troops)
  - [Attributes & Status Effects](#attributes--status-effects)
  - [Battle Animations](#battle-animations)
  - [Chipsets & Passability](#chipsets--passability)
  - [Terrain Settings](#terrain-settings)
  - [Global Switches & Variables](#global-switches--variables)
  - [System Settings & Terms](#system-settings--terms)
- [Event Creation & Scripting](#event-creation--scripting)
  - [Map Event Editor](#map-event-editor)
  - [Common Events](#common-events)
- [Project Tools, Audio & Utilities](#project-tools-audio--utilities)
  - [Resource Manager & Asset Viewer](#resource-manager--asset-viewer)
  - [Sound Test & Jukebox](#sound-test--jukebox)
  - [MIDI & Audio Configuration](#midi--audio-configuration)
  - [XML Export Utility](#xml-export-utility)
  - [Project Health Analyzer](#project-health-analyzer)
  - [New Project Creation](#new-project-creation)
  - [RTP Path Configuration](#rtp-path-configuration)
  - [Themes & Internationalization](#themes--internationalization)

---

## Main Workspace & Map Editor

The main workspace provides a clean, responsive layout with a top menu bar, quick-action toolbar, collapsible map tree on the left, live tileset palette, and a scrollable, zoomable map canvas with high-performance rendering.

### Tile Editing Mode (F5 / F6)
In **Tiles Mode** (Lower Layer `F5`, Upper Layer `F6`), you can paint tiles directly onto the map canvas with full undo/redo (`Ctrl+Z` / `Ctrl+Y`). 
* **Drawing Tools**: Pen, Rectangle, Ellipse, Flood Fill, and Eyedropper (`Alt+Click` or Middle-Click).
* **Display Overlays**: Toggle grid lines, passability markers, inactive layer dimming, and event markers.

![Map Editor - Tiles Mode](images/mainwindow-tiles.png)

---

### Event Editing Mode (F7)
In **Events Mode** (`F7`), all map events are rendered with their active character graphic (from CharSet) or event ID badge.
* Double-clicking an event opens the Event Editor.
* Double-clicking an empty tile creates a new event at that coordinate.

![Map Editor - Events Mode](images/mainwindow-events.png)

---

### Right-Click Context Menu & Quick Templates
Right-clicking anywhere on the map canvas while in **Events Mode** opens a context menu:
* **Empty Tile**: Options for *New Event Here*, *Paste Event*, *Set Starting Position (Party / Boat)*, and **Quick Event Templates** (Doors with automatic map transfers, Treasure Chests with item/gold rewards, Inns, Map Transitions, Save Points, and Recovery Springs).
* **Occupied Tile**: Direct actions for *Edit Event*, *Delete*, *Cut*, and *Copy*. Prevents creating overlapping events on the same tile.

![Map Right-Click Menu](images/map-rightbutton.png)

---

### Drag-and-Drop Event Repositioning
You can directly grab any event with the primary mouse button and drag it to a new location.
* Live position feedback coordinates are displayed.
* **Collision Safety**: If dropped onto a tile already occupied by another event, it automatically snaps back to its original coordinates to prevent stacking.

![Map Drag and Drop Events](images/map-drag-events.png)

---

### Map Tree & Hierarchy
The left sidebar organizes your maps in a hierarchical tree. Right-clicking any map item reveals options to create child maps, duplicate existing maps, shift map layers, edit properties, or delete maps.

![Map Tree Context Menu](images/main-maps.png)

---

### Map Properties
Configure map dimensions, assign chipsets, define scroll wrapping, set custom background music (BGM) and background parallax images, and specify random encounter step rates along with monster troop formation tables.

![Map Properties](images/main-properties.png)

---

## Database Suite & Engine Compatibility (2000 vs 2003)

The Database (`F9`) provides access to every component of your game's data structure, featuring **Adaptive Engine Visibility** that tailors available tabs and fields to your project's engine version.

### Adaptive Engine Modes (2000 vs 2003)
The editor automatically detects whether a project is **RPG Maker 2000** or **RPG Maker 2003** and adapts the UI:
* In **RPG Maker 2000** mode, 2003-only features (such as the *Classes* category, Actor Special Combat Traits, Battler Animation IDs, Skill SP percentages, and System2 graphics) are hidden to prevent saving incompatible data.
* In **RPG Maker 2003** mode, full side-view battle parameters, classes, and battle commands are enabled.

| RPG Maker 2000 Mode | RPG Maker 2003 Mode |
| :---: | :---: |
| ![Database - 2000 Mode](images/db-2000-mode.png) | ![Database - 2003 Mode](images/db-2003-mode.png) |

---

### Actors & Classes
Customize party members with initial/max levels, animated CharSet previews, FaceSet previews, starting equipment, and interactive parameter stat growth curves (Max HP, Max SP, Attack, Defense, Spirit, Agility). In RPG Maker 2003 mode, configure hero classes, battle commands, dual-wielding, and combat traits.

| Actor General Settings | Actor Parameter Growth Curves |
| :---: | :---: |
| ![Database - Actors General](images/db-actors1.png) | ![Database - Actor Stats](images/db-actors2.png) |

| RPG Maker 2003 Classes |
| :---: |
| ![Database - Classes](images/db-classes.png) |

---

### Skills & Items
Define offensive magic, restorative skills, consumables, weapons, and armor with custom cost calculations (flat SP or % Max SP in 2003), range scopes, animation links, and attribute modifiers.

| Skills | Items Overview |
| :---: | :---: |
| ![Database - Skills](images/db-skills.png) | ![Database - Items Overview](images/db-items-1.png) |

| Equipment & Parameter Modifiers | Consumables & Medicine |
| :---: | :---: |
| ![Database - Equipment Items](images/db-item-2.png) | ![Database - Medicine Items](images/db-item-3.png) |

---

### Enemies & Battle Troops
Set up monster stats, gold/experience payouts, item drops, and behavioral action conditions (turn counts, HP thresholds, switch triggers). Assemble enemies into Troops with custom battle background positioning and page-based battle events.

| Enemies | Battle Troops & Formations |
| :---: | :---: |
| ![Database - Enemies](images/db-enemies.png) | ![Database - Troops](images/db-troops.png) |

---

### Attributes & Status Effects
Fine-tune elemental/weapon damage multipliers (Rank A through E) and status ailments with turn-based duration, stat penalties, message text, and recovery conditions.

| Attribute Elements | States & Status Effects |
| :---: | :---: |
| ![Database - Attributes](images/db-attributes.png) | ![Database - States](images/db-states.png) |

---

### Battle Animations
Build frame-by-frame visual effects using sprite sheets with live preview playback. Position animation cells, configure scale and flash effects, and synchronize sound effects with frame playback.

![Database - Battle Animations](images/db-animations.png)

---

### Chipsets & Passability
Configure tileset graphics (supporting PNG, BMP, and XYZ formats), assign terrain IDs, and define passability flags (Allow, Block, Star/Overlay, Counter) as well as directional passage blocks (Up, Down, Left, Right) for both lower and upper layers.

| Chipset Graphics & Terrain | Basic Passability Flags | Directional Block Flags |
| :---: | :---: | :---: |
| ![Chipset Graphics](images/db-chipset-1.png) | ![Chipset Passability](images/db-chipset-2.png) | ![Chipset Directional](images/db-chipset-3.png) |

---

### Terrain Settings
Configure terrain attributes including movement damage, encounter multipliers, special backgrounds, and vehicle passability (Boat, Ship, Airship).

![Database - Terrain](images/db-terrain.png)

---

### Global Switches & Variables
Organize global flags and numeric variables with quick naming and batch search/range management.

| Global Switches | Global Variables |
| :---: | :---: |
| ![Database - Switches](images/db-switches.png) | ![Database - Variables](images/db-variables.png) |

---

### System Settings & Terms
Configure starting party members, system windowskins, vehicle sprites, screen transitions, sound effects (SE), and background music (BGM). Customize standard UI labels and system vocabulary (full 153-field coverage).

| System Settings | Terms & Vocabulary |
| :---: | :---: |
| ![Database - System](images/db-system.png) | ![Database - Terms](images/db-terms.png) |

---

## Event Creation & Scripting

### Map Event Editor
The Event Editor gives full control over multi-page map events. Each page supports custom starting conditions (switches, variables, items), movement patterns and frequencies, trigger types (Action Button, Player Touch, Event Touch, Auto Start, Parallel Process), and a complete event command list.

![Event Editor](images/event-editor.png)

---

### Common Events
Manage global background scripts, parallel processes, and reusable subroutines with switch triggers and conditions directly within the database.

![Common Events](images/db-events.png)

---

## Project Tools, Audio & Utilities

### Resource Manager & Asset Viewer
Browse, import, export, and delete project assets across CharSet, ChipSet, FaceSet, Battle, Title, GameOver, Panorama, Monster, Music, Sound, and System folders. Features live image previews with transparency checkerboards and in-app audio playback.

![Resource Manager](images/menu-assets.png)

---

### Sound Test & Jukebox
Test background music (BGM), background sounds (BGS), and sound effects (SE) with playback controls, volume sliders, pitch modulation, and loop testing.

![Sound Test Jukebox](images/jukebox.png)

---

### MIDI & Audio Configuration
Configure the cross-platform MIDI software synthesizer powered by `rustysynth`. The editor can auto-detect standard SoundFonts (`.sf2` files) in system and RTP directories or let you load custom soundfont files for authentic RPG Maker 2000/2003 soundtrack reproduction across Windows, Linux, and macOS.

![MIDI Configuration](images/midi-config.png)

---

### XML Export Utility
Export the Database (`RPG_RT.ldb`), Map Tree (`RPG_RT.lmt`), individual Maps (`MapXXXX.lmu`), or Save Files (`SaveXX.lsd`) into human-readable XML for debugging, diffing, and inspecting project internals.

![XML Export Tool](images/xml-export.png)

---

### Project Health Analyzer
Scan your project for broken references, missing asset files, invalid switch/variable references, and unreachable maps to identify integrity issues before playtesting.

![Project Health Analyzer](images/health.png)

---

### New Project Creation
Quickly initialize a new project scaffold from built-in starter templates for RPG Maker 2000 or RPG Maker 2003.

![New Project Creation](images/new-game.png)

---

### RTP Path Configuration
Auto-detect or configure the location of the EasyRPG RTP (or standard RPG Maker 2000/2003 RTP) so default assets, chipsets, and audio are resolved seamlessly.

![RTP Configuration](images/main-rtp.png)

---

### Themes & Internationalization
Personalize the editor with high-contrast, dark, light, and custom color themes, or switch between 8 available UI languages.

| Color Themes | Language Selector |
| :---: | :---: |
| ![Theme Selector](images/main-theme.png) | ![Language Switcher](images/main-language.png) |

