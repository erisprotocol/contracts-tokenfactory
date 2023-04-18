use std::vec;

use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    coin, to_binary, Addr, Coin, CosmosMsg, Decimal, DistributionMsg, OwnedDeps, StdResult, SubMsg,
    Uint128, WasmMsg,
};

use eris::hub::{
    CallbackMsg, ConfigResponse, DelegationStrategy, ExecuteMsg, FeeConfig, InstantiateMsg,
    PendingBatch, QueryMsg, StateResponse,
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
    check_received_coin, get_stake_full_denom, mock_dependencies, mock_env_at_timestamp,
    query_helper, MOCK_UTOKEN,
};

use super::custom_querier::CustomQuerier;

//--------------------------------------------------------------------------------------------------
// Test setup
//--------------------------------------------------------------------------------------------------

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
            stages_preset: Some(vec![
                vec![(StageType::fin("fin1"), "f1".into(), None, None)],
                vec![(StageType::fin("fin2"), "f2".into(), None, None)],
            ]),
            withdrawals_preset: Some(vec![
                (WithdrawType::bw("bw1"), "LP1".into()),
                (WithdrawType::bow("bow1"), "LP2".into()),
            ]),
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
fn harvest_anyone() {
    let mut deps = setup_test();

    deps.querier.set_staking_delegations(&[Delegation::new("val1", 10, MOCK_UTOKEN)]);
    deps.querier.set_bank_balances(&[
        coin(50, MOCK_UTOKEN),
        coin(100, get_stake_full_denom()),
        coin(1, "f1"),
        coin(2, "LP1"),
        coin(3, "LP2"),
    ]);

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("anyone", &[]),
        ExecuteMsg::Harvest {
            validators: None,
            withdrawals: None,
            stages: None,
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 6);
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
            validator: "val1".into(),
        })
    );
    assert_eq!(
        res.messages[1].msg,
        CallbackMsg::WithdrawLps {
            withdrawals: vec![
                (WithdrawType::bw("bw1"), "LP1".into()),
                (WithdrawType::bow("bow1"), "LP2".into()),
            ]
        }
        .into_cosmos_msg(&Addr::unchecked(MOCK_CONTRACT_ADDR))
        .unwrap()
    );
    assert_eq!(
        res.messages[2].msg,
        CallbackMsg::SingleStageSwap {
            stage: vec![(StageType::fin("fin1"), "f1".into(), None, None)]
        }
        .into_cosmos_msg(&Addr::unchecked(MOCK_CONTRACT_ADDR))
        .unwrap()
    );
    assert_eq!(
        res.messages[3].msg,
        CallbackMsg::SingleStageSwap {
            stage: vec![(StageType::fin("fin2"), "f2".into(), None, None)],
        }
        .into_cosmos_msg(&Addr::unchecked(MOCK_CONTRACT_ADDR))
        .unwrap()
    );
    assert_eq!(res.messages[4], check_received_coin(50, 100));
    assert_eq!(
        res.messages[5].msg,
        CallbackMsg::Reinvest {}.into_cosmos_msg(&Addr::unchecked(MOCK_CONTRACT_ADDR)).unwrap()
    );
}

#[test]
fn harvest_specific_validator() {
    let mut deps = setup_test();

    deps.querier.set_staking_delegations(&[Delegation::new("val1", 10, MOCK_UTOKEN)]);
    deps.querier.set_staking_delegations(&[Delegation::new("val2", 10, MOCK_UTOKEN)]);
    deps.querier.set_staking_delegations(&[Delegation::new("val3", 10, MOCK_UTOKEN)]);

    deps.querier.set_bank_balances(&[
        coin(50, MOCK_UTOKEN),
        coin(100, get_stake_full_denom()),
        coin(1, "f1"),
        coin(2, "LP1"),
        coin(3, "LP2"),
    ]);

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("anyone", &[]),
        ExecuteMsg::Harvest {
            validators: Some(vec!["val3".to_string(), "val2".to_string()]),
            withdrawals: None,
            stages: None,
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 7);
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
            validator: "val3".into(),
        })
    );
    assert_eq!(
        res.messages[1].msg,
        CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
            validator: "val2".into(),
        })
    );
    assert_eq!(
        res.messages[2].msg,
        CallbackMsg::WithdrawLps {
            withdrawals: vec![
                (WithdrawType::bw("bw1"), "LP1".into()),
                (WithdrawType::bow("bow1"), "LP2".into()),
            ]
        }
        .into_cosmos_msg(&Addr::unchecked(MOCK_CONTRACT_ADDR))
        .unwrap()
    );
    assert_eq!(
        res.messages[3].msg,
        CallbackMsg::SingleStageSwap {
            stage: vec![(StageType::fin("fin1"), "f1".into(), None, None)]
        }
        .into_cosmos_msg(&Addr::unchecked(MOCK_CONTRACT_ADDR))
        .unwrap()
    );
    assert_eq!(
        res.messages[4].msg,
        CallbackMsg::SingleStageSwap {
            stage: vec![(StageType::fin("fin2"), "f2".into(), None, None)],
        }
        .into_cosmos_msg(&Addr::unchecked(MOCK_CONTRACT_ADDR))
        .unwrap()
    );
    assert_eq!(res.messages[5], check_received_coin(50, 100));
    assert_eq!(
        res.messages[6].msg,
        CallbackMsg::Reinvest {}.into_cosmos_msg(&Addr::unchecked(MOCK_CONTRACT_ADDR)).unwrap()
    );
}

#[test]
fn harvest_anyone_override() {
    let mut deps = setup_test();

    deps.querier.set_staking_delegations(&[Delegation::new("val1", 10, MOCK_UTOKEN)]);
    deps.querier.set_bank_balances(&[
        coin(50, MOCK_UTOKEN),
        coin(100, get_stake_full_denom()),
        coin(1, "f1"),
        coin(2, "LP1"),
        coin(3, "LP2"),
    ]);

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("anyone", &[]),
        ExecuteMsg::Harvest {
            validators: None,
            withdrawals: Some(vec![]),
            stages: Some(vec![vec![(StageType::fin("x"), "x".into(), None, None)]]),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::UnauthorizedSenderNotOperator {});

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("anyone", &[]),
        ExecuteMsg::Harvest {
            validators: None,
            withdrawals: Some(vec![(WithdrawType::bow("x"), "x".into())]),
            stages: Some(vec![]),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::UnauthorizedSenderNotOperator {});

    // empty overrides are allowed
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("anyone", &[]),
        ExecuteMsg::Harvest {
            validators: None,
            withdrawals: Some(vec![]),
            stages: Some(vec![]),
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 3);
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
            validator: "val1".into(),
        })
    );
    assert_eq!(res.messages[1], check_received_coin(50, 100));
    assert_eq!(
        res.messages[2].msg,
        CallbackMsg::Reinvest {}.into_cosmos_msg(&Addr::unchecked(MOCK_CONTRACT_ADDR)).unwrap()
    );
}

#[test]
fn harvest_operator_override() {
    let mut deps = setup_test();

    deps.querier.set_staking_delegations(&[Delegation::new("val1", 10, MOCK_UTOKEN)]);
    deps.querier.set_bank_balances(&[
        coin(50, MOCK_UTOKEN),
        coin(100, get_stake_full_denom()),
        coin(1, "f1"),
        coin(2, "LP1"),
        coin(3, "LP2"),
    ]);

    // override stages
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("operator", &[]),
        ExecuteMsg::Harvest {
            validators: None,
            withdrawals: Some(vec![]),
            stages: Some(vec![vec![(StageType::fin("x"), "x".into(), None, None)]]),
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 4);

    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
            validator: "val1".into(),
        })
    );
    assert_eq!(
        res.messages[1].msg,
        CallbackMsg::SingleStageSwap {
            stage: vec![(StageType::fin("x"), "x".into(), None, None)]
        }
        .into_cosmos_msg(&Addr::unchecked(MOCK_CONTRACT_ADDR))
        .unwrap()
    );
    assert_eq!(res.messages[2], check_received_coin(50, 100));
    assert_eq!(
        res.messages[3].msg,
        CallbackMsg::Reinvest {}.into_cosmos_msg(&Addr::unchecked(MOCK_CONTRACT_ADDR)).unwrap()
    );

    // override withdrawals
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("operator", &[]),
        ExecuteMsg::Harvest {
            validators: None,
            withdrawals: Some(vec![(WithdrawType::bow("x"), "x".into())]),
            stages: Some(vec![]),
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 4);
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
            validator: "val1".into(),
        })
    );
    assert_eq!(
        res.messages[1].msg,
        CallbackMsg::WithdrawLps {
            withdrawals: vec![(WithdrawType::bow("x"), "x".into()),]
        }
        .into_cosmos_msg(&Addr::unchecked(MOCK_CONTRACT_ADDR))
        .unwrap()
    );
    assert_eq!(res.messages[2], check_received_coin(50, 100));
    assert_eq!(
        res.messages[3].msg,
        CallbackMsg::Reinvest {}.into_cosmos_msg(&Addr::unchecked(MOCK_CONTRACT_ADDR)).unwrap()
    );

    // empty overrides are allowed
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("anyone", &[]),
        ExecuteMsg::Harvest {
            validators: None,
            withdrawals: Some(vec![]),
            stages: Some(vec![]),
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 3);
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
            validator: "val1".into(),
        })
    );
    assert_eq!(res.messages[1], check_received_coin(50, 100));
    assert_eq!(
        res.messages[2].msg,
        CallbackMsg::Reinvest {}.into_cosmos_msg(&Addr::unchecked(MOCK_CONTRACT_ADDR)).unwrap()
    );
}
