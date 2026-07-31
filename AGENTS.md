# AGENTS.md

Guidance for AI agents (and human contributors) working in this repository.

## Project overview

`adb` (airport database) is a command-line tool for looking up airport
information, measuring distances between airports/coordinates, and searching
airports by name. It is written in Rust (edition 2024) and builds an on-disk
indexed database from the bundled [OurAirports](https://github.com/davidmegginson/ourairports-data)
CSV data on first run.

The current version is in `Cargo.toml`. Bump it (and update `Cargo.lock`) when
cutting a release.

## Common commands

- **Build (debug):** `cargo build`
- **Build (release):** `cargo build --release` — uses LTO, single codegen unit,
  and `panic = "abort"` (see `Cargo.toml`).
- **Run:** `cargo run --release -- [args]` (e.g. `cargo run --release -- KSEA`)
- **Test:** `cargo test` — unit tests live inline under `#[cfg(test)] mod tests`
  (see `src/model.rs`).
- **Check fast:** `cargo check`
- **Update bundled data:** `scripts/update-data.sh` re-downloads
  `airports.csv` / `runways.csv` from OurAirports, then run
  `cargo run --release -- update` to rebuild the local index.

There is no configured linter or formatter beyond the default `cargo` tooling.
The workspace uses a custom `.cargo/config.toml` (lld linker, native CPU target,
macOS deployment target) — be aware this affects build behavior on this machine.

## Architecture

```
src/
  main.rs      CLI parsing (clap), subcommands, output formatting
  model.rs     Airport / Runway / Coords types, CSV templates, Display impls
  database.rs  Read-side queries: lookup by identifier, fuzzy search
  search.rs    DB build/indexing (redb tables, fst term set), tokenization
  waypoint.rs  Uniform distance abstraction over airports and raw coordinates
  pairs.rs     Iterator adapter yielding consecutive (a, b) pairs
  error.rs     Error enum + From conversions for io/redb/fst/bitcode/csv
build.rs       Hashes resource CSVs and emits DATA_HASH for cache invalidation
resource/      Bundled CSV snapshots from OurAirports (committed)
scripts/       Data refresh script
```

### Data flow / indexing

1. `build.rs` hashes `resource/{airports,runways}.csv` and writes
   `DATA_HASH: u64` into `OUT_DIR/data_hash.rs`, which `search.rs` includes.
2. On startup `search::initialize(force)` checks whether the persisted index at
   the OS data dir (`airdb.redb`, located via the `directories` crate) matches
   `DATA_HASH`; it rebuilds automatically when the bundled data changes.
3. The index (`redb`) stores four tables:
   - `airports` — ident → bitcode-encoded `Airport`
   - `codes`    — multimap of lowercased codes (ident/iata/gps/local) → ident
   - `postings` — multimap of token → ident (inverted index for search)
   - `meta`     — serialized fst term set (`terms`) and `data_hash`
4. `Database::search` combines exact postings matches (weight 8) with
   Levenshtein-1 fuzzy matches (weight 1) over the fst, ranks by score, and
   truncates to 25 results.
5. `Database::by_identifier` tries the ident directly, then falls back to the
   `codes` multimap so IATA/GPS/local codes resolve to the canonical airport.

### Distance computation

`print_distance` accepts a mix of airport identifiers and raw coordinate
strings (parsed by the `latlon` crate; see `Coords::from_str` and its tests in
`src/model.rs`). Distances use `geoutils` (Vicenty, falling back to haversine).
Output is in nautical miles with a miles total.

### CLI shape

`Args` (in `main.rs`) takes a variadic `IDENTIFIERS` list that prints airport
details when no subcommand is given, plus three subcommands: `dist`, `search`
(aliases `find`/`s`/`f`), and `update`. `subcommand_negates_reqs` is enabled so
subcommands work without an identifier.

## Conventions

- **No comments** unless absolutely necessary — code is expected to be
  self-documenting.
- Keep the `Error` enum and its `From` impls in `error.rs` complete; every
  external error type used in the crate should convert cleanly into `Error`.
- Tests are colocated with the code they test under `#[cfg(test)]`.
- When adding fields to `Airport`/`Runway`, remember they derive
  `bitcode::Encode`/`Decode` — changing the binary layout invalidates existing
  on-disk records, which is fine here because the data-hash mechanism rebuilds
  the index, but be deliberate about it.
- Do not commit the generated `airdb.redb` (it lives in the OS data dir, not
  the repo). Only `resource/*.csv` data snapshots belong in the repo.

## Verification checklist before finishing a task

1. `cargo test` passes.
2. `cargo build --release` succeeds.
3. Manually exercise the affected subcommand, e.g.
   `cargo run --release -- KSEA`,
   `cargo run --release -- dist KSEA KLAX`,
   `cargo run --release -- search seattle`.
