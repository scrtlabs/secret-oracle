use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
    StdError, StdResult, Storage, Uint128,
};
use secret_toolkit::storage::{AppendStore, TypedStore, TypedStoreMut};

use crate::msg::{
    HandleMsg, InitMsg, Price, QueryMsg, QueryPricesResponse, QuerySymbolsResponse,
    QueryValidatorsResponse,
};
use crate::state::{
    config, config_read, get_all_predictions, get_validator_from_key, store_prediction,
    store_symbol, symbol_exists, Prediction, State, ValidatorKey,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        owner: deps.api.canonical_address(&env.message.sender)?,
        validators: vec![],
        symbols: vec![],
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Register { validator_key } => register(deps, env, validator_key),
        HandleMsg::PredictPrices {
            prices,
            validator_key,
        } => predict_prices(deps, env, prices, validator_key),
    }
}

pub fn register<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    validator_key: Option<String>,
) -> StdResult<HandleResponse> {
    let validator = env.message.sender;

    let (_, data) = bech32_no_std::decode(&validator.0)
        .map_err(|_| StdError::generic_err("failed to decode address as bech32"))?;

    let valoperaddr = HumanAddr(
        bech32_no_std::encode("secretvaloper", data)
            .map_err(|_| StdError::generic_err("failed to encode address as bech32"))?,
    );

    let vals = deps.querier.query_validators()?;

    // validates that a validator is part of the active validator set
    if !vals.iter().any(|v| v.address == valoperaddr) {
        return Err(StdError::generic_err(format!(
            "{} is not in the current validator set",
            validator
        )));
    }

    let validator_record = ValidatorKey {
        validator_key: validator_key.unwrap(),
        validator: valoperaddr,
    };

    config(&mut deps.storage).update(|mut state| {
        state.validators.push(validator_record);
        Ok(state)
    })?;

    Ok(HandleResponse::default())
}

pub fn predict_prices<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    prices: Vec<Price>,
    validator_key: String,
) -> StdResult<HandleResponse> {
    // let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;

    // todo: use this later
    let timestamp = env.block.time;

    // todo: get validator from validator key
    let validator = get_validator_from_key(&deps.storage, validator_key)?;

    for price in prices {
        // todo: validate price format

        // store all symbols so we can query them easily
        if !symbol_exists(&deps.storage, &price.symbol)? {
            store_symbol(&mut deps.storage, &price.symbol);
        }

        let prediction = Prediction {
            prediction: price.clone(),
            validator: validator.clone(),
            timestamp,
        };
        store_prediction(&mut deps.storage, price.symbol, &prediction);
    }

    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrice { symbols } => to_binary(&query_price(deps, symbols)?),
        // todo: store the symbols
        QueryMsg::GetSymbols {} => to_binary(&query_symbols(deps)?),
        QueryMsg::GetValidators {} => to_binary(&query_validators(deps)?),
    }
}

fn query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbols: Vec<String>,
) -> StdResult<QueryPricesResponse> {
    // 1. query voting power of all validators <--- impossible right now
    // 2. for each symbol:
    // 2.0 convert string price to something we can work with
    // 2.1 average price based on voting power & prediction
    // 2.2 todo: check the time of each prediction
    // 2.3 todo: give some sort of confidence score
    let mut prices: Vec<Price> = vec![];

    for s in symbols {
        let all_prices = get_all_predictions(&deps.storage, s.clone())?;
        let mut average: Uint128 = Uint128::zero();
        let mut num = 0;
        for p in all_prices.iter() {
            average += p.prediction.price;
            num += 1;
        }

        if num == 0 {
            continue;
        }

        average = Uint128::from(average.u128() / num);
        prices.push(Price {
            symbol: s.clone(),
            price: average,
        })
    }

    Ok(QueryPricesResponse { prices })
}

fn query_symbols<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QuerySymbolsResponse> {
    let state = config_read(&deps.storage).load()?;
    Ok(QuerySymbolsResponse {
        symbols: state.symbols,
    })
}

fn query_validators<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryValidatorsResponse> {
    let state = config_read(&deps.storage).load()?;

    let mut validators: Vec<HumanAddr> = vec![];
    for v in state.validators {
        validators.push(v.validator);
    }

    Ok(QueryValidatorsResponse { validators })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};

    use super::*;
}
