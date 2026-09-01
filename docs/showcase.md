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
In **Tiles Mode** (Lower Layer `F5`, Upper Layer `F6`), you can paint tiles onto the map canvas with undo/redo support (`Ctrl+Z` / `Ctrl+Y`), similar to the original RPG Maker editor. 
* **Drawing Tools**: Pen, Rectangle, Ellipse, Flood Fill, and Eyedropper (`Alt+Click` or Middle-Click).
* **Display Overlays**: Optional grid lines, passability markers, inactive layer dimming, and event markers for enhanced visibility.

![Map Editor - Tiles Mode](images/mainwindow-tiles.png)

---

### Event Editing Mode (F7)
In **Events Mode** (`F7`), map events are displayed with their character graphic (from CharSet) or event ID badge.
* Double-clicking an event opens the Event Editor.
* Double-clicking an empty tile creates a new event at that coordinate.

![Map Editor - Events Mode](images/mainwindow-events.png)

---

### Right-Click Context Menu & Quick Templates
Right-clicking on the map canvas while in **Events Mode** opens a context menu:
* **Empty Tile**: Options for *New Event Here*, *Paste Event*, *Set Starting Position (Party / Boat)*, and **Quick Event Templates** (Doors with map transfers, Treasure Chests with item/gold rewards, Inns, Map Transitions, Save Points, and Recovery Springs).
* **Occupied Tile**: Options for *Edit Event*, *Delete*, *Cut*, and *Copy*. The editor respects existing events on the same tile.

![Map Right-Click Menu](images/map-rightbutton.png)

---

### Drag-and-Drop Event Repositioning
Events can be repositioned by dragging them with the primary mouse button.
* Position feedback coordinates are shown during dragging.
* **Collision Handling**: If dropped onto a tile with another event, the event returns to its original position to maintain proper spacing.

![Map Drag and Drop Events](images/map-drag-events.png)

---

### Map Tree & Hierarchy
The left sidebar displays your maps in a hierarchical tree structure. Right-clicking any map item offers options to create child maps, duplicate maps, adjust map order, edit properties, or delete maps.

![Map Tree Context Menu](images/main-maps.png)

---

### Map Properties
Edit map dimensions, select chipsets, configure scroll behavior, set background music (BGM) and parallax backgrounds, and adjust encounter rates and monster troop formations.

![Map Properties](images/main-properties.png)

---

## Database Suite & Engine Compatibility (2000 vs 2003)

The Database (`F9`) provides access to your game's data, with **Adaptive Engine Visibility** that adjusts available tabs and fields based on your project's engine version.

### Adaptive Engine Modes (2000 vs 2003)
The editor detects your project's engine version (**RPG Maker 2000** or **RPG Maker 2003**) and adjusts the interface:
* In **RPG Maker 2000** mode, 2003-only features (such as *Classes*, Actor Special Combat Traits, Battler Animation IDs, Skill SP percentages, and System2 graphics) are not displayed.
* In **RPG Maker 2003** mode, additional options like side-view battle parameters, classes, and battle commands are available.

| RPG Maker 2000 Mode | RPG Maker 2003 Mode |
| :---: | :---: |
| ![Database - 2000 Mode](images/db-2000-mode.png) | ![Database - 2003 Mode](images/db-2003-mode.png) |

---

### Actors & Classes
Configure party members with initial/max levels, CharSet and FaceSet previews, starting equipment, and parameter growth curves (Max HP, Max SP, Attack, Defense, Spirit, Agility). In RPG Maker 2003 mode, you can also assign classes, battle commands, dual-wielding, and combat traits.

| Actor General Settings | Actor Parameter Growth Curves |
| :---: | :---: |
| ![Database - Actors General](images/db-actors1.png) | ![Database - Actor Stats](images/db-actors2.png) |

| RPG Maker 2003 Classes |
| :---: |
| ![Database - Classes](images/db-classes.png) |

---

### Skills & Items
Create offensive magic, restorative skills, consumables, weapons, and armor with cost calculations (flat SP or % Max SP in 2003), range options, animation effects, and attribute adjustments.

| Skills | Items Overview |
| :---: | :---: |
| ![Database - Skills](images/db-skills.png) | ![Database - Items Overview](images/db-items-1.png) |

| Equipment & Parameter Modifiers | Consumables & Medicine |
| :---: | :---: |
| ![Database - Equipment Items](images/db-item-2.png) | ![Database - Medicine Items](images/db-item-3.png) |

---

### Enemies & Battle Troops
Configure monster stats, gold/experience payouts, item drops, and action conditions (turn counts, HP thresholds, switch triggers). Organize enemies into Troops with battle background positioning and page-based battle events.

| Enemies | Battle Troops & Formations |
| :---: | :---: |
| ![Database - Enemies](images/db-enemies.png) | ![Database - Troops](images/db-troops.png) |

---

### Attributes & Status Effects
Adjust elemental and weapon damage multipliers (Rank A through E) and configure status ailments with duration, stat effects, messages, and recovery conditions.

| Attribute Elements | States & Status Effects |
| :---: | :---: |
| ![Database - Attributes](images/db-attributes.png) | ![Database - States](images/db-states.png) |

---

### Battle Animations
Create frame-by-frame visual effects from sprite sheets with preview playback. Position animation cells, adjust scale and flash effects, and align sound effects with frames.

![Database - Battle Animations](images/db-animations.png)

---

### Chipsets & Passability
Select tileset graphics (PNG, BMP, and XYZ formats supported), assign terrain IDs, and set passability flags (Allow, Block, Star/Overlay, Counter) and directional blocks (Up, Down, Left, Right) for each layer.

| Chipset Graphics & Terrain | Basic Passability Flags | Directional Block Flags |
| :---: | :---: | :---: |
| ![Chipset Graphics](images/db-chipset-1.png) | ![Chipset Passability](images/db-chipset-2.png) | ![Chipset Directional](images/db-chipset-3.png) |

---

### Terrain Settings
Set terrain properties such as movement damage, encounter rates, backgrounds, and vehicle passability (Boat, Ship, Airship).

![Database - Terrain](images/db-terrain.png)

---

### Global Switches & Variables
Manage global flags and numeric variables with naming and batch search/range tools.

| Global Switches | Global Variables |
| :---: | :---: |
| ![Database - Switches](images/db-switches.png) | ![Database - Variables](images/db-variables.png) |

---

### System Settings & Terms
Set starting party members, windowskins, vehicle graphics, screen transitions, and sound/music settings. Customize UI text and vocabulary (153 fields).

| System Settings | Terms & Vocabulary |
| :---: | :---: |
| ![Database - System](images/db-system.png) | ![Database - Terms](images/db-terms.png) |

---

## Event Creation & Scripting

### Map Event Editor
Edit multi-page map events with customizable conditions (switches, variables, items), movement patterns and frequencies, trigger types (Action Button, Player Touch, Event Touch, Auto Start, Parallel Process), and event commands.

![Event Editor](images/event-editor.png)

---

### Common Events
Create global events, parallel processes, and reusable routines with switch triggers and conditions in the database.

![Common Events](images/db-events.png)

---

## Project Tools, Audio & Utilities

### Resource Manager & Asset Viewer
Manage project assets (CharSet, ChipSet, FaceSet, Battle, Title, GameOver, Panorama, Monster, Music, Sound, and System folders) with import, export, and delete options. Preview images with transparency checkerboards and preview audio files.

![Resource Manager](images/menu-assets.png)

---

### Sound Test & Jukebox
Preview background music (BGM), background sounds (BGS), and sound effects (SE) with playback controls, volume adjustment, pitch options, and loop testing.

![Sound Test Jukebox](images/jukebox.png)

---

### MIDI & Audio Configuration
Configure MIDI playback with the `rustysynth` software synthesizer, supporting standard SoundFont files (`.sf2`). The editor can detect SoundFonts in system and RTP directories, or you can select custom files for RPG Maker 2000/2003 audio playback on Windows, Linux, and macOS.

![MIDI Configuration](images/midi-config.png)

---

### XML Import/Export Utility
Export Database (`RPG_RT.ldb`), Map Tree (`RPG_RT.lmt`), Maps (`MapXXXX.lmu`), and Save Files (`SaveXX.lsd`) to human-readable XML for review, comparison, and analysis. Import edited XML files back to update project files (with automatic backup).

![XML Export Tool](images/xml-export.png)

---

### Project Health Analyzer
Check your project for broken references, missing assets, invalid switch/variable references, and unreachable maps. Also detects Maniac Patch indicators (patch flags, Maniac variables, extended Terms, or special common events) and lists Maniac commands found in maps, common events, and troop battles with their locations.

![Project Health Analyzer](images/health.png)

---

### New Project Creation
Create new projects using starter templates for RPG Maker 2000 or RPG Maker 2003.

![New Project Creation](images/new-game.png)

---

### RTP Path Configuration
Set or auto-detect the EasyRPG RTP (or standard RPG Maker 2000/2003 RTP) location so assets and audio are found properly.

![RTP Configuration](images/main-rtp.png)

---

### Themes & Internationalization
Customize the interface with various color themes (high-contrast, dark, light, and custom options) and choose from 8 available UI languages.

| Color Themes | Language Selector |
| :---: | :---: |
| ![Theme Selector](images/main-theme.png) | ![Language Switcher](images/main-language.png) |

