use astroport::asset::AssetInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coin, to_json_binary, Addr, CosmosMsg, StdResult, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

use crate::types::CustomMsgType;

#[cw_serde]
pub struct Hub(pub Addr);

#[cw_serde]
pub enum ExecuteMsg {
    /// Bond specified amount of Token
    Bond {
        receiver: Option<String>,
    },
}

impl Hub {
    pub fn bond_msg(
        &self,
        denom: AssetInfo,
        amount: Uint128,
        receiver: Option<String>,
    ) -> StdResult<CosmosMsg<CustomMsgType>> {
        match denom {
            AssetInfo::Token {
                contract_addr,
            } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::Send {
                    contract: self.0.to_string(),
                    amount,
                    msg: to_json_binary(&ExecuteMsg::Bond {
                        receiver,
                    })?,
                })?,
                funds: vec![],
            })),
            AssetInfo::NativeToken {
                denom,
            } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: self.0.to_string(),
                msg: to_json_binary(&ExecuteMsg::Bond {
                    receiver,
                })?,
                funds: vec![coin(amount.u128(), denom)],
            })),
        }
    }
}
