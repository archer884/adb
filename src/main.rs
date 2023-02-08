use std::process;

mod database;
mod model;
mod pairs;
mod search;

use clap::Parser;
use database::Database;
use hashbrown::HashMap;
use pairs::Pairs;

use crate::model::Airport;

#[derive(Debug, Parser)]
#[command(subcommand_negates_reqs(true))]
struct Args {
    #[arg(required = true)]
    identifiers: Vec<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Parser)]
enum Command {
    /// measure distance between airports
    Dist {
        origin: String,
        waypoints: Vec<String>,
    },

    /// search airports
    #[command(alias = "find", alias = "s", alias = "f")]
    Search { query: String },

    /// update database
    Update,
}

fn main() {
    if let Err(e) = run(&Args::parse()) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run(args: &Args) -> tantivy::Result<()> {
    if let Some(command) = &args.command {
        match command {
            Command::Dist { origin, waypoints } => {
                // This is a filthy, filthy hack and we should probably just change the contract
                // for this function instead.
                let mut identifiers = vec![origin];
                identifiers.extend(waypoints);
                print_distance(&identifiers)?;
            },
            Command::Search { query } => print_search(query)?,
            Command::Update => {
                search::initialize(true)?;
                return Ok(());
            }
        }
    }

    let db = Database::initialize()?;

    for identifier in &args.identifiers {
        match db.by_identifier(identifier)? {
            Some(airport) => {
                println!("{airport}");
            }
            None => {
                eprintln!("{identifier} not found");
            }
        }
    }

    Ok(())
}

fn print_distance<T: AsRef<str>>(identifiers: &[T]) -> tantivy::Result<()> {
    const METERS_PER_NAUTICAL_MILE: f64 = 1852.001;

    let db = Database::initialize()?;
    let cache: tantivy::Result<HashMap<_, Airport>> = identifiers
        .iter()
        .map(|identifier| identifier.as_ref())
        .filter_map(|identifier| {
            Some(
                db.by_identifier(identifier)
                    .transpose()?
                    .map(|airport| (identifier, airport)),
            )
        })
        .collect();
    let cache = cache?;

    let airport_pairs = identifiers.pairs().map(|(a, b)| {
        (
            cache.get(a.as_ref()).unwrap(),
            cache.get(b.as_ref()).unwrap(),
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
        preformat_records.push((&a.ident, &b.ident, formatted_distance));
        dist += leg;
    }

    for (a, b, dist) in preformat_records {
        println!("{a:>4} -> {b:>4}  {dist:>dist_column_width$}");
    }

    println!(
        "\nTotal distance: {:.01} nm",
        dist / METERS_PER_NAUTICAL_MILE
    );

    Ok(())
}

fn print_search(query: &str) -> tantivy::Result<()> {
    use std::io::{self, Write};

    let db = Database::initialize()?;
    let candidates = db.search(query)?;

    let mut handle = io::stdout().lock();

    for candidate in candidates {
        writeln!(
            handle,
            "{} {} {}",
            candidate.ident, candidate.iso_region, candidate.name
        )
        .unwrap();
    }

    Ok(())
}
