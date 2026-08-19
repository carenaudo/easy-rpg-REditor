# Scoping a full pure-Rust liblcf port (no C/C++ dependencies)

Written while fighting vcpkg/ICU to get the C++ liblcf building for REditor.
This document scopes what a **full native-Rust replacement of liblcf**
would take — not a minimal viewer subset — grounded in reading liblcf's
real source (not guessing).

## Scope note

This is now a commitment to a **full port**: read + write, all four liblcf
file formats, EasyRPG's format extensions included, via a standalone
Rust-native codegen tool. Every previously-open question is now decided:

| Question | Decision |
|---|---|
| XML support | **In scope** — needed |
| File formats | **All four**: Database (`.ldb`), MapTree (`.lmt`), Map (`.lmu`), SaveData (`.lsd`) |
| EasyRPG extensions | **In scope** — `structs_easyrpg.csv`/`fields_easyrpg.csv`/`enums_easyrpg.csv`/`flags_easyrpg.csv` included alongside the base RPG Maker schema |
| Codegen implementation | **Standalone Rust codegen crate** reading `generator/csv/*.csv` directly and emitting `.rs` — not liblcf's existing Python/Jinja2 pipeline |

This is the full data model: with the `_easyrpg` extensions folded in,
that's **70 structs** (38 ldb + 5 lmt + 5 lmu + 17 lsd from the base
schema, plus 4 more from `structs_easyrpg.csv`: `StringVariable` (ldb),
`SaveEasyRpgData`/`SaveEasyRpgWindow`/`SaveEasyRpgText` (lsd)), roughly
**1,200 fields** total (1,044 base + 160 easyrpg), plus XML read/write for
all of them. This changes the shape of the plan from "hand-port a few
structs, maybe build codegen later" to "validate the core primitives on a
small slice, then generate everything else" — see "Recommended sequence"
below.

## TL;DR

1. The binary LCF format is self-describing at the chunk level (every field
   is `[chunk_id][length][payload]`, unknown chunks are skippable) — this
   makes the core reader/writer primitives small and independently
   testable, which matters because...
2. **A full port means porting `PersistIfDefault`/`Is2k3` write semantics
   for ~1,200 fields across 70 structs** (base schema + EasyRPG
   extensions), not just a handful. Doing that by hand is exactly the
   repetitive, error-prone work liblcf's own `generator/csv/*.csv` schema
   exists to make mechanical — so a standalone Rust codegen crate reading
   those CSVs directly is the right architecture, not hand-writing 70
   structs by hand.
3. Don't build the codegen *first*, though: build and validate the core
   `LcfReader`/`LcfWriter` primitives (varint encode/decode, chunk loop,
   default-value comparison, string encoding) against 2-3 real structs by
   hand first, with round-trip byte-identity tests passing on real game
   files. Codegen built on unvalidated primitives compounds one bug across
   all 70 structs simultaneously instead of catching it early on one.
4. XML read/write (via `quick-xml`, replacing expat) needs the same
   codegen treatment as the binary format — each generated struct needs
   both a binary and an XML `read()`/`write()` pair, since that's how
   liblcf itself works (`reader_xml.cpp`/`writer_xml.cpp` mirror
   `reader_lcf.cpp`/`writer_lcf.cpp`).
5. ICU and inih still have direct Rust replacements (`encoding_rs`, a
   trivial hand-rolled INI parser) — this removes the vcpkg/MSVC-static-
   linking pain we hit today, permanently, once the port is complete.

## Recommended sequence

1. **Bootstrap (hand-written, ~2-3 days):** `LcfReader`/`LcfWriter` core
   (varint, chunk loop, skip) + `TreeMap`/`MapInfo`, `Map` (subset), and
   `Database`'s `Chipset` entries, hand-written. This both unblocks
   REditor's existing read-only viewer immediately *and* serves as the
   proving ground for the primitives codegen will rely on.
   Round-trip byte-identity tests (read → write → byte-diff against the
   original file) must pass here before moving on — this is the point
   where reader/writer bugs are cheapest to find, one struct at a time.
2. **Codegen crate (once step 1 is validated):** a standalone Rust
   build-time tool that reads `generator/csv/{structs,fields,enums,flags}.csv`
   *and* their `_easyrpg` counterparts directly (plain CSV — no Python,
   Jinja2, or pandas dependency needed to consume them) and emits Rust
   structs + binary `read()`/`write()` + XML `read_xml()`/`write_xml()`
   impls, following the exact pattern validated in step 1 — including
   `PersistIfDefault`/`Is2k3` default-value logic driven directly by the
   CSV columns rather than hand-transcribed. This covers all 70 structs
   (66 base + 4 EasyRPG extensions) in one pass.
3. **Wire REditor's data layer onto the generated model**, replacing the
   bootstrap hand-written structs from step 1 (keep them only if the
   codegen output doesn't yet cover 100% of what they did).
4. **Round-trip test the generated code against real project files** before
   trusting any write path in the actual editor UI — same bar as step 1,
   just automated across all structs instead of 2-3.

This sequencing means the "should we build codegen" question isn't a
someday-maybe gated on struct count anymore — it's step 2, scheduled right
after the primitives are proven correct.

## What REditor uses today (starting point for step 1)

Checked `src/lcf_bridge.rs` + `cpp/bridge.cpp` + `src/main.rs`:

- `load_project()` — reads `RPG_RT.lmt`, lists map names from `TreeMap.maps`
- `get_map_chipset()` — reads `MapN.lmu` for `chipset_id`, cross-references
  `RPG_RT.ldb`'s `Database.chipsets` for the chipset's name, loads a PNG

Both are read-only today, and this is exactly the slice step 1 above should
hand-port first — it's already scoped, already working in the current C++
bridge, and touches the same reader machinery (chunk loop, `DBString`,
`Array<T>`, `Ref<T>`) that every other struct also uses.

## How the binary format actually works

Read `src/reader_lcf.cpp` and `src/reader_struct_impl.h` directly:

- Every field is stored as `[chunk_id: varint][length: varint][payload]`.
  `ReadInt()` implements a 7-bit-per-byte varint (top bit = continuation).
- A struct reader loops over chunks by ID. If the ID maps to a known field,
  it decodes the payload by that field's declared type. If the ID is
  unrecognized, the reader just seeks forward by `length` and moves on
  (`stream.Skip(chunk_info, ...)` in `reader_struct_impl.h:92`).

Even for a full port, this property is still useful defensively — it means
step 1's bootstrap reader can safely ignore fields it hasn't ported yet
without corrupting the read of fields around them, which is exactly what
makes incremental, testable rollout (rather than all-or-nothing) possible.

## What `generator/` actually is, and the codegen approach we're taking

- `generator/csv/*.csv` — a hand-maintained schema: `structs.csv` (66
  structs), `fields.csv` (1,044 fields), `enums.csv` (473 enum values),
  `flags.csv`, plus `structs_easyrpg.csv`/`fields_easyrpg.csv`/
  `enums_easyrpg.csv`/`flags_easyrpg.csv` for EasyRPG's own format
  extensions (4 more structs, 160 more fields) — all in scope for this port.
- `generator/templates/*.tmpl` — Jinja2 templates turning the schema into
  the ~19,800 lines of C++ in `src/generated/`. **Not reused** — see below.
- `generator/generate.py` — the driver script (Python 3 + Jinja2 + pandas).
  **Not reused.**

For a full port, the CSV schema itself is a genuine asset, not overhead:
it's the same source of truth liblcf's own C++ is generated from, already
battle-tested against real RPG Maker output for years, including all the
version-conditional defaults (`Is2k3`) and persistence rules
(`PersistIfDefault`) that are the actual hard part of write support.
Consuming it means the Rust port's correctness is bounded by "did we
translate the schema correctly," not "did we manually transcribe ~1,200
fields correctly."

**Decision: a standalone Rust-native codegen crate**, not liblcf's
`generate.py`/Jinja2 pipeline. It reads the CSVs directly (plain CSV
parsing, e.g. via `csv` + `serde`) and emits Rust source, either via
`syn`/`quote` (structured, typed codegen) or plain string templates
(simpler to read/debug, easier to keep close to the hand-written step-1
code it's modeled on). This keeps REditor's own build self-contained —
no Python/Jinja2/pandas dependency for contributors — and decouples the
Rust port's codegen from upstream liblcf's C++ generator entirely, so
changes to one don't risk breaking the other. It also means the codegen
tool lives *in this repo* (REditor), not liblcf's, which matches where
its output is actually consumed.

## Dependency replacement plan

| C dependency | Used for | Rust replacement |
|---|---|---|
| ICU | Text encoding detection/conversion (Shift-JIS, Windows-1252, etc.) | [`encoding_rs`](https://docs.rs/encoding_rs) — what Firefox uses, no C toolchain, covers everything liblcf needs |
| expat | XML variant of LCF files | [`quick-xml`](https://docs.rs/quick-xml) — **confirmed in scope**; every generated struct needs an XML `read`/`write` pair alongside the binary one |
| inih | `.ini` config parsing (e.g. `RPG_RT.ini`) | Trivial hand-rolled parser or a small crate; format is simple key=value with sections |

None of these require `cxx`/FFI or a C++ toolchain — this removes the
entire vcpkg/MSVC-static-linking problem we hit today, permanently, once
the full port lands.

## Write support: the part that determines most of the effort

Read `src/writer_lcf.cpp` and the `IsDefault`/`isPresentIfDefault` machinery
in `src/reader_struct.h:378-512` and `src/reader_struct_impl.h:127,152`
directly — this is the part that actually determines effort, not the varint
encoding (a trivial mirror of the reader, see `WriteInt` in
`writer_lcf.cpp:50`).

**The core rule:** when writing a struct, a field's chunk is emitted only if
either (a) the field is marked `PersistIfDefault=1` in `fields.csv` (348 of
1,044 fields), or (b) its current value differs from its declared default.
Fields equal to their default are omitted — this is what keeps liblcf's
output byte-compatible with what RPG Maker's own editor writes.

Three things make this the hard part of a full port, in increasing order of
pain — and exactly why generating this logic from `fields.csv` beats
hand-transcribing it 1,044 times:

1. **Default values aren't always trivial constants.** Most are (`False`,
   `0`, empty string). Some are version-conditional — `fields.csv`'s
   `Default Value` column has entries like `50|99` for
   `Actor.final_level` (2000 vs 2003 default), keyed by the `Is2k3` column
   and the file's `DatabaseVersion` chunk.
2. **String encoding on write is the reverse of read.** `DBString` fields
   need to be *encoded* back into the project's original 8-bit encoding
   (Shift-JIS, Windows-1252, ...), not just decoded. `encoding_rs` handles
   this fine (it's bidirectional), but the encode path needs wiring in
   wherever `DBString` gets written.
3. **Arrays/indices need to preserve ordering and index semantics** exactly
   as the `Index available?` column in `structs.csv` implies — an
   off-by-one or reordering bug here produces a file RPG Maker will happily
   load with subtly wrong data (e.g. wrong event on the wrong map slot),
   which is a worse failure mode than a read bug (which just shows wrong
   data in REditor, harmlessly, and is easy to spot).

## Testing strategy for writes

Because a write bug can silently corrupt a user's actual RPG Maker project
(unlike a read bug, which just displays something wrong), **every writable
struct — hand-written in step 1 or generated in step 2 — needs a
round-trip byte-identity test before being trusted:**

1. Read a real `.lmu`/`.lmt`/`.ldb`/`.lsd` file with the Rust reader.
2. Write it back out unmodified.
3. Byte-diff the output against the original input.

Any difference must be explainable (liblcf itself doesn't guarantee
byte-identical round-trips in 100% of edge cases either — worth checking
liblcf's own `tests/` suite for precedent), not silently accepted. For the
generated code (step 2), this test should run automatically for every
struct as part of CI, gating any change to the CSV schema or templates —
treat it as a hard requirement given the failure mode is "quietly damages
someone's game project."

## Effort estimate

- **Step 1 — bootstrap + validate primitives (hand-written):**
  Core `LcfReader`/`LcfWriter` (varint, chunk loop, skip, string
  decode/encode) + `TreeMap`/`MapInfo`, `Map` (subset), `Chipset` +
  round-trip test harness: **~3-4 days**, including getting the
  `PersistIfDefault` pattern right on a small, verifiable slice. This also
  unblocks REditor's current viewer immediately. XML isn't validated here
  yet — binary format first, since it's what step 1's real usage (today's
  `bridge.cpp` behavior) actually needs.
- **Step 1b — validate XML on the same slice:** once binary round-trips
  pass, add `read_xml`/`write_xml` for the same handful of structs using
  `quick-xml`, and round-trip test those too (`LoadXml` → `SaveXml` →
  byte-diff). Cheaper to shake out `quick-xml` integration issues on 3
  structs than on 70: **~1-2 days**.
- **Step 2 — codegen crate targeting all 70 structs, binary + XML:**
  The standalone Rust codegen tool (CSV parsing + Rust code emission) +
  wiring `PersistIfDefault`/`Is2k3` logic generically from the CSV columns
  for both serialization formats: **~2-3 weeks**. This is the bulk of the
  full-port effort and the main cost of including XML + EasyRPG extensions
  + save data — but it's a fixed cost that doesn't scale up further even
  as REditor's editing UI grows, since everything comes from one pipeline.
- **Step 3 — wiring + validation:**
  Round-trip tests (binary and XML) across all 70 generated structs against
  real project/save files, replacing the bootstrap hand-written structs:
  **~3-4 days** — a `.lsd` save-file test corpus needs sourcing/generating
  in addition to `.ldb`/`.lmt`/`.lmu` project files, and there's simply
  more surface area to validate than a binary-only, project-data-only port.
- **Step 4 — swap `cxx-build` out entirely:** once REditor's data layer no
  longer needs `bridge.cpp`, drop `cxx`/`cxx-build` and the C++ toolchain
  requirement from `build.rs` completely: **~0.5 day**.
- **Total: roughly 5-6 focused weeks** for a full read/write port (binary
  + XML, all four file formats, EasyRPG extensions included) with no C/C++
  build dependencies afterward. Most of the added cost versus a
  binary-only/project-data-only port is in step 2 (XML doubles the
  per-struct codegen surface) and step 3 (save-file test corpus); step 1's
  bootstrap cost barely changes.

## Practical next step

Nothing is architecturally open anymore — the remaining work is sourcing a
**test corpus**: a handful of real RPG Maker 2000/2003 projects (varied
encodings — at least one Shift-JIS and one Windows-1252 project) plus
actual save files, to round-trip test against from step 1 onward. Worth
lining up before starting step 1, since the round-trip tests are the
correctness gate for every step after.
