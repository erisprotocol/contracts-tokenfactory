use crate::kujira_types::{
    CustomMsgType, DenomType, HubChainConfig, HubChainConfigInput, StageType, WithdrawType,
};
use eris_chain_shared::test_trait::TestInterface;

pub struct KujiraTest {}

impl
    TestInterface<
        CustomMsgType,
        DenomType,
        WithdrawType,
        StageType,
        HubChainConfig,
        HubChainConfigInput,
    > for KujiraTest
{
    fn default_chain_config(&self) -> HubChainConfigInput {
        HubChainConfigInput {}
    }
}
