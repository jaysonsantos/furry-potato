use std::{
    fmt::{Debug, Display, Formatter},
    result,
};

use async_stream::stream;
use futures::{Stream, TryStreamExt};
use sled::{Mode, Tree};
use sync::mpsc;
use tokio::{sync, task};

use crate::{
    errors::{Data, OpeningStorage, Result},
    ToFromStorage,
};

const DEFAULT_NUMBER_OF_SHARDS: usize = 10;

pub struct Sled {
    number_of_shards: usize,
    shards: Vec<Tree>,
}

impl Sled {
    pub fn new() -> Result<Self> {
        Ok(Self::new_internal()?)
    }

    fn new_internal() -> result::Result<Self, OpeningStorage> {
        let db = sled::Config::new()
            .temporary(true)
            .mode(Mode::HighThroughput)
            .open()
            .map_err(|e| OpeningStorage {
                message: "failed to open database".into(),
                source: e,
            })?;
        let shards: sled::Result<Vec<_>> = (0..DEFAULT_NUMBER_OF_SHARDS)
            .map(|shard| db.open_tree(format!("db-shard-{shard}")))
            .collect();
        let shards = shards.map_err(|e| OpeningStorage {
            message: "failed to open database trees".into(),
            source: e,
        })?;
        Ok(Self {
            number_of_shards: DEFAULT_NUMBER_OF_SHARDS,
            shards,
        })
    }

    fn get_shard(&self, partition: usize) -> &sled::Tree {
        let shard_number = partition % self.number_of_shards;
        // This should be safe because it is a circular array
        unsafe { self.shards.get_unchecked(shard_number) }
    }

    pub fn create_or_update<T, F>(&self, entity: &T, update_fn: F) -> Result<()>
    where
        T: ToFromStorage,
        F: FnOnce(&T, &T) -> result::Result<T, Data>,
    {
        Ok(self.create_or_update_internal(entity, update_fn)?)
    }

    fn create_or_update_internal<T, F>(&self, entity: &T, update_fn: F) -> result::Result<(), Data>
    where
        T: ToFromStorage,
        F: FnOnce(&T, &T) -> result::Result<T, Data>,
    {
        let shard = self.get_shard(entity.partition());
        let primary_key = &entity.primary_key();
        let existing = shard
            .get(&primary_key)
            .map_err(|e| Data::Sled(format!("failed to get data for {}", primary_key), e))?;
        let existing = if let Some(existing) = existing {
            existing
        } else {
            let entity = entity.to_bytes();
            shard
                .insert(primary_key, entity)
                .map_err(|e| Data::Sled("failed to insert data".to_string(), e))?;
            return Ok(());
        };
        let decoded_existing = T::from_bytes(existing.as_ref())?;
        let new = update_fn(&decoded_existing, entity)?;
        shard
            .compare_and_swap(primary_key, Some(existing), Some(new.to_bytes()))
            .map_err(|e| Data::Sled("sled configuration error".into(), e))??;
        Ok(())
    }

    pub fn get<T: ToFromStorage>(&self, partial: &T) -> Result<T> {
        Ok(self.get_internal(partial)?)
    }

    fn get_internal<T: ToFromStorage>(&self, partial: &T) -> result::Result<T, Data> {
        let shard = self.get_shard(partial.partition());
        let primary_key = &partial.primary_key();
        let data = shard
            .get(primary_key)
            .map_err(|e| Data::Sled(format!("failed to get data for key {}", primary_key), e))?;
        if let Some(data) = data {
            T::from_bytes(data.as_ref())
        } else {
            Err(Data::KeyNotFound(primary_key.clone()))
        }
    }

    pub async fn list<T: ToFromStorage + Debug + 'static>(
        &self,
        prefix: &'static str,
    ) -> impl Stream<Item = Result<T>> + '_ {
        self.list_internal(prefix).await.map_err(|e| e.into())
    }

    pub async fn list_internal<'a, T: ToFromStorage + Debug + 'static>(
        &self,
        prefix: &'static str,
    ) -> impl Stream<Item = result::Result<T, Data>> + 'a {
        let shards: Vec<Tree> = self.shards.to_vec();
        let (tx, mut rx) = mpsc::channel(10);
        let handler = task::spawn_blocking(move || {
            let tx = tx.clone();
            for shard in shards {
                let iter = shard.scan_prefix(prefix).values();
                for e in iter {
                    let rv = e
                        .map_err(|e| {
                            Data::Sled(format!("failed to list keys from prefix {}", prefix), e)
                        })
                        .and_then(|e| T::from_bytes(e.as_ref()));
                    tx.blocking_send(rv).expect("failed to send results");
                }
            }
        });
        stream! {
            while let Some(result) = rx.recv().await {
                yield result;
            }
            handler.await.expect("x");
        }
    }
}

impl Display for Sled {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("sled")
    }
}
