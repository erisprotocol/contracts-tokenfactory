use astroport::asset::{native_asset_info, AssetInfo};
use cosmwasm_std::{
    coin, to_binary, Addr, CosmosMsg, Decimal, Deps, QueryRequest, Uint128, WasmMsg, WasmQuery,
};

use eris::hub::{
    Batch, ExecuteMsg, PendingBatch, QueryMsg, StateResponse, UnbondRequestsByUserResponseItem,
};
use eris_chain_adapter::types::CustomMsgType;

use crate::error::{adapter_error, CustomResult};

use super::lsdadapter::LsdAdapter;

pub struct ErisTf {
    pub state_cache: Option<StateResponse>,
    pub undelegation_records_cache: Option<Vec<UndelegationCacheItem>>,

    pub wallet: Addr,
    pub addr: Addr,
    pub denom: String,
}

pub struct UndelegationCacheItem {
    pub token_amount: Uint128,
    pub reconciled: bool,
    pub exchange_rate: Decimal,
    pub batch_id: u64,
}

impl ErisTf {
    fn query_unbond_requests_by_user(
        &mut self,
        deps: &Deps,
    ) -> CustomResult<Vec<UnbondRequestsByUserResponseItem>> {
        deps.querier
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: self.addr.to_string(),
                msg: to_binary(&QueryMsg::UnbondRequestsByUser {
                    user: self.wallet.to_string(),
                    limit: Some(100u32),
                    start_after: None,
                })
                .unwrap(),
            }))
            .map_err(|a| adapter_error("eris", "query_unbond_requests_by_user", a))
    }

    fn query_previous_batch(
        &mut self,
        deps: &Deps,
        id: u64,
    ) -> Result<Batch, crate::error::ContractError> {
        deps.querier
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: self.addr.to_string(),
                msg: to_binary(&QueryMsg::PreviousBatch(id)).unwrap(),
            }))
            .map_err(|a| adapter_error("eris", "query_previous_batch", a))
    }

    fn query_pending_batch(
        &mut self,
        deps: &Deps,
    ) -> Result<PendingBatch, crate::error::ContractError> {
        deps.querier
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: self.addr.to_string(),
                msg: to_binary(&QueryMsg::PendingBatch {}).unwrap(),
            }))
            .map_err(|a| adapter_error("eris", "query_pending_batch", a))
    }

    fn query_state(&mut self, deps: &Deps) -> CustomResult<StateResponse> {
        deps.querier
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: self.addr.to_string(),
                msg: to_binary(&QueryMsg::State {}).unwrap(),
            }))
            .map_err(|a| adapter_error("eris", "query_state", a))
    }

    fn get_unbond_msg(&self, amount: Uint128) -> CustomResult<CosmosMsg<CustomMsgType>> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.addr.to_string(),
            funds: vec![coin(amount.u128(), self.denom.to_string())],
            msg: to_binary(&ExecuteMsg::QueueUnbond {
                receiver: None,
            })?,
        }))
    }

    fn get_withdraw_unbonded_msg(&mut self) -> CustomResult<CosmosMsg<CustomMsgType>> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.addr.to_string(),
            funds: vec![],
            msg: to_binary(&ExecuteMsg::WithdrawUnbonded {
                receiver: None,
            })?,
        }))
    }

    fn cached_query_undelegation_records(
        &mut self,
        deps: &Deps,
    ) -> CustomResult<&Vec<UndelegationCacheItem>> {
        if self.undelegation_records_cache.is_none() {
            let result: Vec<UnbondRequestsByUserResponseItem> =
                self.query_unbond_requests_by_user(deps)?;

            if result.is_empty() {
                self.undelegation_records_cache = Some(vec![]);
                return Ok(self.undelegation_records_cache.as_ref().unwrap());
            }

            let current: PendingBatch = self.query_pending_batch(deps)?;

            let result: Vec<UndelegationCacheItem> = result
                .iter()
                .map(|item| {
                    Ok(if item.id == current.id {
                        let state = self.cached_state(deps)?;
                        let exchange_rate = state.exchange_rate;
                        UndelegationCacheItem {
                            batch_id: item.id,
                            token_amount: item.shares,
                            reconciled: false,
                            exchange_rate,
                        }
                    } else {
                        let previous: Batch = self.query_previous_batch(deps, item.id)?;

                        UndelegationCacheItem {
                            batch_id: item.id,
                            token_amount: item.shares,
                            reconciled: previous.reconciled,
                            exchange_rate: Decimal::from_ratio(
                                previous.utoken_unclaimed,
                                previous.total_shares,
                            ),
                        }
                    })
                })
                .collect::<CustomResult<Vec<UndelegationCacheItem>>>()?;

            self.undelegation_records_cache = Some(result);
        }

        Ok(self.undelegation_records_cache.as_ref().unwrap())
    }

    fn cached_state(&mut self, deps: &Deps) -> CustomResult<&StateResponse> {
        if self.state_cache.is_none() {
            let result: StateResponse = self.query_state(deps)?;
            self.state_cache = Some(result);
        }

        Ok(self.state_cache.as_ref().unwrap())
    }
}

impl LsdAdapter for ErisTf {
    fn used_contracts(&self) -> Vec<Addr> {
        vec![self.addr.clone()]
    }

    fn asset(&self) -> AssetInfo {
        native_asset_info(self.denom.clone())
    }

    fn unbond(&self, _deps: &Deps, amount: Uint128) -> CustomResult<Vec<CosmosMsg<CustomMsgType>>> {
        Ok(vec![self.get_unbond_msg(amount)?])
    }

    fn query_unbonding(&mut self, deps: &Deps) -> CustomResult<Uint128> {
        Ok(self
            .cached_query_undelegation_records(deps)?
            .iter()
            .filter(|a| !a.reconciled)
            .map(|batch| batch.exchange_rate * batch.token_amount)
            .sum())
    }

    fn withdraw(
        &mut self,
        _deps: &Deps,
        _amount: Uint128,
    ) -> CustomResult<Vec<CosmosMsg<CustomMsgType>>> {
        Ok(vec![self.get_withdraw_unbonded_msg()?])
    }

    fn query_withdrawable(&mut self, deps: &Deps) -> CustomResult<Uint128> {
        Ok(self
            .cached_query_undelegation_records(deps)?
            .iter()
            .filter(|a| a.reconciled)
            .map(|batch| batch.exchange_rate * batch.token_amount)
            .sum())
    }

    fn query_factor_x_to_normal(&mut self, deps: &Deps) -> CustomResult<Decimal> {
        Ok(self.cached_state(deps)?.exchange_rate)
    }
}
