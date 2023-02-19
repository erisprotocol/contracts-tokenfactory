use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coins, to_binary, Addr, CosmosMsg, Decimal, StdError, StdResult, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::whitewhale_types::{CustomMsgType, DenomType};

#[cw_serde]
pub enum AssetInfo {
    Token {
        contract_addr: String,
    },
    NativeToken {
        denom: String,
    },
}

#[cw_serde]
pub struct Asset {
    pub info: AssetInfo,
    pub amount: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Used to trigger the [Cw20HookMsg] messages
    Receive(Cw20ReceiveMsg),
    /// Swap an offer asset to the other
    Swap {
        offer_asset: Asset,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        to: Option<String>,
    },
    /// Withdraws liquidity
    WithdrawLiquidity {},
}

#[cw_serde]
pub enum Cw20HookMsg {
    /// Sell a given amount of asset
    Swap {
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        to: Option<String>,
    },
    /// Withdraws liquidity
    WithdrawLiquidity {},
}

#[cw_serde]
pub struct WhiteWhalePair(pub Addr);

impl WhiteWhalePair {
    pub fn swap_msg(
        &self,
        denom: DenomType,
        amount: Uint128,
    ) -> StdResult<CosmosMsg<CustomMsgType>> {
        match denom {
            cw_asset::AssetInfoBase::Cw20(cw20) => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cw20.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: self.0.to_string(),
                    amount,
                    msg: to_binary(&Cw20HookMsg::Swap {
                        belief_price: None,
                        max_spread: None,
                        to: None,
                    })?,
                })?,
            })),
            cw_asset::AssetInfoBase::Native(native) => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: self.0.to_string(),
                funds: coins(amount.u128(), native.clone()),
                msg: to_binary(&ExecuteMsg::Swap {
                    offer_asset: Asset {
                        info: AssetInfo::NativeToken {
                            denom: native,
                        },
                        amount,
                    },
                    belief_price: None,
                    max_spread: None,
                    to: None,
                })?,
            })),
            _ => Err(StdError::generic_err("WhiteWhalePair.swap_msg: not supported")),
        }
    }

    pub fn withdraw_msg(
        &self,
        denom: DenomType,
        amount: Uint128,
    ) -> StdResult<CosmosMsg<CustomMsgType>> {
        match denom {
            cw_asset::AssetInfoBase::Cw20(cw20) => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cw20.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: self.0.to_string(),
                    amount,
                    msg: to_binary(&Cw20HookMsg::WithdrawLiquidity {})?,
                })?,
            })),
            cw_asset::AssetInfoBase::Native(native) => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: self.0.to_string(),
                funds: coins(amount.u128(), native),
                msg: to_binary(&ExecuteMsg::WithdrawLiquidity {})?,
            })),
            _ => Err(StdError::generic_err("WhiteWhalePair.withdraw_msg: not supported")),
        }
    }
}
