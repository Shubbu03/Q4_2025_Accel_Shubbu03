use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Source account not whitelisted")]
    SourceNotWhitelisted,

    #[msg("Destination account not whitelisted")]
    DestinationNotWhitelisted,

    #[msg("Transfer amount exceeds maximum allowed")]
    ExceedsMaxTransferAmount,

    #[msg("Whitelist entry is inactive")]
    WhitelistEntryInactive,
}
