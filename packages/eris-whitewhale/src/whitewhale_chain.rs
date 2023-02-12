use cosmwasm_std::{Addr, Coin, CosmosMsg, StdResult, Uint128};
use eris_chain_shared::chain_trait::ChainInterface;

use crate::whitewhale_types::{CustomMsgType, DenomType, HubChainConfig, StageType, WithdrawType};

pub struct WhiteWhaleChain {}

impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig>
    for WhiteWhaleChain
{
    fn create_denom_msg(&self, _full_denom: String, subdenom: String) -> CosmosMsg<CustomMsgType> {
        panic!("todo");
    }

    fn create_mint_msg(
        &self,
        full_denom: String,
        amount: Uint128,
        recipient: Addr,
    ) -> CosmosMsg<CustomMsgType> {
        panic!("todo");
    }

    fn create_burn_msg(&self, full_denom: String, amount: Uint128) -> CosmosMsg<CustomMsgType> {
        panic!("todo");
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
        panic!("todo");

        // match withdraw_type {
        //     WithdrawType::Dex {
        //         addr,
        //     } => Ok(Some(BlackWhaleVault(addr).withdraw_msg(denom, coin.amount)?)),
        // }
    }

    fn use_multi_stages_swap(&self) -> bool {
        false
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
        Err(cosmwasm_std::StdError::generic_err("not supported on whitewhale"))
        // let config = get_chain_config()?;

        // let fin_swaps: Vec<Vec<(Addr, DenomType)>> = stages
        //     .into_iter()
        //     .map(|stage| {
        //         stage
        //             .into_iter()
        //             .map(|(stage_type, denom)| match stage_type {
        //                 StageType::Fin {
        //                     addr,
        //                 } => (addr, denom),
        //             })
        //             .collect()
        //     })
        //     .collect();

        // Ok(vec![config.fin_multi.swap_msg(fin_swaps, balances)?])
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
        panic!("todo");
    }
}
