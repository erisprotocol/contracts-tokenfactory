use cosmwasm_std::{OverflowError, Response, StdError};
use eris_chain_adapter::types::CustomMsgType;
use thiserror::Error;

pub type ContractResult = Result<Response<CustomMsgType>, ContractError>;

/// This enum describes hub contract errors
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Unauthorized: sender is not owner")]
    Unauthorized {},

    #[error("Unauthorized: sender is not new owner")]
    UnauthorizedSenderNotNewOwner {},

    #[error("Unauthorized: sender is not vote operator")]
    UnauthorizedSenderNotVoteOperator {},

    #[error("Unauthorized: sender is not operator")]
    UnauthorizedSenderNotOperator {},

    #[error("Expecting only single coin")]
    ExpectingSingleCoin {},

    #[error("Expecting stake token, received {0}")]
    ExpectingAllianceStakeToken(String),

    #[error("Protocol_reward_fee greater than max")]
    ProtocolRewardFeeTooHigh {},

    #[error("{0} can't be zero")]
    CantBeZero(String),

    #[error("Batch can only be submitted for unbonding after {0}")]
    SubmitBatchAfter(u64),

    #[error("Callbacks can only be invoked by the contract itself")]
    CallbackOnlyCalledByContract {},

    #[error("Invalid reply id: {0}")]
    InvalidReplyId(u64),

    #[error("Donations are disabled")]
    DonationsDisabled {},

    #[error("No {0} available to be bonded")]
    NoTokensAvailable(String),

    #[error("validator {0} is already whitelisted")]
    ValidatorAlreadyWhitelisted(String),

    #[error("validator {0} is not whitelisted")]
    ValidatorNotWhitelisted(String),

    #[error("Swap from {0} is not allowed")]
    SwapFromNotAllowed(String),

    #[error("Setting a belief Price is not allowed")]
    BeliefPriceNotAllowed {},

    #[error("Can only set fee payment to the first stage")]
    FeePaymentNotAllowed {},

    #[error("cannot find `instantiate` event")]
    CannotFindInstantiateEvent {},

    #[error("cannot find `_contract_address` attribute")]
    CannotFindContractAddress {},

    #[error("No vote operator set")]
    NoVoteOperatorSet {},

    #[error("Contract can't be migrated!")]
    MigrationError {},

    #[error("No claims provided.")]
    NoClaimsProvided {},

    #[error("No validators configured. There needs to be at least one validator available.")]
    NoValidatorsConfigured,

    #[error("Not supported: {0}")]
    NotSupported(String),

    #[error("State has changed, recreate the slashing to apply it. ({0})")]
    StateChanged(String),

    #[error("Submit Batch Failed: {0}")]
    SubmitBatchFailure(String),
}
