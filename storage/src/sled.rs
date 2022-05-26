use std::result;

use sled::{Db, Mode, Tree};

use crate::{
    errors::{OpeningStorage, Result},
    Storage,
};

const DEFAULT_NUMBER_OF_SHARDS: usize = 10;

pub struct Sled {
    number_of_shards: usize,
    db: Db,
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
            db,
            shards,
        })
    }
}

impl Storage for Sled {
    fn name(&self) -> &str {
        "sled"
    }
}
