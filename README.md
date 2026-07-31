# adb

`adb` is a command-line tool for looking up airport information, measuring
distances between airports (or raw coordinates), and searching airports by
name. Data is bundled from the
[OurAirports](https://github.com/davidmegginson/ourairports-data) project and
indexed into a local on-disk database on first run.

- **Look up** an airport by ICAO ident or IATA/GPS/local code.
- **Measure distance** between two or more airports, with optional raw
  coordinate waypoints.
- **Search** airports by free-text query with fuzzy matching.

## Installation

Build from source with a recent Rust toolchain:

```sh
cargo install --path .
```

or clone and run directly:

```sh
cargo run --release -- [args]
```

The indexed database (`airdb.redb`) is created in your OS data directory on
first run (via the `directories` crate). It rebuilds automatically when the
bundled data changes.

## Usage

### Look up an airport

```sh
adb KSEA
adb JFK        # resolves via the IATA code
```

```
KSEA Seattle–Tacoma International Airport (433 feet)
  Seattle
  US-WA
  47.4479°N 122.3103°W

Runways:
  16C/34C   9426ft  +L
  16L/34R  11901ft  +L
  16R/34L   8500ft  +L
```

### Measure distance

The `dist` subcommand computes the total distance of a route through one or
more waypoints. Each waypoint may be an airport identifier or a raw coordinate
string (decimal degrees, DMS, hemisphere-prefixed/suffixed, etc.):

```sh
adb dist KSEA KLAX
adb dist "40.6413 -73.7781" KSEA
```

```
KSEA -> KLAX  828.9

Total distance: 828.9 nm (953.8 miles)
```

Distances are reported in nautical miles, with a statute-miles total. Geodesic
distance is computed with Vicenty's formula (haversine fallback) via
[`geoutils`](https://crates.io/crates/geoutils).

### Search

The `search` subcommand (aliases: `find`, `s`, `f`) performs a ranked
free-text search over airport names and locations, returning the top matches:

```sh
adb search seattle
adb f new york
```

Search combines exact token matches with Levenshtein-1 fuzzy matches over an
[fst](https://github.com/BurntSushi/fst) term set.

### Update the database

Rebuild the local index from the bundled CSV data:

```sh
adb update
```

This is only needed when the bundled data has been refreshed (see below); the
database otherwise rebuilds automatically when its data hash changes.

## Updating the bundled data

Refresh `resource/airports.csv` and `resource/runways.csv` from upstream, then
rebuild the index:

```sh
scripts/update-data.sh
cargo run --release -- update
```

`update-data.sh` verifies each file's header before replacing the committed
snapshot.

## Data source

Airport and runway data © [OurAirports](https://ourairports.com/),
distributed under the [Open Database License (ODbL)](https://opendatacommons.org/licenses/odbl/).
See the [OurAirports data repository](https://github.com/davidmegginson/ourairports-data)
for details.
