use anchor_lang::prelude::*;

#[error_code]
pub enum VaultCode {
    #[msg("Insufficient balance for withdrawal")]
    InsufficientBalance,
}
