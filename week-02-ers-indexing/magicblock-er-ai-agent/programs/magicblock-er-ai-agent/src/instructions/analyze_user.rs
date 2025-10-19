use crate::state::Agent;
use anchor_lang::prelude::*;
use solana_gpt_oracle::{
    cpi::{accounts::InteractWithLlm, interact_with_llm},
    program::SolanaGptOracle,
    ContextAccount, ID,
};

#[derive(Accounts)]
pub struct AnalyzeUser<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: the correct interaction account
    #[account(
        mut,
        seeds = [solana_gpt_oracle::Interaction::seed(), payer.key().as_ref(), context_account.key().as_ref()],
        bump
    )]
    pub interaction: AccountInfo<'info>,

    #[account(
        seeds = [b"agent"],
        bump = agent.bump
    )]
    pub agent: Account<'info, Agent>,

    /// CHECK: we accept any context
    pub context_account: Account<'info, ContextAccount>,

    /// CHECK: Checked oracle id
    #[account(
        address = ID
    )]
    pub oracle_program: Program<'info, SolanaGptOracle>,

    pub system_program: Program<'info, System>,
}

impl<'info> AnalyzeUser<'info> {
    pub fn analyse_user(&mut self, user_pubkey: Pubkey) -> Result<()> {
        let cpi_program = self.oracle_program.to_account_info();

        let cpi_accounts = InteractWithLlm {
            payer: self.payer.to_account_info(),
            interaction: self.interaction.to_account_info(),
            context_account: self.context_account.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // passing the user pubkey as string to the llm

        let callback_discriminator =
            anchor_lang::solana_program::hash::hash("global:callback_from_agent".as_bytes())
                .to_bytes()[..8]
                .try_into()
                .unwrap();
        // let callback_discriminator = instruction::CallbackFromAgent::DISCRIMINATOR
        //     .try_into()
        //     .expect("Incorrect discriminator, it should be of 8 bytes");

        interact_with_llm(
            cpi_ctx,
            user_pubkey.to_string(),
            crate::ID,
            callback_discriminator,
            None,
        )?;

        Ok(())
    }
}
