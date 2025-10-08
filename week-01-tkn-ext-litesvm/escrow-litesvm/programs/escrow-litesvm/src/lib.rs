#![allow(unexpected_cfgs, deprecated)]
pub mod error;
pub mod instructions;
pub mod state;
pub mod tests;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;
pub use tests::*;

declare_id!("7Brfv9ixTj71Nvt8kbQJRj4RWw71y6cwyzSVMKFZzYr9");

#[program]
pub mod escrow_litesvm {
    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.init_escrow(seed, receive, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close_vault()
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }
}
