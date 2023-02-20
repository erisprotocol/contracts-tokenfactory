use cosmwasm_std::{coin, Addr, CosmosMsg, Decimal, StdResult, Uint128};
use eris_chain_shared::chain_trait::ChainInterface;
use kujira::msg::DenomMsg;

use crate::{
    adapters::{bow_vault::BowVault, bw_vault::BlackWhaleVault, fin::Fin},
    kujira_types::{CustomMsgType, DenomType, HubChainConfig, StageType, WithdrawType},
};

pub struct KujiraChain {}

impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig>
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

    fn create_withdraw_msg<F>(
        &self,
        _get_chain_config: F,
        withdraw_type: WithdrawType,
        denom: DenomType,
        amount: Uint128,
    ) -> StdResult<Option<CosmosMsg<CustomMsgType>>>
    where
        F: FnOnce() -> StdResult<HubChainConfig>,
    {
        match withdraw_type {
            WithdrawType::BlackWhale {
                addr,
            } => Ok(Some(BlackWhaleVault(addr).withdraw_msg(denom, amount)?)),
            WithdrawType::Bow {
                addr,
            } => Ok(Some(BowVault(addr).withdraw_msg(denom, amount)?)),
        }
    }

    fn create_single_stage_swap_msgs<F>(
        &self,
        _get_chain_config: F,
        stage_type: StageType,
        denom: DenomType,
        amount: Uint128,
        belief_price: Option<Decimal>,
        max_spread: Decimal,
    ) -> StdResult<CosmosMsg<CustomMsgType>>
    where
        F: FnOnce() -> StdResult<HubChainConfig>,
    {
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
}
