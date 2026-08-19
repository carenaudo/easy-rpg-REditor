# EasyRPG REditor

An experimental, unofficial map/database editor for RPG Maker 2000 and RPG Maker 2003
projects (the `.ldb`/`.lmt`/`.lmu`/`.lsd` "LCF" format), written in Rust.

## What this is — and what it isn't

This is a **fan-made, hobbyist project**, built almost entirely through "vibe coding" —
iterating with an AI coding assistant rather than hand-writing every line — as an
experiment in how far that approach can go on a real, file-format-accurate desktop tool.
It is **not** an official [EasyRPG](https://easyrpg.org/) project, is **not affiliated
with, endorsed by, or supported by the EasyRPG team**, and it is **not trying to replace
[EasyRPG Editor](https://github.com/EasyRPG/Editor)**, the project's own actively
maintained, professionally engineered editor. If you want a mature, well-tested tool for
your projects, use theirs.

What this project *does* build on, with real gratitude, is EasyRPG's open documentation
of the LCF format and their [`liblcf`](https://github.com/EasyRPG/liblcf) reference
implementation, which [`lcf-core`](crates/lcf-core) (this repo's own pure-Rust
reimplementation of the format) was built and tested against. [`EasyRPG/RTP`](https://github.com/EasyRPG/RTP),
their free-licensed replacement runtime package, is what this editor's Resource Manager
and audio preview are designed to browse — that repository is separate from this one and
not bundled here.

**Use at your own risk, and always keep your own backups or version control of any
project you open with this.** Every write does create a one-time `.bak` copy of the file
it's about to touch before the first save in a session, but that is a safety net for this
tool's own mistakes, not a substitute for a real backup.

## Features

- **Maps**: tile painting (pen, rectangle, ellipse, flood fill, eyedropper), a live
  tileset palette, passability/grid overlays, undo/redo, a searchable map tree with
  create/duplicate/delete/properties, and a "Shift Map" tool for repositioning an entire
  map's contents.
- **Events**: create, edit, delete, cut/copy/paste, and drag-to-reposition directly on
  the canvas; a categorized event-command editor (messages, switches/variables,
  character & stats, movement, audio/screen, flow control, scenes & system); multi-page
  events; and one-click templates for common patterns (door, treasure chest, inn, map
  transfer, save point, recovery spring).
- **Database**: Actors, Classes, Items, Skills, Attributes, Enemies, Troops, Common
  Events, Switches, Variables, Chipsets, States, Terrains, Animations, System settings,
  and Terms/Vocabulary (a curated subset of the ~150 real fields — see Limitations).
- **Save files** (`.lsd`): edit hero name/level/gold/map position and party member
  stats.
- **Resource Manager**: browse, import, export, and delete project assets, with image
  previews and in-app audio preview (WAV/MP3/OGG-Vorbis, plus MIDI via the Windows
  system synth — see Limitations).
- **XML export** of the database, map tree, an individual map, or a save file, for
  inspecting the raw structure (import is not implemented).
- A basic project health analyzer, project-wide search, a sound test/jukebox, RTP
  path auto-detection, recent-projects/last-project restore, 8 UI languages, and a
  dozen color themes across light/dark/system modes.
- Everything that writes to disk tracks its own unsaved-changes state independently,
  and the editor will not silently discard your work — switching maps, opening a
  different project, or closing the window with unsaved changes prompts you to save,
  discard, or cancel.

## Limitations

- **This has not been tested anywhere near as broadly as EasyRPG's own tools.** It has
  been exercised against a handful of real and synthetic test projects during
  development, not the enormous variety of real-world RPG Maker games that exist. Bugs
  that corrupt data on write are a real, live risk category for a project at this stage.
- **Terms/Vocabulary** exposes roughly 35 of the format's ~150 real string fields — the
  core UI vocabulary (menu commands, stat labels), not the full set of battle-message
  and shop-dialogue variants.
- **XML is export-only.** There is no way to import an XML file back into a project.
- **MIDI playback is Windows-only**, via the Microsoft GS Wavetable Synth (the same
  device the original RPG Maker 2000/2003 games were authored against) — there's no
  cross-platform MIDI synthesizer bundled, and no tempo/pitch control for MIDI tracks
  specifically (only volume).
- **This editor is Windows-only in practice** — it hasn't been built, run, or tested on
  Linux or macOS, even though most of its dependencies are cross-platform.
- No battle-test simulation, no scripting/plugin system, and no equivalent of engine
  extensions like Maniac Patch's scripting conventions (files following that pattern are
  simply treated as inert data, not specially interpreted).

## Building from source

Requires a recent stable Rust toolchain ([rustup.rs](https://rustup.rs)) and, on
Windows, the Visual Studio Build Tools (for the MSVC linker).

```bash
cargo build --release
cargo run --release
```

Point the editor at an existing RPG Maker 2000/2003 project folder (one containing
`RPG_RT.ldb`) via **Project → Open Project**, or create a new one from a blank template
via **Project → New Project**. Configure an RTP folder (an [EasyRPG/RTP](https://github.com/EasyRPG/RTP)
checkout, or an official RTP install) via the 📦 button in the toolbar if you want
chipset/graphic previews to resolve assets a project doesn't ship itself.

## License

This project is licensed under the [MIT License](LICENSE) — see that file for the
full text. In short: you can do essentially anything with the code, provided the
copyright notice is preserved, and **it is provided with absolutely no warranty of
any kind** — use it entirely at your own risk.

Third-party Rust crates this project depends on are each under their own license
(overwhelmingly MIT and/or Apache-2.0); see [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md)
for the full list and license texts. RPG Maker 2000/2003 project data, RTP assets, and
any game content you open with this editor remain the property of their own respective
rights holders and are entirely outside the scope of this project's license.
