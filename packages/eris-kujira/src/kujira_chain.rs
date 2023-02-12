use cosmwasm_std::{Addr, Coin, CosmosMsg, StdResult, Uint128};
use eris_chain_shared::chain_trait::ChainInterface;
use kujira::msg::DenomMsg;

use crate::{
    adapters::{bow_vault::BowVault, bw_vault::BlackWhaleVault},
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

    fn create_mint_msg(
        &self,
        full_denom: String,
        amount: Uint128,
        recipient: Addr,
    ) -> CosmosMsg<CustomMsgType> {
        DenomMsg::Mint {
            denom: full_denom.into(),
            amount,
            recipient,
        }
        .into()
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
        coin: &Coin,
    ) -> StdResult<Option<CosmosMsg<CustomMsgType>>>
    where
        F: FnOnce() -> StdResult<HubChainConfig>,
    {
        match withdraw_type {
            WithdrawType::BlackWhale {
                addr,
            } => Ok(Some(BlackWhaleVault(addr).withdraw_msg(denom, coin.amount)?)),
            WithdrawType::Bow {
                addr,
            } => Ok(Some(BowVault(addr).withdraw_msg(denom, coin.amount)?)),
        }
    }

    fn use_multi_stages_swap(&self) -> bool {
        true
    }

    fn create_multi_stages_swap_msgs<F>(
        &self,
        get_chain_config: F,
        stages: Vec<Vec<(StageType, DenomType)>>,
        balances: Vec<Coin>,
    ) -> StdResult<Vec<CosmosMsg<CustomMsgType>>>
    where
        F: FnOnce() -> StdResult<HubChainConfig>,
    {
        let config = get_chain_config()?;

        let fin_swaps: Vec<Vec<(Addr, DenomType)>> = stages
            .into_iter()
            .map(|stage| {
                stage
                    .into_iter()
                    .map(|(stage_type, denom)| match stage_type {
                        StageType::Fin {
                            addr,
                        } => (addr, denom),
                    })
                    .collect()
            })
            .collect();

        Ok(vec![config.fin_multi.swap_msg(fin_swaps, balances)?])
    }

    fn create_single_stage_swap_msgs<F>(
        &self,
        _get_chain_config: F,
        _stages: (StageType, DenomType),
        _balance: &Coin,
    ) -> StdResult<CosmosMsg<CustomMsgType>>
    where
        F: FnOnce() -> StdResult<HubChainConfig>,
    {
        Err(cosmwasm_std::StdError::generic_err("not supported on kujira"))
    }
}
