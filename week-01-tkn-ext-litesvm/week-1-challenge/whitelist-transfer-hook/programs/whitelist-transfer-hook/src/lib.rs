pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("2Bc7QG4A4sxTsEhefSRBQRVuWcgJvHA5jd4FcKZ5TDxm");

#[program]
pub mod whitelist_transfer_hook {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }
}
