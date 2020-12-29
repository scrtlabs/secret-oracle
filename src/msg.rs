use cosmwasm_std::{CanonicalAddr, HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Price {
    pub symbol: String,
    pub price: Uint128,
    // todo: add confidence for responses etc.
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    PredictPrices {
        /// array of prices to add
        prices: Vec<Price>,
        /// authenticates the prediction
        validator_key: String,
    },
    Register {
        //validator: HumanAddr,
        /// optional key to set - will generate a key for you if you don't supply one?
        validator_key: Option<String>,
    },
    // todo: maybe?
    // Unregister {
    //
    // },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetPrice { symbols: Vec<String> },
    GetSymbols {},
    GetValidators {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryValidatorsResponse {
    pub validators: Vec<HumanAddr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QuerySymbolsResponse {
    pub symbols: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryPricesResponse {
    pub prices: Vec<Price>,
}
