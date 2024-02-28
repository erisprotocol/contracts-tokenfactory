use cosmwasm_std::{
    coins, to_json_binary, Addr, CosmosMsg, Decimal, StdError, StdResult, Uint128, WasmMsg,
};
use eris_chain_shared::chain_trait::ChainInterface;

use crate::{
    adapters::{hub::Hub, whitewhaledex::WhiteWhalePair},
    custom_execute_msg::CustomExecuteMsg,
    types::{CustomMsgType, DenomType, HubChainConfig, StageType, WithdrawType},
};

pub struct Chain {
    pub contract: Addr,
}

impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig> for Chain {
    fn create_denom_msg(&self, _full_denom: String, subdenom: String) -> CosmosMsg<CustomMsgType> {
        CosmosMsg::Custom(CustomExecuteMsg::Token(
            crate::custom_execute_msg::TokenExecuteMsg::CreateDenom {
                subdenom,
            },
        ))
    }

    fn create_mint_msgs(
        &self,
        full_denom: String,
        amount: Uint128,
        recipient: Addr,
    ) -> Vec<CosmosMsg<CustomMsgType>> {
        vec![
            CosmosMsg::Custom(CustomExecuteMsg::Token(
                crate::custom_execute_msg::TokenExecuteMsg::MintTokens {
                    denom: full_denom,
                    amount,
                    mint_to_address: recipient.to_string(),
                },
            )),
            // CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
            //     to_address: recipient.to_string(),
            //     amount: coins(amount.u128(), full_denom),
            // }),
        ]
    }

    fn create_burn_msg(&self, full_denom: String, amount: Uint128) -> CosmosMsg<CustomMsgType> {
        CosmosMsg::Custom(CustomExecuteMsg::Token(
            crate::custom_execute_msg::TokenExecuteMsg::BurnTokens {
                denom: full_denom,
                amount,
                burn_from_address: self.contract.to_string(),
            },
        ))
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
            StageType::Eris {
                addr,
            } => Hub(addr).bond_msg(denom, amount, None),
            StageType::Dex {
                addr,
            } => WhiteWhalePair(addr).swap_msg(denom, amount, belief_price, Some(max_spread)),
            StageType::Manta {
                addr,
                msg,
            } => match denom {
                astroport::asset::AssetInfo::Token {
                    ..
                } => Err(StdError::generic_err("not supported by mnta")),
                astroport::asset::AssetInfo::NativeToken {
                    denom,
                } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: addr.to_string(),
                    funds: coins(amount.u128(), denom),
                    msg: to_json_binary(&msg)?,
                })),
            },
        }
    }

    // fn create_multi_swap_router_msgs(
    //     &self,
    //     router_type: MultiSwapRouterType,
    //     assets: Vec<CoinType>,
    // ) -> StdResult<Vec<CosmosMsg<CustomMsgType>>> {
    //     let funds: Vec<Coin> =
    //         assets.iter().map(|asset| asset.to_coin()).collect::<StdResult<_>>()?;

    //     match router_type {
    //         MultiSwapRouterType::Manta {
    //             addr,
    //             msg,
    //         } => Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
    //             contract_addr: addr.to_string(),
    //             funds,
    //             msg: to_json_binary(&msg)?,
    //         })]),
    //     }
    // }

    // fn equals_asset_info(
    //     &self,
    //     denom: &DenomType,
    //     asset_info: &astroport::asset::AssetInfo,
    // ) -> bool {
    //     denom == asset_info
    // }

    // fn get_coin(&self, denom: DenomType, amount: Uint128) -> CoinType {
    //     denom.with_balance(amount)
    // }
}
