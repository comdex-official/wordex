use cosmwasm_std::{Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

//TODO: logo stuff

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct InstantiateMsg {
    pub denom: String,
    pub max_cap: u64,
}

// fn is_valid_name(name: &str) -> bool {
//     let bytes = name.as_bytes();
//     if bytes.len() < 3 || bytes.len() > 50 {
//         return false;
//     }
//     true
// }

// fn is_valid_symbol(symbol: &str) -> bool {
//     let bytes = symbol.as_bytes();
//     if bytes.len() < 3 || bytes.len() > 12 {
//         return false;
//     }
//     for byte in bytes.iter() {
//         if (*byte != 45) && (*byte < 65 || *byte > 90) && (*byte < 97 || *byte > 122) {
//             return false;
//         }
//     }
//     true
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreatePlayer{name: String, addr: Addr},
    StartGame{addr: Addr},
    EndGame{addr: Addr},
    UpdateGame{addr: Addr, game: bool, guess: bool, correct_guess: bool,
    wrong_guess:bool},
    RewardPlayer{addr: Addr},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    //query the player details as it is stored in db
    QueryPlayer{addr : Addr},
    QueryPlayerExists{addr : Addr},
}


