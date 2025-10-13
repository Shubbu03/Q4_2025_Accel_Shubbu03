use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{mint_to, MintTo},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub user: SystemAccount<'info>,

    #[account(
        mut,
        mint::decimals = 6,
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> MintToken<'info> {
    pub fn mint_token(&mut self, amount: u64) -> Result<()> {
        let mint_token_cpi_program = self.token_program.to_account_info();

        let mint_token_cpi_accounts = MintTo {
            to: self.user_ata.to_account_info(),
            mint: self.mint.to_account_info(),
            authority: self.admin.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(mint_token_cpi_program, mint_token_cpi_accounts);

        mint_to(cpi_ctx, amount)?;

        Ok(())
    }
}
