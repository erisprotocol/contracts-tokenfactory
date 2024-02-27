use cosmwasm_std::testing::{mock_env, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    coin, from_binary, to_json_binary, Addr, BlockInfo, ContractInfo, CosmosMsg, Deps, Env,
    OwnedDeps, QuerierResult, SubMsg, SystemError, SystemResult, Timestamp, Uint128, WasmMsg,
};
use eris_chain_adapter::types::{
    chain, CustomMsgType, DenomType, HubChainConfig, StageType, WithdrawType,
};
use serde::de::DeserializeOwned;

use eris::hub::{CallbackMsg, ExecuteMsg, QueryMsg, StakeToken};

use eris_chain_shared::chain_trait::ChainInterface;
use eris_staking_hub_tokenfactory::contract::query;
use eris_staking_hub_tokenfactory::state::State;

use super::custom_querier::CustomQuerier;

pub const MOCK_UTOKEN: &str = "ukuji";

pub(super) fn err_unsupported_query<T: std::fmt::Debug>(request: T) -> QuerierResult {
    SystemResult::Err(SystemError::InvalidRequest {
        error: format!("[mock] unsupported query: {:?}", request),
        request: Default::default(),
    })
}

pub(super) fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, CustomQuerier> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: CustomQuerier::default(),
        custom_query_type: std::marker::PhantomData::default(),
    }
}

pub(super) fn mock_env_at_timestamp(timestamp: u64) -> Env {
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

pub(super) fn query_helper<T: DeserializeOwned>(deps: Deps, msg: QueryMsg) -> T {
    from_binary(&query(deps, mock_env(), msg).unwrap()).unwrap()
}

pub(super) fn query_helper_env<T: DeserializeOwned>(
    deps: Deps,
    msg: QueryMsg,
    timestamp: u64,
) -> T {
    from_binary(&query(deps, mock_env_at_timestamp(timestamp), msg).unwrap()).unwrap()
}

pub(super) fn get_stake_full_denom() -> String {
    // pub const STAKE_DENOM: &str = "factory/cosmos2contract/stake";
    chain_test().get_token_denom(MOCK_CONTRACT_ADDR, "stake".into())
}

pub(super) fn set_total_stake_supply(
    state: &State,
    deps: &mut OwnedDeps<cosmwasm_std::MemoryStorage, MockApi, CustomQuerier>,
    total_supply: u128,
) {
    state
        .stake_token
        .save(
            deps.as_mut().storage,
            &StakeToken {
                utoken: MOCK_UTOKEN.into(),
                denom: "factory/cosmos2contract/stake".into(),
                total_supply: Uint128::new(total_supply),
            },
        )
        .unwrap();
}

pub fn check_received_coin(amount: u128, amount_stake: u128) -> SubMsg<CustomMsgType> {
    SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: MOCK_CONTRACT_ADDR.to_string(),
        msg: to_json_binary(&ExecuteMsg::Callback(CallbackMsg::CheckReceivedCoin {
            snapshot: coin(amount, MOCK_UTOKEN),
            snapshot_stake: coin(amount_stake, "factory/cosmos2contract/stake"),
        }))
        .unwrap(),
        funds: vec![],
    }))
}

pub fn chain_test(
) -> impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig> {
    chain(&mock_env())
}
