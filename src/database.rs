use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Set, Streamer};
use hashbrown::HashMap;
use redb::{Database as Redb, ReadableDatabase};

use crate::model::Airport;
use crate::search::{
    tokenize, AIRPORTS_TABLE, CODES_TABLE, META_TABLE, META_TERMS, POSTINGS_TABLE,
};
use crate::{Error, Result};

pub struct Database {
    db: Redb,
    terms: Set<Vec<u8>>,
}

impl Database {
    pub fn initialize() -> Result<Self> {
        let db = crate::search::initialize(false)?;
        let terms = load_terms(&db)?;
        Ok(Self { db, terms })
    }

    pub fn by_identifier(&self, ident: &str) -> Result<Option<Airport>> {
        if let Some(airport) = self.fetch(ident)? {
            return Ok(Some(airport));
        }

        let lower = ident.to_lowercase();
        let resolved = {
            let txn = self.db.begin_read()?;
            let table = match txn.open_multimap_table(CODES_TABLE) {
                Ok(t) => t,
                Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
                Err(e) => return Err(e.into()),
            };
            let mut found = None;
            let iter = table.get(lower.as_str())?;
            for item in iter {
                found = Some(item?.value().to_owned());
                break;
            }
            found
        };

        match resolved {
            Some(ident) => self.fetch(&ident),
            None => Ok(None),
        }
    }

    pub fn search(&self, query: &str) -> Result<Vec<Airport>> {
        let mut scores: HashMap<String, u32> = HashMap::new();
        let txn = self.db.begin_read()?;
        let postings = match txn.open_multimap_table(POSTINGS_TABLE) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(vec![]),
            Err(e) => return Err(e.into()),
        };

        const EXACT_WEIGHT: u32 = 8;
        const FUZZY_WEIGHT: u32 = 1;

        for token in tokenize(query) {
            let token_str = token.as_str();

            for item in postings.get(token_str)? {
                let ident = item?.value().to_owned();
                *scores.entry(ident).or_insert(0) += EXACT_WEIGHT;
            }

            let lev = Levenshtein::new(token_str, 1)?;
            let mut stream = self.terms.search(&lev).into_stream();
            while let Some(term_bytes) = stream.next() {
                let term =
                    std::str::from_utf8(term_bytes).map_err(|e| Error::Codec(e.to_string()))?;
                if term == token_str {
                    continue;
                }
                for item in postings.get(term)? {
                    let ident = item?.value().to_owned();
                    *scores.entry(ident).or_insert(0) += FUZZY_WEIGHT;
                }
            }
        }

        let mut ranked: Vec<(String, u32)> = scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        ranked.truncate(25);

        let mut results = Vec::with_capacity(ranked.len());
        for (ident, _) in ranked {
            if let Some(airport) = self.fetch(&ident)? {
                results.push(airport);
            }
        }
        Ok(results)
    }

    fn fetch(&self, ident: &str) -> Result<Option<Airport>> {
        let txn = self.db.begin_read()?;
        let table = match txn.open_table(AIRPORTS_TABLE) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let Some(guard) = table.get(ident)? else {
            return Ok(None);
        };
        let airport =
            bitcode::decode::<Airport>(guard.value()).map_err(|e| Error::Codec(e.to_string()))?;
        Ok(Some(airport))
    }
}

fn load_terms(db: &Redb) -> Result<Set<Vec<u8>>> {
    let txn = db.begin_read()?;
    let table = txn.open_table(META_TABLE)?;
    match table.get(META_TERMS)? {
        Some(guard) => Ok(Set::new(guard.value().to_vec())?),
        None => Ok(Set::default()),
    }
}
