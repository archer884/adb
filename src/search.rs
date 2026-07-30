use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use csv::Reader;
use directories::ProjectDirs;
use fst::SetBuilder;
use hashbrown::HashMap;
use redb::{Database, MultimapTableDefinition, ReadableDatabase, TableDefinition};

use crate::model::{Airport, AirportTemplate, Runway, RunwayTemplate};
use crate::Result;

include!(concat!(env!("OUT_DIR"), "/data_hash.rs"));

const AIRPORTS: &str = include_str!("../resource/airports.csv");
const RUNWAYS: &str = include_str!("../resource/runways.csv");

pub const AIRPORTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("airports");
pub const CODES_TABLE: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("codes");
pub const POSTINGS_TABLE: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("postings");
pub const META_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("meta");

pub const META_TERMS: &str = "terms";
pub const META_HASH: &str = "data_hash";

pub fn db_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("org", "Hack Commons", "airdatabase")
        .ok_or_else(|| io::Error::other("cannot determine data directory"))?;
    let dir = dirs.data_dir();
    fs::create_dir_all(dir)?;
    Ok(dir.join("airdb.redb"))
}

pub fn initialize(force: bool) -> Result<Database> {
    let path = db_path()?;
    if force || !path.exists() {
        return fresh_build(&path);
    }
    let db = Database::create(&path)?;
    if needs_rebuild(&db)? {
        drop(db);
        return fresh_build(&path);
    }
    Ok(db)
}

fn fresh_build(path: &Path) -> Result<Database> {
    let _ = fs::remove_file(path);
    let db = Database::create(path)?;
    build(&db)?;
    Ok(db)
}

fn needs_rebuild(db: &Database) -> Result<bool> {
    let txn = db.begin_read()?;
    let table = match txn.open_table(META_TABLE) {
        Ok(t) => t,
        Err(redb::TableError::TableDoesNotExist(_)) => return Ok(true),
        Err(e) => return Err(e.into()),
    };
    let stored = match table.get(META_HASH)? {
        Some(guard) => {
            let bytes = guard.value();
            (bytes.len() == 8).then(|| u64::from_le_bytes(bytes.try_into().unwrap()))
        }
        None => None,
    };
    Ok(stored != Some(DATA_HASH))
}

fn build(db: &Database) -> Result<()> {
    let runways = load_runways(RUNWAYS)?;

    let txn = db.begin_write()?;
    {
        let mut airports_t = txn.open_table(AIRPORTS_TABLE)?;
        let mut codes_t = txn.open_multimap_table(CODES_TABLE)?;
        let mut postings_t = txn.open_multimap_table(POSTINGS_TABLE)?;
        let mut meta_t = txn.open_table(META_TABLE)?;
        let mut terms: BTreeSet<String> = BTreeSet::new();

        let mut source = AIRPORTS.as_bytes();
        let mut reader = Reader::from_reader(&mut source);
        for record in reader.deserialize() {
            let template: AirportTemplate = record?;
            let Some(mut airport) = Airport::from_template(template) else {
                continue;
            };
            if let Some(rwys) = runways.get(&airport.ident) {
                airport.runways = rwys.clone();
            }
            let ident = airport.ident.clone();

            for code in [
                &airport.ident,
                &airport.iata_code,
                &airport.gps_code,
                &airport.local_code,
            ] {
                if !code.is_empty() {
                    codes_t.insert(code.to_lowercase().as_str(), ident.as_str())?;
                }
            }

            let description = format!(
                "{} {} {} {} {}",
                airport.ident,
                airport.name,
                airport.municipality,
                airport.iso_region,
                airport.iso_country
            );
            for token in tokenize(&description) {
                terms.insert(token.clone());
                postings_t.insert(token.as_str(), ident.as_str())?;
            }

            let bytes = bitcode::encode(&airport);
            airports_t.insert(ident.as_str(), bytes.as_slice())?;
        }

        let mut builder = SetBuilder::new(Vec::new())?;
        for term in &terms {
            builder.insert(term.as_bytes())?;
        }
        let fst_bytes = builder.into_inner()?;
        meta_t.insert(META_TERMS, fst_bytes.as_slice())?;
        let hash_bytes = DATA_HASH.to_le_bytes();
        meta_t.insert(META_HASH, hash_bytes.as_slice())?;
    }
    txn.commit()?;
    Ok(())
}

pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() > 2)
        .map(|s| s.to_lowercase())
        .collect()
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
