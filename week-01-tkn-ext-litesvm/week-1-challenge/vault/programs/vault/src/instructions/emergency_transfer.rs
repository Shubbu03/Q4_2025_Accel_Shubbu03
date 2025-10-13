use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::VaultConfig;

#[derive(Accounts)]
pub struct EmergencyTransfer<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"vault_config"],
        bump = vault_config.bump,
        constraint = vault_config.admin == admin.key()
    )]
    pub vault_config: Account<'info, VaultConfig>,

    #[account(
        mint::decimals = 6,
        mint::token_program = token_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = mint,
        token::token_program = token_program,
    )]
    pub from_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = mint,
        token::token_program = token_program,
    )]
    pub to_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> EmergencyTransfer<'info> {
    pub fn emergency_transfer(&mut self, amount: u64) -> Result<()> {
       
        // Admin acts as permanent delegate - can transfer without owner signature
        let transfer_cpi_program = self.token_program.to_account_info();

        let transfer_cpi_accounts = TransferChecked {
            from: self.from_account.to_account_info(),
            to: self.to_account.to_account_info(),
            mint: self.mint.to_account_info(),
            authority: self.admin.to_account_info(), // Admin is permanent delegate
        };

        let cpi_ctx = CpiContext::new(transfer_cpi_program, transfer_cpi_accounts);

        transfer_checked(cpi_ctx, amount, self.mint.decimals)?;

        Ok(())
    }
}
