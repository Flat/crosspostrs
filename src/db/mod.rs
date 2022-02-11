use anyhow::{Context, Result};
use bincode::{config, Decode, Encode};
use poise::serenity_prelude::{ChannelId, GuildId};
use std::error::Error;
use std::fmt;
use std::fs;

#[derive(Encode, Decode, Debug)]
pub struct Key {
    guild: u64,
    source: u64,
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
    pub db: sled::Db,
}

impl CrossoverDb {
    pub fn new(path: &str) -> Result<Self> {
        fs::create_dir_all(path).context("Failed to create db path.")?;
        let db = sled::open(path).map_err(|e| DbError(e.to_string()))?;
        Ok(CrossoverDb { db })
    }

    pub fn get_crossover(&self, guild: GuildId, source: ChannelId) -> Result<Option<ChannelId>> {
        let guild = guild.0;
        let source = source.0;
        let key: Vec<u8> = bincode::encode_to_vec(Key { guild, source }, config::standard())?;
        let ret = bincode::decode_from_slice::<u64, _>(
            &self
                .db
                .get(key)?
                .ok_or_else(|| DbError("Unable to get from crossover from DB".to_string()))?[..],
            config::standard(),
        )?;
        let ret = ChannelId(ret.0);
        Ok(Some(ret))
    }

    pub fn put_crossover(
        &self,
        guild: GuildId,
        source: ChannelId,
        target: ChannelId,
    ) -> Result<()> {
        let guild = guild.0;
        let source = source.0;
        let target = target.0;
        let key: Vec<u8> = bincode::encode_to_vec(Key { guild, source }, config::standard())?;
        self.db
            .insert(key, bincode::encode_to_vec(target, config::standard())?)?;
        Ok(())
    }

    pub fn remove_crossover(
        &self,
        guild: GuildId,
        source: ChannelId,
        _target: ChannelId,
    ) -> Result<bool> {
        let guild = guild.0;
        let source = source.0;
        let key: Vec<u8> = bincode::encode_to_vec(Key { guild, source }, config::standard())?;
        let deleted = self.db.remove(key)?.is_some();
        Ok(deleted)
    }

    pub fn get_all(&self, guild: GuildId) -> Result<Vec<(ChannelId, ChannelId)>> {
        let list: Vec<(ChannelId, ChannelId)> = self
            .db
            .iter()
            .filter_map(|i| i.ok())
            .map(|x| {
                (
                    bincode::decode_from_slice::<Key, _>(&x.0, config::standard())
                        .unwrap()
                        .0,
                    ChannelId(
                        bincode::decode_from_slice::<u64, _>(&x.1, config::standard())
                            .unwrap()
                            .0,
                    ),
                )
            })
            .filter(|(key, _value)| key.guild == guild.0)
            .map(|(key, value)| (ChannelId(key.source), value))
            .collect();
        Ok(list)
    }
}
