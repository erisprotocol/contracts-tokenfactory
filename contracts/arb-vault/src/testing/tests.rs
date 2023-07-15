use std::str::FromStr;

use crate::{
    contract::execute,
    error::ContractError,
    query::{query_state, query_takeable, query_unbond_requests},
    testing::helpers::{
        _mock_env_at_timestamp, chain_test, create_default_lsd_configs, mock_env, setup_test,
    },
};

use crate::query::{query_config, query_user_info};

use astroport::asset::{native_asset, native_asset_info, token_asset_info};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::testing::{mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, coin, from_binary, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Deps, OwnedDeps,
    Response, Uint128, WasmMsg,
};
use eris::arb_vault::{
    Balances, ClaimBalance, Config, ConfigResponse, ExecuteMsg, ExecuteSubMsg, FeeConfig, LpToken,
    StateDetails, StateResponse, TakeableResponse, UnbondItem, UnbondRequestsResponse,
    UserInfoResponse, UtilizationMethod,
};

use eris_chain_shared::chain_trait::ChainInterface;
use itertools::Itertools;

use super::{custom_querier::CustomQuerier, helpers::TEST_LP_TOKEN};

#[cw_serde]
struct Empty {}

#[test]
fn proper_initialization() {
    let deps = setup_test();

    let config: ConfigResponse = query_config(deps.as_ref()).unwrap();

    assert_eq!(
        config,
        ConfigResponse {
            config: Config {
                utoken: "utoken".into(),
                utilization_method: eris::arb_vault::UtilizationMethod::Steps(vec![
                    (Decimal::from_ratio(10u128, 1000u128), Decimal::from_ratio(50u128, 100u128),),
                    (Decimal::from_ratio(15u128, 1000u128), Decimal::from_ratio(70u128, 100u128),),
                    (Decimal::from_ratio(20u128, 1000u128), Decimal::from_ratio(90u128, 100u128),),
                    (Decimal::from_ratio(25u128, 1000u128), Decimal::from_ratio(100u128, 100u128),),
                ]),
                unbond_time_s: 100,
                lsds: create_default_lsd_configs()
                    .into_iter()
                    .map(|a| a.validate(deps.as_ref().api).unwrap())
                    .collect_vec()
            },
            fee_config: eris::arb_vault::FeeConfig {
                protocol_fee_contract: Addr::unchecked("fee"),
                protocol_performance_fee: Decimal::from_str("0.01").unwrap(),
                protocol_withdraw_fee: Decimal::from_str("0.02").unwrap(),
                immediate_withdraw_fee: Decimal::from_str("0.05").unwrap(),
            },
            lp_token: LpToken {
                denom: TEST_LP_TOKEN.into(),
                total_supply: Uint128::zero()
            },
            whitelist: Some(vec![Addr::unchecked("whitelisted_exec")]),
            owner: Addr::unchecked("owner"),
        }
    );
}

#[test]
fn update_config() {
    let mut deps = setup_test();

    let upd_msg = ExecuteMsg::UpdateConfig {
        utilization_method: None,
        unbond_time_s: Some(10u64),
        disable_lsd: None,
        insert_lsd: None,
        remove_lsd: None,
        force_remove_lsd: None,
        fee_config: None,
        set_whitelist: None,
        remove_whitelist: None,
    };

    let res =
        execute(deps.as_mut(), mock_env(), mock_info("user", &[]), upd_msg.clone()).unwrap_err();
    assert_eq!(res, ContractError::Unauthorized {});

    let _res = execute(deps.as_mut(), mock_env(), mock_info("owner", &[]), upd_msg).unwrap();

    let config = query_config(deps.as_ref()).unwrap();

    assert_eq!(
        config,
        ConfigResponse {
            config: Config {
                utoken: "utoken".into(),
                utilization_method: UtilizationMethod::Steps(vec![
                    (Decimal::from_ratio(10u128, 1000u128), Decimal::from_ratio(50u128, 100u128),),
                    (Decimal::from_ratio(15u128, 1000u128), Decimal::from_ratio(70u128, 100u128),),
                    (Decimal::from_ratio(20u128, 1000u128), Decimal::from_ratio(90u128, 100u128),),
                    (Decimal::from_ratio(25u128, 1000u128), Decimal::from_ratio(100u128, 100u128),),
                ]),
                unbond_time_s: 10,
                lsds: create_default_lsd_configs()
                    .into_iter()
                    .map(|a| a.validate(deps.as_ref().api).unwrap())
                    .collect_vec()
            },
            fee_config: FeeConfig {
                protocol_fee_contract: Addr::unchecked("fee"),
                protocol_performance_fee: Decimal::from_str("0.01").unwrap(),
                protocol_withdraw_fee: Decimal::from_str("0.02").unwrap(),
                immediate_withdraw_fee: Decimal::from_str("0.05").unwrap(),
            },
            lp_token: LpToken {
                denom: TEST_LP_TOKEN.into(),
                total_supply: Uint128::zero()
            },
            whitelist: Some(vec![Addr::unchecked("whitelisted_exec")]),
            owner: Addr::unchecked("owner"),
        }
    );

    let upd_msg = ExecuteMsg::UpdateConfig {
        utilization_method: Some(UtilizationMethod::Steps(vec![])),
        unbond_time_s: None,
        disable_lsd: None,
        insert_lsd: None,
        remove_lsd: None,
        force_remove_lsd: None,
        fee_config: None,
        remove_whitelist: None,
        set_whitelist: None,
    };

    let _res = execute(deps.as_mut(), mock_env(), mock_info("owner", &[]), upd_msg).unwrap();

    let config = query_config(deps.as_ref()).unwrap();

    assert_eq!(
        config,
        ConfigResponse {
            config: Config {
                utoken: "utoken".into(),
                utilization_method: UtilizationMethod::Steps(vec![]),
                unbond_time_s: 10,
                lsds: create_default_lsd_configs()
                    .into_iter()
                    .map(|a| a.validate(deps.as_ref().api).unwrap())
                    .collect_vec()
            },
            fee_config: FeeConfig {
                protocol_fee_contract: Addr::unchecked("fee"),
                protocol_performance_fee: Decimal::from_str("0.01").unwrap(),
                protocol_withdraw_fee: Decimal::from_str("0.02").unwrap(),
                immediate_withdraw_fee: Decimal::from_str("0.05").unwrap(),
            },
            lp_token: LpToken {
                denom: TEST_LP_TOKEN.into(),
                total_supply: Uint128::zero()
            },
            whitelist: Some(vec![Addr::unchecked("whitelisted_exec")]),
            owner: Addr::unchecked("owner"),
        }
    );
}

#[test]
fn provide_liquidity_wrong_token() {
    let mut deps = setup_test();

    let provide_msg = ExecuteMsg::Deposit {
        asset: native_asset("notsupported".into(), Uint128::new(100_000000)),
        receiver: None,
    };

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("user", &[coin(100_000000, "notsupported")]),
        provide_msg,
    );

    assert_eq!(res, Err(ContractError::AssetMismatch {}))
}

#[test]
fn provide_liquidity_no_token() {
    let mut deps = setup_test();

    let provide_msg = ExecuteMsg::Deposit {
        asset: native_asset("notsupported".into(), Uint128::new(100_000000)),
        receiver: None,
    };

    let res = execute(deps.as_mut(), mock_env(), mock_info("user", &[]), provide_msg).unwrap_err();

    assert_eq!(res.to_string(), "Generic error: No funds sent")
}

#[test]
fn provide_liquidity_wrong_amount() {
    let mut deps = setup_test();

    let provide_msg = ExecuteMsg::Deposit {
        asset: native_asset("utoken".into(), Uint128::new(123_000000)),
        receiver: None,
    };

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("user", &[coin(100_000000, "utoken")]),
        provide_msg,
    )
    .unwrap_err();

    assert_eq!(
        res.to_string(),
        "Generic error: Native token balance mismatch between the argument and the transferred"
            .to_string()
    )
}

#[test]
fn provide_liquidity_zero_throws() {
    let mut deps = setup_test();

    let provide_msg = ExecuteMsg::Deposit {
        asset: native_asset("utoken".into(), Uint128::new(0)),
        receiver: None,
    };

    let res =
        execute(deps.as_mut(), mock_env(), mock_info("user", &[coin(0, "utoken")]), provide_msg)
            .unwrap_err();

    assert_eq!(res.to_string(), "Generic error: No funds sent")
}

fn _provide_liquidity() -> (OwnedDeps<MockStorage, MockApi, CustomQuerier>, Response) {
    let mut deps = setup_test();

    // pre apply utoken amount
    deps.querier.set_bank_balance(100_000000);
    // deps.querier.set_cw20_total_supply(TEST_LP_TOKEN, 0);
    // this is used to fake calculating the share.
    deps.querier.set_cw20_balance(TEST_LP_TOKEN, "share_user", 50_000000u128);

    let provide_msg = ExecuteMsg::Deposit {
        asset: native_asset("utoken".to_string(), Uint128::new(100_000000)),
        receiver: None,
    };

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("user", &[coin(100_000000, "utoken")]),
        provide_msg,
    )
    .unwrap();

    // deps.querier.set_cw20_total_supply(TEST_LP_TOKEN, 100_000000);
    deps.querier.set_cw20_balance(TEST_LP_TOKEN, "user", 100_000000);

    (deps, res)
}

#[test]
fn provide_liquidity_success() {
    let (_deps, res) = _provide_liquidity();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "arb/execute_deposit"),
            attr("sender", "user"),
            attr("recipient", "user"),
            attr("deposit_amount", "100000000"),
            attr("share", "100000000"),
            attr("vault_utoken_new", "100000000"),
        ]
    );
}

fn _provide_liquidity_again() -> (OwnedDeps<MockStorage, MockApi, CustomQuerier>, Response) {
    let (mut deps, _res) = _provide_liquidity();

    deps.querier.set_bank_balance(100_000000 + 120_000000);

    let provide_msg = ExecuteMsg::Deposit {
        asset: native_asset("utoken".to_string(), Uint128::new(120_000000)),
        receiver: None,
    };

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("user", &[coin(120_000000, "utoken")]),
        provide_msg,
    )
    .unwrap();

    // deps.querier.set_cw20_total_supply(TEST_LP_TOKEN, 220_000000);
    deps.querier.set_cw20_balance(TEST_LP_TOKEN, "user", 220_000000);

    (deps, res)
}

#[test]
fn provide_liquidity_again_success() {
    let (_deps, res) = _provide_liquidity_again();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "arb/execute_deposit"),
            attr("sender", "user"),
            attr("recipient", "user"),
            attr("deposit_amount", "120000000"),
            attr("share", "120000000"),
            attr("vault_utoken_new", "220000000"),
        ]
    );
}

#[test]
fn query_user_info_check() {
    let (mut deps, _res) = _provide_liquidity_again();

    let response = query_user_info(deps.as_ref(), mock_env(), "user".to_string()).unwrap();
    assert_eq!(
        response,
        UserInfoResponse {
            utoken_amount: Uint128::new(220_000000),
            lp_amount: Uint128::new(220_000000),
        }
    );

    // arbs executed and created 2 luna
    deps.querier.set_bank_balances(&[coin(222_000000, "utoken")]);

    let response = query_user_info(deps.as_ref(), mock_env(), "user".to_string()).unwrap();
    assert_eq!(
        response,
        UserInfoResponse {
            utoken_amount: Uint128::new(222_000000),
            lp_amount: Uint128::new(220_000000),
        }
    );

    /* through arbs, 3 more luna are currently unbonding were generated */
    deps.querier.with_unbonding(Uint128::new(3_000000u128));

    let response = query_user_info(deps.as_ref(), mock_env(), "user".to_string()).unwrap();

    let steak_unbonding = Uint128::new(3_000000u128);
    let eris_unbonding = Decimal::from_str("1.1").unwrap() * Uint128::new(3_000000u128);

    assert_eq!(
        response,
        UserInfoResponse {
            utoken_amount: Uint128::new(222_000000) + steak_unbonding + eris_unbonding,
            lp_amount: Uint128::new(220_000000),
        }
    );

    /* through arbs, 4 more luna can currently be claimed */
    deps.querier.with_withdrawable(Uint128::new(4_000000u128));

    let steak_withdrawing = Uint128::new(4_000000u128);
    let eris_withdrawing = Decimal::from_str("1.1").unwrap() * Uint128::new(4_000000u128);

    let response = query_user_info(deps.as_ref(), mock_env(), "user".to_string()).unwrap();
    assert_eq!(
        response,
        UserInfoResponse {
            utoken_amount: Uint128::new(222_000000)
                + steak_unbonding
                + eris_unbonding
                + steak_withdrawing
                + eris_withdrawing,
            lp_amount: Uint128::new(220_000000),
        }
    );
}

#[test]
fn throws_if_provided_profit_not_found() {
    let mut deps = setup_test();

    let whitelist_info = mock_info("whitelisted_exec", &[]);

    let exec_msg = ExecuteMsg::ExecuteArbitrage {
        msg: ExecuteSubMsg {
            contract_addr: None,
            msg: to_binary(&Empty {}).unwrap(),
            funds_amount: Uint128::new(100_000000u128),
        },
        result_token: native_asset_info("eriscw".into()),
        wanted_profit: Decimal::from_ratio(10u128, 100u128),
    };

    let result = execute(deps.as_mut(), mock_env(), whitelist_info, exec_msg).unwrap_err();

    assert_eq!(result, ContractError::NotSupportedProfitStep(Decimal::from_str("0.1").unwrap()));
}

#[test]
fn throws_if_not_whitelisted_executor() {
    let mut deps = setup_test();

    let user_info = mock_info("user", &[]);
    let whitelist_info = mock_info("whitelisted_exec", &[]);

    let execute_msg = ExecuteMsg::ExecuteArbitrage {
        msg: ExecuteSubMsg {
            contract_addr: None,
            msg: to_binary(&Empty {}).unwrap(),
            funds_amount: Uint128::new(100_000000u128),
        },
        result_token: native_asset_info("eriscw".into()),
        wanted_profit: Decimal::from_ratio(1u128, 100u128),
    };

    let withdraw_msg = ExecuteMsg::WithdrawFromLiquidStaking {
        names: None,
    };

    //
    // NOT WHITELISTED
    //
    let result =
        execute(deps.as_mut(), mock_env(), user_info.clone(), execute_msg.clone()).unwrap_err();
    assert_eq!(result, ContractError::UnauthorizedNotWhitelisted {});

    let result = execute(deps.as_mut(), mock_env(), user_info, withdraw_msg.clone()).unwrap_err();
    assert_eq!(result, ContractError::UnauthorizedNotWhitelisted {});

    //
    // WHITELISTED
    //
    let result =
        execute(deps.as_mut(), mock_env(), whitelist_info.clone(), execute_msg).unwrap_err();

    assert_eq!(result, ContractError::NotEnoughFundsTakeable {});

    let result = execute(deps.as_mut(), mock_env(), whitelist_info, withdraw_msg).unwrap_err();
    assert_eq!(result, ContractError::NothingToWithdraw {});
}

// #[test]
// fn throws_if_has_withdraw() {
//     let mut deps = setup_test();

//     let whitelist_info = mock_info("whitelisted_exec", &[]);

//     let withdraw_msg = ExecuteMsg::WithdrawFromLiquidStaking {};

//     let result =
//         execute(deps.as_mut(), mock_env(), whitelist_info.clone(), withdraw_msg).unwrap_err();
//     assert_eq!(result, ContractError::NothingToWithdraw {});

//     deps.querier.with_withdrawable(Uint128::new(10));
//     deps.querier.set_bank_balances(&[coin(222_000000, "utoken")]);

//     let execute_msg = ExecuteMsg::ExecuteArbitrage {
//         msg: ExecuteSubMsg {
//             contract_addr: None,
//             msg: to_binary(&Empty {}).unwrap(),
//             funds_amount: Uint128::new(100_000000u128),
//         },
//         result_token: native_asset_info("eriscw".into()),
//         wanted_profit: Decimal::from_ratio(1u128, 100u128),
//     };
//     let result = execute(deps.as_mut(), mock_env(), whitelist_info, execute_msg).unwrap_err();

//     assert_eq!(result, ContractError::WithdrawBeforeExecute {});
// }

#[test]
fn check_withdrawing() {
    let mut deps = setup_test();

    let whitelist_info = mock_info("whitelisted_exec", &[]);

    let withdraw_msg = ExecuteMsg::WithdrawFromLiquidStaking {
        names: None,
    };

    deps.querier.with_withdrawable(Uint128::new(10_000000u128));

    let result = execute(deps.as_mut(), mock_env(), whitelist_info, withdraw_msg)
        .expect("expected response");

    assert_eq!(
        result.attributes,
        vec![
            attr("action", "arb/execute_withdraw_liquidity"),
            attr("type", "eris"), // eris has factor 1.1
            attr("withdraw_amount", "11000000"),
            attr("type", "backbone"),
            attr("withdraw_amount", "10000000"),
        ]
    );

    // eris + backbone
    assert_eq!(result.messages.len(), 2);

    // eris
    match result.messages[0].msg.clone() {
        CosmosMsg::Wasm(WasmMsg::Execute {
            funds,
            contract_addr,
            msg,
        }) => {
            assert_eq!(contract_addr, "eris".to_string());
            assert_eq!(funds.len(), 0);
            let sub_msg: eris::hub::ExecuteMsg = from_binary(&msg).unwrap();

            assert_eq!(
                sub_msg,
                eris::hub::ExecuteMsg::WithdrawUnbonded {
                    receiver: None
                }
            );
        },
        _ => panic!("DO NOT ENTER HERE"),
    }

    // backbone
    match result.messages[1].msg.clone() {
        CosmosMsg::Wasm(WasmMsg::Execute {
            funds,
            contract_addr,
            msg,
        }) => {
            assert_eq!(contract_addr, "backbone".to_string());
            assert_eq!(funds.len(), 0);
            let sub_msg: steak::hub::ExecuteMsg = from_binary(&msg).unwrap();

            assert_eq!(
                sub_msg,
                steak::hub::ExecuteMsg::WithdrawUnbonded {
                    receiver: None
                }
            );
        },
        _ => panic!("DO NOT ENTER HERE"),
    }
}

fn _unbonding_slow_120() -> (OwnedDeps<MockStorage, MockApi, CustomQuerier>, Response) {
    // deposit 100
    // deposit 120
    // withdraw 120

    let (mut deps, _res) = _provide_liquidity_again();

    let user001 = mock_info("user001", &[coin(120_000000u128, TEST_LP_TOKEN)]);
    let withdraw = ExecuteMsg::Unbond {
        immediate: Some(false),
    };
    let res = execute(deps.as_mut(), mock_env(), user001, withdraw).unwrap();

    deps.querier.set_bank_balance(220_000000u128);
    // deps.querier.set_cw20_total_supply(TEST_LP_TOKEN, 100_000000);

    // println!("{:?}", query_config(deps.as_ref()).unwrap().lp_token);
    (deps, res)
}

#[test]
fn withdrawing_liquidity_success() {
    let (deps, res) = _unbonding_slow_120();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "arb/execute_unbond"),
            attr("from", "user001"),
            attr("withdraw_amount", "120000000"),
            attr("receive_amount", "117600000"),
            attr("protocol_fee", "2400000"),
            attr("vault_total", "220000000"),
            attr("total_supply", "220000000"),
            attr("unbond_time_s", "100"),
            attr("burnt_amount", "120000000")
        ]
    );

    // withdraw + fee
    assert_eq!(res.messages.len(), 1);

    assert_eq!(
        res.messages[0].msg,
        chain_test().create_burn_msg(TEST_LP_TOKEN.to_string(), Uint128::new(120_000000u128))
    );

    // check unbonding history correct start
    let unbonding = query_unbond_requests(
        deps.as_ref(),
        _mock_env_at_timestamp(1),
        "user001".to_string(),
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        unbonding,
        UnbondRequestsResponse {
            requests: vec![UnbondItem {
                start_time: 1,
                release_time: 1 + 100,
                amount_asset: Uint128::new(120000000),
                id: 0,
                withdraw_protocol_fee: Uint128::new(2400000),
                // 0.05 * 120000000 = 6000000
                withdraw_pool_fee: Uint128::new(6000000),
                released: false,
            }]
        }
    );

    // check unbonding history correct in the middle
    let unbonding = query_unbond_requests(
        deps.as_ref(),
        _mock_env_at_timestamp(10),
        "user001".to_string(),
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        unbonding,
        UnbondRequestsResponse {
            requests: vec![UnbondItem {
                start_time: 1,
                release_time: 1 + 100,
                amount_asset: Uint128::new(120000000),
                id: 0,
                withdraw_protocol_fee: Uint128::new(2400000),
                withdraw_pool_fee: Uint128::new(5460000),
                released: false,
            }]
        }
    );

    // check unbonding history correct after release
    let unbonding = query_unbond_requests(
        deps.as_ref(),
        _mock_env_at_timestamp(101),
        "user001".to_string(),
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        unbonding,
        UnbondRequestsResponse {
            requests: vec![UnbondItem {
                start_time: 1,
                release_time: 1 + 100,
                amount_asset: Uint128::new(120000000),
                id: 0,
                withdraw_protocol_fee: Uint128::new(2400000),
                withdraw_pool_fee: Uint128::new(0),
                released: true,
            }]
        }
    );
}

fn _unbonding_slow_with_pool_unbonding(
) -> (OwnedDeps<MockStorage, MockApi, CustomQuerier>, Response) {
    let (mut deps, _res) = _provide_liquidity_again();

    // arbs executed and created 2 luna
    deps.querier.set_bank_balance(100_000000);
    deps.querier.with_unbonding(Uint128::new(24_000000u128));

    let user001 = mock_info("user001", &[coin(120_000000u128, TEST_LP_TOKEN)]);
    let withdraw = ExecuteMsg::Unbond {
        immediate: Some(false),
    };
    let res = execute(deps.as_mut(), mock_env(), user001, withdraw).unwrap();

    // deps.querier.set_cw20_total_supply(TEST_LP_TOKEN, 100_000000);
    deps.querier.set_cw20_balance(TEST_LP_TOKEN, "user", 120_000000);

    (deps, res)
}

fn get_unbonding_value(set: u128) -> Uint128 {
    let set = Uint128::new(set);
    let eris_unbonding = Decimal::from_str("1.1").unwrap() * set;
    let steak_unbonding = set;

    eris_unbonding + steak_unbonding
}

fn get_withdraw_value(set: u128) -> Uint128 {
    let set = Uint128::new(set);
    let eris = Decimal::from_str("1.1").unwrap() * set;
    let steak = set;

    eris + steak
}

#[test]
fn withdrawing_liquidity_with_unbonding_success() {
    let (_deps, res) = _unbonding_slow_with_pool_unbonding();

    let pool_value = Uint128::new(100_000000u128) + get_unbonding_value(24_000000u128);
    let expected_asset = pool_value.multiply_ratio(120u128, 220u128);
    let fee = Decimal::from_str("0.02").unwrap() * expected_asset;
    let receive = expected_asset - fee;

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "arb/execute_unbond"),
            attr("from", "user001"),
            attr("withdraw_amount", expected_asset),
            attr("receive_amount", receive),
            attr("protocol_fee", fee),
            attr("vault_total", pool_value),
            attr("total_supply", "220000000"),
            attr("unbond_time_s", "100"),
            attr("burnt_amount", "120000000")
        ]
    );

    // withdraw + fee
    assert_eq!(res.messages.len(), 1);

    assert_eq!(
        res.messages[0].msg,
        chain_test().create_burn_msg(TEST_LP_TOKEN.into(), Uint128::new(120_000000u128))
    );
}

#[test]
fn withdraw_liquidity_immediate_user_unbonding_no_liquidity_throws() {
    let (mut deps, _res) = _unbonding_slow_with_pool_unbonding();

    let user001 = mock_info("user001", &[coin(100_000000u128, TEST_LP_TOKEN)]);
    let withdraw = ExecuteMsg::Unbond {
        immediate: Some(true),
    };
    let result = execute(deps.as_mut(), mock_env(), user001, withdraw).unwrap_err();

    // withdraw + fee
    assert_eq!(result, ContractError::NotEnoughAssetsInThePool {});
}

#[test]
fn withdraw_liquidity_immediate_tokens_unbonding_no_liquidity_throws() {
    let (mut deps, _res) = _provide_liquidity_again();

    deps.querier.set_bank_balance(100_000000);

    // is some factor of 120 LUNA unbonding + some rewards = 2*48
    deps.querier.with_unbonding(Uint128::new(48_000000u128));

    let user001 = mock_info("user001", &[coin(120_000000u128, TEST_LP_TOKEN)]);
    let withdraw = ExecuteMsg::Unbond {
        immediate: Some(true),
    };
    let result =
        execute(deps.as_mut(), mock_env(), user001, withdraw).expect_err("expected an error");

    // withdraw + fee
    assert_eq!(result, ContractError::NotEnoughAssetsInThePool {});
}

#[test]
fn withdraw_liquidity_immediate_success() {
    let (mut deps, _res) = _provide_liquidity_again();

    // total_asset: 220
    // pool made 2 through arbs
    let total_pool = Uint128::new(100_000000u128 + 120_000000u128 + 2_000000u128);

    // arbs executed and created 2 luna
    deps.querier.set_bank_balance(222_000000);

    let user001 = mock_info("user001", &[coin(100_000000u128, "anything")]);
    let withdraw = ExecuteMsg::Unbond {
        immediate: Some(false),
    };
    let result = execute(deps.as_mut(), mock_env(), user001, withdraw).unwrap_err();

    assert_eq!(result, ContractError::ExpectingLPToken("100000000anything".to_string()));

    let user001 = mock_info("user001", &[coin(100_000000u128, TEST_LP_TOKEN)]);
    let withdraw = ExecuteMsg::Unbond {
        immediate: Some(true),
    };
    let result = execute(deps.as_mut(), mock_env(), user001, withdraw).expect("expected a result");

    let withdraw_pool_amount = Decimal::from_ratio(100u128, 220u128) * total_pool;
    let pool_fee = Decimal::from_str("0.05").unwrap() * withdraw_pool_amount;
    let protocol_fee = Decimal::from_str("0.02").unwrap() * withdraw_pool_amount;
    assert_eq!(
        result.attributes,
        vec![
            attr("action", "arb/execute_withdraw"),
            attr("from", MOCK_CONTRACT_ADDR),
            attr("receiver", "user001"),
            attr("withdraw_amount", withdraw_pool_amount),
            attr("receive_amount", withdraw_pool_amount - pool_fee - protocol_fee),
            attr("protocol_fee", protocol_fee),
            attr("pool_fee", pool_fee),
            attr("immediate", true.to_string()),
            attr("burnt_amount", "100000000")
        ]
    );

    // withdraw + fee + burn
    assert_eq!(result.messages.len(), 3);

    match result.messages[0].msg.clone() {
        CosmosMsg::Bank(BankMsg::Send {
            to_address,
            amount,
        }) => {
            assert_eq!(to_address, "user001".to_string());
            assert_eq!(amount.len(), 1);
            assert_eq!(
                amount[0],
                Coin {
                    denom: "utoken".to_string(),
                    amount: withdraw_pool_amount - pool_fee - protocol_fee
                }
            );
        },

        _ => panic!("DO NOT ENTER HERE"),
    }

    match result.messages[1].msg.clone() {
        CosmosMsg::Bank(BankMsg::Send {
            to_address,
            amount,
        }) => {
            assert_eq!(to_address, "fee".to_string());
            assert_eq!(amount.len(), 1);
            assert_eq!(
                amount[0],
                Coin {
                    denom: "utoken".to_string(),
                    amount: protocol_fee
                }
            );
        },

        _ => panic!("DO NOT ENTER HERE"),
    }

    assert_eq!(
        result.messages[2].msg,
        chain_test().create_burn_msg(TEST_LP_TOKEN.into(), Uint128::new(100_000000u128))
    );
}

#[test]
fn withdraw_liquidity_unbonding_query_requests_success() {
    let (mut deps, _res) = _unbonding_slow_120();

    //
    // UNBONDING AGAIN WITH OTHER TIME
    //

    let user = mock_info("user001", &[]);
    let mid_time = _mock_env_at_timestamp(51);
    let end_time = _mock_env_at_timestamp(200);

    let user001 = mock_info("user001", &[coin(10_000000u128, TEST_LP_TOKEN)]);
    let withdraw = ExecuteMsg::Unbond {
        immediate: Some(false),
    };
    let res = execute(deps.as_mut(), mid_time.clone(), user001, withdraw).unwrap();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "arb/execute_unbond"),
            attr("from", "user001"),
            attr("withdraw_amount", "10000000"),
            attr("receive_amount", "9800000"),
            attr("protocol_fee", "200000"),
            attr("vault_total", "100000000"),
            attr("total_supply", "100000000"),
            attr("unbond_time_s", "100"),
            attr("burnt_amount", "10000000")
        ]
    );

    let unbonding =
        query_unbond_requests(deps.as_ref(), mid_time.clone(), "user001".to_string(), None, None)
            .unwrap();

    assert_eq!(
        unbonding,
        UnbondRequestsResponse {
            requests: vec![
                UnbondItem {
                    start_time: 1,
                    release_time: 1 + 100,
                    amount_asset: Uint128::new(120_000000u128),
                    id: 0,
                    withdraw_protocol_fee: Uint128::new(2400000),
                    withdraw_pool_fee: Uint128::new(3000000),
                    released: false
                },
                UnbondItem {
                    start_time: 51,
                    release_time: 51 + 100,
                    amount_asset: Uint128::new(10_000000u128),
                    id: 1,
                    withdraw_protocol_fee: Uint128::new(200000),
                    withdraw_pool_fee: Uint128::new(500000),
                    released: false,
                }
            ]
        },
    );

    let share = query_utoken(deps.as_ref());
    //
    // WITHDRAW IMMEDIATE
    //
    let withdraw_immediate = ExecuteMsg::WithdrawImmediate {
        id: 0,
    };

    let res = execute(deps.as_mut(), mid_time.clone(), user.clone(), withdraw_immediate).unwrap();

    let withdraw_pool_amount = Uint128::new(120_000000u128);
    let pool_fee = Decimal::from_str("0.05").unwrap()
        * withdraw_pool_amount
        * Decimal::from_str("0.5").unwrap();
    let protocol_fee = Decimal::from_str("0.02").unwrap() * withdraw_pool_amount;
    let receive_amount = withdraw_pool_amount - pool_fee - protocol_fee;

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "arb/execute_withdraw"),
            attr("from", "cosmos2contract"),
            attr("receiver", "user001"),
            attr("withdraw_amount", withdraw_pool_amount),
            attr("receive_amount", receive_amount),
            attr("protocol_fee", protocol_fee),
            attr("pool_fee", pool_fee),
            attr("immediate", true.to_string()),
        ]
    );

    // println!("{:?}", res.attributes);

    // withdraw + fee (without burn)
    assert_eq!(res.messages.len(), 2);

    match res.messages[0].msg.clone() {
        CosmosMsg::Bank(BankMsg::Send {
            to_address,
            amount,
        }) => {
            assert_eq!(to_address, "user001".to_string());
            assert_eq!(amount.len(), 1);
            assert_eq!(
                amount[0],
                Coin {
                    denom: "utoken".to_string(),
                    amount: receive_amount
                }
            );
        },

        _ => panic!("DO NOT ENTER HERE"),
    }

    match res.messages[1].msg.clone() {
        CosmosMsg::Bank(BankMsg::Send {
            to_address,
            amount,
        }) => {
            assert_eq!(to_address, "fee".to_string());
            assert_eq!(amount.len(), 1);
            assert_eq!(
                amount[0],
                Coin {
                    denom: "utoken".to_string(),
                    amount: protocol_fee
                }
            );
        },

        _ => panic!("DO NOT ENTER HERE"),
    }

    let unbonding =
        query_unbond_requests(deps.as_ref(), mid_time, "user001".to_string(), None, None).unwrap();

    assert_eq!(
        unbonding,
        UnbondRequestsResponse {
            requests: vec![UnbondItem {
                start_time: 51,
                release_time: 51 + 100,
                amount_asset: Uint128::new(10_000000u128),
                id: 1,
                withdraw_protocol_fee: Uint128::new(200000),
                withdraw_pool_fee: Uint128::new(500000),
                released: false
            }]
        }
    );

    deps.querier.set_bank_balance(220_000000 - receive_amount.u128() - protocol_fee.u128());

    // println!("{:?}", query_config(deps.as_ref()).unwrap());

    // share value is increased by the half protocol fee (share is 50 / 100)
    let share2 = query_utoken(deps.as_ref());
    assert_eq!(share + pool_fee * Decimal::from_ratio(50u128, 90u128), share2);

    //
    // WITHDRAW IMMEDIATE AFTER END
    //
    let unbonding =
        query_unbond_requests(deps.as_ref(), end_time.clone(), "user001".to_string(), None, None)
            .unwrap();

    assert_eq!(
        unbonding,
        UnbondRequestsResponse {
            requests: vec![UnbondItem {
                start_time: 51,
                release_time: 51 + 100,
                amount_asset: Uint128::new(10_000000u128),
                id: 1,
                withdraw_protocol_fee: Uint128::new(200000),
                withdraw_pool_fee: Uint128::new(0u128),
                released: true
            }]
        }
    );

    let withdraw_immediate = ExecuteMsg::WithdrawImmediate {
        id: 1,
    };

    let res = execute(deps.as_mut(), end_time.clone(), user, withdraw_immediate).unwrap();

    let withdraw_pool_amount = Uint128::new(10_000000u128);
    let pool_fee2 = Uint128::zero();
    let protocol_fee2 = Decimal::from_str("0.02").unwrap() * withdraw_pool_amount;
    let receive_amount2 = withdraw_pool_amount - pool_fee2 - protocol_fee2;

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "arb/execute_withdraw"),
            attr("from", "cosmos2contract"),
            attr("receiver", "user001"),
            attr("withdraw_amount", withdraw_pool_amount),
            attr("receive_amount", receive_amount2),
            attr("protocol_fee", protocol_fee2),
            attr("pool_fee", pool_fee2),
            attr("immediate", false.to_string()),
        ]
    );

    // withdraw + fee (without burn)
    assert_eq!(res.messages.len(), 2);

    match res.messages[0].msg.clone() {
        CosmosMsg::Bank(BankMsg::Send {
            to_address,
            amount,
        }) => {
            assert_eq!(to_address, "user001".to_string());
            assert_eq!(amount.len(), 1);
            assert_eq!(
                amount[0],
                Coin {
                    denom: "utoken".to_string(),
                    amount: receive_amount2
                }
            );
        },

        _ => panic!("DO NOT ENTER HERE"),
    }

    match res.messages[1].msg.clone() {
        CosmosMsg::Bank(BankMsg::Send {
            to_address,
            amount,
        }) => {
            assert_eq!(to_address, "fee".to_string());
            assert_eq!(amount.len(), 1);
            assert_eq!(
                amount[0],
                Coin {
                    denom: "utoken".to_string(),
                    amount: protocol_fee2
                }
            );
        },

        _ => panic!("DO NOT ENTER HERE"),
    }

    let unbonding =
        query_unbond_requests(deps.as_ref(), end_time, "user001".to_string(), None, None).unwrap();

    assert_eq!(
        unbonding,
        UnbondRequestsResponse {
            requests: vec![],
        }
    );

    deps.querier.set_bank_balance(
        220_000000u128
            - receive_amount.u128()
            - protocol_fee.u128()
            - receive_amount2.u128()
            - protocol_fee2.u128(),
    );

    let share3 = query_utoken(deps.as_ref());
    // share is not allowed to change by withdrawing after the end time
    assert_eq!(share2, share3);
}

#[test]
fn withdraw_liquidity_unbonded_all_success() {
    let (mut deps, _res) = _unbonding_slow_120();

    //
    // UNBONDING AGAIN WITH OTHER TIME
    //

    let user = mock_info("user001", &[]);
    let mid_time = _mock_env_at_timestamp(51);
    let end_time = _mock_env_at_timestamp(200);

    let user001 = mock_info("user001", &[coin(10_000000u128, TEST_LP_TOKEN)]);
    let withdraw = ExecuteMsg::Unbond {
        immediate: Some(false),
    };
    let _res = execute(deps.as_mut(), mid_time.clone(), user001, withdraw).unwrap();

    let unbonding =
        query_unbond_requests(deps.as_ref(), end_time.clone(), "user001".to_string(), None, None)
            .unwrap();

    assert_eq!(
        unbonding,
        UnbondRequestsResponse {
            requests: vec![
                UnbondItem {
                    start_time: 1,
                    release_time: 1 + 100,
                    amount_asset: Uint128::new(120_000000u128),
                    id: 0,
                    withdraw_protocol_fee: Uint128::new(2400000),
                    withdraw_pool_fee: Uint128::new(0_000000u128),
                    released: true
                },
                UnbondItem {
                    start_time: 51,
                    release_time: 51 + 100,
                    amount_asset: Uint128::new(10_000000u128),
                    id: 1,
                    withdraw_protocol_fee: Uint128::new(200000u128),
                    withdraw_pool_fee: Uint128::new(0_000000u128),
                    released: true
                }
            ]
        }
    );

    deps.querier.set_cw20_balance(TEST_LP_TOKEN, "share_user", 50_000000);
    let share = query_utoken(deps.as_ref());

    //
    // WITHDRAW UNBONDED FAILED
    //
    let withdraw_unbonded = ExecuteMsg::WithdrawUnbonded {};

    let res =
        execute(deps.as_mut(), mid_time, user.clone(), withdraw_unbonded.clone()).unwrap_err();

    assert_eq!(res, ContractError::NoWithdrawableAsset {});

    //
    // WITHDRAW UNBONDED
    //
    let res = execute(deps.as_mut(), end_time.clone(), user.clone(), withdraw_unbonded)
        .expect("expect response");

    let withdraw_pool_amount = Uint128::from(130_000000u128);
    let pool_fee = Uint128::zero();
    let protocol_fee = Decimal::from_str("0.02").unwrap() * withdraw_pool_amount;
    let receive_amount = withdraw_pool_amount - pool_fee - protocol_fee;

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "arb/execute_withdraw"),
            attr("from", "cosmos2contract"),
            attr("receiver", "user001"),
            attr("withdraw_amount", withdraw_pool_amount),
            attr("receive_amount", receive_amount),
            attr("protocol_fee", protocol_fee),
            attr("pool_fee", pool_fee),
            attr("immediate", false.to_string()),
            // no burn, as it already happend during normal withdraw
            // attr("burnt_amount", "100000000")
        ]
    );

    // withdraw + fee (without burn)
    assert_eq!(res.messages.len(), 2);

    match res.messages[0].msg.clone() {
        CosmosMsg::Bank(BankMsg::Send {
            to_address,
            amount,
        }) => {
            assert_eq!(to_address, "user001".to_string());
            assert_eq!(amount.len(), 1);
            assert_eq!(
                amount[0],
                Coin {
                    denom: "utoken".to_string(),
                    amount: receive_amount
                }
            );
        },

        _ => panic!("DO NOT ENTER HERE"),
    }

    match res.messages[1].msg.clone() {
        CosmosMsg::Bank(BankMsg::Send {
            to_address,
            amount,
        }) => {
            assert_eq!(to_address, "fee".to_string());
            assert_eq!(amount.len(), 1);
            assert_eq!(
                amount[0],
                Coin {
                    denom: "utoken".to_string(),
                    amount: protocol_fee
                }
            );
        },

        _ => panic!("DO NOT ENTER HERE"),
    }

    deps.querier.set_bank_balance(220_000000u128 - receive_amount.u128() - protocol_fee.u128());

    // share value is not changed, as there is no pool fee
    let share2 = query_utoken(deps.as_ref());
    assert_eq!(share, share2);

    let unbonding =
        query_unbond_requests(deps.as_ref(), end_time.clone(), "user001".to_string(), None, None)
            .unwrap();

    // no items
    assert_eq!(
        unbonding,
        UnbondRequestsResponse {
            requests: vec![]
        }
    );

    //
    // WITHDRAW UNBONDED FAILED
    //
    let withdraw_unbonded = ExecuteMsg::WithdrawUnbonded {};

    let res = execute(deps.as_mut(), end_time, user, withdraw_unbonded).unwrap_err();

    assert_eq!(res, ContractError::NoWithdrawableAsset {});
}

#[test]
fn withdraw_liquidity_unbonded_half_success() {
    let (mut deps, _res) = _unbonding_slow_120();

    // difference is that we only unbond part of the history instead of everything
    //
    // UNBONDING AGAIN WITH OTHER TIME
    //

    let user = mock_info("user001", &[]);
    let mid_time = _mock_env_at_timestamp(51);
    let before_end_time = _mock_env_at_timestamp(130);
    let end_time = _mock_env_at_timestamp(200);

    let user001 = mock_info("user001", &[coin(10_000000u128, TEST_LP_TOKEN)]);
    let withdraw = ExecuteMsg::Unbond {
        immediate: Some(false),
    };
    let _res = execute(deps.as_mut(), mid_time, user001, withdraw).unwrap();

    let unbonding =
        query_unbond_requests(deps.as_ref(), end_time.clone(), "user001".to_string(), None, None)
            .unwrap();

    assert_eq!(
        unbonding,
        UnbondRequestsResponse {
            requests: vec![
                UnbondItem {
                    start_time: 1,
                    release_time: 1 + 100,
                    amount_asset: Uint128::new(120_000000u128),
                    id: 0,
                    withdraw_protocol_fee: Uint128::new(2400000),
                    withdraw_pool_fee: Uint128::new(0u128),
                    released: true,
                },
                UnbondItem {
                    start_time: 51,
                    release_time: 51 + 100,
                    amount_asset: Uint128::new(10_000000u128),
                    id: 1,
                    withdraw_protocol_fee: Uint128::new(200000),
                    withdraw_pool_fee: Uint128::new(0u128),
                    released: true,
                }
            ],
        }
    );

    deps.querier.set_cw20_balance(TEST_LP_TOKEN, "share_user", 50_000000);
    let share = query_utoken(deps.as_ref());

    //
    // WITHDRAW UNBONDED
    //
    let withdraw_unbonded = ExecuteMsg::WithdrawUnbonded {};
    let res = execute(deps.as_mut(), before_end_time.clone(), user.clone(), withdraw_unbonded)
        .expect("expect response");

    let withdraw_pool_amount = Uint128::new(120_000000u128);
    let pool_fee = Uint128::zero();
    let protocol_fee = Decimal::from_str("0.02").unwrap() * withdraw_pool_amount;
    let receive_amount = withdraw_pool_amount - pool_fee - protocol_fee;

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "arb/execute_withdraw"),
            attr("from", "cosmos2contract"),
            attr("receiver", "user001"),
            attr("withdraw_amount", withdraw_pool_amount),
            attr("receive_amount", receive_amount),
            attr("protocol_fee", protocol_fee),
            attr("pool_fee", pool_fee),
            attr("immediate", false.to_string()),
        ]
    );

    // withdraw + fee (without burn)
    assert_eq!(res.messages.len(), 2);

    match res.messages[0].msg.clone() {
        CosmosMsg::Bank(BankMsg::Send {
            to_address,
            amount,
        }) => {
            assert_eq!(to_address, "user001".to_string());
            assert_eq!(amount.len(), 1);
            assert_eq!(
                amount[0],
                Coin {
                    denom: "utoken".to_string(),
                    amount: receive_amount
                }
            );
        },

        _ => panic!("DO NOT ENTER HERE"),
    }

    match res.messages[1].msg.clone() {
        CosmosMsg::Bank(BankMsg::Send {
            to_address,
            amount,
        }) => {
            assert_eq!(to_address, "fee".to_string());
            assert_eq!(amount.len(), 1);
            assert_eq!(
                amount[0],
                Coin {
                    denom: "utoken".to_string(),
                    amount: protocol_fee
                }
            );
        },

        _ => panic!("DO NOT ENTER HERE"),
    }

    deps.querier.set_bank_balance(220_000000 - receive_amount.u128() - protocol_fee.u128());

    // share value is not changed, as there is no pool fee
    let share2 = query_utoken(deps.as_ref());
    assert_eq!(share, share2);

    let unbonding =
        query_unbond_requests(deps.as_ref(), end_time, "user001".to_string(), None, None).unwrap();

    // 1 item
    assert_eq!(
        unbonding,
        UnbondRequestsResponse {
            requests: vec![UnbondItem {
                start_time: 51,
                release_time: 51 + 100,
                amount_asset: Uint128::new(10_000000u128),
                id: 1,
                withdraw_protocol_fee: Uint128::new(200000),
                withdraw_pool_fee: Uint128::new(0u128),
                released: true
            }],
        }
    );

    //
    // WITHDRAW UNBONDED FAILED
    //
    let withdraw_unbonded = ExecuteMsg::WithdrawUnbonded {};

    let res = execute(deps.as_mut(), before_end_time, user, withdraw_unbonded).unwrap_err();

    assert_eq!(res, ContractError::NoWithdrawableAsset {});
}

#[test]
fn query_check_balances() {
    let (mut deps, _res) = _unbonding_slow_120();

    deps.querier.with_unbonding(Uint128::new(24_000000u128));
    deps.querier.with_withdrawable(Uint128::new(10_000000u128));

    let pool_available = Uint128::new(220_000000u128);
    let locked = Uint128::new(120_000000u128);
    let pool_takeable = pool_available - locked;

    let unbonding_per_lsd = Uint128::new(24_000000u128);
    let withdrawable_per_lsd = Uint128::new(10_000000u128);
    let eris_exchange_rate = Decimal::from_str("1.1").unwrap();
    let unbonding = get_unbonding_value(unbonding_per_lsd.u128());
    let withdrawable = get_withdraw_value(withdrawable_per_lsd.u128());

    let total_value = pool_available + unbonding + withdrawable - locked;

    let balance = query_state(deps.as_ref(), mock_env(), None).unwrap();
    assert_eq!(
        balance,
        StateResponse {
            total_lp_supply: Uint128::new(100000000u128),
            balances: Balances {
                tvl_utoken: total_value + locked,
                vault_total: total_value,
                vault_available: pool_available,
                vault_takeable: pool_takeable,
                locked_user_withdrawls: locked,
                lsd_unbonding: unbonding,
                lsd_withdrawable: withdrawable,
                lsd_xvalue: Uint128::zero(),
                details: None,
            },
            exchange_rate: Decimal::from_str("1.714").unwrap(),
            details: None
        }
    );

    let balance_detail = query_state(deps.as_ref(), mock_env(), Some(true)).unwrap();
    assert_eq!(
        balance_detail,
        StateResponse {
            total_lp_supply: Uint128::new(100000000u128),
            balances: Balances {
                tvl_utoken: total_value + locked,
                vault_total: total_value,
                vault_available: pool_available,
                vault_takeable: pool_takeable,
                locked_user_withdrawls: locked,
                lsd_unbonding: unbonding,
                lsd_withdrawable: withdrawable,
                lsd_xvalue: Uint128::zero(),
                details: Some(vec![
                    ClaimBalance {
                        name: "eris".to_string(),
                        withdrawable: eris_exchange_rate * withdrawable_per_lsd,
                        unbonding: eris_exchange_rate * unbonding_per_lsd,
                        xfactor: eris_exchange_rate,
                        xbalance: Uint128::zero(),
                    },
                    ClaimBalance {
                        name: "backbone".to_string(),
                        withdrawable: withdrawable_per_lsd,
                        unbonding: unbonding_per_lsd,
                        xfactor: Decimal::one(),
                        xbalance: Uint128::zero(),
                    }
                ]),
            },
            exchange_rate: Decimal::from_str("1.714").unwrap(),
            details: Some(StateDetails {
                takeable_steps: vec![
                    // 1% = 50% of pool
                    (Decimal::from_ratio(10u128, 1000u128), Uint128::new(14300000),),
                    (Decimal::from_ratio(15u128, 1000u128), Uint128::new(48580000),),
                    (Decimal::from_ratio(20u128, 1000u128), Uint128::new(82860000),),
                    (Decimal::from_ratio(25u128, 1000u128), Uint128::new(100000000),),
                ]
            })
        }
    );
}

#[test]
fn query_check_available() {
    let (mut deps, _res) = _unbonding_slow_120();
    deps.querier.with_unbonding(Uint128::new(24_000000u128));
    deps.querier.with_withdrawable(Uint128::new(10_000000u128));

    let pool_available = Uint128::new(220_000000u128);
    let locked = Uint128::new(120_000000u128);
    let pool_takeable = pool_available - locked;
    let unbonding = get_unbonding_value(24_000000u128);
    let withdrawable = get_withdraw_value(10_000000u128);

    let total_value = pool_available + unbonding + withdrawable - locked;

    let available = query_takeable(deps.as_ref(), mock_env(), None).unwrap();

    assert_eq!(
        available,
        TakeableResponse {
            takeable: None,
            steps: vec![
                // 50%
                (
                    Decimal::from_ratio(10u128, 1000u128),
                    calc_takeable(total_value, pool_takeable, "0.5")
                ),
                // 70%
                (
                    Decimal::from_ratio(15u128, 1000u128),
                    calc_takeable(total_value, pool_takeable, "0.7")
                ),
                // 90%
                (
                    Decimal::from_ratio(20u128, 1000u128),
                    calc_takeable(total_value, pool_takeable, "0.9")
                ),
                (
                    Decimal::from_ratio(25u128, 1000u128),
                    calc_takeable(total_value, pool_takeable, "1.0")
                ),
            ],
        },
    );

    let available =
        query_takeable(deps.as_ref(), mock_env(), Some(Decimal::from_str("0.01").unwrap()))
            .unwrap();

    assert_eq!(
        available,
        TakeableResponse {
            takeable: Some(calc_takeable(total_value, pool_takeable, "0.5")),
            steps: vec![
                // 50%
                (
                    Decimal::from_ratio(10u128, 1000u128),
                    calc_takeable(total_value, pool_takeable, "0.5")
                ),
                // 70%
                (
                    Decimal::from_ratio(15u128, 1000u128),
                    calc_takeable(total_value, pool_takeable, "0.7")
                ),
                // 90%
                (
                    Decimal::from_ratio(20u128, 1000u128),
                    calc_takeable(total_value, pool_takeable, "0.9")
                ),
                (
                    Decimal::from_ratio(25u128, 1000u128),
                    calc_takeable(total_value, pool_takeable, "1.0")
                ),
            ],
        },
    );

    let available =
        query_takeable(deps.as_ref(), mock_env(), Some(Decimal::from_str("0.6").unwrap()))
            .unwrap_err();

    // currently no interpolation possible
    assert_eq!(available, ContractError::NotSupportedProfitStep(Decimal::from_str("0.6").unwrap()));
}

#[test]
fn execute_arb_throws() {
    let (mut deps, _res) = _unbonding_slow_120();

    deps.querier.with_unbonding(Uint128::new(24_000000u128));
    deps.querier.with_withdrawable(Uint128::new(10_000000u128));

    deps.querier.set_cw20_balance(TEST_LP_TOKEN, "share_user", 50_000000);
    let start_share = query_utoken(deps.as_ref());
    assert_eq!(start_share, Uint128::new(85700000));

    let whitelist_info = mock_info("whitelisted_exec", &[]);
    let contract_info = mock_info(MOCK_CONTRACT_ADDR, &[]);

    let exec_msg = ExecuteMsg::ExecuteArbitrage {
        msg: ExecuteSubMsg {
            contract_addr: None,
            funds_amount: Uint128::new(1000_000000u128),
            msg: to_binary("exec_any_swap").unwrap(),
        },
        result_token: native_asset_info("eriscw".into()),
        wanted_profit: Decimal::from_str("0.025").unwrap(),
    };
    let res = execute(deps.as_mut(), mock_env(), whitelist_info.clone(), exec_msg)
        .expect_err("expects error");
    assert_eq!(res, ContractError::NotEnoughFundsTakeable {});

    let exec_msg = ExecuteMsg::ExecuteArbitrage {
        msg: ExecuteSubMsg {
            contract_addr: None,
            funds_amount: Uint128::new(10_000000u128),
            msg: to_binary("exec_any_swap").unwrap(),
        },
        result_token: token_asset_info(Addr::unchecked("xxx")),
        wanted_profit: Decimal::from_str("0.025").unwrap(),
    };
    let res = execute(deps.as_mut(), mock_env(), whitelist_info.clone(), exec_msg)
        .expect_err("expects error");
    assert_eq!(res, ContractError::AdapterNotFound("token - xxx".to_string()));

    let exec_msg = ExecuteMsg::ExecuteArbitrage {
        msg: ExecuteSubMsg {
            contract_addr: None,
            funds_amount: Uint128::zero(),
            msg: to_binary("exec_any_swap").unwrap(),
        },
        result_token: native_asset_info("eriscw".into()),
        wanted_profit: Decimal::from_str("0.025").unwrap(),
    };
    let res = execute(deps.as_mut(), mock_env(), whitelist_info.clone(), exec_msg)
        .expect_err("expects error");
    assert_eq!(res, ContractError::InvalidZeroAmount {});

    let res = execute(
        deps.as_mut(),
        mock_env(),
        contract_info,
        ExecuteMsg::Callback(eris::arb_vault::CallbackMsg::AssertResult {
            result_token: native_asset_info("eriscw".into()),
            wanted_profit: Decimal::from_str("0.01").unwrap(),
        }),
    )
    .unwrap_err();
    assert_eq!(res, ContractError::NotExecuting {});

    let wanted_profit = Decimal::from_str("0.015").unwrap();
    let takeable = query_takeable(deps.as_ref(), mock_env(), Some(wanted_profit))
        .unwrap()
        .takeable
        .expect("expects takeable");

    let exec_msg = ExecuteMsg::ExecuteArbitrage {
        msg: ExecuteSubMsg {
            contract_addr: Some("eris".to_string()),
            funds_amount: takeable,
            msg: to_binary("exec_any_swap").unwrap(),
        },
        result_token: native_asset_info("eriscw".into()),
        wanted_profit,
    };
    let res = execute(deps.as_mut(), mock_env(), whitelist_info, exec_msg).unwrap_err();
    assert_eq!(res, ContractError::CannotCallLsdContract {});
}

#[test]
fn execute_arb() {
    let (mut deps, _res) = _unbonding_slow_120();

    deps.querier.set_bank_balance(100_000000 + 120_000000);
    deps.querier.with_unbonding(Uint128::new(24_000000u128));
    deps.querier.with_withdrawable(Uint128::zero());

    let pool_available = Uint128::new(220_000000u128);
    let locked = Uint128::new(120_000000u128);
    let _pool_takeable = pool_available - locked;
    let unbonding = get_unbonding_value(24_000000u128);

    let old_tvl = pool_available + unbonding;

    let old_state = query_state(deps.as_ref(), mock_env(), None).unwrap();

    deps.querier.set_cw20_balance(TEST_LP_TOKEN, "share_user", 50_000000);
    let start_share = query_utoken(deps.as_ref());
    assert_eq!(start_share, Uint128::new(75200000));

    let whitelist_info = mock_info("whitelisted_exec", &[]);
    let user_info = mock_info("user", &[]);
    let contract_info = mock_info(MOCK_CONTRACT_ADDR, &[]);

    let wanted_profit = Decimal::from_str("0.015").unwrap();
    let takeable = query_takeable(deps.as_ref(), mock_env(), Some(wanted_profit))
        .unwrap()
        .takeable
        .expect("expects takeable");

    let exec_msg = ExecuteMsg::ExecuteArbitrage {
        msg: ExecuteSubMsg {
            contract_addr: None,
            funds_amount: takeable,
            msg: to_binary("exec_any_swap").unwrap(),
        },
        result_token: native_asset_info("eriscw".into()),
        wanted_profit,
    };
    let res = execute(deps.as_mut(), mock_env(), whitelist_info.clone(), exec_msg).unwrap();

    assert_eq!(res.attributes, vec![attr("action", "arb/execute_arbitrage")]);
    assert_eq!(res.messages.len(), 2);
    match res.messages[0].msg.clone() {
        CosmosMsg::Wasm(WasmMsg::Execute {
            funds,
            contract_addr,
            msg,
        }) => {
            assert_eq!(contract_addr, whitelist_info.sender.to_string());
            assert_eq!(
                funds,
                vec![Coin {
                    denom: "utoken".to_string(),
                    amount: takeable
                }]
            );

            let sub_msg: String = from_binary(&msg).unwrap();
            assert_eq!(sub_msg, "exec_any_swap");
        },
        _ => panic!("DO NOT ENTER HERE"),
    }

    let sub_msg: ExecuteMsg;
    match res.messages[1].msg.clone() {
        CosmosMsg::Wasm(WasmMsg::Execute {
            funds,
            contract_addr,
            msg,
        }) => {
            assert_eq!(contract_addr, MOCK_CONTRACT_ADDR.to_string());
            assert_eq!(funds.len(), 0);
            sub_msg = from_binary(&msg).unwrap();

            assert_eq!(
                sub_msg,
                ExecuteMsg::Callback(eris::arb_vault::CallbackMsg::AssertResult {
                    result_token: native_asset_info("eriscw".into()),
                    wanted_profit
                })
            );
        },
        _ => panic!("DO NOT ENTER HERE"),
    }

    //
    // EXPECT PROVIDING LIQUIDITY WHILE EXECUTION TO THROW
    //

    let res = execute(
        deps.as_mut(),
        mock_env(),
        user_info,
        ExecuteMsg::Deposit {
            asset: native_asset("utoken".to_string(), Uint128::new(100)),
            receiver: None,
        },
    )
    .unwrap_err();

    assert_eq!(res, ContractError::AlreadyExecuting {});

    let res = execute(
        deps.as_mut(),
        mock_env(),
        whitelist_info.clone(),
        ExecuteMsg::ExecuteArbitrage {
            msg: ExecuteSubMsg {
                contract_addr: None,
                msg: to_binary(&Empty {}).unwrap(),
                funds_amount: Uint128::new(100u128),
            },
            result_token: native_asset_info("eriscw".into()),
            wanted_profit,
        },
    )
    .unwrap_err();

    assert_eq!(res, ContractError::AlreadyExecuting {});

    //
    // APPLYING SUB MSG TO NEW BALANCE
    //
    let profit_factor = Decimal::one() + wanted_profit;
    let eris_exchange_rate = Decimal::from_str("1.1").unwrap();
    // 100 bluna -> 101 luna
    let eris_amount = takeable * (profit_factor / eris_exchange_rate);

    // we have taken the takeable amount from the balance
    // deps.querier.set_bank_balance(100_000000 + 120_000000 - takeable.u128());

    // and received the result in bluna

    // deps.querier.set_cw20_balance("eriscw", MOCK_CONTRACT_ADDR, eris_amount.u128());
    deps.querier.set_bank_balances(&[
        coin(eris_amount.u128(), "eriscw"),
        coin(100_000000 + 120_000000 - takeable.u128(), "utoken"),
    ]);

    //
    // END APPLYING SUB MSG TO NEW BALANCE
    //

    // println!("{:?}", eris_amount);

    let res = execute(deps.as_mut(), mock_env(), contract_info, sub_msg).unwrap();

    let new_tvl = old_tvl + eris_exchange_rate * eris_amount - takeable;
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "arb/assert_result"),
            attr("type", "eris"),
            attr("result_token", "eriscw"),
            attr("received_xamount", "50639272"),
            attr("old_tvl", old_tvl.to_string()),
            attr("new_tvl", new_tvl.to_string()),
            attr("used_balance", takeable.to_string()),
            attr("profit", "823199"),
            attr("exchange_rate", "1.51214968"),
            attr("fee_amount", "8231"),
        ]
    );

    assert_eq!(
        old_state,
        StateResponse {
            exchange_rate: Decimal::from_str("1.504").unwrap(),
            total_lp_supply: Uint128::new(100000000),
            balances: Balances {
                tvl_utoken: old_tvl,
                vault_total: Uint128::new(150400000),
                vault_available: Uint128::new(220000000),
                vault_takeable: Uint128::new(100000000),
                locked_user_withdrawls: Uint128::new(120000000),
                lsd_unbonding: Uint128::new(50400000),
                lsd_withdrawable: Uint128::new(0),
                lsd_xvalue: Uint128::new(0),
                details: None
            },
            details: None
        }
    );

    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0].msg,
        native_asset("utoken".to_string(), Uint128::new(8231)).into_msg("fee").unwrap()
    );

    let res = execute(
        deps.as_mut(),
        mock_env(),
        whitelist_info,
        ExecuteMsg::UnbondFromLiquidStaking {
            names: None,
        },
    )
    .unwrap();

    //
    // APPLYING SUB MSG TO NEW BALANCE
    //
    // xasset moved to unbonding
    deps.querier.set_bank_balances(&[
        coin(0, "eriscw"),
        coin(100_000000 + 120_000000 - takeable.u128(), "utoken"),
    ]);
    deps.querier.with_unbonding_eris(eris_amount);

    //
    // END APPLYING SUB MSG TO NEW BALANCE
    //

    let new_state = query_state(deps.as_ref(), mock_env(), None).unwrap();
    assert_eq!(
        new_state,
        StateResponse {
            exchange_rate: Decimal::from_str("1.51223199").unwrap(),
            total_lp_supply: Uint128::new(100000000),
            balances: Balances {
                tvl_utoken: new_tvl,
                vault_total: Uint128::new(151223199),
                vault_available: Uint128::new(165120000),
                vault_takeable: Uint128::new(45120000),
                locked_user_withdrawls: Uint128::new(120000000),
                lsd_unbonding: Uint128::new(50400000 + (eris_amount * eris_exchange_rate).u128()),
                lsd_withdrawable: Uint128::new(0),
                lsd_xvalue: Uint128::new(0),
                details: None
            },
            details: None
        }
    );

    assert_eq!(res.messages.len(), 1);
    match res.messages[0].msg.clone() {
        CosmosMsg::Wasm(WasmMsg::Execute {
            funds,
            contract_addr,
            msg,
        }) => {
            assert_eq!(contract_addr, "eris".to_string());
            assert_eq!(funds, vec![coin(eris_amount.u128(), "eriscw")]);

            let sub_msg: eris::hub::ExecuteMsg = from_binary(&msg).unwrap();

            assert_eq!(
                sub_msg,
                eris::hub::ExecuteMsg::QueueUnbond {
                    receiver: None
                }
            );
        },
        _ => panic!("DO NOT ENTER HERE"),
    }

    //
    // EXPECT NEW SHARE TO BE BIGGER
    //
    let new_share = query_utoken(deps.as_ref());

    assert!(new_share.gt(&start_share), "new share must be bigger than start");
    assert_eq!(new_share, Uint128::new(75611599));

    // expect takeable to be 0 afterwards
    let takeable =
        query_takeable(deps.as_ref(), mock_env(), Some(wanted_profit)).unwrap().takeable.unwrap();

    assert_eq!(takeable, Uint128::zero());
}

fn calc_takeable(total_value: Uint128, pool_takeable: Uint128, share: &str) -> Uint128 {
    // total value * share = total pool that can be used for that share
    // + takeable - total value

    // Example:
    // share = 0.7
    // total_value: 1000
    // total_value_for_profit 700
    // pool_takeable: 400
    // pool_takeable_for_profit -> 100 (total_for_profit+pool_takeable-total)
    (total_value * Decimal::from_str(share).expect("expect value"))
        .checked_add(pool_takeable)
        .unwrap_or(Uint128::zero())
        .checked_sub(total_value)
        .unwrap_or(Uint128::zero())
}

fn query_utoken(deps: Deps) -> Uint128 {
    let response = query_user_info(deps, mock_env(), "share_user".to_string()).unwrap();
    // println!("{:?}", response);
    response.utoken_amount
}
