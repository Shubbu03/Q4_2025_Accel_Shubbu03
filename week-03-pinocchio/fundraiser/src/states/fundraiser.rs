use pinocchio::pubkey::Pubkey;

use crate::states::DataLen;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fundraiser {
    pub maker: Pubkey,
    pub mint_to_raise: Pubkey,
    pub amount_to_raise: u64,
    pub current_amount: u64,
    pub time_started: i64,
    pub duration: [u8; 1],
    pub bump: [u8; 1],
}

impl DataLen for Fundraiser {
    const LEN: usize = core::mem::size_of::<Fundraiser>();
}
