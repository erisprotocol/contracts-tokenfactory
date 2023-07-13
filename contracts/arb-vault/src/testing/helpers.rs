use std::str::FromStr;

use cosmwasm_std::testing::{mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, Addr, BlockInfo, ContractInfo, Decimal, Deps, Env, OwnedDeps, QuerierResult,
    ReplyOn, SubMsg, SystemError, SystemResult, Timestamp,
};
use eris_chain_adapter::types::{
    chain, CustomMsgType, DenomType, HubChainConfig, StageType, WithdrawType,
};
use eris_chain_shared::chain_trait::ChainInterface;
use serde::de::DeserializeOwned;

use crate::contract::{instantiate, query};
use eris::arb_vault::{InstantiateMsg, LsdConfig, QueryMsg};

pub const TEST_LP_TOKEN: &str = "factory/cosmos2contract/arbtoken";

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
    from_binary(&query(deps, mock_env(), msg).unwrap()).unwrap()
}

pub(super) fn _query_helper_env<T: DeserializeOwned>(
    deps: Deps,
    msg: QueryMsg,
    timestamp: u64,
) -> T {
    from_binary(&query(deps, _mock_env_at_timestamp(timestamp), msg).unwrap()).unwrap()
}

pub fn create_default_lsd_configs() -> Vec<LsdConfig<String>> {
    vec![
        LsdConfig {
            disabled: false,
            name: "eris".into(),
            lsd_type: eris::arb_vault::LsdType::Eris {
                addr: "eris".into(),
                denom: "eriscw".into(),
            },
        },
        LsdConfig {
            disabled: false,
            name: "backbone".into(),
            lsd_type: eris::arb_vault::LsdType::Backbone {
                addr: "backbone".into(),
                denom: "backbonecw".into(),
            },
        },
    ]
}

pub fn mock_env() -> Env {
    Env {
        block: BlockInfo {
            height: 12_345,
            time: Timestamp::from_seconds(1),
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        contract: ContractInfo {
            address: Addr::unchecked(MOCK_CONTRACT_ADDR),
        },
        transaction: None,
    }
}

// fn mock_env_51() -> Env {
//     Env {
//         block: BlockInfo {
//             height: 12_345,
//             time: Timestamp::from_seconds(51),
//             chain_id: "cosmos-testnet-14002".to_string(),
//         },
//         contract: ContractInfo {
//             address: Addr::unchecked(MOCK_CONTRACT_ADDR),
//         },
//         transaction: None,
//     }
// }
// fn mock_env_200() -> Env {
//     Env {
//         block: BlockInfo {
//             height: 12_345,
//             time: Timestamp::from_seconds(200),
//             chain_id: "cosmos-testnet-14002".to_string(),
//         },
//         contract: ContractInfo {
//             address: Addr::unchecked(MOCK_CONTRACT_ADDR),
//         },
//         transaction: None,
//     }
// }
// fn mock_env_130() -> Env {
//     Env {
//         block: BlockInfo {
//             height: 12_345,
//             time: Timestamp::from_seconds(130),
//             chain_id: "cosmos-testnet-14002".to_string(),
//         },
//         contract: ContractInfo {
//             address: Addr::unchecked(MOCK_CONTRACT_ADDR),
//         },
//         transaction: None,
//     }
// }

// fn create_init_params() -> Option<Binary> {
//     Some(to_binary(&create_default_lsd_configs()).unwrap())
// }

pub fn create_default_init() -> InstantiateMsg {
    InstantiateMsg {
        denom: "arbtoken".into(),
        owner: "owner".into(),
        utoken: "utoken".into(),
        utilization_method: eris::arb_vault::UtilizationMethod::Steps(vec![
            (
                // 1% = 50% of pool
                Decimal::from_ratio(10u128, 1000u128),
                Decimal::from_ratio(50u128, 100u128),
            ),
            (
                // 1% = 50% of pool
                Decimal::from_ratio(15u128, 1000u128),
                Decimal::from_ratio(70u128, 100u128),
            ),
            (
                // 1% = 50% of pool
                Decimal::from_ratio(20u128, 1000u128),
                Decimal::from_ratio(90u128, 100u128),
            ),
            (
                // 1% = 50% of pool
                Decimal::from_ratio(25u128, 1000u128),
                Decimal::from_ratio(100u128, 100u128),
            ),
        ]),
        unbond_time_s: 100,
        lsds: create_default_lsd_configs(),
        fee_config: eris::arb_vault::FeeConfig {
            protocol_fee_contract: "fee".into(),
            protocol_performance_fee: Decimal::from_str("0.01").unwrap(),
            protocol_withdraw_fee: Decimal::from_str("0.02").unwrap(),
            immediate_withdraw_fee: Decimal::from_str("0.05").unwrap(),
        },
        whitelist: vec!["whitelisted_exec".to_string()],
    }
}

pub(super) fn setup_test() -> OwnedDeps<MockStorage, MockApi, CustomQuerier> {
    let mut deps = mock_dependencies();
    let msg = create_default_init();
    let owner = "owner";
    let owner_info = mock_info(owner, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), owner_info, msg).unwrap();

    assert_eq!(
        res.messages,
        vec![SubMsg {
            msg: chain_test().create_denom_msg(TEST_LP_TOKEN.to_string(), "arbtoken".to_string()),
            id: 0,
            gas_limit: None,
            reply_on: ReplyOn::Never
        },]
    );

    // deps.querier.set_cw20_balance("eriscw", MOCK_CONTRACT_ADDR, 0u128);
    // deps.querier.set_cw20_balance("backbonecw", MOCK_CONTRACT_ADDR, 0u128);
    // deps.querier.set_cw20_balance("stadercw", MOCK_CONTRACT_ADDR, 0u128);
    // deps.querier.set_cw20_balance("prismcw", MOCK_CONTRACT_ADDR, 0u128);

    deps
}

pub fn chain_test(
) -> impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig> {
    chain(&mock_env())
}
