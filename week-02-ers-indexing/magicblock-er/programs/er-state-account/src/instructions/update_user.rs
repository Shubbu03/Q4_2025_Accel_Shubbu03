use crate::state::UserAccount;
use anchor_lang::prelude::*;
use ephemeral_vrf_sdk::{consts::VRF_PROGRAM_IDENTITY, rnd::random_u64};

#[derive(Accounts)]
pub struct UpdateUser<'info> {
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        address = VRF_PROGRAM_IDENTITY
    )]
    pub vrf_program_identity: Signer<'info>,
}

impl<'info> UpdateUser<'info> {
    pub fn update(&mut self, randomness: [u8; 32]) -> Result<()> {
        // getting the generated random value
        let random_value = random_u64(&randomness);

        // updating user data with the random value
        self.user_account.data = random_value;

        msg!("Randomness consumed: {}", random_value);

        Ok(())
    }
}
