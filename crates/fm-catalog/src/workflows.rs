use super::*;
use rusqlite::OptionalExtension;
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone, Copy)]
pub enum Collection {
    Agents,
    Inbox,
    Rules,
    Examples,
    Activity,
    Proposals,
}
impl Collection {
    fn key(self) -> &'static str {
        match self {
            Self::Agents => "agents",
            Self::Inbox => "inbox",
            Self::Rules => "rules",
            Self::Examples => "examples",
            Self::Activity => "activity",
            Self::Proposals => "proposals",
        }
    }
}
impl Catalog {
    pub fn entry<T: DeserializeOwned>(
        &self,
        collection: Collection,
        id: &str,
    ) -> Result<Option<T>> {
        let c = self.connection.lock().map_err(error)?;
        let data: Option<String> = c
            .query_row(
                "SELECT data FROM workflow_entries WHERE collection=?1 AND id=?2",
                params![collection.key(), id],
                |r| r.get(0),
            )
            .optional()
            .map_err(error)?;
        data.map(|s| serde_json::from_str(&s).map_err(error))
            .transpose()
    }
    pub fn entries<T: DeserializeOwned>(&self, collection: Collection) -> Result<Vec<T>> {
        self.entries_page(collection, u32::MAX, 0)
    }
    pub fn entries_page<T: DeserializeOwned>(
        &self,
        collection: Collection,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<T>> {
        let c = self.connection.lock().map_err(error)?;
        let mut q = c.prepare("SELECT data FROM workflow_entries WHERE collection=?1 ORDER BY updated_at DESC,rowid DESC LIMIT ?2 OFFSET ?3").map_err(error)?;
        let values = q
            .query_map(params![collection.key(), limit, offset], |r| {
                r.get::<_, String>(0)
            })
            .map_err(error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(error)?;
        values
            .into_iter()
            .map(|s| serde_json::from_str(&s).map_err(error))
            .collect()
    }
    pub fn entry_count(&self, collection: Collection) -> Result<u64> {
        self.connection
            .lock()
            .map_err(error)?
            .query_row(
                "SELECT COUNT(*) FROM workflow_entries WHERE collection=?1",
                [collection.key()],
                |r| r.get(0),
            )
            .map_err(error)
    }
    pub fn put_entry<T: Serialize>(
        &self,
        collection: Collection,
        id: &str,
        value: &T,
    ) -> Result<()> {
        self.execute("INSERT INTO workflow_entries(collection,id,data,updated_at) VALUES(?1,?2,?3,?4) ON CONFLICT(collection,id) DO UPDATE SET data=excluded.data,updated_at=excluded.updated_at",
            &[&collection.key(), &id, &serde_json::to_string(value).map_err(error)?, &fm_core::now()])?;
        Ok(())
    }
    pub fn delete_entry(&self, collection: Collection, id: &str) -> Result<()> {
        self.execute(
            "DELETE FROM workflow_entries WHERE collection=?1 AND id=?2",
            &[&collection.key(), &id],
        )?;
        Ok(())
    }
}
