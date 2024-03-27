use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

/// This enum describes contract errors
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("overflow: {0}")]
    Overflow(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("You can't vote with zero voting power")]
    ZeroVotingPower {},

    #[error("Invalid gauge key: {0}")]
    InvalidGaugeKey(String),

    #[error("Votes contain duplicated gauge keys")]
    DuplicatedGauges {},

    #[error("Contract can't be migrated!")]
    MigrationError {},

    #[error("Config error: {0}")]
    ConfigError(String),
}
