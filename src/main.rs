mod pairs;

use adb_data::AotAirport;
use pairs::Pairs;
use std::fmt::{self, Display};
use std::process;

include!(concat!(env!("OUT_DIR"), "/database.rs"));

enum Cmd {
    Distance(Vec<String>),
    Find(String),
    Listing(String),
}

struct AirportFormatter<'a>(&'a AotAirport);

impl Display for AirportFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let airport = self.0;
        match airport.elevation_ft {
            Some(elevation) => write!(
                f,
                "{} {} ({} feet)\n  {}\n  {}\n  {}\n  {:?}",
                airport.ident,
                airport.name,
                elevation,
                airport.kind,
                airport.municipality,
                airport.iso_region,
                airport.coordinates
            ),

            None => write!(
                f,
                "{} {}\n  {}\n  {}\n  {}\n  {:?}",
                airport.ident,
                airport.name,
                airport.kind,
                airport.municipality,
                airport.iso_region,
                airport.coordinates
            ),
        }
    }
}

fn main() {
    match read_options() {
        Cmd::Distance(identifiers) => print_distance(&identifiers),
        Cmd::Find(query) => print_find(&query),
        Cmd::Listing(identifier) => print_listing(&identifier),
    }
}

fn print_distance<T: AsRef<str>>(identifiers: &[T]) {
    const METERS_PER_NAUTICAL_MILE: f64 = 1852.001;

    let mut dist = 0.0;
    let airport_pairs = identifiers.pairs().map(|(a, b)| {
        (
            find_by_identifier(a.as_ref()),
            find_by_identifier(b.as_ref()),
        )
    });

    for (a, b) in airport_pairs {
        let a = a.coordinates.location();
        let b = b.coordinates.location();
        dist += a.distance_to(&b).unwrap().meters();
    }

    println!("{:.02} nmi", dist / METERS_PER_NAUTICAL_MILE);
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

        // This search mechanism is STUPID expensive, but I'm not convinced
        // it's ever gonna get used. We could, in theory, speed this up at
        // compile time by modifying the generated code to include capitalized
        // forms of the strings in question.
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

fn print_listing(identifier: &str) {
    println!("{}", AirportFormatter(find_by_identifier(&identifier)));
}

fn find_by_identifier(identifier: &str) -> &'static AotAirport {
    let identifier = identifier.to_ascii_uppercase();
    let result = AIRPORTS
        .binary_search_by(|probe| probe.ident.cmp(&identifier))
        .ok()
        .and_then(|x| AIRPORTS.get(x));

    match result {
        Some(airport) => airport,
        None => {
            eprintln!("{} not found", identifier);
            process::exit(1);
        }
    }
}

fn read_options() -> Cmd {
    use clap::{
        crate_authors, crate_version, value_t, values_t, App, AppSettings, Arg, SubCommand,
    };

    let app = App::new("adb")
        .setting(AppSettings::SubcommandsNegateReqs)
        .author(crate_authors!())
        .version(crate_version!())
        .about("Airport code database")
        .arg(Arg::with_name("IDENT").takes_value(true).required(true));

    let dist_cmd = SubCommand::with_name("dist")
        .about("Calculate the distance between two airports")
        .arg(
            Arg::with_name("IDENTS")
                .takes_value(true)
                .required(true)
                .multiple(true)
                .min_values(2),
        );

    let find_cmd = SubCommand::with_name("find")
        .about("Find an airport by name or town")
        .arg(Arg::with_name("QUERY").takes_value(true).required(true));

    let options = app.subcommand(dist_cmd).subcommand(find_cmd).get_matches();

    if let Some(options) = options.subcommand_matches("dist") {
        let identifiers = values_t!(options, "IDENTS", String).unwrap();
        return Cmd::Distance(identifiers);
    }

    if let Some(options) = options.subcommand_matches("find") {
        let query = value_t!(options, "QUERY", String).unwrap();
        return Cmd::Find(query);
    }

    let identifier = value_t!(options, "IDENT", String).unwrap();
    Cmd::Listing(identifier)
}
