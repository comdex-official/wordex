use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

//TODO: logo stuff

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct InstantiateMsg {
    pub denom: String,
    pub max_cap: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreatePlayer{name: String},
    StartGame{game_words: Vec<String>},
    EndGame{},
    UpdateGame{game: u64, guess: u64, game_won: u64, correct_guess: u64,
    wrong_guess:u64},
    RewardPlayer{},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    //query the player details as it is stored in db
    QueryPlayer{addr : String},
    QueryPlayerExists{addr : String},
    QueryPlayerWord{addr: String, pos: u64}, //to know what word is to be guessed
    QueryCorrectGuess{addr: String, guessed: String, pos: u64}, //matching corresponding letters 
}


