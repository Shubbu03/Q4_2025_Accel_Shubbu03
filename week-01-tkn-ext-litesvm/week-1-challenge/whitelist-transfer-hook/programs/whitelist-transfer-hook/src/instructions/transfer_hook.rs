use std::cell::RefMut;

use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022::{
        extension::{
            transfer_hook::TransferHookAccount, 
            BaseStateWithExtensionsMut, 
            PodStateWithExtensionsMut
        }, 
        pod::PodAccount
    }, 
    token_interface::{
        Mint, 
        TokenAccount
    }
};

use crate::{state::WhitelistEntry, error::ErrorCode};

#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(
        token::mint = mint, 
        token::authority = owner,
    )]
    pub source_token: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,
    
    #[account(
        token::mint = mint,
    )]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: source token account owner, can be SystemAccount or PDA owned by another program
    pub owner: UncheckedAccount<'info>,
    
    /// CHECK: ExtraAccountMetaList Account,
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()], 
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    #[account(
        seeds = [b"whitelist", mint.key().as_ref(), owner.key().as_ref()], 
        bump = source_whitelist.bump,
    )]
    pub source_whitelist: Account<'info, WhitelistEntry>,
    
    #[account(
        seeds = [b"whitelist", mint.key().as_ref(), destination_token.owner.as_ref()], 
        bump = destination_whitelist.bump,
    )]
    pub destination_whitelist: Account<'info, WhitelistEntry>,
}

impl<'info> TransferHook<'info> {
    /// This function is called when the transfer hook is executed.
    pub fn transfer_hook(&mut self, amount: u64) -> Result<()> {
        // Fail this instruction if it is not called from within a transfer hook
        self.check_is_transferring()?;

        // Validate source is whitelisted and active
        require!(
            self.source_whitelist.is_active,
            ErrorCode::SourceNotWhitelisted
        );
        
        // Validate destination is whitelisted and active
        require!(
            self.destination_whitelist.is_active,
            ErrorCode::DestinationNotWhitelisted
        );
        
        // Check amount limits for source (if set, 0 = unlimited)
        if self.source_whitelist.max_transfer_amount > 0 {
            require!(
                amount <= self.source_whitelist.max_transfer_amount,
                ErrorCode::ExceedsMaxTransferAmount
            );
        }
        
        msg!(
            "Transfer validated: {} tokens from {} to {}",
            amount,
            self.source_whitelist.user,
            self.destination_whitelist.user
        );

        Ok(())
    }

    /// Checks if the transfer hook is being executed during a transfer operation.
    fn check_is_transferring(&mut self) -> Result<()> {
        // Ensure that the source token account has the transfer hook extension enabled

        // Get the account info of the source token account
        let source_token_info = self.source_token.to_account_info();
        // Borrow the account data mutably
        let mut account_data_ref: RefMut<&mut [u8]> = source_token_info.try_borrow_mut_data()?;

        // Unpack the account data as a PodStateWithExtensionsMut
        // This will allow us to access the extensions of the token account
        // We use PodStateWithExtensionsMut because TokenAccount is a POD (Plain Old Data) type
        let mut account = PodStateWithExtensionsMut::<PodAccount>::unpack(*account_data_ref)?;
        // Get the TransferHookAccount extension
        // Search for the TransferHookAccount extension in the token account
        // The returning struct has a `transferring` field that indicates if the account is in the middle of a transfer operation
        let account_extension = account.get_extension_mut::<TransferHookAccount>()?;
    
        // Check if the account is in the middle of a transfer operation
        if !bool::from(account_extension.transferring) {
            panic!("TransferHook: Not transferring");
        }
    
        Ok(())
    }
}