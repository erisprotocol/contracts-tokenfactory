#[cfg(feature = "X-kujira-X")]
pub mod types {
    use eris_chain_shared::chain_trait::ChainInterface;
    use eris_kujira::kujira_chain::KujiraChain;

    pub use eris_kujira::kujira_types::CustomMsgType;
    pub use eris_kujira::kujira_types::DenomType;
    pub use eris_kujira::kujira_types::HubChainConfig;
    pub use eris_kujira::kujira_types::HubChainConfigInput;
    pub use eris_kujira::kujira_types::StageType;
    pub use eris_kujira::kujira_types::WithdrawType;

    #[inline(always)]
    pub fn main_denom() -> &'static str {
        "ukuji"
    }

    #[inline(always)]
    pub fn chain(
    ) -> impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig>
    {
        KujiraChain {}
    }

    #[inline(always)]
    pub fn test_chain_config() -> HubChainConfigInput {
        HubChainConfigInput {
            fin_multi: "fin_multi".to_string(),
        }
    }
}

#[cfg(feature = "X-whitewhale-X")]
pub mod types {
    use eris_chain_shared::chain_trait::ChainInterface;
    use eris_whitewhale::whitewhale_chain::WhiteWhaleChain;
    pub use eris_whitewhale::whitewhale_types::CustomMsgType;
    pub use eris_whitewhale::whitewhale_types::DenomType;
    pub use eris_whitewhale::whitewhale_types::HubChainConfig;
    pub use eris_whitewhale::whitewhale_types::HubChainConfigInput;
    pub use eris_whitewhale::whitewhale_types::StageType;
    pub use eris_whitewhale::whitewhale_types::WithdrawType;

    #[inline(always)]
    pub fn main_denom() -> &'static str {
        "uwhale"
    }

    #[inline(always)]
    pub fn chain(
    ) -> impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig>
    {
        WhiteWhaleChain {}
    }

    #[inline(always)]
    pub fn test_chain_config() -> HubChainConfigInput {
        HubChainConfigInput {}
    }
}
