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
        let mut c = self.connection.lock().map_err(error)?;
        let tx = c.transaction().map_err(error)?;
        tx.execute("INSERT INTO document_versions(path,hash,created_at) SELECT path,hash,?2 FROM documents WHERE path=?1 AND hash<>?3",params![d.path,fm_core::now(),d.hash]).map_err(error)?;
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
        tx.commit().map_err(error)
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
    pub fn snapshot(&self, guard: &Guard, path: &str) -> Result<()> {
        let c = self.connection.lock().map_err(error)?;
        c.backup(rusqlite::MAIN_DB, guard.resolve(path)?, None)
            .map_err(error)
    }
}
