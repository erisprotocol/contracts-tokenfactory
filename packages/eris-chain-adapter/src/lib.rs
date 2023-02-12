#[cfg(feature = "kujira")]
pub mod types {
    use eris_chain_shared::chain_trait::ChainInterface;
    use eris_chain_shared::test_trait::TestInterface;
    use eris_kujira::kujira_chain::KujiraChain;

    pub use eris_kujira::kujira_types::CustomMsgType;
    pub use eris_kujira::kujira_types::DenomType;
    pub use eris_kujira::kujira_types::HubChainConfig;
    pub use eris_kujira::kujira_types::HubChainConfigInput;
    pub use eris_kujira::kujira_types::StageType;
    pub use eris_kujira::kujira_types::WithdrawType;

    #[inline(always)]
    pub fn chain(
    ) -> impl ChainInterface<CustomMsgType, DenomType, WithdrawType, StageType, HubChainConfig>
    {
        KujiraChain {}
    }

    #[inline(always)]
    pub fn main_denom() -> &'static str {
        "ukuji"
    }

    pub fn test_config() -> impl TestInterface<
        CustomMsgType,
        DenomType,
        WithdrawType,
        StageType,
        HubChainConfig,
        HubChainConfigInput,
    > {
        use eris_kujira::kujira_test::KujiraTest;

        KujiraTest {}
    }
}

#[cfg(feature = "whitewhale")]
pub mod types {
    pub use eris_migaloo::types::CustomMsgType;
    pub use eris_migaloo::types::DenomType;
    pub use eris_migaloo::types::StageType;
    pub use eris_migaloo::types::WithdrawType;
}
