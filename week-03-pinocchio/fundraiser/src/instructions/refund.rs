use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};
use pinocchio_token::instructions::Transfer;

use crate::{
    constants::SECONDS_TO_DAYS,
    errors::FundraiserError,
    states::{load_acc_mut_unchecked, Contributor, Fundraiser},
};

pub fn refund_to_contributors(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [contributor, maker, mint_to_raise, fundraiser, contributor_account, contributor_ata, vault, _token_program, _system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // validating that fundraiser account exists and is initialized
    if fundraiser.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    // loading fundraiser data
    let fundraiser_data =
        unsafe { load_acc_mut_unchecked::<Fundraiser>(fundraiser.borrow_mut_data_unchecked()) }?;

    // validating that mint_to_raise matches fundraiser's mint
    if mint_to_raise.key() != &fundraiser_data.mint_to_raise {
        return Err(FundraiserError::MintMismatch.into());
    }

    // validating that contributor account exists and is initialized
    if contributor_account.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    // loading contributor account data
    let contributor_account_data = unsafe {
        load_acc_mut_unchecked::<Contributor>(contributor_account.borrow_mut_data_unchecked())
    }?;

    // fundraising duration has been reached
    let current_time = Clock::get()?.unix_timestamp;
    let days_elapsed = ((current_time - fundraiser_data.time_started) / SECONDS_TO_DAYS) as u8;

    if fundraiser_data.duration[0] < days_elapsed {
        return Err(FundraiserError::FundraiserEnded.into());
    }

    // loading token account data to get the amount
    let vault_data = vault.try_borrow_data()?;
    let vault_amount = u64::from_le_bytes([
        vault_data[64],
        vault_data[65],
        vault_data[66],
        vault_data[67],
        vault_data[68],
        vault_data[69],
        vault_data[70],
        vault_data[71],
    ]);

    // checking if the target has been met (if so, no refunds allowed)
    if vault_amount >= fundraiser_data.amount_to_raise {
        return Err(FundraiserError::TargetNotMet.into());
    }

    // transfer funds from vault back to contributor
    let signer_seeds = [
        Seed::from(Fundraiser::SEED.as_bytes()),
        Seed::from(maker.key()),
        Seed::from(&fundraiser_data.bump),
    ];

    let signers = [Signer::from(&signer_seeds[..])];

    Transfer {
        from: vault,
        to: contributor_ata,
        authority: fundraiser,
        amount: contributor_account_data.amount,
    }
    .invoke_signed(&signers)?;

    // updating the fundraiser state by reducing the amount contributed
    fundraiser_data.current_amount -= contributor_account_data.amount;

    // closing the contributor account and return rent to contributor

    Ok(())
}
