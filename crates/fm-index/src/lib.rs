use fm_core::{Chunk, Guard, Result, error};
use std::sync::Mutex;
use tantivy::{
    Index,
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{STORED, Schema, TEXT, Value},
};
pub struct SearchIndex {
    index: Index,
    writer: Mutex<tantivy::IndexWriter>,
    id: tantivy::schema::Field,
    text: tantivy::schema::Field,
}
impl SearchIndex {
    pub fn open(guard: &Guard) -> Result<Self> {
        let mut b = Schema::builder();
        let id = b.add_text_field("id", STORED);
        let text = b.add_text_field("text", TEXT);
        let schema = b.build();
        let dir = guard.resolve(".folder-mind/index")?;
        let index = Index::open_in_dir(&dir)
            .or_else(|_| Index::create_in_dir(&dir, schema))
            .map_err(error)?;
        let writer = index.writer(20_000_000).map_err(error)?;
        Ok(Self {
            index,
            writer: Mutex::new(writer),
            id,
            text,
        })
    }
    pub fn rebuild(&self, chunks: &[Chunk]) -> Result<()> {
        let mut w = self.writer.lock().map_err(error)?;
        w.delete_all_documents().map_err(error)?;
        for c in chunks {
            w.add_document(doc!(self.id=>c.id.clone(),self.text=>c.text.clone()))
                .map_err(error)?;
        }
        w.commit().map_err(error)?;
        Ok(())
    }
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        let reader = self.index.reader().map_err(error)?;
        let s = reader.searcher();
        let p = QueryParser::for_index(&self.index, vec![self.text]);
        let (q, _) = p.parse_query_lenient(query);
        let docs = s.search(&q, &TopDocs::with_limit(limit)).map_err(error)?;
        docs.into_iter()
            .map(|(_, a)| {
                let d: tantivy::TantivyDocument = s.doc(a).map_err(error)?;
                Ok(d.get_first(self.id)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string())
            })
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retrieval() {
        let t = tempfile::tempdir().unwrap();
        let g = Guard::new(t.path()).unwrap();
        g.mkdir(".folder-mind/index").unwrap();
        let i = SearchIndex::open(&g).unwrap();
        let c = Chunk {
            id: "tcp".into(),
            path: "a.txt".into(),
            hash: "h".into(),
            location: "Z. 1".into(),
            text: "TCP transportiert Netzwerkpakete zuverlässig".into(),
            extracted_at: 0,
            extractor: "txt".into(),
        };
        i.rebuild(&[c]).unwrap();
        assert_eq!(i.search("Netzwerkpakete", 5).unwrap(), vec!["tcp"]);
        assert!(i.search("Quantenbanane", 5).unwrap().is_empty());
    }
}
