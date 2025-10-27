use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    sysvars::rent::Rent,
    ProgramResult,
};

use pinocchio_associated_token_account::instructions::CreateIdempotent;
use pinocchio_system::instructions::CreateAccount;

use crate::states::{
    utils::{load_ix_data, DataLen},
    Fundraiser,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Initialize {
    pub amount: u64,
    pub duration: [u8; 1],
    pub bump: [u8; 1],
}

impl DataLen for Initialize {
    const LEN: usize = core::mem::size_of::<Initialize>();
}

pub fn initialize(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, mint_to_raise, fundraiser, vault, sysvar_rent_acc, token_program, system_program, _associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !fundraiser.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if mint_to_raise.owner() != &pinocchio_token::ID {
        return Err(ProgramError::InvalidAccountOwner);
    };

    // extracting mint decimals from mint account data
    let mint_data = mint_to_raise.try_borrow_data()?;
    let mint_decimals = mint_data[44]; // Mint decimals are at offset 44

    let rent = Rent::from_account_info(sysvar_rent_acc)?;

    let ix_data = unsafe { load_ix_data::<Initialize>(data)? };

    let pda_bump_bytes = ix_data.bump;

    Fundraiser::validate_pda(ix_data.bump, fundraiser.key())?;

    // signer seeds
    let signer_seeds = [
        Seed::from(Fundraiser::SEED.as_bytes()),
        Seed::from(maker.key()),
        Seed::from(&pda_bump_bytes[..]),
    ];
    let signers = [Signer::from(&signer_seeds[..])];

    CreateAccount {
        from: maker,
        to: fundraiser,
        space: Fundraiser::LEN as u64,
        owner: &crate::ID,
        lamports: rent.minimum_balance(Fundraiser::LEN),
    }
    .invoke_signed(&signers)?;

    // creating vault ATA if it doesn't exist
    CreateIdempotent {
        //idempotent
        funding_account: maker,
        account: vault,
        wallet: fundraiser,
        mint: mint_to_raise,
        system_program: system_program,
        token_program: token_program,
    }
    .invoke_signed(&signers)?;

    Fundraiser::initialize(
        fundraiser,
        ix_data,
        maker.key(),
        mint_to_raise.key(),
        mint_decimals,
    )?;

    Ok(())
}
