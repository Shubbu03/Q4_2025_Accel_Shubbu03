use bytemuck;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    ProgramResult,
};
use pinocchio_associated_token_account::instructions::CreateIdempotent;
use pinocchio_token::instructions::Transfer;

use crate::{
    errors::FundraiserError,
    states::{load_acc_mut, Fundraiser},
};

pub fn check_contributions(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [maker, mint_to_raise, fundraiser, vault, maker_ata, _sysvar_rent_acc, token_program, system_program, _associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    };

    if fundraiser.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    // loading fundraiser data
    let fundraiser_data =
        unsafe { load_acc_mut::<Fundraiser>(fundraiser.borrow_mut_data_unchecked())? };

    if mint_to_raise.key() != &fundraiser_data.mint_to_raise {
        return Err(FundraiserError::MintMismatch.into());
    }

    // let mint_data = mint_to_raise.try_borrow_data()?;
    // let mint_decimals = mint_data[44];

    // let rent = Rent::from_account_info(sysvar_rent_acc)?;

    // creating maker ATA if it doesn't exist
    CreateIdempotent {
        funding_account: maker,
        account: maker_ata,
        wallet: maker,
        mint: mint_to_raise,
        system_program: system_program,
        token_program: token_program,
    }
    .invoke()?;

    // loading vault token account data to get the amount
    let vault_data = vault.try_borrow_data()?;

    // token account amount is at offset 64 (after discriminator + owner + mint + amount)
    let amount_bytes = &vault_data[64..72];
    let vault_amount = *bytemuck::try_from_bytes::<u64>(amount_bytes)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // validation logic
    if vault_amount < fundraiser_data.amount_to_raise {
        return Err(FundraiserError::TargetNotMet.into());
    }

    // transfer funds from vault to maker
    let fundraiser_bump = fundraiser_data.bump;
    let signer_seeds = [
        Seed::from(Fundraiser::SEED.as_bytes()),
        Seed::from(maker.key()),
        Seed::from(&fundraiser_bump[..]),
    ];

    let signers = &[Signer::from(&signer_seeds[..])];

    Transfer {
        from: vault,
        to: maker_ata,
        authority: fundraiser,
        amount: vault_amount,
    }
    .invoke_signed(signers)?;
    Ok(())
}
