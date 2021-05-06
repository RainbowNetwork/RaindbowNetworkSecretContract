use cosmwasm_std::{to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
                   StdError, StdResult, Storage, HumanAddr, Uint128};
use ethereum_types::Address;
use crate::msg::{HandleMsg, InitMsg, QueryMsg, HandleAnswer, QueryAnswer};
use crate::state::{load, save, State, CoinInfo, TransactionInfo, CONFIG_KEY};
use std::{str::FromStr, collections::HashMap};
use secret_toolkit::snip20::{burn_from_msg, mint_msg};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {

    let admin = env.message.sender;

    let coins: HashMap<String, CoinInfo> = HashMap::new();

    let txs: Vec<TransactionInfo> = Vec::new();

    let config = State {
        admin,
        coins,
        txs,
    };

    save(&mut deps.storage, CONFIG_KEY, &config)?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::ChangeAdmin { address } => try_change_admin(deps, env, address),
        HandleMsg::RemoveCoin { coin } => try_remove_coin(deps, env, coin),
        HandleMsg::AddCoin { coin, secret_addr, secret_hash, matic_addr: ethereum_addr } => try_add_coin(deps, env, coin, secret_addr, secret_hash, ethereum_addr),
        HandleMsg::TransferToMaticAddr { recipient, coin, amount } => try_transfer_to_matic(deps, env, recipient, coin, amount),
        HandleMsg::ReceiveFromMaticAddr { recipient, coin, amount } => try_receive_from_matic(deps, env, recipient, coin, amount),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin { } => query_admin(deps),
        QueryMsg::Coins{ } => query_coins(deps),
        QueryMsg::Coin{ coin } => query_coin(deps, coin),
        QueryMsg::GetTxs{ start } => query_txs(deps, start),
    }
}


// Handle Functions
fn try_change_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_admin: HumanAddr,
) -> StdResult<HandleResponse> {

    // Get the config
    let mut config: State = load(&mut deps.storage, CONFIG_KEY)?;
    let old_admin = config.admin.clone();

    // Check if user is admin
    check_if_admin(&config, &env.message.sender)?;

    // Change Admin
    config.admin = new_admin;
    save(&mut deps.storage, CONFIG_KEY, &config)?;

    // Return a HandleResponse
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ChangeAdmin {
            old_admin,
            new_admin: config.admin,
        })?),
    })
}

fn try_remove_coin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    coin: String,
) -> StdResult<HandleResponse> {

    let status: String;

    // Get the config
    let mut config: State = load(&mut deps.storage, CONFIG_KEY)?;

    // Check if user is admin
    check_if_admin(&config, &env.message.sender)?;

    if config.coins.contains_key(&coin) {
        config.coins.remove(&coin);
        status = String::from("Coin removed");
        save(&mut deps.storage, CONFIG_KEY, &config)?;
    } else {
        status = String::from("Coin does not exist");
    }

    // Return a HandleResponse
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::GenericResponse {
            response: status,
        })?),
    })
}

fn try_add_coin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    coin: String,
    secret_addr: HumanAddr,
    secret_hash: String,
    ethereum_addr: String,
) -> StdResult<HandleResponse> {

    let status: String;

    // Get the config
    let mut config: State = load(&mut deps.storage, CONFIG_KEY)?;

    // Check if user is admin
    check_if_admin(&config, &env.message.sender)?;

    if !config.coins.contains_key(&coin) {

        // Check that ethereum address is valid
        validate_address(&ethereum_addr)?;

        // Check that secret address is valid
        deps.api.canonical_address(&secret_addr)?;

        config.coins.insert(coin, CoinInfo { secret_addr, secret_hash, matic_addr: ethereum_addr });
        save(&mut deps.storage, CONFIG_KEY, &config)?;

        status = String::from("Coin added");
    } else {
        status = String::from("Coin already exists");
    }

    // Return a HandleResponse
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::GenericResponse {
            response: status,
        })?),
    })
}

fn try_transfer_to_matic<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: String,
    coin: String,
    amount: Uint128,
) -> StdResult<HandleResponse> {

    // Get the config
    let mut config: State = load(&deps.storage, CONFIG_KEY)?;

    let mut response_messages = vec![];

    // Check if coin exists
    let coin_info ;
    match config.coins.get(&coin) {
        Some(ret_coin) => {
            coin_info = ret_coin;
        },
        _ => { return Err(StdError::generic_err("Coin does not exist", )); },
    }

    let cosmos_msg = burn_from_msg(
        env.message.sender,
        amount,
        None,
        256,
        coin_info.secret_hash.clone(),
        coin_info.secret_addr.clone())?;

    response_messages.push(cosmos_msg);

    config.txs.push(TransactionInfo{
        recipient: String::from(&recipient),
        coin: String::from(&coin),
        amount,
    });

    save(&mut deps.storage, CONFIG_KEY, &config)?;

    // Return a HandleResponse
    Ok(HandleResponse {
        messages: response_messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::TransferToMaticResponse {
            recipient,
            coin,
            amount,
        })?),
    })
}

fn try_receive_from_matic<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: HumanAddr,
    coin: String,
    amount: Uint128,
) -> StdResult<HandleResponse> {

    let status: String;

    // Get the config
    let config: State = load(&deps.storage, CONFIG_KEY)?;

    // Check if user is admin
    check_if_admin(&config, &env.message.sender)?;

    let mut response_messages = vec![];

    // Check if coin exists
    let coin_info ;
    match config.coins.get(&coin) {
        Some(ret_coin) => {
            coin_info = ret_coin;
        },
        _ => { return Err(StdError::generic_err("Coin does not exist", )); },
    }

    let cosmos_msg = mint_msg(
        recipient,
        amount,
        None,
        256,
        coin_info.secret_hash.clone(),
        coin_info.secret_addr.clone())?;

    response_messages.push(cosmos_msg);

    status = String::from("Transfered");

    // Return a HandleResponse
    Ok(HandleResponse {
        messages: response_messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::GenericResponse {
            response: status,
        })?),
    })
}


// Internal functions
fn is_admin(config: &State, account: &HumanAddr) -> StdResult<bool> {
    if &config.admin != account {
        return Ok(false);
    }

    Ok(true)
}

fn check_if_admin(config: &State, account: &HumanAddr) -> StdResult<()> {
    if !is_admin(config, account)? {
        return Err(StdError::generic_err(
            "This is an admin command. Admin commands can only be run from admin address",
        ));
    }

    Ok(())
}

fn validate_address(address: &str) -> StdResult<Address> {
    return if address.starts_with("0x") {
        Address::from_str(address.trim_start_matches("0x"))
            .map_err(|_| StdError::parse_err(address, "Failed to parse Ethereum address"))
    } else {
        Address::from_str(address)
            .map_err(|_| StdError::parse_err(address, "Failed to parse Ethereum address"))
    };
}

// Queries
fn query_admin<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    // retrieve the config state from storage
    let config: State = load(&deps.storage, CONFIG_KEY)?;
    to_binary(&QueryAnswer::Admin{ admin: config.admin })
}

fn query_coins<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    // retrieve the config state from storage
    let config: State = load(&deps.storage, CONFIG_KEY)?;


    let mut coins_arr: Vec<String> = vec![];

    for (key, _value) in config.coins.iter() {
        coins_arr.push(key.clone());
    }

    to_binary(&QueryAnswer::Coins{ coins: coins_arr })
}

fn query_coin<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, coin: String) -> StdResult<Binary> {
    // retrieve the config state from storage
    let config: State = load(&deps.storage, CONFIG_KEY)?;

    let coin_info;

    match config.coins.get(&coin) {
        Some(ret_coin) => {
            coin_info = ret_coin;
        },
        _ => { return Err(StdError::generic_err("Coin does not exist", )); },
    }

    to_binary(&QueryAnswer::Coin{
        coin,
        secret_addr: coin_info.secret_addr.clone(),
        secret_hash: coin_info.secret_hash.clone(),
        matic_addr: coin_info.matic_addr.clone(),
    })
}

fn query_txs<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, start: u64) -> StdResult<Binary> {
    // retrieve the config state from storage
    let config: State = load(&deps.storage, CONFIG_KEY)?;

    let size = config.txs.len();

    if start > size as u64 {
        return Err(StdError::generic_err("Start is greater than total transactions", ));
    }

    let end = if start+100 > size as u64 {size as u64} else {start+100};

    let mut return_txs: Vec<TransactionInfo> = vec![];

    for i in start..end {
        return_txs.push(config.txs[i as usize].clone());
    }

    to_binary(&QueryAnswer::Txs{
        txs: return_txs,
    })
}