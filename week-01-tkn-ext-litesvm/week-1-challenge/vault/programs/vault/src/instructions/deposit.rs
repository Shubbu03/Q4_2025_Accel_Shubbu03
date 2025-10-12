use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{transfer_checked, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{Amount, VaultConfig};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"amount", user.key().as_ref()],
        bump,
        space = 8 + Amount::INIT_SPACE
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
        associated_token::mint = mint, 
        associated_token::authority = vault_config
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64, bumps: &DepositBumps) -> Result<()> {
        self.amount_pda.set_inner(Amount {
            amount: self.amount_pda.amount + amount,
            bump: bumps.amount_pda,
        });

        let transfer_cpi_program = self.token_program.to_account_info();

        let transfer_cpi_accounts = TransferChecked {
            from: self.user_ata.to_account_info(),
            to: self.vault.to_account_info(),
            mint: self.mint.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(transfer_cpi_program, transfer_cpi_accounts);

        transfer_checked(cpi_ctx, amount, self.mint.decimals)?;
        
        Ok(())
    }
}
