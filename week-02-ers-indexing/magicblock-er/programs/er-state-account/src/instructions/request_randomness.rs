use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction};
use ephemeral_vrf_sdk::{
    anchor::vrf,
    instructions::{create_request_randomness_ix, RequestRandomnessParams},
    types::SerializableAccountMeta,
    consts::DEFAULT_QUEUE
};

use crate::instruction;
use crate::state::UserAccount;

#[vrf]
#[derive(Accounts)]
pub struct RequestRandomness<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    /// CHECK: VRF program identity PDA
    pub vrf_identity: UncheckedAccount<'info>,

    /// CHECK: Oracle queue for randomness
    #[account(
        mut, 
        address = DEFAULT_QUEUE
    )]
    pub oracle_queue: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: Slot hashes sysvar
    pub slot_hashes: UncheckedAccount<'info>,
}

impl<'info> RequestRandomness<'info> {
    pub fn request_randomness(&mut self, caller_seed: [u8; 32]) -> Result<()> {
        // let callback_discriminator =
        //     anchor_lang::solana_program::hash::hash(b"global:consume_randomness").to_bytes()[..8]
        //         .to_vec();

        let callback_discriminator = instruction::Update::DISCRIMINATOR.to_vec();

        let callback_accounts_metas = vec![
            // SerializableAccountMeta {
            //     pubkey: self.user.key(),
            //     is_signer: true,
            //     is_writable: true,
            // },
            SerializableAccountMeta {
                pubkey: self.user_account.key(),
                is_signer: false,
                is_writable: true,
            },
        ];

        let params = RequestRandomnessParams {
            payer: self.user.key(),
            oracle_queue: self.oracle_queue.key(),
            callback_program_id: crate::ID,
            callback_discriminator,
            accounts_metas: Some(callback_accounts_metas),
            caller_seed,
            callback_args: None,
        };

        let ix: Instruction = create_request_randomness_ix(params);

        self.invoke_signed_vrf(&self.user.to_account_info(), &ix)?;

        Ok(())
    }
}
