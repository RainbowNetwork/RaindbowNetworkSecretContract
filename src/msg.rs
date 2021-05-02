use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // add HandleMsg types here

    // Admin TODO: add transfer admin power and give feature to disable contract
    ChangeAdmin {
        address: HumanAddr,
    },

    // Handle list of supported coins
    RemoveCoin {
        coin: String,
    },
    AddCoin {
        coin: String,
        secret_addr: HumanAddr,
        secret_hash: String,
        ethereum_addr: String,
    },

    // Bridge functions
    // Users use this to transfer to eth
    TransferToEthAddr {
        recipient: String,
        coin: String,
        amount: Uint128,
    },
    // Admin use this to give users their transfered assets
    ReceiveFromEthAddr {
        recipient: HumanAddr,
        coin: String,
        amount: Uint128,
    },

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // add QueryMsg types here

    Admin {},
}

/// Responses from handle function
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    // add HandleMsg response types here

    ChangeAdmin {
        old_admin: HumanAddr,
        new_admin: HumanAddr,
    },

    // Placeholder response
    GenericResponse {
        response: String,
    },

}

/// Responses from query function
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    // add QueryMsg response types here

    Admin {
        admin: HumanAddr,
    },
}