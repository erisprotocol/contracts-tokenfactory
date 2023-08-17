use std::collections::HashMap;

use astroport::asset::{Asset, AssetInfo};
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, MessageInfo, QuerierWrapper, StdError,
    StdResult, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Expiration};
use eris_chain_adapter::types::CustomMsgType;

pub trait AssetInfosEx {
    fn query_balances(&self, querier: &QuerierWrapper, address: &Addr) -> StdResult<Vec<Asset>>;
}

impl AssetInfosEx for Vec<AssetInfo> {
    fn query_balances(&self, querier: &QuerierWrapper, address: &Addr) -> StdResult<Vec<Asset>> {
        let assets: Vec<Asset> = self
            .iter()
            .map(|asset| {
                let result = asset.query_pool(querier, address)?;
                Ok(Asset {
                    info: asset.clone(),
                    amount: result,
                })
            })
            .collect::<StdResult<_>>()?;

        Ok(assets.into_iter().collect())
    }
}
pub trait AssetInfoEx {
    fn to_addr(&self) -> Addr;
}

impl AssetInfoEx for AssetInfo {
    fn to_addr(&self) -> Addr {
        match self {
            AssetInfo::NativeToken {
                denom,
            } => Addr::unchecked(denom.clone()),
            AssetInfo::Token {
                contract_addr,
            } => contract_addr.clone(),
        }
    }
}

pub trait AssetsEx {
    fn query_balance_diff(
        self,
        querier: &QuerierWrapper,
        address: &Addr,
        max_amount: Option<Vec<Asset>>,
    ) -> StdResult<Vec<Asset>>;
}

impl AssetsEx for Vec<Asset> {
    fn query_balance_diff(
        self,
        querier: &QuerierWrapper,
        address: &Addr,
        max_amount: Option<Vec<Asset>>,
    ) -> StdResult<Vec<Asset>> {
        let hash_map = max_amount.map(|max| {
            let hash: HashMap<AssetInfo, Uint128> =
                max.into_iter().map(|asset| (asset.info, asset.amount)).collect();
            hash
        });

        let assets: Vec<Asset> = self
            .into_iter()
            .map(|asset| {
                let result = asset.info.query_pool(querier, address)?;
                let mut amount = result.checked_sub(asset.amount)?;

                if let Some(hash_map) = &hash_map {
                    if let Some(max) = hash_map.get(&asset.info) {
                        if !max.is_zero() {
                            amount = std::cmp::min(amount, *max);
                        }
                    }
                }

                Ok(Asset {
                    info: asset.info,
                    amount,
                })
            })
            .collect::<StdResult<_>>()?;

        Ok(assets.into_iter().filter(|asset| !asset.amount.is_zero()).collect())
    }
}

pub trait AssetEx {
    fn transfer_msg(&self, to: &Addr) -> StdResult<CosmosMsg<CustomMsgType>>;
    fn transfer_msg_target(
        &self,
        to_addr: &Addr,
        to_msg: Option<Binary>,
    ) -> StdResult<CosmosMsg<CustomMsgType>>;
    fn transfer_from_msg(&self, from: &Addr, to: &Addr) -> StdResult<CosmosMsg<CustomMsgType>>;
    fn increase_allowance_msg(
        &self,
        spender: String,
        expires: Option<Expiration>,
    ) -> StdResult<CosmosMsg<CustomMsgType>>;

    fn deposit_asset(
        &self,
        info: &MessageInfo,
        recipient: &Addr,
        messages: &mut Vec<CosmosMsg<CustomMsgType>>,
    ) -> StdResult<()>;
}

impl AssetEx for Asset {
    fn transfer_msg(&self, to: &Addr) -> StdResult<CosmosMsg<CustomMsgType>> {
        match &self.info {
            AssetInfo::Token {
                contract_addr,
            } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: to.to_string(),
                    amount: self.amount,
                })?,
                funds: vec![],
            })),
            AssetInfo::NativeToken {
                denom,
            } => Ok(CosmosMsg::Bank(BankMsg::Send {
                to_address: to.to_string(),
                amount: vec![Coin {
                    denom: denom.to_string(),
                    amount: self.amount,
                }],
            })),
        }
    }

    fn transfer_msg_target(
        &self,
        to_addr: &Addr,
        to_msg: Option<Binary>,
    ) -> StdResult<CosmosMsg<CustomMsgType>> {
        if let Some(msg) = to_msg {
            match &self.info {
                AssetInfo::Token {
                    contract_addr,
                } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: to_addr.to_string(),
                        amount: self.amount,
                        msg,
                    })?,
                    funds: vec![],
                })),
                AssetInfo::NativeToken {
                    denom,
                } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: to_addr.to_string(),
                    msg,
                    funds: vec![Coin {
                        denom: denom.to_string(),
                        amount: self.amount,
                    }],
                })),
            }
        } else {
            self.transfer_msg(to_addr)
        }
    }

    fn transfer_from_msg(&self, from: &Addr, to: &Addr) -> StdResult<CosmosMsg<CustomMsgType>> {
        match &self.info {
            AssetInfo::Token {
                contract_addr,
            } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: from.to_string(),
                    recipient: to.to_string(),
                    amount: self.amount,
                })?,
                funds: vec![],
            })),
            AssetInfo::NativeToken {
                ..
            } => Err(StdError::generic_err("TransferFrom does not apply to native tokens")),
        }
    }

    fn increase_allowance_msg(
        &self,
        spender: String,
        expires: Option<Expiration>,
    ) -> StdResult<CosmosMsg<CustomMsgType>> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.info.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
                spender,
                amount: self.amount,
                expires,
            })?,
            funds: vec![],
        }))
    }

    fn deposit_asset(
        &self,
        info: &MessageInfo,
        recipient: &Addr,
        messages: &mut Vec<CosmosMsg<CustomMsgType>>,
    ) -> StdResult<()> {
        if self.amount.is_zero() {
            return Ok(());
        }

        match &self.info {
            AssetInfo::Token {
                ..
            } => {
                messages.push(self.transfer_from_msg(&info.sender, recipient)?);
            },
            AssetInfo::NativeToken {
                denom,
            } => {
                has_paid(info, denom)?;
            },
        };
        Ok(())
    }
}

/// Similar to must_pay, but it any payment is optional. Returns an error if a different
/// denom was sent. Otherwise, returns the amount of `denom` sent, or 0 if nothing sent.
fn has_paid(info: &MessageInfo, denom: &str) -> Result<Uint128, StdError> {
    if info.funds.is_empty() {
        return Err(StdError::generic_err("No funds sent"));
    }

    let coin = info.funds.iter().find(|c| c.denom == denom);

    if let Some(coin) = coin {
        Ok(coin.amount)
    } else {
        Err(StdError::generic_err(format!("Must send reserve token '{0}'", denom)))
    }
}
