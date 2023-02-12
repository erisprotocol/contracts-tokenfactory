use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, StdResult, Uint128, WasmMsg};
use kujira::{denom::Denom, msg::KujiraMsg};

#[cw_serde]
pub enum BowExecuteMsg {
    Withdraw {},
}

#[cw_serde]
pub struct BowVault(pub Addr);

impl BowVault {
    pub fn withdraw_msg(&self, denom: Denom, amount: Uint128) -> StdResult<CosmosMsg<KujiraMsg>> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            funds: vec![Coin {
                amount,
                denom: denom.to_string(),
            }],
            msg: to_binary(&BowExecuteMsg::Withdraw {})?,
        }))
    }
}
