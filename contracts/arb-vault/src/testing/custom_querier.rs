use super::cw20_querier::Cw20Querier;
use super::helpers::err_unsupported_query;
use cosmwasm_std::testing::{BankQuerier, StakingQuerier, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    coin, from_binary, from_slice, to_json_binary, Coin, Decimal, Empty, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use cw20::Cw20QueryMsg;
use std::str::FromStr;
use std::vec;

#[derive(Default)]
pub(super) struct CustomQuerier {
    pub cw20_querier: Cw20Querier,
    pub bank_querier: BankQuerier,
    pub staking_querier: StakingQuerier,
    pub unbonding_amount: Uint128,
    pub unbonding_amount_eris: Option<Uint128>,
    pub withdrawable_amount: Uint128,
}

impl Querier for CustomQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<_> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
                .into()
            },
        };
        self.handle_query(&request)
    }
}

impl CustomQuerier {
    #[allow(dead_code)]
    pub fn set_bank_balances(&mut self, balances: &[Coin]) {
        self.bank_querier.update_balance(MOCK_CONTRACT_ADDR, balances.to_vec());
    }

    pub fn set_cw20_balance(
        &mut self,
        denom: impl Into<String>,
        user: impl Into<String>,
        amount: u128,
    ) {
        self.bank_querier.update_balance(user, vec![coin(amount, denom)]);
    }

    pub fn with_unbonding(&mut self, amount: Uint128) {
        self.unbonding_amount = amount;
    }

    pub fn with_unbonding_eris(&mut self, amount: Uint128) {
        self.unbonding_amount_eris = Some(amount);
    }

    pub fn with_withdrawable(&mut self, amount: Uint128) {
        self.withdrawable_amount = amount;
    }

    // pub fn set_staking_delegations(&mut self, delegations: &[Delegation]) {
    //     let fds = delegations
    //         .iter()
    //         .map(|d| FullDelegation {
    //             delegator: Addr::unchecked(MOCK_CONTRACT_ADDR),
    //             validator: d.validator.clone(),
    //             amount: Coin::new(d.amount, "uluna"),
    //             can_redelegate: Coin::new(0, "uluna"),
    //             accumulated_rewards: vec![],
    //         })
    //         .collect::<Vec<_>>();

    //     self.staking_querier = StakingQuerier::new("uluna", &[], &fds);
    // }

    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg,
            }) => {
                if let Ok(query) = from_binary::<Cw20QueryMsg>(msg) {
                    return self.cw20_querier.handle_query(contract_addr, query);
                }

                if contract_addr == "eris" {
                    return match from_binary(msg).unwrap() {
                        eris::hub::QueryMsg::PendingBatch {} => SystemResult::Ok(
                            to_json_binary(&eris::hub::PendingBatch {
                                id: 3,
                                ustake_to_burn: Uint128::from(1000u128),
                                est_unbond_start_time: 123,
                            })
                            .into(),
                        ),
                        eris::hub::QueryMsg::PreviousBatch(id) => SystemResult::Ok(
                            to_json_binary(&eris::hub::Batch {
                                id,
                                reconciled: id < 2,
                                total_shares: Uint128::from(1000u128),
                                utoken_unclaimed: Uint128::from(1100u128),
                                est_unbond_end_time: 100,
                            })
                            .into(),
                        ),
                        eris::hub::QueryMsg::UnbondRequestsByUser {
                            ..
                        } => {
                            let mut res = vec![
                                eris::hub::UnbondRequestsByUserResponseItem {
                                    id: 1,
                                    shares: self.withdrawable_amount,
                                },
                                eris::hub::UnbondRequestsByUserResponseItem {
                                    id: 2,
                                    shares: self.unbonding_amount,
                                },
                            ];

                            if let Some(unbonding_amount_eris) = self.unbonding_amount_eris {
                                res.push(eris::hub::UnbondRequestsByUserResponseItem {
                                    id: 3,
                                    shares: unbonding_amount_eris,
                                })
                            }

                            SystemResult::Ok(to_json_binary(&res).into())
                        },
                        eris::hub::QueryMsg::State {} => SystemResult::Ok(
                            to_json_binary(&eris::hub::StateResponse {
                                total_ustake: Uint128::from(1000u128),
                                total_utoken: Uint128::from(1100u128),
                                exchange_rate: Decimal::from_str("1.1").unwrap(),
                                unlocked_coins: vec![],
                                unbonding: Uint128::new(0),
                                available: Uint128::new(0),
                                tvl_utoken: Uint128::new(1100),
                            })
                            .into(),
                        ),
                        _ => err_unsupported_query(msg),
                    };
                } else if contract_addr == "backbone" {
                    return match from_binary(msg).unwrap() {
                        steak::hub::QueryMsg::PendingBatch {} => SystemResult::Ok(
                            to_json_binary(&steak::hub::PendingBatch {
                                id: 3,
                                usteak_to_burn: Uint128::from(1000u128),
                                est_unbond_start_time: 123,
                            })
                            .into(),
                        ),
                        steak::hub::QueryMsg::PreviousBatch(id) => SystemResult::Ok(
                            to_json_binary(&steak::hub::Batch {
                                id,
                                reconciled: id < 2,
                                total_shares: Uint128::from(1000u128),
                                amount_unclaimed: Uint128::from(1000u128),
                                est_unbond_end_time: 100,
                            })
                            .into(),
                        ),
                        steak::hub::QueryMsg::UnbondRequestsByUser {
                            ..
                        } => SystemResult::Ok(
                            to_json_binary(&vec![
                                steak::hub::UnbondRequestsByUserResponseItem {
                                    id: 1,
                                    shares: self.withdrawable_amount,
                                },
                                steak::hub::UnbondRequestsByUserResponseItem {
                                    id: 2,
                                    shares: self.unbonding_amount,
                                },
                            ])
                            .into(),
                        ),
                        steak::hub::QueryMsg::State {} => SystemResult::Ok(
                            to_json_binary(&steak::hub::StateResponse {
                                total_usteak: Uint128::from(1000u128),
                                total_native: Uint128::from(1000u128),
                                exchange_rate: Decimal::one(),
                                unlocked_coins: vec![],
                            })
                            .into(),
                        ),
                        _ => err_unsupported_query(msg),
                    };
                }

                err_unsupported_query(msg)
            },

            QueryRequest::Bank(query) => self.bank_querier.query(query),

            QueryRequest::Staking(query) => self.staking_querier.query(query),

            _ => err_unsupported_query(request),
        }
    }

    pub(crate) fn set_bank_balance(&mut self, amount: u128) {
        self.set_bank_balances(&[coin(amount, "utoken")]);
    }
}
