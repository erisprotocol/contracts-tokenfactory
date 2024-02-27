use cosmwasm_std::{coin, to_json_binary, Addr, CosmosMsg, Decimal, StdResult, Uint128, WasmMsg};
use eris_chain_shared::chain_trait::ChainInterface;
use kujira::msg::DenomMsg;

use crate::{
    adapters::{bow_vault::BowVault, bw_vault::BlackWhaleVault, fin::Fin},
    types::{CoinType, CustomMsgType, DenomType, MultiSwapRouterType, StageType, WithdrawType},
};

pub struct KujiraChain {}

impl
    ChainInterface<CustomMsgType, DenomType, CoinType, WithdrawType, StageType, MultiSwapRouterType>
    for KujiraChain
{
    fn create_denom_msg(&self, _full_denom: String, subdenom: String) -> CosmosMsg<CustomMsgType> {
        DenomMsg::Create {
            subdenom: subdenom.into(),
        }
        .into()
    }

    fn create_mint_msgs(
        &self,
        full_denom: String,
        amount: Uint128,
        recipient: Addr,
    ) -> Vec<CosmosMsg<CustomMsgType>> {
        vec![DenomMsg::Mint {
            denom: full_denom.into(),
            amount,
            recipient,
        }
        .into()]
    }

    fn create_burn_msg(&self, full_denom: String, amount: Uint128) -> CosmosMsg<CustomMsgType> {
        DenomMsg::Burn {
            denom: full_denom.into(),
            amount,
        }
        .into()
    }

    fn create_withdraw_msg(
        &self,
        withdraw_type: WithdrawType,
        denom: DenomType,
        amount: Uint128,
    ) -> StdResult<Option<CosmosMsg<CustomMsgType>>> {
        match withdraw_type {
            WithdrawType::BlackWhale {
                addr,
            } => Ok(Some(BlackWhaleVault(addr).withdraw_msg(denom, amount)?)),
            WithdrawType::Bow {
                addr,
            } => Ok(Some(BowVault(addr).withdraw_msg(denom, amount)?)),
        }
    }

    fn create_single_stage_swap_msgs(
        &self,
        stage_type: StageType,
        denom: DenomType,
        amount: Uint128,
        belief_price: Option<Decimal>,
        max_spread: Decimal,
    ) -> StdResult<CosmosMsg<CustomMsgType>> {
        match stage_type {
            StageType::Fin {
                addr,
            } => Fin(addr).swap_msg(
                &coin(amount.u128(), denom.to_string()),
                belief_price,
                Some(max_spread),
            ),
        }
    }

    fn create_multi_swap_router_msgs(
        &self,
        router_type: MultiSwapRouterType,
        funds: Vec<CoinType>,
    ) -> StdResult<Vec<CosmosMsg<CustomMsgType>>> {
        match router_type {
            MultiSwapRouterType::Manta {
                addr,
                msg,
            } => Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addr.to_string(),
                funds,
                msg: to_json_binary(&msg)?,
            })]),
        }
    }

    fn equals_asset_info(
        &self,
        denom_type: &DenomType,
        asset_info: &astroport::asset::AssetInfo,
    ) -> bool {
        match asset_info {
            astroport::asset::AssetInfo::Token {
                ..
            } => false,
            astroport::asset::AssetInfo::NativeToken {
                denom,
            } => *denom == denom_type.to_string(),
        }
    }

    fn get_coin(&self, denom: DenomType, amount: Uint128) -> CoinType {
        denom.coin(&amount)
    }
}
