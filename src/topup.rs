use solana_program::{
    account_info::AccountInfo,
    borsh as solana_borsh,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};
use spl_token::amount_to_ui_amount;

use crate::{
    error::SfError,
    state::{find_escrow_account, save_account_info, Contract},
    utils::{calculate_fee_from_amount, unpack_mint_account, unpack_token_account},
};

#[derive(Clone, Debug)]
pub struct TopupAccounts<'a> {
    pub sender: AccountInfo<'a>,
    pub sender_tokens: AccountInfo<'a>,
    pub metadata: AccountInfo<'a>,
    pub escrow_tokens: AccountInfo<'a>,
    pub streamflow_treasury: AccountInfo<'a>,
    pub streamflow_treasury_tokens: AccountInfo<'a>,
    pub partner: AccountInfo<'a>,
    pub partner_tokens: AccountInfo<'a>,
    pub mint: AccountInfo<'a>,
    pub token_program: AccountInfo<'a>,
}

pub fn topup(pid: &Pubkey, acc: TopupAccounts, amount: u64) -> ProgramResult {
    msg!("Topping up escrow account");

    // Sanity checks
    //account_sanity_check

    if amount == 0 {
        return Err(SfError::AmountIsZero.into())
    }

    let mut data = acc.metadata.try_borrow_mut_data()?;
    let mut metadata: Contract = match solana_borsh::try_from_slice_unchecked(&data) {
        Ok(v) => v,
        Err(_) => return Err(SfError::InvalidMetadata.into()),
    };

    if !metadata.ix.can_topup {
        return Err(SfError::InvalidMetadata.into())
    }

    // Taking the protocol version from the metadata, we check that the token
    // escrow account is correct:
    if &find_escrow_account(metadata.version, acc.metadata.key.as_ref(), pid).0 !=
        acc.escrow_tokens.key
    {
        return Err(ProgramError::InvalidAccountData)
    }

    let escrow_tokens = unpack_token_account(&acc.escrow_tokens)?;
    metadata.sync_balance(escrow_tokens.amount);

    //metadata_sanity_check(acc.clone())?;

    let now = Clock::get()?.unix_timestamp as u64;
    if metadata.end_time < now {
        return Err(SfError::StreamClosed.into())
    }

    msg!("Transferring funds into escrow account");
    invoke(
        &spl_token::instruction::transfer(
            acc.token_program.key,
            acc.sender_tokens.key,
            acc.escrow_tokens.key,
            acc.sender.key,
            &[],
            amount,
        )?,
        &[
            acc.sender_tokens.clone(),
            acc.escrow_tokens.clone(),
            acc.sender.clone(),
            acc.token_program.clone(),
        ],
    )?;

    metadata.deposit(amount);
    save_account_info(&metadata, data)?;

    let mint_info = unpack_mint_account(&acc.mint)?;

    msg!(
        "Successfully topped up {} to token stream {} on behalf of {}",
        amount_to_ui_amount(amount, mint_info.decimals),
        acc.escrow_tokens.key,
        acc.sender.key,
    );

    Ok(())
}
