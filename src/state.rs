use crate::msg::Price;
use cosmwasm_std::{CanonicalAddr, HumanAddr, StdError, StdResult, Storage};
use cosmwasm_storage::{
    singleton, singleton_read, PrefixedStorage, ReadonlyPrefixedStorage, ReadonlySingleton,
    Singleton,
};
use schemars::JsonSchema;
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &[u8] = b"config";
pub static PREDICTIONS_KEY: &[u8] = b"predictions";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValidatorKey {
    pub validator: HumanAddr,
    pub validator_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: CanonicalAddr,
    pub validators: Vec<ValidatorKey>,
    pub symbols: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Prediction {
    pub prediction: Price,
    pub validator: HumanAddr,
    pub timestamp: u64,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}

/// returns if a validator key matches a validator or not
pub fn get_validator_from_key<S: Storage>(
    storage: &S,
    validator_key: String,
) -> StdResult<HumanAddr> {
    let state = config_read(storage).load()?;

    for v in state.validators {
        if v.validator_key == validator_key {
            return Ok(v.validator);
        }
    }
    // "Validator key not found"
    Err(StdError::unauthorized())
}

// todo: clean up symbols at some point

pub fn symbol_exists<S: Storage>(storage: &S, symbol: &String) -> StdResult<bool> {
    let state = config_read(storage).load()?;
    for s in state.symbols {
        if &s == symbol {
            return Ok(true);
        }
    }
    return Ok(false);
}

pub fn store_symbol<S: Storage>(storage: &mut S, symbol: &String) -> StdResult<()> {
    let mut state = config_read(storage).load()?;
    state.symbols.push(symbol.clone());
    Ok(config(storage).save(&state)?)
}

pub fn store_prediction<S: Storage>(
    storage: &mut S,
    symbol: String,
    prediciton: &Prediction,
) -> StdResult<()> {
    // validate that the prediction was made by an authorized party
    let mut store = PrefixedStorage::multilevel(&[PREDICTIONS_KEY, symbol.as_bytes()], storage);

    let mut store: AppendStoreMut<Prediction, PrefixedStorage<S>> =
        AppendStoreMut::attach_or_create(&mut store)?;

    let mut found = false;
    let mut pos = 0;
    for p in store.iter() {
        if p?.validator == prediciton.validator {
            found = true;
            break;
        }
        pos += 1;
    }

    if found {
        store.set_at(pos as u32, prediciton)
    } else {
        store.push(prediciton)
    }
}

pub fn get_all_predictions<S: Storage>(storage: &S, symbol: String) -> StdResult<Vec<Prediction>> {
    // validate that the prediction was made by an authorized party
    let mut store =
        ReadonlyPrefixedStorage::multilevel(&[PREDICTIONS_KEY, symbol.as_bytes()], storage);

    let store = if let Some(result) = AppendStore::<Prediction, _>::attach(&store) {
        result?
    } else {
        return Ok(vec![]);
    };

    // Take `page_size` txs starting from the latest tx, potentially skipping `page * page_size`
    // txs from the start.
    let prediction_iter = store.iter();

    let mut result: Vec<Prediction> = vec![];

    for p in prediction_iter {
        result.push(p.unwrap());
    }

    Ok(result)
    // The `and_then` here flattens the `StdResult<StdResult<Tx>>` to an `StdResult<Tx>`
    // let txs: StdResult<Vec<Prediction>> = tx_iter
    //     .map(|tx| tx.map(|tx| tx.into_humanized(api)).and_then(|x| x))
    //     .collect();
}
