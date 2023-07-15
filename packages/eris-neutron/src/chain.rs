use cosmwasm_std::{coins, StdError};
use cosmwasm_std::{Addr, CosmosMsg, Decimal, StdResult, Uint128};
use eris_chain_shared::chain_trait::ChainInterface;
use osmosis_std::types::cosmos::base::v1beta1::Coin;
use osmosis_std::types::osmosis::gamm::v1beta1::MsgExitPool;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgBurn;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgCreateDenom;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgMint;

use crate::types::{CustomMsgType, DenomType, HubChainConfig, StageType, WithdrawType};

pub struct NeutronChain {
    pub contract: Addr,
}

impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig>
    for NeutronChain
{
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
        vec![
            MsgMint {
                sender: self.contract.to_string(),
                // osmosis cant mint to a different address than itself.
                mint_to_address: self.contract.to_string(),
                amount: Some(Coin {
                    denom: full_denom.clone(),
                    amount: amount.to_string(),
                }),
            }
            .into(),
            CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
                to_address: recipient.to_string(),
                amount: coins(amount.u128(), full_denom),
            }),
        ]
    }

    fn create_burn_msg(&self, full_denom: String, amount: Uint128) -> CosmosMsg<CustomMsgType> {
        MsgBurn {
            sender: self.contract.to_string(),
            burn_from_address: self.contract.to_string(),
            amount: Some(Coin {
                denom: full_denom,
                amount: amount.to_string(),
            }),
        }
        .into()
    }

    fn create_withdraw_msg<F>(
        &self,
        _get_chain_config: F,
        withdraw_type: WithdrawType,
        _denom: DenomType,
        amount: Uint128,
    ) -> StdResult<Option<CosmosMsg<CustomMsgType>>>
    where
        F: FnOnce() -> StdResult<HubChainConfig>,
    {
        match withdraw_type {
            WithdrawType::Withdraw {
                pool_id,
                token_out_mins,
            } => Ok(Some(
                MsgExitPool {
                    share_in_amount: amount.to_string(),
                    pool_id,
                    sender: self.contract.to_string(),
                    token_out_mins: token_out_mins
                        .into_iter()
                        .map(|c| Coin {
                            amount: c.amount.to_string(),
                            denom: c.denom,
                        })
                        .collect(),
                }
                .into(),
            )),
        }
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
