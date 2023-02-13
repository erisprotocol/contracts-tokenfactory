use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    coin, from_binary, to_binary, Addr, BlockInfo, ContractInfo, CosmosMsg, Decimal, Deps, Env,
    OwnedDeps, QuerierResult, SubMsg, SystemError, SystemResult, Timestamp, Uint128, WasmMsg,
};
use eris_chain_adapter::types::{chain, main_denom, test_chain_config, CustomMsgType};
use serde::de::DeserializeOwned;

use eris::hub::{CallbackMsg, ExecuteMsg, InstantiateMsg, QueryMsg, StakeToken};

use crate::contract::{instantiate, query};
use crate::state::State;
use eris_chain_shared::chain_trait::ChainInterface;

use super::custom_querier::CustomQuerier;

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
    chain().get_token_denom(MOCK_CONTRACT_ADDR, "stake".into())
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
                denom: get_stake_full_denom(),
                total_supply: Uint128::new(total_supply),
            },
        )
        .unwrap();
}

pub fn check_received_coin(amount: u128, amount_stake: u128) -> SubMsg<CustomMsgType> {
    SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: MOCK_CONTRACT_ADDR.to_string(),
        msg: to_binary(&ExecuteMsg::Callback(CallbackMsg::CheckReceivedCoin {
            snapshot: coin(amount, main_denom()),
            snapshot_stake: coin(amount_stake, get_stake_full_denom()),
        }))
        .unwrap(),
        funds: vec![],
    }))
}

//--------------------------------------------------------------------------------------------------
// Test setup
//--------------------------------------------------------------------------------------------------

pub(super) fn setup_test() -> OwnedDeps<MockStorage, MockApi, CustomQuerier> {
    let mut deps = mock_dependencies();

    let res = instantiate(
        deps.as_mut(),
        mock_env_at_timestamp(10000),
        mock_info("deployer", &[]),
        InstantiateMsg {
            owner: "owner".to_string(),
            denom: "stake".to_string(),
            epoch_period: 259200,   // 3 * 24 * 60 * 60 = 3 days
            unbond_period: 1814400, // 21 * 24 * 60 * 60 = 21 days
            validators: vec!["alice".to_string(), "bob".to_string(), "charlie".to_string()],
            protocol_fee_contract: "fee".to_string(),
            protocol_reward_fee: Decimal::from_ratio(1u128, 100u128),
            operator: "operator".to_string(),
            delegation_strategy: None,
            vote_operator: None,
            chain_config: test_chain_config(),
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 1);

    assert_eq!(
        res.messages[0].msg,
        chain().create_denom_msg(get_stake_full_denom(), "stake".to_string())
    );

    deps
}
