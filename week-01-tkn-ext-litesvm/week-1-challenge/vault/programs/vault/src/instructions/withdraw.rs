use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{transfer_checked, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{error::VaultCode, Amount, VaultConfig};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        close = user,
        seeds = [b"amount", user.key().as_ref()],
        bump = amount_pda.bump,
    )]
    pub amount_pda: Account<'info, Amount>,

    #[account(
        seeds = [b"vault_config"], 
        bump = vault_config.bump
    )]
    pub vault_config: Account<'info, VaultConfig>,

    #[account(
        mint::decimals = 6,
        mint::token_program = token_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint, 
        associated_token::authority = vault_config
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        require!(self.amount_pda.amount >= amount,VaultCode::InsufficientBalance);

        let withdraw_cpi_program = self.token_program.to_account_info();

        let withdraw_cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.user_ata.to_account_info(),
            authority: self.vault_config.to_account_info(),
        };

        let seeds: &[&[u8]] = &[
            b"vault_config", 
            &[self.vault_config.bump]
        ];
        let signer_seeds = &[seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            withdraw_cpi_program, 
            withdraw_cpi_accounts, 
            signer_seeds
        );

        transfer_checked(cpi_ctx, amount, self.mint.decimals)?;

        self.amount_pda.amount -= amount;
        
        Ok(())
    }
}
