use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.

    #[error("Maximum mintable limit exceeded")]
    MaxCapReached{},

    #[error("Min of 5 guesses has to be used ")]
    MinGuessNotCrossed(u64),
    #[error("All games in set has to be played; played_games ")]
    AllGamesNotPlayed(u64),
    #[error("All games in set has to be won ")]
    AllGamesNotWon(u64),
}
