#[cfg(feature = "X-kujira-X")]
pub mod types {
    use cosmwasm_std::DepsMut;
    use cosmwasm_std::Env;
    use cosmwasm_std::StdResult;
    use cosmwasm_std::Uint128;
    use std::collections::HashMap;

    use eris_chain_shared::chain_trait::ChainInterface;
    use eris_kujira::kujira_chain::KujiraChain;

    pub use eris_kujira::kujira_types::CustomMsgType;
    pub use eris_kujira::kujira_types::DenomType;
    pub use eris_kujira::kujira_types::HubChainConfig;
    pub use eris_kujira::kujira_types::HubChainConfigInput;
    pub use eris_kujira::kujira_types::StageType;
    pub use eris_kujira::kujira_types::WithdrawType;

    #[inline(always)]
    pub fn chain(
        _env: &Env,
    ) -> impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig>
    {
        KujiraChain {}
    }

    #[inline(always)]
    pub fn test_chain_config() -> HubChainConfigInput {
        HubChainConfigInput {}
    }

    /// queries all balances and converts it to a hashmap
    pub fn get_balances_hashmap<F>(
        deps: &DepsMut,
        env: Env,
        _get_denoms: F,
    ) -> StdResult<HashMap<String, Uint128>>
    where
        F: FnOnce() -> Vec<DenomType>,
    {
        let balances = deps.querier.query_all_balances(env.contract.address)?;
        let balances: HashMap<_, _> =
            balances.into_iter().map(|item| (item.denom.clone(), item.amount)).collect();
        Ok(balances)
    }
}

#[cfg(feature = "X-whitewhale-X")]
pub mod types {
    use cosmwasm_std::DepsMut;
    use cosmwasm_std::Env;
    use cosmwasm_std::StdError;
    use cosmwasm_std::StdResult;
    use cosmwasm_std::Uint128;
    use std::collections::HashMap;

    use eris_chain_shared::chain_trait::ChainInterface;
    use eris_whitewhale::whitewhale_chain::WhiteWhaleChain;
    use eris_whitewhale::whitewhale_types::get_asset;

    use eris_whitewhale::whitewhale_types::CoinType;
    pub use eris_whitewhale::whitewhale_types::CustomMsgType;
    pub use eris_whitewhale::whitewhale_types::DenomType;
    pub use eris_whitewhale::whitewhale_types::HubChainConfig;
    pub use eris_whitewhale::whitewhale_types::HubChainConfigInput;
    pub use eris_whitewhale::whitewhale_types::StageType;
    pub use eris_whitewhale::whitewhale_types::WithdrawType;

    #[inline(always)]
    pub fn chain(
        env: &Env,
    ) -> impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig>
    {
        WhiteWhaleChain {
            contract: env.contract.address.clone(),
        }
    }

    #[inline(always)]
    pub fn test_chain_config() -> HubChainConfigInput {
        HubChainConfigInput {}
    }

    /// queries all balances and converts it to a hashmap
    pub fn get_balances_hashmap<F>(
        deps: &DepsMut,
        env: Env,
        get_denoms: F,
    ) -> StdResult<HashMap<String, Uint128>>
    where
        F: FnOnce() -> Vec<DenomType>,
    {
        let balances: HashMap<_, _> = get_denoms()
            .into_iter()
            .map(|denom| {
                let balance = denom
                    .query_balance(&deps.querier, env.contract.address.clone())
                    .map_err(|e| StdError::generic_err(e.to_string()))?;

                Ok(get_asset(denom, balance))
            })
            .collect::<StdResult<Vec<CoinType>>>()?
            .into_iter()
            .map(|element| (element.info.to_string(), element.amount))
            .collect();

        Ok(balances)
    }
}
