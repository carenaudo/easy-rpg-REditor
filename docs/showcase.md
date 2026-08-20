# EasyRPG REditor — Visual Showcase & User Guide

A visual tour and guide for **EasyRPG REditor**, a fast, desktop-native map and database editor for RPG Maker 2000 and RPG Maker 2003 projects (`.ldb`, `.lmt`, `.lmu`, `.lsd`), built in Rust with `egui`.

---

## Table of Contents
- [Main Workspace & Map Editor](#main-workspace--map-editor)
  - [Tile Editing Mode](#tile-editing-mode)
  - [Event Editing Mode](#event-editing-mode)
  - [Map Tree & Hierarchy](#map-tree--hierarchy)
  - [Map Properties](#map-properties)
- [Event Creation & Scripting](#event-creation--scripting)
  - [Map Event Editor](#map-event-editor)
  - [Common Events](#common-events)
- [Database Suite](#database-suite)
  - [Actors & Classes](#actors--classes)
  - [Skills & Items](#skills--items)
  - [Enemies & Battle Troops](#enemies--battle-troops)
  - [Attributes & Status Effects](#attributes--status-effects)
  - [Battle Animations](#battle-animations)
  - [Chipsets & Passability](#chipsets--passability)
  - [Terrain Settings](#terrain-settings)
  - [Global Switches & Variables](#global-switches--variables)
  - [System Settings & Terms](#system-settings--terms)
- [Project Tools & Customization](#project-tools--customization)
  - [Project Health Analyzer](#project-health-analyzer)
  - [New Project Creation](#new-project-creation)
  - [RTP Path Configuration](#rtp-path-configuration)
  - [Themes & Internationalization](#themes--internationalization)

---

## Main Workspace & Map Editor

The main window provides a responsive workspace with a top menu and toolbar, a collapsible map tree on the left, a live tileset palette, and a scrollable/zoomable map canvas with smooth grid rendering.

### Tile Editing Mode
In **Tiles Mode** (`F5`), you can paint tiles directly onto the map canvas with full undo/redo support. Drawing tools include Pen, Rectangle, Ellipse, Flood Fill, and Eyedropper. You can switch between Lower (Layer 1) and Upper (Layer 2) tile layers, toggle the tile grid, passability markers, and preview event placements.

![Map Editor - Tiles Mode](images/mainwindow-tiles.png)

### Event Editing Mode
In **Events Mode** (`F6`), map events are highlighted on the canvas with their configured graphics or ID labels. You can drag and drop events to reposition them, right-click for quick actions (New, Edit, Cut, Copy, Paste, Delete, Templates), or double-click any tile to open the Event Editor.

![Map Editor - Events Mode](images/mainwindow-events.png)

### Map Tree & Hierarchy
The map tree on the left sidebar organizes your maps in a hierarchical tree. Right-clicking a map reveals options to create child maps, duplicate existing maps, edit properties, shift all map contents, or delete maps.

![Map Tree Context Menu](images/main-maps.png)

### Map Properties
Configure map dimensions, assign chipsets, define scroll boundaries, set custom background music (BGM) and background graphics (parallax), and specify random encounter step rates along with monster troop tables.

![Map Properties](images/main-properties.png)

---

## Event Creation & Scripting

### Map Event Editor
The Event Editor gives full control over multi-page map events. Each page supports custom starting conditions (switches, variables, items), movement patterns and frequencies, trigger types (Action Button, Player Touch, Event Touch, Auto Start, Parallel Process), and a complete event command list.

![Event Editor](images/event-editor.png)

### Common Events
Manage global background scripts, parallel processes, and reusable subroutines with switch triggers and conditions directly within the database.

![Common Events](images/db-events.png)

---

## Database Suite

The Database (`F9`) provides access to every component of your game's data structure.

### Actors & Classes
Customize party members with initial/max levels, battle graphics, face graphics, starting equipment, and stat curves (Max HP, Max SP, Attack, Defense, Spirit, Agility). In RPG Maker 2003 mode, define hero classes, battle commands, and class-specific growth tables.

| Actor General Settings | Actor Parameter Curves |
| :---: | :---: |
| ![Database - Actors General](images/db-actors1.png) | ![Database - Actor Stats](images/db-actors2.png) |

| RPG Maker 2003 Classes |
| :---: |
| ![Database - Classes](images/db-classes.png) |

---

### Skills & Items
Define offensive spells, restorative skills, usable items, weapons, and armor with custom cost calculations, range/target scopes, animation links, and attribute modifiers.

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
Build frame-by-frame visual effects using sprite sheets. Position animation cells, configure scale and flash effects, and synchronize sound effects with frame playback.

![Database - Battle Animations](images/db-animations.png)

---

### Chipsets & Passability
Configure tileset graphics, assign terrain IDs, and define passability flags (Allow, Block, Star/Overlay, Counter) as well as directional passage blocks (Up, Down, Left, Right) for both lower and upper layers.

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
Configure default starting party members, system graphics, screen transitions, sound effects (SE), and background music (BGM). Customize standard UI labels and system vocabulary.

| System Settings | Terms & Vocabulary |
| :---: | :---: |
| ![Database - System](images/db-system.png) | ![Database - Terms](images/db-terms.png) |

---

## Project Tools & Customization

### Project Health Analyzer
Scan your project for broken references, missing asset files, invalid switch/variable references, and unreachable maps to identify integrity issues before playtesting.

![Project Health Analyzer](images/health.png)

### New Project Creation
Quickly initialize a new project scaffold from built-in starter templates.

![New Project Creation](images/new-game.png)

### RTP Path Configuration
Auto-detect or configure the location of the EasyRPG RTP (or standard RPG Maker 2000/2003 RTP) so default assets, chipsets, and audio are resolved seamlessly.

![RTP Configuration](images/main-rtp.png)

### Themes & Internationalization
Personalize the look and feel with high-contrast, dark, light, and custom color themes, or switch between 8 available UI languages.

| Color Themes | Language Selector |
| :---: | :---: |
| ![Theme Selector](images/main-theme.png) | ![Language Switcher](images/main-language.png) |
