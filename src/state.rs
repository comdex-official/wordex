use std::time::SystemTime;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Storage};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read,Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};

static CONFIG_KEY: &[u8] = b"config";
static PLAYER_KEY: &[u8] = b"player";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub creator: Addr,
    pub denom: String,
    pub games_played: u64,
    pub minted_tokens: i64,
    pub max_cap: u64,
    pub curr_id:u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Player {
    pub name: String,
    pub address: Addr,
    pub id: u64,
    pub balance: Option<Vec<OurCoin>>,
    pub prev_correct_guesses: u64,
    pub prev_wrong_guesses: u64,
    pub rem_games_set: u64,
    pub guesses_rem: u64,
    pub time_to_renew: Option<SystemTime>,
    pub game_ongoing: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OurCoin {
    pub denom: String,
    pub amount: f64,
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PlayerInfo {
    pub players: Option<Vec<(Addr,Player)>>,
    pub games_ongoing: u64,
}

impl Default for PlayerInfo{
    fn default() -> Self {
        PlayerInfo { players: None, games_ongoing: 0 }
    }
}

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn player_bank(storage: &mut dyn Storage) -> Bucket<PlayerInfo> {
    bucket(storage, PLAYER_KEY)
}

pub fn player_bank_read(storage: &dyn Storage) -> ReadonlyBucket<PlayerInfo> {
    bucket_read(storage, PLAYER_KEY)
}


