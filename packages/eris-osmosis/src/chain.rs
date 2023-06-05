use std::ops::Div;

use cosmwasm_std::StdError;
use cosmwasm_std::{Addr, CosmosMsg, Decimal, StdResult, Uint128};
use eris_chain_shared::chain_trait::ChainInterface;
use osmosis_std::types::cosmos::base::v1beta1::Coin;
use osmosis_std::types::osmosis::gamm::v1beta1::MsgExitPool;
use osmosis_std::types::osmosis::gamm::v1beta1::MsgSwapExactAmountIn;
use osmosis_std::types::osmosis::poolmanager::v1beta1::SwapAmountInRoute;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgBurn;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgCreateDenom;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgMint;

use crate::types::{CustomMsgType, DenomType, HubChainConfig, StageType, WithdrawType};

pub struct OsmosisChain {
    pub contract: Addr,
}

impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig>
    for OsmosisChain
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
        vec![MsgMint {
            sender: self.contract.to_string(),
            mint_to_address: recipient.to_string(),
            amount: Some(Coin {
                denom: full_denom,
                amount: amount.to_string(),
            }),
        }
        .into()]
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
            StageType::Osmo {
                pool_id,
                token_out_denom,
            } => {
                if let Some(belief_price) = belief_price {
                    // Spread 0.1
                    // belief price 1.5
                    // in tokens 1000
                    // out tokens @ belief price 666
                    // out tokens @ belief - spread = 606
                    // 1000 / (1.5 + 10%)

                    let slippage_price = Decimal::one().checked_add(max_spread)? * belief_price;
                    let out_amount = Decimal::one().div(slippage_price) * amount;

                    Ok(MsgSwapExactAmountIn {
                        sender: self.contract.to_string(),
                        routes: vec![SwapAmountInRoute {
                            pool_id,
                            token_out_denom,
                        }],
                        token_in: Some(osmosis_std::types::cosmos::base::v1beta1::Coin {
                            denom,
                            amount: amount.to_string(),
                        }),

                        token_out_min_amount: out_amount.to_string(),
                    }
                    .into())
                } else {
                    Err(StdError::generic_err("belief_price required"))
                }
            },
        }
    }
}
