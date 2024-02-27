use cosmwasm_std::testing::{mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, Coin, Decimal, Uint128};

use eris::alliance_lst::AllianceStakeToken;
use eris::alliance_lst::{ExecuteMsg, QueryMsg};
use eris::hub::{CallbackMsg, ExchangeRatesResponse};
use eris::DecimalCheckedOps;

use crate::constants::DAY;
use crate::contract::execute;
use crate::state::State;
use crate::testing::helpers::{
    get_stake_full_denom, query_helper_env, set_total_stake_supply, setup_test, MOCK_UTOKEN,
};
use crate::testing::test_defined_delegations::STAKE_DENOM;
use crate::types::Delegation;

use super::helpers::mock_env_at_timestamp;

//--------------------------------------------------------------------------------------------------
// Execution
//--------------------------------------------------------------------------------------------------

#[test]
fn reinvesting_check_exchange_rates() {
    let mut deps = setup_test();
    let state = State::default();

    deps.querier.set_staking_delegations(&[
        Delegation::new("alice", 333334, MOCK_UTOKEN),
        Delegation::new("bob", 333333, MOCK_UTOKEN),
        Delegation::new("charlie", 333333, MOCK_UTOKEN),
    ]);

    // After the swaps, `unlocked_coins` should contain only utoken and unknown denoms
    state
        .unlocked_coins
        .save(
            deps.as_mut().storage,
            &vec![Coin::new(234, MOCK_UTOKEN), Coin::new(111, get_stake_full_denom())],
        )
        .unwrap();

    set_total_stake_supply(&state, &mut deps, 100000, 100000);

    let res = execute(
        deps.as_mut(),
        mock_env_at_timestamp(0),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        ExecuteMsg::Callback(CallbackMsg::Reinvest {}),
    )
    .unwrap();
    assert_eq!(res.messages.len(), 4);

    // ustake: (0_100000 - (111 - fees)), utoken: 1_000000 + (234 - fees)
    assert_eq!(
        res.attributes,
        vec![attr("action", "erishub/reinvest"), attr("exchange_rate", "10.013334668134948443")]
    );
    assert_eq!(
        state.stake_token.load(deps.as_mut().storage).unwrap(),
        AllianceStakeToken {
            utoken: MOCK_UTOKEN.to_string(),
            denom: STAKE_DENOM.to_string(),
            total_supply: Uint128::new(100000 - 110), // only 110 because of fee
            total_utoken_bonded: Uint128::new(100000 + 234 - 2)
        }
    );

    // added delegation of 234 - fees
    let total = Uint128::from(234u128);
    let fee = Decimal::from_ratio(1u128, 100u128).checked_mul_uint(total).unwrap();
    let delegated = total.saturating_sub(fee);
    deps.querier.set_staking_delegations(&[
        Delegation::new("alice", 333334, MOCK_UTOKEN),
        Delegation::new("bob", 333333 + delegated.u128(), MOCK_UTOKEN),
        Delegation::new("charlie", 333333, MOCK_UTOKEN),
    ]);

    state
        .unlocked_coins
        .save(
            deps.as_mut().storage,
            &vec![Coin::new(200, MOCK_UTOKEN), Coin::new(300, get_stake_full_denom())],
        )
        .unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env_at_timestamp(DAY),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        ExecuteMsg::Callback(CallbackMsg::Reinvest {}),
    )
    .unwrap();
    assert_eq!(res.messages.len(), 4);

    let res: ExchangeRatesResponse = query_helper_env(
        deps.as_ref(),
        QueryMsg::ExchangeRates {
            start_after: None,
            limit: None,
        },
        2083600,
    );
    assert_eq!(
        res.exchange_rates
            .into_iter()
            .map(|a| format!("{0};{1}", a.0, a.1))
            .collect::<Vec<String>>(),
        vec!["86400;10.045183898466759712".to_string(), "0;10.013334668134948443".to_string()]
    );

    // 10.013334668134948443 -> 10.045183898466759712 within 1 day
    assert_eq!(res.apr.map(|a| a.to_string()), Some("0.003180681699690299".to_string()));
}
