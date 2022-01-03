use solana_program::{msg, program_error::ProgramError};
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum SfError {
    #[error("Accounts not writable!")]
    AccountsNotWritable,

    #[error("Invalid Metadata!")]
    InvalidMetadata,

    #[error("Invalid metadata account")]
    InvalidMetadataAccount,

    #[error("Provided accounts don't match the ones in contract.")]
    MetadataAccountMismatch,

    #[error("Invalid escrow account")]
    InvalidEscrowAccount,

    #[error("Provided account(s) is/are not valid associated token accounts.")]
    NotAssociated,

    #[error("Sender mint does not match accounts mint!")]
    MintMismatch,

    #[error("Recipient not transferable for account")]
    TransferNotAllowed,

    #[error("Stream closed")]
    StreamClosed,

    #[error("Invalid Streamflow Treasury accounts supplied")]
    InvalidTreasury,

    #[error("Given timestamps are invalid")]
    InvalidTimestamps,

    #[error("Deposited amount must be <= Total amount")]
    InvalidDeposit,

    #[error("Amount cannot be zero")]
    AmountIsZero,

    #[error("Amount requested is larger than available")]
    AmountMoreThanAvailable,
}

impl From<SfError> for ProgramError {
    fn from(e: SfError) -> Self {
        msg!(&e.to_string());
        ProgramError::Custom(e as u32)
    }
}
