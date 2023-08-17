use crate::types::{
    CustomMsgType, DenomType, HubChainConfig, HubChainConfigInput, StageType, WithdrawType,
};
use eris_chain_shared::test_trait::TestInterface;

pub struct SeiTest {}

impl
    TestInterface<
        CustomMsgType,
        DenomType,
        WithdrawType,
        StageType,
        HubChainConfig,
        HubChainConfigInput,
    > for SeiTest
{
    fn default_chain_config(&self) -> HubChainConfigInput {
        HubChainConfigInput {}
    }
}
