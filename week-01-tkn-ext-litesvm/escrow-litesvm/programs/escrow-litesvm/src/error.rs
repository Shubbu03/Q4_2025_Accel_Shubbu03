use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Take offer accept time not elapsed yet")]
    TakeOfferTimeNotElapsed,
}
