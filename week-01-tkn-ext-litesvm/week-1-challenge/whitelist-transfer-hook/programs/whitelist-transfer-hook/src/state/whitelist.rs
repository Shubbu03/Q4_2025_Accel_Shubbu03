use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct WhitelistEntry {
    pub user: Pubkey,
    pub is_active: bool,
    pub max_transfer_amount: u64,
    pub bump: u8,
}
