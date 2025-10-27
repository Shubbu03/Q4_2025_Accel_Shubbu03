use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Transfer;

use crate::{
    constants::{MAX_CONTRIBUTION_PERCENTAGE, PERCENTAGE_SCALER, SECONDS_TO_DAYS},
    errors::FundraiserError,
    states::{load_acc_mut, load_ix_data, Contributor, DataLen, Fundraiser},
};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct ContributeIxData {
    pub amount: u64,
    pub bump: [u8; 1],
}

impl DataLen for ContributeIxData {
    const LEN: usize = core::mem::size_of::<ContributeIxData>();
}

pub fn contribute(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [contributor, mint_to_raise, fundraiser, contributor_account, contributor_ata, vault, sysvar_rent_acc, token_program, _system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // fundraiser account should exists and is initialized
    if fundraiser.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    // loading fundraiser data
    let fundraiser_data =
        unsafe { load_acc_mut::<Fundraiser>(fundraiser.borrow_mut_data_unchecked())? };

    // validating that mint_to_raise matches fundraiser's mint
    if mint_to_raise.key() != &fundraiser_data.mint_to_raise {
        return Err(FundraiserError::MintMismatch.into());
    }

    let mint_data = mint_to_raise.try_borrow_data()?;
    let mint_decimals = mint_data[44];

    //load ix data
    let contributor_ix_data = load_ix_data::<ContributeIxData>(data)?;

    let rent = Rent::from_account_info(sysvar_rent_acc)?;

    let pda_bump = contributor_ix_data.bump;

    // validation logic
    let amount = contributor_ix_data.amount;

    // amount to contribute should meet the minimum amount required
    if amount <= 1_u64.pow(mint_decimals as u32) {
        return Err(FundraiserError::ContributionTooSmall.into());
    }

    // amount to contribute is less than the maximum allowed contribution
    let max_contribution =
        (fundraiser_data.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER;
    if amount > max_contribution {
        return Err(FundraiserError::ContributionTooBig.into());
    }

    // fundraising duration has been reached
    let current_time = Clock::get()?.unix_timestamp;
    let days_elapsed = ((current_time - fundraiser_data.time_started) / SECONDS_TO_DAYS) as u8;
    if fundraiser_data.duration[0] <= days_elapsed {
        return Err(FundraiserError::FundraiserEnded.into());
    }

    // contributor account exists, if not create it
    let contributor_account_data = if contributor_account.data_is_empty() {
        // Create contributor account
        let seed = [
            Seed::from(Contributor::SEED.as_bytes()),
            Seed::from(fundraiser.key()),
            Seed::from(contributor.key()),
            Seed::from(&pda_bump[..]),
        ];
        let signer = &[Signer::from(&seed[..])];

        CreateAccount {
            from: contributor,
            to: contributor_account,
            space: Contributor::LEN as u64,
            owner: &crate::ID,
            lamports: rent.minimum_balance(Contributor::LEN),
        }
        .invoke_signed(signer)?;

        // initialize contributor account
        let contributor_data = unsafe {
            load_acc_mut::<Contributor>(contributor_account.borrow_mut_data_unchecked())?
        };
        contributor_data.amount = 0;
        contributor_data
    } else {
        // Load existing contributor account
        unsafe { load_acc_mut::<Contributor>(contributor_account.borrow_mut_data_unchecked())? }
    };

    // validating contributor ATA PDA
    let (derived_ata, _bump_seed) = pinocchio::pubkey::find_program_address(
        &[
            contributor.key().as_ref(),
            token_program.key().as_ref(),
            mint_to_raise.key().as_ref(),
        ],
        &pinocchio_associated_token_account::id(),
    );
    if derived_ata != *contributor_ata.key() {
        return Err(ProgramError::InvalidAccountData);
    }
    // validating ATA account exists (check if data is non-empty)
    if contributor_ata.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    // current amount OR (current + new) amount exceeds the limit
    if contributor_account_data.amount > max_contribution
        || contributor_account_data.amount + amount > max_contribution
    {
        return Err(FundraiserError::MaximumContributionsReached.into());
    }

    // transfer tokens from contributor's ata to vault
    Transfer {
        from: contributor_ata,
        to: vault,
        authority: contributor,
        amount,
    }
    .invoke()?;

    // updating fundraiser current amount
    fundraiser_data.current_amount += amount;

    // updating contributor amount
    contributor_account_data.amount += amount;

    Ok(())
}
