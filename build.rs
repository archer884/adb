use adb_data::Airport;
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("database.rs");

    let airport_database = read_data();
    let mut buf = String::new();

    writeln!(buf, include_str!("./resource/mod_head.txt")).unwrap();
    for item in airport_database {
        writeln!(
            buf,
            "AotAirport {{ \
            ident: \"{}\", \
            kind: \"{}\", \
            name: \"{}\", \
            elevation_ft: {:?}, \
            continent: \"{}\", \
            iso_country: \"{}\", \
            iso_region: \"{}\", \
            municipality: \"{}\", \
            gps_code: \"{}\", \
            iata_code: \"{}\", \
            local_code: \"{}\", \
            coordinates: {:?} \
        }},",
            item.ident,
            item.kind,
            safe_string(&item.name),
            item.elevation_ft,
            item.continent,
            item.iso_country,
            item.iso_region,
            safe_string(&item.municipality),
            item.gps_code,
            item.iata_code,
            item.local_code,
            item.coordinates,
        )
        .unwrap();
    }
    writeln!(buf, "];").unwrap();

    fs::write(&dest_path, buf).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=resource/airport-codes-short.csv");
    println!("cargo:rerun-if-changed=resource/mod_head.txt");
}

fn read_data() -> Vec<Airport> {
    use csv::Reader;
    use std::io::Cursor;

    // For a start, since this happens at compile time, let's just bail if there's
    // any data we can't actually read. Additionally, in order to save on carbon
    // emissions, we provide only a faux csv for debug purposes.
    #[cfg(debug_assertions)]
    static CSV_DATA: &str = include_str!("./resource/airport-codes-short.csv");

    #[cfg(not(debug_assertions))]
    static CSV_DATA: &str = include_str!("./resource/airport-codes.csv");

    let data: Result<Vec<Airport>, _> = Reader::from_reader(Cursor::new(CSV_DATA))
        .deserialize()
        .collect();
    let mut data = data.unwrap();
    data.sort_by(|a, b| a.ident.cmp(&b.ident));
    data
}

fn safe_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
