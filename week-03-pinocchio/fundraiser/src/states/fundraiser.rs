use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};

use crate::{
    constants::MIN_AMOUNT_TO_RAISE,
    errors::FundraiserError,
    instructions::Initialize,
    states::{load_acc_mut_unchecked, DataLen},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fundraiser {
    pub maker: Pubkey,
    pub mint_to_raise: Pubkey,
    pub amount_to_raise: u64,
    pub current_amount: u64,
    pub time_started: i64,
    pub duration: [u8; 1],
    pub bump: [u8; 1],
}

impl DataLen for Fundraiser {
    const LEN: usize = core::mem::size_of::<Fundraiser>();
}

impl Fundraiser {
    pub const SEED: &'static str = "fundraiser";

    pub fn validate_pda(bump: [u8; 1], pda: &Pubkey) -> Result<(), ProgramError> {
        let seed_with_bump = &[Self::SEED.as_bytes(), &bump];
        let derived = pubkey::create_program_address(seed_with_bump, &crate::ID)?;
        if derived != *pda {
            return Err(FundraiserError::PdaMismatch.into());
        }
        Ok(())
    }

    pub fn initialize(
        my_stata_acc: &AccountInfo,
        ix_data: &Initialize,
        maker: &Pubkey,
        mint_to_raise: &Pubkey,
        mint_decimals: u8,
    ) -> ProgramResult {
        let fundraiser = unsafe {
            load_acc_mut_unchecked::<Fundraiser>(my_stata_acc.borrow_mut_data_unchecked())
        }?;

        if ix_data.amount < MIN_AMOUNT_TO_RAISE.pow(mint_decimals as u32) {
            return Err(FundraiserError::InvalidAmount.into());
        }

        fundraiser.maker = *maker;
        fundraiser.mint_to_raise = *mint_to_raise;
        fundraiser.amount_to_raise = ix_data.amount;
        fundraiser.duration = ix_data.duration;
        fundraiser.bump = ix_data.bump;
        fundraiser.current_amount = 0;
        fundraiser.time_started = Clock::get()?.unix_timestamp;

        Ok(())
    }
}
