#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, DepsMut, Env, MessageInfo, Response, StdResult, Binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State,config, config_read, Player, player_bank, player_bank_read, OurCoin};

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
        ExecuteMsg::StartGame {} => start_game(deps, info),
        ExecuteMsg::UpdateGame { game, guess, correct_guess, wrong_guess} => update_game(deps, info, game, guess, correct_guess, wrong_guess),//update guesses, sets
        ExecuteMsg::RewardPlayer{} => reward_player(deps, info),
        ExecuteMsg::EndGame{} => end_game(deps, info),
    }
}

//if game is not ongoing and time is not over, new game can't be started
//if time is over, new game can surely be started.

pub fn query(deps: DepsMut, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryPlayer {addr} =>
        { to_binary(&player_bank_read(deps.storage).may_load(addr.as_str().as_bytes())?) },
        QueryMsg::QueryPlayerExists { addr } =>
        {
            //read the state
            let state = config_read(deps.storage).load()?;
            let res = match state.players{
                None => false,
                Some(x) => x.contains(&addr)
            };
            to_binary(&res)
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
        id: state.curr_id+1,
        balance: None,
        prev_correct_guesses: 0,
        prev_wrong_guesses: 0,
        rem_games_set: 0,
        guesses_rem: 0,
        time_to_renew: None,
        game_ongoing: false,
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


fn start_game(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError>{
    //read playerinfo
    let key = info.sender.as_str().as_bytes();

    //read all the players present
    let mut player = player_bank_read(deps.storage).load(key)?;

    //details to be set at start of game
    player.game_ongoing = true;
    player.rem_games_set = 5;
    player.guesses_rem = 18;

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

pub fn update_game(deps: DepsMut, info: MessageInfo, game: bool, guess: bool, correct_guess: bool, wrong_guess: bool) -> Result<Response, ContractError>{
    //read playerinfo
    let key = info.sender.as_str().as_bytes();

    //read all the players present
    let mut player = player_bank_read(deps.storage).load(key)?;

    //update relevant details
    if guess{
        player.guesses_rem -= 1;
    }
    else if game {
        player.rem_games_set -= 1;
    }
    else if correct_guess{
        player.prev_correct_guesses += 1;
    }
    else if wrong_guess{
        player.prev_wrong_guesses += 1;
    }

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

    //save the details
    player_bank(deps.storage).save(key, &player)?;

    Ok(Response::default())
}

pub fn reward_player(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError>{
     //read playerinfo
    let key = info.sender.as_str().as_bytes();

    //read all the players present
    let mut player = player_bank_read(deps.storage).load(key)?;

    //calculate how many moves the player made to reach to the winning postion
    let used_guesses = 18 - player.guesses_rem;

    //calculate reward to be given
    let reward = 25 as u32/(used_guesses-4) as u32;

    player.balance =  match player.balance{
        None => Some(OurCoin{denom:String::from("wdx"),amount:reward}),
        Some(x) => {
            Some(OurCoin{ denom: String::from("wdx"), amount: x.amount+reward})
        }
    };

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
