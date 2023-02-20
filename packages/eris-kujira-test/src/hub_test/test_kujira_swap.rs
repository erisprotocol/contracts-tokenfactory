use std::vec;

use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    coin, to_binary, Addr, Coin, CosmosMsg, Decimal, DistributionMsg, OwnedDeps, StdResult, SubMsg,
    Uint128, WasmMsg,
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
            )]]),
            withdrawls_preset: None,
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

//--------------------------------------------------------------------------------------------------
// Execution
//--------------------------------------------------------------------------------------------------

#[test]
fn proper_instantiation() {
    let deps = setup_test();

    let res: ConfigResponse = query_helper(deps.as_ref(), QueryMsg::Config {});
    assert_eq!(
        res,
        ConfigResponse {
            owner: "owner".to_string(),
            new_owner: None,
            stake_token: STAKE_DENOM.to_string(),
            epoch_period: 259200,
            unbond_period: 1814400,
            validators: vec!["alice".to_string(), "bob".to_string(), "charlie".to_string()],
            fee_config: FeeConfig {
                protocol_fee_contract: Addr::unchecked("fee"),
                protocol_reward_fee: Decimal::from_ratio(1u128, 100u128)
            },
            operator: "operator".to_string(),
            stages_preset: vec![vec![(StageType::fin("fin1"), "test".into(), None)]],
            withdrawls_preset: vec![],
            allow_donations: false,
            delegation_strategy: DelegationStrategy::Uniform,
            vote_operator: Some("vote_operator".into()),
            utoken: MOCK_UTOKEN.to_string(),
        }
    );

    let res: StateResponse = query_helper(deps.as_ref(), QueryMsg::State {});
    assert_eq!(
        res,
        StateResponse {
            total_ustake: Uint128::zero(),
            total_utoken: Uint128::zero(),
            exchange_rate: Decimal::one(),
            unlocked_coins: vec![],
            unbonding: Uint128::zero(),
            available: Uint128::zero(),
            tvl_utoken: Uint128::zero(),
        },
    );

    let res: PendingBatch = query_helper(deps.as_ref(), QueryMsg::PendingBatch {});
    assert_eq!(
        res,
        PendingBatch {
            id: 1,
            ustake_to_burn: Uint128::zero(),
            est_unbond_start_time: 269200, // 10,000 + 259,200
        },
    );
}

#[test]
fn harvesting_with_options() {
    let mut deps = setup_test();

    // Assume users have bonded a total of 1,000,000 utoken and minted the same amount of ustake
    deps.querier.set_staking_delegations(&[
        Delegation::new("alice", 341667, MOCK_UTOKEN),
        Delegation::new("bob", 341667, MOCK_UTOKEN),
        Delegation::new("charlie", 341666, MOCK_UTOKEN),
    ]);
    // deps.querier.set_cw20_total_supply("stake_token", 1000000);

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("worker", &[]),
        ExecuteMsg::Harvest {
            stages: Some(vec![vec![(StageType::fin("fin1"), "test".into(), None)]]),
            withdrawals: Some(vec![(WithdrawType::bw("bw1"), BW_DENOM1.into())]),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::UnauthorizedSenderNotOperator {});

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("operator", &[]),
        ExecuteMsg::Harvest {
            stages: Some(vec![vec![(StageType::fin("fin1"), MOCK_UTOKEN.into(), None)]]),
            withdrawals: Some(vec![(WithdrawType::bw("bw1"), BW_DENOM1.into())]),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::SwapFromNotAllowed(MOCK_UTOKEN.into()));

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("operator", &[]),
        ExecuteMsg::Harvest {
            stages: Some(vec![vec![(StageType::fin("fin2"), STAKE_DENOM.into(), None)]]),
            withdrawals: Some(vec![(WithdrawType::bw("bw1"), BW_DENOM1.into())]),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::SwapFromNotAllowed(STAKE_DENOM.into()));

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("operator", &[]),
        ExecuteMsg::Harvest {
            stages: Some(vec![vec![(StageType::fin("fin1"), "test".into(), None)]]),
            withdrawals: Some(vec![(WithdrawType::bw("bw1"), BW_DENOM1.into())]),
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 7);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
            validator: "alice".to_string(),
        }))
    );
    assert_eq!(
        res.messages[1],
        SubMsg::new(CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
            validator: "bob".to_string(),
        }))
    );
    assert_eq!(
        res.messages[2],
        SubMsg::new(CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
            validator: "charlie".to_string(),
        }))
    );

    assert_eq!(
        res.messages[3],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_CONTRACT_ADDR.to_string(),
            msg: to_binary(&ExecuteMsg::Callback(CallbackMsg::WithdrawLps {
                withdrawals: vec![(WithdrawType::bw("bw1"), BW_DENOM1.into())],
            }))
            .unwrap(),
            funds: vec![]
        }))
    );

    assert_eq!(
        res.messages[4],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_CONTRACT_ADDR.to_string(),
            msg: to_binary(&ExecuteMsg::Callback(CallbackMsg::SingleStageSwap {
                stage: vec![(
                    StageType::Fin {
                        addr: Addr::unchecked("fin1")
                    },
                    "test".into(),
                    None
                )],
            }))
            .unwrap(),
            funds: vec![]
        }))
    );

    assert_eq!(res.messages[5], check_received_coin(0, 0));

    assert_eq!(
        res.messages[6],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_CONTRACT_ADDR.to_string(),
            msg: to_binary(&ExecuteMsg::Callback(CallbackMsg::Reinvest {})).unwrap(),
            funds: vec![]
        }))
    );
}

#[test]
fn claim_funds() -> StdResult<()> {
    let mut deps = setup_test();
    deps.querier.set_bank_balances(&[coin(100, BW_DENOM1), coin(101, BOW_DENOM1)]);

    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("worker", &[]),
        ExecuteMsg::Callback(CallbackMsg::WithdrawLps {
            withdrawals: vec![(WithdrawType::bw("bw1"), BW_DENOM1.into())],
        }),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::CallbackOnlyCalledByContract {});

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        ExecuteMsg::Callback(CallbackMsg::WithdrawLps {
            withdrawals: vec![
                (WithdrawType::bw("bw1"), BW_DENOM1.into()),
                (WithdrawType::bw("bw2"), BW_DENOM2.into()),
                (WithdrawType::bow("bow1"), BOW_DENOM1.into()),
                (WithdrawType::bow("bow2"), BOW_DENOM2.into()),
            ],
        }),
    )
    .unwrap();

    assert_eq!(res.messages.len(), 2);

    let contract = "bw1";
    let amount = Uint128::new(100);
    let denom = BW_DENOM1;

    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract.to_string(),
            funds: vec![Coin {
                amount,
                denom: denom.to_string(),
            }],
            msg: to_binary(&BlackwhaleExecuteMsg::WithdrawLiquidity {
                amount,
            })?,
        }))
    );

    let contract = "bow1";
    let amount = Uint128::new(101);
    let denom = BOW_DENOM1;

    assert_eq!(
        res.messages[1],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract.to_string(),
            funds: vec![Coin {
                amount,
                denom: denom.to_string(),
            }],
            msg: to_binary(&BowExecuteMsg::Withdraw {})?,
        }))
    );

    Ok(())
}

#[test]
fn swap() -> StdResult<()> {
    let mut deps = setup_test();

    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("operator", &[]),
        ExecuteMsg::Callback(CallbackMsg::SingleStageSwap {
            stage: vec![(StageType::fin("fin1"), "test".into(), None)],
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
        vec![(StageType::fin("fin1"), "test".into(), None)],
        vec![
            (StageType::fin("fin2"), "abc".into(), None),
            (StageType::fin("fin3"), "test2".into(), None),
            (StageType::fin("fin5"), "anything".into(), None),
        ],
        vec![(StageType::fin("fin4"), "rest".into(), None)],
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
            .swap_msg(&coin(100, "test"), None, Some(Decimal::percent(10)))
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
            .swap_msg(&coin(200, "test2"), None, Some(Decimal::percent(10)))
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
