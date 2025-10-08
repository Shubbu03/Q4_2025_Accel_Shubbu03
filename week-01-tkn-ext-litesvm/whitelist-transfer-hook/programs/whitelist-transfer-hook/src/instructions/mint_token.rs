use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};
// use spl_tlv_account_resolution::state::ExtraAccountMetaList;
// use spl_transfer_hook_interface::instruction::ExecuteInstruction;

// use crate::instructions::InitializeExtraAccountMetaList;

#[derive(Accounts)]
pub struct TokenFactory<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        mint::decimals = 9,
        mint::authority = user,
        extensions::transfer_hook::authority = user,
        extensions::transfer_hook::program_id = crate::ID,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: ExtraAccountMetaList Account, will be checked by the transfer hook
    // #[account(
    //     init,
    //     payer = user,
    //     space = ExtraAccountMetaList::size_of(
    //         InitializeExtraAccountMetaList::extra_account_metas()?.len()
    //     )?,
    //     seeds = [b"extra-account-metas", mint.key().as_ref()],
    //     bump
    // )]
    // pub extra_account_meta_list: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> TokenFactory<'info> {
    pub fn init_mint(&mut self, _bumps: &TokenFactoryBumps) -> Result<()> {
        // let extra_account_metas = InitializeExtraAccountMetaList::extra_account_metas()?;

        // ExtraAccountMetaList::init::<ExecuteInstruction>(
        //     &mut self.extra_account_meta_list.try_borrow_mut_data()?,
        //     &extra_account_metas,
        // )?;

        Ok(())
    }
}
