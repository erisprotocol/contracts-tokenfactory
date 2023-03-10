use cosmwasm_std::{Addr, Api, CosmosMsg, Decimal, StdResult, Uint128};

pub trait ChainInterface<TCustom, TDenomType, TWithdrawType, TStageType, THubChainConfig> {
    fn get_token_denom(&self, contract_addr: impl Into<String>, sub_denom: String) -> String {
        format!("factory/{0}/{1}", contract_addr.into(), sub_denom)
    }

    fn create_denom_msg(&self, full_denom: String, sub_denom: String) -> CosmosMsg<TCustom>;
    // this can sometimes be multiple messages to mint + transfer
    fn create_mint_msgs(
        &self,
        full_denom: String,
        amount: Uint128,
        recipient: Addr,
    ) -> Vec<CosmosMsg<TCustom>>;

    fn create_burn_msg(&self, full_denom: String, amount: Uint128) -> CosmosMsg<TCustom>;

    fn create_withdraw_msg<F>(
        &self,
        get_chain_config: F,
        withdraw_type: TWithdrawType,
        denom: TDenomType,
        amount: Uint128,
    ) -> StdResult<Option<CosmosMsg<TCustom>>>
    where
        F: FnOnce() -> StdResult<THubChainConfig>;

    fn create_single_stage_swap_msgs<F>(
        &self,
        get_chain_config: F,
        stage_type: TStageType,
        denom: TDenomType,
        amount: Uint128,
        belief_price: Option<Decimal>,
        max_spread: Decimal,
    ) -> StdResult<CosmosMsg<TCustom>>
    where
        F: FnOnce() -> StdResult<THubChainConfig>;
}

pub trait Validateable<T> {
    fn validate(&self, api: &dyn Api) -> StdResult<T>;
}
