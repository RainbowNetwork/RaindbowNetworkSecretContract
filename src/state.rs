use std::{any::type_name, collections::HashMap};
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Storage, StdResult, ReadonlyStorage, StdError};
use serde::de::DeserializeOwned;
use secret_toolkit::serialization::{Bincode2, Serde};
use schemars::JsonSchema;

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub admin: HumanAddr,
    pub coins: HashMap<String, CoinInfo>,
}

// Store each currency with an address for ethereum and secret
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CoinInfo {
    pub secret_addr: HumanAddr,
    pub secret_hash: String,
    pub ethereum_addr: String,
}

pub fn save<T: Serialize, S: Storage>(storage: &mut S, key: &[u8], value: &T) -> StdResult<()> {
    storage.set(key, &Bincode2::serialize(&value).map_err(|e| StdError::serialize_err(type_name::<T>(), e))?);
    Ok(())
}

pub fn load<T: DeserializeOwned, S: ReadonlyStorage>(storage: &S, key: &[u8]) -> StdResult<T> {
    let bin_data = storage.get(key);

    match bin_data {
        None => Err(StdError::not_found("Key not found in storage")),
        Some(bin_data) => Ok(Bincode2::deserialize(&bin_data)
            .map_err(|e| StdError::serialize_err(type_name::<T>(), e))?),
    }
}

// Same as load but wont break if no data is found
pub fn may_load<T: DeserializeOwned, S: ReadonlyStorage>(storage: &S, key: &[u8]) -> StdResult<Option<T>> {
    let bin_data = storage.get(key);

    match bin_data {
        None => Ok(None),
        Some(bin_data) => Ok(Bincode2::deserialize(&bin_data)
            .map_err(|e| StdError::serialize_err(type_name::<T>(), e))?),
    }
}