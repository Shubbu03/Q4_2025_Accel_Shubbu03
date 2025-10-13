use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::state::whitelist::WhitelistEntry;

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct AddToWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = admin,
        space = 8 + WhitelistEntry::INIT_SPACE,
        seeds = [b"whitelist", mint.key().as_ref(), user.as_ref()],
        bump,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct RemoveFromWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        close = admin,
        seeds = [b"whitelist", mint.key().as_ref(), user.as_ref()],
        bump = whitelist_entry.bump,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
}

impl<'info> AddToWhitelist<'info> {
    pub fn add_to_whitelist(&mut self, user: Pubkey, bumps: AddToWhitelistBumps) -> Result<()> {
        self.whitelist_entry.set_inner(WhitelistEntry {
            user,
            is_active: true,
            max_transfer_amount: 0, // 0 = unlimited
            bump: bumps.whitelist_entry,
        });
        Ok(())
    }
}

impl<'info> RemoveFromWhitelist<'info> {
    pub fn remove_from_whitelist(&mut self, _user: Pubkey) -> Result<()> {
        // Closed via `close = admin`
        Ok(())
    }
}
