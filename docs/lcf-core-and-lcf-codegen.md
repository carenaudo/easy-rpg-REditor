# `lcf-core` and `lcf-codegen`

A pure-Rust implementation of the LCF binary formats (`lcf-core`) and the schema generator
that produces it (`lcf-codegen`), with no C/C++ dependency and no external toolchain
required to build.

## Status: alpha

This is alpha software and should be treated as such.

An earlier version of this document opened by claiming "complete 100% API-level feature
parity with C++ `liblcf`." **That claim was wrong and has been removed.** Nobody enumerated
liblcf's API surface and verified it item by item; what actually exists is the subsystem
coverage listed below and the round-trip tests at the end of this page. Those are a real but
narrow result, and they are not a parity measurement.

Concretely, what is *not* established:

- Parity with liblcf's public API has not been measured.
- The round-trip corpus is a handful of real and synthetic projects, not the range of real
  RPG Maker games in the wild.
- Write paths can corrupt project data. Keep backups of anything you open.

## Implemented subsystems

"Implemented" here means present and exercised by the tests below where those tests cover it
— not that every behaviour has been verified against liblcf.

| Subsystem | Components & capabilities |
|---|---|
| **Binary file formats** | **`.ldb` (Database)**, **`.lmt` (MapTree)**, **`.lmu` (Map)**, **`.lsd` (SaveData)** |
| **Data structs** | 70 total: 38 Database + 5 MapTree + 5 MapUnit + 20 SaveData, EasyRPG extensions included |
| **Typed enums** | 73 Rust enums with `repr(i32)` and keyword sanitation |
| **Text encoding** | Multi-codepage translation via `encoding_rs` (Shift-JIS, Windows-1250..1258, GBK, EUC-KR, Big5, UTF-8) |
| **`ReaderUtil`** | Encoding detection heuristic (`detect_encoding`), `codepage_to_encoding`, `encoding_to_codepage`, `get_engine_version`, Delphi OLE `to_t_date_time` & `to_unix_timestamp`, `generate_timestamp` |
| **`Setup`** | Project & actor template setup (`actor`, `parameters`) for level-cap migrations and stats initialisation |
| **`IniReader`** | Section-aware, case-insensitive INI parser for `RPG_RT.ini` and EasyRPG config |
| **XML** | Serialisation (`save_xml` / `save_xml_to_writer`) using the official tag names |
| **Editor integration** | Native Rust bridge in `easy-rpg REditor`, replacing the FFI and MSVC static linking |

## Test results

```text
running 8 tests
test test_ini_parsing ... ok
test test_lsd_save_roundtrip ... ok
test test_reader_util_and_setup ... ok
test test_lmt_roundtrip_2003 ... ok (22 maps)
test test_lmt_roundtrip_2000 ... ok (81 maps)
test test_ldb_roundtrip_2000 ... ok (8 actors, 14 chipsets, 132 skills, 86 items, 81 enemies, 86 troops)
test test_lmu_roundtrip_all_maps_2003 ... ok (20 maps)
test test_lmu_roundtrip_all_maps_2000 ... ok (80 maps)

test result: ok. 8 passed; 0 failed; 0 ignored; finished in 1.24s
```

What this shows: for the project files in the test corpus, reading a file and writing it
back produces a byte-identical result, so the chunk machinery, default-value rules and
string encoding survive a full round trip on those inputs.

What it does not show: correctness on files outside the corpus, or completeness relative to
liblcf.
