use std::vec;

use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    coin, to_json_binary, Addr, Coin, CosmosMsg, Decimal, DistributionMsg, OwnedDeps, StdResult,
    SubMsg, Uint128, WasmMsg,
};

use eris::hub::{
    CallbackMsg, ConfigResponse, DelegationStrategy, ExecuteMsg, FeeConfig, InstantiateMsg,
    PendingBatch, QueryMsg, SingleSwapConfig, StateResponse,
};
use eris_chain_adapter::types::{test_chain_config, DenomType, StageType, WithdrawType};
use eris_chain_shared::test_trait::TestInterface;
use eris_kujira::adapters::bow_vault::BowExecuteMsg;
use eris_kujira::adapters::bw_vault::BlackwhaleExecuteMsg;
use eris_kujira::adapters::fin::Fin;
use kujira::msg::{DenomMsg, KujiraMsg};

use eris_staking_hub_tokenfactory::contract::{execute, instantiate};
use eris_staking_hub_tokenfactory::error::ContractError;

use eris_staking_hub_tokenfactory::types::Delegation;

use crate::hub_test::helpers::{
    check_received_coin, mock_dependencies, mock_env_at_timestamp, query_helper, MOCK_UTOKEN,
};

use super::custom_querier::CustomQuerier;

//--------------------------------------------------------------------------------------------------
// Test setup
//--------------------------------------------------------------------------------------------------

pub const STAKE_DENOM: &str = "factory/cosmos2contract/stake";
pub const BW_DENOM1: &str = "factory/anycontract/btoken";
pub const BW_DENOM2: &str = "factory/anycontract/btoken2";
pub const BOW_DENOM1: &str = "factory/anycontract/bow1";
pub const BOW_DENOM2: &str = "factory/anycontract/bow2";

fn setup_test() -> OwnedDeps<MockStorage, MockApi, CustomQuerier> {
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
            vote_operator: Some("vote_operator".to_string()),
            delegation_strategy: Some(DelegationStrategy::Uniform),
            chain_config: test_chain_config(),
            utoken: MOCK_UTOKEN.to_string(),
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10000),
        mock_info("owner", &[]),
        ExecuteMsg::UpdateConfig {
            protocol_fee_contract: None,
            protocol_reward_fee: None,
            operator: None,
            stages_preset: Some(vec![vec![(
                StageType::Fin {
                    addr: Addr::unchecked("fin1"),
                },
                "test".into(),
                None,
                None,
            )]]),
            withdrawals_preset: None,
            allow_donations: None,
            delegation_strategy: None,
            vote_operator: None,
            chain_config: None,
            default_max_spread: None,
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Custom(KujiraMsg::Denom(DenomMsg::Create {
            subdenom: "stake".into(),
        })))
    );

    deps
}

#[test]
fn swap_max_amount() -> StdResult<()> {
    let mut deps = setup_test();

    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("operator", &[]),
        ExecuteMsg::Callback(CallbackMsg::SingleStageSwap {
            stage: vec![(StageType::fin("fin1"), "test".into(), None, None)],
        }),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::CallbackOnlyCalledByContract {});

    deps.querier.set_bank_balances(&[
        coin(100, "test"),
        coin(200, "abc"),
        coin(100, "rest"),
        coin(200, "test2"),
        coin(1000, "not_relevant"),
    ]);

    let stages: Vec<Vec<SingleSwapConfig>> = vec![
        vec![(StageType::fin("fin1"), "test".into(), None, Some(Uint128::new(44)))],
        vec![
            (StageType::fin("fin2"), "abc".into(), None, Some(Uint128::new(0))),
            (StageType::fin("fin3"), "test2".into(), None, Some(Uint128::new(123))),
            (StageType::fin("fin5"), "anything".into(), None, None),
        ],
        vec![(StageType::fin("fin4"), "rest".into(), None, None)],
    ];

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        ExecuteMsg::Callback(CallbackMsg::SingleStageSwap {
            stage: stages[0].clone(),
        }),
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0].msg,
        Fin(Addr::unchecked("fin1"))
            .swap_msg(&coin(44, "test"), None, Some(Decimal::percent(10)))
            .unwrap()
    );

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        ExecuteMsg::Callback(CallbackMsg::SingleStageSwap {
            stage: stages[1].clone(),
        }),
    )
    .unwrap();
    assert_eq!(res.messages.len(), 2);
    assert_eq!(
        res.messages[0].msg,
        Fin(Addr::unchecked("fin2"))
            .swap_msg(&coin(200, "abc"), None, Some(Decimal::percent(10)))
            .unwrap()
    );
    assert_eq!(
        res.messages[1].msg,
        Fin(Addr::unchecked("fin3"))
            .swap_msg(&coin(123, "test2"), None, Some(Decimal::percent(10)))
            .unwrap()
    );

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        ExecuteMsg::Callback(CallbackMsg::SingleStageSwap {
            stage: stages[2].clone(),
        }),
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0].msg,
        Fin(Addr::unchecked("fin4"))
            .swap_msg(&coin(100, "rest"), None, Some(Decimal::percent(10)))
            .unwrap()
    );

    Ok(())
}
