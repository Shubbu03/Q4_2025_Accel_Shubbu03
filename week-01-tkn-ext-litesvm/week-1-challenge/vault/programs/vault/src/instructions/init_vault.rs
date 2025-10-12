use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::VaultConfig;

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + VaultConfig::INIT_SPACE,
        seeds = [b"vault_config"],
        bump
    )]
    pub vault_config: Account<'info, VaultConfig>,

    #[account(
        init,
        payer = admin,
        mint::authority = admin,
        mint::decimals = 6,
        mint::token_program = token_program,
        extensions::transfer_hook::authority = admin,
        extensions::transfer_hook::program_id = transfer_hook_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: this will be the separate program created for the whitelist transfer hook
    pub transfer_hook_program: UncheckedAccount<'info>,

    #[account(
        init, 
        payer = admin, 
        associated_token::mint = mint, 
        associated_token::authority = vault_config,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitVault<'info> {
    pub fn init_vault(&mut self, bumps: &InitVaultBumps) -> Result<()> {
        self.vault_config.set_inner(VaultConfig { 
            admin: self.admin.key(), 
            vault: self.vault.key(), 
            mint: self.mint.key(), 
            bump:  bumps.vault_config
        });
        
        Ok(())
    }
}
