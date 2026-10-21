use fm_core::{Chunk, Document, Guard, Result, error};
use rusqlite::{Connection, params};
use std::sync::Mutex;
pub struct Catalog {
    connection: Mutex<Connection>,
}
impl Catalog {
    pub fn open(guard: &Guard) -> Result<Self> {
        let c =
            Connection::open(guard.resolve(".folder-mind/folder-mind.sqlite")?).map_err(error)?;
        c.execute_batch(include_str!("../../../migrations/001_initial.sql"))
            .map_err(error)?;
        c.execute_batch(include_str!("../../../migrations/002_source_archive.sql"))
            .map_err(error)?;
        Ok(Self {
            connection: Mutex::new(c),
        })
    }
    pub fn documents(&self) -> Result<Vec<Document>> {
        let c = self.connection.lock().map_err(error)?;
        let mut q = c
            .prepare("SELECT path,hash,size,status,error FROM documents ORDER BY path")
            .map_err(error)?;
        q.query_map([], |r| {
            Ok(Document {
                path: r.get(0)?,
                hash: r.get(1)?,
                size: r.get(2)?,
                status: r.get(3)?,
                error: r.get(4)?,
            })
        })
        .map_err(error)?
        .collect::<std::result::Result<_, _>>()
        .map_err(error)
    }
    pub fn record(&self, d: &Document, chunks: &[Chunk]) -> Result<()> {
        fm_core::DocumentStatus::parse(&d.status)?;
        let mut c = self.connection.lock().map_err(error)?;
        let tx = c.transaction().map_err(error)?;
        tx.execute("INSERT INTO document_versions(path,hash,created_at) SELECT path,hash,?2 FROM documents WHERE path=?1 AND hash<>?3 AND hash<>''",params![d.path,fm_core::now(),d.hash]).map_err(error)?;
        tx.execute("INSERT INTO documents(path,hash,size,status,error) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(path) DO UPDATE SET hash=excluded.hash,size=excluded.size,status=excluded.status,error=excluded.error",params![d.path,d.hash,d.size,d.status,d.error]).map_err(error)?;
        tx.execute("DELETE FROM chunks WHERE path=?1", [&d.path])
            .map_err(error)?;
        for chunk in chunks {
            tx.execute(
                "INSERT INTO chunks(id,path,data) VALUES(?1,?2,?3)",
                params![
                    chunk.id,
                    chunk.path,
                    serde_json::to_string(chunk).map_err(error)?
                ],
            )
            .map_err(error)?;
        }
        tx.execute("INSERT INTO jobs(path,status,error) VALUES(?1,?2,?3) ON CONFLICT(path) DO UPDATE SET status=excluded.status,error=excluded.error",params![d.path,d.status,d.error]).map_err(error)?;
        tx.execute("INSERT OR IGNORE INTO chunk_versions(id,data) SELECT id,data FROM chunks WHERE path=?1",[&d.path]).map_err(error)?;
        tx.commit().map_err(error)
    }
    pub fn source(&self, id: &str) -> Result<Option<Chunk>> {
        use rusqlite::OptionalExtension;
        let c = self.connection.lock().map_err(error)?;
        let value: Option<String> = c
            .query_row("SELECT data FROM chunk_versions WHERE id=?1", [id], |r| {
                r.get(0)
            })
            .optional()
            .map_err(error)?;
        value
            .map(|s| serde_json::from_str(&s).map_err(error))
            .transpose()
    }
    pub fn chunks(&self) -> Result<Vec<Chunk>> {
        let c = self.connection.lock().map_err(error)?;
        let mut q = c.prepare("SELECT data FROM chunks").map_err(error)?;
        let values = q
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(error)?;
        values
            .into_iter()
            .map(|s| serde_json::from_str(&s).map_err(error))
            .collect()
    }
    pub fn execute(&self, sql: &str, args: &[&dyn rusqlite::ToSql]) -> Result<usize> {
        self.connection
            .lock()
            .map_err(error)?
            .execute(sql, args)
            .map_err(error)
    }
    pub fn rows(&self, sql: &str) -> Result<Vec<Vec<String>>> {
        let c = self.connection.lock().map_err(error)?;
        let mut q = c.prepare(sql).map_err(error)?;
        let count = q.column_count();
        q.query_map([], |r| (0..count).map(|i| r.get::<_, String>(i)).collect())
            .map_err(error)?
            .collect::<std::result::Result<_, _>>()
            .map_err(error)
    }
    pub fn restore(&self, guard: &Guard, path: &str) -> Result<()> {
        let source = guard.resolve(path)?;
        let check =
            Connection::open_with_flags(&source, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .map_err(error)?;
        let integrity: String = check
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .map_err(error)?;
        if integrity != "ok" {
            return Err(error("Die Sicherungsdatenbank ist beschädigt"));
        }
        drop(check);
        let mut c = self.connection.lock().map_err(error)?;
        c.restore(
            rusqlite::MAIN_DB,
            source,
            None::<fn(rusqlite::backup::Progress)>,
        )
        .map_err(error)?;
        c.execute_batch(include_str!("../../../migrations/001_initial.sql"))
            .map_err(error)?;
        c.execute_batch(include_str!("../../../migrations/002_source_archive.sql"))
            .map_err(error)?;
        Ok(())
    }
    pub fn snapshot(&self, guard: &Guard, path: &str) -> Result<()> {
        let c = self.connection.lock().map_err(error)?;
        c.backup(rusqlite::MAIN_DB, guard.resolve(path)?, None)
            .map_err(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_sources_survive_reprocessing_and_deletion() {
        let t = tempfile::tempdir().unwrap();
        let g = Guard::new(t.path()).unwrap();
        fm_core::initialize(&g, &fm_core::Config::default()).unwrap();
        let c = Catalog::open(&g).unwrap();
        let mut d = Document {
            path: "unterlagen/x.txt".into(),
            hash: "old".into(),
            size: 1,
            status: "indiziert".into(),
            error: String::new(),
        };
        let mut chunk = Chunk {
            id: "old-id".into(),
            path: d.path.clone(),
            hash: d.hash.clone(),
            location: "Z. 1".into(),
            text: "Alter Text".into(),
            extracted_at: 0,
            extractor: "txt".into(),
        };
        c.record(&d, &[chunk.clone()]).unwrap();
        d.hash = "new".into();
        chunk.id = "new-id".into();
        chunk.hash = d.hash.clone();
        chunk.text = "Neuer Text".into();
        c.record(&d, &[chunk]).unwrap();
        assert_eq!(c.chunks().unwrap().len(), 1);
        assert_eq!(c.source("old-id").unwrap().unwrap().text, "Alter Text");
        c.execute("DELETE FROM documents", &[]).unwrap();
        assert!(c.chunks().unwrap().is_empty());
        assert_eq!(c.source("old-id").unwrap().unwrap().text, "Alter Text");
    }
}
