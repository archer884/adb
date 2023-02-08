use tantivy::{
    collector::TopDocs,
    query::{Query, QueryParser},
    Index,
};

use crate::{
    model::Airport,
    search::{self, Fields},
};

pub struct Database {
    index: Index,
    fields: Fields,
}

impl Database {
    pub fn initialize() -> tantivy::Result<Self> {
        let (index, fields) = search::initialize(false)?;
        Ok(Self { index, fields })
    }

    // FIXME: We probably want some way to not re-create the reader every time we perform a query.
    pub fn by_identifier(&self, identifier: &str) -> tantivy::Result<Option<Airport>> {
        let query = QueryParser::for_index(&self.index, vec![self.fields.identifier])
            .parse_query(identifier)?;

        Ok(self.materialize_query(&query, 1)?.into_iter().next())
    }

    pub fn search(&self, query: &str) -> tantivy::Result<Vec<Airport>> {
        let query = QueryParser::for_index(&self.index, vec![self.fields.description])
            .parse_query(query)?;

        self.materialize_query(&query, 25)
    }

    fn materialize_query(&self, query: &dyn Query, limit: usize) -> tantivy::Result<Vec<Airport>> {
        let searcher = self.index.reader()?.searcher();
        let candidates: Vec<_> = searcher
            .search(query, &TopDocs::with_limit(limit))?
            .into_iter()
            .filter_map(|(_, address)| searcher.doc(address).ok())
            .filter_map(|document| {
                let text = document.get_first(self.fields.object)?.as_text()?;
                serde_json::from_str(text).ok()
            })
            .collect();

        Ok(candidates)
    }
}
