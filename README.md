# EasyRPG REditor

An experimental, unofficial map/database editor for RPG Maker 2000 and RPG Maker 2003
projects (the `.ldb`/`.lmt`/`.lmu`/`.lsd` "LCF" format), written in Rust.

## What this is — and what it isn't

This is a **fan-made, hobbyist project**, built almost entirely through "vibe coding" — iterating with an AI coding assistant rather than hand-writing every line — as an experiment in how far that approach can go on a real, file-format-accurate desktop tool.
It is **not** an official [EasyRPG](https://easyrpg.org/) project, is **not affiliated with, endorsed by, or supported by the EasyRPG team**, and it is **not trying to replace [EasyRPG Editor](https://github.com/EasyRPG/Editor)**, the project's own actively maintained, professionally engineered editor. If you want a mature, well-tested tool for your projects, use theirs.

What this project *does* build on, with real gratitude, is EasyRPG's open documentation of the LCF format and their [`liblcf`](https://github.com/EasyRPG/liblcf) reference implementation, which [`lcf-core`](crates/lcf-core) (this repo's own pure-Rust reimplementation of the format) was built and tested against. [`EasyRPG/RTP`](https://github.com/EasyRPG/RTP),
their free-licensed replacement runtime package, is what this editor's Resource Manager and audio preview are designed to browse — that repository is separate from this one and not bundled here.

**Use at your own risk, and always keep your own backups or version control of any project you open with this.** Every write does create a one-time `.bak` copy of the file it's about to touch before the first save in a session, but that is a safety net for this tool's own mistakes, not a substitute for a real backup.

> 📸 **Visual Tour & Guide**: See the [**Visual Showcase & User Guide**](docs/showcase.md) for screenshots and walkthroughs of all editor views, tools, and database editors.

## Features

- **Maps**: Tile painting (pen, rectangle, ellipse, flood fill, eyedropper `Alt+Click`), live tileset palette, passability/grid overlays, undo/redo (`Ctrl+Z`/`Ctrl+Y`), searchable map tree with create/duplicate/delete/properties, and a "Shift Map" tool for repositioning an entire map's contents. Supports PNG, BMP, and XYZ graphic formats.
- **Events & Canvas Interactions**: Events mode (`F7`), canvas right-click context menu (New Event, Quick Templates, Party/Boat Starting Positions, Cut/Copy/Paste), drag-and-drop event repositioning with automatic anti-collision snap-back protection, categorized event-command dialog, multi-page events, and one-click quick event templates (doors, treasure chests, inns, transitions, save points, recovery springs).
- **Database & Engine Compatibility**: Full support for Actors, Classes, Items, Skills, Attributes, Enemies, Troops, Common Events, Switches, Variables, Chipsets, States, Terrains, Animations, System settings, and Terms/Vocabulary (153-field coverage). Includes **Adaptive Engine Modes** that automatically tailor the UI to RPG Maker 2000 vs RPG Maker 2003 projects.
- **Save files** (`.lsd`): Edit hero name/level/gold/map position and party member stats.
- **Resource Manager**: Browse, import, export, and delete project assets across all standard folders with live checkerboard image previews and in-app audio playback.
- **Audio & MIDI**: In-app sound test/jukebox and audio preview (WAV, MP3, OGG Vorbis, and cross-platform MIDI synthesis via `rustysynth` with SoundFont `.sf2` support).
- **XML import/export** of the database, map tree, individual maps, or save files, in liblcf-compatible XML - for inspecting raw structures, hand-editing, or round-tripping through external tools.
- Project health analyzer (with Maniac Patch detection), project-wide search (`Ctrl+F`), RTP path auto-detection, recent-projects restore, 8 UI languages, and 12 light/dark color themes.
- Comprehensive unsaved-changes tracking across all views with prompt safeguards.

## Limitations

- **This has not been tested anywhere near as broadly as EasyRPG's own tools.** It has been exercised against a handful of real and synthetic test projects during development, not the enormous variety of real-world RPG Maker games that exist. Bugs that corrupt data on write are a real, live risk category for a project at this stage.
- **MIDI playback requires a SoundFont (`.sf2`)**, rendered in real time via the built-in `rustysynth` software synthesizer. The editor can auto-detect standard SoundFonts (such as `FluidR3_GM.sf2`, `GeneralUser_GS.sf2`, or Windows/system soundfonts) and lets you configure custom `.sf2` paths via the MIDI settings dialog.
- **This editor is developed primarily on Windows** — although its dependencies (Rust, `eframe`/`egui`, `wgpu`, `rodio`, `rustysynth`, `symphonia`) are cross-platform, automated CI and release binaries are currently focused on Windows.
- No battle-test simulation and no scripting/plugin system. Maniac Patch content is **detected** (a badge in the Database view, plus a full command-by-command report in Project Health) but not specially interpreted or editable - its extra event commands, string variables, and other extensions are read/written as inert data, same as everything else.

## Building from source

Requires a recent stable Rust toolchain ([rustup.rs](https://rustup.rs)). On Windows, if you use the default `*-pc-windows-msvc` Rust target, you still need the Visual Studio Build Tools because Rust links through the MSVC linker. If you use a GNU/MinGW Rust target instead, that MSVC requirement can differ.

```bash
cargo build --release
cargo run --release
```

Point the editor at an existing RPG Maker 2000/2003 project folder (one containing `RPG_RT.ldb`) via **Project → Open Project**, or create a new one from a blank template via **Project → New Project**. Configure an RTP folder (an [EasyRPG/RTP](https://github.com/EasyRPG/RTP) checkout, or an official RTP install) via the 📦 button in the toolbar if you want chipset/graphic previews to resolve assets a project doesn't ship itself.

## License

This project is licensed under the [MIT License](LICENSE) — see that file for the full text. In short: you can do essentially anything with the code, provided the copyright notice is preserved, and **it is provided with absolutely no warranty of
any kind** — use it entirely at your own risk.

Third-party Rust crates this project depends on are each under their own license (overwhelmingly MIT and/or Apache-2.0); see [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for the full list and license texts. RPG Maker 2000/2003 project data, RTP assets, and any game content you open with this editor remain the property of their own respective rights holders and are entirely outside the scope of this project's license.
