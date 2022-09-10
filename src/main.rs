use std::fmt::{self, Display};
use std::process;

mod pairs;

use adb_data::AotAirport;
use clap::Parser;
use pairs::Pairs;

include!(concat!(env!("OUT_DIR"), "/database.rs"));

#[derive(Clone, Debug, Parser)]
enum Args {
    Dist {
        #[clap(min_values(2))]
        identifiers: Vec<String>,
    },
    Find {
        query: String,
    },
    Search {
        query: String,
    },
    Info {
        identifier: String,
    },
}

struct AirportFormatter<'a>(&'a AotAirport);

impl Display for AirportFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let airport = self.0;
        match airport.elevation_ft {
            Some(elevation) => write!(
                f,
                "{} {} ({} feet)\n  {}\n  {}\n  {}\n  {}",
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
                "{} {}\n  {}\n  {}\n  {}\n  {}",
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
    let args = Args::parse();
    match args {
        Args::Dist { identifiers } => print_distance(&identifiers),
        Args::Find { query } | Args::Search { query } => print_find(&query),
        Args::Info { identifier } => print_listing(&identifier),
    }
}

fn print_distance<T: AsRef<str>>(identifiers: &[T]) {
    const METERS_PER_NAUTICAL_MILE: f64 = 1852.001;

    let airport_pairs = identifiers.pairs().map(|(a, b)| {
        (
            find_by_identifier(a.as_ref()),
            find_by_identifier(b.as_ref()),
        )
    });

    let mut dist = 0.0;
    let mut preformat_records = Vec::new();
    let mut dist_column_width = 0;

    for (a, b) in airport_pairs {
        let leg = a
            .coordinates
            .location()
            .distance_to(&b.coordinates.location())
            .unwrap()
            .meters();

        let formatted_distance = format!("{:.01}", leg / METERS_PER_NAUTICAL_MILE);
        dist_column_width = std::cmp::max(dist_column_width, formatted_distance.len());
        preformat_records.push((a.ident, b.ident, formatted_distance));
        dist += leg;
    }

    for (a, b, dist) in preformat_records {
        println!(
            "{:>4} -> {:>4}  {:>width$}",
            a,
            b,
            dist,
            width = dist_column_width
        );
    }

    println!(
        "\nTotal distance: {:.01} nm",
        dist / METERS_PER_NAUTICAL_MILE
    );
}

fn print_find(query: &str) {
    let pattern = regex::RegexBuilder::new(&*format!(".*{}.*", query))
        .case_insensitive(true)
        .build();

    match pattern {
        Ok(pattern) => {
            let candidates =
                select_candidates(|x| pattern.is_match(x.name) || pattern.is_match(x.municipality));

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
    println!("{}", AirportFormatter(find_by_identifier(identifier)));
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
