use csv::Reader;
use geoutils::Location;
use hashbrown::HashMap;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;
use std::{io::Cursor, process};

// Ordinarily, I would just use structopt, but I felt like this program would
// potentially wind up with optional subcommands.

enum Cmd {
    Listing(String),
    Distance(String, String),
}

#[derive(Clone, Debug)]
struct Airport {
    ident: String,
    kind: String,
    name: String,
    elevation_ft: i32,
    continent: String,
    iso_country: String,
    iso_region: String,
    municipality: String,
    gps_code: String,
    iata_code: String,
    local_code: String,
    coordinates: Coords,
}

#[derive(Clone, Debug, Deserialize)]
struct AirportTemplate {
    ident: String,
    #[serde(rename = "type")]
    kind: String,
    name: String,
    elevation_ft: i32,
    continent: String,
    iso_country: String,
    iso_region: String,
    municipality: String,
    gps_code: String,
    iata_code: String,
    local_code: String,
    coordinates: String,
}

impl<'de> Deserialize<'de> for Airport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let AirportTemplate {
            ident,
            kind,
            name,
            elevation_ft,
            continent,
            iso_country,
            iso_region,
            municipality,
            gps_code,
            iata_code,
            local_code,
            coordinates,
        } = AirportTemplate::deserialize(deserializer)?;

        Ok(Airport {
            ident,
            kind,
            name,
            elevation_ft,
            continent,
            iso_country,
            iso_region,
            municipality,
            gps_code,
            iata_code,
            local_code,
            coordinates: coordinates.parse().map_err(serde::de::Error::custom)?,
        })
    }
}

#[derive(Clone, Debug)]
struct Coords {
    latitude: f64,
    longitude: f64,
}

impl Coords {
    fn location(&self) -> Location {
        let &Coords {
            latitude,
            longitude,
        } = self;
        Location::new(latitude, longitude)
    }
}

impl FromStr for Coords {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(',').map(|x| x.trim());
        let latitude = parts
            .next()
            .ok_or("Missing latitude")?
            .parse()
            .map_err(|_| "Bad latitude")?;
        let longitude = parts
            .next()
            .ok_or("Missing longitude")?
            .parse()
            .map_err(|_| "Bad longitude")?;
        Ok(Coords {
            latitude,
            longitude,
        })
    }
}

fn main() {
    let cmd = read_options();
    let data = include_str!("../resource/airport-codes.csv");
    let data = build_database(data);
    let index = build_index(&data);

    match cmd {
        Cmd::Listing(identifier) => print_listing(&identifier, &index),
        Cmd::Distance(a, b) => print_distance(&a, &b, &index),
    }
}

fn print_listing(identifier: &str, database: &HashMap<String, &Airport>) {
    match database.get(&*identifier.to_ascii_uppercase()) {
        Some(&airport) => {
            println!("{:#?}", airport);
        }

        None => {
            eprintln!("Airport not found");
            process::exit(1);
        }
    }
}

fn print_distance(a: &str, b: &str, database: &HashMap<String, &Airport>) {
    const METERS_PER_NAUTICAL_MILE: f64 = 1852.001;

    fn get_airport<'a>(identifier: &str, database: &HashMap<String, &'a Airport>) -> &'a Airport {
        match database.get(&*identifier.to_ascii_uppercase()) {
            Some(&airport) => airport,
            None => {
                eprintln!("{} not found", identifier);
                process::exit(2);
            }
        }
    }

    let a = get_airport(a, database).coordinates.location();
    let b = get_airport(b, database).coordinates.location();

    println!(
        "{:.02} nmi",
        a.distance_to(&b).unwrap().meters() / METERS_PER_NAUTICAL_MILE
    )
}

fn build_database(data: &str) -> Vec<Airport> {
    Reader::from_reader(Cursor::new(data))
        .deserialize()
        .filter_map(Result::ok)
        .collect()
}

fn build_index(data: &[Airport]) -> HashMap<String, &Airport> {
    data.iter()
        .map(|x| (x.ident.to_ascii_uppercase(), x))
        .collect()
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
