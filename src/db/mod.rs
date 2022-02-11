use anyhow::{Context, Result};
use heed::{types::SerdeBincode, Database, Env, EnvOpenOptions};
use poise::serenity_prelude::{ChannelId, GuildId};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Serialize, Debug)]
pub struct Key {
    guild: GuildId,
    source: ChannelId,
}

#[derive(Debug)]
pub struct DbError(pub String);

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Database Error: {}", self.0)
    }
}

impl Error for DbError {}

pub struct CrossoverDb {
    pub env: Env,
    pub db: Database<SerdeBincode<Key>, SerdeBincode<ChannelId>>,
}

impl CrossoverDb {
    pub fn new(path: &str) -> Result<Self> {
        fs::create_dir_all(path).context("Failed to create db path.")?;
        let env = EnvOpenOptions::new()
            .max_dbs(3000)
            .open(Path::new(path))
            .map_err(|e| DbError(e.to_string()))?;
        let db: Database<SerdeBincode<Key>, SerdeBincode<ChannelId>> = env
            .create_database(Some("crossovers"))
            .map_err(|e| DbError(e.to_string()))?;
        Ok(CrossoverDb { env, db })
    }

    pub fn get_crossover(&self, guild: GuildId, source: ChannelId) -> Result<Option<ChannelId>> {
        let key = Key { guild, source };
        let rtxn = self.env.read_txn().map_err(|e| DbError(e.to_string()))?;
        let ret = self
            .db
            .get(&rtxn, &key)
            .map_err(|e| DbError(e.to_string()))?;
        rtxn.commit().map_err(|e| DbError(e.to_string()))?;
        Ok(ret)
    }

    pub fn put_crossover(
        &self,
        guild: GuildId,
        source: ChannelId,
        target: ChannelId,
    ) -> Result<()> {
        let key = Key { guild, source };
        let mut wtxn = self.env.write_txn().map_err(|e| DbError(e.to_string()))?;
        self.db
            .put(&mut wtxn, &key, &target)
            .map_err(|e| DbError(e.to_string()))?;
        wtxn.commit().map_err(|e| DbError(e.to_string()))?;
        Ok(())
    }

    pub fn remove_crossover(
        &self,
        guild: GuildId,
        source: ChannelId,
        target: ChannelId,
    ) -> Result<bool> {
        let key = Key { guild, source };
        let mut wtxn = self.env.write_txn().map_err(|e| DbError(e.to_string()))?;
        let deleted = self
            .db
            .delete(&mut wtxn, &key)
            .map_err(|e| DbError(e.to_string()))?;
        wtxn.commit().map_err(|e| DbError(e.to_string()))?;
        Ok(deleted)
    }

    pub fn get_all(&self, guild: GuildId) -> Result<Vec<(ChannelId, ChannelId)>> {
        let rtxn = self.env.read_txn().map_err(|e| DbError(e.to_string()))?;
        let ret = self.db.iter(&rtxn).map_err(|e| DbError(e.to_string()))?;
        let list = ret
            .filter_map(|res| res.ok())
            .filter(|row| row.0.guild == guild)
            .map(|(key, target)| (key.source, target))
            .collect();
        rtxn.commit().map_err(|e| DbError(e.to_string()))?;
        Ok(list)
    }
}
