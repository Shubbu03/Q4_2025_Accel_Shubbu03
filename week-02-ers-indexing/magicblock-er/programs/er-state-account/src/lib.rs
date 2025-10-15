#![allow(unexpected_cfgs, deprecated)]

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;

mod instructions;
mod state;

use instructions::*;

declare_id!("CvJmPTXStp3EGL5SZJ9iwUCJ6k99vSMVTFxir1pqwd46");

#[ephemeral]
#[program]
pub mod er_state_account {

    use super::*;

    pub fn initialize(ctx: Context<InitUser>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    pub fn update(ctx: Context<UpdateUser>, randomness: [u8; 32]) -> Result<()> {
        ctx.accounts.update(randomness)
    }

    pub fn update_commit(ctx: Context<UpdateCommit>, new_data: u64) -> Result<()> {
        ctx.accounts.update_commit(new_data)
    }

    pub fn delegate(ctx: Context<Delegate>) -> Result<()> {
        ctx.accounts.delegate()
    }

    pub fn undelegate(ctx: Context<Undelegate>) -> Result<()> {
        ctx.accounts.undelegate()
    }

    pub fn close(ctx: Context<CloseUser>) -> Result<()> {
        ctx.accounts.close()
    }

    pub fn request_randomness(
        ctx: Context<RequestRandomness>,
        caller_seed: [u8; 32],
    ) -> Result<()> {
        ctx.accounts.request_randomness(caller_seed)
    }
}
