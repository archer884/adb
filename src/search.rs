use std::{fs, io};

use csv::Reader;
use directories::ProjectDirs;
use hashbrown::HashMap;
use tantivy::{
    directory::MmapDirectory,
    doc,
    schema::{self, Facet, Field, Schema},
    Index, IndexWriter,
};

use crate::model::{Airport, Runway, RunwayTemplate};

static AIRPORTS: &str = include_str!("../resource/airports.csv");
static RUNWAYS: &str = include_str!("../resource/runways.csv");

pub struct Fields {
    pub identifier: Field,
    pub description: Field,
    pub facet: Field,
    pub object: Field,
}

pub fn initialize(force: bool) -> tantivy::Result<(Index, Fields)> {
    initialize_with_source(AIRPORTS, RUNWAYS, force)
}

pub fn initialize_with_source(
    airports: &str,
    runways: &str,
    force: bool,
) -> tantivy::Result<(Index, Fields)> {
    let dirs = ProjectDirs::from("org", "Hack Commons", "airdatabase").unwrap();
    let path = dirs.data_dir();

    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    let mut builder = Schema::builder();
    let fields = Fields {
        identifier: builder.add_text_field("identifier", schema::TEXT),
        description: builder.add_text_field("description", schema::TEXT),
        facet: builder.add_facet_field("facet", schema::INDEXED | schema::STORED),
        object: builder.add_text_field("object", schema::STORED),
    };
    let schema = builder.build();
    let mmap_dir = MmapDirectory::open(path)?;

    if force && Index::exists(&mmap_dir)? {
        fs::remove_dir_all(path)?;
        fs::create_dir_all(path)?;
    }

    if !Index::exists(&mmap_dir)? {
        const MEGABYTE: usize = 0x100000;
        const ARENA_SIZE: usize = MEGABYTE * 1000;

        let index = Index::create_in_dir(path, schema)?;
        write_index(airports, runways, &fields, &mut index.writer(ARENA_SIZE)?)?;
        Ok((index, fields))
    } else {
        Ok((Index::open(mmap_dir)?, fields))
    }
}

fn write_index(
    airports: &str,
    runways: &str,
    fields: &Fields,
    writer: &mut IndexWriter,
) -> tantivy::Result<()> {
    let mut source = airports.as_bytes();
    let mut reader = Reader::from_reader(&mut source);

    let mut runways = load_runways(runways).unwrap();

    for airport in reader.deserialize() {
        let mut airport = Airport::from_template(airport.unwrap()).unwrap();
        let ident = &airport.ident;
        let name = &airport.name;
        let iso_country = &airport.iso_country;
        let iso_region = &airport.iso_region;
        let municipality = &airport.municipality;

        // For my next trick, when available, I'm going to pull runways for each airport.
        // ...Since I'm doing it this way, ICAO identifiers better be unique.
        if let Some(runways) = runways.remove(&airport.ident) {
            airport.runways = runways;
        }

        writer.add_document(doc!(
            fields.identifier => ident.to_string(),
            fields.description => format!("{ident} {name}, {municipality}, {iso_region}, {iso_country}"),
            fields.facet => Facet::from(&format!("/{iso_country}/{iso_region}/{municipality}/{ident}/{name}")),
            fields.object => serde_json::to_string(&airport).unwrap(),
        ))?;
    }

    writer.commit()?;
    Ok(())
}

fn load_runways(runways: &str) -> io::Result<HashMap<String, Vec<Runway>>> {
    let mut source = runways.as_bytes();
    let mut reader = Reader::from_reader(&mut source);
    let mut map: HashMap<_, Vec<_>> = HashMap::new();

    for runway in reader.deserialize::<RunwayTemplate>() {
        let runway: Runway = runway?.into();
        map.entry(runway.airport.clone()).or_default().push(runway);
    }

    Ok(map)
}
