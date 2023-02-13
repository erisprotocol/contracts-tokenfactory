use cosmwasm_std::{Addr, Coin, CosmosMsg, StdResult, Uint128};
use eris_chain_shared::chain_trait::ChainInterface;

use crate::whitewhale_types::{CustomMsgType, DenomType, HubChainConfig, StageType, WithdrawType};

pub struct WhiteWhaleChain {}

impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig>
    for WhiteWhaleChain
{
    fn create_denom_msg(&self, _full_denom: String, _subdenom: String) -> CosmosMsg<CustomMsgType> {
        panic!("todo");
    }

    fn create_mint_msg(
        &self,
        _full_denom: String,
        _amount: Uint128,
        _recipient: Addr,
    ) -> CosmosMsg<CustomMsgType> {
        panic!("todo");
    }

    fn create_burn_msg(&self, _full_denom: String, _amount: Uint128) -> CosmosMsg<CustomMsgType> {
        panic!("todo");
    }

    fn create_withdraw_msg<F>(
        &self,
        _get_chain_config: F,
        _withdraw_type: WithdrawType,
        _denom: DenomType,
        _coin: &Coin,
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

    fn create_single_stage_swap_msgs<F>(
        &self,
        _get_chain_config: F,
        _stage_type: StageType,
        _denom: DenomType,
        _balance: &Coin,
    ) -> StdResult<CosmosMsg<CustomMsgType>>
    where
        F: FnOnce() -> StdResult<HubChainConfig>,
    {
        panic!("todo");
    }
}
