use adb_data::{AotAirport, Coords};
use std::process;

include!(concat!(env!("OUT_DIR"), "/database.rs"));

// Ordinarily, I would just use structopt, but I felt like this program would
// potentially wind up with optional subcommands.

enum Cmd {
    Listing(String),
    Distance(String, String),
}

fn main() {
    match read_options() {
        Cmd::Listing(identifier) => print_listing(&identifier),
        Cmd::Distance(a, b) => print_distance(&a, &b),
    }
}

fn print_listing(identifier: &str) {
    match find(&*identifier.to_ascii_uppercase()) {
        Some(airport) => {
            println!("{:#?}", airport);
        }

        None => {
            eprintln!("Airport not found");
            process::exit(1);
        }
    }
}

fn print_distance(a: &str, b: &str) {
    const METERS_PER_NAUTICAL_MILE: f64 = 1852.001;

    fn get_airport<'a>(identifier: &str) -> &'a AotAirport {
        match find(&*identifier.to_ascii_uppercase()) {
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

fn find(identifier: &str) -> Option<&'static AotAirport> {
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

    let options = app.subcommand(dist_cmd).get_matches();
    if let Some(options) = options.subcommand_matches("dist") {
        let origin = value_t!(options, "ORIGIN", String).unwrap_or_else(|e| e.exit());
        let destination = value_t!(options, "DESTINATION", String).unwrap_or_else(|e| e.exit());

        return Cmd::Distance(origin, destination);
    }

    let identifier = value_t!(options, "IDENT", String).unwrap_or_else(|e| e.exit());
    Cmd::Listing(identifier)
}
