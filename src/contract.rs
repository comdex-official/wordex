use chrono::DateTime;
use chrono::offset::Utc;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{attr, DepsMut, Deps, Env, MessageInfo, Response, Addr, StdResult, Binary};
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
        ExecuteMsg::CreatePlayer {name, addr} => create_player(deps, info, name, addr),
        ExecuteMsg::LoadPlayer { addr } => load_player(deps, info, addr),
        ExecuteMsg::StartGame {addr} => start_game(deps, info, addr),
        ExecuteMsg::UpdateGame {addr, game, guess, correct_guess, wrong_guess} => update_game(deps, info, addr, game, guess, correct_guess, wrong_guess),//update guesses, sets
        ExecuteMsg::RewardPlayer{addr} => reward_player(deps, info, addr),
        ExecuteMsg::EndGame{addr} => end_game(deps, info, addr),
    }
}

//if game is not ongoing and time is not over, new game can't be started
//if time is over, new game can surely be started.


#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryPlayer {addr: Addr} => ,
    }
}

pub fn create_player(deps: DepsMut, info: MessageInfo, name: String, addr: Addr) -> Result<Response, ContractError> {
    //read the state of the contract to get current players and curr id
    let mut state = config_read(deps.storage).load()?;

    //create new player struct
    let player = Player{
        name,
        address: addr.clone(),
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

    //read all the players present
    let mut player_manager = player_bank_read(deps.storage).may_load(key)?.unwrap_or_default();

    //these are the current players
    let cur_players = player_manager.players;

    //matching players to none or some value and appending new players
    player_manager.players =  match cur_players{
        None => Some(vec![(addr,player)]),
        Some(x) => Some([x,vec![(addr,player)]].concat())
    };

    //saving player manager data
    player_bank(deps.storage).save(key, &player_manager)?;

    //changing state changes
    state.curr_id += 1;

    //saving state data
    config(deps.storage).save(&state)?;

    //return response
    Ok(Response::default())

}


//load player details and send in response
pub fn load_player(deps: DepsMut, info: MessageInfo, addr: Addr)
-> Result<Response, ContractError>{
     //this stores that players details
    let player_detail = query_player_detail(deps, info, addr).unwrap();

    //get the coin balances info in a vec of pairs
    let mut coin_vec = Vec::new();

    let coin_info = match player_detail.balance.clone(){
        None => coin_vec,
        Some(x) => {
            for c in x.iter(){
                coin_vec.push((c.denom.clone(), c.amount.to_string()));
            }
            coin_vec
        }
    };

    //time to renew stored in date time format
    let time_stored: DateTime<Utc> = player_detail.time_to_renew.unwrap().into();

    let mut attributes = vec![
        attr("action", "load_player"),
        attr("name", &player_detail.name),
        attr("id", &player_detail.id.to_string()),
        attr("correct_guesses", &player_detail.prev_correct_guesses.to_string()),
        attr("wrong_guesses", &player_detail.prev_wrong_guesses.to_string()),
        attr("games_set", &player_detail.rem_games_set.to_string()),
        attr("guesses_rem", &player_detail.guesses_rem.to_string()),
        attr("renew_time", time_stored.to_rfc2822()),
    ];

    //concatenate coin details to attributes
    for c in coin_info.iter(){
        attributes.push(attr(&c.0,c.1));
    }

    let res = Response::new().add_attributes(attributes);

    Ok(res)

}


fn start_game(deps: DepsMut, info: MessageInfo, addr: Addr) -> Result<Response, ContractError>{
    //read playerinfo
    let key = info.sender.as_str().as_bytes();

    //read all the players present
    let player_manager = player_bank_read(deps.storage).may_load(key)?.unwrap();

    //this stores that players details
    let mut player_detail = None;
    let players = player_manager.players.as_ref().unwrap();

    //go through player base and find the relevant player's details
    for player in players.iter(){
        if player.0 == addr{
            player_detail = Some(player.1.clone());
            break;
        }
    }

    //details to be set at start of game
    player_detail.clone().unwrap().game_ongoing = true;
    player_detail.clone().unwrap().rem_games_set = 5;
    player_detail.unwrap().guesses_rem = 18;

    //save the details
    player_bank(deps.storage).save(key, &player_manager)?;

    //read state details
    let mut state = config_read(deps.storage).load()?;

    //update number of games played
    state.games_played += 1;

    //saving state data
    config(deps.storage).save(&state)?;

    Ok(Response::default())

}

pub fn update_game(deps: DepsMut, info: MessageInfo, addr: Addr, game: bool, guess: bool, correct_guess: bool, wrong_guess: bool) -> Result<Response, ContractError>{
    //read playerinfo
    let key = info.sender.as_str().as_bytes();

    //read all the players present
    let player_manager = player_bank_read(deps.storage).may_load(key)?.unwrap();

    //this stores that players details
    let mut player_detail = None;
    let players = player_manager.players.as_ref().unwrap();

    //go through player base and find the relevant player's details
    for player in players.iter(){
        if player.0 == addr{
            player_detail = Some(player.1.clone());
        }
    }

    //update relevant details
    if guess{
        player_detail.clone().unwrap().guesses_rem -= 1;
    }
    else if game {
        player_detail.clone().unwrap().rem_games_set -= 1;
    }
    else if correct_guess{
        player_detail.clone().unwrap().prev_correct_guesses += 1;
    }
    else if wrong_guess{
        player_detail.clone().unwrap().prev_wrong_guesses += 1;
    }

    //save the details
    player_bank(deps.storage).save(key, &player_manager)?;

    Ok(Response::default())
}



pub fn end_game(deps: DepsMut, info: MessageInfo, addr: Addr)
-> Result<Response, ContractError>{
    //read playerinfo
    let key = info.sender.as_str().as_bytes();

    //read all the players present
    let player_manager = player_bank_read(deps.storage).may_load(key)?.unwrap();

    //this stores that players details
    let mut player_detail = None;
    let players = player_manager.players.as_ref().unwrap();

    //go through player base and find the relevant player's details
    for player in players.iter(){
        if player.0 == addr{
            player_detail = Some(player.1.clone());
        }
    }

    //make game not ongoing, rem games = 0 and rem guesses = 0
    player_detail.clone().unwrap().game_ongoing = false;
    player_detail.clone().unwrap().rem_games_set = 0;
    player_detail.unwrap().guesses_rem = 0;

    //save the details
    player_bank(deps.storage).save(key, &player_manager)?;

    Ok(Response::default())
}

pub fn reward_player(deps: DepsMut, info: MessageInfo, addr: Addr) -> Result<Response, ContractError>{
     //read playerinfo
    let key = info.sender.as_str().as_bytes();

    //read all the players present
    let player_manager = player_bank_read(deps.storage).may_load(key)?.unwrap();

    //this stores that players details
    let mut player_detail = None;
    let players = player_manager.players.unwrap();

    //go through player base and find the relevant player's details
    for player in players.iter(){
        if player.0 == addr{
            player_detail = Some(player.1.clone());
        }
    }

    //calculate how many moves the player made to reach to the winning postion
    let used_guesses = 18 - player_detail.clone().unwrap().guesses_rem;

    //calculate reward to be given
    let reward = 25 as f64/(used_guesses-4) as f64;

    player_detail.clone().unwrap().balance =  match player_detail.clone().unwrap().balance{
        None => Some(vec![OurCoin{denom:String::from("wdx"),amount:reward}]),
        Some(x) => {
            for &c in x.iter(){
                if c.denom == String::from("wdx"){
                    c.amount += reward;
                }
            }
            Some(x)
        }
    };

    Ok(Response::default())
}


// function to load player details
pub fn query_player_detail(deps:DepsMut, info: MessageInfo ,addr: Addr) -> Result<Player, ContractError>{
    //read playerinfo
    let key = info.sender.as_str().as_bytes();

    //read all the players present
    let player_manager = player_bank_read(deps.storage).may_load(key)?.unwrap();

    //this stores that players details
    let mut player_detail = None;
    let players = player_manager.players.unwrap();

    //go through player base and find the relevant player's details
    for player in players.iter(){
        if player.0 == addr{
            player_detail = Some(player.1.clone());
            break;
        }
    }

    Ok(player_detail.unwrap())
}


// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
//     use cosmwasm_std::{coins, from_binary};

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies(&[]);

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(1000, "earth"));

//         // we can just call .unwrap() to assert this was a success
//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // it worked, let's query the state
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(17, value.count);
//     }

//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
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

//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let unauth_info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
//         match res {
//             Err(ContractError::Unauthorized {}) => {}
//             _ => panic!("Must return unauthorized error"),
//         }

//         // only the original creator can reset the counter
//         let auth_info = mock_info("creator", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

//         // should now be 5
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(5, value.count);
//     }
// }
