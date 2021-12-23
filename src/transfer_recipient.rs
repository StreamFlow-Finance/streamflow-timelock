use borsh::BorshSerialize;
use solana_program::{
    borsh as solana_borsh, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_associated_token_account::get_associated_token_address;


use crate::{
    error::SfError,
    state::{InstructionAccounts, TokenStreamData},
    stream_safety::{initialized_account_sanity_check, metadata_sanity_check},
    utils::Invoker
};

pub fn transfer_recipient(
    program_id: &Pubkey,
    acc: InstructionAccounts,
    recipient: Pubkey,
    recipient_tokens: Pubkey,
) -> ProgramResult {
    msg!("Transferring stream recipient");

    if !acc.authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature)
    }

    // Sanity checks
    initialized_account_sanity_check(program_id, acc.clone())?;
    metadata_sanity_check(acc.clone())?;

    let mut data = acc.metadata.try_borrow_mut_data()?;
    let mut metadata: TokenStreamData = match solana_borsh::try_from_slice_unchecked(&data) {
        Ok(v) => v,
        Err(_) => return Err(SfError::InvalidMetadata.into()),
    };

    let cancel_authority = Invoker::new(
        &acc.authorized_wallet.key,
        &metadata.sender,
        &metadata.recipient,
    );
    if !cancel_authority.can_transfer(&metadata.ix) {
        return Err(SfError::TransferNotAllowed.into());
    }

    // Check if the passed arg is an associated token address
    let new_recipient_tokens = get_associated_token_address(&recipient, acc.mint.key);
    if new_recipient_tokens != recipient_tokens {
        return Err(ProgramError::InvalidAccountData)
    }

    //todo: should we withdraw what's available before transferring recipient? I'd say YES.
    //(that means we also need streamflow_treasury_tokens, partner_tokens, escrow_tokens in account list

    metadata.recipient = recipient;
    metadata.recipient_tokens = recipient_tokens;

    let bytes = metadata.try_to_vec()?;
    data[0..bytes.len()].clone_from_slice(&bytes);

    Ok(())
}
