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

    pub const CHAIN_TYPE: &str = "kujira";

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
        deps: &DepsMut<CustomQueryType>,
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

    pub use eris_whitewhale::whitewhale_types::AssetExt;
    pub use eris_whitewhale::whitewhale_types::AssetInfoExt;

    use eris_whitewhale::whitewhale_types::CoinType;
    pub use eris_whitewhale::whitewhale_types::CustomMsgType;
    pub use eris_whitewhale::whitewhale_types::CustomQueryType;
    pub use eris_whitewhale::whitewhale_types::DenomType;
    pub use eris_whitewhale::whitewhale_types::HubChainConfig;
    pub use eris_whitewhale::whitewhale_types::HubChainConfigInput;
    pub use eris_whitewhale::whitewhale_types::StageType;
    pub use eris_whitewhale::whitewhale_types::WithdrawType;

    pub const CHAIN_TYPE: &str = "migaloo";

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
        deps: &DepsMut<CustomQueryType>,
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

#[cfg(feature = "X-injective-X")]
pub mod types {
    use cosmwasm_std::DepsMut;
    use cosmwasm_std::Env;
    use cosmwasm_std::StdError;
    use cosmwasm_std::StdResult;
    use cosmwasm_std::Uint128;
    use std::collections::HashMap;

    use eris_chain_shared::chain_trait::ChainInterface;
    use eris_injective::injective_chain::InjectiveChain;
    use eris_injective::injective_types::get_asset;

    use eris_injective::injective_types::CoinType;
    pub use eris_injective::injective_types::CustomMsgType;
    pub use eris_injective::injective_types::DenomType;
    pub use eris_injective::injective_types::HubChainConfig;
    pub use eris_injective::injective_types::HubChainConfigInput;
    pub use eris_injective::injective_types::StageType;
    pub use eris_injective::injective_types::WithdrawType;

    pub const CHAIN_TYPE: &str = "migaloo";

    #[inline(always)]
    pub fn chain(
        env: &Env,
    ) -> impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig>
    {
        InjectiveChain {
            contract: env.contract.address.clone(),
        }
    }

    #[inline(always)]
    pub fn test_chain_config() -> HubChainConfigInput {
        HubChainConfigInput {}
    }

    /// queries all balances and converts it to a hashmap
    pub fn get_balances_hashmap<F>(
        deps: &DepsMut<CustomQueryType>,
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

#[cfg(feature = "X-osmosis-X")]
pub mod types {
    use cosmwasm_std::DepsMut;
    use cosmwasm_std::Env;
    use cosmwasm_std::StdResult;
    use cosmwasm_std::Uint128;
    use std::collections::HashMap;

    use eris_chain_shared::chain_trait::ChainInterface;
    use eris_osmosis::chain::OsmosisChain;

    pub use eris_osmosis::types::CustomMsgType;
    pub use eris_osmosis::types::CustomQueryType;
    pub use eris_osmosis::types::DenomType;
    pub use eris_osmosis::types::HubChainConfig;
    pub use eris_osmosis::types::HubChainConfigInput;
    pub use eris_osmosis::types::StageType;
    pub use eris_osmosis::types::WithdrawType;

    pub const CHAIN_TYPE: &str = "osmosis";

    #[inline(always)]
    pub fn chain(
        env: &Env,
    ) -> impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig>
    {
        OsmosisChain {
            contract: env.contract.address.clone(),
        }
    }

    #[inline(always)]
    pub fn test_chain_config() -> HubChainConfigInput {
        HubChainConfigInput {}
    }

    /// queries all balances and converts it to a hashmap
    pub fn get_balances_hashmap<F>(
        deps: &DepsMut<CustomQueryType>,
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

#[cfg(feature = "X-terra-X")]
pub mod types {
    use cosmwasm_std::DepsMut;
    use cosmwasm_std::Env;
    use cosmwasm_std::StdError;
    use cosmwasm_std::StdResult;
    use cosmwasm_std::Uint128;
    use std::collections::HashMap;

    use eris_chain_shared::chain_trait::ChainInterface;

    use eris_terra::chain::Chain;
    pub use eris_terra::types::get_asset;
    pub use eris_terra::types::AssetInfoExt;
    pub use eris_terra::types::CoinType;
    pub use eris_terra::types::CustomMsgType;
    pub use eris_terra::types::CustomQueryType;
    pub use eris_terra::types::DenomType;
    pub use eris_terra::types::HubChainConfig;
    pub use eris_terra::types::HubChainConfigInput;
    pub use eris_terra::types::MantaMsg;
    pub use eris_terra::types::MantaSwap;
    pub use eris_terra::types::MultiSwapRouterType;
    pub use eris_terra::types::StageType;
    pub use eris_terra::types::WithdrawType;

    pub const CHAIN_TYPE: &str = "terra";

    #[inline(always)]
    pub fn chain(
        env: &Env,
    ) -> impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig>
    {
        Chain {
            contract: env.contract.address.clone(),
        }
    }

    /// queries all balances and converts it to a hashmap
    pub fn get_balances_hashmap<F>(
        deps: &DepsMut<CustomQueryType>,
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
                    .query_pool(&deps.querier.into_empty(), env.contract.address.clone())
                    .map_err(|e| StdError::generic_err(e.to_string()))?;

                Ok(get_asset(denom, balance))
            })
            .collect::<StdResult<Vec<CoinType>>>()?
            .into_iter()
            .map(|element| (element.info.to_string(), element.amount))
            .collect();

        Ok(balances)
    }

    #[inline(always)]
    pub fn test_chain_config() -> HubChainConfigInput {
        HubChainConfigInput {}
    }
}

#[cfg(feature = "X-sei-X")]
pub mod types {
    use cosmwasm_std::DepsMut;
    use cosmwasm_std::Env;
    use cosmwasm_std::StdResult;
    use cosmwasm_std::Uint128;
    use std::collections::HashMap;

    use eris_chain_shared::chain_trait::ChainInterface;
    use eris_sei::chain::Chain;

    pub use eris_sei::types::CustomMsgType;
    pub use eris_sei::types::CustomQueryType;
    pub use eris_sei::types::DenomType;
    pub use eris_sei::types::HubChainConfig;
    pub use eris_sei::types::HubChainConfigInput;
    pub use eris_sei::types::StageType;
    pub use eris_sei::types::WithdrawType;

    pub const CHAIN_TYPE: &str = "neutron";

    #[inline(always)]
    pub fn chain(
        env: &Env,
    ) -> impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig>
    {
        Chain {
            contract: env.contract.address.clone(),
        }
    }

    #[inline(always)]
    pub fn test_chain_config() -> HubChainConfigInput {
        HubChainConfigInput {}
    }

    /// queries all balances and converts it to a hashmap
    pub fn get_balances_hashmap<F>(
        deps: &DepsMut<CustomQueryType>,
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
