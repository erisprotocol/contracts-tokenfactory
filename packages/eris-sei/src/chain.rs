use cosmwasm_std::{coin, coins, Coin, StdError};
use cosmwasm_std::{Addr, CosmosMsg, Decimal, StdResult, Uint128};
use eris_chain_shared::chain_trait::ChainInterface;
use sei_cosmwasm::SeiMsg;

use crate::types::{CustomMsgType, DenomType, HubChainConfig, StageType, WithdrawType};

pub struct Chain {
    pub contract: Addr,
}

impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig> for Chain {
    fn create_denom_msg(&self, _full_denom: String, subdenom: String) -> CosmosMsg<CustomMsgType> {
        SeiMsg::CreateDenom {
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
        vec![
            SeiMsg::MintTokens {
                amount: coin(amount.u128(), full_denom.clone()),
            }
            .into(),
            CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
                to_address: recipient.to_string(),
                amount: coins(amount.u128(), full_denom),
            }),
        ]
    }

    fn create_burn_msg(&self, full_denom: String, amount: Uint128) -> CosmosMsg<CustomMsgType> {
        SeiMsg::BurnTokens {
            amount: Coin {
                denom: full_denom,
                amount,
            },
        }
        .into()
    }

    fn create_withdraw_msg<F>(
        &self,
        _get_chain_config: F,
        _withdraw_type: WithdrawType,
        _denom: DenomType,
        _amount: Uint128,
    ) -> StdResult<Option<CosmosMsg<CustomMsgType>>>
    where
        F: FnOnce() -> StdResult<HubChainConfig>,
    {
        Err(StdError::generic_err("not supported"))
    }

    fn create_single_stage_swap_msgs<F>(
        &self,
        _get_chain_config: F,
        _stage_type: StageType,
        _denom: DenomType,
        _amount: Uint128,
        _belief_price: Option<Decimal>,
        _max_spread: Decimal,
    ) -> StdResult<CosmosMsg<CustomMsgType>>
    where
        F: FnOnce() -> StdResult<HubChainConfig>,
    {
        Err(StdError::generic_err("not supported"))
    }
}
