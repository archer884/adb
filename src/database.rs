use tantivy::{
    collector::TopDocs,
    query::{Query, QueryParser},
    schema::Value,
    Index, IndexReader, TantivyDocument,
};

use crate::{
    model::Airport,
    search::{self, Fields},
};

pub struct Database {
    index: Index,
    reader: IndexReader,
    fields: Fields,
}

impl Database {
    pub fn initialize() -> tantivy::Result<Self> {
        let (index, fields) = search::initialize(false)?;
        let reader = index.reader()?;

        Ok(Self {
            index,
            reader,
            fields,
        })
    }

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
        let searcher = self.reader.searcher();
        let candidates: Vec<_> = searcher
            .search(query, &TopDocs::with_limit(limit))?
            .into_iter()
            .filter_map(|(_, address)| searcher.doc(address).ok())
            .filter_map(|document: TantivyDocument| {
                let data = document.get_first(self.fields.object)?.as_bytes()?;
                serde_json::from_slice(data).ok()
            })
            .collect();

        Ok(candidates)
    }
}
