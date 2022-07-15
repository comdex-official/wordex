use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Storage, Timestamp};
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
    pub minted_tokens: u64,
    pub max_cap: u64,
    pub curr_id:u64,
    pub players: Option<Vec<Addr>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Player {
    pub name: String,
    pub address: Addr,
    pub id: u64,
    pub balance: u64,
    pub prev_correct_guesses: u64,
    pub prev_wrong_guesses: u64,
    pub rem_games_set: u64,
    pub guesses_rem: u64,
    pub games_won_in_set: u64,
    pub time_renewed: Option<Timestamp>,
    pub game_ongoing: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Gamewords {
    pub words: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Allwords{
    pub allwords : Vec<String>,
}


pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn player_bank(storage: &mut dyn Storage) -> Bucket<Player> {
    bucket(storage, PLAYER_KEY)
}

pub fn player_bank_read(storage: &dyn Storage) -> ReadonlyBucket<Player> {
    bucket_read(storage, PLAYER_KEY)
}


pub fn words_bank(storage: &mut dyn Storage) -> Bucket<Gamewords> {
    bucket(storage, PLAYER_KEY)
}

pub fn words_bank_read(storage: &dyn Storage) -> ReadonlyBucket<Gamewords> {
    bucket_read(storage, PLAYER_KEY)
}


