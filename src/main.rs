use adb_data::AotAirport;
use std::fmt::{self, Display};
use std::process;

include!(concat!(env!("OUT_DIR"), "/database.rs"));

// Ordinarily, I would just use structopt, but I felt like this program would
// potentially wind up with optional subcommands.

enum Cmd {
    Distance(String, String),
    Find(String),
    Listing(String),
}

struct AirportFormatter<'a>(&'a AotAirport);

// AotAirport {
//     ident: "KEB",
//     kind: "small_airport",   
//     name: "Nanwalek Airport",
//     elevation_ft: Some(
//         27,
//     ),
//     continent: "NA",
//     iso_country: "US",
//     iso_region: "US-AK",
//     municipality: "Nanwalek",
//     gps_code: "KEB",
//     iata_code: "KEB",
//     local_code: "KEB",
//     coordinates: Coords {
//         latitude: 59.4,
//         longitude: -151.9,
//     },
// }

impl Display for AirportFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let airport = self.0;
        match airport.elevation_ft {
            Some(elevation) => write!(f, "{} {} ({} feet)\n  {}\n  {}\n  {}\n  {:?}", airport.ident, airport.name, elevation, airport.kind, airport.municipality, airport.iso_region, airport.coordinates),
            None => write!(f, "{} {}\n  {}\n  {}\n  {}\n  {:?}", airport.ident, airport.name, airport.kind, airport.municipality, airport.iso_region, airport.coordinates),
        }
    }
}

fn main() {
    match read_options() {
        Cmd::Distance(a, b) => print_distance(&a, &b),
        Cmd::Find(query) => print_find(&query),
        Cmd::Listing(identifier) => print_listing(&identifier),
    }
}

fn print_find(query: &str) {
    let pattern = regex::RegexBuilder::new(&*format!(".*{}.*", query))
        .case_insensitive(true)
        .build();

    match pattern {
        Ok(pattern) => {
            let candidates = select_candidates(|x| {
                pattern.is_match(&x.name) || pattern.is_match(&x.municipality)
            });

            format_candidates(candidates);
        }

        Err(_) => {
            let query = query.to_ascii_uppercase();
            let candidates = select_candidates(|x| {
                x.name.to_ascii_uppercase().contains(&query)
                    || x.municipality.to_ascii_uppercase().contains(&query)
            });

            format_candidates(candidates);
        }
    }
}

fn select_candidates<'a>(
    filter: impl Fn(&AotAirport) -> bool + 'a,
) -> impl Iterator<Item = (&'static str, &'static str, &'static str)> + 'a {
    AIRPORTS
        .iter()
        .filter(move |&x| filter(x))
        .map(|x| (x.ident, x.iso_region, x.name))
}

fn format_candidates(candidates: impl Iterator<Item = (&'static str, &'static str, &'static str)>) {
    use std::io::{self, Write};

    let handle = io::stdout();
    let mut handle = handle.lock();

    for (identifier, region, name) in candidates {
        writeln!(handle, "{} {} {}", identifier, region, name).unwrap();
    }
}

fn print_distance(a: &str, b: &str) {
    const METERS_PER_NAUTICAL_MILE: f64 = 1852.001;

    fn get_airport<'a>(identifier: &str) -> &'a AotAirport {
        match find_by_identifier(&*identifier.to_ascii_uppercase()) {
            Some(airport) => airport,
            None => {
                eprintln!("{} not found", identifier);
                process::exit(2);
            }
        }
    }

    let a = get_airport(a).coordinates.location();
    let b = get_airport(b).coordinates.location();

    println!(
        "{:.02} nmi",
        a.distance_to(&b).unwrap().meters() / METERS_PER_NAUTICAL_MILE
    )
}

fn print_listing(identifier: &str) {
    match find_by_identifier(&*identifier.to_ascii_uppercase()) {
        Some(airport) => {
            println!("{}", AirportFormatter(airport));
        }

        None => {
            eprintln!("Airport not found");
            process::exit(1);
        }
    }
}

fn find_by_identifier(identifier: &str) -> Option<&'static AotAirport> {
    AIRPORTS
        .binary_search_by(|probe| probe.ident.cmp(identifier))
        .ok()
        .and_then(|x| AIRPORTS.get(x))
}

fn read_options() -> Cmd {
    use clap::{crate_authors, crate_version, value_t, App, AppSettings, Arg, SubCommand};

    let app = App::new("adb")
        .setting(AppSettings::SubcommandsNegateReqs)
        .author(crate_authors!())
        .version(crate_version!())
        .about("Airport code database")
        .arg(Arg::with_name("IDENT").takes_value(true).required(true));

    let dist_cmd = SubCommand::with_name("dist")
        .about("Calculate the distance between two airports")
        .arg(Arg::with_name("ORIGIN").takes_value(true).required(true))
        .arg(
            Arg::with_name("DESTINATION")
                .takes_value(true)
                .required(true),
        );

    let find_cmd = SubCommand::with_name("find")
        .about("Find an airport by name or town")
        .arg(Arg::with_name("QUERY").takes_value(true).required(true));

    let options = app.subcommand(dist_cmd).subcommand(find_cmd).get_matches();

    if let Some(options) = options.subcommand_matches("dist") {
        let origin = value_t!(options, "ORIGIN", String).unwrap_or_else(|e| e.exit());
        let destination = value_t!(options, "DESTINATION", String).unwrap_or_else(|e| e.exit());
        return Cmd::Distance(origin, destination);
    }

    if let Some(options) = options.subcommand_matches("find") {
        let query = value_t!(options, "QUERY", String).unwrap_or_else(|e| e.exit());
        return Cmd::Find(query);
    }

    let identifier = value_t!(options, "IDENT", String).unwrap_or_else(|e| e.exit());
    Cmd::Listing(identifier)
}
