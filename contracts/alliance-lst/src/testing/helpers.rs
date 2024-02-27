use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    coin, from_json, to_json_binary, Addr, BlockInfo, ContractInfo, CosmosMsg, Decimal, Deps, Env,
    OwnedDeps, QuerierResult, SubMsg, SystemError, SystemResult, Timestamp, Uint128, WasmMsg,
};
use eris::alliance_lst::{AllianceStakeToken, ExecuteMsg, InstantiateMsg, QueryMsg};
use eris_chain_adapter::types::{
    chain, CustomMsgType, CustomQueryType, DenomType, HubChainConfig, StageType, WithdrawType,
};
use serde::de::DeserializeOwned;

use eris::hub::CallbackMsg;

use crate::contract::{instantiate, query};
use crate::state::State;
use eris_chain_shared::chain_trait::ChainInterface;

use super::custom_querier::CustomQuerier;

pub const MOCK_UTOKEN: &str = "utoken";

pub(super) fn err_unsupported_query<T: std::fmt::Debug>(request: T) -> QuerierResult {
    SystemResult::Err(SystemError::InvalidRequest {
        error: format!("[mock] unsupported query: {:?}", request),
        request: Default::default(),
    })
}

pub(super) fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, CustomQuerier, CustomQueryType>
{
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: CustomQuerier::default(),
        custom_query_type: std::marker::PhantomData,
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

pub(super) fn query_helper<T: DeserializeOwned>(deps: Deps<CustomQueryType>, msg: QueryMsg) -> T {
    from_json(query(deps, mock_env(), msg).unwrap()).unwrap()
}

pub(super) fn query_helper_env<T: DeserializeOwned>(
    deps: Deps<CustomQueryType>,
    msg: QueryMsg,
    timestamp: u64,
) -> T {
    from_json(query(deps, mock_env_at_timestamp(timestamp), msg).unwrap()).unwrap()
}

pub(super) fn get_stake_full_denom() -> String {
    // pub const STAKE_DENOM: &str = "factory/cosmos2contract/stake";
    chain(&mock_env()).get_token_denom(MOCK_CONTRACT_ADDR, "stake".into())
}

pub(super) fn set_total_stake_supply(
    state: &State,
    deps: &mut OwnedDeps<cosmwasm_std::MemoryStorage, MockApi, CustomQuerier, CustomQueryType>,
    total_supply: u128,
    total_utoken_bonded: u128,
) {
    state
        .stake_token
        .save(
            deps.as_mut().storage,
            &AllianceStakeToken {
                utoken: MOCK_UTOKEN.to_string(),
                denom: get_stake_full_denom(),
                total_supply: Uint128::new(total_supply),
                total_utoken_bonded: Uint128::new(total_utoken_bonded),
            },
        )
        .unwrap();
}

pub fn check_received_coin(amount: u128, amount_stake: u128) -> SubMsg<CustomMsgType> {
    SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: MOCK_CONTRACT_ADDR.to_string(),
        msg: to_json_binary(&ExecuteMsg::Callback(CallbackMsg::CheckReceivedCoin {
            snapshot: coin(amount, MOCK_UTOKEN),
            snapshot_stake: coin(amount_stake, get_stake_full_denom()),
        }))
        .unwrap(),
        funds: vec![],
    }))
}

//--------------------------------------------------------------------------------------------------
// Test setup
//--------------------------------------------------------------------------------------------------

pub(super) fn setup_test() -> OwnedDeps<MockStorage, MockApi, CustomQuerier, CustomQueryType> {
    let mut deps = mock_dependencies();

    let res = instantiate(
        deps.as_mut(),
        mock_env_at_timestamp(10000),
        mock_info("deployer", &[]),
        InstantiateMsg {
            owner: "owner".to_string(),
            denom: "stake".to_string(),
            utoken: MOCK_UTOKEN.to_string(),
            epoch_period: 259200,   // 3 * 24 * 60 * 60 = 3 days
            unbond_period: 1814400, // 21 * 24 * 60 * 60 = 21 days
            protocol_fee_contract: "fee".to_string(),
            protocol_reward_fee: Decimal::from_ratio(1u128, 100u128),
            operator: "operator".to_string(),
            delegation_strategy: None,
            validator_proxy: "proxy".to_string(),
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 1);

    assert_eq!(
        res.messages[0].msg,
        chain_test().create_denom_msg(get_stake_full_denom(), "stake".to_string())
    );

    deps
}

pub fn chain_test(
) -> impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig> {
    chain(&mock_env())
}
