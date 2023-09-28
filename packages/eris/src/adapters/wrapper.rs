use astroport::asset::{Asset, AssetInfo};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, CosmosMsg, StdError, StdResult, WasmMsg};
use cw20::Cw20ExecuteMsg;

pub struct Wrapper(pub Addr);

#[cw_serde]
pub enum CustomCw20HookMsg {
    /// Swap a given amount of asset
    Unwrap {},
}

impl Wrapper {
    pub fn unwrap_msg(&self, from: &Asset) -> StdResult<CosmosMsg> {
        let wasm_msg = match &from.info {
            AssetInfo::Token {
                contract_addr,
            } => WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: self.0.to_string(),
                    amount: from.amount,
                    msg: to_binary(&CustomCw20HookMsg::Unwrap {})?,
                })?,
                funds: vec![],
            },

            AssetInfo::NativeToken {
                ..
            } => return Err(StdError::generic_err("Unwrapping not supported by native token")),
        };

        Ok(CosmosMsg::Wasm(wasm_msg))
    }
}
