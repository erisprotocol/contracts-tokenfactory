use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coins, to_json_binary, Addr, CosmosMsg, Decimal, StdResult, Uint128, WasmMsg};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::types::{CustomMsgType, DenomType};

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
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> StdResult<CosmosMsg<CustomMsgType>> {
        match denom {
            astroport::asset::AssetInfo::Token {
                contract_addr,
            } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                funds: vec![],
                msg: to_json_binary(&Cw20ExecuteMsg::Send {
                    contract: self.0.to_string(),
                    amount,
                    msg: to_json_binary(&Cw20HookMsg::Swap {
                        belief_price,
                        max_spread,
                        to: None,
                    })?,
                })?,
            })),
            astroport::asset::AssetInfo::NativeToken {
                denom,
            } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: self.0.to_string(),
                funds: coins(amount.u128(), denom.clone()),
                msg: to_json_binary(&ExecuteMsg::Swap {
                    offer_asset: Asset {
                        info: AssetInfo::NativeToken {
                            denom,
                        },
                        amount,
                    },
                    belief_price,
                    max_spread,
                    to: None,
                })?,
            })),
        }
    }

    pub fn withdraw_msg(
        &self,
        denom: DenomType,
        amount: Uint128,
    ) -> StdResult<CosmosMsg<CustomMsgType>> {
        match denom {
            astroport::asset::AssetInfo::Token {
                contract_addr,
            } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                funds: vec![],
                msg: to_json_binary(&Cw20ExecuteMsg::Send {
                    contract: self.0.to_string(),
                    amount,
                    msg: to_json_binary(&Cw20HookMsg::WithdrawLiquidity {})?,
                })?,
            })),
            astroport::asset::AssetInfo::NativeToken {
                denom,
            } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: self.0.to_string(),
                funds: coins(amount.u128(), denom),
                msg: to_json_binary(&ExecuteMsg::WithdrawLiquidity {})?,
            })),
        }
    }
}
