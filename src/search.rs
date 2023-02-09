use std::{fs, io::Read};

use csv::Reader;
use directories::ProjectDirs;
use libflate::gzip::Decoder;
use tantivy::{
    directory::MmapDirectory,
    doc,
    schema::{self, Facet, Field, Schema},
    Index, IndexWriter,
};

use crate::model::Airport;

static SOURCE_DATA: &[u8] = include_bytes!("../resource/airports.csv.gz");

pub struct Fields {
    pub identifier: Field,
    pub description: Field,
    pub facet: Field,
    pub object: Field,
}

pub fn initialize(force: bool) -> tantivy::Result<(Index, Fields)> {
    let mut decoder = Decoder::new(SOURCE_DATA)?;
    let mut result = Vec::new();
    let mut buf = String::new();

    decoder.read_to_end(&mut result)?;
    (&mut &*result).read_to_string(&mut buf)?;

    initialize_with_source(&buf, force)
}

pub fn initialize_with_source(source: &str, force: bool) -> tantivy::Result<(Index, Fields)> {
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
        write_index(source, &fields, &mut index.writer(ARENA_SIZE)?)?;
        Ok((index, fields))
    } else {
        Ok((Index::open(mmap_dir)?, fields))
    }
}

fn write_index(source: &str, fields: &Fields, writer: &mut IndexWriter) -> tantivy::Result<()> {
    let mut source = source.as_bytes();
    let mut reader = Reader::from_reader(&mut source);

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
