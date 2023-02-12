use std::collections::HashSet;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, StdResult, WasmMsg};
use kujira::{denom::Denom, msg::KujiraMsg};

#[cw_serde]
pub struct FinMultiExecuteMsg {
    pub stages: Vec<Vec<(Addr, Denom)>>,
    pub recipient: Option<Addr>,
}

#[cw_serde]
pub struct FinMulti(pub Addr);

impl FinMulti {
    pub fn swap_msg(
        &self,
        stages: Vec<Vec<(Addr, Denom)>>,
        balances: Vec<Coin>,
    ) -> StdResult<CosmosMsg<KujiraMsg>> {
        let mut set = HashSet::new();
        for stage in stages.iter() {
            for (_, denom) in stage {
                set.insert(denom.to_string());
            }
        }

        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            funds: balances.into_iter().filter(|b| set.contains(&b.denom)).collect(),
            msg: to_binary(&FinMultiExecuteMsg {
                stages,
                recipient: None,
            })?,
        }))
    }
}
