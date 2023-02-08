use std::{fs, io::Cursor};

use csv::Reader;
use directories::ProjectDirs;
use tantivy::{
    directory::MmapDirectory,
    doc,
    schema::{self, Facet, Field, Schema},
    Index, IndexWriter,
};

use crate::model::Airport;

static RAW: &str = include_str!("../resource/airports.csv");

pub struct Fields {
    pub identifier: Field,
    pub description: Field,
    pub facet: Field,
    pub object: Field,
}

pub fn initialize(force: bool) -> tantivy::Result<(Index, Fields)> {
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
        write_index(&fields, &mut index.writer(ARENA_SIZE)?)?;
        Ok((index, fields))
    } else {
        Ok((Index::open(mmap_dir)?, fields))
    }
}

fn write_index(fields: &Fields, writer: &mut IndexWriter) -> tantivy::Result<()> {
    let mut reader = Reader::from_reader(Cursor::new(RAW));

    for airport in reader.deserialize() {
        let airport = Airport::from_template(airport.unwrap()).unwrap();
        let ident = &airport.ident;
        let name = &airport.name;
        let iso_country = &airport.iso_country;
        let iso_region = &airport.iso_region;
        let municipality = &airport.municipality;

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
