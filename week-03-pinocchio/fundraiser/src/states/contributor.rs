use crate::{errors::FundraiserError, states::DataLen};
use bytemuck::{Pod, Zeroable};
use pinocchio::{
    program_error::ProgramError,
    pubkey::{self, Pubkey},
};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Contributor {
    pub amount: u64,
}

impl DataLen for Contributor {
    const LEN: usize = core::mem::size_of::<Contributor>();
}

impl Contributor {
    pub const SEED: &'static str = "contributor";

    pub fn validate_pda(
        bump: [u8; 1],
        pda: &Pubkey,
        fundraiser_pubkey: &Pubkey,
        contributor_pubkey: &Pubkey,
    ) -> Result<(), ProgramError> {
        let seed_with_bump = &[
            Self::SEED.as_bytes(),
            fundraiser_pubkey,
            contributor_pubkey,
            &bump,
        ];
        let derived = pubkey::create_program_address(seed_with_bump, &crate::ID)?;
        if derived != *pda {
            return Err(FundraiserError::PdaMismatch.into());
        }
        Ok(())
    }
}
