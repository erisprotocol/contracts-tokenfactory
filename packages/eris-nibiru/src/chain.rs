use cosmwasm_std::{Addr, CosmosMsg, Decimal, StdResult, Uint128};
use eris_chain_shared::chain_trait::ChainInterface;

use crate::{
    adapters::whitewhaledex::WhiteWhalePair,
    denom::{MsgBurn, MsgCreateDenom, MsgMint},
    types::{CustomMsgType, DenomType, HubChainConfig, StageType, WithdrawType},
};

pub struct Chain {
    pub contract: Addr,
}

impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig> for Chain {
    fn create_denom_msg(&self, _full_denom: String, subdenom: String) -> CosmosMsg<CustomMsgType> {
        MsgCreateDenom {
            sender: self.contract.to_string(),
            subdenom,
        }
        .into()
    }

    fn create_mint_msgs(
        &self,
        full_denom: String,
        amount: Uint128,
        recipient: Addr,
    ) -> Vec<CosmosMsg<CustomMsgType>> {
        vec![MsgMint {
            sender: self.contract.to_string(),
            amount: Some(crate::denom::Coin {
                denom: full_denom.to_string(),
                amount: amount.to_string(),
            }),
            mint_to: recipient.to_string(),
        }
        .into()]
    }

    fn create_burn_msg(&self, full_denom: String, amount: Uint128) -> CosmosMsg<CustomMsgType> {
        MsgBurn {
            sender: self.contract.to_string(),
            amount: Some(crate::denom::Coin {
                denom: full_denom,
                amount: amount.to_string(),
            }),
            burn_from: self.contract.to_string(),
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
            WithdrawType::Dex {
                addr,
            } => Ok(Some(WhiteWhalePair(addr).withdraw_msg(denom, amount)?)),
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
            StageType::Dex {
                addr,
            } => WhiteWhalePair(addr).swap_msg(denom, amount, belief_price, Some(max_spread)),
        }
    }
}
