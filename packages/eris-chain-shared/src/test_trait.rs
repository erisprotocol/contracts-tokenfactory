pub trait TestInterface<
    TCustom,
    TDenomType,
    TWithdrawType,
    TStageType,
    THubChainConfig,
    THubChainConfigInput,
>
{
    fn default_chain_config(&self) -> THubChainConfigInput;
}
