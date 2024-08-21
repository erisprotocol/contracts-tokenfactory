use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_json, Addr, BlockInfo, ContractInfo, Deps, Env, QuerierResult, SystemError, SystemResult,
    Timestamp,
};
use eris::astroport_farm::QueryMsg;
use eris_chain_adapter::types::{
    chain, CustomMsgType, DenomType, HubChainConfig, StageType, WithdrawType,
};
use eris_chain_shared::chain_trait::ChainInterface;
use serde::de::DeserializeOwned;

use crate::contract::query;

pub(super) fn err_unsupported_query<T: std::fmt::Debug>(request: T) -> QuerierResult {
    SystemResult::Err(SystemError::InvalidRequest {
        error: format!("[mock] unsupported query: {:?}", request),
        request: Default::default(),
    })
}

pub(super) fn _mock_env_at_timestamp(timestamp: u64) -> Env {
    Env {
        block: BlockInfo {
            height: 12_345,
            time: Timestamp::from_seconds(timestamp),
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        contract: ContractInfo {
            address: Addr::unchecked(MOCK_CONTRACT_ADDR),
        },
        transaction: None,
    }
}

pub(super) fn _query_helper<T: DeserializeOwned>(deps: Deps, msg: QueryMsg) -> T {
    from_json(query(deps, mock_env(), msg).unwrap()).unwrap()
}

pub(super) fn _query_helper_env<T: DeserializeOwned>(
    deps: Deps,
    msg: QueryMsg,
    timestamp: u64,
) -> T {
    from_json(query(deps, _mock_env_at_timestamp(timestamp), msg).unwrap()).unwrap()
}

pub fn chain_test(
) -> impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig> {
    chain(&mock_env())
}
