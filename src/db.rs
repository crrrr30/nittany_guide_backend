use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Database {
    documents: sled::Tree, // stores all docs
}

impl Database {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, sled::Error> {
        let db = sled::Config::default()
            .use_compression(true)
            .compression_factor(10)
            .path(path)
            .open()?;

        let documents = db.open_tree("documents")?;

        Ok(Self { documents })
    }

    pub fn insert_document(&self, doc: &DocumentRecord) -> Result<DocumentId, Error> {
        let id = Self::calculate_document_id(&doc.content);
        Self::insert_and_transform::<_, _, DocumentRecord>(&self.documents, &id, doc)?;
        Ok(id)
    }

    pub fn get_document(&self, id: &DocumentId) -> Result<Option<DocumentRecord>, Error> {
        Self::get_and_transform(&self.documents, id)
    }

    pub fn remove_document(&self, id: &DocumentId) -> Result<Option<DocumentRecord>, Error> {
        Self::remove(&self.documents, id)
    }

    pub fn contains_document(&self, id: &DocumentId) -> Result<bool, Error> {
        Self::contains_key(&self.documents, id)
    }

    fn calculate_document_id(content: &str) -> DocumentId {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        DocumentId(hasher.finalize().into())
    }

    fn insert_and_transform<K, V, T>(
        store: &sled::Tree,
        key: K,
        value: V,
    ) -> Result<Option<T>, Error>
    where
        K: serde::Serialize,
        V: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        let previous = store.insert(
            bincode::serialize(&key)?,
            bincode::serialize(&value)?,
        )?;
        Ok(previous
            .map(|p| bincode::deserialize(&p))
            .transpose()?)
    }

    fn get_and_transform<K, V>(store: &sled::Tree, key: K) -> Result<Option<V>, Error>
    where
        K: serde::Serialize,
        V: serde::de::DeserializeOwned,
    {
        let value = store.get(bincode::serialize(&key)?)?;
        Ok(value.map(|p| bincode::deserialize(&p)).transpose()?)
    }

    fn remove<K, V>(store: &sled::Tree, key: K) -> Result<Option<V>, Error>
    where
        K: serde::Serialize,
        V: serde::de::DeserializeOwned,
    {
        Ok(store
            .remove(bincode::serialize(&key)?)?
            .map(|p| bincode::deserialize(&p))
            .transpose()?)
    }

    fn contains_key<K>(store: &sled::Tree, key: K) -> Result<bool, Error>
    where
        K: serde::Serialize,
    {
        store
            .contains_key(bincode::serialize(&key)?)
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(pub [u8; 32]);

impl AsRef<[u8; 32]> for DocumentId {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentRecord {
    pub content: String,
    pub created: DateTime<Utc>,
}