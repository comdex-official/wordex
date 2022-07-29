#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, DepsMut, Deps, Env, MessageInfo, Response, StdResult, Binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State,config, config_read, Player, player_bank, player_bank_read};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:wordex";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    let state = State {
        creator: info.clone().sender,
        denom: msg.denom,
        minted_tokens: 0,
        games_played: 0,
        max_cap: msg.max_cap,
        curr_id: 0,
        players: None,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    config(deps.storage).save(&state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePlayer {name} => create_player(deps, info, name),
        ExecuteMsg::StartGame {game_words} => start_game(deps, _env,info, game_words),
        ExecuteMsg::UpdateGame { game, guess, game_won , correct_guess, wrong_guess} => update_game(deps, info, game, guess, game_won, correct_guess, wrong_guess),//update guesses, sets and won games
        ExecuteMsg::RewardPlayer{} => reward_player(deps, info),
        ExecuteMsg::EndGame{} => end_game(deps, info),
    }
}

//if game is not ongoing and time is not over, new game can't be started
//if time is over, new game can surely be started.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryPlayer {addr} =>
        {   to_binary(&player_bank_read(deps.storage).may_load(addr.as_bytes())?) },
        QueryMsg::QueryPlayerExists { addr } =>
        {
            //read the state
            let api = deps.api;
            let state = config_read(deps.storage).load()?;
            let addr_in_native_form = api.addr_validate(&addr)?;
            let res = match state.players{
                None => false,
                Some(x) => x.contains(&addr_in_native_form)
            };
            to_binary(&res)
        },
        QueryMsg::QueryPlayerWord { addr, pos} => 
        {
            //read wordset generated after initiation of game
            let player = player_bank_read(deps.storage).load(addr.as_bytes())?;
            let words = player.set_words.unwrap();
            let word = words[(pos-1) as usize].clone();
            to_binary(&word)
        },
        QueryMsg::QueryCorrectGuess { addr, guessed, pos } =>
        {
            //read wordset generated after initiation of game
            let player = player_bank_read(deps.storage).load(addr.as_bytes())?;

            //this stores the corresponding letters matchings
            let mut matches = Vec::new();

            //the words for the set 
            let set_words = player.set_words.unwrap();
            //the word that needed to be guessed
            let to_be_guessed = set_words[(pos-1) as usize].clone();

            //now iterate over the strings and check equality
            for (i, c) in to_be_guessed.chars().enumerate() {
                let b: u8 = guessed.as_bytes()[i];
                let c2: char = b as char;
                matches.push(c == c2);
            };
            return to_binary(&matches);
        }
    }
}

pub fn create_player(deps: DepsMut, info: MessageInfo, name: String) -> Result<Response, ContractError> {
    //read the state of the contract to get current players and curr id
    let mut state = config_read(deps.storage).load()?;

    //create new player struct
    let player = Player{
        name,
        address: info.sender.clone(),
        id: state.curr_id,
        balance: 0,
        prev_correct_guesses: 0,
        prev_wrong_guesses: 0,
        rem_games_set: 0,
        guesses_rem: 0,
        games_won_in_set:0,
        time_renewed: None,
        game_ongoing: false,
        set_words: None,
    };

    //read playerinfo
    let key = info.sender.as_str().as_bytes();

    //saving player manager data
    player_bank(deps.storage).save(key, &player)?;

    //changing state changes
    state.curr_id += 1;
    state.players = match state.players{
        None => Some(vec![info.sender.clone()]),
        Some(mut v) => {
            v.append(&mut(vec![info.sender.clone()])); Some(v)
        }
    };

    //saving state data
    config(deps.storage).save(&state)?;

    //return response
    Ok(Response::default())

}


fn start_game(deps: DepsMut, env: Env, info: MessageInfo, game_words: Vec<String>) -> Result<Response, ContractError>{
    //read playerinfo
    let key = info.sender.as_str().as_bytes();

    //read all the players present
    let mut player = player_bank_read(deps.storage).load(key)?;


    //details to be set at start of game
    player.game_ongoing = true;
    player.rem_games_set = 5;
    player.guesses_rem = 18;
    player.games_won_in_set = 0;    
    
    //starting the game also resets the time to renew
    //set it to current blocktime
    player.time_renewed = Some(env.block.time);    
    
    //starting the game also provides 5 random words from frontend and stores
    player.set_words = Some(game_words);

    //save the details
    player_bank(deps.storage).save(key, &player)?;


    //read state details
    let mut state = config_read(deps.storage).load()?;
    //update number of games played
    state.games_played += 1;
    //saving state data
    config(deps.storage).save(&state)?;

    Ok(Response::default())

}

pub fn update_game(deps: DepsMut, info: MessageInfo, game: u64, guess: u64, game_won:u64, correct_guess: u64, wrong_guess: u64) -> Result<Response, ContractError>{
    //read playerinfo
    let key = info.sender.as_str().as_bytes();

    //read all the players present
    let mut player = player_bank_read(deps.storage).load(key)?;

    //update relevant details
    //multiple parameters can be changed simultaneously, others will be zero
    player.guesses_rem -= guess;
    player.rem_games_set -= game;
    player.games_won_in_set += game_won;
    player.prev_correct_guesses += correct_guess;
    player.prev_wrong_guesses += wrong_guess;

    //save the details
    player_bank(deps.storage).save(key, &player)?;

    Ok(Response::default())
}



pub fn end_game(deps: DepsMut, info: MessageInfo)
-> Result<Response, ContractError>{
    //read playerinfo
    let key = info.sender.as_str().as_bytes();

    //read all the players present
    let mut player = player_bank_read(deps.storage).load(key)?;

    //make game not ongoing, rem games = 0 and rem guesses = 0
    player.game_ongoing = false;
    player.rem_games_set = 0;
    player.guesses_rem = 0;
    player.games_won_in_set = 0;
    player.time_renewed = None;
    player.set_words = None;

    //save the details
    player_bank(deps.storage).save(key, &player)?;

    Ok(Response::default())
}

//incase one wins the player is rewarded and the game is ended
pub fn reward_player(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError>{
     //read playerinfo
    let key = info.sender.as_str().as_bytes();

    //read all the players present
    let mut player = player_bank_read(deps.storage).load(key)?;

    //check whether player has played all five games and won all five games
    //also this must have been checked in front end
    let games_played = 5 - player.clone().rem_games_set; 
    if games_played < 5 {
        return Err(ContractError::AllGamesNotPlayed (games_played))
    }

    let won_games_in_set = player.clone().games_won_in_set;
    if won_games_in_set < 5 {
        return Err(ContractError::AllGamesNotWon (won_games_in_set))
    }

    //calculate how many moves the player made to reach to the winning postion
    let used_guesses = 18 - player.clone().guesses_rem;
    //each game in set would take minimum of 1 guess to arrive correctly
    if used_guesses <= 4{
        return Err(ContractError::MinGuessNotCrossed (used_guesses))
    }

    //calculate reward to be given
    let reward = 25 as u64/(used_guesses-4) as u64;

    //read state 
    let mut state = config_read(deps.storage).load()?;
    //check for minted tokens
    let minted_tokens = state.minted_tokens;
    if minted_tokens+reward > state.max_cap{
        return Err(ContractError::MaxCapReached{})
    }

    //mint excess tokens in state config
    state.minted_tokens += reward;
    //store new store config
    config(deps.storage).save(&state)?;


    //update player balance; increase by reward points
    player.balance += reward;

    //make game not ongoing, rem games = 0 and rem guesses = 0
    player.game_ongoing = false;
    player.rem_games_set = 0;
    player.guesses_rem = 0;
    player.games_won_in_set = 0;
    player.time_renewed = None;
    player.set_words = None;

    //save the details
    player_bank(deps.storage).save(key, &player)?;
    Ok(Response::default())


}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{ coins, Addr};

    const TEST_CREATOR: &str = "creator";
    pub const TOKEN: &str = "wdx";

    fn init_msg() -> InstantiateMsg {
        InstantiateMsg {
            denom: String::from(TOKEN),
            max_cap: 1000
        }
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = init_msg();
        let info = mock_info(TEST_CREATOR, &coins(2, TOKEN));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let state = config_read(&mut deps.storage).load().unwrap();
        assert_eq!(
            state,
            State {
                creator: Addr::unchecked(TEST_CREATOR),
                denom: String::from("wdx"),
                minted_tokens: 0,
                games_played: 0,
                max_cap: 1000,
                curr_id: 0,
                players: None,
            }
        );
    }

//     #[test]
//     fn test_create_player() {
//         let mut deps = mock_dependencies();

//         let msg = ExecuteMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Increment {};
//         let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // should increase counter by 1
//        
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }

// //     #[test]
// //     fn reset() {
// //         let mut deps = mock_dependencies(&coins(2, "token"));

// //         let msg = InstantiateMsg { count: 17 };
// //         let info = mock_info("creator", &coins(2, "token"));
// //         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

// //         // beneficiary can release it
// //         let unauth_info = mock_info("anyone", &coins(2, "token"));
// //         let msg = ExecuteMsg::Reset { count: 5 };
// //         let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
// //         match res {
// //             Err(ContractError::Unauthorized {}) => {}
// //             _ => panic!("Must return unauthorized error"),
// //         }

// //         // only the original creator can reset the counter
// //         let auth_info = mock_info("creator", &coins(2, "token"));
// //         let msg = ExecuteMsg::Reset { count: 5 };
// //         let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

// //         // should now be 5
// //         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
// //         let value: CountResponse = from_binary(&res).unwrap();
// //         assert_eq!(5, value.count);
// //     }
}
