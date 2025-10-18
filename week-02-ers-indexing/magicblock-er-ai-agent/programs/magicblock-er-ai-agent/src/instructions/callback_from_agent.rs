use crate::{error::ErrorCode, state::AnalysisResult};
use anchor_lang::prelude::*;
use solana_gpt_oracle::Identity;

#[derive(Accounts)]
pub struct CallbackFromAgent<'info> {
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + AnalysisResult::INIT_SPACE,
        seeds = [b"analysis", user_pubkey.key().as_ref()],
        bump
    )]
    pub analysis_result: Account<'info, AnalysisResult>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: This is the user pubkey we analyzed
    pub user_pubkey: AccountInfo<'info>,

    /// CHECK: This is the oracle program identity
    pub identity: Account<'info, Identity>,

    /// CHECK: This is the oracle program that will call this callback
    #[account(
        address = solana_gpt_oracle::ID
    )]
    pub oracle_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> CallbackFromAgent<'info> {
    pub fn callback_from_agent(
        &mut self,
        response: String,
        bumps: &CallbackFromAgentBumps,
    ) -> Result<()> {
        // verify callback is from oracle program
        if !self.identity.to_account_info().is_signer {
            return Err(ErrorCode::InvalidOracleCallback.into());
        }

        // storing the analysis result

        self.analysis_result.set_inner(AnalysisResult {
            user: self.user_pubkey.key(),
            analysis: response,
            timestamp: Clock::get()?.unix_timestamp,
            bump: bumps.analysis_result,
        });

        Ok(())
    }
}
